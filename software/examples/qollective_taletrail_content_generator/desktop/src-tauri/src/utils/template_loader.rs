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

        templates.push(TemplateInfo {
            server_name,
            template_name,
            file_path: path.to_string_lossy().to_string(),
            description: template_data.description,
            tool_name: template_data.tool_name,
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
            "tool_name": "orchestrate_generation",
            "arguments": {
                "generation_request": {
                    "theme": "Test Theme",
                    "age_group": "9-11",
                    "language": "en"
                }
            },
            "description": "Test orchestrator template"
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
            "tool_name": "generate_structure",
            "arguments": {
                "theme": "Space Adventure"
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
        assert_eq!(template_data.tool_name, "orchestrate_generation");
        assert_eq!(
            template_data.description,
            Some("Test orchestrator template".to_string())
        );
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
        assert_eq!(
            orch_template.description,
            Some("Test orchestrator template".to_string())
        );

        // Check story-generator template
        let story_template = templates
            .iter()
            .find(|t| t.server_name == "story-generator")
            .unwrap();
        assert_eq!(story_template.template_name, "space_story");
        assert_eq!(story_template.tool_name, "generate_structure");
    }
}
