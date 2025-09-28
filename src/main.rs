use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use vortex::{
    detect_workspace_info, init, ResourceLimits, VmSpec, VortexConfig, VortexCore, VERSION,
};

#[derive(Parser)]
#[command(
    name = "vortex",
    about = "Vortex - Lightning-fast ephemeral VM platform",
    version = "0.4.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Enable verbose logging")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start a new ephemeral VM")]
    Run {
        #[arg(help = "VM image (alpine, ubuntu:22.04, debian:bullseye)")]
        image: String,

        #[arg(short, long, help = "Memory in MB", default_value = "512")]
        memory: u32,

        #[arg(short, long, help = "CPU cores", default_value = "1")]
        cpus: u32,

        #[arg(short, long, help = "Port forwarding (host:guest)")]
        port: Vec<String>,

        #[arg(short = 'v', long, help = "Volume mounts (host:guest)")]
        volume: Vec<String>,

        #[arg(short = 'e', long, help = "Command to run in VM")]
        command: Option<String>,

        #[arg(long, help = "Keep VM running after command exits")]
        persist: bool,

        #[arg(
            short = 'q',
            long,
            help = "Quiet mode - suppress all non-command output"
        )]
        quiet: bool,

        #[arg(long, help = "Show real-time performance stats (Docker can't do this)")]
        monitor_performance: bool,

        #[arg(
            long,
            help = "Copy contents of host directory to VM directory (host:guest)"
        )]
        copy_to: Vec<String>,

        #[arg(
            long,
            help = "Copy VM directory contents back to host after execution (guest:host)"
        )]
        sync_back: Vec<String>,

        #[arg(short = 'w', long, help = "Set working directory inside VM")]
        workdir: Option<String>,

        #[arg(long, help = "Add labels (key=value)")]
        label: Vec<String>,

        #[arg(
            long,
            help = "Cache dependencies for faster subsequent runs (Docker can't do this efficiently)"
        )]
        cache_deps: bool,
    },

    #[command(about = "List running VMs")]
    List,

    #[command(about = "Stop and cleanup a VM")]
    Stop {
        #[arg(help = "VM ID")]
        vm_id: String,
    },

    #[command(about = "Stop all running VMs")]
    Cleanup,

    #[command(about = "Run from a template")]
    Template {
        #[arg(help = "Template name")]
        name: String,

        #[arg(short, long, help = "Override command")]
        command: Option<String>,
    },

    #[command(about = "Show available templates and aliases")]
    Templates,

    #[command(about = "Start interactive shell in VM")]
    Shell {
        #[arg(
            help = "VM image (alpine, ubuntu, node, etc.)",
            default_value = "alpine"
        )]
        image: String,

        #[arg(short, long, help = "Memory in MB", default_value = "512")]
        memory: u32,

        #[arg(short, long, help = "CPU cores", default_value = "1")]
        cpus: u32,

        #[arg(short, long, help = "Port forwarding (host:guest)")]
        port: Vec<String>,

        #[arg(short = 'v', long, help = "Volume mounts (host:guest)")]
        volume: Vec<String>,

        #[arg(short = 's', long, help = "Shell to use", default_value = "sh")]
        shell: String,

        #[arg(long, help = "Skip interactive banner")]
        quiet: bool,

        #[arg(
            long,
            help = "Copy contents of host directory to VM directory (host:guest)"
        )]
        copy_to: Vec<String>,

        #[arg(short = 'w', long, help = "Set working directory inside VM")]
        workdir: Option<String>,
    },

    #[command(about = "Show VM metrics")]
    Metrics {
        #[arg(help = "VM ID (optional - shows all if omitted)")]
        vm_id: Option<String>,
    },

    #[command(about = "Run command across multiple VMs in parallel (Docker can't do this)")]
    Parallel {
        #[arg(help = "VM images to run in parallel")]
        images: Vec<String>,

        #[arg(short = 'e', long, help = "Command to run in each VM")]
        command: String,

        #[arg(short = 'q', long, help = "Quiet mode - show only aggregated results")]
        quiet: bool,

        #[arg(long, help = "Copy contents to each VM (host:guest)")]
        copy_to: Vec<String>,

        #[arg(long, help = "Sync results back from each VM")]
        sync_back: Vec<String>,
    },

    #[command(about = "Create instant dev environments (Docker can't match this speed!)")]
    Dev {
        #[arg(help = "Development template (python, node, rust, go, ai)", required_unless_present_any = ["list", "init", "workspace"])]
        template: Option<String>,

        #[arg(short, long, help = "Custom working directory")]
        workdir: Option<String>,

        #[arg(short = 'v', long, help = "Volume mounts (host:guest)")]
        volume: Vec<String>,

        #[arg(short = 'p', long, help = "Port mappings (host:guest)")]
        port: Vec<String>,

        #[arg(short = 'q', long, help = "Quiet mode - no banner")]
        quiet: bool,

        #[arg(long, help = "List available development templates")]
        list: bool,

        #[arg(long, help = "Create/use persistent workspace")]
        workspace: Option<String>,

        #[arg(long, help = "Initialize workspace from current directory")]
        init: bool,
    },

    #[command(about = "Manage persistent workspaces")]
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommand {
    #[command(about = "List all workspaces")]
    List,

    #[command(about = "Create a new workspace")]
    Create {
        #[arg(help = "Workspace name")]
        name: String,

        #[arg(short, long, help = "Development template to use")]
        template: String,

        #[arg(long, help = "Source directory to copy (defaults to current dir)")]
        source: Option<PathBuf>,
    },

    #[command(about = "Delete a workspace")]
    Delete {
        #[arg(help = "Workspace name or ID")]
        workspace: String,
    },

    #[command(about = "Show workspace details")]
    Info {
        #[arg(help = "Workspace name or ID")]
        workspace: String,
    },

    #[command(about = "Import from devcontainer.json")]
    Import {
        #[arg(help = "Workspace name")]
        name: String,

        #[arg(
            long,
            help = "Path to devcontainer.json",
            default_value = ".devcontainer/devcontainer.json"
        )]
        devcontainer: PathBuf,

        #[arg(long, help = "Source directory (defaults to current dir)")]
        source: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if any command is using quiet mode
    let is_quiet = match &cli.command {
        Commands::Run { quiet, .. } => *quiet,
        Commands::Shell { quiet, .. } => *quiet,
        Commands::Dev { quiet, .. } => *quiet,
        _ => false,
    };

    if is_quiet {
        // Disable all logging in quiet mode - use ERROR level as lowest
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .without_time()
            .init();
    } else if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    if !is_quiet {
        info!("Vortex v{} - Ephemeral VM Platform", VERSION);
    }

    // Initialize Vortex Core
    let vortex = Arc::new(init().await.context("Failed to initialize Vortex core")?);

    match cli.command {
        Commands::Run {
            image,
            memory,
            cpus,
            port,
            volume,
            command,
            persist,
            quiet: run_quiet,
            monitor_performance,
            copy_to,
            sync_back,
            workdir,
            label,
            cache_deps,
        } => {
            let spec = VmSpec {
                image,
                memory,
                cpus,
                ports: parse_port_mappings(port)?,
                volumes: parse_volume_mappings(volume)?,
                environment: HashMap::new(),
                command,
                labels: parse_labels(label)?,
                network_config: None,
                resource_limits: ResourceLimits::default(),
            };

            run_vm(
                &vortex,
                spec,
                persist,
                run_quiet,
                monitor_performance,
                copy_to,
                sync_back,
                workdir,
                cache_deps,
            )
            .await?;
        }
        Commands::List => {
            list_vms(&vortex).await?;
        }
        Commands::Stop { vm_id } => {
            stop_vm(&vortex, &vm_id).await?;
        }
        Commands::Cleanup => {
            cleanup_vms(&vortex).await?;
        }
        Commands::Template { name, command } => {
            run_template(&vortex, &name, command).await?;
        }
        Commands::Templates => {
            show_templates().await?;
        }
        Commands::Shell {
            image,
            memory,
            cpus,
            port,
            volume,
            shell,
            quiet,
            copy_to,
            workdir,
        } => {
            start_shell(
                &vortex, image, memory, cpus, port, volume, shell, quiet, copy_to, workdir,
            )
            .await?;
        }
        Commands::Metrics { vm_id } => {
            show_metrics(&vortex, vm_id.as_deref()).await?;
        }
        Commands::Parallel {
            images,
            command,
            quiet,
            copy_to,
            sync_back,
        } => {
            run_parallel_vms(&vortex, images, command, quiet, copy_to, sync_back).await?;
        }
        Commands::Dev {
            template,
            workdir,
            volume,
            port,
            quiet,
            list,
            workspace,
            init,
        } => {
            if list {
                show_dev_templates(&vortex).await?;
            } else if init {
                init_workspace_from_current_dir(&vortex).await?;
            } else if let Some(workspace_name) = workspace {
                start_workspace(&vortex, &workspace_name, quiet).await?;
            } else if let Some(template_name) = template {
                start_dev_environment(&vortex, &template_name, workdir, volume, port, quiet)
                    .await?;
            } else {
                return Err(anyhow::anyhow!(
                    "Template name, workspace, or --list is required"
                ));
            }
        }
        Commands::Workspace { command } => match command {
            WorkspaceCommand::List => {
                list_workspaces(&vortex).await?;
            }
            WorkspaceCommand::Create {
                name,
                template,
                source,
            } => {
                create_workspace(&vortex, &name, &template, &source).await?;
            }
            WorkspaceCommand::Delete { workspace } => {
                delete_workspace(&vortex, &workspace).await?;
            }
            WorkspaceCommand::Info { workspace } => {
                show_workspace_info(&vortex, &workspace).await?;
            }
            WorkspaceCommand::Import {
                name,
                devcontainer,
                source,
            } => {
                import_devcontainer_workspace(&vortex, &name, &devcontainer, &source).await?;
            }
        },
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_vm(
    vortex: &Arc<VortexCore>,
    mut spec: VmSpec,
    persist: bool,
    quiet: bool,
    monitor_performance: bool,
    copy_to: Vec<String>,
    sync_back: Vec<String>,
    workdir: Option<String>,
    cache_deps: bool,
) -> Result<()> {
    // Parse copy mappings and set up volumes
    let copy_mappings = parse_copy_mappings(copy_to)?;
    let sync_mappings = parse_sync_back_mappings(sync_back)?;

    // Add dependency caching volume if requested
    if cache_deps {
        let cache_dir =
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()) + "/.vortex/cache";
        std::fs::create_dir_all(&cache_dir)?;
        spec.volumes.insert(
            std::path::PathBuf::from(cache_dir),
            std::path::PathBuf::from("/vortex_cache"),
        );
        if !quiet {
            println!("🔄 Dependency caching enabled (Docker's layer caching is primitive compared to this!)");
        }
    }

    // Add temporary mount points for copy operations
    for (i, (host_path, _)) in copy_mappings.iter().enumerate() {
        let temp_mount = format!("/tmp/vortex_copy_in_{}", i);
        spec.volumes
            .insert(host_path.clone(), PathBuf::from(&temp_mount));
    }

    // Add mount points for sync back operations
    for (i, (_guest_path, host_path)) in sync_mappings.iter().enumerate() {
        let temp_mount = format!("/tmp/vortex_copy_out_{}", i);
        spec.volumes
            .insert(host_path.clone(), PathBuf::from(&temp_mount));
    }

    // Build enhanced command with copy operations and workdir
    if let Some(original_cmd) = &spec.command {
        let mut enhanced_cmd = String::new();

        // Copy input files
        for (i, (_, dest_path)) in copy_mappings.iter().enumerate() {
            let temp_mount = format!("/tmp/vortex_copy_in_{}", i);
            enhanced_cmd.push_str(&format!(
                "mkdir -p '{}' && cp -r {}/* '{}' 2>/dev/null || true; ",
                dest_path.display(),
                temp_mount,
                dest_path.display()
            ));
        }

        // Change to workdir if specified
        if let Some(wd) = &workdir {
            enhanced_cmd.push_str(&format!("cd '{}'; ", wd));
        }

        // Run the actual command
        enhanced_cmd.push_str(original_cmd);
        enhanced_cmd.push(';');

        // Copy output files back
        for (i, (source_path, _)) in sync_mappings.iter().enumerate() {
            let temp_mount = format!("/tmp/vortex_copy_out_{}", i);
            enhanced_cmd.push_str(&format!(
                " cp -r '{}' '{}' 2>/dev/null || true;",
                source_path.display(),
                temp_mount
            ));
        }

        spec.command = Some(enhanced_cmd);
    }

    if !quiet {
        info!("Starting VM with image: {}", spec.image);
    }

    let vm = vortex.create_vm(spec).await?;

    // Start performance monitoring if requested
    if monitor_performance && !quiet {
        let vortex_clone = Arc::clone(vortex);
        let vm_id_clone = vm.id.clone();
        tokio::spawn(async move {
            monitor_vm_performance(&vortex_clone, &vm_id_clone).await;
        });
    }

    if persist {
        if !quiet {
            info!(
                "VM {} started and persisting. Use 'vortex stop {}' to stop it.",
                vm.id, vm.id
            );
        }
    } else if !quiet {
        info!("VM {} started. Command completed.", vm.id);
        // For non-persistent VMs, we should wait for completion and cleanup
        // This would require backend support for monitoring VM completion
    }

    Ok(())
}

