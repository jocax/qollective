use thiserror::Error;

/// Application error types for TaleTrail Desktop Viewer
#[derive(Debug, Error)]
pub enum AppError {
    /// I/O operation failed
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Validation failed
    #[error("Validation Error: {0}")]
    ValidationError(String),

    /// Resource not found
    #[error("Not Found: {0}")]
    NotFound(String),

    /// NATS client error
    #[error("NATS Error: {0}")]
    NatsError(String),

    /// Connection error
    #[error("Connection Error: {0}")]
    ConnectionError(String),

    /// Service error
    #[error("Service Error: {0}")]
    ServiceError(String),
}

/// Type alias for Result with AppError
pub type AppResult<T> = Result<T, AppError>;
