# Vortex Testing Guide

This document describes the comprehensive test suite for Vortex, the lightning-fast ephemeral VM platform.

## Test Structure

```
tests/
├── cli_integration_test.rs     # CLI integration tests
├── discovery_engine_tests.rs   # Discovery engine tests
├── orchestrator_integration_tests.rs  # Orchestrator tests
├── e2e/                       # End-to-end scenarios
│   ├── test_workspace_demo.sh
│   └── devcontainer_migration_test.sh
└── docs/
    └── TESTING.md             # This file
```

## Running Tests

### Quick Test Run
```bash
# Run all tests
./test_runner.sh

# Run with options
SKIP_E2E=true ./test_runner.sh           # Skip E2E tests
SKIP_PERFORMANCE=true ./test_runner.sh   # Skip performance tests
VERBOSE=true ./test_runner.sh            # Verbose output
```

### Individual Test Categories

#### Unit Tests
```bash
cargo test --lib --release
cargo test --bin vortex --release
```

#### Integration Tests
```bash
cargo test --test cli_integration_test --release
cargo test --test cli_integration_test --release
cargo test --test discovery_engine_tests --release
cargo test --test discovery_engine_tests --release
cargo test --test orchestrator_integration_tests --release
```

#### Performance Tests
```bash
cargo test --test cli_integration_test --release -- --nocapture
```

#### End-to-End Tests
```bash
./tests/e2e/test_workspace_demo.sh
./tests/e2e/devcontainer_migration_test.sh
```

## Test Categories

### 🔬 Unit Tests
- **Core library functionality**
- **Binary command parsing**
- **Individual module validation**

**Coverage:**
- Template system
- Workspace management
- Error handling
- Configuration parsing

### 🔗 Integration Tests
- **CLI command integration**
- **Workspace lifecycle management**
- **DevContainer import/export**
- **Multi-template scenarios**

**Key Tests:**
- `test_workspace_creation_and_listing()`: Basic CRUD operations
- `test_workspace_persistence()`: File persistence across sessions
- `test_devcontainer_import()`: Docker migration compatibility
- `test_workspace_init_detection()`: Smart project detection

### ⚡ Performance Tests
- **Workspace creation speed**
- **File operation performance**
- **Concurrent operation handling**
- **Large project scenarios**

**Benchmarks:**
- Small workspace (10 files): < 2 seconds
- Medium workspace (100 files): < 5 seconds
- Large workspace (1000+ files): < 10 seconds
- Info queries: < 500ms

### 🎯 End-to-End Tests
- **Complete workflow validation**
- **Real-world scenario testing**
- **DevContainer migration demos**
- **Performance comparisons**

**Scenarios:**
- Complete development workflow
- Docker DevContainer migration
- Multi-project workspace management
- CI/CD integration testing

### 🔒 Security & Quality Tests
- **Cargo audit for vulnerabilities**
- **Clippy linting**
- **Code formatting**
- **Unsafe code detection**

## Test Data & Fixtures

### Workspace Test Fixtures
Tests create realistic project structures:

**Python Project:**
```
app.py
requirements.txt
README.md
src/
  __init__.py
  main.py
tests/
  test_app.py
```

**Node.js Project:**
```
package.json
index.js
src/
  app.js
  utils.js
.eslintrc.js
```

**DevContainer Configuration:**
```json
{
  "name": "Test Container",
  "image": "node:18-slim",
  "forwardPorts": [3000, 8080],
  "postCreateCommand": "npm install"
}
```

## Performance Benchmarks

### Vortex vs Docker Comparison

| Operation | Vortex | Docker DevContainer |
|-----------|--------|-------------------|
| Environment Creation | ~50ms | ~30-60 seconds |
| Startup Time | ~2-3 seconds | ~60-100 seconds |
| File Operations | Instant | Variable |
| Memory Usage | Isolated VM | Shared kernel |
| Security | Hardware isolation | Container escape risks |

### Performance Targets

**Workspace Operations:**
- Creation: < 5 seconds (any size)
- Listing: < 1 second
- Info queries: < 500ms
- File persistence: Instant

**Scalability:**
- Support 100+ workspaces
- Handle projects with 10,000+ files
- Concurrent operations without degradation

## Continuous Integration

### GitHub Actions Workflows

**Main Test Workflow** (`.github/workflows/test.yml`):
- Runs on every push/PR
- Tests on Ubuntu and macOS
- Includes security audits
- Performance benchmarking
- Integration matrix testing

