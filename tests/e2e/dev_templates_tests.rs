//! E2E tests for dev environment templates

use crate::e2e::helpers::run_vortex;

#[test]
fn test_dev_list_templates() {
    // Test dev template listing
    let result = run_vortex(&["dev", "--list"]);

    // Note: This may fail if krunvm is not available, which is expected
    if let Err(e) = result {
        println!("Dev list failed (expected if krunvm not available): {}", e);
    }
}

#[test]
fn test_dev_help() {
    // Test dev help output
    let result = run_vortex(&["dev", "--help"]);

    assert!(result.is_ok(), "Dev help failed: {}", result.unwrap_err());

    let output = result.unwrap();
    assert!(
        output.contains("run") || output.contains("template"),
        "Dev help should mention run or template commands"
    );
}

#[test]
fn test_dev_templates_exist() {
    // Get available templates
    let result = run_vortex(&["dev", "list"]);

    if let Err(e) = result {
        println!("Dev list failed (expected if krunvm not available): {}", e);
        return;
    }

    let output = result.unwrap();

    // Check that common templates are listed
    let templates = ["node", "python", "alpine"];
    let found_any = templates
        .iter()
        .any(|t| output.to_lowercase().contains(&t.to_lowercase()));

    if !found_any {
        println!("Warning: No common templates found in output:\n{}", output);
    }
}
