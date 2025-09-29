#!/usr/bin/env rust-script
//! 🚀 Vortex Quick Session Manager - Beautiful CLI Edition
use std::process::Command;

// ANSI color codes for beautiful output
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        show_help();
        return;
    }
    
    match args[1].as_str() {
        "sessions" | "list" => list_sessions(),
        "create" => {
            if args.len() < 3 {
                eprintln!("Usage: {} create <template> [name]", args[0]);
                return;
            }
            let template = &args[2];
            let name = args.get(3).map(|s| s.as_str());
            create_session(template, name);
        }
        "attach" => {
            if args.len() < 3 {
                eprintln!("Usage: {} attach <vm-name>", args[0]);
                return;
            }
            attach_session(&args[2]);
        }
        "stop" => {
            if args.len() < 3 {
                eprintln!("Usage: {} stop <vm-name>", args[0]);
                return;
            }
            stop_session(&args[2]);
        }
        "templates" => list_templates(),
        "help" | "-h" | "--help" => show_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            show_help();
        }
    }
}

fn list_sessions() {
    println!();
    print_header("Background Sessions");
    
    match Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .arg("list")
        .output() 
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let vortex_vms: Vec<_> = stdout
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.starts_with("vortex-") {
                        Some(line.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            
            if vortex_vms.is_empty() {
                println!("{}📭 No background sessions found.{}", YELLOW, RESET);
                println!("{}💡 Create one with:{} {}vortex dev <template> --name <name> --detach{}", 
                    CYAN, RESET, BRIGHT_BLUE, RESET);
            } else {
                println!("{}Found {} active session{}:{}", 
                    WHITE, vortex_vms.len(), if vortex_vms.len() == 1 { "" } else { "s" }, RESET);
                println!();
                
                for (i, vm) in vortex_vms.iter().enumerate() {
                    let session_name = vm.strip_prefix("vortex-").unwrap_or(vm);
                    println!("{}{}. {}{}{} {}{}{}", 
                        BRIGHT_BLUE, i + 1, 
                        BRIGHT_GREEN, session_name, RESET,
                        WHITE, get_session_status(), RESET);
                    println!("   {}└─ VM: {}{}{} • {}512MB RAM{} • {}2 CPUs{}", 
                        WHITE, CYAN, vm, RESET,
                        YELLOW, RESET,
                        YELLOW, RESET);
                    if i < vortex_vms.len() - 1 {
                        println!();
                    }
                }
                
                println!();
                print_tip(&format!("{}vortex_quick attach <session-name>{}", BRIGHT_BLUE, RESET));
            }
        },
        Err(e) => print_error(&format!("Error listing sessions: {}", e)),
    }
    println!();
}

fn get_session_status() -> &'static str {
    "🟢 Running"
}

fn print_header(title: &str) {
    println!("{}🚀 Vortex • {}{}{}", BRIGHT_MAGENTA, BOLD, title, RESET);
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", MAGENTA, RESET);
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

// Template definitions - the smart template system!
struct Template {
    name: &'static str,
    image: &'static str,
    description: &'static str,
    ports: &'static [u16],
    tools: &'static [&'static str],
    emoji: &'static str,
}

const TEMPLATES: &[Template] = &[
    Template {
        name: "python",
        image: "python:3.11-slim",
        description: "Python development with pip, venv, and common tools",
        ports: &[8000, 5000],
        tools: &["pip", "venv", "pytest", "black", "flake8"],
        emoji: "🐍",
    },
    Template {
        name: "node",
        image: "node:18-alpine",
        description: "Node.js with npm, yarn, and development tools",
        ports: &[3000, 8080],
        tools: &["npm", "yarn", "nodemon", "eslint", "prettier"],
        emoji: "📦",
    },
    Template {
        name: "rust",
        image: "rust:1.70",
        description: "Rust development with cargo and clippy",
        ports: &[8080],
        tools: &["cargo", "rustc", "clippy", "rustfmt"],
        emoji: "🦀",
    },
    Template {
        name: "go",
        image: "golang:1.21-alpine",
        description: "Go development with modules and tools",
        ports: &[8080, 8000],
        tools: &["go", "gofmt", "golint"],
        emoji: "🔵",
    },
    Template {
        name: "ubuntu",
        image: "ubuntu:22.04",
        description: "Clean Ubuntu environment for any project",
        ports: &[],
        tools: &["apt", "curl", "wget", "git", "vim"],
        emoji: "🐧",
    },
    Template {
        name: "alpine",
        image: "alpine:latest",
        description: "Minimal Alpine Linux for lightweight development",
        ports: &[],
        tools: &["apk", "ash", "curl", "wget"],
        emoji: "🏔️",
    },
];

fn get_template(name: &str) -> Option<&Template> {
    TEMPLATES.iter().find(|t| t.name == name)
}

fn generate_smart_name(template: &str, user_name: Option<&str>) -> String {
    if let Some(name) = user_name {
        name.to_string()
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        format!("{}-{}", template, timestamp % 10000)
    }
}