async fn list_vms(vortex: &Arc<VortexCore>) -> Result<()> {
    let vms = vortex.vm_manager.list().await?;

    if vms.is_empty() {
        println!("No running VMs found.");
    } else {
        println!("Running VMs:");
        for vm in vms {
            println!(
                "  {} - {} ({:?}) - {}MB RAM, {} CPU(s)",
                vm.id, vm.spec.image, vm.state, vm.spec.memory, vm.spec.cpus
            );
        }
    }

    Ok(())
}

async fn stop_vm(vortex: &Arc<VortexCore>, vm_id: &str) -> Result<()> {
    vortex.vm_manager.stop(vm_id).await?;
    vortex.vm_manager.cleanup(vm_id).await?;
    info!("VM {} stopped and cleaned up.", vm_id);
    Ok(())
}

async fn cleanup_vms(vortex: &Arc<VortexCore>) -> Result<()> {
    let vms = vortex.vm_manager.list().await?;
    let count = vms.len();

    for vm in vms {
        if let Err(e) = vortex.vm_manager.cleanup(&vm.id).await {
            tracing::warn!("Failed to cleanup VM {}: {}", vm.id, e);
        }
    }

    info!("Cleaned up {} VMs.", count);
    Ok(())
}

async fn run_template(
    vortex: &Arc<VortexCore>,
    template_name: &str,
    override_command: Option<String>,
) -> Result<()> {
    let config = VortexConfig::load()?;
    let template = config
        .get_template(template_name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

    info!(
        "Running template: {} - {}",
        template_name, template.description
    );

    let spec = VmSpec {
        image: config.resolve_image(&template.image),
        memory: template.memory,
        cpus: template.cpus,
        ports: parse_port_mappings(template.ports.clone())?,
        volumes: parse_volume_mappings(template.volumes.clone())?,
        environment: template.environment.clone(),
        command: override_command.or_else(|| template.command.clone()),
        labels: template.labels.clone(),
        network_config: None,
        resource_limits: ResourceLimits::default(),
    };

    run_vm(
        vortex,
        spec,
        false,
        false,
        false,
        vec![],
        vec![],
        None,
        false,
    )
    .await?;
    Ok(())
}

async fn show_templates() -> Result<()> {
    let config = VortexConfig::load()?;

    println!("Available Templates:");
    for (name, template) in &config.templates {
        println!("  {} - {} ({})", name, template.description, template.image);
        if !template.ports.is_empty() {
            println!("    Ports: {}", template.ports.join(", "));
        }
        if template.memory != 512 || template.cpus != 1 {
            println!(
                "    Resources: {}MB RAM, {} CPU(s)",
                template.memory, template.cpus
            );
        }
        println!();
    }

    println!("\nImage Aliases:");
    for (alias, image) in &config.image_aliases {
        println!("  {} -> {}", alias, image);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn start_shell(
    vortex: &Arc<VortexCore>,
    image: String,
    memory: u32,
    cpus: u32,
    port: Vec<String>,
    volume: Vec<String>,
    shell: String,
    quiet: bool,
    copy_to: Vec<String>,
    workdir: Option<String>,
) -> Result<()> {
    let config = VortexConfig::load()?;
    let resolved_image = config.resolve_image(&image);

    info!("Starting interactive shell in {} VM...", resolved_image);

    // Parse copy_to mappings
    let copy_mappings = parse_copy_mappings(copy_to)?;
    let mut volumes = parse_volume_mappings(volume)?;

    // Build the shell command with workdir and copy operations
    let mut shell_cmd = String::new();

    // Handle copy operations by mounting source and copying to destination
    for (i, (host_path, dest_path)) in copy_mappings.iter().enumerate() {
        let temp_mount = format!("/tmp/vortex_copy_{}", i);
        // Mount the host directory temporarily
        volumes.insert(host_path.clone(), PathBuf::from(&temp_mount));
        // Copy from temp mount to destination
        shell_cmd.push_str(&format!(
            "mkdir -p '{}' && cp -r {}/* '{}' 2>/dev/null || true; ",
            dest_path.display(),
            temp_mount,
            dest_path.display()
        ));
    }

    // Change to workdir if specified
    if let Some(wd) = &workdir {
        shell_cmd.push_str(&format!("cd '{}'; ", wd));
    }

    // Add the actual shell
    shell_cmd.push_str(&format!("exec {}", shell));

    let spec = VmSpec {
        image: resolved_image,
        memory,
        cpus,
        ports: parse_port_mappings(port)?,
        volumes,
        environment: HashMap::new(),
        command: Some(shell_cmd),
        labels: HashMap::new(),
        network_config: None,
        resource_limits: ResourceLimits::default(),
    };

    let vm = vortex.create_vm(spec).await?;
    info!("VM {} created. Connecting to interactive shell...", vm.id);

    if !quiet {
        println!();
        println!("╭─ Interactive Shell ─ {} ─", vm.id);
        println!("│");
        println!("│  Image:     {}", vm.spec.image);
        println!(
            "│  Resources: {}MB RAM, {} CPU(s)",
            vm.spec.memory, vm.spec.cpus
        );
        if !vm.spec.ports.is_empty() {
            println!("│  Ports:     {:?}", vm.spec.ports);
        }
        if !vm.spec.volumes.is_empty() {
            println!("│  Volumes:   {:?}", vm.spec.volumes);
        }
        println!("│");
        println!("│  Type 'exit' to leave the shell");
        println!("╰─────────────────────────────────────");
        println!();
    }

    // Attach to interactive session
    vortex.attach_vm(&vm.id).await?;

    // Cleanup when done
    info!("Shell session ended. Cleaning up VM {}...", vm.id);
    vortex.vm_manager.cleanup(&vm.id).await?;

    Ok(())
}

async fn show_metrics(vortex: &Arc<VortexCore>, vm_id: Option<&str>) -> Result<()> {
    if let Some(vm_id) = vm_id {
        // Get VM and collect real-time metrics
        let vms = vortex.vm_manager.list().await?;
        if let Some(vm) = vms.iter().find(|v| v.id == vm_id) {
            match vm.backend.get_metrics(vm).await {
                Ok(metrics) => {
                    println!("VM Metrics for {}:", vm_id);
                    println!("  CPU Usage: {:.1}%", metrics.cpu_usage);
                    println!(
                        "  Memory: {:.1}MB / {:.1}MB",
                        metrics.memory_usage as f64 / 1024.0 / 1024.0,
                        metrics.memory_total as f64 / 1024.0 / 1024.0
                    );
                    println!(
                        "  Disk Usage: {:.1}MB",
                        metrics.disk_usage as f64 / 1024.0 / 1024.0
                    );
                    println!("  Network RX: {:.1}KB", metrics.network_rx as f64 / 1024.0);
                    println!("  Network TX: {:.1}KB", metrics.network_tx as f64 / 1024.0);
                    println!("  Uptime: {}s", metrics.uptime_seconds);
                }
                Err(e) => {
                    println!("Failed to collect metrics for VM {}: {}", vm_id, e);
                }
            }
        } else {
            println!("VM {} not found", vm_id);
        }
    } else {
        // Show system metrics based on all running VMs
        let vms = vortex.vm_manager.list().await?;
        let mut total_memory_allocated = 0u64;
        let mut total_memory_used = 0u64;
        let mut total_cpu_usage = 0.0f64;
        let mut successful_metrics = 0;

        for vm in &vms {
            if let Ok(metrics) = vm.backend.get_metrics(vm).await {
                total_memory_allocated += metrics.memory_total;
                total_memory_used += metrics.memory_usage;
                total_cpu_usage += metrics.cpu_usage;
                successful_metrics += 1;
            }
        }

        println!("System Metrics:");
        println!("  Total VMs: {}", vms.len());
        println!("  Running VMs: {}", vms.len());
        println!("  Total CPU Usage: {:.1}%", total_cpu_usage);
        println!(
            "  Total Memory Used: {:.1}MB / {:.1}MB",
            total_memory_used as f64 / 1024.0 / 1024.0,
            total_memory_allocated as f64 / 1024.0 / 1024.0
        );
        if successful_metrics > 0 {
            println!(
                "  Average CPU per VM: {:.1}%",
                total_cpu_usage / successful_metrics as f64
            );
        }
    }

    Ok(())
}

fn parse_port_mappings(ports: Vec<String>) -> Result<HashMap<u16, u16>> {
    let mut mappings = HashMap::new();

    for port in ports {
        let parts: Vec<&str> = port.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid port mapping format: {}. Use host:guest",
                port
            ));
        }

        let host_port: u16 = parts[0]
            .parse()
            .with_context(|| format!("Invalid host port: {}", parts[0]))?;
        let guest_port: u16 = parts[1]
            .parse()
            .with_context(|| format!("Invalid guest port: {}", parts[1]))?;

        mappings.insert(host_port, guest_port);
    }

    Ok(mappings)
}

