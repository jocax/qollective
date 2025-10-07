//! External API types (v1) - Simple, versioned, stable API

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::*;

/// Simplified request structure for content generation (External API V1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalGenerationRequestV1 {
    /// Story theme (e.g., "underwater adventure")
    #[serde(rename = "theme")]
    pub theme: String,

    /// Target age group
    #[serde(rename = "age_group")]
    pub age_group: AgeGroup,

    /// Content language
    #[serde(rename = "language")]
    pub language: Language,
}

/// Trail structure ready for database insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailInsertData {
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

    /// Public visibility
    pub is_public: bool,

    /// Price in coins (null for free)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<i32>,
}

/// Trail step structure ready for database insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailStepInsertData {
    /// Sequential order of step
    pub step_order: i32,

    /// Step title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Step metadata
    pub metadata: serde_json::Value,

    /// Interactive story node content
    pub content_data: serde_json::Value,

    /// Whether step is required
    pub is_required: bool,
}

/// Simplified response structure (External API V1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalGenerationResponseV1 {
    /// Job identifier for tracking
    pub job_id: Uuid,

    /// Final job status
    pub status: String, // "completed" or "failed"

    /// Trail structure ready for DB insert (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_data: Option<TrailInsertData>,

    /// Trail steps ready for DB insert (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps_data: Option<Vec<TrailStepInsertData>>,

    /// Error details (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExternalError>,

    /// Generation statistics and metadata
    pub metadata: serde_json::Value,
}

/// Status polling response for asynchronous generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalJobStatus {
    /// Job identifier
    pub job_id: Uuid,

    /// Current status
    pub status: String, // "pending", "in_progress", "completed", "failed"

    /// Completion percentage (0-100)
    pub progress_percentage: i32,

    /// Human-readable phase description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<String>,

    /// Estimated time to completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_seconds: Option<i32>,
}

/// External API error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalError {
    /// Error code
    pub error_code: String,

    /// Error message
    pub error_message: String,

    /// Error timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Whether retry is possible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_possible: Option<bool>,
}