fn create_session(template_name: &str, user_name: Option<&str>) {
    println!();
    print_header("Create Session");
    
    let template = match get_template(template_name) {
        Some(t) => t,
        None => {
            print_error(&format!("Unknown template: {}", template_name));
            println!();
            println!("{}Available templates:{}", WHITE, RESET);
            for t in TEMPLATES {
                println!("  {} {}", t.emoji, t.name);
            }
            println!();
            print_tip("Use './vortex_quick templates' to see all available templates");
            return;
        }
    };
    
    let session_name = generate_smart_name(template_name, user_name);
    let vm_name = format!("vortex-{}", session_name);
    
    println!("{}Creating session:{} {}{}{}", WHITE, RESET, BRIGHT_GREEN, session_name, RESET);
    println!("{}Template:{} {}{} {}", WHITE, RESET, template.emoji, template.name, template.description);
    println!("{}Image:{} {}{}{}", WHITE, RESET, CYAN, template.image, RESET);
    
    if !template.tools.is_empty() {
        println!("{}Tools:{} {}", WHITE, RESET, template.tools.join(", "));
    }
    
    if !template.ports.is_empty() {
        let ports: Vec<String> = template.ports.iter().map(|p| p.to_string()).collect();
        println!("{}Ports:{} {}", WHITE, RESET, ports.join(", "));
    }
    
    println!();
    println!("{}🚀 Pulling image and creating VM...{}", YELLOW, RESET);
    
    let status = Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .args(["create", template.image, "--name", &vm_name])
        .status();
    
    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                print_success(&format!("Session '{}' created successfully!", session_name));
                println!();
                print_tip(&format!("{}./vortex_quick attach {}{}", BRIGHT_BLUE, session_name, RESET));
            } else {
                print_error(&format!("Failed to create session '{}'", session_name));
            }
        },
        Err(e) => print_error(&format!("Error creating session: {}", e)),
    }
    println!();
}

fn list_templates() {
    println!();
    print_header("Available Templates");
    
    println!("{}Choose from {} powerful development environments:{}", WHITE, TEMPLATES.len(), RESET);
    println!();
    
    for (i, template) in TEMPLATES.iter().enumerate() {
        println!("{}{}. {}{} {}{}{}", 
            BRIGHT_BLUE, i + 1, 
            template.emoji, template.name,
            WHITE, template.description, RESET);
        
        println!("   {}└─ Image: {}{}{}", WHITE, CYAN, template.image, RESET);
        
        if !template.tools.is_empty() {
            println!("   {}└─ Tools: {}{}{}", WHITE, YELLOW, template.tools.join(", "), RESET);
        }
        
        if !template.ports.is_empty() {
            let ports: Vec<String> = template.ports.iter().map(|p| p.to_string()).collect();
            println!("   {}└─ Default ports: {}{}{}", WHITE, GREEN, ports.join(", "), RESET);
        }
        
        if i < TEMPLATES.len() - 1 {
            println!();
        }
    }
    
    println!();
    print_tip(&format!("{}./vortex_quick create <template> [name]{}", BRIGHT_BLUE, RESET));
    println!();
}

fn attach_session(vm_name: &str) {
    let full_vm_name = if vm_name.starts_with("vortex-") {
        vm_name.to_string()
    } else {
        format!("vortex-{}", vm_name)
    };
    
    println!();
    print_info(&format!("Attaching to session: {}{}{}", BRIGHT_GREEN, full_vm_name, RESET));
    println!("{}💫 Entering VM environment...{}", YELLOW, RESET);
    println!("{}🚪 Press Ctrl+D or type 'exit' to detach{}", WHITE, RESET);
    println!();
    
    let status = Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .args(["start", &full_vm_name])
        .status();
    
    match status {
        Ok(_) => {
            println!();
            print_success("Session detached successfully");
        },
        Err(e) => print_error(&format!("Failed to attach to session '{}': {}", full_vm_name, e)),
    }
}

fn stop_session(vm_name: &str) {
    let full_vm_name = if vm_name.starts_with("vortex-") {
        vm_name.to_string()
    } else {
        format!("vortex-{}", vm_name)
    };
    
    println!();
    println!("{}⚠️  Warning:{} This will permanently delete session: {}{}{}", 
        YELLOW, RESET, BRIGHT_GREEN, full_vm_name, RESET);
    
    let status = Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .args(["delete", &full_vm_name])
        .status();
    
    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                print_success(&format!("Session '{}' stopped and removed", full_vm_name));
            } else {
                print_error(&format!("Failed to stop session '{}'", full_vm_name));
            }
        },
        Err(e) => print_error(&format!("Error stopping session: {}", e)),
    }
}

fn show_help() {
    println!();
    print_header("Session Manager Help");
    
    println!("{}Commands:{}", BOLD, RESET);
    println!("  {}templates{}          List available development templates", BRIGHT_BLUE, RESET);
    println!("  {}create{} <template> [name]  Create a new session from template", BRIGHT_BLUE, RESET);
    println!("  {}sessions{}, {}list{}     List background sessions", BRIGHT_BLUE, RESET, BRIGHT_BLUE, RESET);
    println!("  {}attach{} <session>   Attach to a session", BRIGHT_BLUE, RESET);
    println!("  {}stop{} <session>     Stop and remove a session", BRIGHT_BLUE, RESET);
    println!("  {}help{}               Show this help", BRIGHT_BLUE, RESET);
    
    println!();
    println!("{}Examples:{}", BOLD, RESET);
    println!("  {}./vortex_quick templates{}", CYAN, RESET);
    println!("  {}./vortex_quick create python myproject{}", CYAN, RESET);
    println!("  {}./vortex_quick create rust{} {}# Auto-generates name{}", CYAN, RESET, WHITE, RESET);
    println!("  {}./vortex_quick sessions{}", CYAN, RESET);
    println!("  {}./vortex_quick attach myproject{}", CYAN, RESET);
    println!("  {}./vortex_quick stop myproject{}", CYAN, RESET);
    
    println!();
    print_tip("Session names can omit the 'vortex-' prefix for convenience");
    print_tip("If no name provided, smart auto-naming will be used");
    println!();
}