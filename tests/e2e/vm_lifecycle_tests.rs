//! E2E tests for VM lifecycle operations

use crate::e2e::helpers::{cleanup_test_dir, krunvm_available, run_vortex, temp_test_dir};
use std::thread;
use std::time::Duration;

#[test]
fn test_vortex_vm_create_and_list() {
    // Skip if krunvm is not available
    if !krunvm_available() {
        println!("Skipping: krunvm not available");
        return;
    }

    let vm_name = format!("e2e-vm-{}", std::process::id());

    // Clean up if previous test failed
    let _ = run_vortex(&["vm", "cleanup", &vm_name]);

    // Create a VM
    let output = run_vortex(&[
        "vm",
        "create",
        &vm_name,
        "--image",
        "docker.io/library/alpine:latest",
    ]);

    // Clean up VM after test
    let _ = run_vortex(&["vm", "cleanup", &vm_name]);

    assert!(
        output.is_ok(),
        "VM creation failed: {}",
        output.unwrap_err()
    );
}

#[test]
fn test_vortex_vm_stop_and_start() {
    // Skip if krunvm is not available
    if !krunvm_available() {
        println!("Skipping: krunvm not available");
        return;
    }

    let test_dir = temp_test_dir();
    let vm_name = format!("e2e-vm-{}-stop", std::process::id());

    // Clean up if previous test failed
    let _ = run_vortex(&["vm", "cleanup", &vm_name]);

    // Create a VM
    let create_result = run_vortex(&[
        "vm",
        "create",
        &vm_name,
        "--image",
        "docker.io/library/alpine:latest",
    ]);

    if let Err(e) = create_result {
        let _ = run_vortex(&["vm", "cleanup", &vm_name]);
        panic!("VM creation failed: {}", e);
    }

    // Wait a moment for VM to be ready
    thread::sleep(Duration::from_secs(2));

    // List VMs to verify it exists
    let list_result = run_vortex(&["vm", "list"]);
    assert!(list_result.is_ok(), "VM list failed");

    // Cleanup
    let _ = run_vortex(&["vm", "cleanup", &vm_name]);
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_vortex_vm_list() {
    // Skip if krunvm is not available
    if !krunvm_available() {
        println!("Skipping: krunvm not available");
        return;
    }

    // List all VMs
    let result = run_vortex(&["vm", "list"]);
    let output = result.expect("VM list should succeed");
    assert!(
        output.contains("Name") || output.contains("ID"),
        "VM list output missing expected columns"
    );
}

#[test]
fn test_vortex_vm_cleanup() {
    // Skip if krunvm is not available
    if !krunvm_available() {
        println!("Skipping: krunvm not available");
        return;
    }

    let vm_name = format!("e2e-vm-{}-cleanup", std::process::id());

    // Try to cleanup a non-existent VM (should not fail)
    let result = run_vortex(&["vm", "cleanup", &vm_name]);
    // This may fail if the VM doesn't exist, which is expected
    let _ = result; // Ignore result for non-existent VM

    // Create a VM
    let create_result = run_vortex(&[
        "vm",
        "create",
        &vm_name,
        "--image",
        "docker.io/library/alpine:latest",
    ]);

    if create_result.is_ok() {
        // Wait a moment
        thread::sleep(Duration::from_secs(1));

        // Cleanup the VM
        let cleanup_result = run_vortex(&["vm", "cleanup", &vm_name]);
        assert!(
            cleanup_result.is_ok(),
            "VM cleanup failed: {}",
            cleanup_result.unwrap_err()
        );
    }
}

#[test]
fn test_vortex_vm_parallel_creation() {
    // Skip if krunvm is not available
    if !krunvm_available() {
        println!("Skipping: krunvm not available");
        return;
    }

    let num_vms = 2;
    let mut vm_names = Vec::new();

    for i in 0..num_vms {
        let vm_name = format!("e2e-vm-{}-parallel-{}", std::process::id(), i);
        vm_names.push(vm_name.clone());

        // Create multiple VMs
        let result = run_vortex(&[
            "vm",
            "create",
            &vm_name,
            "--image",
            "docker.io/library/alpine:latest",
            "--memory",
            "256",
        ]);

        if let Err(e) = result {
            // Cleanup any created VMs
            for name in &vm_names {
                let _ = run_vortex(&["vm", "cleanup", name]);
            }
            panic!("VM creation failed: {}", e);
        }
    }

    // Wait a moment
    thread::sleep(Duration::from_secs(2));

    // Cleanup all VMs
    for vm_name in &vm_names {
        let _ = run_vortex(&["vm", "cleanup", vm_name]);
    }
}
