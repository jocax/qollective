//! Internal MCP API types - Rich, complete parameters

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::enums::*;
use super::models::*;
use super::prompts::*;

/// Internal generation request (complete parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    /// Story theme
    pub theme: String,

    /// Target age group
    pub age_group: AgeGroup,

    /// Content language
    pub language: Language,

    /// Learning objectives
    #[serde(default)]
    pub educational_goals: Vec<String>,

    /// Number of story nodes in DAG
    #[serde(default = "default_node_count")]
    pub node_count: usize,

    /// Word complexity level
    #[serde(default)]
    pub vocabulary_level: VocabularyLevel,

    /// Must-include story elements
    #[serde(default)]
    pub required_elements: Vec<String>,

    /// Categorization tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// TaleTrails user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<i32>,

    /// TaleTrails tenant ID (extracted by Qollective)
    pub tenant_id: i32,

    /// Cached prompts from prompt-helper (Phase 0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_packages: Option<HashMap<String, PromptPackage>>,
}

fn default_node_count() -> usize {
    16
}

impl Default for VocabularyLevel {
    fn default() -> Self {
        VocabularyLevel::Basic
    }
}

/// Complete trail generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// Correlation ID for async tracking
    pub request_id: Uuid,

    /// Generation status
    pub status: GenerationStatus,

    /// Completion percentage (0-100)
    pub progress_percentage: i32,

    /// Complete trail structure (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<Trail>,

    /// Array of trail steps (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps: Option<Vec<TrailStep>>,

    /// Generation statistics and metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<GenerationMetadata>,

    /// Prompt generation audit trail (Phase 0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_generation_metadata: Option<PromptGenerationSummary>,

    /// Array of errors if failed
    #[serde(default)]
    pub errors: Vec<GenerationError>,
}

/// Generation statistics and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// ISO 8601 timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,

    /// Total words across all nodes
    pub total_word_count: i32,

    /// LLM model identifier
    pub ai_model_version: String,

    /// Total generation time (seconds)
    pub generation_duration_seconds: i32,

    /// Number of validation iterations
    pub validation_rounds: i32,

    /// Orchestrator service version
    pub orchestrator_version: String,
}

/// Generation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationError {
    /// Error code
    pub error_code: String,

    /// Error message
    pub error_message: String,

    /// Error timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Whether retry is possible
    pub retry_possible: bool,
}
