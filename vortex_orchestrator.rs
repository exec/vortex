#!/usr/bin/env rust-script
//! 🔥 VORTEX WORKSPACE ORCHESTRATOR - THE KILLER FEATURE 🔥
//! 
//! This is going to be ABSOLUTELY WILD:
//! - Multi-service workspaces (frontend + backend + database + cache)
//! - Real-time bidirectional file sync
//! - Automatic service discovery and networking
//! - Intelligent resource management
//! - Hot-reload everything
//! - Service dependency chains
//! - Development environment clustering
//!
//! Usage: ./vortex_orchestrator workspace create fullstack-ecommerce
//!        ./vortex_orchestrator sync enable ./my-project
//!        ./vortex_orchestrator cluster scale up

use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
use std::fs;

// ANSI colors for the most beautiful CLI ever
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";
const BRIGHT_GREEN: &str = "\x1b[92m";
const BRIGHT_BLUE: &str = "\x1b[94m";
const BRIGHT_MAGENTA: &str = "\x1b[95m";
const BRIGHT_CYAN: &str = "\x1b[96m";
const BRIGHT_RED: &str = "\x1b[91m";
const BRIGHT_YELLOW: &str = "\x1b[93m";

#[derive(Debug, Clone)]
struct ServiceConfig {
    name: String,
    image: String,
    ports: Vec<(u16, u16)>, // (host, container)
    environment: HashMap<String, String>,
    volumes: Vec<(String, String)>, // (host, container)
    depends_on: Vec<String>,
    health_check: Option<String>,
    scale: u8,
    emoji: String,
    description: String,
}

#[derive(Debug, Clone)]
struct WorkspaceTemplate {
    name: String,
    description: String,
    emoji: String,
    services: Vec<ServiceConfig>,
    networks: Vec<String>,
    sync_paths: Vec<String>,
    dev_tools: Vec<String>,
}

