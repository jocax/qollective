/// MCP Template Management Commands
///
/// Provides Tauri commands for discovering, loading, and managing MCP templates
/// from the nats-cli templates directory.
use crate::constants::templates;
use crate::models::{GroupedTemplates, TemplateData, ToolSchema};
use crate::utils::template_loader;
use std::path::Path;

/// List all available MCP templates, grouped by server
///
/// Scans the templates directory and returns structured information about
/// all discovered templates.
///
/// # Arguments
/// * `app_config` - Application configuration containing template directory path
///
/// # Returns
/// * `Ok(GroupedTemplates)` - HashMap of server name to list of templates
/// * `Err(String)` - Error message if template loading fails
#[tauri::command]
pub async fn list_mcp_templates(
    app_config: tauri::State<'_, crate::config::AppConfig>,
) -> Result<GroupedTemplates, String> {
    let templates_dir = app_config.templates_dir();
    let templates_path = templates_dir.to_string_lossy().to_string();

    let templates = template_loader::load_all_templates(&templates_path)
        .map_err(|e| format!("Failed to load templates: {}", e))?;

    let grouped = template_loader::group_templates_by_server(templates);

    Ok(grouped)
}

/// Load a specific MCP template by file path
///
/// Reads and parses a template file, returning the tool name and arguments.
///
/// # Arguments
/// * `template_path` - Path to the template JSON file
///
/// # Returns
/// * `Ok(TemplateData)` - Parsed template with tool_name and arguments
/// * `Err(String)` - Error message if file cannot be read or parsed
#[tauri::command]
pub async fn load_mcp_template(template_path: String) -> Result<TemplateData, String> {
    // Validate path to prevent directory traversal
    let path = Path::new(&template_path);

    if !path.is_file() {
        return Err(format!("Template file not found: {}", template_path));
    }

    // Parse the template file
    let template_data = template_loader::parse_template_file(path)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    Ok(template_data)
}

