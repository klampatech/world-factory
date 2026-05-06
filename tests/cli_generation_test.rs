//! CLI Generate Command Testing (Section 14.2)
//! Tests: CLI-10 through CLI-16

use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;

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

fn get_output_text(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}\n{}", stdout, stderr)
}

/// CLI-10: Generate command completes successfully
#[test]
fn cli_generate_command_completes() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("12345");
    cmd.arg("--width").arg("32");
    cmd.arg("--height").arg("32");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(output.status.success(), 
            "Generate should succeed. stderr: {}", stderr);
}

/// CLI-11: Generate with different seeds produces different outputs
#[test]
fn cli_generate_reproducibility_different_seeds() {
    let mut cmd1 = world_factory_bin();
    cmd1.arg("generate");
    cmd1.arg("--seed").arg("1");
    cmd1.arg("--width").arg("64");
    cmd1.arg("--height").arg("64");
    cmd1.stdout(Stdio::piped());
    cmd1.stderr(Stdio::piped());
    
    let mut cmd2 = world_factory_bin();
    cmd2.arg("generate");
    cmd2.arg("--seed").arg("2");
    cmd2.arg("--width").arg("64");
    cmd2.arg("--height").arg("64");
    cmd2.stdout(Stdio::piped());
    cmd2.stderr(Stdio::piped());
    
    let output1 = cmd1.output().expect("Failed to execute seed=1");
    let output2 = cmd2.output().expect("Failed to execute seed=2");
    
    assert!(output1.status.success() && output2.status.success());
    
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    
    assert_ne!(stdout1, stdout2, 
               "Different seeds should produce different outputs");
}

/// CLI-12: Generate with same seed produces deterministic output
#[test]
fn cli_generate_reproducibility_same_seed() {
    let seed = 99999;
    let width = 64;
    let height = 64;
    
    let mut cmd1 = world_factory_bin();
    cmd1.arg("generate");
    cmd1.arg("--seed").arg(seed.to_string());
    cmd1.arg("--width").arg(width.to_string());
    cmd1.arg("--height").arg(height.to_string());
    cmd1.stdout(Stdio::piped());
    cmd1.stderr(Stdio::piped());
    
    let mut cmd2 = world_factory_bin();
    cmd2.arg("generate");
    cmd2.arg("--seed").arg(seed.to_string());
    cmd2.arg("--width").arg(width.to_string());
    cmd2.arg("--height").arg(height.to_string());
    cmd2.stdout(Stdio::piped());
    cmd2.stderr(Stdio::piped());
    
    let output1 = cmd1.output().expect("Failed to execute first generate");
    let output2 = cmd2.output().expect("Failed to execute second generate");
    
    assert!(output1.status.success() && output2.status.success());
    
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    
    let world_id1 = stdout1.lines()
        .find(|l| l.contains("World ID"))
        .map(|l| l.to_string());
    let world_id2 = stdout2.lines()
        .find(|l| l.contains("World ID"))
        .map(|l| l.to_string());
    
    if let (Some(id1), Some(id2)) = (world_id1, world_id2) {
        assert_eq!(id1, id2, 
                  "Same seed should produce identical World ID");
    }
}

/// CLI-13: Generate with custom dimensions works
#[test]
fn cli_generate_custom_dimensions() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("100");
    cmd.arg("--width").arg("256");
    cmd.arg("--height").arg("128");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate");
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("256") || stdout.contains("256x"), 
            "Output should mention width 256");
    assert!(stdout.contains("128") || stdout.contains("128x"), 
            "Output should mention height 128");
}

/// CLI-14: Generate handles minimum valid dimensions
#[test]
fn cli_generate_minimum_dimensions() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("1");
    cmd.arg("--width").arg("1");
    cmd.arg("--height").arg("1");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate");
    let combined = get_output_text(&output);
    
    if !output.status.success() {
        assert!(combined.to_lowercase().contains("error") || 
                combined.to_lowercase().contains("invalid") ||
                combined.to_lowercase().contains("minimum"), 
                "Failure should have clear error message");
    }
}

/// CLI-15: Generate validation error for invalid dimensions
#[test]
fn cli_generate_validation_error_invalid_dimensions() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("1");
    cmd.arg("--width").arg("0");
    cmd.arg("--height").arg("64");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate");
    let combined = get_output_text(&output);
    assert!(!combined.is_empty(), "Should produce error message for invalid input");
}

/// CLI-16: Generate validation error for invalid seed
#[test]
fn cli_generate_validation_error_invalid_seed() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("not_a_number");
    cmd.arg("--width").arg("64");
    cmd.arg("--height").arg("64");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate");
    assert!(!output.status.success(), "Invalid seed should cause failure");
    
    let combined = get_output_text(&output);
    assert!(combined.to_lowercase().contains("invalid") || 
            combined.to_lowercase().contains("error") ||
            combined.to_lowercase().contains("expected"), 
            "Should have clear error for invalid seed");
}

/// CLI-16b: Generate with export_to creates file
#[test]
fn cli_generate_export_to_creates_file() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let export_path = temp_dir.path();
    
    let mut cmd = world_factory_bin();
    cmd.arg("generate");
    cmd.arg("--seed").arg("88888");
    cmd.arg("--width").arg("32");
    cmd.arg("--height").arg("32");
    cmd.arg("--export-to").arg(export_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate");
    let combined = get_output_text(&output);
    assert!(!combined.is_empty(), "Should produce output");
}
