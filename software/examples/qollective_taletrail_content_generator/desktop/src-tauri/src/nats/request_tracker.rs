/// Request tracker module for monitoring active generation requests
///
/// Provides thread-safe tracking of active requests with automatic cleanup
/// of stale requests based on configurable timeout.

use crate::models::GenerationEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a tracked generation request with its current state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedRequest {
    /// Unique request ID
    pub request_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// When the request was first tracked
    pub start_time: DateTime<Utc>,
    /// Current service phase (e.g., "story-generator", "quality-control")
    pub current_phase: String,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f32,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
    /// Current component processing the request
    pub component: String,
    /// Current status (e.g., "in_progress", "completed", "failed")
    pub status: String,
    /// Optional error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Optional file path for completed trails
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

impl TrackedRequest {
    /// Create a new tracked request from a GenerationEvent
    pub fn from_event(event: &GenerationEvent) -> Self {
        Self {
            request_id: event.request_id.clone(),
            tenant_id: event.tenant_id.clone(),
            start_time: Utc::now(),
            current_phase: event.service_phase.clone(),
            progress: event.progress.unwrap_or(0.0),
            last_update: Utc::now(),
            component: event.service_phase.clone(),
            status: event.status.clone(),
            error_message: event.error_message.clone(),
            file_path: event.file_path.clone(),
        }
    }

    /// Update from a GenerationEvent
    pub fn update_from_event(&mut self, event: &GenerationEvent) {
        self.current_phase = event.service_phase.clone();
        self.progress = event.progress.unwrap_or(self.progress);
        self.last_update = Utc::now();
        self.component = event.service_phase.clone();
        self.status = event.status.clone();

        if let Some(ref error) = event.error_message {
            self.error_message = Some(error.clone());
        }

        if let Some(ref file_path) = event.file_path {
            self.file_path = Some(file_path.clone());
        }
    }
}

/// Thread-safe request tracker for managing active generation requests
pub struct RequestTracker {
    /// Map of request_id -> TrackedRequest
    active_requests: Arc<RwLock<HashMap<String, TrackedRequest>>>,
    /// Cleanup timeout in seconds (requests older than this are removed)
    cleanup_timeout_secs: u64,
}

impl RequestTracker {
    /// Create a new RequestTracker with the specified cleanup timeout
    ///
    /// # Arguments
    /// * `cleanup_timeout_secs` - Timeout in seconds after which inactive requests are removed
    pub fn new(cleanup_timeout_secs: u64) -> Self {
        Self {
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            cleanup_timeout_secs,
        }
    }

    /// Track a new request or update an existing one
    ///
    /// # Arguments
    /// * `request` - The TrackedRequest to track
    pub async fn track_request(&self, request: TrackedRequest) {
        let mut requests = self.active_requests.write().await;
        requests.insert(request.request_id.clone(), request);
    }

    /// Update an existing tracked request
    ///
    /// # Arguments
    /// * `request_id` - The request ID to update
    /// * `phase` - New service phase
    /// * `progress` - New progress value (0.0 to 1.0)
    /// * `component` - Component currently processing
    /// * `status` - Current status
    pub async fn update_request(
        &self,
        request_id: &str,
        phase: String,
        progress: f32,
        component: String,
        status: String,
    ) {
        let mut requests = self.active_requests.write().await;
        if let Some(request) = requests.get_mut(request_id) {
            request.current_phase = phase;
            request.progress = progress;
            request.component = component;
            request.status = status;
            request.last_update = Utc::now();
        }
    }

    /// Update a tracked request from a GenerationEvent
    ///
    /// Creates a new tracked request if it doesn't exist, or updates the existing one.
    ///
    /// # Arguments
    /// * `event` - The GenerationEvent to process
    pub async fn update_from_event(&self, event: &GenerationEvent) {
        let mut requests = self.active_requests.write().await;

        match requests.get_mut(&event.request_id) {
            Some(request) => {
                // Update existing request
                request.update_from_event(event);
            }
            None => {
                // Create new tracked request
                let tracked_request = TrackedRequest::from_event(event);
                requests.insert(event.request_id.clone(), tracked_request);
            }
        }
    }

    /// Get all active requests
    ///
    /// Returns a vector of all currently tracked requests.
    pub async fn get_active_requests(&self) -> Vec<TrackedRequest> {
        let requests = self.active_requests.read().await;
        requests.values().cloned().collect()
    }

    /// Get a specific tracked request by ID
    ///
    /// # Arguments
    /// * `request_id` - The request ID to look up
    ///
    /// # Returns
    /// Some(TrackedRequest) if found, None otherwise
    pub async fn get_request(&self, request_id: &str) -> Option<TrackedRequest> {
        let requests = self.active_requests.read().await;
        requests.get(request_id).cloned()
    }

    /// Remove a specific request from tracking
    ///
    /// # Arguments
    /// * `request_id` - The request ID to remove
    pub async fn remove_request(&self, request_id: &str) {
        let mut requests = self.active_requests.write().await;
        requests.remove(request_id);
    }

    /// Clean up old requests that haven't been updated within the timeout period
    ///
    /// This should be called periodically (e.g., every 5 minutes) to prevent
    /// the tracker from growing unbounded.
    ///
    /// Returns the number of requests that were removed.
    pub async fn cleanup_old_requests(&self) -> usize {
        let mut requests = self.active_requests.write().await;
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.cleanup_timeout_secs as i64);

        let initial_count = requests.len();

        requests.retain(|request_id, request| {
            let age = now.signed_duration_since(request.last_update);
            let should_keep = age < timeout;

            if !should_keep {
                eprintln!(
                    "[RequestTracker] Removing stale request {} (age: {} seconds)",
                    request_id,
                    age.num_seconds()
                );
            }

            should_keep
        });

        let removed_count = initial_count - requests.len();

        if removed_count > 0 {
            eprintln!(
                "[RequestTracker] Cleanup completed: removed {} stale requests, {} active remaining",
                removed_count,
                requests.len()
            );
        }

        removed_count
    }

    /// Get the number of currently tracked requests
    pub async fn count(&self) -> usize {
        let requests = self.active_requests.read().await;
        requests.len()
    }

    /// Clear all tracked requests (useful for testing)
    pub async fn clear(&self) {
        let mut requests = self.active_requests.write().await;
        requests.clear();
    }
}

