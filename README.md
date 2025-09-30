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

### Auto-Discovery Workflow (Phase 5 - Current)
```bash
# Scan any project directory and auto-detect services
./vortex_orchestrator workspace init ./my-complex-project

# Creates vortex.yaml with detected services:
# frontend/ → Node.js VM
# backend/  → Python VM  
# worker/   → Go VM
# database/ → PostgreSQL VM

# Deploy the entire distributed workspace
./vortex_orchestrator workspace create --config vortex.yaml

# Monitor everything in real-time
./vortex_orchestrator monitor
```

### Manual Template Workflow  
```bash
# Create pre-built workspace templates
./vortex_orchestrator workspace create fullstack-webapp myapp
./vortex_orchestrator workspace create microservices-api enterprise  
./vortex_orchestrator workspace create ai-ml-pipeline research

# Session management (like tmux for VMs)
./vortex_quick sessions                    # List background VMs
./vortex_quick attach my-session          # Connect to running VM
./vortex_quick create python test         # Create new session
```

## 🔍 Project Auto-Discovery

Vortex automatically detects project structure and suggests optimal VM configurations:

| File Found | Language | Suggested Image | Default Ports |
|------------|----------|----------------|---------------|
| `package.json` | Node.js | `node:18-alpine` | 3000, 3001 |
| `requirements.txt` | Python | `python:3.11-slim` | 8000, 8001 |
| `go.mod` | Go | `golang:1.21-alpine` | 8080, 8081 |
| `Cargo.toml` | Rust | `rust:1.70` | 8080 |
| `composer.json` | PHP | `php:8.2-fpm-alpine` | 9000 |
| `Gemfile` | Ruby | `ruby:3.2-alpine` | 3000 |

**Service Type Detection:**
- `frontend/`, `ui/`, `web/` → Frontend service with hot reload
- `backend/`, `api/`, `server/` → Backend API service
- `worker/`, `jobs/`, `tasks/` → Background worker service
- `database/`, `migrations/` → Database service

## 🎪 Workspace Templates

### 🌐 Full-Stack Web App
- ⚛️ **Frontend**: React with hot reload (3000:3000)
- 🐍 **Backend**: FastAPI with auto-reload (8000:8000)
- 🐘 **Database**: PostgreSQL with persistence (5432:5432)
- 🔴 **Cache**: Redis for sessions (6379:6379)

### 🔬 Microservices Platform
- 🚪 **API Gateway**: Load balancing and routing (8080:8080)
- 👤 **User Service**: Go microservice (8001:8000)
- 📦 **Order Service**: Go microservice (8002:8000)
- 📡 **Message Queue**: NATS for service communication (4222:4222)
- 🍃 **Database**: MongoDB for microservices (27017:27017)

### 🤖 AI/ML Pipeline
- 📓 **Jupyter Lab**: TensorFlow with GPU support (8888:8888)
- 🧠 **ML API**: FastAPI model serving (8000:8000)
- ⚙️ **Data Processor**: ETL pipeline
- 🐘 **Database**: PostgreSQL for ML data (5432:5432)
- 🔴 **Cache**: Redis for job queues (6379:6379)

## 🛠 Installation

### Prerequisites
- macOS or Linux
- [krunvm](https://github.com/containers/krunvm) installed
- Docker/Podman for container images

### Install Vortex
```bash
git clone https://github.com/exec/vortex.git
cd vortex
cargo build --release

# Use the session manager directly
./vortex_quick sessions
./vortex_quick create python myproject

# Use the orchestrator for complex workspaces  
./vortex_orchestrator workspace init ./my-project
./vortex_orchestrator monitor
```

## 🔄 File Synchronization

Vortex provides real-time bidirectional file sync between your host and VMs:

```bash
# Enable live file sync
./vortex_orchestrator sync enable ./my-project

# Watch file changes in real-time
./vortex_orchestrator sync watch

# Check sync status
./vortex_orchestrator sync status
```

## 📊 Monitoring & Logs

```bash
# Real-time dashboard with service health
./vortex_orchestrator monitor

# Aggregated logs from all services
./vortex_orchestrator logs

# Service-specific logs
./vortex_orchestrator logs frontend

# Cluster resource utilization
./vortex_orchestrator cluster status
./vortex_orchestrator cluster scale up
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

See [ROADMAP.md](ROADMAP.md) for detailed development plans.

## 🌟 Examples

### Auto-Discovering a Complex Project
```bash
cd ~/my-ecommerce-platform
./vortex_orchestrator workspace init .

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
./vortex_orchestrator workspace status

# Output:
# 🔥 Found 3 active workspaces:
# 1. 🔥 crazy-ecommerce (3 services)
# 2. 🔥 enterprise-platform (5 services)  
# 3. 🔥 my-project (4 services)

# Stop entire workspace
./vortex_orchestrator workspace stop my-project
```

### Individual Session Management
```bash
# List all background sessions
./vortex_quick sessions

# Create new development session
./vortex_quick create python myproject

# Attach to running session
./vortex_quick attach myproject

# Stop individual session
./vortex_quick stop myproject
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

See [ROADMAP.md](ROADMAP.md) for planned features and [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

---

*"The future of development is distributed, isolated, and context-aware. Vortex makes it reality."* 🔥