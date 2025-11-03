use serde::{Deserialize, Serialize};

/// Represents a generation event from the TaleTrail content pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationEvent {
    /// Type of event (e.g., "generation_started", "generation_progress", "generation_completed")
    pub event_type: String,

    /// Tenant ID associated with this generation
    pub tenant_id: String,

    /// Unique request ID for tracking
    pub request_id: String,

    /// ISO 8601 timestamp of the event
    pub timestamp: String,

    /// Service phase (e.g., "prompt-helper", "story-generator", "constraint-enforcer")
    pub service_phase: String,

    /// Current status (e.g., "in_progress", "completed", "failed")
    pub status: String,

    /// Optional progress percentage (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,

    /// Optional error message if status is "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Optional file path for completed trails
    /// Populated only when status = "completed" and trail has been saved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

impl GenerationEvent {
    /// Create a new generation event
    pub fn new(
        event_type: String,
        tenant_id: String,
        request_id: String,
        service_phase: String,
        status: String,
    ) -> Self {
        Self {
            event_type,
            tenant_id,
            request_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            service_phase,
            status,
            progress: None,
            error_message: None,
            file_path: None,
        }
    }

    /// Create a new event with progress
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Create a new event with an error message
    pub fn with_error(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self
    }

    /// Create a new event with a file path
    pub fn with_file_path(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }
}

/// Application-level events for UI state updates
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// NATS connection status changed
    NatsConnectionChanged { connected: bool, error: Option<String> },

    /// New NATS message received
    NatsMessageReceived { subject: String, payload: Vec<u8> },

    /// MCP request sent
    McpRequestSent { request_id: String, server: String, tool: String },

    /// MCP response received
    McpResponseReceived { request_id: String, success: bool },

    /// Trail loaded successfully
    TrailLoaded { trail_id: String, file_path: String },

    /// Trail load failed
    TrailLoadFailed { file_path: String, error: String },

    /// Bookmark toggled
    BookmarkToggled { trail_id: String, bookmarked: bool },

    /// View changed
    ViewChanged { new_view: String },

    /// Error occurred
    Error { message: String, details: Option<String> },

    /// Request to quit application
    Quit,
}

impl AppEvent {
    /// Check if this is an error event
    pub fn is_error(&self) -> bool {
        matches!(self, AppEvent::Error { .. })
    }

    /// Get error message if this is an error event
    pub fn error_message(&self) -> Option<&str> {
        match self {
            AppEvent::Error { message, .. } => Some(message),
            _ => None,
        }
    }
}
