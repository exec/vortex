# 🚀 Vortex

A next-generation distributed development platform that automatically maps project directories to optimized VMs with intelligent orchestration.

[![Tests](https://github.com/exec/vortex/actions/workflows/test.yml/badge.svg)](https://github.com/exec/vortex/actions/workflows/test.yml)
[![Quick Tests](https://github.com/exec/vortex/actions/workflows/quick-test.yml/badge.svg)](https://github.com/exec/vortex/actions/workflows/quick-test.yml)
[![Deploy](https://github.com/exec/vortex/actions/workflows/deploy.yml/badge.svg)](https://github.com/exec/vortex/actions/workflows/deploy.yml)
[![Security Audit](https://img.shields.io/badge/security-audited-green.svg)](https://github.com/exec/vortex/actions/workflows/test.yml)

## 🎯 What Makes Vortex Revolutionary

- 🔍 **Auto-Discovery** - Scans your project and automatically detects services (frontend, backend, database, workers)
- 🎪 **Directory-to-VM Mapping** - Each service runs in its own optimized microVM with perfect isolation
- ⚡ **Lightning-Fast Setup** - From `git clone` to running distributed environment in under 5 minutes
- 🎨 **Beautiful CLI** - Rich colors, emojis, and intuitive commands that make development fun
- 🔧 **Multi-Service Orchestration** - Manage complex workspaces with multiple interconnected services
- 📊 **Real-time Monitoring** - Live dashboards showing service health, metrics, and logs

## 🚀 Quick Start

### Interactive Auto-Discovery (Phase 5 - Current)
```bash
# Start interactive workspace setup
vortex workspace init

# Answer a few questions about your project structure
# The system will auto-detect languages, services, and generate a configuration

# For non-interactive use:
vortex workspace init ./my-project --non-interactive
```

### Manual Workspace Creation
```bash
# Create from pre-built templates
vortex workspace create fullstack-webapp myapp
vortex workspace create microservices-api enterprise
vortex workspace create ai-ml-pipeline research
```

### Session Management
```bash
vortex session list                       # List background VMs
vortex session attach my-session          # Connect to running VM
vortex session create python test         # Create new session
```

## 🏗️ Workspace Initialization

Vortex's workspace initialization provides two modes for setting up your development environment:

### Interactive Mode
```bash
vortex workspace init
```

This mode walks you through creating a workspace with guided questions:
1. **Project name** - What should we call your project?
2. **Service types** - Select from frontend, backend, worker, database, cache, queue
3. **Languages** - Automatically detected from project files
4. **Port configurations** - Set default ports for each service

### Non-Interactive Mode
```bash
# Auto-scan current directory
vortex workspace init --non-interactive

# Auto-scan specific directory
vortex workspace init ./my-complex-project --non-interactive

# Custom output location
vortex workspace init --non-interactive --output ./my-vortex.yaml
```

### Generated Configuration
The system creates a `vortex.yaml` file like this:
```yaml
name: my-project
description: Auto-generated workspace for my-project

services:
  frontend:
    type: frontend
    language: node
    image: node:18-alpine
    ports:
      - 3000:3000
    path: ./frontend

  backend:
    type: backend
    language: python
    image: python:3.11-slim
    ports:
      - 8000:8000
    path: ./backend
```

## 🔍 Project Auto-Discovery

Vortex automatically detects project structure and suggests optimal VM configurations:

### Language Detection

| File Found | Language | Suggested Image | Default Ports |
|------------|----------|----------------|---------------|
| `package.json` | Node.js | `node:18-alpine` | 3000, 3001 |
| `requirements.txt` | Python | `python:3.11-slim` | 8000, 8001 |
| `go.mod` | Go | `golang:1.21-alpine` | 8080, 8081 |
| `Cargo.toml` | Rust | `rust:1.70` | 8080 |
| `composer.json` | PHP | `php:8.2-fpm-alpine` | 9000 |
| `Gemfile` | Ruby | `ruby:3.2-alpine` | 3000 |
| `build.sbt` | Scala | `scala:3.3-alpine` | 8080 |
| `pom.xml` | Java | `openjdk:17-slim` | 8080 |

### Service Type Detection

| Directory | Service Type | Features |
|-----------|--------------|----------|
| `frontend/`, `ui/`, `web/` | Frontend | Hot reload, static serving |
| `backend/`, `api/`, `server/` | Backend | API routing, middleware |
| `worker/`, `jobs/`, `tasks/` | Worker | Background processing |
| `database/`, `migrations/` | Database | Persistent storage |
| `cache/`, `redis/` | Cache | Fast key-value storage |

### Service Port Mapping

When Vortex detects services, it automatically maps common ports:
- Frontend: 3000, 8080, 5173 (Vite), 3001
- Backend: 8000, 8080, 3000, 5000
- Database: 5432 (PostgreSQL), 3306 (MySQL), 27017 (MongoDB)
- Cache: 6379 (Redis), 11211 (Memcached)

## 🎪 Workspace Templates

Vortex provides pre-configured templates for common development stacks:

### 🐍 Python Template
Complete Python development environment with:
- Python 3.11 slim base image
- pip, virtualenv, and debugging tools
- Default port: 8000 (FastAPI/Flask)
- IDE extensions: Python, DebugPy

### 🟦 Node.js Template
Node.js development with:
- Node 18 Alpine base image
- npm, yarn, and development tools
- Default ports: 3000, 8080, 9229 (Node Inspector)
- IDE extensions: TypeScript Next

### 🦀 Rust Template
Rust development environment:
- Rust 1.75 slim base image
- cargo, rustc, rustfmt, clippy
- Default port: 8000
- IDE extensions: rust-analyzer

### ⚡ Go Template
Go development with:
- Golang 1.21 Alpine base image
- go, gofmt, git
- Default ports: 8080, 2345 (Delve debugger)
- IDE extensions: Go

### 🤖 AI/ML Template
AI/ML development environment:
- Python 3.11 slim base
- PyTorch, TensorFlow, Jupyter
- Default ports: 8888 (Jupyter), 6006 (TensorBoard)
- IDE extensions: Python, Jupyter

## 🛠 Installation

### Prerequisites
- macOS or Linux (Windows support coming soon)
- For VM creation: [krunvm](https://github.com/containers/krunvm) or Firecracker
- Docker/Podman for container images (optional for development)

### Install Vortex

#### From Source
```bash
git clone https://github.com/exec/vortex.git
cd vortex
cargo build --release

# Install to system path
sudo cp target/release/vortex /usr/local/bin/

# Verify installation
vortex --version
```

#### Using the Install Script
```bash
curl -fsSL https://raw.githubusercontent.com/exec/vortex/main/install.sh | bash
```

### Backend Options

Vortex supports multiple VM backends:

| Backend | Description | Installation |
|---------|-------------|--------------|
| **krunvm** | Lightweight VM runtime for Linux | `cargo install krunvm` or follow [krunvm docs](https://github.com/containers/krunvm) |
| **firecracker** | AWS microVM runtime | Follow [Firecracker docs](https://github.com/firecracker-microvm/firecracker) |

### Config-Only Operations
Vortex can generate workspace configurations without a backend:
```bash
# Generate vortex.yaml without any backend installed
vortex workspace init --non-interactive
vortex workspace create myapp --template python

# Use --backend flag to specify backend
vortex workspace init --backend krunvm
vortex workspace init --backend firecracker
```

### Verify Installation
```bash
# Show version
vortex --version

# List available commands
vortex help

# List templates
vortex templates

# Initialize a new workspace
vortex workspace init
```

## 🔄 File Synchronization

Vortex provides real-time bidirectional file sync between your host and VMs:

```bash
# Enable live file sync
vortex workspace sync enable ./my-project

# Watch file changes in real-time
vortex workspace sync watch

# Check sync status
vortex workspace sync status
```

## 📊 Monitoring & Logs

```bash
# Real-time dashboard with service health
vortex workspace monitor

# Aggregated logs from all services
vortex workspace logs

# Service-specific logs
vortex workspace logs frontend

# Cluster resource utilization
vortex workspace cluster status
vortex workspace cluster scale up
```

## 🎯 Architecture

### Phase 5 (Current): Directory-to-VM Mapping
- ✅ **Auto-discovery** of project structure
- ✅ **Language detection** and optimal image selection
- ✅ **Service type inference** from directory patterns
- ✅ **YAML configuration** generation
- ✅ **Integrated orchestration** with existing templates

### Upcoming Phases
- **Phase 6**: Context-aware environments (dev/staging/prod)
- **Phase 7**: Real file synchronization engine
- **Phase 8**: Editor integration (VS Code, Neovim, etc.)
- **Phase 9**: Advanced orchestration (dependencies, scaling)
- **Phase 10**: Cloud deployment and team collaboration

See [TODO.md](TODO.md) for detailed development plans.

## 🌟 Examples

### Auto-Discovering a Complex Project
```bash
cd ~/my-ecommerce-platform
vortex workspace init .

# Output:
# 🔍 Discovered project structure:
#   frontend/     → Node.js (package.json detected)
#   api/          → Python (requirements.txt detected)  
#   worker/       → Go (go.mod detected)
#   database/     → PostgreSQL (migrations/ detected)
#
# ✅ Configuration saved to: vortex.yaml
# 💡 Run: vortex workspace create --config vortex.yaml
```

### Managing Distributed Workspaces
```bash
# Show all active workspaces
vortex workspace status

# Output:
# 🔥 Found 3 active workspaces:
# 1. 🔥 crazy-ecommerce (3 services)
# 2. 🔥 enterprise-platform (5 services)  
# 3. 🔥 my-project (4 services)

# Stop entire workspace
vortex workspace stop my-project
```

### Individual Session Management
```bash
# List all background sessions
vortex session list

# Create new development session
vortex session create python myproject

# Attach to running session
vortex session attach myproject

# Stop individual session
vortex session stop myproject
```

## 🎯 Use Cases

### **Distributed Development**
- **Multi-service applications**: Each service in its own VM with perfect isolation
- **Language polyglot projects**: Different runtime environments for each component
- **Microservice development**: True service boundaries with networking

### **Team Collaboration**
- **Shared development environments**: Consistent setup across team members
- **Environment parity**: Dev environments match staging and production
- **Onboarding**: New developers productive in minutes, not hours

### **Enterprise Development**
- **Compliance requirements**: Hardware-level isolation for sensitive projects
- **Resource allocation**: Dedicated CPU/memory per service
- **Security boundaries**: True isolation between components

## 🔧 Legacy Features (v0.3.0)

Vortex retains its powerful single-VM capabilities:

### **Scriptable Execution**
```bash
# One-liner workflows with file sync
vortex run alpine -e "echo 'Hello World'" -q

# Copy project in, build it, sync results back
vortex run node:18 -q \
  --copy-to ./my-app:/workspace \
  --workdir /workspace \
  --sync-back /workspace/dist:./build-output \
  -e "npm install && npm run build"
```

### **Interactive Development**
```bash
# Start interactive shell in VM
vortex shell alpine

# Development environment with port forwarding
vortex shell node:18 -p 3000:3000 \
  --copy-to ./web-app:/workspace \
  --workdir /workspace
```

### **Parallel Execution**
```bash
# Test across different platforms
vortex parallel alpine ubuntu debian \
  -e "echo 'Testing on:' && uname -a" \
  --copy-to ./tests:/workspace
```

## 🧪 Testing & Quality Assurance

Vortex maintains comprehensive test coverage across all features:

### **Running Tests**
```bash
# Comprehensive test suite
./test_runner.sh

# Individual test categories
cargo test --test cli_integration_test --release
cargo test --test workspace_integration_tests --release
```

### **CI/CD Pipeline**
- ✅ Multi-platform builds (Linux, macOS)
- ✅ Security auditing and vulnerability scanning
- ✅ Performance benchmarking
- ✅ Automated deployment with validated artifacts

## 🤝 Contributing

Vortex is building the future of distributed development environments:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make changes with comprehensive tests
4. Run `./test_runner.sh` to validate
5. Submit a Pull Request

See [TODO.md](TODO.md) for planned features and [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 📚 CLI Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `vortex --help` | Show help message |
| `vortex --version` | Show version information |
| `vortex --verbose` | Enable verbose logging |

### Workspace Commands

| Command | Description |
|---------|-------------|
| `vortex workspace init` | Initialize workspace interactively |
| `vortex workspace init --non-interactive` | Auto-scan and generate config |
| `vortex workspace init --backend krunvm` | Specify backend (krunvm/firecracker) |
| `vortex workspace init --output ./path/to/config.yaml` | Custom output path |
| `vortex workspace create <name>` | Create new workspace from template |
| `vortex workspace create <name> --template python` | Specify template |
| `vortex workspace create <name> --backend firecracker` | Specify backend |
| `vortex workspace list` | List all workspaces |
| `vortex workspace info <name>` | Show workspace details |
| `vortex workspace delete <name>` | Delete workspace |

### Dev Commands

| Command | Description |
|---------|-------------|
| `vortex dev <template>` | Create dev environment |
| `vortex dev <template> --name <name>` | Named session |
| `vortex dev <template> --workspace <name>` | Use persistent workspace |
| `vortex dev <template> --detach` | Run in background |
| `vortex dev --list` | List available templates |
| `vortex dev --init` | Initialize from current directory |
| `vortex dev <template> --port 8080:8080` | Port forwarding |
| `vortex dev <template> --volume ./src:/workspace` | Volume mount |

### Session Commands

| Command | Description |
|---------|-------------|
| `vortex session create <template> <name>` | Create new session |
| `vortex session list` | List all sessions |
| `vortex session info <id>` | Show session details |
| `vortex session start <id>` | Start stopped session |
| `vortex session stop <id>` | Stop running session |
| `vortex session attach <id>` | Attach to session |
| `vortex session delete <id>` | Delete session |

### Single-VM Commands

| Command | Description |
|---------|-------------|
| `vortex run <image>` | Run single ephemeral VM |
| `vortex run <image> --command "echo hello"` | Run command |
| `vortex run <image> -p 8080:8080` | Port forwarding |
| `vortex shell <image>` | Interactive shell |
| `vortex templates` | Show available templates |

### Daemon Commands

| Command | Description |
|---------|-------------|
| `vortex daemon start` | Start background daemon |
| `vortex daemon stop` | Stop daemon |
| `vortex daemon status` | Show daemon status |
| `vortex daemon logs` | Show daemon logs |

### Session Management Commands

| Command | Description |
|---------|-------------|
| `vortex list` | List running VMs |
| `vortex stop <vm_id>` | Stop VM |
| `vortex cleanup` | Stop all running VMs |
| `vortex attach <session>` | Attach to session |
| `vortex metrics <vm_id>` | Show VM metrics |
| `vortex parallel [images...]` | Run across multiple VMs |

---

*"The future of development is distributed, isolated, and context-aware. Vortex makes it reality."* 🔥