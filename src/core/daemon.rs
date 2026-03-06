use crate::error::{Result, VortexError};
use crate::session::{SessionCommand, SessionManager, SessionResponse};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

// Rate limiting configuration
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB limit
const MAX_REQUESTS_PER_SECOND: u32 = 50;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct RateLimitState {
    count: u32,
    window_start: u64, // Unix timestamp in seconds
}

pub struct VortexDaemon {
    session_manager: Arc<SessionManager>,
    socket_path: PathBuf,
    running: Arc<RwLock<bool>>,
    rate_limiter: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl VortexDaemon {
    pub async fn new(session_manager: SessionManager) -> Result<Self> {
        let socket_path = Self::get_socket_path()?;

        // Clean up any existing socket
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path)
                .await
                .map_err(|e| VortexError::VmError {
                    message: format!("Failed to remove existing socket: {}", e),
                })?;
        }

        Ok(Self {
            session_manager: Arc::new(session_manager),
            socket_path,
            running: Arc::new(RwLock::new(false)),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn get_socket_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| VortexError::VmError {
            message: "Could not determine home directory".to_string(),
        })?;

        let vortex_dir = home.join(".vortex");
        std::fs::create_dir_all(&vortex_dir).map_err(|e| VortexError::VmError {
            message: format!("Failed to create vortex directory: {}", e),
        })?;

        Ok(vortex_dir.join("daemon.sock"))
    }

