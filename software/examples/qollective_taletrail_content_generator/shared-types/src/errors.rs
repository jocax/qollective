//! TaleTrail Error Types

use thiserror::Error;

/// TaleTrail error types
#[derive(Error, Debug)]
pub enum TaleTrailError {
    /// Network-related errors
    #[error("Network error: {0}")]
    NetworkError(String),

    /// NATS connection errors
    #[error("NATS error: {0}")]
    NatsError(String),

    /// NATS TLS configuration errors
    #[error("NATS TLS error: {0}")]
    NatsTlsError(String),

    /// NATS NKey authentication errors
    #[error("NATS NKey error: {0}")]
    NatsNKeyError(String),

    /// Qollective framework errors
    #[error("Qollective error: {0}")]
    QollectiveError(String),

    /// TLS certificate errors
    #[error("TLS certificate error: {0}")]
    TlsCertificateError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Content generation errors
    #[error("Generation error: {0}")]
    GenerationError(String),

    /// LLM-related errors
    #[error("LLM error: {0}")]
    LLMError(String),

    /// Timeout errors
    #[error("Operation timed out")]
    TimeoutError,

    /// Retry exhausted errors
    #[error("Retry attempts exhausted")]
    RetryExhausted,

    /// Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid request errors
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Convenience type alias for TaleTrail Results
pub type Result<T> = std::result::Result<T, TaleTrailError>;
