/// Directory management for TaleTrail data structure
///
/// Implements CONSTANTS FIRST principle with configurable root directory.
/// Manages templates and execution directories for MCP server request/response tracking.

use std::fs;
use std::path::{Path, PathBuf};
use crate::constants::mcp;

/// MCP server names from constants
pub const MCP_SERVERS: &[&str] = mcp::AVAILABLE_SERVERS;

/// Subdirectory names for the root directory structure
pub mod directory_names {
    /// Templates directory name
    pub const TEMPLATES: &str = "templates";

    /// Execution directory name
    pub const EXECUTION: &str = "execution";
}

/// Ensure the complete directory structure exists under the root path
///
/// Creates:
/// - [root]/
/// - [root]/templates/
/// - [root]/templates/[mcp_server]/ for each MCP server
/// - [root]/execution/
///
/// # Arguments
/// * `root_path` - Root directory path to create structure under
///
/// # Returns
/// * `Ok(())` - Directory structure created successfully
/// * `Err(String)` - Error message if creation fails
pub fn ensure_directory_structure(root_path: &Path) -> Result<(), String> {
    // Create root directory if it doesn't exist
    if !root_path.exists() {
        fs::create_dir_all(root_path)
            .map_err(|e| format!("Failed to create root directory {:?}: {}", root_path, e))?;
    }

    // Create templates directory
    let templates_path = root_path.join(directory_names::TEMPLATES);
    if !templates_path.exists() {
        fs::create_dir_all(&templates_path)
            .map_err(|e| format!("Failed to create templates directory {:?}: {}", templates_path, e))?;
    }

    // Create subdirectories for each MCP server
    for server in MCP_SERVERS {
        let server_path = templates_path.join(server);
        if !server_path.exists() {
            fs::create_dir_all(&server_path)
                .map_err(|e| format!("Failed to create template directory for {}: {}", server, e))?;
        }
    }

    // Create execution directory
    let execution_path = root_path.join(directory_names::EXECUTION);
    if !execution_path.exists() {
        fs::create_dir_all(&execution_path)
            .map_err(|e| format!("Failed to create execution directory {:?}: {}", execution_path, e))?;
    }

    Ok(())
}

/// Get the path to a specific MCP server's template directory
///
/// # Arguments
/// * `root_path` - Root directory path
/// * `mcp_server` - MCP server name (e.g., "orchestrator")
///
/// # Returns
/// * PathBuf to [root]/templates/[mcp_server]/
pub fn get_templates_path(root_path: &Path, mcp_server: &str) -> PathBuf {
    root_path
        .join(directory_names::TEMPLATES)
        .join(mcp_server)
}

/// Prepare execution directory for a new request
///
/// If the directory exists, it will be deleted recursively and recreated fresh.
/// Creates subdirectories for each MCP server.
///
/// # Arguments
/// * `root_path` - Root directory path
/// * `request_id` - Unique request identifier
///
/// # Returns
/// * `Ok(PathBuf)` - Path to created execution directory
/// * `Err(String)` - Error message if preparation fails
pub fn prepare_execution_directory(root_path: &Path, request_id: &str) -> Result<PathBuf, String> {
    let execution_path = root_path
        .join(directory_names::EXECUTION)
        .join(request_id);

    // Delete existing directory if it exists
    if execution_path.exists() {
        fs::remove_dir_all(&execution_path)
            .map_err(|e| format!("Failed to remove existing execution directory {:?}: {}", execution_path, e))?;
    }

    // Create fresh directory structure
    fs::create_dir_all(&execution_path)
        .map_err(|e| format!("Failed to create execution directory {:?}: {}", execution_path, e))?;

    // Create subdirectories for each MCP server
    for server in MCP_SERVERS {
        let server_path = execution_path.join(server);
        fs::create_dir_all(&server_path)
            .map_err(|e| format!("Failed to create execution subdirectory for {}: {}", server, e))?;
    }

    Ok(execution_path)
}

/// List all execution directories (request IDs)
///
/// # Arguments
/// * `root_path` - Root directory path
///
/// # Returns
/// * `Ok(Vec<String>)` - Vector of request ID strings
/// * `Err(String)` - Error message if listing fails
pub fn list_execution_directories(root_path: &Path) -> Result<Vec<String>, String> {
    let execution_path = root_path.join(directory_names::EXECUTION);

    // Return empty list if execution directory doesn't exist yet
    if !execution_path.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&execution_path)
        .map_err(|e| format!("Failed to read execution directory {:?}: {}", execution_path, e))?;

    let mut request_ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Only include directories
        if path.is_dir() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    request_ids.push(name_str.to_string());
                }
            }
        }
    }

    // Sort for consistent ordering
    request_ids.sort();

    Ok(request_ids)
}

