/// Template loader utilities for MCP testing UI
use crate::constants::templates;
use crate::error::AppError;
use crate::models::{GroupedTemplates, TemplateData, TemplateInfo};
use std::collections::HashMap;
use std::fs;
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
pub fn scan_template_directory(base_path: &str) -> Result<Vec<PathBuf>, AppError> {
    let base = Path::new(base_path);

    if !base.exists() {
        return Err(AppError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Template directory not found: {}", base_path),
        )));
    }

    let mut template_paths = Vec::new();

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
pub fn parse_template_file(path: &Path) -> Result<TemplateData, AppError> {
    let content = fs::read_to_string(path)?;
    let template_data: TemplateData = serde_json::from_str(&content).map_err(|e| {
        AppError::JsonError(e)
    })?;

    Ok(template_data)
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
pub fn load_all_templates(base_path: &str) -> Result<Vec<TemplateInfo>, AppError> {
    let template_paths = scan_template_directory(base_path)?;
    let mut templates = Vec::new();

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
        if !templates::servers::ALL_SERVERS.contains(&server_name.as_str()) {
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

        // Parse template file
        let template_data = match parse_template_file(path) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template_structure() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Create orchestrator directory and template
        let orchestrator_dir = base_path.join("orchestrator");
        fs::create_dir(&orchestrator_dir).unwrap();

        let template_content = r#"{
            "subject": "mcp.orchestrator.request",
            "envelope": {
                "meta": {
                    "request_id": "550e8400-e29b-41d4-a716-446655440000",
                    "tenant": "1",
                    "tracing": {
                        "trace_id": "test-trace",
                        "operation_name": "orchestrate_generation"
                    }
                },
                "payload": {
                    "tool_call": {
                        "method": "tools/call",
                        "params": {
                            "name": "orchestrate_generation",
                            "arguments": {
                                "generation_request": {
                                    "theme": "Test Theme",
                                    "age_group": "9-11",
                                    "language": "en"
                                }
                            }
                        }
                    }
                }
            }
        }"#;

        fs::write(
            orchestrator_dir.join("test_template.json"),
            template_content,
        )
        .unwrap();

        // Create story-generator directory and template
        let story_dir = base_path.join("story-generator");
        fs::create_dir(&story_dir).unwrap();

        let story_template = r#"{
            "subject": "mcp.story-generator.request",
            "envelope": {
                "meta": {
                    "request_id": "550e8400-e29b-41d4-a716-446655440001",
                    "tenant": "1",
                    "tracing": {
                        "trace_id": "test-story-trace",
                        "operation_name": "generate_structure"
                    }
                },
                "payload": {
                    "tool_call": {
                        "method": "tools/call",
                        "params": {
                            "name": "generate_structure",
                            "arguments": {
                                "theme": "Space Adventure"
                            }
                        }
                    }
                }
            }
        }"#;

        fs::write(story_dir.join("space_story.json"), story_template).unwrap();

        (temp_dir, base_path)
    }

    #[test]
    fn test_scan_template_directory() {
        let (_temp_dir, base_path) = create_test_template_structure();
        let base_str = base_path.to_string_lossy().to_string();

        let result = scan_template_directory(&base_str);
        assert!(result.is_ok());

        let paths = result.unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_scan_template_directory_missing() {
        let result = scan_template_directory("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_template_file() {
        let (_temp_dir, base_path) = create_test_template_structure();
        let template_path = base_path.join("orchestrator/test_template.json");

        let result = parse_template_file(&template_path);
        assert!(result.is_ok());

        let template_data = result.unwrap();
        assert_eq!(template_data.subject, "mcp.orchestrator.request");
        // Verify envelope structure
        assert!(template_data.envelope.payload.tool_call.is_some());
        let tool_call = template_data.envelope.payload.tool_call.as_ref().unwrap();
        assert_eq!(tool_call.params.name.as_ref(), "orchestrate_generation");
    }

    #[test]
    fn test_extract_server_name() {
        let base = "/templates";
        let path = Path::new("/templates/orchestrator/test.json");

        let server_name = extract_server_name(path, base);
        assert_eq!(server_name, Some("orchestrator".to_string()));
    }

    #[test]
    fn test_extract_template_name() {
        let path = Path::new("/templates/orchestrator/test_template.json");
        let name = extract_template_name(path);
        assert_eq!(name, Some("test_template".to_string()));
    }

    #[test]
    fn test_group_templates_by_server() {
        let templates = vec![
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "test1".to_string(),
                file_path: "/path/test1.json".to_string(),
                description: None,
                tool_name: "tool1".to_string(),
            },
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "test2".to_string(),
                file_path: "/path/test2.json".to_string(),
                description: None,
                tool_name: "tool2".to_string(),
            },
            TemplateInfo {
                server_name: "story-generator".to_string(),
                template_name: "story1".to_string(),
                file_path: "/path/story1.json".to_string(),
                description: None,
                tool_name: "tool3".to_string(),
            },
        ];

        let grouped = group_templates_by_server(templates);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("orchestrator").unwrap().len(), 2);
        assert_eq!(grouped.get("story-generator").unwrap().len(), 1);
    }

    #[test]
    fn test_load_all_templates() {
        let (_temp_dir, base_path) = create_test_template_structure();
        let base_str = base_path.to_string_lossy().to_string();

        let result = load_all_templates(&base_str);
        assert!(result.is_ok());

        let templates = result.unwrap();
        assert_eq!(templates.len(), 2);

        // Check orchestrator template
        let orch_template = templates
            .iter()
            .find(|t| t.server_name == "orchestrator")
            .unwrap();
        assert_eq!(orch_template.template_name, "test_template");
        assert_eq!(orch_template.tool_name, "orchestrate_generation");
        assert_eq!(orch_template.description, None);

        // Check story-generator template
        let story_template = templates
            .iter()
            .find(|t| t.server_name == "story-generator")
            .unwrap();
        assert_eq!(story_template.template_name, "space_story");
        assert_eq!(story_template.tool_name, "generate_structure");
    }

    #[test]
    fn test_extract_server_from_subject_valid() {
        // Test valid subject for prompt-helper
        let result = extract_server_from_subject("mcp.prompt-helper.request");
        assert_eq!(result, Some("prompt-helper".to_string()));

        // Test valid subject for orchestrator
        let result = extract_server_from_subject("mcp.orchestrator.request");
        assert_eq!(result, Some("orchestrator".to_string()));

        // Test valid subject for story-generator
        let result = extract_server_from_subject("mcp.story-generator.request");
        assert_eq!(result, Some("story-generator".to_string()));

        // Test valid subject for quality-control
        let result = extract_server_from_subject("mcp.quality-control.request");
        assert_eq!(result, Some("quality-control".to_string()));

        // Test valid subject for constraint-enforcer
        let result = extract_server_from_subject("mcp.constraint-enforcer.request");
        assert_eq!(result, Some("constraint-enforcer".to_string()));
    }

    #[test]
    fn test_extract_server_from_subject_invalid_pattern() {
        // Test invalid subject with wrong number of parts
        let result = extract_server_from_subject("invalid.pattern");
        assert_eq!(result, None);

        // Test invalid subject with too many parts
        let result = extract_server_from_subject("mcp.server.request.extra");
        assert_eq!(result, None);

        // Test invalid subject with single part
        let result = extract_server_from_subject("invalid");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_server_from_subject_empty_server_name() {
        // Test empty server name
        let result = extract_server_from_subject("mcp..request");
        assert_eq!(result, None);

        // Test whitespace-only server name
        let result = extract_server_from_subject("mcp.   .request");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_server_from_subject_wrong_prefix() {
        // Test wrong prefix
        let result = extract_server_from_subject("rpc.server.request");
        assert_eq!(result, None);

        // Test wrong prefix (different protocol)
        let result = extract_server_from_subject("http.server.request");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_server_from_subject_wrong_suffix() {
        // Test wrong suffix
        let result = extract_server_from_subject("mcp.server.response");
        assert_eq!(result, None);

        // Test wrong suffix (different operation)
        let result = extract_server_from_subject("mcp.server.reply");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_server_from_subject_whitespace_trimming() {
        // Test that whitespace in server name is trimmed (though unlikely in practice)
        let result = extract_server_from_subject("mcp. orchestrator .request");
        assert_eq!(result, Some("orchestrator".to_string()));
    }

    #[test]
    fn test_extract_server_from_subject_empty_string() {
        // Test empty string
        let result = extract_server_from_subject("");
        assert_eq!(result, None);
    }
}
