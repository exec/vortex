use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;

fn get_vortex_binary() -> PathBuf {
    // Try multiple possible locations for the binary
    let possible_paths = [
        "./target/release/vortex",
        "../target/release/vortex",
        "../../target/release/vortex",
        "target/release/vortex",
    ];

    for path in &possible_paths {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return path_buf;
        }
    }

    // Fallback to the standard path
    PathBuf::from("./target/release/vortex")
}

fn run_vortex(args: &[&str]) -> Result<std::process::Output> {
    let output = Command::new(get_vortex_binary()).args(args).output()?;
    Ok(output)
}

fn run_vortex_expect_success(args: &[&str]) -> Result<String> {
    let output = run_vortex(args)?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Command failed: vortex {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn extract_workspace_path(info_output: &str) -> Result<String> {
    for line in info_output.lines() {
        if line.contains("📂 Path:") {
            return Ok(line.split("Path: ").nth(1).unwrap().trim().to_string());
        }
    }
    Err(anyhow::anyhow!("Could not find workspace path in output"))
}

#[test]
fn test_workspace_creation_and_listing() -> Result<()> {
    let workspace_name = format!("test-{}", Uuid::new_v4().to_string()[..8].to_string());

    // Create workspace
    let output = run_vortex_expect_success(&[
        "workspace",
        "create",
        &workspace_name,
        "--template",
        "python",
    ])?;

    assert!(output.contains("✅ Workspace"));
    assert!(output.contains(&workspace_name));
    assert!(output.contains("Template: python"));

    // List workspaces
    let list_output = run_vortex_expect_success(&["workspace", "list"])?;
    assert!(list_output.contains(&workspace_name));
    assert!(list_output.contains("Template: python"));

    // Get workspace info
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    assert!(info_output.contains(&workspace_name));
    assert!(info_output.contains("Template: python"));
    assert!(info_output.contains("Working directory: /workspace"));

    // Cleanup - skip user confirmation for tests
    let _ = run_vortex(&["workspace", "delete", &workspace_name]);

    Ok(())
}

#[test]
fn test_workspace_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.py");
    let workspace_name = format!("persist-{}", Uuid::new_v4().to_string()[..8].to_string());

    // Create a test Python file
    fs::write(
        &test_file,
        r#"
print("Hello from persistent workspace!")
import os
print(f"Current directory: {os.getcwd()}")
with open("workspace_data.txt", "w") as f:
    f.write("This data should persist between VM sessions")
"#,
    )?;

    // Create workspace from the temp directory
    let output = run_vortex_expect_success(&[
        "workspace",
        "create",
        &workspace_name,
        "--template",
        "python",
        "--source",
        temp_dir.path().to_str().unwrap(),
    ])?;

    assert!(output.contains("✅ Workspace"));
    assert!(output.contains(&workspace_name));

    // Check that files were copied to workspace
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    assert!(info_output.contains("test.py"));

    // Get workspace path from info output
    let workspace_path = extract_workspace_path(&info_output)?;

    // Verify the test file exists in workspace
    let workspace_test_file = PathBuf::from(&workspace_path).join("test.py");
    assert!(workspace_test_file.exists());

    // Cleanup
    let _ = run_vortex(&["workspace", "delete", &workspace_name]);

    Ok(())
}

#[test]
fn test_devcontainer_import() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let devcontainer_dir = temp_dir.path().join(".devcontainer");
    fs::create_dir_all(&devcontainer_dir)?;

    // Create a devcontainer.json
    let devcontainer_json = devcontainer_dir.join("devcontainer.json");
    fs::write(
        &devcontainer_json,
        r#"{
        "name": "Test Dev Container",
        "image": "python:3.11-slim",
        "postCreateCommand": "pip install requests",
        "forwardPorts": [8000, 8080],
        "workspaceFolder": "/workspace",
        "customizations": {
            "vscode": {
                "extensions": ["ms-python.python"]
            }
        }
    }"#,
    )?;

    // Create a test Python project
    fs::write(
        temp_dir.path().join("main.py"),
        "print('Hello DevContainer!')",
    )?;
    fs::write(temp_dir.path().join("requirements.txt"), "requests\nflask")?;

    let workspace_name = format!(
        "devcontainer-{}",
        Uuid::new_v4().to_string()[..8].to_string()
    );

    // Import devcontainer
    let output = run_vortex_expect_success(&[
        "workspace",
        "import",
        &workspace_name,
        "--devcontainer",
        devcontainer_json.to_str().unwrap(),
        "--source",
        temp_dir.path().to_str().unwrap(),
    ])?;

    assert!(output.contains("✅ Workspace"));
    assert!(output.contains("imported from devcontainer"));
    assert!(output.contains("Port forwards: 8000, 8080"));

    // Check workspace info
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    assert!(info_output.contains("DevContainer source"));
    assert!(info_output.contains("main.py"));
    assert!(info_output.contains("requirements.txt"));
    assert!(info_output.contains("Port forwards: 8000, 8080"));

    // Cleanup
    let _ = run_vortex(&["workspace", "delete", &workspace_name]);

    Ok(())
}

