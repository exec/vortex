use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum VmBackend {
    Krunvm,
    Firecracker,
}

#[derive(Debug, Clone)]
pub struct VmConfig {
    pub image: String,
    pub memory: u32,
    pub cpus: u32,
    pub ports: HashMap<u16, u16>,
    pub volumes: HashMap<PathBuf, PathBuf>,
    pub command: Option<String>,
    pub persist: bool,
}

#[derive(Debug, Clone)]
pub struct VmInstance {
    pub id: String,
    pub image: String,
    pub status: String,
    pub config: VmConfig,
    pub backend: VmBackend,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmMetadata {
    id: String,
    image: String,
    backend: String,
    created_at: String,
}

impl VmInstance {
    pub async fn wait(&self) -> Result<()> {
        match self.backend {
            VmBackend::Krunvm => self.wait_krunvm().await,
            VmBackend::Firecracker => self.wait_firecracker().await,
        }
    }
    
    pub async fn cleanup(&self) -> Result<()> {
        match self.backend {
            VmBackend::Krunvm => self.cleanup_krunvm().await,
            VmBackend::Firecracker => self.cleanup_firecracker().await,
        }
    }
    
    async fn wait_krunvm(&self) -> Result<()> {
        debug!("Waiting for krunvm instance: {}", self.id);
        
        let mut cmd = Command::new("krunvm");
        cmd.args(&["start", &self.id]);
        
        if let Some(command) = &self.config.command {
            cmd.arg("--");
            // For complex shell commands, pass them to sh
            if command.contains("&&") || command.contains("||") || command.contains("|") || command.contains(";") {
                cmd.args(&["sh", "-c", command]);
            } else {
                // For simple commands, split on whitespace
                cmd.args(command.split_whitespace());
            }
        }
        
        debug!("Running start command: {:?}", cmd);
        
        let output = cmd.output().await.context("Failed to start krunvm")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("krunvm start failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", stdout);
        }
        
        Ok(())
    }
    
    async fn wait_firecracker(&self) -> Result<()> {
        debug!("Waiting for Firecracker instance: {}", self.id);
        
        Ok(())
    }
    
    async fn cleanup_krunvm(&self) -> Result<()> {
        debug!("Cleaning up krunvm instance: {}", self.id);
        
        let output = Command::new("krunvm")
            .args(&["delete", &self.id])
            .output()
            .await
            .context("Failed to delete krunvm")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("krunvm delete failed (may already be deleted): {}", stderr);
        }
        
        Ok(())
    }
    
    async fn cleanup_firecracker(&self) -> Result<()> {
        debug!("Cleaning up Firecracker instance: {}", self.id);
        
        Ok(())
    }
}

pub fn detect_backend() -> Result<VmBackend> {
    if is_krunvm_available()? {
        Ok(VmBackend::Krunvm)
    } else if is_firecracker_available()? {
        Ok(VmBackend::Firecracker)
    } else {
        Err(anyhow::anyhow!(
            "No supported VM backend found. Install krunvm (macOS) or Firecracker (Linux)"
        ))
    }
}

fn is_krunvm_available() -> Result<bool> {
    let output = std::process::Command::new("which")
        .arg("krunvm")
        .output()
        .context("Failed to check for krunvm")?;
    
    Ok(output.status.success())
}

fn is_firecracker_available() -> Result<bool> {
    let output = std::process::Command::new("which")
        .arg("firecracker")
        .output()
        .context("Failed to check for firecracker")?;
    
    Ok(output.status.success())
}

pub async fn create_vm(backend: VmBackend, config: VmConfig) -> Result<VmInstance> {
    let vm_id = generate_vm_id();
    
    match backend {
        VmBackend::Krunvm => create_krunvm(&vm_id, &config).await,
        VmBackend::Firecracker => create_firecracker(&vm_id, &config).await,
    }
}

