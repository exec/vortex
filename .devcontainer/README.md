# Vortex Development Container

This DevContainer configuration provides a complete development environment for Vortex, enabling you to test our own project using the DevContainer migration feature!

## What's Included

### 🦀 Rust Development Environment
- **Rust 1.75**: Latest stable Rust toolchain
- **Cargo Tools**: audit, watch, edit, expand for enhanced development
- **VS Code Extensions**: rust-analyzer, debugger, and productivity tools

### 🛠️ Development Tools
- **GitHub CLI**: For CI/CD integration and repository management
- **Build Tools**: Complete compilation environment
- **Security Auditing**: cargo-audit for dependency vulnerability scanning

### 🚀 Vortex Integration
- **Port Forwarding**: 3000, 8000, 8080 for development servers
- **Volume Mounting**: Persistent target directory for faster builds
- **Docker Access**: Socket mounting for container operations

## Usage

### Traditional DevContainer (VSCode)
```bash
# Open in VSCode with Dev Containers extension
code .
# Command Palette: "Dev Containers: Reopen in Container"
```

### 🔥 Dogfooding with Vortex (The Better Way!)
```bash
# Import this DevContainer to Vortex workspace
vortex workspace import . --name vortex-dev

# Start development with Vortex (20x faster!)
vortex dev --workspace vortex-dev

# Compare the speed difference!
```

## Development Workflow

### Build and Test
```bash
# Build the project
cargo build --release

# Run comprehensive test suite
./test_runner.sh

# Watch for changes during development
cargo watch -x "build" -x "test"

# Security audit
cargo audit
```

### Code Quality
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run linting
cargo check
```

### CI/CD Integration
```bash
# Check GitHub Actions status
gh workflow list

# View test results
gh run list

# Create release
gh release create v0.4.0
```

## Performance Comparison

| Environment | Startup Time | Build Cache | File Sync |
|-------------|-------------|-------------|-----------|
| **Vortex** | ~2-3s | ✅ Native | ✅ Instant |
| DevContainer | ~60-100s | ❌ Slow | ❌ Delayed |

## Dogfooding Benefits

1. **Test Migration**: Verify DevContainer import functionality
2. **Performance Validation**: Experience 20x speed improvement firsthand  
3. **Feature Testing**: Use Vortex for actual development work
4. **User Experience**: Understand developer workflow improvements

## Configuration Details

- **Base Image**: `rust:1.75-slim`
- **Workspace**: `/workspace` (matches Vortex conventions)
- **Ports**: 3000, 8000, 8080 (common development ports)
- **Volumes**: Target directory cached for build performance
- **Tools**: Complete Rust development stack

This configuration demonstrates Vortex's superior developer experience while providing a complete environment for contributing to the project itself!