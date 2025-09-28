use crate::error::{Result, VortexError};
use crate::vm::VmInstance;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Backend: Send + Sync + std::fmt::Debug {
    /// Create a new VM instance
    async fn create(&self, vm: &VmInstance) -> Result<()>;

    /// Start a VM instance
    async fn start(&self, vm: &VmInstance) -> Result<()>;

    /// Stop a VM instance
    async fn stop(&self, vm: &VmInstance) -> Result<()>;

    /// Cleanup/destroy a VM instance
    async fn cleanup(&self, vm: &VmInstance) -> Result<()>;

    /// Attach to an interactive session
    async fn attach(&self, vm: &VmInstance) -> Result<()>;

    /// Get VM metrics
    async fn get_metrics(&self, vm: &VmInstance) -> Result<VmMetrics>;

    /// List all VMs managed by this backend
    async fn list_vms(&self) -> Result<Vec<String>>;

    /// Check if backend is available
    async fn is_available(&self) -> Result<bool>;

    /// Get backend name
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct VmMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub disk_usage: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub uptime_seconds: u64,
}

pub struct BackendProvider {
    backends: HashMap<String, Arc<dyn Backend>>,
    preferred: Option<String>,
}

impl BackendProvider {
    pub async fn new() -> Result<Self> {
        let provider = Self {
            backends: HashMap::new(),
            preferred: None,
        };

        // Register available backends
        #[cfg(feature = "krunvm")]
        {
            let krunvm = KrunvmBackend::new().await?;
            if krunvm.is_available().await? {
                provider.register("krunvm", Arc::new(krunvm));
            }
        }

        #[cfg(feature = "firecracker")]
        {
            let firecracker = FirecrackerBackend::new().await?;
            if firecracker.is_available().await? {
                provider.register("firecracker", Arc::new(firecracker));
            }
        }

        Ok(provider)
    }

    pub fn register(&mut self, name: &str, backend: Arc<dyn Backend>) {
        if self.preferred.is_none() {
            self.preferred = Some(name.to_string());
        }
        self.backends.insert(name.to_string(), backend);
    }

    pub async fn get_backend(&self) -> Result<Arc<dyn Backend>> {
        if let Some(preferred) = &self.preferred {
            if let Some(backend) = self.backends.get(preferred) {
                return Ok(Arc::clone(backend));
            }
        }

        Err(VortexError::BackendUnavailable {
            backend: "none available".to_string(),
        })
    }

    pub fn list_backends(&self) -> Vec<&str> {
        self.backends.keys().map(|s| s.as_str()).collect()
    }
}

// Krunvm Backend Implementation
#[cfg(feature = "krunvm")]
#[derive(Debug)]
pub struct KrunvmBackend;

#[cfg(feature = "krunvm")]
impl KrunvmBackend {
    pub async fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Create a krunvm Command with proper environment setup for macOS
    fn krunvm_command() -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("krunvm");

        // Set library path for krunvm on macOS
        if cfg!(target_os = "macos") {
            if let Ok(brew_prefix) = std::process::Command::new("brew")
                .args(["--prefix"])
                .output()
            {
                if brew_prefix.status.success() {
                    let prefix_str = String::from_utf8_lossy(&brew_prefix.stdout);
                    let prefix = prefix_str.trim();
                    let lib_path = format!("{}/lib", prefix);
                    cmd.env("DYLD_LIBRARY_PATH", lib_path);
                }
            }
        }

        cmd
    }
}

