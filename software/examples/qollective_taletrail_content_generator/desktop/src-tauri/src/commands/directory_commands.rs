/// Tauri commands for directory management
///
/// Provides frontend interface to directory management functionality.
/// All paths are resolved through AppConfig to maintain CONSTANTS FIRST principle.

use std::fs;
use std::path::PathBuf;
use tauri::State;
use crate::config::AppConfig;
use crate::utils::directory_manager;

/// Initialize the root directory structure
///
/// Creates the complete directory hierarchy under the configured root path.
///
/// # Arguments
/// * `path` - Root directory path (optional, uses config default if empty)
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(())` - Directory structure initialized successfully
/// * `Err(String)` - Error message if initialization fails
#[tauri::command]
pub async fn initialize_root_directory(
    path: String,
    config: State<'_, AppConfig>,
) -> Result<(), String> {
    let root_path = if path.is_empty() {
        // Use default from config
        config.root_directory()
    } else {
        // Use provided path
        config.resolve_path(&path)
    };

    directory_manager::ensure_directory_structure(&root_path)
}

/// Get the path to a specific MCP server's template directory
///
/// # Arguments
/// * `server` - MCP server name (e.g., "orchestrator")
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(String)` - Absolute path to template directory
/// * `Err(String)` - Error message if server name is invalid
#[tauri::command]
pub async fn get_templates_directory(
    server: String,
    config: State<'_, AppConfig>,
) -> Result<String, String> {
    // Validate server name
    if !crate::constants::mcp::AVAILABLE_SERVERS.contains(&server.as_str()) {
        return Err(format!(
            "Invalid MCP server name: {}. Valid servers: {:?}",
            server,
            crate::constants::mcp::AVAILABLE_SERVERS
        ));
    }

    let root_path = config.root_directory();
    let templates_path = directory_manager::get_templates_path(&root_path, &server);

    Ok(templates_path
        .to_str()
        .ok_or("Failed to convert path to string")?
        .to_string())
}

/// Prepare execution directory for a new request
///
/// Deletes existing directory if present and creates fresh directory structure.
///
/// # Arguments
/// * `request_id` - Unique request identifier
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(String)` - Absolute path to prepared execution directory
/// * `Err(String)` - Error message if preparation fails
#[tauri::command]
pub async fn prepare_execution_directory(
    request_id: String,
    config: State<'_, AppConfig>,
) -> Result<String, String> {
    let root_path = config.root_directory();
    let execution_path = directory_manager::prepare_execution_directory(&root_path, &request_id)?;

    Ok(execution_path
        .to_str()
        .ok_or("Failed to convert path to string")?
        .to_string())
}

/// List all execution directories (request IDs)
///
/// # Arguments
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(Vec<String>)` - Vector of request ID strings
/// * `Err(String)` - Error message if listing fails
#[tauri::command]
pub async fn list_execution_directories(
    config: State<'_, AppConfig>,
) -> Result<Vec<String>, String> {
    let root_path = config.root_directory();
    directory_manager::list_execution_directories(&root_path)
}

/// Save request file for a specific MCP server
///
/// Writes request content to [root]/execution/[request-id]/[server]/request.json
///
/// # Arguments
/// * `request_id` - Request identifier
/// * `server` - MCP server name (extracted from NATS subject)
/// * `content` - JSON content to save
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(())` - File saved successfully
/// * `Err(String)` - Error message if save fails
#[tauri::command]
pub async fn save_request_file(
    request_id: String,
    server: String,
    content: String,
    config: State<'_, AppConfig>,
) -> Result<(), String> {
    eprintln!("[TaleTrail] Saving request file for server: {}, request_id: {}", server, request_id);

    // Validate server name
    if !crate::constants::mcp::AVAILABLE_SERVERS.contains(&server.as_str()) {
        return Err(format!(
            "Invalid MCP server name: {}. Valid servers: {:?}",
            server,
            crate::constants::mcp::AVAILABLE_SERVERS
        ));
    }

    let root_path = config.root_directory();
    let execution_path = directory_manager::get_execution_path(&root_path, &request_id, &server);

    // Ensure the execution directory exists
    if !execution_path.exists() {
        fs::create_dir_all(&execution_path)
            .map_err(|e| format!("Failed to create execution directory {:?}: {}", execution_path, e))?;
    }

    let file_path = execution_path.join("request.json");
    eprintln!("[TaleTrail] Writing request to: {:?}", file_path);

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write request file {:?}: {}", file_path, e))?;

    Ok(())
}

/// Save response file for a specific MCP server
///
/// Writes response content to [root]/execution/[request-id]/[server]/response.json
///
/// # Arguments
/// * `request_id` - Request identifier
/// * `server` - MCP server name (extracted from NATS subject)
/// * `content` - JSON content to save
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(())` - File saved successfully
/// * `Err(String)` - Error message if save fails
#[tauri::command]
pub async fn save_response_file(
    request_id: String,
    server: String,
    content: String,
    config: State<'_, AppConfig>,
) -> Result<(), String> {
    eprintln!("[TaleTrail] Saving response file for server: {}, request_id: {}", server, request_id);

    // Validate server name
    if !crate::constants::mcp::AVAILABLE_SERVERS.contains(&server.as_str()) {
        return Err(format!(
            "Invalid MCP server name: {}. Valid servers: {:?}",
            server,
            crate::constants::mcp::AVAILABLE_SERVERS
        ));
    }

    let root_path = config.root_directory();
    let execution_path = directory_manager::get_execution_path(&root_path, &request_id, &server);

    // Ensure the execution directory exists
    if !execution_path.exists() {
        fs::create_dir_all(&execution_path)
            .map_err(|e| format!("Failed to create execution directory {:?}: {}", execution_path, e))?;
    }

    let file_path = execution_path.join("response.json");
    eprintln!("[TaleTrail] Writing response to: {:?}", file_path);

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write response file {:?}: {}", file_path, e))?;

    Ok(())
}

