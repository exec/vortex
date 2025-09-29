# Vortex Python Web App Example

This example demonstrates the power of Vortex for Python development environments.

## 🚀 Quick Start with Vortex

```bash
# Navigate to this directory
cd examples/python-webapp

# Create instant Python environment (2-3 seconds vs 60-100s with Docker!)
vortex dev python

# Inside the environment, install dependencies and run
pip install -r requirements.txt
python app.py
```

## 🌐 Access the Application

Once running, visit:
- **Web Interface**: http://localhost:8000
- **Status API**: http://localhost:8000/api/status  
- **Environment API**: http://localhost:8000/api/env
- **Test API**: http://localhost:8000/api/test

## ⚡ Performance Comparison

| Environment Setup | Startup Time | Developer Experience |
|-------------------|--------------|---------------------|
| **Vortex** | ~2-3 seconds | ✅ Instant productivity |
| Docker DevContainer | ~60-100 seconds | ❌ Coffee break required |

## 🔥 What This Demonstrates

- **Instant Environment**: Python environment ready in seconds
- **Port Forwarding**: Automatic port mapping (8000:8000)
- **File System Access**: Direct filesystem access without container overhead
- **Network Connectivity**: Full network access for dependencies
- **Hardware Isolation**: True VM-level security vs container namespaces
- **Development Workflow**: Real development environment, not a toy example

## 🧪 Testing Commands

```bash
# Test the APIs from your host machine
curl http://localhost:8000/api/status
curl http://localhost:8000/api/env  
curl http://localhost:8000/api/test

# Check environment inside Vortex
vortex dev python --workspace my-webapp
pip install -r requirements.txt
python -c "import flask; print('Flask version:', flask.__version__)"
```

## 🎯 Try Different Scenarios

1. **Persistent Workspace**: `vortex dev python --workspace persistent-webapp`
2. **Custom Ports**: `vortex dev python --port 5000:5000`  
3. **Volume Mounting**: `vortex dev python --volume ./data:/app/data`
4. **Multiple Environments**: Run several instances simultaneously

## 🔄 DevContainer Migration

If you have an existing DevContainer setup:

```bash
# Vortex can import your DevContainer configuration
vortex workspace import . --name imported-webapp

# Then start the imported environment  
vortex dev --workspace imported-webapp
```

## 🚀 Production Benefits

- **20x Faster**: Instant environment vs slow container builds
- **True Isolation**: Hardware-level VM boundaries
- **Better Security**: No shared kernel vulnerabilities  
- **Native Performance**: Direct hardware access
- **Easy Migration**: Import existing Docker workflows seamlessly

This example shows why developers are switching from Docker DevContainers to Vortex for their daily development workflow!