/// Data models for MCP (Model Context Protocol) integration
use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a discovered MCP template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// The MCP server this template is for (e.g., "orchestrator", "story-generator")
    pub server_name: String,

    /// The template filename without extension
    pub template_name: String,

    /// Full path to the template file
    pub file_path: String,

    /// Optional description extracted from the template
    pub description: Option<String>,

    /// The tool name this template calls
    pub tool_name: String,
}

/// Raw template data parsed from JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateData {
    /// NATS subject to send the request to
    pub subject: String,

    /// Complete Qollective envelope with metadata and payload
    pub envelope: Envelope<McpData>,

    /// Optional schema for validation (kept for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ToolSchema>,
}

/// MCP tool schema for form generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// The tool name
    pub tool_name: String,

    /// JSON schema describing the tool's arguments
    pub schema: serde_json::Value,

    /// Optional description of the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Grouped templates by server name
pub type GroupedTemplates = HashMap<String, Vec<TemplateInfo>>;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name (e.g., "orchestrator", "story-generator")
    pub name: String,

    /// NATS subject pattern for this server
    pub subject: String,

    /// Whether this server is currently available
    pub available: bool,

    /// Optional description of the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl McpServerConfig {
    /// Create a new MCP server configuration
    pub fn new(name: String, subject: String) -> Self {
        Self {
            name,
            subject,
            available: false,
            description: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set availability status
    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }
}

/// Predefined MCP servers for TaleTrail
pub const MCP_SERVERS: &[(&str, &str, &str)] = &[
    (
        "orchestrator",
        "mcp.orchestrator.>",
        "Main orchestrator for story generation workflow"
    ),
    (
        "story-generator",
        "mcp.story-generator.>",
        "Generates story content and narrative elements"
    ),
    (
        "quality-control",
        "mcp.quality-control.>",
        "Validates and ensures story quality standards"
    ),
    (
        "constraint-enforcer",
        "mcp.constraint-enforcer.>",
        "Enforces story constraints and requirements"
    ),
    (
        "prompt-helper",
        "mcp.prompt-helper.>",
        "Assists with prompt generation and optimization"
    ),
];

impl McpServerConfig {
    /// Get all predefined MCP server configurations
    pub fn all_servers() -> Vec<Self> {
        MCP_SERVERS
            .iter()
            .map(|(name, subject, desc)| {
                Self::new(name.to_string(), subject.to_string())
                    .with_description(desc.to_string())
            })
            .collect()
    }

    /// Get server config by name
    pub fn get_server(name: &str) -> Option<Self> {
        MCP_SERVERS
            .iter()
            .find(|(n, _, _)| *n == name)
            .map(|(name, subject, desc)| {
                Self::new(name.to_string(), subject.to_string())
                    .with_description(desc.to_string())
            })
    }
}