impl Clone for RequestTracker {
    fn clone(&self) -> Self {
        Self {
            active_requests: Arc::clone(&self.active_requests),
            cleanup_timeout_secs: self.cleanup_timeout_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_tracker_new() {
        let tracker = RequestTracker::new(3600);
        assert_eq!(tracker.get_active_requests().await.len(), 0);
    }

    #[tokio::test]
    async fn test_track_request() {
        let tracker = RequestTracker::new(3600);
        let request = TrackedRequest {
            request_id: "test-123".to_string(),
            tenant_id: "tenant-1".to_string(),
            start_time: Utc::now(),
            current_phase: "started".to_string(),
            progress: 0.0,
            last_update: Utc::now(),
            component: "orchestrator".to_string(),
            status: "in_progress".to_string(),
            error_message: None,
            file_path: None,
        };

        tracker.track_request(request).await;
        assert_eq!(tracker.get_active_requests().await.len(), 1);
        assert_eq!(tracker.count().await, 1);
    }

    #[tokio::test]
    async fn test_update_request() {
        let tracker = RequestTracker::new(3600);
        let request = TrackedRequest {
            request_id: "test-123".to_string(),
            tenant_id: "tenant-1".to_string(),
            start_time: Utc::now(),
            current_phase: "started".to_string(),
            progress: 0.0,
            last_update: Utc::now(),
            component: "orchestrator".to_string(),
            status: "in_progress".to_string(),
            error_message: None,
            file_path: None,
        };

        tracker.track_request(request).await;

        tracker
            .update_request(
                "test-123",
                "story-generator".to_string(),
                0.5,
                "story-generator".to_string(),
                "in_progress".to_string(),
            )
            .await;

        let requests = tracker.get_active_requests().await;
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].current_phase, "story-generator");
        assert_eq!(requests[0].progress, 0.5);
    }

    #[tokio::test]
    async fn test_get_request() {
        let tracker = RequestTracker::new(3600);
        let request = TrackedRequest {
            request_id: "test-123".to_string(),
            tenant_id: "tenant-1".to_string(),
            start_time: Utc::now(),
            current_phase: "started".to_string(),
            progress: 0.0,
            last_update: Utc::now(),
            component: "orchestrator".to_string(),
            status: "in_progress".to_string(),
            error_message: None,
            file_path: None,
        };

        tracker.track_request(request).await;

        let retrieved = tracker.get_request("test-123").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().request_id, "test-123");

