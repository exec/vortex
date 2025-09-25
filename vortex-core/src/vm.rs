use crate::backend::{Backend, BackendProvider};
use crate::error::{Result, VortexError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmSpec {
    pub image: String,
    pub memory: u32,
    pub cpus: u32,
    pub ports: HashMap<u16, u16>,
    pub volumes: HashMap<PathBuf, PathBuf>,
    pub environment: HashMap<String, String>,
    pub command: Option<String>,
    pub labels: HashMap<String, String>,
    pub network_config: Option<String>,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory: Option<u32>,
    pub max_cpus: Option<u32>,
    pub max_disk: Option<u64>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmState {
    Creating,
    Running,
    Paused,
    Stopped,
    Error { message: String },
    Snapshotting,
    Restoring,
}

#[derive(Debug, Clone)]
pub struct VmInstance {
    pub id: String,
    pub spec: VmSpec,
    pub state: VmState,
    pub backend: Arc<dyn Backend>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmEvent {
    Created {
        vm_id: String,
    },
    Started {
        vm_id: String,
    },
    Stopped {
        vm_id: String,
    },
    Error {
        vm_id: String,
        error: String,
    },
    SnapshotCreated {
        vm_id: String,
        snapshot_id: String,
    },
    ResourceUsage {
        vm_id: String,
        cpu: f64,
        memory: u64,
    },
}

pub struct VmManager {
    instances: RwLock<HashMap<String, VmInstance>>,
    backend_provider: BackendProvider,
    event_handlers: RwLock<Vec<Box<dyn VmEventHandler>>>,
}

#[async_trait]
pub trait VmEventHandler: Send + Sync {
    async fn handle(&self, event: VmEvent) -> Result<()>;
}

impl VmManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            instances: RwLock::new(HashMap::new()),
            backend_provider: BackendProvider::new().await?,
            event_handlers: RwLock::new(Vec::new()),
        })
    }

    pub async fn create(&self, spec: VmSpec) -> Result<VmInstance> {
        let vm_id = generate_vm_id();
        let backend = self.backend_provider.get_backend().await?;

        tracing::info!("Creating VM {} with spec: {:?}", vm_id, spec);

        // Validate resource limits
        self.validate_spec(&spec).await?;

        let vm = VmInstance {
            id: vm_id.clone(),
            spec: spec.clone(),
            state: VmState::Creating,
            backend,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Store instance
        {
            let mut instances = self.instances.write().await;
            instances.insert(vm_id.clone(), vm.clone());
        }

        // Create VM via backend
        match vm.backend.create(&vm).await {
            Ok(_) => {
                let mut updated_vm = vm.clone();
                updated_vm.state = VmState::Running;
                updated_vm.updated_at = chrono::Utc::now();

                {
                    let mut instances = self.instances.write().await;
                    instances.insert(vm_id.clone(), updated_vm.clone());
                }

                self.emit_event(VmEvent::Created {
                    vm_id: vm_id.clone(),
                })
                .await?;
                self.emit_event(VmEvent::Started { vm_id }).await?;

                Ok(updated_vm)
            }
            Err(e) => {
                let mut failed_vm = vm;
                failed_vm.state = VmState::Error {
                    message: e.to_string(),
                };

                {
                    let mut instances = self.instances.write().await;
                    instances.insert(vm_id.clone(), failed_vm);
                }

                self.emit_event(VmEvent::Error {
                    vm_id,
                    error: e.to_string(),
                })
                .await?;

                Err(e)
            }
        }
    }

    pub async fn get(&self, vm_id: &str) -> Result<Option<VmInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.get(vm_id).cloned())
    }

    pub async fn list(&self) -> Result<Vec<VmInstance>> {
        // First try to get from our in-memory instances
        let instances = self.instances.read().await;
        if !instances.is_empty() {
            return Ok(instances.values().cloned().collect());
        }

        // If no in-memory instances, query the backend directly
        let backend = self.backend_provider.get_backend().await?;
        let vm_names = backend.list_vms().await?;

        let mut vm_instances = Vec::new();
        for vm_name in vm_names {
            // Only include VMs that match our naming pattern
            if vm_name.starts_with("vortex-") {
                // Create a minimal VmInstance for display purposes
                let vm = VmInstance {
                    id: vm_name.clone(),
                    spec: VmSpec {
                        image: "unknown".to_string(),
                        memory: 512, // Default values since we can't query these from krunvm easily
                        cpus: 1,
                        ports: HashMap::new(),
                        volumes: HashMap::new(),
                        environment: HashMap::new(),
                        command: None,
                        labels: HashMap::new(),
                        network_config: None,
                        resource_limits: ResourceLimits::default(),
                    },
                    state: VmState::Running,
                    backend: Arc::clone(&backend),
                    created_at: chrono::Utc::now(), // We don't know the real creation time
                    updated_at: chrono::Utc::now(),
                };
                vm_instances.push(vm);
            }
        }

        Ok(vm_instances)
    }

    pub async fn stop(&self, vm_id: &str) -> Result<()> {
        // First check if we have the VM in memory
        let vm_opt = {
            let instances = self.instances.read().await;
            instances.get(vm_id).cloned()
        };

        let vm = if let Some(vm) = vm_opt {
            vm
        } else {
            // If not in memory, check if it exists in the backend
            let backend = self.backend_provider.get_backend().await?;
            let vm_names = backend.list_vms().await?;

            if vm_names.contains(&vm_id.to_string()) {
                // Create a minimal VM instance to use for stopping
                VmInstance {
                    id: vm_id.to_string(),
                    spec: VmSpec {
                        image: "unknown".to_string(),
                        memory: 512,
                        cpus: 1,
                        ports: HashMap::new(),
                        volumes: HashMap::new(),
                        environment: HashMap::new(),
                        command: None,
                        labels: HashMap::new(),
                        network_config: None,
                        resource_limits: ResourceLimits::default(),
                    },
                    state: VmState::Running,
                    backend: Arc::clone(&backend),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            } else {
                return Err(VortexError::VmError {
                    message: format!("VM {} not found", vm_id),
                });
            }
        };

        vm.backend.stop(&vm).await?;

        let mut updated_vm = vm;
        updated_vm.state = VmState::Stopped;
        updated_vm.updated_at = chrono::Utc::now();

        {
            let mut instances = self.instances.write().await;
            instances.insert(vm_id.to_string(), updated_vm);
        }

        self.emit_event(VmEvent::Stopped {
            vm_id: vm_id.to_string(),
        })
        .await?;

        Ok(())
    }

    pub async fn cleanup(&self, vm_id: &str) -> Result<()> {
        // First check if we have the VM in memory
        let vm_opt = {
            let mut instances = self.instances.write().await;
            instances.remove(vm_id)
        };

        let vm = if let Some(vm) = vm_opt {
            vm
        } else {
            // If not in memory, check if it exists in the backend
            let backend = self.backend_provider.get_backend().await?;
            let vm_names = backend.list_vms().await?;

            if vm_names.contains(&vm_id.to_string()) {
                // Create a minimal VM instance to use for cleanup
                VmInstance {
                    id: vm_id.to_string(),
                    spec: VmSpec {
                        image: "unknown".to_string(),
                        memory: 512,
                        cpus: 1,
                        ports: HashMap::new(),
                        volumes: HashMap::new(),
                        environment: HashMap::new(),
                        command: None,
                        labels: HashMap::new(),
                        network_config: None,
                        resource_limits: ResourceLimits::default(),
                    },
                    state: VmState::Running,
                    backend: Arc::clone(&backend),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            } else {
                return Err(VortexError::VmError {
                    message: format!("VM {} not found", vm_id),
                });
            }
        };

        vm.backend.cleanup(&vm).await?;
        Ok(())
    }

    pub async fn attach(&self, vm_id: &str) -> Result<()> {
        let vm = {
            let instances = self.instances.read().await;
            instances.get(vm_id).cloned()
        };

        if let Some(vm) = vm {
            vm.backend.attach(&vm).await
        } else {
            Err(VortexError::VmError {
                message: format!("VM {} not found", vm_id),
            })
        }
    }

    pub async fn add_event_handler(&self, handler: Box<dyn VmEventHandler>) {
        let mut handlers = self.event_handlers.write().await;
        handlers.push(handler);
    }

    async fn emit_event(&self, event: VmEvent) -> Result<()> {
        let handlers = self.event_handlers.read().await;

        for handler in handlers.iter() {
            if let Err(e) = handler.handle(event.clone()).await {
                tracing::warn!("Event handler failed: {}", e);
            }
        }

        Ok(())
    }

    async fn validate_spec(&self, spec: &VmSpec) -> Result<()> {
        if spec.memory == 0 {
            return Err(VortexError::InvalidInput {
                field: "memory".to_string(),
                message: "Memory must be greater than 0".to_string(),
            });
        }

        if spec.cpus == 0 {
            return Err(VortexError::InvalidInput {
                field: "cpus".to_string(),
                message: "CPUs must be greater than 0".to_string(),
            });
        }

        // Check resource limits
        if let Some(max_memory) = spec.resource_limits.max_memory {
            if spec.memory > max_memory {
                return Err(VortexError::ResourceLimitExceeded {
                    resource: format!("memory: {} > {}", spec.memory, max_memory),
                });
            }
        }

        Ok(())
    }
}

fn generate_vm_id() -> String {
    let uuid_str = Uuid::new_v4().to_string();
    format!("vortex-{}", &uuid_str[..8])
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: None,
            max_cpus: None,
            max_disk: None,
            timeout_seconds: None,
        }
    }
}