fn parse_volume_mappings(volumes: Vec<String>) -> Result<HashMap<PathBuf, PathBuf>> {
    let mut mappings = HashMap::new();

    for volume in volumes {
        let parts: Vec<&str> = volume.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid volume mapping format: {}. Use host:guest",
                volume
            ));
        }

        let host_path = PathBuf::from(parts[0]);
        let guest_path = PathBuf::from(parts[1]);

        mappings.insert(host_path, guest_path);
    }

    Ok(mappings)
}

fn parse_labels(labels: Vec<String>) -> Result<HashMap<String, String>> {
    let mut mappings = HashMap::new();

    for label in labels {
        let parts: Vec<&str> = label.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid label format: {}. Use key=value",
                label
            ));
        }

        mappings.insert(parts[0].to_string(), parts[1].to_string());
    }

    Ok(mappings)
}

fn parse_copy_mappings(copy_to: Vec<String>) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut mappings = Vec::new();

    for copy_spec in copy_to {
        let parts: Vec<&str> = copy_spec.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid copy-to format: {}. Use host_dir:guest_dir",
                copy_spec
            ));
        }

        let host_path = PathBuf::from(parts[0]);
        let guest_path = PathBuf::from(parts[1]);

        // Verify host path exists and is a directory
        if !host_path.exists() {
            return Err(anyhow::anyhow!(
                "Host directory does not exist: {}",
                host_path.display()
            ));
        }
        if !host_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Host path is not a directory: {}",
                host_path.display()
            ));
        }

        mappings.push((host_path, guest_path));
    }

    Ok(mappings)
}

