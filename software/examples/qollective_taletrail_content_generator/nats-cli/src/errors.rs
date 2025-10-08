//! Error types for NATS CLI
//!
//! Structured error handling using thiserror

/// Result type alias for NATS CLI operations
pub type Result<T> = std::result::Result<T, NatsCliError>;

/// NATS CLI error types
#[derive(Debug, thiserror::Error)]
pub enum NatsCliError {
    /// NATS connection failed
    #[error("Failed to connect to NATS: {0}")]
    ConnectionError(String),

    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Invalid template format
    #[error("Invalid template format: {0}")]
    InvalidTemplate(String),

    /// Request timeout
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// NATS protocol error
    #[error("NATS error: {0}")]
    NatsError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Server returned error response
    #[error("Server error: {0}")]
    ServerError(String),

    /// Envelope validation error
    #[error("Envelope validation error: {0}")]
    EnvelopeValidationError(String),

    /// TLS error
    #[error("TLS error: {0}")]
    TlsError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

// Conversions from other error types

impl From<serde_json::Error> for NatsCliError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_eof() || err.is_syntax() {
            Self::DeserializationError(err.to_string())
        } else {
            Self::SerializationError(err.to_string())
        }
    }
}

impl From<async_nats::Error> for NatsCliError {
    fn from(err: async_nats::Error) -> Self {
        Self::NatsError(err.to_string())
    }
}

impl From<figment::Error> for NatsCliError {
    fn from(err: figment::Error) -> Self {
        Self::ConfigError(err.to_string())
    }
}

impl From<shared_types::TaleTrailError> for NatsCliError {
    fn from(err: shared_types::TaleTrailError) -> Self {
        match err {
            shared_types::TaleTrailError::ConfigError(msg) => Self::ConfigError(msg),
            shared_types::TaleTrailError::NetworkError(msg) => Self::NatsError(msg),
            shared_types::TaleTrailError::SerializationError(msg) => {
                Self::SerializationError(msg)
            }
            shared_types::TaleTrailError::ValidationError(msg) => {
                Self::EnvelopeValidationError(msg)
            }
            _ => Self::NatsError(err.to_string()),
        }
    }
}

impl From<std::io::Error> for NatsCliError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}
