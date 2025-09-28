use crate::error::{Result, VortexError};
use crate::templates::DevTemplate;
use crate::vm::VmSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevContainerConfig {
    #[serde(rename = "dockerComposeFile")]
    pub docker_compose_file: Option<String>,
    #[serde(rename = "dockerFile")]
    pub dockerfile: Option<String>,
    pub image: Option<String>,
    pub name: Option<String>,

    #[serde(rename = "customizations")]
    pub customizations: Option<DevContainerCustomizations>,

    #[serde(rename = "forwardPorts")]
    pub forward_ports: Option<Vec<u16>>,

    #[serde(rename = "postCreateCommand")]
    pub post_create_command: Option<String>,
    #[serde(rename = "postStartCommand")]
    pub post_start_command: Option<String>,

    #[serde(rename = "workspaceFolder")]
    pub workspace_folder: Option<String>,
    #[serde(rename = "workspaceMount")]
    pub workspace_mount: Option<String>,

    #[serde(rename = "remoteUser")]
    pub remote_user: Option<String>,

    #[serde(rename = "mounts")]
    pub mounts: Option<Vec<String>>,

    #[serde(rename = "features")]
    pub features: Option<HashMap<String, serde_json::Value>>,

    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevContainerCustomizations {
    pub vscode: Option<VsCodeCustomizations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeCustomizations {
    pub extensions: Option<Vec<String>>,
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VortexWorkspaceConfig {
    pub name: String,
    pub template: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub custom_commands: Vec<String>,
    pub preferred_workdir: String,
    pub environment_vars: HashMap<String, String>,
    pub port_forwards: Vec<u16>,

    /// If present, indicates this workspace was created from a devcontainer.json
    pub devcontainer_source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub config: VortexWorkspaceConfig,
}

#[derive(Debug)]
pub struct WorkspaceManager {
    workspaces_dir: PathBuf,
}

impl WorkspaceManager {
    pub fn new() -> Result<Self> {
        let workspaces_dir = Self::get_workspaces_dir()?;
        fs::create_dir_all(&workspaces_dir)?;

        Ok(Self { workspaces_dir })
    }

    fn get_workspaces_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| VortexError::ConfigError {
            message: "HOME environment variable not set".to_string(),
        })?;
        Ok(PathBuf::from(home).join(".vortex").join("workspaces"))
    }

    /// Create a new workspace
    pub fn create_workspace(
        &self,
        name: &str,
        template: &str,
        source_dir: Option<&Path>,
    ) -> Result<Workspace> {
        let workspace_id = Uuid::new_v4().to_string();
        let workspace_dir = self.workspaces_dir.join(&workspace_id);

        fs::create_dir_all(&workspace_dir)?;

        let config = VortexWorkspaceConfig {
            name: name.to_string(),
            template: template.to_string(),
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
            custom_commands: Vec::new(),
            preferred_workdir: "/workspace".to_string(),
            environment_vars: HashMap::new(),
            port_forwards: Vec::new(),
            devcontainer_source: None,
        };

        // Save config
        self.save_workspace_config(&workspace_id, &config)?;

        // Copy initial source if provided
        if let Some(source) = source_dir {
            copy_dir_all(source, &workspace_dir)?;
        }

        Ok(Workspace {
            id: workspace_id,
            name: name.to_string(),
            path: workspace_dir,
            config,
        })
    }

    /// Create workspace from existing devcontainer.json
    pub fn create_from_devcontainer(
        &self,
        name: &str,
        devcontainer_path: &Path,
        source_dir: &Path,
    ) -> Result<Workspace> {
        let devcontainer_config = self.parse_devcontainer(devcontainer_path)?;

        // Convert devcontainer config to Vortex template
        let template = self.devcontainer_to_template(&devcontainer_config)?;

        let workspace_id = Uuid::new_v4().to_string();
        let workspace_dir = self.workspaces_dir.join(&workspace_id);

        fs::create_dir_all(&workspace_dir)?;

        let config = VortexWorkspaceConfig {
            name: name.to_string(),
            template: template.clone(),
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
            custom_commands: self.extract_commands(&devcontainer_config),
            preferred_workdir: devcontainer_config
                .workspace_folder
                .clone()
                .unwrap_or_else(|| "/workspace".to_string()),
            environment_vars: HashMap::new(),
            port_forwards: devcontainer_config
                .forward_ports
                .clone()
                .unwrap_or_default(),
            devcontainer_source: Some(devcontainer_path.to_string_lossy().to_string()),
        };

        // Save config and copy source
        self.save_workspace_config(&workspace_id, &config)?;
        copy_dir_all(source_dir, &workspace_dir)?;

        Ok(Workspace {
            id: workspace_id,
            name: name.to_string(),
            path: workspace_dir,
            config,
        })
    }

    /// Get workspace by ID
    pub fn get_workspace(&self, workspace_id: &str) -> Result<Option<Workspace>> {
        let workspace_dir = self.workspaces_dir.join(workspace_id);
        if !workspace_dir.exists() {
            return Ok(None);
        }

        let config = self.load_workspace_config(workspace_id)?;

        Ok(Some(Workspace {
            id: workspace_id.to_string(),
            name: config.name.clone(),
            path: workspace_dir,
            config,
        }))
    }

    /// Find workspace by name
    pub fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>> {
        for workspace in self.list_workspaces()? {
            if workspace.name == name {
                return Ok(Some(workspace));
            }
        }
        Ok(None)
    }

    /// List all workspaces
    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let mut workspaces = Vec::new();

        if !self.workspaces_dir.exists() {
            return Ok(workspaces);
        }

        for entry in fs::read_dir(&self.workspaces_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let workspace_id = entry.file_name().to_string_lossy().to_string();
                if let Some(workspace) = self.get_workspace(&workspace_id)? {
                    workspaces.push(workspace);
                }
            }
        }

        // Sort by last used, most recent first
        workspaces.sort_by(|a, b| b.config.last_used.cmp(&a.config.last_used));

        Ok(workspaces)
    }

    /// Update workspace last used time
    pub fn touch_workspace(&self, workspace_id: &str) -> Result<()> {
        if let Some(mut workspace) = self.get_workspace(workspace_id)? {
            workspace.config.last_used = chrono::Utc::now();
            self.save_workspace_config(workspace_id, &workspace.config)?;
        }
        Ok(())
    }

    /// Delete workspace
    pub fn delete_workspace(&self, workspace_id: &str) -> Result<()> {
        let workspace_dir = self.workspaces_dir.join(workspace_id);
        if workspace_dir.exists() {
            fs::remove_dir_all(workspace_dir)?;
        }
        Ok(())
    }

    /// Convert workspace to VM spec
    pub fn workspace_to_vm_spec(
        &self,
        workspace: &Workspace,
        base_template: &DevTemplate,
    ) -> Result<VmSpec> {
        let mut spec = VmSpec {
            image: base_template.base_image.clone(),
            memory: 2048,
            cpus: 2,
            ports: HashMap::new(),
            volumes: HashMap::new(),
            environment: base_template.environment.clone(),
            command: None,
            labels: HashMap::from([
                ("vortex.workspace".to_string(), workspace.id.clone()),
                ("vortex.workspace-name".to_string(), workspace.name.clone()),
            ]),
            network_config: None,
            resource_limits: crate::vm::ResourceLimits::default(),
        };

        // Add workspace volume mount
        spec.volumes.insert(
            workspace.path.clone(),
            PathBuf::from(&workspace.config.preferred_workdir),
        );

        // Add port forwards
        for port in &workspace.config.port_forwards {
            spec.ports.insert(*port, *port);
        }

        // Add environment variables
        for (key, value) in &workspace.config.environment_vars {
            spec.environment.insert(key.clone(), value.clone());
        }

        // Build startup command
        let mut startup_commands = base_template.startup_commands.clone();
        startup_commands.extend(workspace.config.custom_commands.clone());

        let setup_commands = startup_commands.join(" && ");
        let full_command = format!(
            "cd {} && {} && echo 'Vortex workspace \"{}\" ready!' && exec bash",
            workspace.config.preferred_workdir, setup_commands, workspace.name
        );

        spec.command = Some(full_command);

        Ok(spec)
    }

    fn save_workspace_config(
        &self,
        workspace_id: &str,
        config: &VortexWorkspaceConfig,
    ) -> Result<()> {
        let config_path = self.workspaces_dir.join(workspace_id).join(".vortex.json");
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    fn load_workspace_config(&self, workspace_id: &str) -> Result<VortexWorkspaceConfig> {
        let config_path = self.workspaces_dir.join(workspace_id).join(".vortex.json");
        let config_json = fs::read_to_string(config_path)?;
        let config: VortexWorkspaceConfig = serde_json::from_str(&config_json)?;
        Ok(config)
    }

    fn parse_devcontainer(&self, devcontainer_path: &Path) -> Result<DevContainerConfig> {
        let content = fs::read_to_string(devcontainer_path)?;
        let config: DevContainerConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn devcontainer_to_template(&self, devcontainer: &DevContainerConfig) -> Result<String> {
        // Try to map common devcontainer images to our templates
        if let Some(image) = &devcontainer.image {
            if image.contains("python") || image.contains("Python") {
                return Ok("python".to_string());
            }
            if image.contains("node") || image.contains("Node") {
                return Ok("node".to_string());
            }
            if image.contains("rust") || image.contains("Rust") {
                return Ok("rust".to_string());
            }
            if image.contains("golang") || image.contains("go:") {
                return Ok("go".to_string());
            }
        }

        // Default to python if we can't determine
        Ok("python".to_string())
    }

    fn extract_commands(&self, devcontainer: &DevContainerConfig) -> Vec<String> {
        let mut commands = Vec::new();

        if let Some(post_create) = &devcontainer.post_create_command {
            commands.push(post_create.clone());
        }

        if let Some(post_start) = &devcontainer.post_start_command {
            commands.push(post_start.clone());
        }

        commands
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

/// Smart workspace detection - looks for common project indicators
pub fn detect_workspace_info(dir: &Path) -> Option<WorkspaceInfo> {
    let mut info = WorkspaceInfo {
        name: dir.file_name()?.to_string_lossy().to_string(),
        suggested_template: "python".to_string(),
        has_devcontainer: false,
        devcontainer_path: None,
    };

    // Check for devcontainer
    let devcontainer_paths = [
        dir.join(".devcontainer").join("devcontainer.json"),
        dir.join(".devcontainer.json"),
    ];

    for path in &devcontainer_paths {
        if path.exists() {
            info.has_devcontainer = true;
            info.devcontainer_path = Some(path.clone());
            break;
        }
    }

    // Detect project type
    if dir.join("Cargo.toml").exists() {
        info.suggested_template = "rust".to_string();
    } else if dir.join("package.json").exists() {
        info.suggested_template = "node".to_string();
    } else if dir.join("go.mod").exists() {
        info.suggested_template = "go".to_string();
    } else if dir.join("requirements.txt").exists()
        || dir.join("pyproject.toml").exists()
        || dir.join("setup.py").exists()
    {
        info.suggested_template = "python".to_string();
    }

    Some(info)
}

#[derive(Debug)]
pub struct WorkspaceInfo {
    pub name: String,
    pub suggested_template: String,
    pub has_devcontainer: bool,
    pub devcontainer_path: Option<PathBuf>,
}
