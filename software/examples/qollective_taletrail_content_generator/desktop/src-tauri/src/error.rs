use std::fmt;

/// Application error types for TaleTrail Desktop Viewer
#[derive(Debug)]
pub enum AppError {
    /// I/O operation failed
    IoError(std::io::Error),
    /// JSON serialization/deserialization failed
    JsonError(serde_json::Error),
    /// Validation failed
    ValidationError(String),
    /// Resource not found
    NotFound(String),
    /// NATS client error
    NatsError(String),
    /// Connection error
    ConnectionError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(err) => write!(f, "IO Error: {}", err),
            AppError::JsonError(err) => write!(f, "JSON Error: {}", err),
            AppError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::NatsError(msg) => write!(f, "NATS Error: {}", msg),
            AppError::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::IoError(err) => Some(err),
            AppError::JsonError(err) => Some(err),
            AppError::ValidationError(_) => None,
            AppError::NotFound(_) => None,
            AppError::NatsError(_) => None,
            AppError::ConnectionError(_) => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonError(err)
    }
}

/// Type alias for Result with AppError
pub type AppResult<T> = Result<T, AppError>;