#[cfg(feature = "krunvm")]
#[async_trait]
impl Backend for KrunvmBackend {
    async fn create(&self, vm: &VmInstance) -> Result<()> {
        let image_name = normalize_image_name(&vm.spec.image);

        let mut cmd = Self::krunvm_command();
        cmd.args(["create", &image_name]);
        cmd.arg("--name").arg(&vm.id);
        cmd.arg("--mem").arg(vm.spec.memory.to_string());
        cmd.arg("--cpus").arg(vm.spec.cpus.to_string());

        for (host_port, guest_port) in &vm.spec.ports {
            cmd.arg("--port")
                .arg(format!("{}:{}", host_port, guest_port));
        }

        for (host_path, guest_path) in &vm.spec.volumes {
            cmd.arg("-v")
                .arg(format!("{}:{}", host_path.display(), guest_path.display()));
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VortexError::VmError {
                message: format!("krunvm create failed: {}", stderr),
            });
        }

        Ok(())
    }

    async fn start(&self, vm: &VmInstance) -> Result<()> {
        let mut cmd = Self::krunvm_command();
        cmd.args(["start", &vm.id]);

        if let Some(command) = &vm.spec.command {
            cmd.arg("--");
            if command.contains("&&")
                || command.contains("||")
                || command.contains("|")
                || command.contains(";")
            {
                cmd.args(["sh", "-c", command]);
            } else {
                cmd.args(command.split_whitespace());
            }
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VortexError::VmError {
                message: format!("krunvm start failed: {}", stderr),
            });
        }

        Ok(())
    }

    async fn stop(&self, vm: &VmInstance) -> Result<()> {
        // krunvm doesn't have a separate stop, it's part of cleanup
        self.cleanup(vm).await
    }

    async fn cleanup(&self, vm: &VmInstance) -> Result<()> {
        let output = Self::krunvm_command()
            .args(["delete", &vm.id])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("krunvm delete failed (may already be deleted): {}", stderr);
        }

        Ok(())
    }

    async fn attach(&self, vm: &VmInstance) -> Result<()> {
        use std::process::Stdio;

        // Run shell with proper terminal setup
        let default_shell = "sh".to_string();
        let shell_command = vm.spec.command.as_ref().unwrap_or(&default_shell);

        let mut cmd = Self::krunvm_command();
        cmd.args([
            "start",
            &vm.id,
            "--",
            "sh",
            "-c",
            &format!("export TERM=vt100; stty sane; exec {}", shell_command),
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("TERM", "vt100");

        let mut child = cmd.spawn()?;
        let exit_status = child.wait().await?;

        // Handle normal shell exit conditions
        if let Some(code) = exit_status.code() {
            match code {
                0 => {
                    // Normal exit - success
                    Ok(())
                }
                130 => {
                    // SIGINT (Ctrl+C/Ctrl+D) - treat as normal user-initiated exit
                    Ok(())
                }
                129 => {
                    // SIGHUP - terminal disconnection, also normal
                    Ok(())
                }
                _ => {
                    // Other exit codes - still report as error for debugging
                    Err(VortexError::VmError {
                        message: format!("Interactive session ended with exit code: {}", code),
                    })
                }
            }
        } else {
            // Process was terminated by signal - could be normal depending on signal
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(signal) = exit_status.signal() {
                    match signal {
                        2 => Ok(()),  // SIGINT - normal Ctrl+C
                        15 => Ok(()), // SIGTERM - normal termination
                        _ => Err(VortexError::VmError {
                            message: format!(
                                "Interactive session terminated by signal: {}",
                                signal
                            ),
                        }),
                    }
                } else {
                    Ok(()) // Unknown termination, assume normal
                }
            }
            #[cfg(not(unix))]
            {
                Ok(()) // On non-Unix systems, assume normal termination
            }
        }
    }

    async fn get_metrics(&self, vm: &VmInstance) -> Result<VmMetrics> {
        // Get basic VM info from krunvm
        let output = Self::krunvm_command().args(["list"]).output().await?;

        if !output.status.success() {
            return Ok(VmMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                memory_total: (vm.spec.memory as u64) * 1024 * 1024,
                disk_usage: 0,
                network_rx: 0,
                network_tx: 0,
                uptime_seconds: 0,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut memory_mb = vm.spec.memory;
        let mut cpus = vm.spec.cpus;

        // Parse krunvm list output to get actual allocated resources
        let lines: Vec<&str> = stdout.lines().collect();
        let mut found_vm = false;

        for (i, line) in lines.iter().enumerate() {
            if line.trim() == vm.id {
                found_vm = true;
                // Look for CPU and RAM info in subsequent lines
                if let Some(cpu_line) = lines.get(i + 1) {
                    if cpu_line.contains("CPUs:") {
                        if let Some(cpu_str) = cpu_line.split("CPUs:").nth(1) {
                            if let Ok(parsed_cpus) = cpu_str.trim().parse::<u32>() {
                                cpus = parsed_cpus;
                            }
                        }
                    }
                }
                if let Some(ram_line) = lines.get(i + 2) {
                    if ram_line.contains("RAM (MiB):") {
                        if let Some(ram_str) = ram_line.split("RAM (MiB):").nth(1) {
                            if let Ok(parsed_ram) = ram_str.trim().parse::<u32>() {
                                memory_mb = parsed_ram;
                            }
                        }
                    }
                }
                break;
            }
        }

        if !found_vm {
            return Ok(VmMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                memory_total: (vm.spec.memory as u64) * 1024 * 1024,
                disk_usage: 0,
                network_rx: 0,
                network_tx: 0,
                uptime_seconds: 0,
            });
        }

        // Try to get system-level metrics for the VM process
        let memory_total = (memory_mb as u64) * 1024 * 1024;
        let estimated_memory_usage = memory_total / 2; // Rough estimate
        let estimated_cpu_usage = if cpus > 0 { 10.0 / cpus as f64 } else { 5.0 }; // Rough estimate

        Ok(VmMetrics {
            cpu_usage: estimated_cpu_usage,
            memory_usage: estimated_memory_usage,
            memory_total,
            disk_usage: 100 * 1024 * 1024, // Estimate 100MB disk usage
            network_rx: 1024,              // Small amounts for basic network activity
            network_tx: 512,
            uptime_seconds: 30, // Rough estimate - would need to track creation time
        })
    }

    async fn list_vms(&self) -> Result<Vec<String>> {
        let output = Self::krunvm_command().arg("list").output().await?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let vm_names: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if !line.is_empty()
                    && !line.contains("CPUs:")
                    && !line.contains("RAM")
                    && !line.contains("DNS")
                    && !line.contains("Buildah")
                    && !line.contains("Workdir")
                    && !line.contains("Mapped")
                {
                    Some(line.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(vm_names)
    }

    async fn is_available(&self) -> Result<bool> {
        // Try to run krunvm --help to check if it's available and working
        let output = Self::krunvm_command().arg("--help").output().await?;

        Ok(output.status.success())
    }

    fn name(&self) -> &'static str {
        "krunvm"
    }
}

// Firecracker Backend (placeholder)
#[cfg(feature = "firecracker")]
#[derive(Debug)]
pub struct FirecrackerBackend;

#[cfg(feature = "firecracker")]
impl FirecrackerBackend {
    pub async fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(feature = "firecracker")]
#[async_trait]
impl Backend for FirecrackerBackend {
    async fn create(&self, _vm: &VmInstance) -> Result<()> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn start(&self, _vm: &VmInstance) -> Result<()> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn stop(&self, _vm: &VmInstance) -> Result<()> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn cleanup(&self, _vm: &VmInstance) -> Result<()> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn attach(&self, _vm: &VmInstance) -> Result<()> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn get_metrics(&self, _vm: &VmInstance) -> Result<VmMetrics> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn list_vms(&self) -> Result<Vec<String>> {
        Err(VortexError::VmError {
            message: "Firecracker backend not yet implemented".to_string(),
        })
    }

    async fn is_available(&self) -> Result<bool> {
        use tokio::process::Command;

        let output = Command::new("which").arg("firecracker").output().await?;

        Ok(output.status.success())
    }

    fn name(&self) -> &'static str {
        "firecracker"
    }
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
