use crate::error::{Result, VortexError};
use crate::session::{SessionCommand, SessionManager, SessionResponse};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error};

pub struct VortexDaemon {
    session_manager: Arc<SessionManager>,
    socket_path: PathBuf,
    running: Arc<RwLock<bool>>,
}

impl VortexDaemon {
    pub async fn new(session_manager: SessionManager) -> Result<Self> {
        let socket_path = Self::get_socket_path()?;
        
        // Clean up any existing socket
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await.map_err(|e| VortexError::VmError {
                message: format!("Failed to remove existing socket: {}", e),
            })?;
        }
        
        Ok(Self {
            session_manager: Arc::new(session_manager),
            socket_path,
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    fn get_socket_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| VortexError::VmError {
            message: "HOME environment variable not set".to_string(),
        })?;
        
        let vortex_dir = PathBuf::from(home).join(".vortex");
        std::fs::create_dir_all(&vortex_dir).map_err(|e| VortexError::VmError {
            message: format!("Failed to create vortex directory: {}", e),
        })?;
        
        Ok(vortex_dir.join("daemon.sock"))
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Starting Vortex daemon on socket: {:?}", self.socket_path);
        
        let listener = UnixListener::bind(&self.socket_path).map_err(|e| VortexError::VmError {
            message: format!("Failed to bind to socket: {}", e),
        })?;
        
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
        
        info!("Vortex daemon started successfully");
        
        // Main connection handling loop
        while *self.running.read().await {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let session_manager = self.session_manager.clone();
                    let running = self.running.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, session_manager, running).await {
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
            tokio::fs::remove_file(&self.socket_path).await.map_err(|e| VortexError::VmError {
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
    ) -> Result<()> {
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
                    
                    let response = match serde_json::from_str::<SessionCommand>(line) {
                        Ok(command) => {
                            // Handle shutdown command specially
                            if matches!(command, SessionCommand::Shutdown) {
                                let mut running_guard = running.write().await;
                                *running_guard = false;
                                SessionResponse::Success
                            } else {
                                session_manager.handle_command(command).await.unwrap_or_else(|e| {
                                    SessionResponse::Error { message: e.to_string() }
                                })
                            }
                        }
                        Err(e) => SessionResponse::Error {
                            message: format!("Invalid command: {}", e),
                        },
                    };
                    
                    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| {
                        serde_json::to_string(&SessionResponse::Error {
                            message: "Failed to serialize response".to_string(),
                        }).unwrap()
                    });
                    
                    if let Err(e) = writer.write_all(format!("{}\n", response_json).as_bytes()).await {
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
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| VortexError::VmError {
            message: format!("Failed to connect to daemon: {}", e),
        })?;
        
        let command_json = serde_json::to_string(&command).map_err(|e| VortexError::VmError {
            message: format!("Failed to serialize command: {}", e),
        })?;
        
        stream.write_all(format!("{}\n", command_json).as_bytes()).await.map_err(|e| VortexError::VmError {
            message: format!("Failed to send command: {}", e),
        })?;
        
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.map_err(|e| VortexError::VmError {
            message: format!("Failed to read response: {}", e),
        })?;
        
        serde_json::from_str(&response_line.trim()).map_err(|e| VortexError::VmError {
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

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new().expect("Failed to create daemon client")
    }
}