async fn create_krunvm(vm_id: &str, config: &VmConfig) -> Result<VmInstance> {
    info!("Creating krunvm instance: {}", vm_id);
    
    let image_name = normalize_image_name(&config.image);
    
    let mut cmd = Command::new("krunvm");
    cmd.args(&["create", &image_name]);
    cmd.arg("--name").arg(vm_id);
    
    cmd.arg("--mem").arg(config.memory.to_string());
    cmd.arg("--cpus").arg(config.cpus.to_string());
    
    for (host_port, guest_port) in &config.ports {
        cmd.arg("--port")
           .arg(format!("{}:{}", host_port, guest_port));
    }
    
    for (host_path, guest_path) in &config.volumes {
        cmd.arg("-v")
           .arg(format!("{}:{}", host_path.display(), guest_path.display()));
    }
    
    debug!("Running command: {:?}", cmd);
    
    let output = cmd.output().await.context("Failed to create krunvm")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("krunvm create failed: {}", stderr));
    }
    
    info!("Successfully created krunvm instance: {}", vm_id);
    
    Ok(VmInstance {
        id: vm_id.to_string(),
        image: config.image.clone(),
        status: "running".to_string(),
        config: config.clone(),
        backend: VmBackend::Krunvm,
    })
}

async fn create_firecracker(vm_id: &str, _config: &VmConfig) -> Result<VmInstance> {
    info!("Creating Firecracker instance: {}", vm_id);
    
    Err(anyhow::anyhow!("Firecracker backend not yet implemented"))
}

pub async fn list_vms(backend: VmBackend) -> Result<Vec<VmInstance>> {
    match backend {
        VmBackend::Krunvm => list_krunvms().await,
        VmBackend::Firecracker => list_firecracker_vms().await,
    }
}

async fn list_krunvms() -> Result<Vec<VmInstance>> {
    let output = Command::new("krunvm")
        .args(&["list"])
        .output()
        .await
        .context("Failed to list krunvms")?;
    
    if !output.status.success() {
        return Ok(Vec::new());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result = Vec::new();
    
    // Parse the text output format
    for line in stdout.lines() {
        if line.starts_with("ephemeral-") && !line.contains(" ") {
            let vm_name = line.trim().to_string();
            
            // Extract image type from buildah container name
            let mut image = "unknown".to_string();
            if let Some(container_line) = stdout.lines()
                .skip_while(|l| !l.starts_with(&vm_name))
                .find(|l| l.trim().starts_with("Buildah container:")) 
            {
                if container_line.contains("alpine-working-container") {
                    image = "alpine".to_string();
                } else if container_line.contains("ubuntu-working-container") {
                    image = "ubuntu".to_string();
                }
            }
            
            result.push(VmInstance {
                id: vm_name,
                image,
                status: "created".to_string(),
                config: VmConfig {
                    image: "unknown".to_string(),
                    memory: 512,
                    cpus: 1,
                    ports: HashMap::new(),
                    volumes: HashMap::new(),
                    command: None,
                    persist: false,
                },
                backend: VmBackend::Krunvm,
            });
        }
    }
    
    Ok(result)
}

async fn list_firecracker_vms() -> Result<Vec<VmInstance>> {
    Ok(Vec::new())
}

pub async fn stop_vm(backend: VmBackend, vm_id: &str) -> Result<()> {
    match backend {
        VmBackend::Krunvm => stop_krunvm(vm_id).await,
        VmBackend::Firecracker => stop_firecracker_vm(vm_id).await,
    }
}

async fn stop_krunvm(vm_id: &str) -> Result<()> {
    let output = Command::new("krunvm")
        .args(&["delete", vm_id])
        .output()
        .await
        .context("Failed to stop krunvm")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("krunvm delete failed: {}", stderr));
    }
    
    Ok(())
}

async fn stop_firecracker_vm(_vm_id: &str) -> Result<()> {
    Err(anyhow::anyhow!("Firecracker backend not yet implemented"))
}

pub async fn cleanup_all_vms(backend: VmBackend) -> Result<usize> {
    let vms = list_vms(backend.clone()).await?;
    let count = vms.len();
    
    for vm in vms {
        if let Err(e) = vm.cleanup().await {
            warn!("Failed to cleanup VM {}: {}", vm.id, e);
        }
    }
    
    Ok(count)
}

fn generate_vm_id() -> String {
    format!("ephemeral-{}", Uuid::new_v4().to_string()[..8].to_lowercase())
}

fn normalize_image_name(image: &str) -> String {
    match image {
        "alpine" => "docker.io/library/alpine:latest".to_string(),
        "ubuntu" => "docker.io/library/ubuntu:latest".to_string(),
        "debian" => "docker.io/library/debian:latest".to_string(),
        img if img.contains(':') => {
            if img.contains('/') {
                img.to_string()
            } else {
                format!("docker.io/library/{}", img)
            }
        }
        img => format!("docker.io/library/{}:latest", img),
    }
}