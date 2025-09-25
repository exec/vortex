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
pub mod vm;

// Re-export core types
pub use auth::{AuthProvider, Permission};
pub use backend::{Backend, BackendProvider};
pub use config::{Template, VortexConfig};
pub use error::{Result, VortexError};
pub use metrics::{MetricsCollector, SystemMetrics, VmMetrics};
pub use network::{NetworkConfig, NetworkManager};
pub use plugin::{Plugin, PluginManager};
pub use storage::{StorageManager, Volume};
pub use vm::{ResourceLimits, VmEvent, VmInstance, VmManager, VmSpec, VmState};

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
}
