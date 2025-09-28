use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;
use uuid::Uuid;

fn get_vortex_binary() -> PathBuf {
    PathBuf::from("./target/release/vortex")
}

fn run_vortex_expect_success(args: &[&str]) -> Result<String> {
    let output = Command::new(get_vortex_binary()).args(args).output()?;
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

#[test]
fn test_workspace_creation_performance() -> Result<()> {
    let test_id = Uuid::new_v4().to_string()[..8].to_string();

    // Test small workspace (10 files)
    let small_dir = TempDir::new()?;
    for i in 0..10 {
        fs::write(
            small_dir.path().join(format!("file_{}.py", i)),
            format!("# Small file {}\nprint('{}')", i, i),
        )?;
    }

    let small_workspace = format!("perf-small-{}", test_id);
    let start = Instant::now();
    run_vortex_expect_success(&[
        "workspace",
        "create",
        &small_workspace,
        "--template",
        "python",
        "--source",
        small_dir.path().to_str().unwrap(),
    ])?;
    let small_time = start.elapsed();

    println!("Small workspace (10 files): {:?}", small_time);
    assert!(
        small_time < std::time::Duration::from_secs(2),
        "Small workspace creation too slow"
    );

    // Test medium workspace (100 files)
    let medium_dir = TempDir::new()?;
    for i in 0..100 {
        fs::write(
            medium_dir.path().join(format!("file_{}.py", i)),
            format!("# Medium file {}\nprint('{}')", i, i),
        )?;
    }

    let medium_workspace = format!("perf-medium-{}", test_id);
    let start = Instant::now();
    run_vortex_expect_success(&[
        "workspace",
        "create",
        &medium_workspace,
        "--template",
        "python",
        "--source",
        medium_dir.path().to_str().unwrap(),
    ])?;
    let medium_time = start.elapsed();

    println!("Medium workspace (100 files): {:?}", medium_time);
    assert!(
        medium_time < std::time::Duration::from_secs(5),
        "Medium workspace creation too slow"
    );

    // Test workspace listing performance
    let start = Instant::now();
    run_vortex_expect_success(&["workspace", "list"])?;
    let list_time = start.elapsed();

    println!("Workspace listing: {:?}", list_time);
    assert!(
        list_time < std::time::Duration::from_secs(1),
        "Workspace listing too slow"
    );

    // Cleanup
    let _ = Command::new(get_vortex_binary())
        .args(&["workspace", "delete", &small_workspace])
        .output();
    let _ = Command::new(get_vortex_binary())
        .args(&["workspace", "delete", &medium_workspace])
        .output();

    Ok(())
}

#[test]
fn test_concurrent_workspace_operations() -> Result<()> {
    let test_id = Uuid::new_v4().to_string()[..8].to_string();

    // Create multiple workspaces concurrently (simulated)
    let mut handles = Vec::new();
    let start = Instant::now();

    for i in 0..3 {
        let workspace_name = format!("concurrent-{}-{}", test_id, i);
        let template = match i % 3 {
            0 => "python",
            1 => "node",
            _ => "rust",
        };

        // Note: We can't actually run these concurrently in tests easily,
        // so we run them sequentially but measure total time
        run_vortex_expect_success(&[
            "workspace",
            "create",
            &workspace_name,
            "--template",
            template,
        ])?;

        handles.push(workspace_name);
    }

    let total_time = start.elapsed();
    println!("3 sequential workspace creations: {:?}", total_time);
    assert!(
        total_time < std::time::Duration::from_secs(10),
        "Concurrent operations too slow"
    );

    // Verify all workspaces exist
    let list_output = run_vortex_expect_success(&["workspace", "list"])?;
    for workspace in &handles {
        assert!(list_output.contains(workspace));
    }

    // Cleanup
    for workspace in handles {
        let _ = Command::new(get_vortex_binary())
            .args(&["workspace", "delete", &workspace])
            .output();
    }

    Ok(())
}

#[test]
fn test_workspace_info_performance() -> Result<()> {
    let test_id = Uuid::new_v4().to_string()[..8].to_string();
    let workspace_name = format!("info-perf-{}", test_id);

    // Create workspace with various files
    let temp_dir = TempDir::new()?;
    fs::write(temp_dir.path().join("app.py"), "print('test')")?;
    fs::write(temp_dir.path().join("requirements.txt"), "flask\nrequests")?;

    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir)?;
    fs::write(subdir.join("main.py"), "# Main module")?;

    run_vortex_expect_success(&[
        "workspace",
        "create",
        &workspace_name,
        "--template",
        "python",
        "--source",
        temp_dir.path().to_str().unwrap(),
    ])?;

    // Test info command performance
    let start = Instant::now();
    let info_output = run_vortex_expect_success(&["workspace", "info", &workspace_name])?;
    let info_time = start.elapsed();

    println!("Workspace info query: {:?}", info_time);
    assert!(
        info_time < std::time::Duration::from_millis(500),
        "Info query too slow"
    );

    // Verify info contains expected content
    assert!(info_output.contains(&workspace_name));
    assert!(info_output.contains("Template: python"));
    assert!(info_output.contains("app.py"));
    assert!(info_output.contains("requirements.txt"));

    // Cleanup
    let _ = Command::new(get_vortex_binary())
        .args(&["workspace", "delete", &workspace_name])
        .output();

    Ok(())
}
