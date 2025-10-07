//! Domain model types (maps to TaleTrails database tables)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::*;

/// Trail metadata (maps to `trails` table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trail {
    /// Trail title
    pub title: String,

    /// Trail description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Trail metadata JSON
    pub metadata: serde_json::Value,

    /// Categorization tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Publication status
    pub status: TrailStatus,

    /// Category (fixed: "story" for generated content)
    pub category: String,

    /// Public visibility
    pub is_public: bool,

    /// Price in coins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<i32>,
}

/// Trail step (maps to `trail_steps` table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailStep {
    /// Sequential order
    pub step_order: i32,

    /// Step title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Step metadata
    pub metadata: serde_json::Value,

    /// Content reference (will map to content_id after DB insert)
    pub content_reference: ContentReference,

    /// Whether step is required
    pub is_required: bool,
}

/// Content reference (temporary during generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReference {
    /// Temporary node ID (before DB insert)
    pub temp_node_id: Uuid,

    /// Content structure
    pub content: Content,
}

/// Interactive story node content (maps to `content.content` JSON field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    /// Content type (fixed: "interactive_story_node")
    #[serde(rename = "type")]
    pub content_type: String,

    /// Unique node identifier
    pub node_id: Uuid,

    /// Story text for this node
    pub text: String,

    /// Available choices at this node
    pub choices: Vec<Choice>,

    /// Whether this is a convergence point
    pub convergence_point: bool,

    /// Possible next node IDs
    pub next_nodes: Vec<Uuid>,

    /// Optional educational content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_content: Option<EducationalContent>,
}

/// User decision point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Unique choice identifier
    pub id: Uuid,

    /// Choice text shown to user
    pub text: String,

    /// Target node after this choice
    pub next_node_id: Uuid,

    /// Choice-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Educational content embedded in node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalContent {
    /// Educational topic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Learning objective
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_objective: Option<String>,

    /// Vocabulary words
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_words: Option<Vec<String>>,

    /// Educational facts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_facts: Option<Vec<String>>,
}
