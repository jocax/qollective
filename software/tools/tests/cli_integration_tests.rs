// ABOUTME: CLI integration tests to verify end-to-end binary functionality
// ABOUTME: Tests the qollective CLI binary with real schema files and various scenarios

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_validate_with_valid_schema() {
    let schema_path = "../schemas/core/config.json";
    if !Path::new(schema_path).exists() {
        // Skip test if schema file doesn't exist in test environment
        return;
    }

    let output = Command::new("cargo")
        .args(&["run", "--", "validate", schema_path, "--quiet"])
        .output()
        .expect("Failed to run CLI command");

    assert!(
        output.status.success(),
        "CLI command failed with exit code: {}",
        output.status
    );
}

#[test]
fn test_cli_validate_with_nonexistent_file() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "validate",
            "nonexistent_schema.json",
            "--quiet",
        ])
        .output()
        .expect("Failed to run CLI command");

    assert!(
        !output.status.success(),
        "CLI should fail with nonexistent file"
    );

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
    assert!(stderr.contains("Schema file not found"));
}

#[test]
fn test_cli_with_no_arguments() {
    let output = Command::new("cargo")
        .args(&["run"])
        .output()
        .expect("Failed to run CLI command");

    assert!(
        !output.status.success(),
        "CLI should fail with no arguments"
    );

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_cli_validate_with_invalid_json() {
    // Create a temporary invalid JSON file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("invalid_schema.json");
    std::fs::write(&temp_file, "{ invalid json }").expect("Failed to write temp file");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "validate",
            temp_file.to_str().unwrap(),
            "--quiet",
        ])
        .output()
        .expect("Failed to run CLI command");

    assert!(
        !output.status.success(),
        "CLI should fail with invalid JSON"
    );

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");
    assert!(stderr.contains("Failed to parse schema"));
}

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to run CLI command");

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    assert!(stdout.contains("Generate type-safe client libraries"));
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("info"));
    assert!(stdout.contains("init"));
}

#[test]
fn test_cli_generate_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "generate", "--help"])
        .output()
        .expect("Failed to run CLI command");

    assert!(
        output.status.success(),
        "Generate help command should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    assert!(stdout.contains("Generate code from a schema file"));
    assert!(stdout.contains("--language"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--format"));
}

#[test]
fn test_cli_init_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("test_project");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "init",
            "test_project",
            "--directory",
            project_dir.to_str().unwrap(),
            "--template",
            "minimal",
            "--quiet",
        ])
        .output()
        .expect("Failed to run CLI command");

    assert!(
        output.status.success(),
        "Init command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that files were created
    assert!(project_dir.exists());
    assert!(project_dir.join("schema.json").exists());
}
