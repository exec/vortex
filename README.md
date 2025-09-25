# 🔥 Vortex - Lightning-fast Ephemeral VM Platform

A powerful Rust CLI for creating and managing ephemeral microVMs using krunvm on macOS. Perfect for isolated development, testing, and CI/CD workflows.

## ✨ Key Features

### ⚡ **Ultra-Fast Performance**
- **Sub-second boot**: VM startup in ~0.8 seconds
- **Dedicated resources**: Each VM gets guaranteed CPU and memory allocation
- **Zero daemon overhead**: Lightweight architecture with no background processes
- **Native performance**: Direct hardware access without virtualization penalties

### 🛡️ **Hardware-Level Security**
- **True isolation**: Hardware hypervisor boundaries provide fortress-level security
- **Escape-proof**: Impossible VM breakouts protect your host system
- **Multi-tenancy ready**: Safely run untrusted code without fear
- **Kernel protection**: Host kernel completely isolated from VM workloads
- **Supply chain safety**: Malicious images are trapped within VM boundaries

### 🚀 **Developer Experience**
- **Truly ephemeral**: VMs completely disappear after use - no cleanup needed
- **Bidirectional file sync**: Seamless data flow between host and VMs
- **Native integration**: Works naturally with your existing workflow
- **Clean slate guarantee**: Fresh kernel and userspace every time
- **Simple networking**: Intuitive port forwarding without complexity

### 💎 **Advanced Capabilities**
- **Full OS testing**: Run different kernels and test system-level components
- **Parallel execution**: Run workloads across multiple VMs simultaneously
- **Real-time monitoring**: Hardware-level performance metrics during execution
- **Smart caching**: Persistent dependency caching for faster subsequent runs
- **Malware analysis**: Safely execute and analyze suspicious code
- **Compliance ready**: Auditable boundaries for enterprise requirements

### 🎯 **Enterprise Reliability**
- **Predictable performance**: Resource guarantees without interference
- **Failure isolation**: VM crashes don't affect other workloads
- **Forensic capabilities**: Complete VM state capture and analysis
- **Regulatory compliance**: Hardware-enforced boundaries for strict requirements

## 🆚 Container Platform Comparison

Unlike container platforms that share the host kernel and rely on namespace isolation, Vortex provides true hardware-level virtualization. This fundamental difference enables capabilities impossible with containers: different kernel versions, system-level testing, guaranteed resource isolation, and security boundaries that can't be bypassed through kernel exploits.

## 🚀 Quick Install

```bash
git clone <repo-url>
cd firecracker-wrapper
cargo build --release
sudo cp target/release/vortex /usr/local/bin/
```

## 📖 Usage

### 🎯 Scriptable Execution (One-liner Workflows)

The most powerful feature - run isolated workloads with bidirectional file sync:

```bash
# Simple command execution (quiet mode shows only command output)
vortex run alpine -e "echo 'Hello World'" -q

# Copy project in, build it, sync results back
vortex run node:18 -q \
  --copy-to ./my-app:/workspace \
  --workdir /workspace \
  --sync-back /workspace/dist:./build-output \
  -e "npm install && npm run build"

# Isolated C compilation with dependency caching
vortex run alpine -q \
  --copy-to ./src:/build \
  --workdir /build \
  --sync-back /build/output:./compiled \
  --cache-deps \
  -e "apk add gcc && gcc -o output main.c"

# Python testing with smart caching
vortex run python:3.9 -q \
  --copy-to ./myproject:/app \
  --workdir /app \
  --cache-deps \
  -e "pip install -r requirements.txt && python -m pytest"
```

### 🚀 Parallel Multi-VM Execution

Run workloads across multiple VMs simultaneously:

```bash
# Test across different platforms in parallel
vortex parallel alpine ubuntu debian \
  -e "echo 'Testing on:' && uname -a && ./run-tests.sh" \
  --copy-to ./tests:/workspace \
  --sync-back /workspace/results:/test-results

# Parallel CI/CD pipeline
vortex parallel node:16 node:18 node:20 \
  -e "npm install && npm test" \
  --copy-to ./app:/workspace \
  --workdir /workspace
```

### 🖥️ Interactive Shell Mode

```bash
# Start interactive shell in VM
vortex shell alpine

# With custom resources and working directory
vortex shell ubuntu -m 2048 -c 4 -w /workspace \
  --copy-to ./project:/workspace

# Use with screen for detachable sessions
screen -S my-vm vortex shell alpine
# Ctrl+A+D to detach, screen -r my-vm to reattach
```

### 🏗️ Development Workflows

```bash
# Rust development environment
vortex shell rust:alpine -m 4096 \
  --copy-to ./src:/app/src \
  --copy-to ./Cargo.toml:/app/Cargo.toml \
  --workdir /app

# Node.js development with port forwarding
vortex shell node:18 -p 3000:3000 \
  --copy-to ./web-app:/workspace \
  --workdir /workspace

# Testing dangerous commands safely
vortex shell alpine -e "rm -rf /tmp/* && echo 'Safe in VM'"
```

### 📊 Performance Monitoring

```bash
# Real-time performance monitoring during execution
vortex run alpine -e "stress --cpu 2 --timeout 30s" --monitor-performance

# View VM metrics
vortex metrics
vortex metrics <vm-id>
```

### 📊 VM Management

```bash
# List running VMs
vortex list

# Stop specific VM
vortex stop <vm-id>

# Cleanup all VMs
vortex cleanup

# Show available templates
vortex templates
```

### 🎨 Templates

```bash
# Use predefined templates
vortex template dev
vortex template web --command "npm start"
vortex template minimal --command "apk update"
```

## ✨ Core Features