/// Load execution file (request or response) for a specific MCP server
///
/// Reads file content from [root]/execution/[request-id]/[server]/[file_type].json
///
/// # Arguments
/// * `request_id` - Request identifier
/// * `server` - MCP server name
/// * `file_type` - File type to load ("request" or "response")
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(String)` - File content
/// * `Err(String)` - Error message if load fails
#[tauri::command]
pub async fn load_execution_file(
    request_id: String,
    server: String,
    file_type: String,
    config: State<'_, AppConfig>,
) -> Result<String, String> {
    // Validate server name
    if !crate::constants::mcp::AVAILABLE_SERVERS.contains(&server.as_str()) {
        return Err(format!(
            "Invalid MCP server name: {}. Valid servers: {:?}",
            server,
            crate::constants::mcp::AVAILABLE_SERVERS
        ));
    }

    // Validate file type
    let filename = match file_type.as_str() {
        "request" => "request.json",
        "response" => "response.json",
        _ => {
            return Err(format!(
                "Invalid file type: {}. Valid types: request, response",
                file_type
            ))
        }
    };

    let root_path = config.root_directory();
    let execution_path = directory_manager::get_execution_path(&root_path, &request_id, &server);
    let file_path = execution_path.join(filename);

    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path));
    }

    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file {:?}: {}", file_path, e))
}

/// Initialize templates from source (manual trigger)
///
/// Copies template files from source directory to runtime directory with "_example.json" suffix.
/// This can be called manually from the UI if automatic initialization fails.
///
/// # Arguments
/// * `config` - Application configuration state
///
/// # Returns
/// * `Ok(String)` - Success message with count of files copied
/// * `Err(String)` - Error message if initialization fails
#[tauri::command]
pub async fn initialize_templates(
    config: State<'_, AppConfig>,
) -> Result<String, String> {
    let root_path = config.root_directory();
    let source_templates_path = config.source_templates_dir();

    eprintln!("[TaleTrail] Manual template initialization requested");
    eprintln!("[TaleTrail] Root: {:?}", root_path);
    eprintln!("[TaleTrail] Source: {:?}", source_templates_path);

    // Ensure directory structure exists
    directory_manager::ensure_directory_structure(&root_path)?;

    // Initialize templates
    directory_manager::initialize_templates_from_source(&root_path, &source_templates_path)?;

    let success_msg = format!("Templates initialized successfully from {:?}", source_templates_path);
    eprintln!("[TaleTrail] {}", success_msg);
    Ok(success_msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use crate::utils::directory_manager;

    fn create_test_config() -> AppConfig {
        let mut config = AppConfig::create_test_app_config();

        // Override root directory to use temp directory for tests
        let temp_dir = env::temp_dir().join(format!("taletrail_cmd_test_{}", uuid::Uuid::new_v4()));
        config.paths.root_directory = temp_dir.to_string_lossy().to_string();

        config
    }

    fn cleanup_test_dir(config: &AppConfig) {
        let root_path = config.root_directory();
        if root_path.exists() {
            fs::remove_dir_all(root_path).ok();
        }
    }

    #[test]
    fn test_directory_manager_integration() {
        let config = create_test_config();
        let root_path = config.root_directory();

        // Test directory structure creation
        let result = directory_manager::ensure_directory_structure(&root_path);
        assert!(result.is_ok(), "Should create directory structure successfully");
        assert!(root_path.exists(), "Root directory should exist");

        // Test templates path
        let templates_path = directory_manager::get_templates_path(&root_path, "orchestrator");
        assert!(templates_path.exists(), "Template directory should exist");

        // Test execution directory preparation
        let exec_result = directory_manager::prepare_execution_directory(&root_path, "test-request");
        assert!(exec_result.is_ok(), "Should prepare execution directory successfully");

        // Test listing execution directories
        let list_result = directory_manager::list_execution_directories(&root_path);
        assert!(list_result.is_ok(), "Should list directories successfully");
        assert_eq!(list_result.unwrap().len(), 1, "Should have 1 execution directory");

        // Test file operations
        let exec_path = directory_manager::get_execution_path(&root_path, "test-request", "orchestrator");
        let request_file = exec_path.join("request.json");
        fs::write(&request_file, r#"{"test": "data"}"#).expect("Failed to write test file");
        assert!(request_file.exists(), "Request file should exist");

        let content = fs::read_to_string(&request_file).expect("Failed to read test file");
        assert_eq!(content, r#"{"test": "data"}"#, "Content should match");

        cleanup_test_dir(&config);
    }

    #[test]
    fn test_server_validation() {
        // Test valid server names
        for server in crate::constants::mcp::AVAILABLE_SERVERS {
            assert!(
                crate::constants::mcp::AVAILABLE_SERVERS.contains(server),
                "Server {} should be in available servers list",
                server
            );
        }

        // Test invalid server name would be rejected
        let invalid_server = "invalid-server";
        assert!(
            !crate::constants::mcp::AVAILABLE_SERVERS.contains(&invalid_server),
            "Invalid server should not be in available servers list"
        );
    }
}