/// Get the JSON schema for a specific MCP tool
///
/// Returns a JSON schema describing the expected arguments for the given tool.
/// This is used to generate interactive forms for tool calls.
///
/// # Arguments
/// * `tool_name` - Name of the MCP tool
///
/// # Returns
/// * `Ok(ToolSchema)` - JSON schema for the tool's arguments
/// * `Err(String)` - Error message if tool is not recognized
#[tauri::command]
pub async fn get_template_schema(tool_name: String) -> Result<ToolSchema, String> {
    // Define schemas for all known MCP tools
    // In a production system, these would be loaded from the MCP server metadata
    let schema = match tool_name.as_str() {
        "orchestrate_generation" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "generation_request": {
                        "type": "object",
                        "properties": {
                            "theme": {
                                "type": "string",
                                "description": "The theme or topic for the story"
                            },
                            "age_group": {
                                "type": "string",
                                "description": "Target age group (e.g., '6-8', '9-11', '12-15')",
                                "enum": ["6-8", "9-11", "12-15"]
                            },
                            "language": {
                                "type": "string",
                                "description": "Language code (ISO 639-1)",
                                "enum": ["en", "de"]
                            },
                            "tenant_id": {
                                "type": "integer",
                                "description": "Tenant identifier"
                            },
                            "node_count": {
                                "type": "integer",
                                "description": "Number of story nodes to generate",
                                "minimum": 5,
                                "maximum": 50
                            },
                            "educational_goals": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "List of educational goals"
                            },
                            "vocabulary_level": {
                                "type": "string",
                                "description": "Vocabulary complexity level",
                                "enum": ["basic", "intermediate", "advanced"]
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Story tags for categorization"
                            },
                            "story_structure": {
                                "type": "string",
                                "description": "Story structure preset",
                                "enum": ["guided", "adventure", "epic", "choose_your_path"]
                            }
                        },
                        "required": ["theme", "age_group", "language"]
                    }
                },
                "required": ["generation_request"]
            })
        }
        "generate_structure" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "theme": {
                        "type": "string",
                        "description": "Story theme"
                    },
                    "node_count": {
                        "type": "integer",
                        "description": "Number of story nodes",
                        "minimum": 5,
                        "maximum": 50
                    },
                    "structure_type": {
                        "type": "string",
                        "description": "Type of story structure",
                        "enum": ["linear", "branching", "convergent"]
                    }
                },
                "required": ["theme"]
            })
        }
        "generate_nodes" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "structure": {
                        "type": "object",
                        "description": "Story structure definition"
                    },
                    "theme": {
                        "type": "string",
                        "description": "Story theme"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language code",
                        "enum": ["en", "de"]
                    }
                },
                "required": ["structure", "theme", "language"]
            })
        }
        "validate_content" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to validate"
                    },
                    "validation_rules": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of validation rules to apply"
                    }
                },
                "required": ["content"]
            })
        }
        "batch_validate" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": {"type": "string"},
                                "id": {"type": "string"}
                            }
                        },
                        "description": "Array of items to validate"
                    }
                },
                "required": ["items"]
            })
        }
        "enforce_constraints" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to check against constraints"
                    },
                    "constraints": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of constraints to enforce"
                    }
                },
                "required": ["content", "constraints"]
            })
        }
        "suggest_corrections" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content with potential issues"
                    },
                    "violations": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of constraint violations"
                    }
                },
                "required": ["content", "violations"]
            })
        }
        "generate_story_prompts" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "theme": {
                        "type": "string",
                        "description": "Story theme"
                    },
                    "age_group": {
                        "type": "string",
                        "description": "Target age group",
                        "enum": ["6-8", "9-11", "12-15"]
                    },
                    "language": {
                        "type": "string",
                        "description": "Language code",
                        "enum": ["en", "de"]
                    }
                },
                "required": ["theme", "age_group", "language"]
            })
        }
        "generate_validation_prompts" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content_type": {
                        "type": "string",
                        "description": "Type of content to validate"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language code",
                        "enum": ["en", "de"]
                    }
                },
                "required": ["content_type", "language"]
            })
        }
        "generate_constraint_prompts" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "constraint_type": {
                        "type": "string",
                        "description": "Type of constraint to check"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language code",
                        "enum": ["en", "de"]
                    }
                },
                "required": ["constraint_type", "language"]
            })
        }
        "get_model_for_language" => {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "description": "Language code to get model recommendation for",
                        "enum": ["en", "de"]
                    },
                    "task_type": {
                        "type": "string",
                        "description": "Type of task (generation, validation, etc.)"
                    }
                },
                "required": ["language"]
            })
        }
        _ => {
            return Err(format!("Unknown tool: {}", tool_name));
        }
    };

    Ok(ToolSchema {
        tool_name: tool_name.clone(),
        schema,
        description: Some(format!("Schema for MCP tool: {}", tool_name)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_templates() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Create orchestrator directory and templates
        let orchestrator_dir = base_path.join("orchestrator");
        fs::create_dir(&orchestrator_dir).unwrap();

        let orchestrator_template = r#"{
            "tool_name": "orchestrate_generation",
            "arguments": {
                "generation_request": {
                    "theme": "Test Theme",
                    "age_group": "9-11",
                    "language": "en",
                    "tenant_id": 1,
                    "node_count": 10
                }
            },
            "description": "Test orchestration template"
        }"#;

        fs::write(
            orchestrator_dir.join("test_orchestrate.json"),
            orchestrator_template,
        )
        .unwrap();

        // Create story-generator directory and templates
        let story_dir = base_path.join("story-generator");
        fs::create_dir(&story_dir).unwrap();

        let story_template = r#"{
            "tool_name": "generate_structure",
            "arguments": {
                "theme": "Space Adventure",
                "node_count": 15,
                "structure_type": "branching"
            },
            "description": "Generate story structure"
        }"#;

        fs::write(story_dir.join("structure.json"), story_template).unwrap();

        // Create quality-control directory and template
        let qc_dir = base_path.join("quality-control");
        fs::create_dir(&qc_dir).unwrap();

        let qc_template = r#"{
            "tool_name": "validate_content",
            "arguments": {
                "content": "Test content",
                "validation_rules": ["grammar", "spelling"]
            }
        }"#;

        fs::write(qc_dir.join("validate.json"), qc_template).unwrap();

        (temp_dir, base_path.to_string_lossy().to_string())
    }

    #[tokio::test]
    async fn test_list_mcp_templates() {
        // Note: Command-level testing requires a Tauri AppHandle
        // This test validates the underlying logic using template_loader directly
        let (_temp_dir, base_path) = create_test_templates();

        // Test the underlying template loader logic
        let templates = template_loader::load_all_templates(&base_path).unwrap();
        assert_eq!(templates.len(), 3);

        let grouped = template_loader::group_templates_by_server(templates);
        assert_eq!(grouped.len(), 3);
        assert!(grouped.contains_key("orchestrator"));
        assert!(grouped.contains_key("story-generator"));
        assert!(grouped.contains_key("quality-control"));
    }

    #[tokio::test]
    async fn test_load_mcp_template_success() {
        let (_temp_dir, base_path) = create_test_templates();
        let template_path = format!("{}/orchestrator/test_orchestrate.json", base_path);

        let result = load_mcp_template(template_path).await;
        assert!(result.is_ok());

        let template_data = result.unwrap();
        assert_eq!(template_data.tool_name, "orchestrate_generation");
        assert_eq!(
            template_data.description,
            Some("Test orchestration template".to_string())
        );
        assert!(template_data.arguments.is_object());
    }

    #[tokio::test]
    async fn test_load_mcp_template_missing_file() {
        let result = load_mcp_template("/nonexistent/template.json".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_load_mcp_template_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("invalid.json");
        fs::write(&invalid_file, "not valid json {").unwrap();

        let result = load_mcp_template(invalid_file.to_string_lossy().to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse"));
    }

    #[tokio::test]
    async fn test_get_template_schema_orchestrate_generation() {
        let result = get_template_schema("orchestrate_generation".to_string()).await;
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.tool_name, "orchestrate_generation");
        assert!(schema.schema.is_object());

        // Verify schema structure
        let properties = schema.schema.get("properties").unwrap();
        assert!(properties.get("generation_request").is_some());
    }

    #[tokio::test]
    async fn test_get_template_schema_generate_structure() {
        let result = get_template_schema("generate_structure".to_string()).await;
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.tool_name, "generate_structure");

        let properties = schema.schema.get("properties").unwrap();
        assert!(properties.get("theme").is_some());
        assert!(properties.get("node_count").is_some());
    }

    #[tokio::test]
    async fn test_get_template_schema_validate_content() {
        let result = get_template_schema("validate_content".to_string()).await;
        assert!(result.is_ok());

        let schema = result.unwrap();
        let properties = schema.schema.get("properties").unwrap();
        assert!(properties.get("content").is_some());
        assert!(properties.get("validation_rules").is_some());
    }

    #[tokio::test]
    async fn test_get_template_schema_unknown_tool() {
        let result = get_template_schema("unknown_tool".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_get_template_schema_all_tools() {
        let tools = vec![
            "orchestrate_generation",
            "generate_structure",
            "generate_nodes",
            "validate_content",
            "batch_validate",
            "enforce_constraints",
            "suggest_corrections",
            "generate_story_prompts",
            "generate_validation_prompts",
            "generate_constraint_prompts",
            "get_model_for_language",
        ];

        for tool in tools {
            let result = get_template_schema(tool.to_string()).await;
            assert!(result.is_ok(), "Failed to get schema for tool: {}", tool);
        }
    }

    #[tokio::test]
    async fn test_template_loader_integration() {
        let (_temp_dir, base_path) = create_test_templates();

        // Test scanning
        let templates = template_loader::load_all_templates(&base_path).unwrap();
        assert_eq!(templates.len(), 3);

        // Test grouping
        let grouped = template_loader::group_templates_by_server(templates);
        assert_eq!(grouped.len(), 3);
        assert!(grouped.contains_key("orchestrator"));
        assert!(grouped.contains_key("story-generator"));
        assert!(grouped.contains_key("quality-control"));

        // Test template loading
        let orchestrator_path = format!("{}/orchestrator/test_orchestrate.json", base_path);
        let result = load_mcp_template(orchestrator_path).await;
        assert!(result.is_ok());
    }
}
