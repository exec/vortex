use crate::error::{Result, VortexError};
use crate::vm::{VmManager, VmSpec};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmSession {
    pub id: String,
    pub name: Option<String>,
    pub vm_id: String,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub last_attached: Option<DateTime<Utc>>,
    pub persistent: bool,
    pub spec: VmSpec,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Creating,
    Running,
    Detached,
    Attached { client_pid: u32 },
    Paused,
    Stopped,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionCommand {
    // Session management
    CreateSession {
        spec: VmSpec,
        name: Option<String>,
        persistent: bool,
    },
    ListSessions,
    GetSession {
        session_id: String,
    },
    DeleteSession {
        session_id: String,
    },

    // VM lifecycle
    StartSession {
        session_id: String,
    },
    StopSession {
        session_id: String,
    },
    PauseSession {
        session_id: String,
    },
    ResumeSession {
        session_id: String,
    },
    RestartSession {
        session_id: String,
    },

    // Interactive
    AttachSession {
        session_id: String,
        client_pid: u32,
    },
    DetachSession {
        session_id: String,
    },

    // Daemon control
    Ping,
    Shutdown,
    GetDaemonStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionResponse {
    Success,
    Error {
        message: String,
    },
    SessionCreated {
        session: VmSession,
    },
    SessionList {
        sessions: Vec<VmSession>,
    },
    Session {
        session: VmSession,
    },
    DaemonStatus {
        uptime: u64,
        sessions_count: usize,
        active_vms: usize,
        memory_usage: u64,
    },
}

pub struct SessionManager {
    sessions: RwLock<HashMap<String, VmSession>>,
    vm_manager: Arc<VmManager>,
    session_file: PathBuf,
    daemon_start_time: DateTime<Utc>,
}

impl SessionManager {
    pub async fn new(vm_manager: Arc<VmManager>) -> Result<Self> {
        let session_file = Self::get_session_file()?;

        let mut manager = Self {
            sessions: RwLock::new(HashMap::new()),
            vm_manager,
            session_file,
            daemon_start_time: Utc::now(),
        };

        // Load existing sessions
        manager.load_sessions().await?;

        // Reconcile with actual VMs
        manager.reconcile_sessions().await?;

        Ok(manager)
    }

    fn get_session_file() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| VortexError::VmError {
            message: "HOME environment variable not set".to_string(),
        })?;

        let vortex_dir = PathBuf::from(home).join(".vortex");
        std::fs::create_dir_all(&vortex_dir).map_err(|e| VortexError::VmError {
            message: format!("Failed to create vortex directory: {}", e),
        })?;

        Ok(vortex_dir.join("sessions.json"))
    }

    async fn load_sessions(&mut self) -> Result<()> {
        if !self.session_file.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.session_file)
            .await
            .map_err(|e| VortexError::VmError {
                message: format!("Failed to read sessions file: {}", e),
            })?;

        let sessions: HashMap<String, VmSession> =
            serde_json::from_str(&content).unwrap_or_default();

        let mut session_map = self.sessions.write().await;
        *session_map = sessions;

        tracing::info!("Loaded {} sessions from disk", session_map.len());
        Ok(())
    }

    async fn save_sessions(&self) -> Result<()> {
        let sessions = self.sessions.read().await;
        let content =
            serde_json::to_string_pretty(&*sessions).map_err(|e| VortexError::VmError {
                message: format!("Failed to serialize sessions: {}", e),
            })?;

        tokio::fs::write(&self.session_file, content)
            .await
            .map_err(|e| VortexError::VmError {
                message: format!("Failed to write sessions file: {}", e),
            })?;

        Ok(())
    }

    async fn reconcile_sessions(&self) -> Result<()> {
        let running_vms = self.vm_manager.list().await?;
        let running_vm_ids: std::collections::HashSet<String> =
            running_vms.iter().map(|vm| vm.id.clone()).collect();

        let mut sessions = self.sessions.write().await;

        for session in sessions.values_mut() {
            match session.state {
                SessionState::Running | SessionState::Attached { .. } => {
                    if !running_vm_ids.contains(&session.vm_id) {
                        tracing::warn!(
                            "Session {} VM {} not found, marking as stopped",
                            session.id,
                            session.vm_id
                        );
                        session.state = SessionState::Stopped;
                    }
                }
                SessionState::Stopped | SessionState::Error { .. } => {
                    if running_vm_ids.contains(&session.vm_id) {
                        tracing::info!(
                            "Session {} VM {} found running, updating state",
                            session.id,
                            session.vm_id
                        );
                        session.state = SessionState::Detached;
                    }
                }
                _ => {}
            }
        }

        self.save_sessions().await?;
        Ok(())
    }

    pub async fn create_session(
        &self,
        spec: VmSpec,
        name: Option<String>,
        persistent: bool,
    ) -> Result<VmSession> {
        let uuid_str = Uuid::new_v4().simple().to_string();
        let session_id = format!("session-{}", &uuid_str[..8]);
        let vm_id = format!("vortex-{}", &session_id);

        // Create the VM
        let mut vm_spec = spec.clone();
        vm_spec
            .labels
            .insert("session_id".to_string(), session_id.clone());
        vm_spec
            .labels
            .insert("persistent".to_string(), persistent.to_string());
        if let Some(ref name) = name {
            vm_spec
                .labels
                .insert("session_name".to_string(), name.clone());
        }

        let session = VmSession {
            id: session_id.clone(),
            name: name.clone(),
            vm_id: vm_id.clone(),
            state: SessionState::Creating,
            created_at: Utc::now(),
            last_attached: None,
            persistent,
            spec: vm_spec.clone(),
            metadata: HashMap::new(),
        };

        // Store session first
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }
        self.save_sessions().await?;

        // Create VM instance
        match self.vm_manager.create(vm_spec).await {
            Ok(vm_instance) => {
                let mut updated_session = session;
                updated_session.vm_id = vm_instance.id;
                updated_session.state = SessionState::Detached;

                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.clone(), updated_session.clone());
                }
                self.save_sessions().await?;

                tracing::info!(
                    "Created session {} with VM {}",
                    session_id,
                    updated_session.vm_id
                );
                Ok(updated_session)
            }
            Err(e) => {
                let mut failed_session = session;
                failed_session.state = SessionState::Error {
                    message: e.to_string(),
                };

                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.clone(), failed_session.clone());
                }
                self.save_sessions().await?;

                Err(e)
            }
        }
    }

    pub async fn list_sessions(&self) -> Result<Vec<VmSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<VmSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            // Stop and cleanup VM if it exists
            if let Err(e) = self.vm_manager.cleanup(&session.vm_id).await {
                tracing::warn!(
                    "Failed to cleanup VM {} for session {}: {}",
                    session.vm_id,
                    session_id,
                    e
                );
            }

            self.save_sessions().await?;
            tracing::info!("Deleted session {}", session_id);
            Ok(())
        } else {
            Err(VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })
        }
    }

    pub async fn start_session(&self, session_id: &str) -> Result<()> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })?;

        match session.state {
            SessionState::Stopped | SessionState::Error { .. } => {
                // Recreate the VM
                let vm_instance = self.vm_manager.create(session.spec.clone()).await?;

                let mut updated_session = session;
                updated_session.vm_id = vm_instance.id;
                updated_session.state = SessionState::Detached;

                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.to_string(), updated_session);
                }
                self.save_sessions().await?;

                tracing::info!("Started session {}", session_id);
                Ok(())
            }
            SessionState::Running | SessionState::Detached | SessionState::Attached { .. } => {
                tracing::info!("Session {} already running", session_id);
                Ok(())
            }
            _ => Err(VortexError::VmError {
                message: format!(
                    "Cannot start session {} in state {:?}",
                    session_id, session.state
                ),
            }),
        }
    }

    pub async fn stop_session(&self, session_id: &str) -> Result<()> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })?;

        if let Err(e) = self.vm_manager.stop(&session.vm_id).await {
            tracing::warn!(
                "Failed to stop VM {} for session {}: {}",
                session.vm_id,
                session_id,
                e
            );
        }

        let mut updated_session = session;
        updated_session.state = SessionState::Stopped;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.to_string(), updated_session);
        }
        self.save_sessions().await?;

        tracing::info!("Stopped session {}", session_id);
        Ok(())
    }

    pub async fn pause_session(&self, session_id: &str) -> Result<()> {
        // Note: krunvm doesn't have native pause, but we can track the state
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })?;

        let mut updated_session = session;
        updated_session.state = SessionState::Paused;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.to_string(), updated_session);
        }
        self.save_sessions().await?;

        tracing::info!("Paused session {}", session_id);
        Ok(())
    }

    pub async fn resume_session(&self, session_id: &str) -> Result<()> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })?;

        let mut updated_session = session;
        updated_session.state = SessionState::Detached;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.to_string(), updated_session);
        }
        self.save_sessions().await?;

        tracing::info!("Resumed session {}", session_id);
        Ok(())
    }

    pub async fn restart_session(&self, session_id: &str) -> Result<()> {
        self.stop_session(session_id).await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        self.start_session(session_id).await?;
        Ok(())
    }

    pub async fn attach_session(&self, session_id: &str, client_pid: u32) -> Result<()> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })?;

        match session.state {
            SessionState::Detached | SessionState::Running => {
                // Update session state
                let mut updated_session = session.clone();
                updated_session.state = SessionState::Attached { client_pid };
                updated_session.last_attached = Some(Utc::now());

                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.to_string(), updated_session);
                }
                self.save_sessions().await?;

                // Attach to VM
                self.vm_manager.attach(&session.vm_id).await?;

                // When attach returns (user detached), update state
                let mut detached_session = session;
                detached_session.state = SessionState::Detached;

                {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(session_id.to_string(), detached_session);
                }
                self.save_sessions().await?;

                Ok(())
            }
            SessionState::Attached { .. } => Err(VortexError::VmError {
                message: format!("Session {} is already attached", session_id),
            }),
            _ => Err(VortexError::VmError {
                message: format!(
                    "Cannot attach to session {} in state {:?}",
                    session_id, session.state
                ),
            }),
        }
    }

    pub async fn detach_session(&self, session_id: &str) -> Result<()> {
        let session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| VortexError::VmError {
                message: format!("Session {} not found", session_id),
            })?;

        let mut updated_session = session;
        updated_session.state = SessionState::Detached;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.to_string(), updated_session);
        }
        self.save_sessions().await?;

        tracing::info!("Detached from session {}", session_id);
        Ok(())
    }

    pub async fn get_daemon_status(&self) -> Result<SessionResponse> {
        let sessions = self.sessions.read().await;
        let uptime = (Utc::now() - self.daemon_start_time).num_seconds() as u64;
        let sessions_count = sessions.len();

        let active_vms = sessions
            .values()
            .filter(|s| {
                matches!(
                    s.state,
                    SessionState::Running | SessionState::Attached { .. } | SessionState::Detached
                )
            })
            .count();

        // Get memory usage (rough estimate)
        let memory_usage = (sessions_count * 512 * 1024 * 1024) as u64; // Estimate 512MB per session

        Ok(SessionResponse::DaemonStatus {
            uptime,
            sessions_count,
            active_vms,
            memory_usage,
        })
    }

    pub async fn handle_command(&self, command: SessionCommand) -> Result<SessionResponse> {
        match command {
            SessionCommand::CreateSession {
                spec,
                name,
                persistent,
            } => match self.create_session(spec, name, persistent).await {
                Ok(session) => Ok(SessionResponse::SessionCreated { session }),
                Err(e) => Ok(SessionResponse::Error {
                    message: e.to_string(),
                }),
            },
            SessionCommand::ListSessions => match self.list_sessions().await {
                Ok(sessions) => Ok(SessionResponse::SessionList { sessions }),
                Err(e) => Ok(SessionResponse::Error {
                    message: e.to_string(),
                }),
            },
            SessionCommand::GetSession { session_id } => {
                match self.get_session(&session_id).await {
                    Ok(Some(session)) => Ok(SessionResponse::Session { session }),
                    Ok(None) => Ok(SessionResponse::Error {
                        message: format!("Session {} not found", session_id),
                    }),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::DeleteSession { session_id } => {
                match self.delete_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::StartSession { session_id } => {
                match self.start_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::StopSession { session_id } => {
                match self.stop_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::PauseSession { session_id } => {
                match self.pause_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::ResumeSession { session_id } => {
                match self.resume_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::RestartSession { session_id } => {
                match self.restart_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::AttachSession {
                session_id,
                client_pid,
            } => match self.attach_session(&session_id, client_pid).await {
                Ok(()) => Ok(SessionResponse::Success),
                Err(e) => Ok(SessionResponse::Error {
                    message: e.to_string(),
                }),
            },
            SessionCommand::DetachSession { session_id } => {
                match self.detach_session(&session_id).await {
                    Ok(()) => Ok(SessionResponse::Success),
                    Err(e) => Ok(SessionResponse::Error {
                        message: e.to_string(),
                    }),
                }
            }
            SessionCommand::Ping => Ok(SessionResponse::Success),
            SessionCommand::Shutdown => Ok(SessionResponse::Success),
            SessionCommand::GetDaemonStatus => self.get_daemon_status().await,
        }
    }

    pub async fn cleanup_stale_sessions(&self) -> Result<()> {
        let mut sessions_to_remove = Vec::new();

        {
            let sessions = self.sessions.read().await;
            for (session_id, session) in sessions.iter() {
                if !session.persistent {
                    // Check if session has been detached for more than 24 hours
                    if let Some(last_attached) = session.last_attached {
                        let hours_since_attach = (Utc::now() - last_attached).num_hours();
                        if hours_since_attach > 24
                            && matches!(session.state, SessionState::Detached)
                        {
                            sessions_to_remove.push(session_id.clone());
                        }
                    }
                }
            }
        }

        for session_id in sessions_to_remove {
            tracing::info!("Cleaning up stale session: {}", session_id);
            if let Err(e) = self.delete_session(&session_id).await {
                tracing::warn!("Failed to cleanup stale session {}: {}", session_id, e);
            }
        }

        Ok(())
    }
}