        let not_found = tracker.get_request("nonexistent").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_remove_request() {
        let tracker = RequestTracker::new(3600);
        let request = TrackedRequest {
            request_id: "test-123".to_string(),
            tenant_id: "tenant-1".to_string(),
            start_time: Utc::now(),
            current_phase: "started".to_string(),
            progress: 0.0,
            last_update: Utc::now(),
            component: "orchestrator".to_string(),
            status: "in_progress".to_string(),
            error_message: None,
            file_path: None,
        };

        tracker.track_request(request).await;
        assert_eq!(tracker.count().await, 1);

        tracker.remove_request("test-123").await;
        assert_eq!(tracker.count().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_old_requests() {
        let tracker = RequestTracker::new(1); // 1 second timeout

        // Create an old request
        let old_request = TrackedRequest {
            request_id: "old-request".to_string(),
            tenant_id: "tenant-1".to_string(),
            start_time: Utc::now() - chrono::Duration::seconds(5),
            current_phase: "started".to_string(),
            progress: 0.0,
            last_update: Utc::now() - chrono::Duration::seconds(5),
            component: "orchestrator".to_string(),
            status: "in_progress".to_string(),
            error_message: None,
            file_path: None,
        };

        tracker.track_request(old_request).await;

        // Wait for timeout
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let removed_count = tracker.cleanup_old_requests().await;
        assert_eq!(removed_count, 1);
        assert_eq!(tracker.count().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_preserves_recent_requests() {
        let tracker = RequestTracker::new(10); // 10 second timeout

        let recent_request = TrackedRequest {
            request_id: "recent-request".to_string(),
            tenant_id: "tenant-1".to_string(),
            start_time: Utc::now(),
            current_phase: "started".to_string(),
            progress: 0.0,
            last_update: Utc::now(),
            component: "orchestrator".to_string(),
            status: "in_progress".to_string(),
            error_message: None,
            file_path: None,
        };

        tracker.track_request(recent_request).await;

        let removed_count = tracker.cleanup_old_requests().await;
        assert_eq!(removed_count, 0);
        assert_eq!(tracker.count().await, 1);
    }

    #[tokio::test]
    async fn test_update_from_event() {
        let tracker = RequestTracker::new(3600);

        let event = GenerationEvent::new(
            "generation_started".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "in_progress".to_string(),
        )
        .with_progress(0.3);

        // First event creates the tracked request
        tracker.update_from_event(&event).await;
        assert_eq!(tracker.count().await, 1);

        let tracked = tracker.get_request("req-456").await.unwrap();
        assert_eq!(tracked.progress, 0.3);
        assert_eq!(tracked.status, "in_progress");

        // Second event updates the existing request
        let event2 = GenerationEvent::new(
            "generation_progress".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "quality-control".to_string(),
            "in_progress".to_string(),
        )
        .with_progress(0.7);

        tracker.update_from_event(&event2).await;
        assert_eq!(tracker.count().await, 1); // Still only one request

        let tracked2 = tracker.get_request("req-456").await.unwrap();
        assert_eq!(tracked2.progress, 0.7);
        assert_eq!(tracked2.current_phase, "quality-control");
    }

    #[tokio::test]
    async fn test_clear() {
        let tracker = RequestTracker::new(3600);

        for i in 0..5 {
            let request = TrackedRequest {
                request_id: format!("test-{}", i),
                tenant_id: "tenant-1".to_string(),
                start_time: Utc::now(),
                current_phase: "started".to_string(),
                progress: 0.0,
                last_update: Utc::now(),
                component: "orchestrator".to_string(),
                status: "in_progress".to_string(),
                error_message: None,
                file_path: None,
            };
            tracker.track_request(request).await;
        }

        assert_eq!(tracker.count().await, 5);

        tracker.clear().await;
        assert_eq!(tracker.count().await, 0);
    }

    #[test]
    fn test_tracked_request_from_event() {
        let event = GenerationEvent::new(
            "generation_started".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "in_progress".to_string(),
        )
        .with_progress(0.5)
        .with_error("Test error".to_string());

        let tracked = TrackedRequest::from_event(&event);

        assert_eq!(tracked.request_id, "req-456");
        assert_eq!(tracked.tenant_id, "tenant-123");
        assert_eq!(tracked.current_phase, "story-generator");
        assert_eq!(tracked.progress, 0.5);
        assert_eq!(tracked.status, "in_progress");
        assert_eq!(tracked.error_message, Some("Test error".to_string()));
    }
}
