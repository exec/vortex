# Contributing to Vortex

Thank you for your interest in contributing to Vortex! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites
- **Rust**: Latest stable version (1.70+)
- **macOS**: Currently required for krunvm support
- **Git**: For version control

### Development Setup
```bash
# Clone the repository
git clone https://github.com/exec/vortex.git
cd vortex

# Build the project
cargo build --release

# Run tests to verify setup
./test_runner.sh
```

## 🧪 Testing

Vortex maintains a comprehensive test suite with 100% pass rate. Before submitting any contribution:

### Run Tests Locally
```bash
# Full test suite (recommended)
./test_runner.sh

# Individual test categories
cargo test --test cli_integration_test --release
cargo test --test workspace_integration_tests --release
cargo test --test workspace_performance_test --release

# Code quality checks
cargo fmt --check
cargo clippy --release -- -D warnings
cargo audit
```

### Test Requirements
- **All tests must pass**: No exceptions for contributions
- **Add tests for new features**: Include comprehensive test coverage
- **Performance benchmarks**: Maintain speed targets
- **Security validation**: No unsafe code or vulnerabilities

## 📝 Code Standards

### Rust Code Style
- **Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: Pass `cargo clippy` with no warnings
- **Safety**: Avoid `unsafe` code unless absolutely necessary
- **Documentation**: Document public APIs with examples

### Commit Message Format
```
type(scope): description

Examples:
feat(workspace): add DevContainer import functionality
fix(cli): resolve help command formatting issue
docs(readme): update installation instructions
test(integration): add workspace persistence tests
```

### Code Review Process
1. **Fork the repository**
2. **Create a feature branch** from `main`
3. **Make your changes** with tests
4. **Run the test suite** locally
5. **Submit a pull request**
6. **Address review feedback**

## 🏗️ Architecture

### Project Structure
```
vortex/
├── src/                    # Main CLI application
├── vortex-core/           # Core VM management library
├── vortex-dev/            # Development tools and extensions
├── vortex-research/       # Experimental features
├── tests/                 # Integration and E2E tests
├── docs/                  # Documentation
└── .github/workflows/     # CI/CD pipelines
```

### Key Components
- **CLI Interface**: `src/main.rs` - User-facing commands
- **Core Library**: `vortex-core/` - VM management abstractions
- **Workspace System**: Persistent development environments
- **Template Engine**: Pre-configured dev environments
- **DevContainer Support**: Docker migration compatibility

## 🎯 Contribution Areas

### 🔥 High Priority
- **Windows Support**: Cross-platform VM management
- **Performance Optimization**: Even faster startup times
- **Additional Templates**: More language/framework support
- **Documentation**: Tutorials, guides, and examples

### 🚀 Features
- **Extended Networking**: Custom network configurations
- **Resource Management**: Configurable CPU/memory limits
- **Plugin System**: Third-party integrations
- **Advanced Monitoring**: Performance and resource tracking

### 🐛 Bug Fixes
- **Platform Compatibility**: macOS and Linux edge cases
- **Error Handling**: Better error messages and recovery
- **Performance Issues**: Speed or memory regressions
- **Documentation Gaps**: Missing or unclear information

## 🔒 Security Guidelines

### Security Standards
- **No unsafe code** without explicit justification
- **Dependency auditing** with `cargo audit`
- **Input validation** for all user inputs
- **Resource limits** to prevent DoS attacks

### Reporting Security Issues
- **Private disclosure**: Email security issues privately
- **No public discussion**: Until patched and released
- **Coordinated disclosure**: Work with maintainers on timeline

## 📊 Performance Requirements

### Speed Targets
- **Workspace Creation**: < 2 seconds for small projects
- **DevContainer Migration**: < 5 seconds end-to-end
- **CLI Responsiveness**: < 500ms for info queries
- **Build Times**: < 5 minutes for release builds

### Benchmarking
- **Automated tests** validate performance targets
- **Comparison metrics** against Docker DevContainers
- **Regression detection** in CI pipeline
- **Performance profiling** for optimization

## 🤝 Community Guidelines

### Code of Conduct
- **Be respectful** and inclusive
- **Provide constructive feedback**
- **Help others learn and improve**
- **Maintain professional communication**

### Getting Help
- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: Questions and community support
- **Documentation**: README and docs/ directory
- **Code Review**: Learn from maintainer feedback

## 🚀 Release Process

### Version Management
- **Semantic Versioning**: MAJOR.MINOR.PATCH format
- **Changelog**: Document all changes
- **Release Notes**: Highlight new features and fixes
- **Migration Guides**: For breaking changes

### Quality Gates
- **100% Test Pass Rate**: All tests must pass
- **Security Audit**: Zero vulnerabilities
- **Performance Validation**: Meet speed targets
- **Documentation Updates**: Keep docs current

## 📈 Roadmap Alignment

### Current Focus (v0.3.x)
- **Workspace System**: Complete persistence functionality
- **DevContainer Migration**: Seamless Docker replacement
- **Performance Optimization**: Maintain 20x speed advantage
- **Cross-platform Support**: Linux and Windows

### Future Vision (v0.4.x+)
- **Plugin Ecosystem**: Third-party extensions
- **Advanced Networking**: Complex network topologies
- **Enterprise Features**: Team management and compliance
- **Cloud Integration**: Remote workspace management

## 🎓 Learning Resources

### Rust Development
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)

### VM and Virtualization
- [krunvm Documentation](https://github.com/containers/krunvm)
- [Firecracker MicroVMs](https://firecracker-microvm.github.io/)
- [Container vs VM Concepts](https://www.docker.com/resources/what-container)

### Project Tools
- [GitHub Actions](https://docs.github.com/en/actions)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [DevContainers Spec](https://containers.dev/)

---

Thank you for contributing to Vortex! Together we're building the future of development environments - 20x faster than Docker and infinitely more secure. 🚀