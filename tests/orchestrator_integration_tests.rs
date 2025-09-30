use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_orchestrator_script() -> PathBuf {
    PathBuf::from("./vortex_orchestrator")
}

fn run_orchestrator(args: &[&str]) -> Result<std::process::Output> {
    let output = Command::new(get_orchestrator_script())
        .args(args)
        .output()?;
    Ok(output)
}

fn run_orchestrator_expect_success(args: &[&str]) -> Result<String> {
    let output = run_orchestrator(args)?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Orchestrator command failed: ./vortex_orchestrator {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn test_orchestrator_help() -> Result<()> {
    let output = run_orchestrator_expect_success(&["help"])?;

    assert!(output.contains("VORTEX WORKSPACE ORCHESTRATOR"));
    assert!(output.contains("workspace"));
    assert!(output.contains("sync"));
    assert!(output.contains("cluster"));
    assert!(output.contains("monitor"));
    assert!(output.contains("logs"));

    Ok(())
}

#[test]
fn test_workspace_template_listing() -> Result<()> {
    let output = run_orchestrator_expect_success(&["workspace", "list"])?;

    assert!(output.contains("Workspace Templates"));
    assert!(output.contains("fullstack-webapp"));
    assert!(output.contains("microservices-api"));
    assert!(output.contains("ai-ml-pipeline"));

    // Check template descriptions
    assert!(output.contains("React + FastAPI + PostgreSQL + Redis"));
    assert!(output.contains("Go APIs + NATS + MongoDB"));
    assert!(output.contains("Jupyter + FastAPI + PostgreSQL + Redis + GPU"));

    // Check service counts
    assert!(output.contains("Services: 4 total")); // fullstack
    assert!(output.contains("Services: 5 total")); // microservices & ai-ml

    Ok(())
}

#[test]
fn test_workspace_discovery_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a multi-service project structure
    let frontend_dir = temp_dir.path().join("frontend");
    fs::create_dir_all(&frontend_dir)?;
    fs::write(
        frontend_dir.join("package.json"),
        r#"{"name": "frontend", "dependencies": {"react": "^18.0.0"}}"#,
    )?;
    fs::create_dir_all(frontend_dir.join("src"))?;

    let backend_dir = temp_dir.path().join("api");
    fs::create_dir_all(&backend_dir)?;
    fs::write(
        backend_dir.join("requirements.txt"),
        "fastapi==0.104.0\nuvicorn==0.24.0",
    )?;

    // Test init command (should trigger discovery)
    let output =
        run_orchestrator_expect_success(&["workspace", "init", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("Initialize Workspace"));
    assert!(output.contains("Running project discovery"));

    // Should have created vortex.yaml in the project directory
    let config_file = temp_dir.path().join("vortex.yaml");
    assert!(config_file.exists());

    let config_content = fs::read_to_string(config_file)?;
    assert!(config_content.contains("frontend:"));
    assert!(config_content.contains("api:"));
    assert!(config_content.contains("type: frontend"));
    assert!(config_content.contains("type: backend"));

    Ok(())
}

#[test]
fn test_config_parsing_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test with invalid config (missing name)
    let invalid_config = r#"
description: Invalid config without name

services:
  test:
    type: service
"#;

    let invalid_config_file = temp_dir.path().join("invalid.yaml");
    fs::write(&invalid_config_file, invalid_config)?;

    let output = run_orchestrator(&[
        "workspace",
        "create",
        "--config",
        invalid_config_file.to_str().unwrap(),
    ])?;

    // Should fail with error about missing name
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No workspace name found") || !output.status.success());

    Ok(())
}

#[test]
fn test_yaml_service_extraction_parsing() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a configuration to test service extraction logic
    let complex_config = r#"
name: parse-test-workspace
description: Test configuration parsing

services:
  web-frontend:
    type: frontend
    image: node:18-alpine
    ports:
      - 3000:3000
      - 3001:3001
    
  api-backend:
    type: backend
    image: python:3.11-slim
    ports:
      - 8000:8000
      
  database:
    type: database
    image: postgres:15-alpine
    ports:
      - 5432:5432

network:
  name: vortex-parse-test
  driver: bridge
"#;

    let config_file = temp_dir.path().join("parse-test.yaml");
    fs::write(&config_file, complex_config)?;

    // Test that the orchestrator can at least parse the config
    // (Even if it can't create VMs, it should parse the YAML correctly)
    let output = run_orchestrator(&[
        "workspace",
        "create",
        "--config",
        config_file.to_str().unwrap(),
    ])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should successfully parse the config and show the workspace name
    assert!(combined.contains("parse-test-workspace"));

    // Should attempt to process services (even if VM creation fails)
    assert!(combined.contains("Loading configuration") || combined.contains("Creating Workspace"));

    Ok(())
}

#[test]
fn test_workspace_command_validation() -> Result<()> {
    // Test invalid workspace commands
    let invalid_commands = [
        vec!["workspace", "create"],          // Missing template name
        vec!["workspace", "stop"],            // Missing workspace name
        vec!["workspace", "invalid-command"], // Invalid subcommand
    ];

    for args in invalid_commands {
        let output = run_orchestrator(&args)?;

        // Should either show usage help or fail gracefully
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        assert!(
            combined.contains("Usage:")
                || combined.contains("Unknown")
                || combined.contains("help")
                || !output.status.success()
        );
    }

    // Test that "workspace" alone shows template list (not an error)
    let output = run_orchestrator(&["workspace"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace Templates") && output.status.success());

    Ok(())
}

#[test]
fn test_error_handling_for_missing_files() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test with non-existent config file
    let nonexistent_config = temp_dir.path().join("nonexistent.yaml");

    let output = run_orchestrator(&[
        "workspace",
        "create",
        "--config",
        nonexistent_config.to_str().unwrap(),
    ])?;

    // Should fail gracefully
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("No such file"));

    Ok(())
}

#[test]
fn test_workspace_templates_have_consistent_structure() -> Result<()> {
    // Test that all built-in templates follow expected patterns
    let template_tests = [
        (
            "fullstack-webapp",
            vec!["frontend", "backend", "database", "cache"],
        ),
        (
            "microservices-api",
            vec![
                "api-gateway",
                "user-service",
                "order-service",
                "message-queue",
                "database",
            ],
        ),
        (
            "ai-ml-pipeline",
            vec!["jupyter", "ml-api", "data-processor", "database", "cache"],
        ),
    ];

    for (template_name, expected_services) in template_tests {
        // Verify template is listed in the template listing
        let output = run_orchestrator_expect_success(&["workspace", "list"])?;

        // Verify template is listed
        assert!(output.contains(template_name));

        // Verify expected service count
        let service_count = expected_services.len();
        let count_pattern = format!("Services: {} total", service_count);
        assert!(output.contains(&count_pattern));

        // Verify service names appear in description
        for service in expected_services {
            // Services should be mentioned in the template description
            assert!(
                output.contains(service)
                    || output.contains(&service.replace("-", " "))
                    || output.contains(&service.replace("-", "_"))
            );
        }
    }

    Ok(())
}

#[test]
fn test_workspace_status_command() -> Result<()> {
    // Test the workspace status command (should not require VMs to exist)
    let output = run_orchestrator(&["workspace", "status"])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should either show active workspaces or indicate none are running
    // Command should not crash regardless of VM state
    assert!(
        combined.contains("workspace")
            || combined.contains("No active")
            || combined.contains("Found")
            || output.status.success()
    );

    Ok(())
}
