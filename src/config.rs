use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Use dirs crate for secure home directory detection
use dirs::home_dir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_backend: Option<String>,
    pub default_memory: u32,
    pub default_cpus: u32,
    pub image_aliases: HashMap<String, String>,
    pub templates: HashMap<String, VmTemplate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VmTemplate {
    pub image: String,
    pub memory: u32,
    pub cpus: u32,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub command: Option<String>,
    pub description: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut image_aliases = HashMap::new();
        image_aliases.insert("alpine".to_string(), "docker.io/library/alpine:latest".to_string());
        image_aliases.insert("ubuntu".to_string(), "docker.io/library/ubuntu:22.04".to_string());
        image_aliases.insert("debian".to_string(), "docker.io/library/debian:bullseye".to_string());
        image_aliases.insert("node".to_string(), "docker.io/library/node:18-alpine".to_string());
        image_aliases.insert("python".to_string(), "docker.io/library/python:3.11-alpine".to_string());
        image_aliases.insert("rust".to_string(), "docker.io/library/rust:alpine".to_string());
        
        let mut templates = HashMap::new();
        
        templates.insert("dev".to_string(), VmTemplate {
            image: "ubuntu:22.04".to_string(),
            memory: 2048,
            cpus: 2,
            ports: vec!["8080:80".to_string(), "3000:3000".to_string()],
            volumes: vec![],
            command: Some("bash".to_string()),
            description: "Development environment with common ports".to_string(),
        });
        
        templates.insert("web".to_string(), VmTemplate {
            image: "node:18-alpine".to_string(),
            memory: 1024,
            cpus: 1,
            ports: vec!["3000:3000".to_string(), "8080:8080".to_string()],
            volumes: vec![],
            command: Some("sh".to_string()),
            description: "Web development with Node.js".to_string(),
        });
        
        templates.insert("minimal".to_string(), VmTemplate {
            image: "alpine:latest".to_string(),
            memory: 256,
            cpus: 1,
            ports: vec![],
            volumes: vec![],
            command: Some("sh".to_string()),
            description: "Minimal Alpine Linux environment".to_string(),
        });
        
        Self {
            default_backend: None,
            default_memory: 512,
            default_cpus: 1,
            image_aliases,
            templates,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    pub fn resolve_image(&self, image: &str) -> String {
        self.image_aliases.get(image)
            .cloned()
            .unwrap_or_else(|| image.to_string())
    }
    
    pub fn get_template(&self, name: &str) -> Option<&VmTemplate> {
        self.templates.get(name)
    }
}

fn get_config_path() -> Result<PathBuf> {
    // Use dirs crate for secure home directory detection
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    // Validate home directory exists and is a directory
    if !home.exists() || !home.is_dir() {
        return Err(anyhow::anyhow!("Invalid home directory: {}", home.display()));
    }

    // Canonicalize to prevent symlink attacks
    let canonical_home = std::fs::canonicalize(&home).map_err(|e| {
        anyhow::anyhow!("Failed to canonicalize home directory: {}", e)
    })?;

    Ok(canonical_home.join(".config").join("vortex").join("config.toml"))
}