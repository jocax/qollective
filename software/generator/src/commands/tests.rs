// ABOUTME: Unit tests for CLI command handlers
// ABOUTME: Tests validation, generation, info, and init command functionality

use super::*;
use crate::cli::{GenerateArgs, InfoArgs, InitArgs, ValidateArgs};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a simple test schema for testing
fn create_test_schema() -> String {
    r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Test Schema",
  "description": "A simple test schema for unit testing",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier"
    },
    "name": {
      "type": "string",
      "description": "Display name"
    }
  },
  "required": ["id", "name"]
}"#
    .to_string()
}

/// Create a test schema file in a temporary directory
fn create_test_schema_file(temp_dir: &TempDir) -> PathBuf {
    let schema_file = temp_dir.path().join("test_schema.json");
    fs::write(&schema_file, create_test_schema()).unwrap();
    schema_file
}

#[test]
fn test_handle_validate_success() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = ValidateArgs {
        schema_file,
        detailed: false,
        lint: false,
    };

    let result = handle_validate(&args, false, true); // quiet mode
    assert!(result.is_ok());
}

#[test]
fn test_handle_validate_with_linting() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = ValidateArgs {
        schema_file,
        detailed: true,
        lint: true,
    };

    let result = handle_validate(&args, false, true); // quiet mode
    assert!(result.is_ok());
}

#[test]
fn test_handle_validate_missing_file() {
    let args = ValidateArgs {
        schema_file: PathBuf::from("nonexistent.json"),
        detailed: false,
        lint: false,
    };

    let result = handle_validate(&args, false, true);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"));
}

#[test]
fn test_handle_info_success() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = InfoArgs {
        schema_file,
        stats: false,
        dependencies: false,
    };

    let result = handle_info(&args, false, true); // quiet mode
    assert!(result.is_ok());
}

#[test]
fn test_handle_info_with_stats() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = InfoArgs {
        schema_file,
        stats: true,
        dependencies: true,
    };

    let result = handle_info(&args, false, true); // quiet mode
    assert!(result.is_ok());
}

#[test]
fn test_handle_generate_rust_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    let args = GenerateArgs {
        schema_file,
        output: output_dir.clone(),
        language: "rust".to_string(),
        format: "single-file".to_string(),
        package_name: Some("test_package".to_string()),
        skip_validation: false,
        force: false,
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true); // quiet mode
    assert!(result.is_ok());

    // Check that the output file was created
    let expected_file = output_dir.join("test_package.rs");
    assert!(expected_file.exists());

    // Check that the file contains generated code
    let content = fs::read_to_string(expected_file).unwrap();
    // Check for either serde import or derive attributes (typify may not always include explicit imports)
    assert!(content.contains("serde") || content.contains("Serialize") || content.contains("Deserialize"));
    assert!(content.contains("struct") || content.contains("enum"));
}

#[test]
fn test_handle_generate_rust_module() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    let args = GenerateArgs {
        schema_file,
        output: output_dir.clone(),
        language: "rust".to_string(),
        format: "module".to_string(),
        package_name: Some("test_module".to_string()),
        skip_validation: false,
        force: false,
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true); // quiet mode
    assert!(result.is_ok());

    // Check that the module directory and file were created
    let module_dir = output_dir.join("test_module");
    let mod_file = module_dir.join("mod.rs");
    assert!(module_dir.exists());
    assert!(mod_file.exists());
}

#[test]
fn test_handle_generate_rust_crate() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    let args = GenerateArgs {
        schema_file,
        output: output_dir.clone(),
        language: "rust".to_string(),
        format: "crate".to_string(),
        package_name: Some("test_crate".to_string()),
        skip_validation: false,
        force: false,
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true); // quiet mode
    assert!(result.is_ok());

    // Check that the crate structure was created
    let crate_dir = output_dir.join("test_crate");
    let cargo_toml = crate_dir.join("Cargo.toml");
    let lib_rs = crate_dir.join("src").join("lib.rs");

    assert!(crate_dir.exists());
    assert!(cargo_toml.exists());
    assert!(lib_rs.exists());

    // Check Cargo.toml content
    let cargo_content = fs::read_to_string(cargo_toml).unwrap();
    assert!(cargo_content.contains("test_crate"));
    assert!(cargo_content.contains("serde"));
}