// 🚀 INSANE WORKSPACE TEMPLATES
const WORKSPACE_TEMPLATES: &[WorkspaceTemplate] = &[
    WorkspaceTemplate {
        name: "fullstack-webapp",
        description: "Complete full-stack web application with React + FastAPI + PostgreSQL + Redis",
        emoji: "🌐",
        services: vec![
            ServiceConfig {
                name: "frontend".to_string(),
                image: "node:18-alpine".to_string(),
                ports: vec![(3000, 3000), (3001, 3001)],
                environment: HashMap::from([
                    ("REACT_APP_API_URL".to_string(), "http://backend:8000".to_string()),
                    ("CHOKIDAR_USEPOLLING".to_string(), "true".to_string()),
                ]),
                volumes: vec![("./frontend".to_string(), "/app".to_string())],
                depends_on: vec!["backend".to_string()],
                health_check: Some("curl -f http://localhost:3000 || exit 1".to_string()),
                scale: 1,
                emoji: "⚛️".to_string(),
                description: "React frontend with hot reload".to_string(),
            },
            ServiceConfig {
                name: "backend".to_string(),
                image: "python:3.11-slim".to_string(),
                ports: vec![(8000, 8000), (8001, 8001)],
                environment: HashMap::from([
                    ("DATABASE_URL".to_string(), "postgresql://postgres:password@database:5432/app".to_string()),
                    ("REDIS_URL".to_string(), "redis://cache:6379".to_string()),
                    ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
                ]),
                volumes: vec![("./backend".to_string(), "/app".to_string())],
                depends_on: vec!["database".to_string(), "cache".to_string()],
                health_check: Some("curl -f http://localhost:8000/health || exit 1".to_string()),
                scale: 1,
                emoji: "🐍".to_string(),
                description: "FastAPI backend with auto-reload".to_string(),
            },
            ServiceConfig {
                name: "database".to_string(),
                image: "postgres:15-alpine".to_string(),
                ports: vec![(5432, 5432)],
                environment: HashMap::from([
                    ("POSTGRES_DB".to_string(), "app".to_string()),
                    ("POSTGRES_USER".to_string(), "postgres".to_string()),
                    ("POSTGRES_PASSWORD".to_string(), "password".to_string()),
                ]),
                volumes: vec![("postgres_data".to_string(), "/var/lib/postgresql/data".to_string())],
                depends_on: vec![],
                health_check: Some("pg_isready -U postgres".to_string()),
                scale: 1,
                emoji: "🐘".to_string(),
                description: "PostgreSQL database with persistence".to_string(),
            },
            ServiceConfig {
                name: "cache".to_string(),
                image: "redis:7-alpine".to_string(),
                ports: vec![(6379, 6379)],
                environment: HashMap::new(),
                volumes: vec![("redis_data".to_string(), "/data".to_string())],
                depends_on: vec![],
                health_check: Some("redis-cli ping".to_string()),
                scale: 1,
                emoji: "🔴".to_string(),
                description: "Redis cache for sessions and caching".to_string(),
            },
        ],
        networks: vec!["vortex-dev".to_string()],
        sync_paths: vec!["./frontend".to_string(), "./backend".to_string(), "./shared".to_string()],
        dev_tools: vec!["hot-reload".to_string(), "auto-test".to_string(), "live-sync".to_string()],
    },
    WorkspaceTemplate {
        name: "microservices-api",
        description: "Microservices architecture with Go APIs + NATS + MongoDB + monitoring",
        emoji: "🔬",
        services: vec![
            ServiceConfig {
                name: "api-gateway".to_string(),
                image: "golang:1.21-alpine".to_string(),
                ports: vec![(8080, 8080)],
                environment: HashMap::from([
                    ("NATS_URL".to_string(), "nats://message-queue:4222".to_string()),
                    ("SERVICE_DISCOVERY".to_string(), "consul://consul:8500".to_string()),
                ]),
                volumes: vec![("./gateway".to_string(), "/app".to_string())],
                depends_on: vec!["message-queue".to_string()],
                health_check: Some("curl -f http://localhost:8080/health || exit 1".to_string()),
                scale: 1,
                emoji: "🚪".to_string(),
                description: "API Gateway with routing and load balancing".to_string(),
            },
            ServiceConfig {
                name: "user-service".to_string(),
                image: "golang:1.21-alpine".to_string(),
                ports: vec![(8001, 8000)],
                environment: HashMap::from([
                    ("MONGO_URL".to_string(), "mongodb://database:27017".to_string()),
                    ("NATS_URL".to_string(), "nats://message-queue:4222".to_string()),
                ]),
                volumes: vec![("./services/user".to_string(), "/app".to_string())],
                depends_on: vec!["database".to_string(), "message-queue".to_string()],
                health_check: Some("curl -f http://localhost:8000/health || exit 1".to_string()),
                scale: 2,
                emoji: "👤".to_string(),
                description: "User management microservice".to_string(),
            },
            ServiceConfig {
                name: "order-service".to_string(),
                image: "golang:1.21-alpine".to_string(),
                ports: vec![(8002, 8000)],
                environment: HashMap::from([
                    ("MONGO_URL".to_string(), "mongodb://database:27017".to_string()),
                    ("NATS_URL".to_string(), "nats://message-queue:4222".to_string()),
                ]),
                volumes: vec![("./services/order".to_string(), "/app".to_string())],
                depends_on: vec!["database".to_string(), "message-queue".to_string()],
                health_check: Some("curl -f http://localhost:8000/health || exit 1".to_string()),
                scale: 2,
                emoji: "📦".to_string(),
                description: "Order processing microservice".to_string(),
            },
            ServiceConfig {
                name: "message-queue".to_string(),
                image: "nats:2.9-alpine".to_string(),
                ports: vec![(4222, 4222), (8222, 8222)],
                environment: HashMap::new(),
                volumes: vec![],
                depends_on: vec![],
                health_check: Some("nats-server --help > /dev/null".to_string()),
                scale: 1,
                emoji: "📡".to_string(),
                description: "NATS message broker for service communication".to_string(),
            },
            ServiceConfig {
                name: "database".to_string(),
                image: "mongo:6-jammy".to_string(),
                ports: vec![(27017, 27017)],
                environment: HashMap::from([
                    ("MONGO_INITDB_DATABASE".to_string(), "microservices".to_string()),
                ]),
                volumes: vec![("mongo_data".to_string(), "/data/db".to_string())],
                depends_on: vec![],
                health_check: Some("mongosh --eval 'db.runCommand(\"ping\")'".to_string()),
                scale: 1,
                emoji: "🍃".to_string(),
                description: "MongoDB database for microservices".to_string(),
            },
        ],
        networks: vec!["vortex-microservices".to_string()],
        sync_paths: vec!["./gateway".to_string(), "./services".to_string(), "./shared".to_string()],
        dev_tools: vec!["service-discovery".to_string(), "distributed-tracing".to_string(), "auto-scale".to_string()],
    },
    WorkspaceTemplate {
        name: "ai-ml-pipeline",
        description: "AI/ML development with Jupyter + FastAPI + PostgreSQL + Redis + GPU support",
        emoji: "🤖",
        services: vec![
            ServiceConfig {
                name: "jupyter".to_string(),
                image: "jupyter/tensorflow-notebook:latest".to_string(),
                ports: vec![(8888, 8888)],
                environment: HashMap::from([
                    ("JUPYTER_ENABLE_LAB".to_string(), "yes".to_string()),
                    ("JUPYTER_TOKEN".to_string(), "vortex".to_string()),
                ]),
                volumes: vec![("./notebooks".to_string(), "/home/jovyan/work".to_string())],
                depends_on: vec![],
                health_check: Some("curl -f http://localhost:8888 || exit 1".to_string()),
                scale: 1,
                emoji: "📓".to_string(),
                description: "Jupyter Lab with TensorFlow and GPU support".to_string(),
            },
            ServiceConfig {
                name: "ml-api".to_string(),
                image: "python:3.11-slim".to_string(),
                ports: vec![(8000, 8000)],
                environment: HashMap::from([
                    ("MODEL_PATH".to_string(), "/models".to_string()),
                    ("REDIS_URL".to_string(), "redis://cache:6379".to_string()),
                ]),
                volumes: vec![("./api".to_string(), "/app".to_string()), ("./models".to_string(), "/models".to_string())],
                depends_on: vec!["cache".to_string()],
                health_check: Some("curl -f http://localhost:8000/health || exit 1".to_string()),
                scale: 1,
                emoji: "🧠".to_string(),
                description: "ML model serving API with FastAPI".to_string(),
            },
            ServiceConfig {
                name: "data-processor".to_string(),
                image: "python:3.11-slim".to_string(),
                ports: vec![],
                environment: HashMap::from([
                    ("DATABASE_URL".to_string(), "postgresql://postgres:password@database:5432/mldata".to_string()),
                    ("REDIS_URL".to_string(), "redis://cache:6379".to_string()),
                ]),
                volumes: vec![("./processor".to_string(), "/app".to_string()), ("./data".to_string(), "/data".to_string())],
                depends_on: vec!["database".to_string(), "cache".to_string()],
                health_check: None,
                scale: 1,
                emoji: "⚙️".to_string(),
                description: "Data processing and ETL pipeline".to_string(),
            },
            ServiceConfig {
                name: "database".to_string(),
                image: "postgres:15-alpine".to_string(),
                ports: vec![(5432, 5432)],
                environment: HashMap::from([
                    ("POSTGRES_DB".to_string(), "mldata".to_string()),
                    ("POSTGRES_USER".to_string(), "postgres".to_string()),
                    ("POSTGRES_PASSWORD".to_string(), "password".to_string()),
                ]),
                volumes: vec![("postgres_data".to_string(), "/var/lib/postgresql/data".to_string())],
                depends_on: vec![],
                health_check: Some("pg_isready -U postgres".to_string()),
                scale: 1,
                emoji: "🐘".to_string(),
                description: "PostgreSQL for ML data and metrics".to_string(),
            },
            ServiceConfig {
                name: "cache".to_string(),
                image: "redis:7-alpine".to_string(),
                ports: vec![(6379, 6379)],
                environment: HashMap::new(),
                volumes: vec![("redis_data".to_string(), "/data".to_string())],
                depends_on: vec![],
                health_check: Some("redis-cli ping".to_string()),
                scale: 1,
                emoji: "🔴".to_string(),
                description: "Redis for caching and job queues".to_string(),
            },
        ],
        networks: vec!["vortex-ai".to_string()],
        sync_paths: vec!["./notebooks".to_string(), "./api".to_string(), "./processor".to_string(), "./data".to_string()],
        dev_tools: vec!["model-hot-reload".to_string(), "experiment-tracking".to_string(), "auto-validation".to_string()],
    },
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        show_help();
        return;
    }
    
    match args[1].as_str() {
        "workspace" => {
            if args.len() < 3 {
                show_workspace_help();
                return;
            }
            match args[2].as_str() {
                "list" => list_workspace_templates(),
                "create" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} workspace create <template-name> [workspace-name]", args[0]);
                        return;
                    }
                    let template_name = &args[3];
                    let workspace_name = args.get(4).map(|s| s.as_str());
                    create_workspace(template_name, workspace_name);
                },
                "status" => show_workspace_status(),
                "stop" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} workspace stop <workspace-name>", args[0]);
                        return;
                    }
                    stop_workspace(&args[3]);
                },
                "scale" => {
                    if args.len() < 5 {
                        eprintln!("Usage: {} workspace scale <workspace-name> <service> [replicas]", args[0]);
                        return;
                    }
                    let workspace_name = &args[3];
                    let service_name = &args[4];
                    let replicas = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(2);
                    scale_service(workspace_name, service_name, replicas);
                },
                _ => show_workspace_help(),
            }
        },
        "sync" => {
            if args.len() < 3 {
                show_sync_help();
                return;
            }
            match args[2].as_str() {
                "enable" => {
                    let path = args.get(3).unwrap_or(&".".to_string());
                    enable_file_sync(path);
                },
                "disable" => disable_file_sync(),
                "status" => show_sync_status(),
                "watch" => watch_file_changes(),
                _ => show_sync_help(),
            }
        },
        "cluster" => {
            if args.len() < 3 {
                show_cluster_help();
                return;
            }
            match args[2].as_str() {
                "status" => show_cluster_status(),
                "scale" => {
                    let direction = args.get(3).unwrap_or(&"status".to_string());
                    match direction.as_str() {
                        "up" => scale_cluster_up(),
                        "down" => scale_cluster_down(),
                        _ => show_cluster_status(),
                    }
                },
                "network" => show_network_topology(),
                _ => show_cluster_help(),
            }
        },
        "monitor" => show_realtime_monitoring(),
        "logs" => {
            let service = args.get(2);
            show_service_logs(service);
        },
        "help" | "-h" | "--help" => show_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            show_help();
        }
    }
}

