use crate::error::Result;
use crate::vm::{VmEvent, VmEventHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmMetrics {
    pub vm_id: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_usage_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub uptime_seconds: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_vms: u32,
    pub running_vms: u32,
    pub total_cpu_usage: f64,
    pub total_memory_usage: u64,
    pub total_memory_allocated: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct MetricsCollector {
    vm_metrics: RwLock<HashMap<String, VmMetrics>>,
    system_metrics: RwLock<SystemMetrics>,
}

impl MetricsCollector {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            vm_metrics: RwLock::new(HashMap::new()),
            system_metrics: RwLock::new(SystemMetrics {
                total_vms: 0,
                running_vms: 0,
                total_cpu_usage: 0.0,
                total_memory_usage: 0,
                total_memory_allocated: 0,
                timestamp: chrono::Utc::now(),
            }),
        })
    }
    
    pub async fn record_vm_metrics(&self, metrics: VmMetrics) {
        let mut vm_metrics = self.vm_metrics.write().await;
        vm_metrics.insert(metrics.vm_id.clone(), metrics);
        
        // Update system metrics
        self.update_system_metrics().await;
    }
    
    pub async fn get_vm_metrics(&self, vm_id: &str) -> Option<VmMetrics> {
        let vm_metrics = self.vm_metrics.read().await;
        vm_metrics.get(vm_id).cloned()
    }
    
    pub async fn get_all_vm_metrics(&self) -> Vec<VmMetrics> {
        let vm_metrics = self.vm_metrics.read().await;
        vm_metrics.values().cloned().collect()
    }
    
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        let system_metrics = self.system_metrics.read().await;
        system_metrics.clone()
    }
    
    async fn update_system_metrics(&self) {
        let vm_metrics = self.vm_metrics.read().await;
        let mut system_metrics = self.system_metrics.write().await;
        
        system_metrics.total_vms = vm_metrics.len() as u32;
        system_metrics.running_vms = vm_metrics.len() as u32; // Simplified
        system_metrics.total_cpu_usage = vm_metrics.values().map(|m| m.cpu_usage_percent).sum();
        system_metrics.total_memory_usage = vm_metrics.values().map(|m| m.memory_usage_bytes).sum();
        system_metrics.total_memory_allocated = vm_metrics.values().map(|m| m.memory_total_bytes).sum();
        system_metrics.timestamp = chrono::Utc::now();
    }
}

#[async_trait]
impl VmEventHandler for MetricsCollector {
    async fn handle(&self, event: VmEvent) -> Result<()> {
        match event {
            VmEvent::Created { vm_id } => {
                tracing::info!("VM {} created - starting metrics collection", vm_id);
            }
            VmEvent::Stopped { vm_id } => {
                let mut vm_metrics = self.vm_metrics.write().await;
                vm_metrics.remove(&vm_id);
                self.update_system_metrics().await;
            }
            VmEvent::ResourceUsage { vm_id, cpu, memory } => {
                let metrics = VmMetrics {
                    vm_id: vm_id.clone(),
                    cpu_usage_percent: cpu,
                    memory_usage_bytes: memory,
                    memory_total_bytes: memory, // Simplified
                    disk_usage_bytes: 0,
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    uptime_seconds: 0,
                    timestamp: chrono::Utc::now(),
                };
                
                self.record_vm_metrics(metrics).await;
            }
            _ => {}
        }
        
        Ok(())
    }
}