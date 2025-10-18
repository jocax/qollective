//! TaleTrail Error Types
//!
//! This module defines all error types used throughout the TaleTrail content generator.
//! Each error type includes descriptive messages and context for debugging.
//!
//! # Examples
//!
//! ```
//! use shared_types::errors::{TaleTrailError, Result};
//!
//! fn validate_input(value: i32) -> Result<i32> {
//!     if value < 0 {
//!         return Err(TaleTrailError::ValidationError(
//!             "Value must be non-negative".to_string()
//!         ));
//!     }
//!     Ok(value)
//! }
//!
//! let result = validate_input(42);
//! assert!(result.is_ok());
//! ```

use thiserror::Error;

/// TaleTrail error types with comprehensive context and debugging information
#[derive(Error, Debug, Clone)]
pub enum TaleTrailError {
    /// Network-related errors (TCP/HTTP connection failures)
    ///
    /// # Debugging
    /// - Check network connectivity
    /// - Verify firewall rules
    /// - Ensure target service is running
    #[error("Network error: {0}")]
    NetworkError(String),

    /// NATS connection and subscription errors
    ///
    /// # Debugging
    /// - Verify NATS server is running (check `docker ps` or `systemctl status nats`)
    /// - Check NATS_URL environment variable or config.toml
    /// - Ensure port 5222 is accessible (TLS) or 4222 (non-TLS)
    /// - Check NATS server logs for connection attempts
    #[error("NATS error: {0}")]
    NatsError(String),

    /// NATS TLS configuration errors (certificate validation failures)
    ///
    /// # Debugging
    /// - Verify TLS certificates exist in certs/ directory
    /// - Check certificate validity: `openssl x509 -in certs/client-cert.pem -noout -dates`
    /// - Ensure CA certificate matches server certificate
    /// - Verify file permissions (should be readable: chmod 644 *.pem)
    #[error("NATS TLS error: {0}")]
    NatsTlsError(String),

    /// NATS NKey authentication errors
    ///
    /// # Debugging
    /// - Verify NKey file exists and is readable
    /// - Check NKey format (should start with 'SU' for user seeds)
    /// - Ensure NATS server is configured for NKey authentication
    #[error("NATS NKey error: {0}")]
    NatsNKeyError(String),

    /// Qollective framework errors (envelope validation, metadata issues)
    ///
    /// # Debugging
    /// - Check envelope structure matches JSON schema
    /// - Verify tenant_id is present in metadata
    /// - Ensure payload type is correctly discriminated
    #[error("Qollective error: {0}")]
    QollectiveError(String),

    /// TLS certificate errors (parsing, validation, expiration)
    ///
    /// # Debugging
    /// - Regenerate certificates: `cd certs && ./setup-tls.sh`
    /// - Verify certificate chain is complete
    /// - Check certificate expiration dates
    #[error("TLS certificate error: {0}")]
    TlsCertificateError(String),

    /// Configuration errors (missing fields, invalid values)
    ///
    /// # Debugging
    /// - Check config.toml syntax
    /// - Verify all required fields are present
    /// - Check environment variables override config.toml correctly
    /// - Review example configuration in .env.example
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation errors (invalid input, constraint violations)
    ///
    /// # Debugging
    /// - Review validation rules in constraint-enforcer config
    /// - Check input matches expected schema
    /// - Verify enum values are correct (e.g., age_group, language)
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Content generation errors (DAG creation, LLM failures)
    ///
    /// # Debugging
    /// - Check LM Studio is running and accessible
    /// - Verify model is loaded in LM Studio
    /// - Review generation parameters (node_count, vocabulary_level)
    /// - Check orchestrator logs for detailed error context
    #[error("Generation error: {0}")]
    GenerationError(String),

    /// LLM-related errors (model not found, API failures, token limits)
    ///
    /// # Debugging
    /// - Verify LM_STUDIO_URL is correct (default: http://127.0.0.1:1234)
    /// - Check model is loaded in LM Studio UI
    /// - Review token limits and reduce node_count if needed
    /// - Check LM Studio logs for errors
    #[error("LLM error: {0}")]
    LLMError(String),

