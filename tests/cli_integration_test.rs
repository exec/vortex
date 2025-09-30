use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

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
    assert!(version_output.contains("0.4.1"));

    Ok(())
}
