use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use vortex_core::{ResourceLimits, VmSpec, VortexCore};

#[derive(Parser)]
#[command(
    name = "vortex",
    about = "Vortex - Lightning-fast ephemeral VM platform",
    version = "0.3.0"
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if any command is using quiet mode
    let is_quiet = match &cli.command {
        Commands::Run { quiet, .. } => *quiet,
        Commands::Shell { quiet, .. } => *quiet,
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
        info!("Vortex v{} - Ephemeral VM Platform", vortex_core::VERSION);
    }

    // Initialize Vortex Core
    let vortex = Arc::new(
        vortex_core::init()
            .await
            .context("Failed to initialize Vortex core")?,
    );

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
    let config = vortex_core::VortexConfig::load()?;
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
    let config = vortex_core::VortexConfig::load()?;

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
    let config = vortex_core::VortexConfig::load()?;
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
            let config = vortex_core::VortexConfig::load()?;
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