    /// Timeout errors (operation exceeded time limit)
    ///
    /// # Debugging
    /// - Increase timeout values in constants.rs
    /// - Check if dependent services are responsive
    /// - Review system resource usage (CPU, memory)
    /// - Consider reducing batch size or node count
    #[error("Operation timed out (check timeout constants in shared-types/src/constants.rs)")]
    TimeoutError,

    /// Retry exhausted errors (max retry attempts reached)
    ///
    /// # Debugging
    /// - Review underlying cause of failures
    /// - Increase RETRY_MAX_ATTEMPTS if transient failures are expected
    /// - Check exponential backoff configuration
    /// - Investigate root cause rather than increasing retries
    #[error("Retry attempts exhausted (check RETRY_MAX_ATTEMPTS in shared-types/src/constants.rs)")]
    RetryExhausted,

    /// Serialization errors (JSON parsing, envelope serialization)
    ///
    /// # Debugging
    /// - Verify JSON structure matches schema
    /// - Check for invalid UTF-8 characters
    /// - Review serde derive attributes on types
    /// - Validate enum serialization format
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid request errors (missing required fields, malformed data)
    ///
    /// # Debugging
    /// - Review API documentation for required fields
    /// - Check request matches ExternalGenerationRequestV1 schema
    /// - Verify age_group and language values are valid
    /// - Ensure tenant_id is present in JWT
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Pipeline orchestration errors (phase transitions, state management)
    ///
    /// # Debugging
    /// - Check pipeline phase progression in orchestrator logs
    /// - Verify all phases complete before advancing
    /// - Review state machine logic for invalid transitions
    #[error("Pipeline error: {0}")]
    PipelineError(String),

    /// Service discovery errors (missing services, tool discovery failures)
    ///
    /// # Debugging
    /// - Ensure all required MCP servers are running
    /// - Check discovery subjects are correct (mcp.discovery.list_tools.*)
    /// - Verify services are subscribed to their discovery endpoints
    /// - Review service logs for discovery handler errors
    #[error("Discovery error: {0}")]
    DiscoveryError(String),
}

impl TaleTrailError {
    /// Get a user-friendly suggestion for resolving this error
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::NatsError(_) => Some("Check if NATS server is running: docker ps | grep nats"),
            Self::NatsTlsError(_) => Some("Regenerate TLS certificates: cd certs && ./setup-tls.sh"),
            Self::ConfigError(_) => Some("Review config.toml and .env.example for required fields"),
            Self::TimeoutError => Some("Increase timeout values or reduce operation complexity"),
            Self::RetryExhausted => Some("Check logs for underlying error cause"),
            Self::LLMError(_) => Some("Verify LM Studio is running and model is loaded"),
            Self::DiscoveryError(_) => Some("Ensure all MCP services are running and check discovery subscriptions"),
            _ => None,
        }
    }

    /// Get the error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            Self::NetworkError(_) | Self::NatsError(_) | Self::NatsTlsError(_) | Self::NatsNKeyError(_) => "network",
            Self::ConfigError(_) => "configuration",
            Self::ValidationError(_) | Self::InvalidRequest(_) => "validation",
            Self::GenerationError(_) | Self::LLMError(_) => "generation",
            Self::QollectiveError(_) | Self::PipelineError(_) | Self::DiscoveryError(_) => "framework",
            Self::TlsCertificateError(_) => "security",
            Self::TimeoutError | Self::RetryExhausted => "reliability",
            Self::SerializationError(_) => "serialization",
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_) | Self::NatsError(_) | Self::TimeoutError | Self::LLMError(_)
        )
    }
}

/// Convenience type alias for TaleTrail Results
///
/// # Examples
///
/// ```
/// use shared_types::errors::{TaleTrailError, Result};
///
/// fn process_data(input: &str) -> Result<String> {
///     if input.is_empty() {
///         return Err(TaleTrailError::ValidationError("Input cannot be empty".to_string()));
///     }
///     Ok(input.to_uppercase())
/// }
/// ```
pub type Result<T> = std::result::Result<T, TaleTrailError>;