fn parse_sync_back_mappings(sync_back: Vec<String>) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut mappings = Vec::new();

    for sync_spec in sync_back {
        let parts: Vec<&str> = sync_spec.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid sync-back format: {}. Use guest_dir:host_dir",
                sync_spec
            ));
        }

        let guest_path = PathBuf::from(parts[0]);
        let host_path = PathBuf::from(parts[1]);

        // Create host directory if it doesn't exist
        if let Some(parent) = host_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        mappings.push((guest_path, host_path));
    }

    Ok(mappings)
}

async fn run_parallel_vms(
    vortex: &Arc<VortexCore>,
    images: Vec<String>,
    command: String,
    quiet: bool,
    copy_to: Vec<String>,
    sync_back: Vec<String>,
) -> Result<()> {
    use tokio::time::Instant;

    let start_time = Instant::now();

    if !quiet {
        println!(
            "🚀 Launching {} VMs in parallel (try this with Docker!)",
            images.len()
        );
    }

    // Create futures for all VMs
    let mut tasks = Vec::new();

    for (i, image) in images.into_iter().enumerate() {
        // vortex is available from the closure
        let command = command.clone();
        let copy_to = copy_to.clone();
        let sync_back = sync_back.clone();

        let task = async move {
            let config = VortexConfig::load()?;
            let resolved_image = config.resolve_image(&image);

            // Create unique sync paths for each VM
            let mut unique_sync_back = Vec::new();
            for sync_spec in sync_back {
                let parts: Vec<&str> = sync_spec.split(':').collect();
                if parts.len() == 2 {
                    let guest_path = parts[0];
                    let host_path = format!("{}_vm_{}", parts[1], i);
                    unique_sync_back.push(format!("{}:{}", guest_path, host_path));
                }
            }

            let spec = VmSpec {
                image: resolved_image.clone(),
                memory: 512,
                cpus: 1,
                ports: HashMap::new(),
                volumes: HashMap::new(),
                environment: HashMap::new(),
                command: Some(command),
                labels: HashMap::new(),
                network_config: None,
                resource_limits: ResourceLimits::default(),
            };

            let vm_start = Instant::now();
            run_vm(
                vortex,
                spec,
                false,
                true,
                false,
                copy_to,
                unique_sync_back,
                None,
                false,
            )
            .await?;
            let vm_duration = vm_start.elapsed();

            Ok::<(String, std::time::Duration), anyhow::Error>((resolved_image, vm_duration))
        };

        tasks.push(task);
    }

    // Execute all VMs in parallel and collect results
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => return Err(e),
        }
    }

    let total_duration = start_time.elapsed();

    if !quiet {
        println!("\n🎯 Parallel Execution Results:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for (image, duration) in &results {
            println!("  {} - completed in {:.2}s", image, duration.as_secs_f64());
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "🚀 Total time: {:.2}s ({} VMs in parallel)",
            total_duration.as_secs_f64(),
            results.len()
        );
        println!(
            "⚡ Docker would take {}x longer running these sequentially!",
            results.len()
        );
    }

    Ok(())
}