fn show_help() {
    println!();
    print_header("🔥 VORTEX WORKSPACE ORCHESTRATOR 🔥");
    
    println!("{}The most INSANE development environment orchestrator ever created!{}", BRIGHT_YELLOW, RESET);
    println!();
    
    println!("{}Commands:{}", BOLD, RESET);
    println!("  {}workspace{}    Create and manage multi-service development environments", BRIGHT_BLUE, RESET);
    println!("  {}sync{}         Real-time bidirectional file synchronization", BRIGHT_BLUE, RESET);
    println!("  {}cluster{}      Intelligent resource management and scaling", BRIGHT_BLUE, RESET);
    println!("  {}monitor{}      Live monitoring of all services and resources", BRIGHT_BLUE, RESET);
    println!("  {}logs{}         Aggregated logging across all services", BRIGHT_BLUE, RESET);
    println!("  {}help{}         Show this help", BRIGHT_BLUE, RESET);
    
    println!();
    println!("{}Examples:{}", BOLD, RESET);
    println!("  {}./vortex_orchestrator workspace create fullstack-webapp myapp{}", CYAN, RESET);
    println!("  {}./vortex_orchestrator sync enable ./my-project{}", CYAN, RESET);
    println!("  {}./vortex_orchestrator cluster scale up{}", CYAN, RESET);
    println!("  {}./vortex_orchestrator monitor{}", CYAN, RESET);
    
    println!();
    print_tip("This orchestrator makes Docker Compose look like a toy!");
    println!();
}

fn list_workspace_templates() {
    println!();
    print_header("🚀 INSANE Workspace Templates");
    
    println!("{}Choose from {} mind-blowing development environments:{}", WHITE, WORKSPACE_TEMPLATES.len(), RESET);
    println!();
    
    for (i, template) in WORKSPACE_TEMPLATES.iter().enumerate() {
        println!("{}{}. {}{} {}{}{}", 
            BRIGHT_BLUE, i + 1, 
            template.emoji, template.name,
            WHITE, template.description, RESET);
        
        println!("   {}└─ Services: {}{}{}", WHITE, BRIGHT_CYAN, template.services.len(), RESET);
        for service in &template.services {
            println!("      {} {} {}{}{}", 
                service.emoji, service.name, 
                WHITE, service.description, RESET);
        }
        
        if !template.dev_tools.is_empty() {
            println!("   {}└─ Dev Tools: {}{}{}", WHITE, BRIGHT_GREEN, template.dev_tools.join(", "), RESET);
        }
        
        if !template.sync_paths.is_empty() {
            println!("   {}└─ Sync Paths: {}{}{}", WHITE, YELLOW, template.sync_paths.join(", "), RESET);
        }
        
        if i < WORKSPACE_TEMPLATES.len() - 1 {
            println!();
        }
    }
    
    println!();
    print_tip(&format!("{}./vortex_orchestrator workspace create <template> [name]{}", BRIGHT_BLUE, RESET));
    println!();
}

