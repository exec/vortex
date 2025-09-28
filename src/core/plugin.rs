use crate::error::Result;
use crate::vm::{VmEvent, VmEventHandler, VmInstance};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub hooks: Vec<PluginHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginHook {
    VmPreCreate,
    VmPostCreate,
    VmPreStart,
    VmPostStart,
    VmPreStop,
    VmPostStop,
    VmPreDelete,
    VmPostDelete,
    SnapshotCreate,
    SnapshotRestore,
}

#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    fn metadata(&self) -> &PluginMetadata;

    async fn initialize(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;

    // Hook implementations
    async fn on_vm_pre_create(&self, _spec: &mut crate::vm::VmSpec) -> Result<()> {
        Ok(())
    }

    async fn on_vm_post_create(&self, _vm: &VmInstance) -> Result<()> {
        Ok(())
    }

    async fn on_vm_pre_start(&self, _vm: &VmInstance) -> Result<()> {
        Ok(())
    }

    async fn on_vm_post_start(&self, _vm: &VmInstance) -> Result<()> {
        Ok(())
    }

    async fn on_vm_pre_stop(&self, _vm: &VmInstance) -> Result<()> {
        Ok(())
    }

    async fn on_vm_post_stop(&self, _vm: &VmInstance) -> Result<()> {
        Ok(())
    }

    async fn on_snapshot_create(&self, _vm: &VmInstance, _snapshot_name: &str) -> Result<()> {
        Ok(())
    }

    async fn on_snapshot_restore(&self, _snapshot_id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            plugins: HashMap::new(),
        })
    }

    pub async fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        tracing::info!(
            "Registering plugin: {} v{}",
            metadata.name,
            metadata.version
        );

        // Initialize the plugin
        // Note: We can't mutate plugin here due to trait object limitations
        // In a real implementation, we'd need a different approach

        self.plugins.insert(metadata.name.clone(), plugin);
        Ok(())
    }

    pub async fn call_hook(&self, hook: PluginHook, context: PluginContext) -> Result<()> {
        for (name, plugin) in &self.plugins {
            if plugin.metadata().hooks.contains(&hook) {
                match (&hook, &context) {
                    (PluginHook::VmPostCreate, PluginContext::VmInstance(vm)) => {
                        if let Err(e) = plugin.on_vm_post_create(vm).await {
                            tracing::warn!("Plugin {} failed on VmPostCreate: {}", name, e);
                        }
                    }
                    (PluginHook::VmPostStart, PluginContext::VmInstance(vm)) => {
                        if let Err(e) = plugin.on_vm_post_start(vm).await {
                            tracing::warn!("Plugin {} failed on VmPostStart: {}", name, e);
                        }
                    }
                    (PluginHook::VmPostStop, PluginContext::VmInstance(vm)) => {
                        if let Err(e) = plugin.on_vm_post_stop(vm).await {
                            tracing::warn!("Plugin {} failed on VmPostStop: {}", name, e);
                        }
                    }
                    _ => {
                        // Handle other hooks
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    pub async fn shutdown_all(&mut self) -> Result<()> {
        for name in self.plugins.keys() {
            tracing::info!("Shutting down plugin: {}", name);
            // plugin.shutdown().await?; // Can't mutate due to trait object limitations
        }

        self.plugins.clear();
        Ok(())
    }
}

pub enum PluginContext {
    VmInstance(VmInstance),
    VmSpec(crate::vm::VmSpec),
    SnapshotId(String),
}

// Plugin event handler to bridge VM events to plugin hooks
pub struct PluginEventHandler {
    plugin_manager: std::sync::Arc<tokio::sync::RwLock<PluginManager>>,
}

impl PluginEventHandler {
    pub fn new(plugin_manager: std::sync::Arc<tokio::sync::RwLock<PluginManager>>) -> Self {
        Self { plugin_manager }
    }
}

#[async_trait]
impl VmEventHandler for PluginEventHandler {
    async fn handle(&self, event: VmEvent) -> Result<()> {
        let _plugin_manager = self.plugin_manager.read().await;

        match event {
            VmEvent::Created { vm_id } => {
                // We'd need the actual VmInstance here
                tracing::debug!("Plugin hook: VM {} created", vm_id);
            }
            VmEvent::Started { vm_id } => {
                tracing::debug!("Plugin hook: VM {} started", vm_id);
            }
            VmEvent::Stopped { vm_id } => {
                tracing::debug!("Plugin hook: VM {} stopped", vm_id);
            }
            _ => {}
        }

        Ok(())
    }
}

// Example logging plugin
#[derive(Debug)]
pub struct LoggingPlugin {
    metadata: PluginMetadata,
}

impl Default for LoggingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "logging".to_string(),
                version: "1.0.0".to_string(),
                description: "Logs all VM lifecycle events".to_string(),
                author: "Vortex Team".to_string(),
                hooks: vec![
                    PluginHook::VmPostCreate,
                    PluginHook::VmPostStart,
                    PluginHook::VmPostStop,
                ],
            },
        }
    }
}

#[async_trait]
impl Plugin for LoggingPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("LoggingPlugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("LoggingPlugin shutting down");
        Ok(())
    }

    async fn on_vm_post_create(&self, vm: &VmInstance) -> Result<()> {
        tracing::info!("VM Created: {} ({})", vm.id, vm.spec.image);
        Ok(())
    }

    async fn on_vm_post_start(&self, vm: &VmInstance) -> Result<()> {
        tracing::info!(
            "VM Started: {} ({}MB RAM, {} CPUs)",
            vm.id,
            vm.spec.memory,
            vm.spec.cpus
        );
        Ok(())
    }

    async fn on_vm_post_stop(&self, vm: &VmInstance) -> Result<()> {
        tracing::info!("VM Stopped: {}", vm.id);
        Ok(())
    }
}
