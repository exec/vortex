//! Project detection and auto-configuration module
//!
//! This module provides functionality to automatically detect project structure
//! and generate vortex.yaml configurations.

use std::path::{Path, PathBuf};

/// Information about a detected project
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Project name (directory name)
    pub name: String,
    /// Root directory of the project
    pub root_dir: PathBuf,
    /// Detected services in the project
    pub services: Vec<ServiceInfo>,
    /// Suggested default template
    pub suggested_template: String,
    /// Whether a devcontainer.json was found
    pub has_devcontainer: bool,
}

/// Information about a detected service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service type (frontend, backend, worker, database, cache, queue)
    pub service_type: String,
    /// Detected language
    pub language: String,
    /// Base image to use
    pub image: String,
    /// Default ports for this service
    pub ports: Vec<(u16, u16)>,
    /// Path to service directory
    pub path: PathBuf,
}

/// Language detection results
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Node,
    Python,
    Go,
    Rust,
    Php,
    Ruby,
    Java,
    Unknown,
}

impl Language {
    /// Detect language from directory structure
    pub fn detect(directory: &Path) -> Self {
        // Check for package.json (Node.js)
        if directory.join("package.json").exists() {
            return Language::Node;
        }

        // Check for requirements.txt (Python)
        if directory.join("requirements.txt").exists() {
            return Language::Python;
        }

        // Check for go.mod (Go)
        if directory.join("go.mod").exists() {
            return Language::Go;
        }

        // Check for Cargo.toml (Rust)
        if directory.join("Cargo.toml").exists() {
            return Language::Rust;
        }

        // Check for composer.json (PHP)
        if directory.join("composer.json").exists() {
            return Language::Php;
        }

        // Check for Gemfile (Ruby)
        if directory.join("Gemfile").exists() {
            return Language::Ruby;
        }

        // Check for pom.xml or build.gradle (Java)
        if directory.join("pom.xml").exists() || directory.join("build.gradle").exists() {
            return Language::Java;
        }

        Language::Unknown
    }

    /// Get the suggested Docker image for this language
    pub fn default_image(&self) -> &'static str {
        match self {
            Language::Node => "node:18-alpine",
            Language::Python => "python:3.11-slim",
            Language::Go => "golang:1.21-alpine",
            Language::Rust => "rust:1.70",
            Language::Php => "php:8.2-fpm-alpine",
            Language::Ruby => "ruby:3.2-alpine",
            Language::Java => "openjdk:17-alpine",
            Language::Unknown => "ubuntu:22.04",
        }
    }

    /// Get the suggested default port for this language
    pub fn default_port(&self) -> Option<u16> {
        match self {
            Language::Node => Some(3000),
            Language::Python => Some(8000),
            Language::Go => Some(8080),
            Language::Rust => Some(8080),
            Language::Php => Some(9000),
            Language::Ruby => Some(3000),
            Language::Java => Some(8080),
            Language::Unknown => None,
        }
    }
}

/// Service type detection
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceType {
    Frontend,
    Backend,
    Worker,
    Database,
    Cache,
    Queue,
    Unknown,
}

impl ServiceType {
    /// Detect service type from directory name
    pub fn from_directory_name(dir_name: &str) -> Self {
        match dir_name.to_lowercase().as_str() {
            "frontend" | "ui" | "client" | "web" => ServiceType::Frontend,
            "backend" | "api" | "server" => ServiceType::Backend,
            "worker" | "jobs" | "tasks" => ServiceType::Worker,
            "database" | "db" | "migrations" => ServiceType::Database,
            "cache" | "redis" => ServiceType::Cache,
            "queue" | "nats" | "rabbitmq" => ServiceType::Queue,
            _ => ServiceType::Unknown,
        }
    }

    /// Get the service name for YAML config
    pub fn to_yaml_name(&self) -> &'static str {
        match self {
            ServiceType::Frontend => "frontend",
            ServiceType::Backend => "backend",
            ServiceType::Worker => "worker",
            ServiceType::Database => "database",
            ServiceType::Cache => "cache",
            ServiceType::Queue => "queue",
            ServiceType::Unknown => "service",
        }
    }
}