#[test]
fn test_handle_generate_unsupported_language() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = GenerateArgs {
        schema_file,
        output: temp_dir.path().join("output"),
        language: "python".to_string(), // Unsupported
        format: "module".to_string(),
        package_name: None,
        skip_validation: false,
        force: false,
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported language"));
}

#[test]
fn test_handle_generate_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = GenerateArgs {
        schema_file,
        output: temp_dir.path().join("output"),
        language: "rust".to_string(),
        format: "invalid".to_string(), // Unsupported
        package_name: None,
        skip_validation: false,
        force: false,
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported format"));
}

#[test]
fn test_handle_generate_with_skip_validation() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);

    let args = GenerateArgs {
        schema_file,
        output: temp_dir.path().join("output"),
        language: "rust".to_string(),
        format: "single-file".to_string(),
        package_name: None,
        skip_validation: true,
        force: false,
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true);
    assert!(result.is_ok());
}

#[test]
fn test_handle_generate_file_exists_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    // Create the output file first
    fs::create_dir_all(&output_dir).unwrap();
    let output_file = output_dir.join("test_schema.rs");
    fs::write(&output_file, "existing content").unwrap();

    let args = GenerateArgs {
        schema_file,
        output: output_dir,
        language: "rust".to_string(),
        format: "single-file".to_string(),
        package_name: None,
        skip_validation: false,
        force: false, // Don't force overwrite
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("already exists"));
}

#[test]
fn test_handle_generate_file_exists_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = create_test_schema_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");

    // Create the output file first
    fs::create_dir_all(&output_dir).unwrap();
    let output_file = output_dir.join("test_schema.rs");
    fs::write(&output_file, "existing content").unwrap();

    let args = GenerateArgs {
        schema_file,
        output: output_dir,
        language: "rust".to_string(),
        format: "single-file".to_string(),
        package_name: None,
        skip_validation: false,
        force: true, // Force overwrite
        schemars: false,
        additional_derives: None,
    };

    let result = handle_generate(&args, false, true);
    assert!(result.is_ok());

    // Check that the file was overwritten
    let content = fs::read_to_string(output_file).unwrap();
    // Check for either serde import or derive attributes (typify may not always include explicit imports)
    assert!(content.contains("serde") || content.contains("Serialize") || content.contains("Deserialize"));
    assert!(!content.contains("existing content"));
}

#[test]
fn test_handle_init_minimal_template() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test_project");

    let args = InitArgs {
        project_name: "test_project".to_string(),
        directory: Some(project_path.clone()),
        template: "minimal".to_string(),
    };

    let result = handle_init(&args, false, true); // quiet mode
    assert!(result.is_ok());

    // Check that the project was created
    assert!(project_path.exists());
    let schema_file = project_path.join("schema.json");
    assert!(schema_file.exists());

    // Check schema content
    let schema_content = fs::read_to_string(schema_file).unwrap();
    assert!(schema_content.contains("Example API Schema"));
}

#[test]
fn test_handle_init_full_template() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test_project_full");

    let args = InitArgs {
        project_name: "test_project_full".to_string(),
        directory: Some(project_path.clone()),
        template: "full".to_string(),
    };

    let result = handle_init(&args, false, true); // quiet mode
    assert!(result.is_ok());

    // Check that the project was created with additional files
    assert!(project_path.exists());
    assert!(project_path.join("schema.json").exists());
    assert!(project_path.join("README.md").exists());

    // Check README content
    let readme_content = fs::read_to_string(project_path.join("README.md")).unwrap();
    assert!(readme_content.contains("test_project_full"));
    assert!(readme_content.contains("Qollective framework"));
}

#[test]
fn test_handle_init_directory_exists() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("existing_dir");

    // Create the directory first
    fs::create_dir_all(&project_path).unwrap();

    let args = InitArgs {
        project_name: "existing_dir".to_string(),
        directory: Some(project_path),
        template: "minimal".to_string(),
    };

    let result = handle_init(&args, false, true);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("already exists"));
}

#[test]
fn test_handle_init_unsupported_template() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test_project");

    let args = InitArgs {
        project_name: "test_project".to_string(),
        directory: Some(project_path),
        template: "invalid_template".to_string(),
    };

    let result = handle_init(&args, false, true);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported template"));
}
