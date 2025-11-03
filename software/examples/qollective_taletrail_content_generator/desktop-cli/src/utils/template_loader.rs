/// Template loader utilities for MCP testing UI
///
/// Uses smol-compatible async primitives for file I/O operations

use crate::constants;
use crate::error::AppError;
use crate::models::{GroupedTemplates, TemplateData, TemplateInfo};
use smol::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scan template directory and return all template file paths
///
/// # Arguments
/// * `base_path` - Base directory to scan for templates
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of template file paths
/// * `Err(AppError)` - If directory cannot be read
pub async fn scan_template_directory(base_path: &str) -> Result<Vec<PathBuf>, AppError> {
    let base = Path::new(base_path);

    if !base.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Template directory not found: {}", base_path),
        )));
    }

    let mut template_paths = Vec::new();

    // Walk directory synchronously (fast for directory traversal)
    for entry in WalkDir::new(base)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only include files with .json extension
        if path.is_file()
            && path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "json")
                .unwrap_or(false)
        {
            template_paths.push(path.to_path_buf());
        }
    }

    Ok(template_paths)
}

/// Parse a template file into TemplateData
///
/// # Arguments
/// * `path` - Path to the template JSON file
///
/// # Returns
/// * `Ok(TemplateData)` - Parsed template data
/// * `Err(AppError)` - If file cannot be read or parsed
pub async fn parse_template_file(path: &Path) -> Result<TemplateData, AppError> {
    let content = fs::read_to_string(path).await?;
    let template_data: TemplateData = serde_json::from_str(&content)
        .map_err(|e| AppError::Serialization(e))?;

    Ok(template_data)
}

/// Load template content as raw JSON string
///
/// # Arguments
/// * `template_path` - Path to the template JSON file
///
/// # Returns
/// * `Ok(String)` - Raw template JSON content
/// * `Err(AppError)` - If file cannot be read
pub async fn load_template_content(template_path: &Path) -> Result<String, AppError> {
    let content = fs::read_to_string(template_path).await?;
    Ok(content)
}

/// Extract server name from template file path
///
/// # Arguments
/// * `path` - Path to the template file
/// * `base_path` - Base templates directory path
///
/// # Returns
/// * `Option<String>` - Server name if path structure is valid
fn extract_server_name(path: &Path, base_path: &str) -> Option<String> {
    let base = Path::new(base_path);
    let relative = path.strip_prefix(base).ok()?;

    // Get the first component (server directory name)
    relative
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.to_string())
}

/// Extract template name from file path (filename without extension)
///
/// # Arguments
/// * `path` - Path to the template file
///
/// # Returns
/// * `Option<String>` - Template name without extension
fn extract_template_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Group templates by server name
///
/// # Arguments
/// * `templates` - List of template information
///
/// # Returns
/// * `GroupedTemplates` - HashMap of server name to list of templates
pub fn group_templates_by_server(templates: Vec<TemplateInfo>) -> GroupedTemplates {
    let mut grouped: GroupedTemplates = HashMap::new();

    for template in templates {
        grouped
            .entry(template.server_name.clone())
            .or_insert_with(Vec::new)
            .push(template);
    }

    grouped
}

/// Extracts the server name from a NATS subject pattern.
///
/// # Examples
/// - "mcp.prompt-helper.request" → Some("prompt-helper")
/// - "mcp.orchestrator.request" → Some("orchestrator")
/// - "invalid.subject" → None
/// - "mcp..request" → None (empty server name)
///
/// # Arguments
/// * `subject` - The NATS subject string
///
/// # Returns
/// * `Some(String)` - The extracted server name
/// * `None` - If subject doesn't match pattern or server name is empty
pub fn extract_server_from_subject(subject: &str) -> Option<String> {
    let parts: Vec<&str> = subject.split('.').collect();

    // Verify it has exactly 3 parts
    if parts.len() != 3 {
        return None;
    }

    // Verify first part is "mcp"
    if parts[0] != "mcp" {
        return None;
    }

    // Verify last part is "request"
    if parts[2] != "request" {
        return None;
    }

    // Extract middle part as server name
    let server_name = parts[1].trim();

    // Return None if server name is empty
    if server_name.is_empty() {
        return None;
    }

    Some(server_name.to_string())
}

/// Load all templates from the base directory and convert to TemplateInfo
///
/// # Arguments
/// * `base_path` - Base directory containing server subdirectories
///
/// # Returns
/// * `Ok(Vec<TemplateInfo>)` - List of discovered templates
/// * `Err(AppError)` - If scanning or parsing fails
pub async fn load_all_templates(base_path: &str) -> Result<Vec<TemplateInfo>, AppError> {
    let template_paths = scan_template_directory(base_path).await?;
    let mut templates = Vec::new();

    // Get list of valid servers from constants
    let valid_servers = constants::MCP_SERVERS;

    for path in &template_paths {
        // Extract server name from path
        let server_name = match extract_server_name(path, base_path) {
            Some(name) => name,
            None => {
                eprintln!("Warning: Could not extract server name from path: {}", path.display());
                continue;
            }
        };

        // Validate server name
        if !valid_servers.contains(&server_name.as_str()) {
            eprintln!("Warning: Unknown server name '{}' in path: {}", server_name, path.display());
            continue;
        }

        // Extract template name
        let template_name = match extract_template_name(path) {
            Some(name) => name,
            None => {
                eprintln!("Warning: Could not extract template name from path: {}", path.display());
                continue;
            }
        };

        // Parse template file (async)
        let template_data = match parse_template_file(path).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Warning: Failed to parse template {}: {}", path.display(), e);
                continue;
            }
        };

        // Extract tool name from envelope's MCP tool_call
        let tool_name = template_data
            .envelope
            .payload
            .tool_call
            .as_ref()
            .map(|tc| tc.params.name.as_ref().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        templates.push(TemplateInfo {
            server_name,
            template_name,
            file_path: path.to_string_lossy().to_string(),
            description: None, // Description is no longer part of TemplateData
            tool_name,
        });
    }

    Ok(templates)
}