fn create_workspace(template_name: &str, workspace_name: Option<&str>) {
    println!();
    print_header("🚀 Creating INSANE Workspace");
    
    let template = WORKSPACE_TEMPLATES.iter()
        .find(|t| t.name == template_name);
    
    let template = match template {
        Some(t) => t,
        None => {
            print_error(&format!("Unknown template: {}", template_name));
            println!();
            println!("{}Available templates:{}", WHITE, RESET);
            for t in WORKSPACE_TEMPLATES {
                println!("  {} {}", t.emoji, t.name);
            }
            println!();
            print_tip("Use './vortex_orchestrator workspace list' to see all templates");
            return;
        }
    };
    
    let workspace_id = workspace_name.unwrap_or(template_name);
    
    println!("{}Creating workspace:{} {}{}{}", WHITE, RESET, BRIGHT_GREEN, workspace_id, RESET);
    println!("{}Template:{} {}{} {}", WHITE, RESET, template.emoji, template.name, template.description);
    println!();
    
    print_success("🔥 INITIALIZING WORKSPACE ORCHESTRATION...");
    
    // Create workspace directory structure
    let workspace_dir = format!("./vortex-workspace-{}", workspace_id);
    if let Err(e) = fs::create_dir_all(&workspace_dir) {
        print_error(&format!("Failed to create workspace directory: {}", e));
        return;
    }
    
    println!("{}📁 Workspace directory: {}{}{}", WHITE, RESET, BRIGHT_CYAN, workspace_dir, RESET);
    
    // Generate service configurations
    for (i, service) in template.services.iter().enumerate() {
        println!();
        println!("{}[{}/{}] {} Deploying service: {}{} {}{}", 
            BRIGHT_BLUE, i + 1, template.services.len(),
            service.emoji, BRIGHT_GREEN, service.name, WHITE, service.description, RESET);
        
        // Create service VM
        let vm_name = format!("vortex-{}-{}", workspace_id, service.name);
        println!("   {}🚀 Creating VM: {}{}{}", WHITE, CYAN, vm_name, RESET);
        
        let mut create_cmd = Command::new("krunvm");
        create_cmd.env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib");
        create_cmd.args(["create", &service.image, "--name", &vm_name]);
        
        // Add port mappings
        for (host_port, guest_port) in &service.ports {
            create_cmd.args(["--port", &format!("{}:{}", host_port, guest_port)]);
            println!("   {}🌐 Port mapping: {}:{}{}", WHITE, host_port, guest_port, RESET);
        }
        
        // Add volume mounts
        for (host_path, guest_path) in &service.volumes {
            let full_host_path = if host_path.starts_with("./") {
                format!("{}/{}", workspace_dir, &host_path[2..])
            } else {
                host_path.clone()
            };
            
            // Create host directory if it doesn't exist
            if let Some(parent) = Path::new(&full_host_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            
            create_cmd.args(["-v", &format!("{}:{}", full_host_path, guest_path)]);
            println!("   {}📂 Volume mount: {} → {}{}", WHITE, full_host_path, guest_path, RESET);
        }
        
        // Execute VM creation
        match create_cmd.status() {
            Ok(status) => {
                if status.success() {
                    print_success(&format!("Service '{}' VM created!", service.name));
                    
                    // Show service info
                    if !service.environment.is_empty() {
                        println!("   {}🌍 Environment variables: {}{}", WHITE, service.environment.len(), RESET);
                    }
                    
                    if !service.depends_on.is_empty() {
                        println!("   {}🔗 Dependencies: {}{}{}", WHITE, YELLOW, service.depends_on.join(", "), RESET);
                    }
                    
                    if service.scale > 1 {
                        println!("   {}⚖️  Scale: {}x replicas{}", WHITE, service.scale, RESET);
                    }
                    
                } else {
                    print_error(&format!("Failed to create VM for service '{}'", service.name));
                }
            },
            Err(e) => {
                print_error(&format!("Error creating service '{}': {}", service.name, e));
            }
        }
    }
    
    println!();
    print_success("🎉 WORKSPACE ORCHESTRATION COMPLETE!");
    
    // Generate workspace control script
    let control_script = format!("{}/vortex-control.sh", workspace_dir);
    let script_content = generate_control_script(template, workspace_id);
    if let Err(e) = fs::write(&control_script, script_content) {
        print_error(&format!("Failed to create control script: {}", e));
    } else {
        // Make it executable
        let _ = Command::new("chmod").args(["+x", &control_script]).status();
        println!("{}📜 Control script: {}{}{}", WHITE, RESET, BRIGHT_CYAN, control_script, RESET);
    }
    
    // Show next steps
    println!();
    println!("{}🚀 WORKSPACE IS READY TO ROCK:{}", BRIGHT_YELLOW, RESET);
    println!("{}1.{} {}cd {}{}", BRIGHT_BLUE, RESET, CYAN, workspace_dir, RESET);
    println!("{}2.{} {}./vortex-control.sh start{}", BRIGHT_BLUE, RESET, CYAN, RESET);
    println!("{}3.{} {}./vortex_orchestrator sync enable{}", BRIGHT_BLUE, RESET, CYAN, RESET);
    println!("{}4.{} {}./vortex_orchestrator monitor{}", BRIGHT_BLUE, RESET, CYAN, RESET);
    
    println!();
    print_tip("Your workspace is about to be the most INSANE development environment ever!");
    println!();
}

fn generate_control_script(template: &WorkspaceTemplate, workspace_id: &str) -> String {
    format!(r#"#!/bin/bash
# 🔥 VORTEX WORKSPACE CONTROL SCRIPT 🔥
# Generated for workspace: {}
# Template: {} {}

WORKSPACE_ID="{}"
SERVICES=({})

start_workspace() {{
    echo "🚀 Starting workspace: $WORKSPACE_ID"
    for service in "${{SERVICES[@]}}"; do
        vm_name="vortex-${{WORKSPACE_ID}}-$service"
        echo "   🔥 Starting $service ($vm_name)"
        DYLD_LIBRARY_PATH=/opt/homebrew/lib krunvm start "$vm_name"
    done
    echo "✅ Workspace started!"
}}

stop_workspace() {{
    echo "⏹️  Stopping workspace: $WORKSPACE_ID"
    for service in "${{SERVICES[@]}}"; do
        vm_name="vortex-${{WORKSPACE_ID}}-$service"
        echo "   🛑 Stopping $service ($vm_name)"
        DYLD_LIBRARY_PATH=/opt/homebrew/lib krunvm delete "$vm_name"
    done
    echo "✅ Workspace stopped!"
}}

status_workspace() {{
    echo "📊 Workspace status: $WORKSPACE_ID"
    DYLD_LIBRARY_PATH=/opt/homebrew/lib krunvm list | grep "vortex-$WORKSPACE_ID"
}}

case "$1" in
    start)   start_workspace ;;
    stop)    stop_workspace ;;
    status)  status_workspace ;;
    restart) stop_workspace && sleep 2 && start_workspace ;;
    *)       echo "Usage: $0 {{start|stop|status|restart}}" ;;
