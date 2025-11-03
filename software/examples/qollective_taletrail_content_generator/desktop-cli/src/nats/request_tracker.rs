/// Request tracker module for monitoring active generation requests
///
/// Provides thread-safe tracking of active requests with automatic cleanup
/// of stale requests based on configurable timeout.
///
/// Adapted for use with smol async executor (tokio::sync::RwLock -> futures::lock::Mutex)

use crate::models::events::GenerationEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use futures::lock::Mutex;

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
/// Uses futures::lock::Mutex for smol compatibility
pub struct RequestTracker {
    /// Map of request_id -> TrackedRequest
    active_requests: Arc<Mutex<HashMap<String, TrackedRequest>>>,
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
            active_requests: Arc::new(Mutex::new(HashMap::new())),
            cleanup_timeout_secs,
        }
    }

    /// Track a new request or update an existing one
    ///
    /// # Arguments
    /// * `request` - The TrackedRequest to track
    pub async fn track_request(&self, request: TrackedRequest) {
        let mut requests = self.active_requests.lock().await;
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
        let mut requests = self.active_requests.lock().await;
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
        let mut requests = self.active_requests.lock().await;

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
        let requests = self.active_requests.lock().await;
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
        let requests = self.active_requests.lock().await;
        requests.get(request_id).cloned()
    }

    /// Remove a specific request from tracking
    ///
    /// # Arguments
    /// * `request_id` - The request ID to remove
    pub async fn remove_request(&self, request_id: &str) {
        let mut requests = self.active_requests.lock().await;
        requests.remove(request_id);
    }

    /// Clean up old requests that haven't been updated within the timeout period
    ///
    /// This should be called periodically (e.g., every 5 minutes) to prevent
    /// the tracker from growing unbounded.
    ///
    /// Returns the number of requests that were removed.
    pub async fn cleanup_old_requests(&self) -> usize {
        let mut requests = self.active_requests.lock().await;
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
        let requests = self.active_requests.lock().await;
        requests.len()
    }

    /// Clear all tracked requests (useful for testing)
    pub async fn clear(&self) {
        let mut requests = self.active_requests.lock().await;
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