#[test]
fn test_workspace_init_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a Python project structure
    fs::write(temp_dir.path().join("requirements.txt"), "flask\nrequests")?;
    fs::write(
        temp_dir.path().join("app.py"),
        "from flask import Flask\napp = Flask(__name__)",
    )?;
    fs::write(temp_dir.path().join("README.md"), "# Test Python Project")?;

    // Change to temp directory and run init
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_dir)?;

    let output = run_vortex_expect_success(&["dev", "--init"])?;

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    assert!(output.contains("Detected project"));
    assert!(output.contains("Suggested template: python"));
    assert!(output.contains("✅ Workspace"));

    // The workspace should be named after the temp directory
    let dir_name = temp_dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Check it was created
    let list_output = run_vortex_expect_success(&["workspace", "list"])?;
    assert!(list_output.contains(&dir_name));

    // Cleanup
    let _ = run_vortex(&["workspace", "delete", &dir_name]);

    Ok(())
}

#[test]
fn test_workspace_template_detection() -> Result<()> {
    let test_cases = vec![
        ("Cargo.toml", "[package]\nname = \"test\"", "rust"),
        ("package.json", r#"{"name": "test"}"#, "node"),
        ("go.mod", "module test", "go"),
        ("pyproject.toml", "[tool.poetry]", "python"),
    ];

    for (filename, content, expected_template) in test_cases {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join(filename), content)?;

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&temp_dir)?;

        let output = run_vortex_expect_success(&["dev", "--init"])?;

        std::env::set_current_dir(&original_dir)?;

        assert!(output.contains(&format!("Suggested template: {}", expected_template)));

        // Cleanup
        let dir_name = temp_dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let _ = run_vortex(&["workspace", "delete", &dir_name]);
    }

    Ok(())
}

#[test]
fn test_workspace_file_persistence_across_sessions() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_name = format!("session-{}", Uuid::new_v4().to_string()[..8].to_string());

    // Create initial files
    fs::write(temp_dir.path().join("initial.py"), "print('Initial file')")?;

    // Create workspace
    run_vortex_expect_success(&[
        "workspace",
        "create",
        &workspace_name,
        "--template",
        "python",
        "--source",
        temp_dir.path().to_str().unwrap(),
    ])?;

    // Get workspace path
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    let workspace_path = extract_workspace_path(&info_output)?;

    // Simulate adding files during a "session"
    fs::write(
        PathBuf::from(&workspace_path).join("session1.py"),
        "print('Added in session 1')",
    )?;

    // Create a subdirectory with files
    let subdir = PathBuf::from(&workspace_path).join("data");
    fs::create_dir_all(&subdir)?;
    fs::write(subdir.join("results.txt"), "Important data from session 1")?;

    // Simulate second session - check files persist
    let info_output2 = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    assert!(info_output2.contains("initial.py"));
    assert!(info_output2.contains("session1.py"));
    assert!(info_output2.contains("data/"));

    // Verify file contents persist
    let session1_file = PathBuf::from(&workspace_path).join("session1.py");
    let content = fs::read_to_string(session1_file)?;
    assert_eq!(content, "print('Added in session 1')");

    let results_file = PathBuf::from(&workspace_path)
        .join("data")
        .join("results.txt");
    let results_content = fs::read_to_string(results_file)?;
    assert_eq!(results_content, "Important data from session 1");

    // Cleanup
    let _ = run_vortex(&["workspace", "delete", &workspace_name]);

    Ok(())
}

#[test]
fn test_multiple_workspaces_management() -> Result<()> {
    let mut workspace_names = Vec::new();

    // Create multiple workspaces with different templates
    let templates = vec!["python", "node", "rust"];

    for (i, template) in templates.iter().enumerate() {
        let name = format!("multi-{}-{}", template, i);

        run_vortex_expect_success(&["workspace", "create", &name, "--template", template])?;

        workspace_names.push(name);
    }

    // List all workspaces
    let list_output = run_vortex_expect_success(&["workspace", "list"])?;

    // Verify all workspaces exist
    for name in &workspace_names {
        assert!(list_output.contains(name));
    }

    // Check individual workspace info
    for (name, template) in workspace_names.iter().zip(templates.iter()) {
        let info_output = run_vortex_expect_success(&["workspace", "info", name])?;
        assert!(info_output.contains(&format!("Template: {}", template)));
    }

    // Cleanup all workspaces
    for name in workspace_names {
        let _ = run_vortex(&["workspace", "delete", &name]);
    }

    Ok(())
}