### 🎯 **Scriptable Execution**
- **One-liner workflows**: Complete CI/CD pipelines in single commands
- **Bidirectional file sync**: Copy files in, get results back automatically
- **Quiet mode (`-q`)**: Clean output perfect for automation
- **Working directory control**: Set initial PWD with `--workdir`
- **Smart dependency caching**: `--cache-deps` for faster subsequent runs

### ⚡ **Performance**
- **Lightning fast**: VMs boot in ~1 second using krunvm
- **Auto-cleanup**: VMs destroyed after execution with zero trace
- **Resource efficient**: Minimal overhead microVMs
- **Parallel execution**: Run multiple VMs simultaneously

### 🛠️ **Development Features**
- **Interactive shells**: Full terminal support with proper TTY
- **File synchronization**: `--copy-to` and `--sync-back` options
- **Port forwarding**: Expose VM services to host
- **Volume mounting**: Persistent file sharing
- **Custom resources**: Configure memory, CPU cores
- **Real-time monitoring**: Live performance metrics during execution

### 🏷️ **Image Management**
- **Built-in aliases**: Simple names for common images
- **Template system**: Predefined development environments
- **OCI compatibility**: Works with any container image

### 📈 **Monitoring & Metrics**
- **Real-time metrics**: CPU, memory, disk, network stats
- **VM listing**: See all running instances
- **Resource tracking**: Monitor usage across VMs
- **Performance insights**: Hardware-level visibility

## 🖥️ Built-in Images & Aliases

- `alpine` → `docker.io/library/alpine:latest`
- `ubuntu` → `docker.io/library/ubuntu:22.04`  
- `debian` → `docker.io/library/debian:latest`
- `node` → Node.js development environment
- `python` → Python development environment
- `rust` → Rust development environment

## 🎯 Use Cases

### **Isolated Development**
```bash
# Safe dependency testing
vortex shell node:18 --copy-to ./package.json:/app --workdir /app

# Cross-platform builds
vortex run ubuntu -q --copy-to ./src:/build --workdir /build \
  --sync-back /build/target:/tmp/linux-build \
  -e "apt update && apt install -y build-essential && make"
```

### **CI/CD Pipelines**
```bash
# Automated testing across versions
vortex parallel python:3.8 python:3.9 python:3.10 \
  --copy-to ./tests:/app/tests \
  --copy-to ./src:/app/src \
  --workdir /app \
  -e "pip install pytest && pytest tests/"

# Documentation generation
vortex run node:18 -q \
  --copy-to ./docs:/workspace \
  --sync-back /workspace/output:./generated-docs \
  --workdir /workspace \
  -e "npm install && npm run docs"
```

### **Security Testing**
```bash
# Test potentially dangerous code safely
vortex shell alpine --copy-to ./suspicious-script:/tmp \
  -e "chmod +x /tmp/script.sh && /tmp/script.sh"

# Malware analysis in isolation
vortex run alpine --copy-to ./malware:/analysis \
  --workdir /analysis \
  -e "file * && strings suspicious.exe"
```

### **System-Level Testing**
```bash
# Test kernel modules
vortex shell alpine -e "modprobe overlay && echo 'Module loaded'"

# Boot process testing
vortex shell ubuntu -e "systemctl --version && systemctl list-units"

# Network isolation verification
vortex run alpine -e "ip addr show && netstat -rn"
```

## 🔧 Configuration

Config auto-created at `~/.config/vortex/config.toml`:

```toml
[image_aliases]
myapp = "registry.local/myapp:latest"
dev = "ubuntu:22.04"

[[templates]]
[templates.fullstack]
image = "node:18"
memory = 4096
cpus = 2
ports = ["3000:3000", "8080:8080"]
volumes = ["./:/workspace"]
command = "bash"
description = "Full-stack development environment"

[templates.testing]
image = "python:3.9"
memory = 2048
cpus = 1
command = "pytest"
description = "Python testing environment"
```

## 🏗️ Architecture

**Modular Design:**
- `vortex-core` - Core VM management and abstractions
- `vortex-dev` - Development-focused extensions  
- `vortex-research` - Advanced research features

**Technology Stack:**
- **CLI**: `clap` for robust argument parsing
- **Backend**: Abstracted for krunvm/Firecracker support
- **Async**: `tokio` for non-blocking operations  
- **Config**: TOML-based templates and aliases
- **Logging**: Structured logging with `tracing`
- **Metrics**: Real-time VM resource monitoring

## 📦 What's New in v0.3.0

### 🆕 **New Features**
- ✅ **Parallel multi-VM execution** with `vortex parallel`
- ✅ **Real-time performance monitoring** with `--monitor-performance`
- ✅ **Advanced dependency caching** with `--cache-deps`
- ✅ **Scriptable execution** with `--copy-to` and `--sync-back`
- ✅ **Quiet mode (`-q`)** for clean automation output  
- ✅ **Working directory** control with `--workdir`
- ✅ **Enhanced shell mode** with proper file copying

### 🔧 **Improvements**
- ✅ **Modular architecture** with workspace structure
- ✅ **Better error handling** with graceful exits (Ctrl+D works perfectly)
- ✅ **Clean interface** optimized for both interactive and automated use
- ✅ **Comprehensive file operations** with validation
- ✅ **True ephemeral behavior** with complete VM cleanup

### 📝 **Recommendations**
- **For persistent/detachable VMs**: Use `screen` sessions for better experience
- **For nano editing**: Use `vi` instead due to terminal compatibility issues

## 🚧 Known Issues

- **Nano editor**: Enter key shows "Justified paragraph" (use `vi` instead)
- **macOS only**: Currently krunvm-based (Linux Firecracker support planned)
- **krunvm limitations**: Single command per VM, no true multi-session support

## 🤝 Contributing

This is a modular platform designed for extensibility. The workspace structure supports adding new backends, development tools, and research features.