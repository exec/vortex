use crate::error::{Result, VortexError};
use crate::vm::VmSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevTemplate {
    pub name: String,
    pub description: String,
    pub base_image: String,
    pub tools: Vec<String>,
    pub environment: HashMap<String, String>,
    pub startup_commands: Vec<String>,
    pub default_workdir: String,
    pub ports: Vec<String>,
    pub extensions: Vec<String>,                // VSCode extensions, etc.
    pub packages: HashMap<String, Vec<String>>, // package_manager -> packages
}

#[derive(Debug)]
pub struct DevEnvironmentManager {
    templates: HashMap<String, DevTemplate>,
}

impl Default for DevEnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DevEnvironmentManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Load built-in templates
        manager.load_builtin_templates();
        manager
    }

    fn load_builtin_templates(&mut self) {
        // Python development environment
        self.templates.insert(
            "python".to_string(),
            DevTemplate {
                name: "python".to_string(),
                description: "Complete Python development environment with pip, virtualenv, and debugging tools".to_string(),
                base_image: "python:3.11-slim".to_string(),
                tools: vec![
                    "python3".to_string(),
                    "pip".to_string(),
                    "virtualenv".to_string(),
                    "git".to_string(),
                    "curl".to_string(),
                    "vim".to_string(),
                    "nano".to_string(),
                ],
                environment: HashMap::from([
                    ("PYTHONPATH".to_string(), "/workspace".to_string()),
                    ("PIP_CACHE_DIR".to_string(), "/cache/pip".to_string()),
                    ("PYTHONDONTWRITEBYTECODE".to_string(), "1".to_string()),
                ]),
                startup_commands: vec![
                    "apt-get update && apt-get install -y git curl vim nano build-essential".to_string(),
                    "pip install --upgrade pip setuptools wheel".to_string(),
                    "pip install pytest black flake8 mypy ipython jupyter".to_string(),
                ],
                default_workdir: "/workspace".to_string(),
                ports: vec!["8000:8000".to_string(), "8888:8888".to_string()], // Common Python ports
                extensions: vec!["ms-python.python".to_string(), "ms-python.debugpy".to_string()],
                packages: HashMap::from([
                    ("pip".to_string(), vec!["requests".to_string(), "fastapi".to_string(), "pandas".to_string()]),
                ]),
            },
        );

        // Node.js development environment
        self.templates.insert(
            "node".to_string(),
            DevTemplate {
                name: "node".to_string(),
                description:
                    "Node.js development environment with npm, yarn, and development tools"
                        .to_string(),
                base_image: "node:18-slim".to_string(),
                tools: vec![
                    "node".to_string(),
                    "npm".to_string(),
                    "yarn".to_string(),
                    "git".to_string(),
                    "curl".to_string(),
                    "vim".to_string(),
                ],
                environment: HashMap::from([
                    ("NODE_ENV".to_string(), "development".to_string()),
                    ("NPM_CONFIG_CACHE".to_string(), "/cache/npm".to_string()),
                ]),
                startup_commands: vec![
                    "apt-get update && apt-get install -y git curl vim python3 make g++"
                        .to_string(),
                    "npm install -g yarn typescript ts-node nodemon eslint prettier".to_string(),
                ],
                default_workdir: "/app".to_string(),
                ports: vec![
                    "3000:3000".to_string(),
                    "8080:8080".to_string(),
                    "9229:9229".to_string(),
                ], // Dev server + debugger
                extensions: vec!["ms-vscode.vscode-typescript-next".to_string()],
                packages: HashMap::from([(
                    "npm".to_string(),
                    vec![
                        "express".to_string(),
                        "lodash".to_string(),
                        "axios".to_string(),
                    ],
                )]),
            },
        );

        // Rust development environment
        self.templates.insert(
            "rust".to_string(),
            DevTemplate {
                name: "rust".to_string(),
                description: "Rust development environment with cargo, clippy, and debugging tools"
                    .to_string(),
                base_image: "rust:1.75-slim".to_string(),
                tools: vec![
                    "cargo".to_string(),
                    "rustc".to_string(),
                    "rustfmt".to_string(),
                    "clippy".to_string(),
                    "git".to_string(),
                ],
                environment: HashMap::from([
                    ("CARGO_HOME".to_string(), "/cache/cargo".to_string()),
                    ("RUSTUP_HOME".to_string(), "/cache/rustup".to_string()),
                ]),
                startup_commands: vec![
                    "apt-get update && apt-get install -y git curl vim build-essential".to_string(),
                    "rustup component add clippy rustfmt rust-src".to_string(),
                    "cargo install cargo-watch cargo-edit cargo-audit".to_string(),
                ],
                default_workdir: "/workspace".to_string(),
                ports: vec!["8000:8000".to_string()],
                extensions: vec!["rust-lang.rust-analyzer".to_string()],
                packages: HashMap::new(),
            },
        );

        // Go development environment
        self.templates.insert(
            "go".to_string(),
            DevTemplate {
                name: "go".to_string(),
                description: "Go development environment with modules, debugging, and tools"
                    .to_string(),
                base_image: "golang:1.21-alpine".to_string(),
                tools: vec!["go".to_string(), "gofmt".to_string(), "git".to_string()],
                environment: HashMap::from([
                    ("GO111MODULE".to_string(), "on".to_string()),
                    ("GOCACHE".to_string(), "/cache/go".to_string()),
                    ("GOMODCACHE".to_string(), "/cache/gomod".to_string()),
                ]),
                startup_commands: vec![
                    "apk add --no-cache git curl vim build-base".to_string(),
                    "go install golang.org/x/tools/cmd/goimports@latest".to_string(),
                    "go install github.com/go-delve/delve/cmd/dlv@latest".to_string(),
                ],
                default_workdir: "/workspace".to_string(),
                ports: vec!["8080:8080".to_string(), "2345:2345".to_string()], // Web server + debugger
                extensions: vec!["golang.go".to_string()],
                packages: HashMap::new(),
            },
        );

        // AI/ML development environment
        self.templates.insert(
            "ai".to_string(),
            DevTemplate {
                name: "ai".to_string(),
                description: "AI/ML development environment with Python, PyTorch, TensorFlow, and Jupyter".to_string(),
                base_image: "python:3.11-slim".to_string(),
                tools: vec![
                    "python3".to_string(),
                    "pip".to_string(),
                    "jupyter".to_string(),
                    "git".to_string(),
                ],
                environment: HashMap::from([
                    ("PYTHONPATH".to_string(), "/workspace".to_string()),
                    ("CUDA_VISIBLE_DEVICES".to_string(), "0".to_string()),
                    ("JUPYTER_ENABLE_LAB".to_string(), "yes".to_string()),
                ]),
                startup_commands: vec![
                    "apt-get update && apt-get install -y git curl vim build-essential".to_string(),
                    "pip install --upgrade pip".to_string(),
                    "pip install torch torchvision tensorflow jupyter pandas numpy matplotlib scikit-learn".to_string(),
                    "pip install transformers datasets accelerate".to_string(),
                ],
                default_workdir: "/workspace".to_string(),
                ports: vec!["8888:8888".to_string(), "6006:6006".to_string()], // Jupyter + TensorBoard
                extensions: vec!["ms-python.python".to_string(), "ms-toolsai.jupyter".to_string()],
                packages: HashMap::from([
                    ("pip".to_string(), vec!["torch".to_string(), "transformers".to_string(), "datasets".to_string()]),
                ]),
            },
        );
    }

    pub fn get_template(&self, name: &str) -> Option<&DevTemplate> {
        self.templates.get(name)
    }

    pub fn list_templates(&self) -> Vec<&DevTemplate> {
        self.templates.values().collect()
    }

    pub fn template_to_vm_spec(
        &self,
        template_name: &str,
        custom_workdir: Option<String>,
    ) -> Result<VmSpec> {
        let template =
            self.get_template(template_name)
                .ok_or_else(|| VortexError::TemplateNotFound {
                    name: template_name.to_string(),
                })?;

        let workdir = custom_workdir.unwrap_or_else(|| template.default_workdir.clone());

        // Validate startup commands for dangerous shell metacharacters
        for command in &template.startup_commands {
            if command.contains('&')
                || command.contains('|')
                || command.contains(';')
                || command.contains('`')
                || command.contains('$')
                || command.contains('(')
                || command.contains(')')
                || command.contains('<')
                || command.contains('>')
                || command.contains('\n')
                || command.contains('\r')
            {
                return Err(VortexError::InvalidInput {
                    field: "startup_commands".to_string(),
                    message: format!(
                        "Startup command contains forbidden shell metacharacters: {}",
                        command
                    ),
                });
            }
        }

        // Create startup command that sets up the environment
        let setup_commands = template.startup_commands.join(" && ");
        let full_command = format!(
            "mkdir -p {} && cd {} && {} && echo 'Vortex dev environment ready!' && exec bash",
            workdir, workdir, setup_commands
        );

        let spec = VmSpec {
            image: template.base_image.clone(),
            memory: 2048, // 2GB default for dev environments
            cpus: 2,      // 2 cores default
            ports: {
                let mut parsed_ports = HashMap::new();
                for p in &template.ports {
                    let parts: Vec<&str> = p.split(':').collect();
                    if parts.len() != 2 {
                        return Err(VortexError::InvalidInput {
                            field: "ports".to_string(),
                            message: format!(
                                "Invalid port mapping format '{}', expected 'host:guest'",
                                p
                            ),
                        });
                    }
                    let host: u16 = parts[0].parse().map_err(|_| VortexError::InvalidInput {
                        field: "ports".to_string(),
                        message: format!("Invalid host port in '{}'", p),
                    })?;
                    let guest: u16 = parts[1].parse().map_err(|_| VortexError::InvalidInput {
                        field: "ports".to_string(),
                        message: format!("Invalid guest port in '{}'", p),
                    })?;
                    parsed_ports.insert(host, guest);
                }
                parsed_ports
            },
            volumes: HashMap::new(), // Will be set up by the caller
            environment: template.environment.clone(),
            command: Some(full_command),
            labels: HashMap::from([
                ("vortex.dev-env".to_string(), "true".to_string()),
                ("vortex.template".to_string(), template_name.to_string()),
            ]),
            network_config: None,
            resource_limits: crate::vm::ResourceLimits::default(),
            backend: None,
        };

        Ok(spec)
    }

    pub fn create_custom_template(&mut self, name: String, template: DevTemplate) -> Result<()> {
        if self.templates.contains_key(&name) {
            return Err(VortexError::TemplateExists { name });
        }

        self.templates.insert(name, template);
        Ok(())
    }
}