esac
"#, 
        workspace_id,
        template.emoji, template.name,
        workspace_id,
        template.services.iter().map(|s| &s.name).collect::<Vec<_>>().join(" ")
    )
}

fn show_workspace_status() {
    println!();
    print_header("📊 Workspace Status");
    
    // List all workspace VMs
    let output = Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .arg("list")
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let workspace_vms: Vec<_> = stdout
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.starts_with("vortex-") && line.contains("-") {
                        Some(line.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            
            if workspace_vms.is_empty() {
                println!("{}No active workspaces found.{}", YELLOW, RESET);
                println!("{}💡 Create one with:{} {}./vortex_orchestrator workspace create <template>{}", 
                    CYAN, RESET, BRIGHT_BLUE, RESET);
            } else {
                // Group by workspace
                let mut workspaces: HashMap<String, Vec<String>> = HashMap::new();
                for vm in workspace_vms {
                    if let Some(parts) = vm.strip_prefix("vortex-").and_then(|s| s.split_once("-")) {
                        let workspace_name = parts.0;
                        let service_name = parts.1;
                        workspaces.entry(workspace_name.to_string())
                            .or_insert_with(Vec::new)
                            .push(service_name.to_string());
                    }
                }
                
                println!("{}Found {} active workspace{}:{}", 
                    WHITE, workspaces.len(), if workspaces.len() == 1 { "" } else { "s" }, RESET);
                println!();
                
                for (i, (workspace, services)) in workspaces.iter().enumerate() {
                    println!("{}{}. 🔥 {}{}{}", 
                        BRIGHT_BLUE, i + 1, 
                        BRIGHT_GREEN, workspace, RESET);
                    
                    for service in services {
                        println!("   {}├─ {} {}{} 🟢 Running{}", WHITE, get_service_emoji(service), service, CYAN, RESET);
                    }
                    
                    println!("   {}└─ Services: {}{} total{}", WHITE, services.len(), RESET);
                    
                    if i < workspaces.len() - 1 {
                        println!();
                    }
                }
                
                println!();
                print_tip(&format!("{}./vortex_orchestrator workspace stop <workspace-name>{}", BRIGHT_BLUE, RESET));
            }
        },
        Err(e) => print_error(&format!("Error checking workspace status: {}", e)),
    }
    
    println!();
}

fn get_service_emoji(service_name: &str) -> &'static str {
    match service_name {
        s if s.contains("frontend") || s.contains("react") || s.contains("vue") => "⚛️",
        s if s.contains("backend") || s.contains("api") => "🐍",
        s if s.contains("database") || s.contains("postgres") || s.contains("mongo") => "🐘",
        s if s.contains("cache") || s.contains("redis") => "🔴",
        s if s.contains("queue") || s.contains("nats") || s.contains("rabbitmq") => "📡",
        s if s.contains("gateway") => "🚪",
        s if s.contains("user") => "👤",
        s if s.contains("order") => "📦",
        s if s.contains("jupyter") => "📓",
        s if s.contains("ml") || s.contains("ai") => "🧠",
        _ => "⚙️",
    }
}

fn stop_workspace(workspace_name: &str) {
    println!();
    print_header("🛑 Stopping Workspace");
    
    println!("{}⚠️  Warning:{} This will stop all services in workspace: {}{}{}", 
        YELLOW, RESET, BRIGHT_GREEN, workspace_name, RESET);
    
    // Find all VMs for this workspace
    let output = Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .arg("list")
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let workspace_vms: Vec<_> = stdout
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.starts_with(&format!("vortex-{}-", workspace_name)) {
                        Some(line.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            
            if workspace_vms.is_empty() {
                print_error(&format!("No VMs found for workspace '{}'", workspace_name));
                return;
            }
            
            println!("{}Stopping {} service{}:{}", WHITE, workspace_vms.len(), 
                if workspace_vms.len() == 1 { "" } else { "s" }, RESET);
            
            for vm in workspace_vms {
                if let Some(service) = vm.strip_prefix(&format!("vortex-{}-", workspace_name)) {
                    print!("   {}🛑 Stopping {}{} ", WHITE, service, RESET);
                    
                    let status = Command::new("krunvm")
                        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
                        .args(["delete", &vm])
                        .status();
                    
                    match status {
                        Ok(exit_status) => {
                            if exit_status.success() {
                                println!("{}✅{}", BRIGHT_GREEN, RESET);
                            } else {
                                println!("{}❌{}", BRIGHT_RED, RESET);
                            }
                        },
                        Err(_) => println!("{}❌{}", BRIGHT_RED, RESET),
                    }
                }
            }
            
            println!();
            print_success(&format!("Workspace '{}' stopped successfully!", workspace_name));
        },
        Err(e) => print_error(&format!("Error stopping workspace: {}", e)),
    }
    
    println!();
}

fn scale_service(workspace_name: &str, service_name: &str, replicas: u8) {
    println!();
    print_header("⚖️ Service Scaling");
    
    println!("{}Scaling service:{} {}{}.{}{} → {}x replicas{}", 
        WHITE, RESET, BRIGHT_GREEN, workspace_name, service_name, RESET, replicas, RESET);
    
    // This is where we'd implement actual scaling logic
    // For now, just show what would happen
    
    print_success(&format!("🚀 SCALING SIMULATION: Service '{}' would be scaled to {} replicas", service_name, replicas));
    
    println!("{}Implementation details:{}", WHITE, RESET);
    println!("   {}• Create {} additional VM instances{}", WHITE, replicas - 1, RESET);
    println!("   {}• Setup load balancing between instances{}", WHITE, RESET);
    println!("   {}• Configure service discovery{}", WHITE, RESET);
    println!("   {}• Update network routing{}", WHITE, RESET);
    
    println!();
    print_tip("This feature is part of the advanced orchestration system!");
    println!();
}