/// Main discovery scanner
pub struct Scanner {
    directory: PathBuf,
}

impl Scanner {
    /// Create a new scanner for a directory
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Scan the directory and return project information
    pub fn scan(&self) -> Result<ProjectInfo, String> {
        let dir_name = self
            .directory
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "my-project".to_string());

        // Scan subdirectories for services
        let mut services = vec![];
        let has_devcontainer = self
            .directory
            .join(".devcontainer/devcontainer.json")
            .exists();

        // Get all subdirectories
        let entries = std::fs::read_dir(&self.directory)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }

            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();

            // Skip common ignore directories
            if ["node_modules", ".git", "target", "dist", "build"].contains(&&*dir_name_str) {
                continue;
            }

            let service_info = self.scan_service_directory(&entry.path())?;
            if let Some(info) = service_info {
                services.push(info);
            }
        }

        // Determine suggested template based on services
        let suggested_template = self.suggest_template(&services);

        Ok(ProjectInfo {
            name: dir_name,
            root_dir: self.directory.clone(),
            services,
            suggested_template,
            has_devcontainer,
        })
    }

    /// Scan a single service directory
    fn scan_service_directory(&self, path: &Path) -> Result<Option<ServiceInfo>, String> {
        let lang = Language::detect(path);

        if lang == Language::Unknown {
            return Ok(None);
        }

        let dir_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let service_type = ServiceType::from_directory_name(&dir_name);

        let ports = if let Some(default_port) = lang.default_port() {
            vec![(default_port, default_port)]
        } else {
            vec![]
        };

        Ok(Some(ServiceInfo {
            name: dir_name,
            service_type: service_type.to_yaml_name().to_string(),
            language: lang.to_string(),
            image: lang.default_image().to_string(),
            ports,
            path: path.to_path_buf(),
        }))
    }

    /// Suggest a template based on detected services
    fn suggest_template(&self, services: &[ServiceInfo]) -> String {
        if services.is_empty() {
            return "python".to_string();
        }

        // Count service types
        let mut frontend_count = 0;
        let mut backend_count = 0;
        let mut database_count = 0;

        for service in services {
            match service.service_type.as_str() {
                "frontend" => frontend_count += 1,
                "backend" | "worker" => backend_count += 1,
                "database" => database_count += 1,
                _ => {}
            }
        }

        // Suggest template based on mix
        if frontend_count > 0 && backend_count > 0 && database_count > 0 {
            "fullstack-webapp".to_string()
        } else if backend_count > 1 || database_count > 0 {
            "microservices-api".to_string()
        } else if frontend_count > 0 {
            "fullstack-webapp".to_string()
        } else {
            "python".to_string()
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Node => write!(f, "node"),
            Language::Python => write!(f, "python"),
            Language::Go => write!(f, "go"),
            Language::Rust => write!(f, "rust"),
            Language::Php => write!(f, "php"),
            Language::Ruby => write!(f, "ruby"),
            Language::Java => write!(f, "java"),
            Language::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detect workspace info from a directory
pub fn detect_workspace_info(directory: &Path) -> Option<ProjectInfo> {
    let scanner = Scanner::new(directory.to_path_buf());
    scanner.scan().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection_node() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert_eq!(Language::detect(temp.path()), Language::Node);
    }

    #[test]
    fn test_language_detection_python() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("requirements.txt"), "").unwrap();
        assert_eq!(Language::detect(temp.path()), Language::Python);
    }

    #[test]
    fn test_service_type_detection() {
        assert_eq!(
            ServiceType::from_directory_name("frontend"),
            ServiceType::Frontend
        );
        assert_eq!(
            ServiceType::from_directory_name("backend"),
            ServiceType::Backend
        );
        assert_eq!(
            ServiceType::from_directory_name("database"),
            ServiceType::Database
        );
    }
}
