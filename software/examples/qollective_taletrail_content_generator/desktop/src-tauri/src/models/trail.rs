use serde::{Deserialize, Serialize};

// Re-export shared types from shared-types-generated
pub use shared_types_generated::{
    Choice,
    ContentNode,
    DAG,
    Edge,
    GenerationMetadata,
    GenerationResponse,
    GenerationStatus,
    PipelineExecutionTrace,
    ServiceInvocation,
    Trail,
    TrailStep as SharedTrailStep,
};

/// List item for trail directory view (desktop-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailListItem {
    pub id: String,
    pub file_path: String,
    pub title: String,
    pub description: String,
    pub theme: String,
    pub age_group: String,
    pub language: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: String,
    pub generated_at: String,
    pub node_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Envelope wrapper for response files (desktop-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub meta: EnvelopeMeta,
    pub payload: EnvelopePayload,
}

/// Metadata section of the envelope (desktop-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeMeta {
    pub request_id: String,
    pub timestamp: String,
    #[serde(default)]
    pub tenant: String,
    #[serde(default)]
    pub version: String,
}

/// Payload section of the envelope (desktop-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopePayload {
    pub tool_response: ToolResponse,
}

/// Tool response within the envelope (desktop-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub content: Vec<ContentItem>,
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

/// Content item within tool response (desktop-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Trail step for linear view (simplified view of a node) - desktop-specific
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailStep {
    pub id: String,
    pub text: String,
    pub choices: Vec<Choice>,
    pub is_convergence: bool,
}

impl From<&ContentNode> for TrailStep {
    fn from(node: &ContentNode) -> Self {
        TrailStep {
            id: node.id.clone(),
            text: node.content.text.clone(),
            choices: node.content.choices.clone(),
            is_convergence: false, // Will be set by the caller based on DAG
        }
    }
}
