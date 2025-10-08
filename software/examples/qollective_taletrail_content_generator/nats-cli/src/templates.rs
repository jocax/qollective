//! Template management for MCP tool call requests
//!
//! Discovers, loads, and parses JSON template files organized by MCP server

use crate::constants::*;
use crate::errors::{NatsCliError, Result};
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam, Extensions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Template information for display
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Server name (directory name under templates/)
    pub server_name: String,
    /// Template name (filename without .json extension)
    pub template_name: String,
    /// Full path to template file
    pub path: PathBuf,
    /// Tool name from template content
    pub tool_name: Option<String>,
}

/// Template data structure (matches JSON template format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateData {
    /// MCP tool name to call
    pub tool_name: String,
    /// Tool arguments (can be any JSON value)
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Template manager for discovering and loading templates
pub struct TemplateManager {
    /// Base templates directory
    templates_dir: PathBuf,
}

impl TemplateManager {
    /// Create a new template manager
    ///
    /// # Arguments
    /// * `templates_dir` - Path to templates directory
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
        }
    }

    /// Create template manager with default directory
    pub fn with_default_dir() -> Result<Self> {
        let templates_dir = PathBuf::from(TEMPLATES_DIR);

        if !templates_dir.exists() {
            return Err(NatsCliError::ConfigError(format!(
                "Templates directory not found: {}",
                templates_dir.display()
            )));
        }

        Ok(Self::new(templates_dir))
    }

    /// List all available templates, optionally filtered by server name
    ///
    /// # Arguments
    /// * `server_filter` - Optional server name to filter by
    ///
    /// # Returns
    /// * `Result<Vec<TemplateInfo>>` - List of template information
    pub fn list_templates(&self, server_filter: Option<&str>) -> Result<Vec<TemplateInfo>> {
        let mut templates = Vec::new();

        debug!(
            "Scanning templates directory: {}",
            self.templates_dir.display()
        );

        // Walk the templates directory
        for entry in WalkDir::new(&self.templates_dir)
            .min_depth(2)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip non-JSON files
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Skip README files
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case(TEMPLATE_README))
                .unwrap_or(false)
            {
                continue;
            }

            // Extract server name from parent directory
            let server_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    NatsCliError::InvalidTemplate(format!(
                        "Cannot determine server name for template: {}",
                        path.display()
                    ))
                })?
                .to_string();

            // Apply server filter if provided
            if let Some(filter) = server_filter {
                if server_name != filter {
                    continue;
                }
            }

            // Extract template name from filename
            let template_name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    NatsCliError::InvalidTemplate(format!(
                        "Invalid template filename: {}",
                        path.display()
                    ))
                })?
                .to_string();

            // Try to extract tool name from template (non-fatal if fails)
            let tool_name = self.extract_tool_name(path).ok();

            templates.push(TemplateInfo {
                server_name,
                template_name,
                path: path.to_path_buf(),
                tool_name,
            });
        }

        if templates.is_empty() {
            info!(
                "{} No templates found in {}",
                WARNING_PREFIX,
                self.templates_dir.display()
            );
        } else {
            debug!("Found {} templates", templates.len());
        }

        // Sort by server name, then template name
        templates.sort_by(|a, b| {
            a.server_name
                .cmp(&b.server_name)
                .then(a.template_name.cmp(&b.template_name))
        });

        Ok(templates)
    }

    /// Load a template from a file path
    ///
    /// # Arguments
    /// * `path` - Path to template file (absolute or relative)
    ///
    /// # Returns
    /// * `Result<CallToolRequest>` - MCP tool call request
    pub fn load_template(&self, path: impl AsRef<Path>) -> Result<CallToolRequest> {
        let path = path.as_ref();

        debug!("Loading template from: {}", path.display());

        // Read template file
        let content = fs::read_to_string(path).map_err(|e| {
            NatsCliError::TemplateNotFound(format!(
                "Cannot read template {}: {}",
                path.display(),
                e
            ))
        })?;

        // Parse template JSON
        let template: TemplateData = serde_json::from_str(&content).map_err(|e| {
            NatsCliError::InvalidTemplate(format!(
                "Invalid template format in {}: {}",
                path.display(),
                e
            ))
        })?;

        // Convert to CallToolRequest - arguments must be a Map
        let arguments = if template.arguments.is_null() {
            None
        } else if let serde_json::Value::Object(map) = template.arguments {
            Some(map)
        } else {
            return Err(NatsCliError::InvalidTemplate(format!(
                "Template arguments must be an object in {}",
                path.display()
            )));
        };

        let request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: template.tool_name.into(),
                arguments,
            },
            extensions: Extensions::default(),
        };

        debug!("Loaded template: tool={}", request.params.name);

        Ok(request)
    }

    /// Load a template by server and template name
    ///
    /// # Arguments
    /// * `server_name` - Server name (directory under templates/)
    /// * `template_name` - Template name (filename without .json)
    ///
    /// # Returns
    /// * `Result<CallToolRequest>` - MCP tool call request
    pub fn load_template_by_name(
        &self,
        server_name: &str,
        template_name: &str,
    ) -> Result<CallToolRequest> {
        let path = self
            .templates_dir
            .join(server_name)
            .join(format!("{}{}", template_name, TEMPLATE_EXTENSION));

        self.load_template(path)
    }

    /// Extract tool name from a template file (non-parsing, just quick peek)
    fn extract_tool_name(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)?;
        let template: TemplateData = serde_json::from_str(&content)?;
        Ok(template.tool_name)
    }

    /// Get list of server names (subdirectories in templates/)
    pub fn list_servers(&self) -> Result<Vec<String>> {
        let mut servers = Vec::new();

        for entry in fs::read_dir(&self.templates_dir).map_err(|e| {
            NatsCliError::ConfigError(format!(
                "Cannot read templates directory {}: {}",
                self.templates_dir.display(),
                e
            ))
        })? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    servers.push(name.to_string());
                }
            }
        }

        servers.sort();
        Ok(servers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_data_parsing() {
        let json = r#"
        {
            "tool_name": "generate_story_prompts",
            "arguments": {
                "theme": "Space Adventure",
                "age_group": "6-8"
            }
        }
        "#;

        let template: TemplateData = serde_json::from_str(json).unwrap();
        assert_eq!(template.tool_name, "generate_story_prompts");
        assert!(template.arguments.is_object());
    }

    #[test]
    fn test_template_data_no_arguments() {
        let json = r#"
        {
            "tool_name": "list_tools"
        }
        "#;

        let template: TemplateData = serde_json::from_str(json).unwrap();
        assert_eq!(template.tool_name, "list_tools");
        assert!(template.arguments.is_null());
    }
}
