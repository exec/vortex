use crate::error::{Result, VortexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub subnet: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
    pub enable_internet: bool,
}

#[derive(Debug, Clone)]
pub struct VmNetwork {
    pub vm_id: String,
    pub network_name: String,
    pub ip_address: String,
    pub mac_address: String,
}

pub struct NetworkManager {
    networks: HashMap<String, NetworkConfig>,
    vm_networks: HashMap<String, VmNetwork>,
}

impl NetworkManager {
    pub async fn new() -> Result<Self> {
        let mut networks = HashMap::new();

        // Create default network
        networks.insert(
            "default".to_string(),
            NetworkConfig {
                name: "default".to_string(),
                subnet: "192.168.100.0/24".to_string(),
                gateway: "192.168.100.1".to_string(),
                dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
                enable_internet: true,
            },
        );

        Ok(Self {
            networks,
            vm_networks: HashMap::new(),
        })
    }

    pub async fn create_network(&mut self, config: NetworkConfig) -> Result<()> {
        self.networks.insert(config.name.clone(), config);
        Ok(())
    }

    pub async fn assign_vm_to_network(
        &mut self,
        vm_id: &str,
        network_name: &str,
    ) -> Result<VmNetwork> {
        if !self.networks.contains_key(network_name) {
            return Err(VortexError::NetworkError {
                message: format!("Network {} does not exist", network_name),
            });
        }

        // For now, assign a static IP (in real implementation, use DHCP or IP pool)
        let ip_address = format!("192.168.100.{}", 10 + self.vm_networks.len());
        let mac_address = format!("02:00:00:00:00:{:02x}", 10 + self.vm_networks.len());

        let vm_network = VmNetwork {
            vm_id: vm_id.to_string(),
            network_name: network_name.to_string(),
            ip_address,
            mac_address,
        };

        self.vm_networks
            .insert(vm_id.to_string(), vm_network.clone());

        Ok(vm_network)
    }

    pub async fn get_vm_network(&self, vm_id: &str) -> Result<Option<&VmNetwork>> {
        Ok(self.vm_networks.get(vm_id))
    }

    pub async fn list_networks(&self) -> Result<Vec<&NetworkConfig>> {
        Ok(self.networks.values().collect())
    }
}
