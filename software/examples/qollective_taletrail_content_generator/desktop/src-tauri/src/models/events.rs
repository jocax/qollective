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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_event_creation() {
        let event = GenerationEvent::new(
            "generation_started".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "in_progress".to_string(),
        );

        assert_eq!(event.event_type, "generation_started");
        assert_eq!(event.tenant_id, "tenant-123");
        assert_eq!(event.request_id, "req-456");
        assert_eq!(event.service_phase, "story-generator");
        assert_eq!(event.status, "in_progress");
        assert!(event.progress.is_none());
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_generation_event_with_progress() {
        let event = GenerationEvent::new(
            "generation_progress".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "in_progress".to_string(),
        )
        .with_progress(0.5);

        assert_eq!(event.progress, Some(0.5));
    }

    #[test]
    fn test_generation_event_with_error() {
        let event = GenerationEvent::new(
            "generation_failed".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "failed".to_string(),
        )
        .with_error("Generation failed due to timeout".to_string());

        assert_eq!(event.status, "failed");
        assert_eq!(event.error_message, Some("Generation failed due to timeout".to_string()));
    }

    #[test]
    fn test_serialization() {
        let event = GenerationEvent::new(
            "generation_completed".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "completed".to_string(),
        )
        .with_progress(1.0);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("generation_completed"));
        assert!(json.contains("tenant-123"));
        assert!(json.contains("req-456"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "eventType": "generation_started",
            "tenantId": "tenant-123",
            "requestId": "req-456",
            "timestamp": "2025-10-22T10:00:00Z",
            "servicePhase": "story-generator",
            "status": "in_progress"
        }"#;

        let event: GenerationEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "generation_started");
        assert_eq!(event.tenant_id, "tenant-123");
        assert_eq!(event.request_id, "req-456");
    }
}
