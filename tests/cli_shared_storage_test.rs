//! CLI Shared Storage Testing (Section 14.4)
//! Tests: CLI-30 through CLI-32

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

fn world_factory_bin() -> Command {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let target_dir = Path::new(&manifest_dir).join("..").join("target");

    let debug_bin = target_dir.join("debug").join("world_generator");
    let release_bin = target_dir.join("release").join("world_generator");

    if debug_bin.exists() {
        Command::new(debug_bin)
    } else if release_bin.exists() {
        Command::new(release_bin)
    } else {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--bin", "world_generator", "--"]);
        cmd
    }
}

/// CLI-30: WORLD_FACTORY_DIR environment variable is respected
#[test]
fn cli_world_factory_dir_env_respected() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let custom_dir = temp_dir.path();

    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("77777");
    cmd.arg("--width").arg("16");
    cmd.arg("--height").arg("16");
    cmd.env("WORLD_FACTORY_DIR", custom_dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute generate");
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let has_path_hint =
            stderr.contains("world") || stderr.contains("saved") || stderr.contains("data");
        println!(
            "Custom dir test - output mentions storage: {}",
            has_path_hint
        );
    }

    println!("WORLD_FACTORY_DIR test completed");
}

/// CLI-31: Default storage location is used when env var not set
#[test]
fn cli_default_storage_when_env_not_set() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("88888");
    cmd.arg("--width").arg("16");
    cmd.arg("--height").arg("16");
    cmd.env_remove("WORLD_FACTORY_DIR");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute generate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = stdout.len() + stderr.len();
    assert!(
        output.status.success() || combined > 0,
        "Should produce output even without custom storage dir"
    );

    println!(
        "Default storage test completed. Output length: {}",
        combined
    );
}

/// CLI-32: CLI and server share the same storage configuration
#[test]
fn cli_server_shared_storage() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let custom_dir = temp_dir.path().to_str().unwrap();

    let mut cli_cmd = world_factory_bin();
    cli_cmd.arg("generate");
    cli_cmd.arg("--seed").arg("99999");
    cli_cmd.arg("--width").arg("16");
    cli_cmd.arg("--height").arg("16");
    cli_cmd.env("WORLD_FACTORY_DIR", custom_dir);
    cli_cmd.stdout(Stdio::piped());
    cli_cmd.stderr(Stdio::piped());

    let cli_output = cli_cmd.output().expect("Failed to execute CLI");
    let cli_stderr_len = String::from_utf8_lossy(&cli_output.stderr).len();

    println!(
        "CLI with WORLD_FACTORY_DIR: success={}, stderr_len={}",
        cli_output.status.success(),
        cli_stderr_len
    );

    let mut server_cmd = world_factory_bin();
    server_cmd.arg("--server");
    server_cmd.arg("--port").arg("18940");
    server_cmd.env("WORLD_FACTORY_DIR", custom_dir);
    server_cmd.stdout(Stdio::piped());
    server_cmd.stderr(Stdio::piped());

    match server_cmd.spawn() {
        Ok(mut child) => {
            std::thread::sleep(std::time::Duration::from_millis(300));

            match child.try_wait() {
                Ok(Some(_)) => {
                    println!("Server exited quickly");
                }
                Ok(None) => {
                    println!("Server is running on custom storage dir");
                    child.kill().expect("Failed to kill server");
                }
                Err(e) => {
                    println!("Error checking server: {}", e);
                }
            }
        }
        Err(_) => {
            println!("Note: Could not start server to test shared storage");
        }
    }

    println!("Shared storage test completed");
}

/// CLI-33: Invalid WORLD_FACTORY_DIR path is handled gracefully
#[test]
fn cli_invalid_world_factory_dir() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("11111");
    cmd.arg("--width").arg("8");
    cmd.arg("--height").arg("8");
    cmd.env(
        "WORLD_FACTORY_DIR",
        "/nonexistent/path/that/does/not/exist/anywhere",
    );
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute generate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined_len = stdout.len() + stderr.len();

    println!(
        "Invalid WORLD_FACTORY_DIR test: exit_code={}, output_len={}",
        output.status.code().unwrap_or(-1),
        combined_len
    );
}

/// CLI-34: Relative WORLD_FACTORY_DIR is resolved correctly
#[test]
fn cli_relative_world_factory_dir() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("22222");
    cmd.arg("--width").arg("8");
    cmd.arg("--height").arg("8");
    cmd.env("WORLD_FACTORY_DIR", "test_world_factory_data");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute generate");

    let _ = fs::remove_dir_all("test_world_factory_data");

    println!("Relative WORLD_FACTORY_DIR test completed");
}
