//! CLI Flag and Argument Testing (Section 14.1)
//! Tests: CLI-1 through CLI-5

use std::process::{Command, Stdio};
use std::path::Path;

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

fn get_stdout_stderr(output: &std::process::Output) -> (String, String) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    (stdout.to_string(), stderr.to_string())
}

/// CLI-1: --help flag works and displays usage information
#[test]
fn cli_help_flag_works() {
    let mut cmd = world_factory_bin();
    cmd.arg("--help");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute --help");
    
    assert!(output.status.success(), " --help should succeed");
    
    let (stdout, stderr) = get_stdout_stderr(&output);
    let combined = format!("{}\n{}", stdout, stderr);
    
    assert!(combined.contains("Usage:") || combined.contains("usage:"), 
            "Help should contain usage information");
    assert!(combined.contains("generate") || combined.contains("Generate"), 
            "Help should mention generate command");
}

/// CLI-2: Short flags are unique (no conflicts)
#[test]
fn cli_short_flags_unique() {
    let mut cmd = world_factory_bin();
    cmd.arg("--help");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute --help");
    let (stdout, stderr) = get_stdout_stderr(&output);
    let help_text = format!("{}\n{}", stdout, stderr);
    
    // Parse short flags from patterns like: "-s, --server" or "-h, --help"
    // Match -X, where X is a letter and is followed by comma at the start of a flag definition
    // Use a pattern that captures short flags in the format: -X, --long
    let short_flag_pattern = r"-([a-z]),\s+--";
    let flags: Vec<_> = regex_lite::Regex::new(short_flag_pattern)
        .unwrap()
        .find_iter(&help_text)
        .map(|m| m.as_str().chars().nth(1).unwrap())
        .collect();;
    
    // Remove duplicates
    let unique_flags: std::collections::HashSet<_> = flags.iter().collect();
    
    println!("Debug: Found {} total flags: {:?}", flags.len(), flags);
    println!("Debug: {} unique flags: {:?}", unique_flags.len(), unique_flags);
    
    // Verify all found flags are unique (no duplicates in definitions)
    assert_eq!(flags.len(), unique_flags.len(), 
               "Short flags should be unique, found duplicates");
    
    // Verify critical flags exist: -s for server, -p for port, -h for help
    assert!(unique_flags.contains(&'s'), "Should have -s flag");
    assert!(unique_flags.contains(&'p'), "Should have -p flag");
    assert!(unique_flags.contains(&'h'), "Should have -h flag");
    
    println!("Debug: Found {} total flags: {:?}", flags.len(), flags);
    println!("Debug: {} unique flags: {:?}", unique_flags.len(), unique_flags);
    println!("Found {} unique short flags: {:?}", flags.len(), unique_flags);
}

/// CLI-3: Required vs optional arguments are correctly distinguished
#[test]
fn cli_required_vs_optional_args() {
    let mut cmd = world_factory_bin();
    cmd.arg("generate").arg("--help");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute generate --help");
    let (stdout, stderr) = get_stdout_stderr(&output);
    let help_text = format!("{} {}", stdout, stderr);
    
    assert!(help_text.contains("default") || help_text.contains("128") || help_text.contains("42"), 
            "Optional arguments should show their defaults");
}

/// CLI-4: Help output is well-formatted
#[test]
fn cli_help_output_well_formatted() {
    let mut cmd = world_factory_bin();
    cmd.arg("--help");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute --help");
    let (stdout, stderr) = get_stdout_stderr(&output);
    
    assert!(!stdout.trim().is_empty(), "Help output should not be empty");
    
    let combined = stdout.len() + stderr.len();
    assert!(combined > 100, "Help should have substantial content");
}

/// CLI-5: --version flag behavior (if supported)
#[test]
fn cli_version_flag_if_supported() {
    let mut cmd = world_factory_bin();
    cmd.arg("--version");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let output = cmd.output().expect("Failed to execute --version");
    let (stdout, stderr) = get_stdout_stderr(&output);
    
    let combined = format!("{}{}", stdout, stderr);
    assert!(!combined.trim().is_empty(), "Version flag should produce output");
}
