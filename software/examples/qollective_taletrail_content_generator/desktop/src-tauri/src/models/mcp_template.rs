/// Data models for MCP template management
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