**Test Jobs:**
1. **test**: Complete test suite on Ubuntu
2. **test-macos**: Platform-specific testing
3. **benchmark**: Performance validation
4. **security**: Security audit and unsafe code detection
5. **integration**: Matrix testing of scenarios

### CI Environment Variables
```bash
SKIP_E2E=false          # Include end-to-end tests
SKIP_PERFORMANCE=false  # Include performance tests
CLEANUP_AFTER=true      # Clean up test artifacts
VERBOSE=true            # Detailed output
```

## Test Development Guidelines

### Writing New Tests

1. **Use descriptive test names**:
   ```rust
   #[test]
   fn test_workspace_survives_vm_destruction() -> Result<()> {
   ```

2. **Include cleanup**:
   ```rust
   // Cleanup
   let _ = run_vortex(&["workspace", "delete", &workspace_name]);
   ```

3. **Test error conditions**:
   ```rust
   // Should fail with invalid template
   let result = run_vortex(&["workspace", "create", "test", "--template", "invalid"]);
   assert!(result.is_err());
   ```

4. **Use realistic data**:
   ```rust
   fs::write(temp_dir.path().join("package.json"), r#"{"name": "real-app"}"#)?;
   ```

### Test Utilities

**Common Functions:**
```rust
fn get_vortex_binary() -> PathBuf;
fn run_vortex_expect_success(args: &[&str]) -> Result<String>;
fn create_test_project(dir: &Path, project_type: &str) -> Result<()>;
fn cleanup_test_workspaces(pattern: &str) -> Result<()>;
```

## Debugging Tests

### Local Debugging
```bash
# Run with verbose output
VERBOSE=true ./test_runner.sh

# Run single test with output
cargo test test_workspace_creation_and_listing --release -- --nocapture

# Keep test artifacts for inspection
CLEANUP_AFTER=false ./test_runner.sh
```

### CI Debugging
- Check GitHub Actions logs
- Download test artifacts
- Review performance summaries
- Check security audit results

## Test Metrics & Reporting

### Success Criteria
- **100% unit test pass rate**
- **95%+ integration test pass rate**
- **Performance within benchmarks**
- **Zero security vulnerabilities**
- **Clean code quality metrics**

### Performance Tracking
Tests automatically track and report:
- Workspace creation times
- File operation speeds
- Memory usage patterns
- Concurrent operation performance

### Coverage Goals
- **Core functionality**: 100% coverage
- **CLI commands**: 95% coverage
- **Error paths**: 90% coverage
- **Performance scenarios**: Key workflows covered

## Troubleshooting

### Common Issues

**Permission Errors:**
```bash
chmod +x test_runner.sh
chmod +x tests/e2e/*.sh
```

**Missing Dependencies:**
```bash
cargo install cargo-audit
```

**Workspace Conflicts:**
```bash
# Clean up conflicting workspaces
./target/release/vortex workspace list
./target/release/vortex workspace delete <name>
```

**Performance Test Failures:**
- Check system load
- Verify disk space
- Review resource limits

### Getting Help

1. **Check test logs** for detailed error information
2. **Run individual tests** to isolate issues
3. **Review CI artifacts** for additional context
4. **Check GitHub Issues** for known problems

## Contributing

### Adding New Tests

1. **Identify test category** (unit/integration/e2e/performance)
2. **Create test file** in appropriate directory
3. **Update test_runner.sh** if needed
4. **Add to CI workflow** if appropriate
5. **Document test purpose** and expectations

### Test Review Checklist

- [ ] Test name is descriptive
- [ ] Proper cleanup implemented
- [ ] Error cases covered
- [ ] Performance expectations set
- [ ] Documentation updated
- [ ] CI integration verified

## Future Enhancements

### Planned Test Improvements

1. **Stress Testing**: Extended load scenarios
2. **Cross-Platform**: Windows support validation
3. **Network Testing**: Multi-host scenarios
4. **Resource Limits**: Memory/CPU constraint testing
5. **Backup/Restore**: Workspace migration testing

### Automation Enhancements

1. **Automated benchmarking** with historical tracking
2. **Visual test reporting** with charts and graphs
3. **Performance regression detection**
4. **Automated security scanning**
5. **Integration with external tools**

---

The Vortex test suite ensures our platform delivers on its promises of speed, security, and reliability while maintaining superiority over Docker in every measurable way. 🚀