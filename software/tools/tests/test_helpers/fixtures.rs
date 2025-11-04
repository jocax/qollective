// ABOUTME: Test fixtures and shared setup utilities
// ABOUTME: Provides reusable test project creation and compilation verification

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Cargo.toml template with all required dependencies
pub const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "schema_test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
"#;

/// Helper function to verify that generated Rust code compiles
///
/// Creates a temporary Cargo project with the generated code and runs `cargo check`
/// to ensure the code is syntactically valid and type-safe.
pub fn verify_compilation(generated_code: &str) -> Result<(), String> {
    // Create temporary directory for test project
    let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
    let project_path = temp_dir.path();

    // Write Cargo.toml
    fs::write(project_path.join("Cargo.toml"), CARGO_TOML_TEMPLATE)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Create src directory
    fs::create_dir(project_path.join("src"))
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    // Write generated code to src/lib.rs
    fs::write(project_path.join("src/lib.rs"), generated_code)
        .map_err(|e| format!("Failed to write generated code: {}", e))?;

    // Run cargo check (faster than full build)
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "Compilation failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        ))
    }
}

/// Create a test Cargo project with generated code
pub fn create_test_project(generated_code: &str) -> Result<TempDir, String> {
    let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
    let project_path = temp_dir.path();

    fs::write(project_path.join("Cargo.toml"), CARGO_TOML_TEMPLATE)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    fs::create_dir(project_path.join("src"))
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    fs::write(project_path.join("src/lib.rs"), generated_code)
        .map_err(|e| format!("Failed to write generated code: {}", e))?;

    Ok(temp_dir)
}
