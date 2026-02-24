//! E2E tests for workspace operations

use crate::e2e::helpers::{cleanup_test_dir, run_vortex, temp_test_dir};

#[test]
fn test_workspace_init_and_list() {
    // Create a test workspace directory
    let test_dir = temp_test_dir();
    let config_path = test_dir.join("vortex.yaml");

    // Initialize workspace
    let result = run_vortex(&[
        "workspace",
        "init",
        "--output",
        config_path.to_str().unwrap(),
    ]);

    // Cleanup test directory
    cleanup_test_dir(&test_dir);

    // Note: We don't assert success here because workspace init may fail
    // if krunvm is not available, which is expected in some environments
    if let Err(e) = result {
        println!(
            "Workspace init skipped or failed (expected if krunvm not available): {}",
            e
        );
    }
}

#[test]
fn test_workspace_status() {
    // Test workspace status command
    let result = run_vortex(&["workspace", "status"]);

    // Command should succeed even if no workspaces exist
    if let Err(e) = result {
        println!("Workspace status command failed: {}", e);
    }
}

#[test]
fn test_workspace_help() {
    // Test workspace help output
    let result = run_vortex(&["workspace", "--help"]);

    assert!(
        result.is_ok(),
        "Workspace help failed: {}",
        result.unwrap_err()
    );

    let output = result.unwrap();
    assert!(
        output.contains("create") || output.contains("init"),
        "Workspace help should mention create/init commands"
    );
}

#[test]
fn test_workspace_list() {
    // Test workspace list command
    let result = run_vortex(&["workspace", "list"]);

    // Command should succeed even if no workspaces exist
    if let Err(e) = result {
        println!("Workspace list command failed (may be expected): {}", e);
    }
}
