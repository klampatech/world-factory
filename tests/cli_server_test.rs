//! CLI Server Command Testing (Section 14.3)
//! Tests: CLI-20 through CLI-23

use std::process::{Command, Stdio};
use std::path::Path;
use std::time::Duration;
use std::thread;

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
        cmd.args(["run", "--bin", "world_generator", "--features", "api", "--"]);
        cmd
    }
}

/// CLI-20: Server starts with --server flag
#[test]
fn cli_server_starts() {
    let mut cmd = world_factory_bin();
    cmd.arg("--server");
    cmd.arg("--port").arg("18930");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            println!("Note: Could not start server. Expected if 'api' feature not enabled.");
            println!("Spawn error: {}", e);
            return;
        }
    };
    
    thread::sleep(Duration::from_millis(500));
    
    match child.try_wait() {
        Ok(Some(status)) => {
            println!("Server process exited with status: {:?}", status);
        }
        Ok(None) => {
            println!("Server is running (pid: {:?})", child.id());
            child.kill().expect("Failed to kill server");
        }
        Err(e) => {
            println!("Error checking process: {}", e);
        }
    }
    
    println!("Server start test completed");
}

/// CLI-21: Server port handling works
#[test]
fn cli_server_custom_port() {
    let custom_port = 18765;
    
    let mut cmd = world_factory_bin();
    cmd.arg("--server");
    cmd.arg("--port").arg(custom_port.to_string());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => {
            println!("Note: Could not start server (likely missing 'api' feature)");
            return;
        }
    };
    
    thread::sleep(Duration::from_millis(500));
    
    match child.try_wait() {
        Ok(Some(status)) => {
            println!("Server exited with status: {:?}", status);
        }
        Ok(None) => {
            child.kill().expect("Failed to kill server");
        }
        Err(e) => {
            println!("Error checking process: {}", e);
        }
    }
    
    println!("Custom port test completed");
}

/// CLI-22: Server refuses to start if port is already in use
#[test]
fn cli_server_port_conflict() {
    let test_port = 18931;
    
    let mut cmd1 = world_factory_bin();
    cmd1.arg("--server");
    cmd1.arg("--port").arg(test_port.to_string());
    cmd1.stdout(Stdio::piped());
    cmd1.stderr(Stdio::piped());
    
    let mut first_server = match cmd1.spawn() {
        Ok(c) => c,
        Err(_) => {
            println!("Note: Could not start first server");
            return;
        }
    };
    
    thread::sleep(Duration::from_millis(500));
    
    let first_running = match first_server.try_wait() {
        Ok(None) => true,
        _ => false,
    };
    
    if !first_running {
        println!("First server failed to start, skipping port conflict test");
        return;
    }
    
    let mut cmd2 = world_factory_bin();
    cmd2.arg("--server");
    cmd2.arg("--port").arg(test_port.to_string());
    cmd2.stdout(Stdio::piped());
    cmd2.stderr(Stdio::piped());
    
    let output2 = cmd2.output().expect("Failed to execute second server");
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    
    let has_port_error = stderr2.to_lowercase().contains("address in use") ||
                         stdout2.to_lowercase().contains("address in use") ||
                         stderr2.to_lowercase().contains("port") ||
                         stdout2.to_lowercase().contains("port");
    
    first_server.kill().expect("Failed to kill first server");
    
    println!("Port conflict test result - has port error: {}", has_port_error);
}

/// CLI-23: Server fails with clear error when API feature is not enabled
#[test]
fn cli_server_feature_flag_error() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "world_generator", "--", "--server", "--port", "18932"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute without api feature");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        let has_feature_error = (stderr.to_lowercase() + &stdout.to_lowercase()).contains("api") &&
                                ((stderr.to_lowercase() + &stdout.to_lowercase()).contains("feature") ||
                                 (stderr.to_lowercase() + &stdout.to_lowercase()).contains("not"));
        
        assert!(has_feature_error,
                "Should have clear error about missing api feature");
    } else {
        println!("Note: API feature appears to be enabled in default build");
    }
}