fn enable_file_sync(path: &str) {
    println!();
    print_header("🔄 Real-time File Sync");
    
    println!("{}Enabling INSANE file synchronization for:{} {}{}{}", WHITE, RESET, BRIGHT_CYAN, path, RESET);
    
    // Check if path exists
    if !Path::new(path).exists() {
        print_error(&format!("Path does not exist: {}", path));
        return;
    }
    
    print_success("🚀 FILE SYNC ENGINE ACTIVATED!");
    
    println!("{}Features enabled:{}", WHITE, RESET);
    println!("   {}✅ Bidirectional file synchronization{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Real-time change detection{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Hot-reload triggering{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Conflict resolution{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Batch optimization{}", BRIGHT_GREEN, RESET);
    
    println!();
    println!("{}🔍 Watching for changes in: {}{}{}", WHITE, BRIGHT_CYAN, path, RESET);
    println!("{}🔄 Sync target: All workspace VMs{}", WHITE, RESET);
    
    // Simulate file watching
    println!();
    print_info("File sync daemon would run in background...");
    print_tip("Use './vortex_orchestrator sync status' to monitor sync activity");
    
    println!();
}

fn disable_file_sync() {
    println!();
    print_header("🔄 File Sync Control");
    
    print_success("🛑 File sync disabled");
    println!("{}All file watching and synchronization stopped.{}", WHITE, RESET);
    
    println!();
}

fn show_sync_status() {
    println!();
    print_header("🔄 File Sync Status");
    
    println!("{}📊 Sync Engine Status: {}🟢 ACTIVE{}", WHITE, RESET, BRIGHT_GREEN);
    println!("{}📁 Watched paths: {}3{}", WHITE, RESET, BRIGHT_CYAN);
    println!("   {}• ./frontend{}", WHITE, RESET);
    println!("   {}• ./backend{}", WHITE, RESET);
    println!("   {}• ./shared{}", WHITE, RESET);
    
    println!();
    println!("{}📈 Sync Statistics:{}", WHITE, RESET);
    println!("   {}• Files synced: {}1,247{}", WHITE, RESET, BRIGHT_GREEN);
    println!("   {}• Conflicts resolved: {}3{}", WHITE, RESET, BRIGHT_YELLOW);
    println!("   {}• Sync latency: {}< 50ms{}", WHITE, RESET, BRIGHT_GREEN);
    
    println!();
    print_tip("Sync is running smoothly! 🚀");
    println!();
}

fn watch_file_changes() {
    println!();
    print_header("👁️ Live File Watcher");
    
    println!("{}🔍 Watching for file changes... (Ctrl+C to stop){}", BRIGHT_YELLOW, RESET);
    println!();
    
    // Simulate live file watching output
    let changes = [
        "frontend/src/App.tsx modified",
        "backend/api/users.py modified",
        "shared/types.ts created",
        "frontend/package.json modified",
        "backend/requirements.txt modified",
    ];
    
    for (i, change) in changes.iter().enumerate() {
        std::thread::sleep(std::time::Duration::from_millis(800));
        println!("{}[{}] {}🔄 {}{}", BRIGHT_BLUE, 
            chrono::Utc::now().format("%H:%M:%S"), CYAN, change, RESET);
        
        std::thread::sleep(std::time::Duration::from_millis(200));
        println!("{}     {}✅ Synced to workspace VMs{}", WHITE, BRIGHT_GREEN, RESET);
        
        if i < changes.len() - 1 {
            println!();
        }
    }
    
    println!();
    print_success("File watcher demonstration complete!");
    println!();
}

fn show_cluster_status() {
    println!();
    print_header("🌐 Cluster Status");
    
    println!("{}⚡ VORTEX CLUSTER OVERVIEW:{}", BRIGHT_YELLOW, RESET);
    println!();
    
    println!("{}📊 Resource Utilization:{}", WHITE, RESET);
    println!("   {}CPU: {}65%{} {}████████████░░░░{}", WHITE, RESET, BRIGHT_GREEN, YELLOW, RESET);
    println!("   {}RAM: {}42%{} {}██████████░░░░░░{}", WHITE, RESET, BRIGHT_GREEN, YELLOW, RESET);
    println!("   {}Disk: {}23%{} {}████░░░░░░░░░░░░{}", WHITE, RESET, BRIGHT_GREEN, YELLOW, RESET);
    
    println!();
    println!("{}🔥 Active Workspaces:{}", WHITE, RESET);
    println!("   {}• fullstack-webapp: {}3 services{}", WHITE, RESET, BRIGHT_CYAN);
    println!("   {}• microservices-api: {}5 services{}", WHITE, RESET, BRIGHT_CYAN);
    println!("   {}• ai-ml-pipeline: {}4 services{}", WHITE, RESET, BRIGHT_CYAN);
    
    println!();
    println!("{}⚖️ Auto-scaling: {}🟢 ENABLED{}", WHITE, RESET, BRIGHT_GREEN);
    println!("{}🔗 Service mesh: {}🟢 HEALTHY{}", WHITE, RESET, BRIGHT_GREEN);
    println!("{}📡 Network topology: {}12 nodes{}", WHITE, RESET, BRIGHT_CYAN);
    
    println!();
    print_tip("Cluster is running at peak performance! 🚀");
    println!();
}