    /// Set secure permissions (0o600 - owner read/write only) on the socket file
    /// This function exists for documentation purposes; actual permission setting
    /// is done inline in the start() method after binding.
    #[allow(dead_code)]
    fn set_socket_permissions(_path: &PathBuf) -> Result<()> {
        // On Unix, socket permissions are set after bind() using chmod
        // On non-Unix systems, socket permissions work differently
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Vortex daemon on socket: {:?}", self.socket_path);

        // Start boot-start sessions
        let session_manager = self.session_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = session_manager.start_boot_start_sessions().await {
                warn!("Failed to start boot-start sessions: {}", e);
            }
        });

        let listener = UnixListener::bind(&self.socket_path).map_err(|e| VortexError::VmError {
            message: format!("Failed to bind to socket: {}", e),
        })?;

        // Set secure permissions on the socket (owner read/write only)
        // This prevents other users from connecting to the daemon
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| VortexError::VmError {
                    message: format!("Failed to set socket permissions: {}", e),
                })?;
        }

        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start cleanup task
        let session_manager = self.session_manager.clone();
        let running_cleanup = self.running.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(3600)); // Every hour
            loop {
                cleanup_interval.tick().await;

                if !*running_cleanup.read().await {
                    break;
                }

                if let Err(e) = session_manager.cleanup_stale_sessions().await {
                    warn!("Failed to cleanup stale sessions: {}", e);
                }
            }
        });

        info!("Vortex daemon started successfully (socket permissions: 0600)");

        // Main connection handling loop
        while *self.running.read().await {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let session_manager = self.session_manager.clone();
                    let running = self.running.clone();
                    let rate_limiter = self.rate_limiter.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            Self::handle_connection(stream, session_manager, running, rate_limiter).await
                        {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    break;
                }
            }
        }

        // Cleanup
        if self.socket_path.exists() {
            tokio::fs::remove_file(&self.socket_path)
                .await
                .map_err(|e| VortexError::VmError {
                    message: format!("Failed to remove socket: {}", e),
                })?;
        }

        info!("Vortex daemon stopped");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Vortex daemon");
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    async fn handle_connection(
        mut stream: UnixStream,
        session_manager: Arc<SessionManager>,
        running: Arc<RwLock<bool>>,
        rate_limiter: Arc<RwLock<HashMap<String, RateLimitState>>>,
    ) -> Result<()> {
        // Get client identifier before splitting (to avoid borrow issues)
        let client_id = format!("{:?}", stream.peer_addr().ok());

        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Check message size limit
                    if line.len() > MAX_MESSAGE_SIZE {
                        let response = SessionResponse::Error {
                            message: format!("Message too large. Maximum size is {} bytes.", MAX_MESSAGE_SIZE),
                        };
                        if let Err(e) = writer.write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes()).await {
                            error!("Failed to write error response: {}", e);
                            break;
                        }
                        continue;
                    }

                    // Rate limiting (client_id already captured before stream.split())
                    let rate_limited = {
                        let mut limiter = rate_limiter.write().await;
                        !Self::check_rate_limit(&mut limiter, &client_id)
                    };

                    if rate_limited {
                        warn!("Rate limit exceeded for client");
                        let response = SessionResponse::Error {
                            message: "Rate limit exceeded. Please slow down.".to_string(),
                        };
                        if let Err(e) = writer.write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes()).await {
                            error!("Failed to write rate limit response: {}", e);
                            break;
                        }
                        continue;
                    }

                    let response = match serde_json::from_str::<SessionCommand>(line) {
                        Ok(command) => {
                            // Handle shutdown command specially
                            if matches!(command, SessionCommand::Shutdown) {
                                let mut running_guard = running.write().await;
                                *running_guard = false;
                                SessionResponse::Success
                            } else {
                                session_manager
                                    .handle_command(command)
                                    .await
                                    .unwrap_or_else(|e| SessionResponse::Error {
                                        message: e.to_string(),
                                    })
                            }
                        }
                        Err(e) => SessionResponse::Error {
                            message: format!("Invalid command: {}", e),
                        },
                    };

                    let response_json = match serde_json::to_string(&response) {
                        Ok(json) => json,
                        Err(_) => {
                            error!("Failed to serialize response");
                            serde_json::to_string(&SessionResponse::Error {
                                message: "Internal server error".to_string(),
                            })
                            .expect("Failed to serialize error response - this should never fail")
                        }
                    };

                    if let Err(e) = writer
                        .write_all(format!("{}\n", response_json).as_bytes())
                        .await
                    {
                        error!("Failed to write response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading from stream: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check if rate limit is exceeded for a client
    fn check_rate_limit(rate_limiter: &mut HashMap<String, RateLimitState>, client_id: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let state = rate_limiter.entry(client_id.to_string()).or_insert(RateLimitState {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now > state.window_start + RATE_LIMIT_WINDOW.as_secs() {
            state.count = 0;
            state.window_start = now;
        }

        // Check limit
        if state.count >= MAX_REQUESTS_PER_SECOND {
            return false; // Rate limited
        }

        state.count += 1;
        true // Allowed
    }
}

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    pub fn new() -> Result<Self> {
        let socket_path = VortexDaemon::get_socket_path()?;
        Ok(Self { socket_path })
    }

    pub async fn is_running(&self) -> bool {
        self.send_command(SessionCommand::Ping).await.is_ok()
    }

    pub async fn send_command(&self, command: SessionCommand) -> Result<SessionResponse> {
        let mut stream =
            UnixStream::connect(&self.socket_path)
                .await
                .map_err(|e| VortexError::VmError {
                    message: format!("Failed to connect to daemon: {}", e),
                })?;

        let command_json = serde_json::to_string(&command).map_err(|e| VortexError::VmError {
            message: format!("Failed to serialize command: {}", e),
        })?;

        stream
            .write_all(format!("{}\n", command_json).as_bytes())
            .await
            .map_err(|e| VortexError::VmError {
                message: format!("Failed to send command: {}", e),
            })?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| VortexError::VmError {
                message: format!("Failed to read response: {}", e),
            })?;

        serde_json::from_str(response_line.trim()).map_err(|e| VortexError::VmError {
            message: format!("Failed to parse response: {}", e),
        })
    }

    pub async fn start_daemon_if_needed() -> Result<()> {
        let client = Self::new()?;

        if client.is_running().await {
            info!("Daemon is already running");
            return Ok(());
        }

        info!("Starting daemon in background");

        // Fork a background process to run the daemon
        let current_exe = std::env::current_exe().map_err(|e| VortexError::VmError {
            message: format!("Failed to get current executable: {}", e),
        })?;

        std::process::Command::new(current_exe)
            .arg("daemon")
            .arg("start")
            .arg("--background")
            .spawn()
            .map_err(|e| VortexError::VmError {
                message: format!("Failed to start daemon: {}", e),
            })?;

        // Wait for daemon to start
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if client.is_running().await {
                info!("Daemon started successfully");
                return Ok(());
            }
        }

        Err(VortexError::VmError {
            message: "Daemon failed to start within timeout".to_string(),
        })
    }
}
