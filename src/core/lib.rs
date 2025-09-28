//! # Vortex Core
//!
//! The foundational library for the Vortex ephemeral VM platform.
//! Provides abstractions for VM lifecycle management, networking, storage,
//! and extensibility for specialized use cases.

pub mod auth;
pub mod backend;
pub mod config;
pub mod error;
pub mod metrics;
pub mod network;
pub mod plugin;
pub mod storage;
pub mod templates;
pub mod vm;
pub mod workspace;

// Re-export core types
pub use auth::{AuthProvider, Permission};
pub use backend::{Backend, BackendProvider};
pub use config::{Template, VortexConfig};
pub use error::{Result, VortexError};
pub use metrics::{MetricsCollector, SystemMetrics, VmMetrics};
pub use network::{NetworkConfig, NetworkManager};
pub use plugin::{Plugin, PluginManager};
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
    pub vm_manager: VmManager,
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
        Ok(Self {
            vm_manager: VmManager::new().await?,
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