/// Get the path to a specific MCP server's execution directory for a request
///
/// # Arguments
/// * `root_path` - Root directory path
/// * `request_id` - Request identifier
/// * `mcp_server` - MCP server name
///
/// # Returns
/// * PathBuf to [root]/execution/[request_id]/[mcp_server]/
pub fn get_execution_path(root_path: &Path, request_id: &str, mcp_server: &str) -> PathBuf {
    root_path
        .join(directory_names::EXECUTION)
        .join(request_id)
        .join(mcp_server)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_temp_dir() -> PathBuf {
        let temp_base = env::temp_dir();
        let temp_dir = temp_base.join(format!("taletrail_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        temp_dir
    }

    fn cleanup_temp_dir(path: &Path) {
        if path.exists() {
            fs::remove_dir_all(path).ok();
        }
    }

    #[test]
    fn test_ensure_directory_structure() {
        let temp_dir = create_temp_dir();

        // Create directory structure
        let result = ensure_directory_structure(&temp_dir);
        assert!(result.is_ok(), "Should create directory structure successfully");

        // Verify root exists
        assert!(temp_dir.exists(), "Root directory should exist");

        // Verify templates directory exists
        let templates_path = temp_dir.join(directory_names::TEMPLATES);
        assert!(templates_path.exists(), "Templates directory should exist");

        // Verify each MCP server directory exists
        for server in MCP_SERVERS {
            let server_path = templates_path.join(server);
            assert!(server_path.exists(), "Template directory for {} should exist", server);
        }

        // Verify execution directory exists
        let execution_path = temp_dir.join(directory_names::EXECUTION);
        assert!(execution_path.exists(), "Execution directory should exist");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_templates_path() {
        let temp_dir = create_temp_dir();
        ensure_directory_structure(&temp_dir).expect("Directory structure creation failed");

        let orchestrator_path = get_templates_path(&temp_dir, "orchestrator");
        assert_eq!(
            orchestrator_path,
            temp_dir.join(directory_names::TEMPLATES).join("orchestrator")
        );

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_prepare_execution_directory() {
        let temp_dir = create_temp_dir();
        ensure_directory_structure(&temp_dir).expect("Directory structure creation failed");

        let request_id = "test-request-123";
        let result = prepare_execution_directory(&temp_dir, request_id);
        assert!(result.is_ok(), "Should prepare execution directory successfully");

        let execution_path = result.unwrap();
        assert!(execution_path.exists(), "Execution directory should exist");

        // Verify each MCP server subdirectory exists
        for server in MCP_SERVERS {
            let server_path = execution_path.join(server);
            assert!(server_path.exists(), "Execution subdirectory for {} should exist", server);
        }

        // Test recreation (should delete existing and create fresh)
        let test_file = execution_path.join("orchestrator").join("test.txt");
        fs::write(&test_file, "test content").expect("Failed to write test file");
        assert!(test_file.exists(), "Test file should exist");

        let result2 = prepare_execution_directory(&temp_dir, request_id);
        assert!(result2.is_ok(), "Should recreate execution directory successfully");
        assert!(!test_file.exists(), "Test file should be deleted after recreation");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_list_execution_directories() {
        let temp_dir = create_temp_dir();
        ensure_directory_structure(&temp_dir).expect("Directory structure creation failed");

        // Initially empty
        let result = list_execution_directories(&temp_dir);
        assert!(result.is_ok(), "Should list directories successfully");
        assert_eq!(result.unwrap().len(), 0, "Should have no execution directories initially");

        // Create some execution directories
        prepare_execution_directory(&temp_dir, "request-1").expect("Failed to prepare directory");
        prepare_execution_directory(&temp_dir, "request-2").expect("Failed to prepare directory");
        prepare_execution_directory(&temp_dir, "request-3").expect("Failed to prepare directory");

        let result = list_execution_directories(&temp_dir);
        assert!(result.is_ok(), "Should list directories successfully");

        let directories = result.unwrap();
        assert_eq!(directories.len(), 3, "Should have 3 execution directories");
        assert_eq!(directories, vec!["request-1", "request-2", "request-3"], "Should be sorted");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_execution_path() {
        let temp_dir = create_temp_dir();
        ensure_directory_structure(&temp_dir).expect("Directory structure creation failed");

        let request_id = "test-request-456";
        prepare_execution_directory(&temp_dir, request_id).expect("Failed to prepare directory");

        let orchestrator_path = get_execution_path(&temp_dir, request_id, "orchestrator");
        assert_eq!(
            orchestrator_path,
            temp_dir.join(directory_names::EXECUTION)
                   .join(request_id)
                   .join("orchestrator")
        );
        assert!(orchestrator_path.exists(), "Execution path should exist");

        cleanup_temp_dir(&temp_dir);
    }
}