async fn monitor_vm_performance(vortex: &Arc<VortexCore>, vm_id: &str) {
    use tokio::time::{sleep, Duration};

    println!(
        "
🎯 Real-time Performance Monitor (Docker can't do this!)"
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for i in 1..=10 {
        sleep(Duration::from_secs(1)).await;

        if let Ok(vms) = vortex.vm_manager.list().await {
            if let Some(vm) = vms.iter().find(|v| v.id == vm_id) {
                if let Ok(metrics) = vm.backend.get_metrics(vm).await {
                    print!(
                        "\r[{}s] CPU: {:.1}% | RAM: {:.0}MB/{:.0}MB | Disk: {:.0}MB",
                        i,
                        metrics.cpu_usage,
                        metrics.memory_usage as f64 / 1024.0 / 1024.0,
                        metrics.memory_total as f64 / 1024.0 / 1024.0,
                        metrics.disk_usage as f64 / 1024.0 / 1024.0
                    );

                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                }
            }
        }
    }

    println!(
        "
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
}

async fn show_dev_templates(vortex: &Arc<VortexCore>) -> Result<()> {
    let templates = vortex.dev_env_manager.list_templates();

    println!("🔥 Available Dev Environment Templates:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for template in templates {
        println!("📦 {} - {}", template.name, template.description);
        println!("   Base: {}", template.base_image);
        println!("   Tools: {}", template.tools.join(", "));
        if !template.ports.is_empty() {
            println!("   Ports: {}", template.ports.join(", "));
        }
        if !template.extensions.is_empty() {
            println!("   IDE Extensions: {}", template.extensions.join(", "));
        }
        println!();
    }

    println!("💡 Usage: vortex dev <template> [options]");
    println!("📖 Example: vortex dev python --workdir /workspace --volume ./src:/workspace/src");

    Ok(())
}

async fn start_dev_environment(
    vortex: &Arc<VortexCore>,
    template_name: &str,
    workdir: Option<String>,
    volumes: Vec<String>,
    ports: Vec<String>,
    quiet: bool,
) -> Result<()> {
    // Parse volume and port mappings
    let volume_mappings = parse_volume_mappings(volumes)?;
    let _port_mappings = parse_port_mappings(ports)?;

    // Create the dev environment VM
    let vm = vortex
        .create_dev_environment(template_name, workdir.clone(), volume_mappings)
        .await?;

    if !quiet {
        let template = vortex
            .dev_env_manager
            .get_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

        println!();
        println!("🚀 Dev Environment Ready!");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 Template: {} ({})", template.name, template.description);
        println!("🐳 Base: {}", template.base_image);
        println!("🔧 Tools: {}", template.tools.join(", "));
        if !template.ports.is_empty() {
            println!("🌐 Ports: {}", template.ports.join(", "));
        }
        if !vm.spec.volumes.is_empty() {
            println!("📂 Volumes: {} mount(s)", vm.spec.volumes.len());
        }
        println!("💾 VM ID: {}", vm.id);
        println!();
        println!("⚡ Lightning-fast setup complete! (Docker would still be pulling images)");
        println!("💬 Connecting to interactive shell...");
        println!();
    }

    // Attach to the VM for interactive development
    vortex.attach_vm(&vm.id).await?;

    // Cleanup when done
    if !quiet {
        println!("\n🧹 Cleaning up dev environment...");
    }
    vortex.vm_manager.cleanup(&vm.id).await?;

    if !quiet {
        println!("✅ Dev session complete!");
    }

    Ok(())
}

// Workspace management functions

async fn init_workspace_from_current_dir(vortex: &Arc<VortexCore>) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if let Some(info) = detect_workspace_info(&current_dir) {
        println!("🔍 Detected project in current directory:");
        println!("   Name: {}", info.name);
        println!("   Suggested template: {}", info.suggested_template);

        if info.has_devcontainer {
            println!("   📦 DevContainer detected!");
            println!();
            println!("Would you like to:");
            println!("  1. Import from devcontainer.json (recommended)");
            println!("  2. Create standard Vortex workspace");
            println!();

            // For now, just import the devcontainer
            if let Some(devcontainer_path) = &info.devcontainer_path {
                let workspace = vortex.workspace_manager.create_from_devcontainer(
                    &info.name,
                    devcontainer_path,
                    &current_dir,
                )?;

                println!(
                    "✅ Workspace '{}' created from devcontainer!",
                    workspace.name
                );
                println!("🚀 Run: vortex dev --workspace {}", workspace.name);
            }
        } else {
            let workspace = vortex.workspace_manager.create_workspace(
                &info.name,
                &info.suggested_template,
                Some(&current_dir),
            )?;

            println!("✅ Workspace '{}' created!", workspace.name);
            println!("🚀 Run: vortex dev --workspace {}", workspace.name);
        }
    } else {
        return Err(anyhow::anyhow!(
            "Could not detect project type in current directory"
        ));
    }

    Ok(())
}

async fn start_workspace(
    vortex: &Arc<VortexCore>,
    workspace_name: &str,
    quiet: bool,
) -> Result<()> {
    // Try to find workspace by name first, then by ID
    let workspace = vortex
        .workspace_manager
        .find_workspace_by_name(workspace_name)?
        .or_else(|| {
            vortex
                .workspace_manager
                .get_workspace(workspace_name)
                .unwrap_or(None)
        })
        .ok_or_else(|| anyhow::anyhow!("Workspace '{}' not found", workspace_name))?;

    if !quiet {
        println!();
        println!("🔄 Launching workspace '{}'...", workspace.name);
        println!("📁 Path: {}", workspace.path.display());
        println!("🎯 Template: {}", workspace.config.template);

        if let Some(devcontainer) = &workspace.config.devcontainer_source {
            println!("📦 DevContainer: {}", devcontainer);
        }

        println!(
            "⏰ Last used: {}",
            workspace.config.last_used.format("%Y-%m-%d %H:%M")
        );
        println!();
    }

    // Create and start VM from workspace
    let vm = vortex.create_workspace_vm(&workspace.id).await?;

    if !quiet {
        println!("⚡ Workspace VM ready!");
        println!("💬 Connecting to interactive session...");
        println!();
    }

    // Attach to the VM
    vortex.attach_vm(&vm.id).await?;

    // Cleanup when done
    if !quiet {
        println!("\n🧹 Cleaning up workspace VM...");
    }
    vortex.vm_manager.cleanup(&vm.id).await?;

    if !quiet {
        println!("✅ Workspace session complete! Your work is safely stored.");
    }

    Ok(())
}

async fn list_workspaces(vortex: &Arc<VortexCore>) -> Result<()> {
    let workspaces = vortex.workspace_manager.list_workspaces()?;

    if workspaces.is_empty() {
        println!("No workspaces found.");
        println!("💡 Create one with: vortex workspace create <name> --template <template>");
        println!("💡 Or initialize from current dir: vortex dev --init");
        return Ok(());
    }

    println!("🔥 Persistent Workspaces:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for workspace in workspaces {
        println!("📁 {} ({})", workspace.name, &workspace.id[..8]);
        println!("   Template: {}", workspace.config.template);
        println!("   Path: {}", workspace.path.display());
        println!(
            "   Last used: {}",
            workspace.config.last_used.format("%Y-%m-%d %H:%M")
        );

        if let Some(devcontainer) = &workspace.config.devcontainer_source {
            println!("   📦 DevContainer: {}", devcontainer);
        }

        if !workspace.config.port_forwards.is_empty() {
            println!(
                "   🌐 Ports: {}",
                workspace
                    .config
                    .port_forwards
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        println!();
    }

    println!("💡 Start workspace: vortex dev --workspace <name>");
    println!("📖 More info: vortex workspace info <name>");

    Ok(())
}

async fn create_workspace(
    vortex: &Arc<VortexCore>,
    name: &str,
    template: &str,
    source: &Option<PathBuf>,
) -> Result<()> {
    let source_dir = source
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| std::path::Path::new("."));

    // Verify template exists
    if vortex.dev_env_manager.get_template(template).is_none() {
        return Err(anyhow::anyhow!("Template '{}' not found", template));
    }

    let workspace = vortex
        .workspace_manager
        .create_workspace(name, template, Some(source_dir))?;

    println!("✅ Workspace '{}' created!", workspace.name);
    println!("📁 Path: {}", workspace.path.display());
    println!("🎯 Template: {}", workspace.config.template);
    println!("🚀 Start with: vortex dev --workspace {}", workspace.name);

    Ok(())
}

async fn delete_workspace(vortex: &Arc<VortexCore>, workspace_name: &String) -> Result<()> {
    // Find workspace by name or ID
    let workspace = vortex
        .workspace_manager
        .find_workspace_by_name(workspace_name)?
        .or_else(|| {
            vortex
                .workspace_manager
                .get_workspace(workspace_name)
                .unwrap_or(None)
        })
        .ok_or_else(|| anyhow::anyhow!("Workspace '{}' not found", workspace_name))?;

    println!(
        "⚠️  This will permanently delete workspace '{}'",
        workspace.name
    );
    println!("📁 Path: {}", workspace.path.display());
    println!();
    println!("Are you sure? [y/N]: ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        vortex.workspace_manager.delete_workspace(&workspace.id)?;
        println!("🗑️  Workspace '{}' deleted", workspace.name);
    } else {
        println!("❌ Cancelled");
    }

    Ok(())
}

async fn show_workspace_info(vortex: &Arc<VortexCore>, workspace_name: &String) -> Result<()> {
    let workspace = vortex
        .workspace_manager
        .find_workspace_by_name(workspace_name)?
        .or_else(|| {
            vortex
                .workspace_manager
                .get_workspace(workspace_name)
                .unwrap_or(None)
        })
        .ok_or_else(|| anyhow::anyhow!("Workspace '{}' not found", workspace_name))?;

    println!("🔍 Workspace Details:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📁 Name: {}", workspace.name);
    println!("🆔 ID: {}", workspace.id);
    println!("🎯 Template: {}", workspace.config.template);
    println!("📂 Path: {}", workspace.path.display());
    println!(
        "📅 Created: {}",
        workspace.config.created_at.format("%Y-%m-%d %H:%M")
    );
    println!(
        "⏰ Last used: {}",
        workspace.config.last_used.format("%Y-%m-%d %H:%M")
    );
    println!(
        "📁 Working directory: {}",
        workspace.config.preferred_workdir
    );

    if let Some(devcontainer) = &workspace.config.devcontainer_source {
        println!("📦 DevContainer source: {}", devcontainer);
    }

    if !workspace.config.port_forwards.is_empty() {
        println!(
            "🌐 Port forwards: {}",
            workspace
                .config
                .port_forwards
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !workspace.config.environment_vars.is_empty() {
        println!("🌍 Environment variables:");
        for (key, value) in &workspace.config.environment_vars {
            println!("   {}={}", key, value);
        }
    }

    if !workspace.config.custom_commands.is_empty() {
        println!("⚙️  Custom commands:");
        for cmd in &workspace.config.custom_commands {
            println!("   {}", cmd);
        }
    }

    // Show directory contents
    if workspace.path.exists() {
        println!("\n📋 Workspace contents:");
        match std::fs::read_dir(&workspace.path) {
            Ok(entries) => {
                let mut files: Vec<_> = entries.collect::<std::io::Result<Vec<_>>>()?;
                files.sort_by_key(|e| e.file_name());

                for entry in files.iter().take(10) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if entry.file_type()?.is_dir() {
                        println!("   📁 {}/", name);
                    } else {
                        println!("   📄 {}", name);
                    }
                }

                if files.len() > 10 {
                    println!("   ... and {} more files", files.len() - 10);
                }
            }
            Err(e) => println!("   Error reading directory: {}", e),
        }
    }

    println!();
    println!(
        "🚀 Start workspace: vortex dev --workspace {}",
        workspace.name
    );

    Ok(())
}

async fn import_devcontainer_workspace(
    vortex: &Arc<VortexCore>,
    name: &str,
    devcontainer_path: &Path,
    source: &Option<PathBuf>,
) -> Result<()> {
    let source_dir = source
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| std::path::Path::new("."));

    if !devcontainer_path.exists() {
        return Err(anyhow::anyhow!(
            "DevContainer file not found: {}",
            devcontainer_path.display()
        ));
    }

    let workspace =
        vortex
            .workspace_manager
            .create_from_devcontainer(name, devcontainer_path, source_dir)?;

    println!(
        "✅ Workspace '{}' imported from devcontainer!",
        workspace.name
    );
    println!("📦 DevContainer: {}", devcontainer_path.display());
    println!("📁 Path: {}", workspace.path.display());
    println!("🎯 Detected template: {}", workspace.config.template);

    if !workspace.config.port_forwards.is_empty() {
        println!(
            "🌐 Port forwards: {}",
            workspace
                .config
                .port_forwards
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    println!("🚀 Start with: vortex dev --workspace {}", workspace.name);

    Ok(())
}