#[test]
fn test_large_workspace_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_name = format!("large-{}", Uuid::new_v4().to_string()[..8].to_string());

    // Create a workspace with many files
    for i in 0..50 {
        // Reduced from 100 for faster tests
        fs::write(
            temp_dir.path().join(format!("file_{}.py", i)),
            format!("# File number {}\nprint('File {}')", i, i),
        )?;
    }

    // Create nested directory structure
    for i in 0..5 {
        // Reduced from 10
        let subdir = temp_dir.path().join(format!("subdir_{}", i));
        fs::create_dir_all(&subdir)?;

        for j in 0..5 {
            // Reduced from 10
            fs::write(
                subdir.join(format!("nested_{}.py", j)),
                format!("# Nested file {}-{}", i, j),
            )?;
        }
    }

    let start_time = std::time::Instant::now();

    // Create workspace (should copy all files)
    run_vortex_expect_success(&[
        "workspace",
        "create",
        &workspace_name,
        "--template",
        "python",
        "--source",
        temp_dir.path().to_str().unwrap(),
    ])?;

    let creation_time = start_time.elapsed();
    println!(
        "Workspace creation with 75+ files took: {:?}",
        creation_time
    );

    // Verify workspace info loads quickly
    let info_start = std::time::Instant::now();
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    let info_time = info_start.elapsed();

    println!("Workspace info query took: {:?}", info_time);

    // Should show some files (limited to 10 in display)
    assert!(info_output.contains("file_0.py"));
    assert!(info_output.contains("... and"));

    // Cleanup
    let _ = run_vortex(&["workspace", "delete", &workspace_name]);

    // Performance assertions
    assert!(
        creation_time < std::time::Duration::from_secs(10),
        "Workspace creation too slow"
    );
    assert!(
        info_time < std::time::Duration::from_secs(1),
        "Workspace info query too slow"
    );

    Ok(())
}

#[test]
fn test_complete_workspace_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Step 1: Create a realistic project structure
    fs::write(
        temp_dir.path().join("main.py"),
        r#"
from flask import Flask
app = Flask(__name__)

@app.route('/')
def hello():
    return "Hello from Vortex workspace!"

if __name__ == '__main__':
    app.run(debug=True)
"#,
    )?;

    fs::write(
        temp_dir.path().join("requirements.txt"),
        "flask==2.3.3\nrequests==2.31.0",
    )?;

    fs::write(
        temp_dir.path().join("README.md"),
        r#"
# Flask Test App

This is a test Flask application for Vortex workspace testing.

## Development

```bash
pip install -r requirements.txt
python main.py
```
"#,
    )?;

    // Create .devcontainer for testing import
    let devcontainer_dir = temp_dir.path().join(".devcontainer");
    fs::create_dir_all(&devcontainer_dir)?;
    fs::write(
        devcontainer_dir.join("devcontainer.json"),
        r#"{
        "name": "Flask Dev Environment",
        "image": "python:3.11-slim",
        "postCreateCommand": "pip install -r requirements.txt",
        "forwardPorts": [5000],
        "workspaceFolder": "/workspace"
    }"#,
    )?;

    let workspace_name = format!("workflow-{}", Uuid::new_v4().to_string()[..8].to_string());

    // Step 2: Import devcontainer workspace
    let import_output = run_vortex_expect_success(&[
        "workspace",
        "import",
        &workspace_name,
        "--devcontainer",
        devcontainer_dir.join("devcontainer.json").to_str().unwrap(),
        "--source",
        temp_dir.path().to_str().unwrap(),
    ])?;

    assert!(import_output.contains("✅ Workspace"));
    assert!(import_output.contains("imported from devcontainer"));

    // Step 3: Verify workspace structure
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    assert!(info_output.contains("main.py"));
    assert!(info_output.contains("requirements.txt"));
    assert!(info_output.contains("README.md"));
    assert!(info_output.contains("DevContainer source"));

    // Step 4: Simulate development work
    let workspace_path = extract_workspace_path(&info_output)?;
    fs::write(
        PathBuf::from(&workspace_path).join("config.py"),
        "DEBUG = True\nSECRET_KEY = 'dev-key'",
    )?;

    // Create a data directory with some files
    let data_dir = PathBuf::from(&workspace_path).join("data");
    fs::create_dir_all(&data_dir)?;
    fs::write(data_dir.join("users.json"), r#"{"users": []}"#)?;

    // Step 5: Verify persistence
    let updated_info = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    assert!(updated_info.contains("config.py"));
    assert!(updated_info.contains("data/"));

    // Step 6: Test workspace listing and management
    let list_output = run_vortex_expect_success(&["workspace", "list"])?;
    assert!(list_output.contains(&workspace_name));
    assert!(list_output.contains("Template: python"));

    // Step 7: Cleanup
    let _ = run_vortex(&["workspace", "delete", &workspace_name]);

    Ok(())
}
