use crate::error::{Result, VortexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VortexConfig {
    pub default_backend: Option<String>,
    pub default_memory: u32,
    pub default_cpus: u32,
    pub image_aliases: HashMap<String, String>,
    pub templates: HashMap<String, Template>,
    pub resource_limits: GlobalResourceLimits,
    pub networking: NetworkingConfig,
    pub storage: StorageConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub image: String,
    pub memory: u32,
    pub cpus: u32,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub environment: HashMap<String, String>,
    pub command: Option<String>,
    pub description: String,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalResourceLimits {
    pub max_memory_per_vm: u32,
    pub max_cpus_per_vm: u32,
    pub max_concurrent_vms: u32,
    pub max_total_memory: u32,
    pub default_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkingConfig {
    pub default_network: String,
    pub enable_inter_vm: bool,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub default_volume_size: u64,
    pub snapshot_directory: PathBuf,
    pub cache_directory: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_interval_seconds: u64,
    pub retention_days: u32,
}

impl Default for VortexConfig {
    fn default() -> Self {
        let mut image_aliases = HashMap::new();
        image_aliases.insert(
            "alpine".to_string(),
            "docker.io/library/alpine:latest".to_string(),
        );
        image_aliases.insert(
            "ubuntu".to_string(),
            "docker.io/library/ubuntu:22.04".to_string(),
        );
        image_aliases.insert(
            "debian".to_string(),
            "docker.io/library/debian:bullseye".to_string(),
        );
        image_aliases.insert(
            "node".to_string(),
            "docker.io/library/node:18-alpine".to_string(),
        );
        image_aliases.insert(
            "python".to_string(),
            "docker.io/library/python:3.11-alpine".to_string(),
        );
        image_aliases.insert(
            "rust".to_string(),
            "docker.io/library/rust:alpine".to_string(),
        );

        let mut templates = HashMap::new();
        templates.insert(
            "dev".to_string(),
            Template {
                image: "ubuntu:22.04".to_string(),
                memory: 2048,
                cpus: 2,
                ports: vec!["8080:80".to_string(), "3000:3000".to_string()],
                volumes: vec![],
                environment: HashMap::new(),
                command: Some("bash".to_string()),
                description: "Development environment with common ports".to_string(),
                labels: HashMap::new(),
            },
        );

        templates.insert(
            "web".to_string(),
            Template {
                image: "node:18-alpine".to_string(),
                memory: 1024,
                cpus: 1,
                ports: vec!["3000:3000".to_string(), "8080:8080".to_string()],
                volumes: vec![],
                environment: HashMap::new(),
                command: Some("sh".to_string()),
                description: "Web development with Node.js".to_string(),
                labels: HashMap::new(),
            },
        );

        templates.insert(
            "minimal".to_string(),
            Template {
                image: "alpine:latest".to_string(),
                memory: 256,
                cpus: 1,
                ports: vec![],
                volumes: vec![],
                environment: HashMap::new(),
                command: Some("sh".to_string()),
                description: "Minimal Alpine Linux environment".to_string(),
                labels: HashMap::new(),
            },
        );

        Self {
            default_backend: None,
            default_memory: 512,
            default_cpus: 1,
            image_aliases,
            templates,
            resource_limits: GlobalResourceLimits::default(),
            networking: NetworkingConfig::default(),
            storage: StorageConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for GlobalResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_per_vm: 8192,
            max_cpus_per_vm: 8,
            max_concurrent_vms: 10,
            max_total_memory: 16384,
            default_timeout_seconds: 3600,
        }
    }
}

impl Default for NetworkingConfig {
    fn default() -> Self {
        Self {
            default_network: "default".to_string(),
            enable_inter_vm: false,
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let base_dir = PathBuf::from(home).join(".vortex");

        Self {
            default_volume_size: 1024 * 1024 * 1024, // 1GB
            snapshot_directory: base_dir.join("snapshots"),
            cache_directory: base_dir.join("cache"),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_interval_seconds: 30,
            retention_days: 7,
        }
    }
}

impl VortexConfig {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: VortexConfig =
                toml::from_str(&content).map_err(|e| VortexError::ConfigError {
                    message: format!("Failed to parse config: {}", e),
                })?;
            Ok(config)
        } else {
            let config = VortexConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| VortexError::ConfigError {
            message: format!("Failed to serialize config: {}", e),
        })?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    pub fn resolve_image(&self, image: &str) -> String {
        self.image_aliases
            .get(image)
            .cloned()
            .unwrap_or_else(|| image.to_string())
    }

    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| VortexError::ConfigError {
            message: "Could not determine home directory".to_string(),
        })?;

    Ok(PathBuf::from(home)
        .join(".config")
        .join("vortex")
        .join("config.toml"))
}
