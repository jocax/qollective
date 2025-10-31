use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::{GenerationRequest, GenerationResponse, TrailListItem};

/// Service for handling generation request submissions
#[async_trait]
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
pub trait RequestService: Send + Sync {
    /// Submit a new generation request to the orchestrator
    ///
    /// # Arguments
    /// * `request` - The generation request to submit
    ///
    /// # Returns
    /// Returns the request_id for tracking if successful
    async fn submit_request(&self, request: GenerationRequest) -> AppResult<String>;

    /// Replay an existing generation request with a new request_id
    ///
    /// # Arguments
    /// * `original_request` - The request to replay
    /// * `new_request_id` - New unique request ID
    ///
    /// # Returns
    /// Returns the new request_id for tracking if successful
    async fn replay_request(
        &self,
        original_request: GenerationRequest,
        new_request_id: String,
    ) -> AppResult<String>;
}

/// Service for NATS messaging operations
#[async_trait]
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
pub trait NatsService: Send + Sync {
    /// Connect to NATS server
    async fn connect(&self) -> AppResult<()>;

    /// Subscribe to generation events for a specific tenant
    ///
    /// # Arguments
    /// * `tenant_id` - Optional tenant ID to filter events
    async fn subscribe(&self, tenant_id: Option<String>) -> AppResult<()>;

    /// Publish a generation request to NATS
    ///
    /// # Arguments
    /// * `request` - The generation request to publish
    async fn publish_request(&self, request: &GenerationRequest) -> AppResult<()>;

    /// Check if connected to NATS server
    async fn is_connected(&self) -> bool;

    /// Disconnect from NATS server
    async fn disconnect(&self) -> AppResult<()>;

    /// Unsubscribe from generation events
    async fn unsubscribe(&self) -> AppResult<()>;
}

/// Service for trail storage operations (loading and managing trail files)
#[async_trait]
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
pub trait TrailStorageService: Send + Sync {
    /// Load a full trail from a file
    ///
    /// # Arguments
    /// * `file_path` - Path to the trail JSON file
    ///
    /// # Returns
    /// The complete GenerationResponse
    async fn load_trail(&self, file_path: &str) -> AppResult<GenerationResponse>;

    /// Load trail metadata from a directory
    ///
    /// # Arguments
    /// * `directory` - Directory to scan for trail files
    ///
    /// # Returns
    /// List of trail metadata items
    async fn load_trails_from_directory(
        &self,
        directory: &str,
    ) -> AppResult<Vec<TrailListItem>>;

    /// Delete a trail file
    ///
    /// # Arguments
    /// * `file_path` - Path to the trail file to delete
    async fn delete_trail(&self, file_path: &str) -> AppResult<()>;
}
