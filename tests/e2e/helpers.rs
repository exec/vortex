//! E2E test helpers for Vortex

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the vortex binary
pub fn get_vortex_binary() -> PathBuf {
    let release_path = PathBuf::from("./target/release/vortex");
    let debug_path = PathBuf::from("./target/debug/vortex");

    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        // Build if needed
        println!("Building vortex...");
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .output()
            .expect("Failed to build vortex");

        if !output.status.success() {
            panic!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Try again after build
        if release_path.exists() {
            release_path
        } else {
            debug_path
        }
    }
}

/// Run vortex command and return output
pub fn run_vortex(args: &[&str]) -> Result<String, String> {
    let binary = get_vortex_binary();
    let output = Command::new(&binary)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run vortex: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "Command failed: vortex {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Check if krunvm is available
pub fn krunvm_available() -> bool {
    let output = Command::new("which").arg("krunvm").output().ok();
    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Check if firecracker is available
pub fn firecracker_available() -> bool {
    let output = Command::new("which").arg("firecracker").output().ok();
    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Create a temporary directory for testing
pub fn temp_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vortex-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// Clean up a test directory
pub fn cleanup_test_dir(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}