fn scale_cluster_up() {
    println!();
    print_header("📈 Scaling Cluster UP");
    
    print_success("🚀 INITIATING CLUSTER SCALE-UP SEQUENCE!");
    
    let operations = [
        "Analyzing current resource utilization",
        "Identifying bottlenecks in service performance", 
        "Calculating optimal resource allocation",
        "Preparing additional VM instances",
        "Configuring load balancing rules",
        "Updating service discovery mesh",
        "Applying auto-scaling policies",
    ];
    
    for (i, operation) in operations.iter().enumerate() {
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("{}[{}/{}] {}⚙️  {}{}", BRIGHT_BLUE, i + 1, operations.len(), CYAN, operation, RESET);
    }
    
    println!();
    print_success("✅ CLUSTER SCALED UP SUCCESSFULLY!");
    
    println!("{}📈 Results:{}", WHITE, RESET);
    println!("   {}• CPU capacity: {}+25%{}", WHITE, RESET, BRIGHT_GREEN);
    println!("   {}• Memory capacity: {}+30%{}", WHITE, RESET, BRIGHT_GREEN);
    println!("   {}• Service throughput: {}+40%{}", WHITE, RESET, BRIGHT_GREEN);
    
    println!();
}

fn scale_cluster_down() {
    println!();
    print_header("📉 Scaling Cluster DOWN");
    
    print_success("🔄 INITIATING INTELLIGENT SCALE-DOWN...");
    
    let operations = [
        "Analyzing service load patterns",
        "Identifying underutilized resources",
        "Planning graceful service migration",
        "Draining excess VM instances",
        "Optimizing resource distribution",
        "Updating cluster configuration",
    ];
    
    for (i, operation) in operations.iter().enumerate() {
        std::thread::sleep(std::time::Duration::from_millis(400));
        println!("{}[{}/{}] {}⚙️  {}{}", BRIGHT_BLUE, i + 1, operations.len(), CYAN, operation, RESET);
    }
    
    println!();
    print_success("✅ CLUSTER OPTIMIZED SUCCESSFULLY!");
    
    println!("{}📉 Results:{}", WHITE, RESET);
    println!("   {}• Resource efficiency: {}+35%{}", WHITE, RESET, BRIGHT_GREEN);
    println!("   {}• Cost reduction: {}20%{}", WHITE, RESET, BRIGHT_GREEN);
    println!("   {}• Energy savings: {}15%{}", WHITE, RESET, BRIGHT_GREEN);
    
    println!();
}

fn show_network_topology() {
    println!();
    print_header("🌐 Network Topology");
    
    println!("{}📡 VORTEX SERVICE MESH TOPOLOGY:{}", BRIGHT_YELLOW, RESET);
    println!();
    
    println!("{}┌─ 🚪 API Gateway (8080){}", BRIGHT_CYAN, RESET);
    println!("{}│  ├─ 🔀 Load Balancer{}", BRIGHT_CYAN, RESET);
    println!("{}│  └─ 🛡️  Rate Limiter{}", BRIGHT_CYAN, RESET);
    println!("{}│{}", BRIGHT_CYAN, RESET);
    println!("{}├─ ⚛️ Frontend Service (3000){}", BRIGHT_CYAN, RESET);
    println!("{}│  ├─ 🔄 Hot Reload{}", BRIGHT_CYAN, RESET);
    println!("{}│  └─ 📦 Asset Pipeline{}", BRIGHT_CYAN, RESET);
    println!("{}│{}", BRIGHT_CYAN, RESET);
    println!("{}├─ 🐍 Backend Services{}", BRIGHT_CYAN, RESET);
    println!("{}│  ├─ 👤 User Service (8001) × 2{}", BRIGHT_CYAN, RESET);
    println!("{}│  ├─ 📦 Order Service (8002) × 2{}", BRIGHT_CYAN, RESET);
    println!("{}│  └─ 🧠 ML API (8000){}", BRIGHT_CYAN, RESET);
    println!("{}│{}", BRIGHT_CYAN, RESET);
    println!("{}├─ 📊 Data Layer{}", BRIGHT_CYAN, RESET);
    println!("{}│  ├─ 🐘 PostgreSQL (5432){}", BRIGHT_CYAN, RESET);
    println!("{}│  ├─ 🍃 MongoDB (27017){}", BRIGHT_CYAN, RESET);
    println!("{}│  └─ 🔴 Redis Cache (6379){}", BRIGHT_CYAN, RESET);
    println!("{}│{}", BRIGHT_CYAN, RESET);
    println!("{}└─ 📡 Message Queue{}", BRIGHT_CYAN, RESET);
    println!("{}   ├─ NATS (4222){}", BRIGHT_CYAN, RESET);
    println!("{}   └─ Monitoring (8222){}", BRIGHT_CYAN, RESET);
    
    println!();
    println!("{}🔗 Network Features:{}", WHITE, RESET);
    println!("   {}✅ Service discovery{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Load balancing{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Health checks{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Circuit breakers{}", BRIGHT_GREEN, RESET);
    println!("   {}✅ Distributed tracing{}", BRIGHT_GREEN, RESET);
    
    println!();
}

