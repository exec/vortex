use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn get_vortex_binary() -> PathBuf {
    // Try release first, fall back to debug
    let release_path = PathBuf::from("./target/release/vortex");
    let debug_path = PathBuf::from("./target/debug/vortex");

    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        // Build if needed
        let output = Command::new("cargo")
            .arg("build")
            .output()
            .expect("Failed to build vortex");

        if !output.status.success() {
            panic!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Try again after build
        if debug_path.exists() {
            debug_path
        } else {
            release_path
        }
    }
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
fn test_vortex_help_commands() -> Result<()> {
    // Test main help
    let help_output = run_vortex_expect_success(&["--help"])?;
    assert!(help_output.contains("Lightning-fast ephemeral VM platform"));
    assert!(help_output.contains("dev"));
    assert!(help_output.contains("workspace"));
    assert!(help_output.contains("parallel"));

    // Test subcommand help
    let dev_help = run_vortex_expect_success(&["dev", "--help"])?;
    assert!(dev_help.contains("instant dev environments"));
    assert!(dev_help.contains("--workspace"));
    assert!(dev_help.contains("--init"));

    let workspace_help = run_vortex_expect_success(&["workspace", "--help"])?;
    assert!(workspace_help.contains("persistent workspaces"));
    assert!(workspace_help.contains("create"));
    assert!(workspace_help.contains("import"));

    Ok(())
}

#[test]
fn test_version_and_basic_info() -> Result<()> {
    let version_output = run_vortex_expect_success(&["--version"])?;
    assert!(version_output.contains("vortex"));
    // Check for the version from Cargo.toml (supports v0.5.3, 1.0.0-rc.1, etc.)
    let version = env!("CARGO_PKG_VERSION");
    assert!(version_output.contains(version));

    Ok(())
}
