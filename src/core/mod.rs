//! # Vortex Core
//!
//! The foundational library for the Vortex ephemeral VM platform.
//! Provides abstractions for VM lifecycle management, networking, storage,
//! and extensibility for specialized use cases.

pub mod auth;
pub mod backend;
pub mod config;
pub mod daemon;
pub mod error;
pub mod metrics;
pub mod network;
pub mod plugin;
pub mod session;
pub mod storage;
pub mod templates;
pub mod vm;
pub mod workspace;

// Re-export core types
pub use auth::{AuthProvider, Permission};
pub use backend::{Backend, BackendProvider};
pub use config::{Template, VortexConfig};
pub use daemon::{DaemonClient, VortexDaemon};
pub use error::{Result, VortexError};
pub use metrics::{MetricsCollector, SystemMetrics, VmMetrics};
pub use network::{NetworkConfig, NetworkManager};
pub use plugin::{Plugin, PluginManager};
pub use session::{SessionCommand, SessionManager, SessionResponse, SessionState, VmSession};
pub use storage::{StorageManager, Volume};
pub use templates::{DevEnvironmentManager, DevTemplate};
pub use vm::{ResourceLimits, VmEvent, VmInstance, VmManager, VmSpec, VmState};
pub use workspace::{detect_workspace_info, Workspace, WorkspaceInfo, WorkspaceManager};

/// Vortex platform version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the Vortex core library
pub async fn init() -> Result<VortexCore> {
    VortexCore::new().await
}

/// Main Vortex core orchestrator
pub struct VortexCore {
    pub vm_manager: std::sync::Arc<VmManager>,
    pub session_manager: SessionManager,
    pub network_manager: NetworkManager,
    pub storage_manager: StorageManager,
    pub metrics_collector: MetricsCollector,
    pub auth_provider: Box<dyn AuthProvider>,
    pub plugin_manager: PluginManager,
    pub dev_env_manager: DevEnvironmentManager,
    pub workspace_manager: WorkspaceManager,
}

impl VortexCore {
    pub async fn new() -> Result<Self> {
        let vm_manager = std::sync::Arc::new(VmManager::new().await?);
        let session_manager = SessionManager::new(vm_manager.clone()).await?;

        Ok(Self {
            vm_manager,
            session_manager,
            network_manager: NetworkManager::new().await?,
            storage_manager: StorageManager::new().await?,
            metrics_collector: MetricsCollector::new().await?,
            auth_provider: Box::new(auth::NoOpAuthProvider),
            plugin_manager: PluginManager::new().await?,
            dev_env_manager: DevEnvironmentManager::new(),
            workspace_manager: WorkspaceManager::new()?,
        })
    }

    /// Create a new VM with full lifecycle management
    pub async fn create_vm(&self, spec: VmSpec) -> Result<VmInstance> {
        self.vm_manager.create(spec).await
    }

    /// Attach to an interactive VM session
    pub async fn attach_vm(&self, vm_id: &str) -> Result<()> {
        self.vm_manager.attach(vm_id).await
    }

    /// Create a new session with optional persistence and boot-start
    pub async fn create_session(
        &self,
        spec: VmSpec,
        name: Option<String>,
        persistent: bool,
        boot_start: bool,
    ) -> Result<VmSession> {
        self.session_manager
            .create_session(spec, name, persistent, boot_start)
            .await
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<VmSession>> {
        self.session_manager.list_sessions().await
    }

    /// Attach to a session by ID
    pub async fn attach_session(&self, session_id: &str) -> Result<()> {
        let client_pid = std::process::id();
        self.session_manager
            .attach_session(session_id, client_pid)
            .await
    }

    /// Stop a session
    pub async fn stop_session(&self, session_id: &str) -> Result<()> {
        self.session_manager.stop_session(session_id).await
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        self.session_manager.delete_session(session_id).await
    }

    /// Create a development environment VM from a template
    pub async fn create_dev_environment(
        &self,
        template_name: &str,
        workdir: Option<String>,
        volumes: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
    ) -> Result<VmInstance> {
        let mut spec = self
            .dev_env_manager
            .template_to_vm_spec(template_name, workdir)?;

        // Add any additional volumes
        for (host, guest) in volumes {
            spec.volumes.insert(host, guest);
        }

        self.vm_manager.create(spec).await
    }

    /// Create a VM from a workspace
    pub async fn create_workspace_vm(&self, workspace_id: &str) -> Result<VmInstance> {
        let workspace = self
            .workspace_manager
            .get_workspace(workspace_id)?
            .ok_or_else(|| VortexError::InvalidInput {
                field: "workspace_id".to_string(),
                message: format!("Workspace '{}' not found", workspace_id),
            })?;

        let template = self
            .dev_env_manager
            .get_template(&workspace.config.template)
            .ok_or_else(|| VortexError::TemplateNotFound {
                name: workspace.config.template.clone(),
            })?;

        let spec = self
            .workspace_manager
            .workspace_to_vm_spec(&workspace, template)?;

        // Update workspace last used time
        self.workspace_manager.touch_workspace(workspace_id)?;

        self.vm_manager.create(spec).await
    }
}