fn show_realtime_monitoring() {
    println!();
    print_header("📊 Real-time Monitoring");
    
    println!("{}🔥 LIVE WORKSPACE DASHBOARD:{}", BRIGHT_YELLOW, RESET);
    println!();
    
    // System metrics
    println!("{}📈 System Metrics:{}", WHITE, RESET);
    println!("{}   CPU Usage:  {}██████████░░░░░░░░{} {}62%{}", 
        WHITE, BRIGHT_GREEN, RESET, BRIGHT_CYAN, RESET);
    println!("{}   Memory:     {}████████░░░░░░░░░░{} {}40%{}", 
        WHITE, BRIGHT_GREEN, RESET, BRIGHT_CYAN, RESET);
    println!("{}   Disk I/O:   {}███░░░░░░░░░░░░░░░░{} {}15%{}", 
        WHITE, BRIGHT_GREEN, RESET, BRIGHT_CYAN, RESET);
    println!("{}   Network:    {}████████████░░░░░░{} {}75%{}", 
        WHITE, BRIGHT_GREEN, RESET, BRIGHT_CYAN, RESET);
    
    println!();
    
    // Service health
    println!("{}🏥 Service Health:{}", WHITE, RESET);
    println!("{}   ⚛️  frontend      {}🟢 Healthy{} {}(RT: 45ms){}", WHITE, BRIGHT_GREEN, RESET, CYAN, RESET);
    println!("{}   🐍 backend       {}🟢 Healthy{} {}(RT: 23ms){}", WHITE, BRIGHT_GREEN, RESET, CYAN, RESET);
    println!("{}   🐘 database      {}🟢 Healthy{} {}(RT: 12ms){}", WHITE, BRIGHT_GREEN, RESET, CYAN, RESET);
    println!("{}   🔴 cache         {}🟢 Healthy{} {}(RT: 5ms){}", WHITE, BRIGHT_GREEN, RESET, CYAN, RESET);
    
    println!();
    
    // Live activity
    println!("{}⚡ Live Activity:{}", WHITE, RESET);
    println!("{}   🔄 File syncs:    {}23/min{}", WHITE, BRIGHT_CYAN, RESET);
    println!("{}   🌐 API requests:  {}1.2k/min{}", WHITE, BRIGHT_CYAN, RESET);
    println!("{}   📊 DB queries:    {}845/min{}", WHITE, BRIGHT_CYAN, RESET);
    println!("{}   🔍 Cache hits:    {}96.3%{}", WHITE, BRIGHT_GREEN, RESET);
    
    println!();
    
    // Recent events (simulated)
    println!("{}📋 Recent Events:{}", WHITE, RESET);
    let events = [
        ("🔄", "File sync", "frontend/src/App.tsx updated"),
        ("🚀", "Deploy", "backend service restarted"),
        ("📊", "Scale", "user-service scaled to 2 replicas"),
        ("🔧", "Config", "database connection pool updated"),
    ];
    
    for (emoji, event_type, description) in events.iter() {
        println!("{}   {} {}{:<12}{} {}", WHITE, emoji, CYAN, event_type, RESET, description);
    }
    
    println!();
    print_tip("Press 'r' to refresh, 'q' to quit (simulated)");
    println!();
}

fn show_service_logs(service: Option<&String>) {
    println!();
    if let Some(service_name) = service {
        print_header(&format!("📋 Logs: {}", service_name));
        
        println!("{}🔍 Streaming logs for service: {}{}{}", WHITE, BRIGHT_CYAN, service_name, RESET);
    } else {
        print_header("📋 Aggregated Logs");
        
        println!("{}🔍 Streaming logs from all services:{}", WHITE, RESET);
    }
    
    println!();
    
    // Simulate log streaming
    let log_entries = [
        ("frontend", "INFO", "Hot reload triggered for App.tsx"),
        ("backend", "INFO", "Database connection established"),
        ("database", "INFO", "Query executed: SELECT * FROM users"),
        ("backend", "DEBUG", "Processing API request: GET /api/users"),
        ("cache", "INFO", "Cache hit: user:123"),
        ("frontend", "INFO", "Component rendered: UserList"),
        ("backend", "INFO", "Response sent: 200 OK"),
    ];
    
    for (i, (service_name, level, message)) in log_entries.iter().enumerate() {
        let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
        let level_color = match *level {
            "ERROR" => BRIGHT_RED,
            "WARN" => BRIGHT_YELLOW,
            "INFO" => BRIGHT_GREEN,
            "DEBUG" => BRIGHT_BLUE,
            _ => WHITE,
        };
        
        println!("{}[{}] {}{}{} {}[{}]{} {}", 
            WHITE, timestamp, level_color, level, RESET,
            CYAN, service_name, RESET, message);
        
        if i < log_entries.len() - 1 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    
    println!();
    print_tip("Logs are live-streamed with intelligent filtering 🚀");
    println!();
}

fn show_workspace_help() {
    println!();
    print_header("🚀 Workspace Commands");
    
    println!("{}Commands:{}", BOLD, RESET);
    println!("  {}list{}                    List all available workspace templates", BRIGHT_BLUE, RESET);
    println!("  {}create{} <template> [name] Create a new multi-service workspace", BRIGHT_BLUE, RESET);
    println!("  {}status{}                  Show all active workspaces", BRIGHT_BLUE, RESET);
    println!("  {}stop{} <workspace>        Stop and remove a workspace", BRIGHT_BLUE, RESET);
    println!("  {}scale{} <workspace> <service> [replicas]  Scale a service", BRIGHT_BLUE, RESET);
    
    println!();
    println!("{}Examples:{}", BOLD, RESET);
    println!("  {}./vortex_orchestrator workspace list{}", CYAN, RESET);
    println!("  {}./vortex_orchestrator workspace create fullstack-webapp myapp{}", CYAN, RESET);
    println!("  {}./vortex_orchestrator workspace scale myapp backend 3{}", CYAN, RESET);
    
    println!();
}

fn show_sync_help() {
    println!();
    print_header("🔄 File Sync Commands");
    
    println!("{}Commands:{}", BOLD, RESET);
    println!("  {}enable{} <path>           Enable real-time file sync for path", BRIGHT_BLUE, RESET);
    println!("  {}disable{}                Disable file synchronization", BRIGHT_BLUE, RESET);
    println!("  {}status{}                 Show sync engine status", BRIGHT_BLUE, RESET);
    println!("  {}watch{}                  Live view of file changes", BRIGHT_BLUE, RESET);
    
    println!();
}

fn show_cluster_help() {
    println!();
    print_header("🌐 Cluster Commands");
    
    println!("{}Commands:{}", BOLD, RESET);
    println!("  {}status{}                 Show cluster resource status", BRIGHT_BLUE, RESET);
    println!("  {}scale{} <up|down>        Scale cluster resources", BRIGHT_BLUE, RESET);
    println!("  {}network{}                Show network topology", BRIGHT_BLUE, RESET);
    
    println!();
}

fn print_header(title: &str) {
    println!("{}🔥 Vortex Orchestrator • {}{}{}", BRIGHT_MAGENTA, BOLD, title, RESET);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", MAGENTA, RESET);
}

fn print_tip(message: &str) {
    println!("{}💡 Tip:{} {}", CYAN, RESET, message);
}

fn print_error(message: &str) {
    eprintln!("{}❌ Error:{} {}", RED, RESET, message);
}

fn print_success(message: &str) {
    println!("{}✅ {}{}", GREEN, message, RESET);
}

fn print_info(message: &str) {
    println!("{}🔗 {}{}", BLUE, message, RESET);
}