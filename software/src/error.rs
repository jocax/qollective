// ABOUTME: Error types and handling for the Qollective framework
// ABOUTME: Provides comprehensive error handling with domain-specific error variants

//! Error types and utilities for the Qollective framework.
//!
//! This module provides a comprehensive error type that covers all possible
//! error scenarios within the framework, from parsing and validation errors
//! to network and serialization failures.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Result type alias for Qollective operations
pub type Result<T> = std::result::Result<T, QollectiveError>;

/// Comprehensive error type for all Qollective operations
#[derive(Debug, Error, Clone)]
pub enum QollectiveError {
    /// Envelope parsing or validation errors
    #[error("envelope error: {0}")]
    Envelope(String),

    /// Configuration errors
    #[error("configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization errors
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Network/transport errors
    #[error("transport error: {0}")]
    Transport(String),

    /// Validation errors
    #[error("validation error: {0}")]
    Validation(String),

    /// Authentication/authorization errors
    #[error("security error: {0}")]
    Security(String),

    /// Internal framework errors
    #[error("internal error: {0}")]
    Internal(String),

    /// External service integration errors
    #[error("external service error: {0}")]
    External(String),

    /// Connection errors
    #[error("connection error: {0}")]
    Connection(String),

    /// Deserialization errors
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Remote service errors
    #[error("remote service error: {0}")]
    Remote(String),

    /// gRPC-specific errors
    #[error("gRPC error: {0}")]
    Grpc(String),

    /// Tenant extraction errors
    #[error("tenant extraction error: {0}")]
    TenantExtraction(String),

    /// NATS connection errors
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[error("NATS connection error: {0}")]
    NatsConnection(String),

    /// NATS message errors
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[error("NATS message error: {0}")]
    NatsMessage(String),

    /// NATS timeout errors
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[error("NATS timeout error: {0}")]
    NatsTimeout(String),

    /// NATS discovery errors
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[error("NATS discovery error: {0}")]
    NatsDiscovery(String),

    /// NATS subject errors
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[error("NATS subject error: {0}")]
    NatsSubject(String),

    /// NATS authentication errors
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[error("NATS authentication error: {0}")]
    NatsAuth(String),

    /// Feature not enabled errors
    #[error("feature not enabled: {0}")]
    FeatureNotEnabled(String),

    /// MCP protocol errors
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    /// rmcp crate errors
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[error("MCP Error: {0}")]
    McpError(#[from] rmcp::ErrorData),

    /// MCP tool execution errors
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[error("MCP tool execution error: {0}")]
    McpToolExecution(String),

    /// MCP server registration errors
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[error("MCP server registration error: {0}")]
    McpServerRegistration(String),

    /// MCP client connection errors
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[error("MCP client connection error: {0}")]
    McpClientConnection(String),

    /// MCP server not found errors
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[error("MCP server not found: {0}")]
    McpServerNotFound(String),

    /// Agent not found errors
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    /// Protocol adapter errors
    #[error("protocol adapter error: {0}")]
    ProtocolAdapter(String),
}

impl QollectiveError {
    /// Create a new envelope error
    pub fn envelope(msg: impl Into<String>) -> Self {
        Self::Envelope(msg.into())
    }

    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a new transport error
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::Transport(msg.into())
    }

    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new security error
    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(msg.into())
    }

    /// Create a new internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a new external service error
    pub fn external(msg: impl Into<String>) -> Self {
        Self::External(msg.into())
    }

    /// Create a new connection error
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a new deserialization error
    pub fn deserialization(msg: impl Into<String>) -> Self {
        Self::Deserialization(msg.into())
    }

    /// Create a new remote service error
    pub fn remote(msg: impl Into<String>) -> Self {
        Self::Remote(msg.into())
    }

    /// Create a new gRPC error
    pub fn grpc(msg: impl Into<String>) -> Self {
        Self::Grpc(msg.into())
    }

    /// Create a new tenant extraction error
    pub fn tenant_extraction(msg: impl Into<String>) -> Self {
        Self::TenantExtraction(msg.into())
    }

    /// Create a new NATS connection error
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_connection(msg: impl Into<String>) -> Self {
        Self::NatsConnection(msg.into())
    }

    /// Create a new NATS message error
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_message(msg: impl Into<String>) -> Self {
        Self::NatsMessage(msg.into())
    }

    /// Create a new NATS timeout error
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_timeout(msg: impl Into<String>) -> Self {
        Self::NatsTimeout(msg.into())
    }

    /// Create a new NATS discovery error
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_discovery(msg: impl Into<String>) -> Self {
        Self::NatsDiscovery(msg.into())
    }

    /// Create a new NATS subject error
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_subject(msg: impl Into<String>) -> Self {
        Self::NatsSubject(msg.into())
    }

    /// Create a new NATS authentication error
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_auth(msg: impl Into<String>) -> Self {
        Self::NatsAuth(msg.into())
    }

    /// Create a new feature not enabled error
    pub fn feature_not_enabled(msg: impl Into<String>) -> Self {
        Self::FeatureNotEnabled(msg.into())
    }

    /// Create a new MCP protocol error
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn mcp_protocol(msg: impl Into<String>) -> Self {
        Self::McpProtocol(msg.into())
    }

    /// Create a new MCP tool execution error
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn mcp_tool_execution(msg: impl Into<String>) -> Self {
        Self::McpToolExecution(msg.into())
    }

    /// Create a new MCP server registration error
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn mcp_server_registration(msg: impl Into<String>) -> Self {
        Self::McpServerRegistration(msg.into())
    }

    /// Create a new MCP client connection error
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn mcp_client_connection(msg: impl Into<String>) -> Self {
        Self::McpClientConnection(msg.into())
    }

    /// Create a new MCP server not found error
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn mcp_server_not_found(msg: impl Into<String>) -> Self {
        Self::McpServerNotFound(msg.into())
    }

    /// Create a new rmcp error (rarely needed - automatic conversion via From trait)
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn rmcp_error(err: rmcp::ErrorData) -> Self {
        Self::McpError(err)
    }

    /// Create a new agent not found error
    pub fn agent_not_found(msg: impl Into<String>) -> Self {
        Self::AgentNotFound(msg.into())
    }

    /// Create a new protocol adapter error
    pub fn protocol_adapter(msg: impl Into<String>) -> Self {
        Self::ProtocolAdapter(msg.into())
    }

    /// Create a new TLS error (using Transport category)
    pub fn tls(msg: impl Into<String>) -> Self {
        Self::Transport(format!("TLS error: {}", msg.into()))
    }

    // =============================================================================
    // CONVENIENCE CONSTRUCTORS FOR ENVELOPE ERROR WITH HTTP STATUS CODES
    // =============================================================================

    /// Create a validation error EnvelopeError with HTTP 400 status code
    ///
    /// This convenience method creates a structured EnvelopeError specifically
    /// for validation failures, automatically setting the appropriate HTTP status
    /// code and error code pattern.
    pub fn validation_error(
        message: impl Into<String>,
        details: Option<serde_json::Value>
    ) -> crate::envelope::EnvelopeError {
        crate::envelope::EnvelopeError {
            code: "VALIDATION_FAILED".to_string(),
            message: message.into(),
            details,
            trace: None,
            #[cfg(any(
                feature = "rest-server",
                feature = "rest-client",
                feature = "websocket-server",
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(400),
        }
    }

    /// Create an authentication error EnvelopeError with HTTP 401 status code
    ///
    /// This convenience method creates a structured EnvelopeError for authentication
    /// failures, such as invalid tokens, expired credentials, or missing auth headers.
    pub fn auth_error(
        message: impl Into<String>,
        details: Option<serde_json::Value>
    ) -> crate::envelope::EnvelopeError {
        crate::envelope::EnvelopeError {
            code: "AUTHENTICATION_FAILED".to_string(),
            message: message.into(),
            details,
            trace: None,
            #[cfg(any(
                feature = "rest-server",
                feature = "rest-client",
                feature = "websocket-server",
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(401),
        }
    }

    /// Create a not found error EnvelopeError with HTTP 404 status code
    ///
    /// This convenience method creates a structured EnvelopeError for resources
    /// that cannot be found, such as missing users, non-existent endpoints, or
    /// unavailable services.
    pub fn not_found_error(
        message: impl Into<String>,
        details: Option<serde_json::Value>
    ) -> crate::envelope::EnvelopeError {
        crate::envelope::EnvelopeError {
            code: "RESOURCE_NOT_FOUND".to_string(),
            message: message.into(),
            details,
            trace: None,
            #[cfg(any(
                feature = "rest-server",
                feature = "rest-client",
                feature = "websocket-server",
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(404),
        }
    }

    /// Create a server error EnvelopeError with HTTP 500 status code
    ///
    /// This convenience method creates a structured EnvelopeError for internal
    /// server failures, such as database errors, service unavailability, or
    /// unexpected system failures.
    pub fn server_error(
        message: impl Into<String>,
        details: Option<serde_json::Value>
    ) -> crate::envelope::EnvelopeError {
        crate::envelope::EnvelopeError {
            code: "INTERNAL_SERVER_ERROR".to_string(),
            message: message.into(),
            details,
            trace: None,
            #[cfg(any(
                feature = "rest-server",
                feature = "rest-client",
                feature = "websocket-server",
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(500),
        }
    }

    /// Create a custom EnvelopeError with specified HTTP status code and validation
    ///
    /// This method allows creating EnvelopeError instances with arbitrary status codes
    /// while validating that the status code is in the valid error range (400-599).
    /// Invalid status codes are automatically normalized to 500.
    ///
    /// # Arguments
    ///
    /// * `code` - Error code for programmatic handling (e.g., "RATE_LIMIT_EXCEEDED")
    /// * `message` - Human-readable error message
    /// * `details` - Optional structured error details as JSON
    /// * `status_code` - HTTP status code (will be validated and normalized if invalid)
    pub fn custom_error(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
        status_code: u16
    ) -> crate::envelope::EnvelopeError {
        // Validate HTTP status code is in error range (400-599)
        let validated_status_code = if status_code >= 400 && status_code < 600 {
            status_code
        } else {
            // Log warning and normalize to 500 for invalid codes
            #[cfg(feature = "tracing")]
            tracing::warn!("Invalid HTTP status code {} provided, normalizing to 500", status_code);
            500
        };

        crate::envelope::EnvelopeError {
            code: code.into(),
            message: message.into(),
            details,
            trace: None,
            #[cfg(any(
                feature = "rest-server",
                feature = "rest-client",
                feature = "websocket-server",
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(validated_status_code),
        }
    }
}

// Conversions from common error types
impl From<serde_json::Error> for QollectiveError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(feature = "rest-client")]
impl From<reqwest::Error> for QollectiveError {
    fn from(err: reqwest::Error) -> Self {
        Self::Transport(err.to_string())
    }
}

#[cfg(feature = "validation")]
impl From<jsonschema::ValidationError<'_>> for QollectiveError {
    fn from(err: jsonschema::ValidationError) -> Self {
        Self::Validation(err.to_string())
    }
}

#[cfg(feature = "tenant-extraction")]
impl From<crate::tenant::ExtractionError> for QollectiveError {
    fn from(err: crate::tenant::ExtractionError) -> Self {
        Self::TenantExtraction(err.to_string())
    }
}

#[cfg(feature = "tenant-extraction")]
impl From<crate::tenant::JwtParseError> for QollectiveError {
    fn from(err: crate::tenant::JwtParseError) -> Self {
        Self::TenantExtraction(format!("JWT parsing failed: {}", err))
    }
}

// NATS error conversions
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl From<async_nats::ConnectError> for QollectiveError {
    fn from(err: async_nats::ConnectError) -> Self {
        Self::NatsConnection(format!("Failed to connect to NATS: {}", err))
    }
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl From<async_nats::PublishError> for QollectiveError {
    fn from(err: async_nats::PublishError) -> Self {
        Self::NatsMessage(format!("Failed to publish NATS message: {}", err))
    }
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl From<async_nats::SubscribeError> for QollectiveError {
    fn from(err: async_nats::SubscribeError) -> Self {
        Self::NatsMessage(format!("Failed to subscribe to NATS subject: {}", err))
    }
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl From<async_nats::RequestError> for QollectiveError {
    fn from(err: async_nats::RequestError) -> Self {
        // Handle different RequestError variants based on their kind
        let error_str = err.to_string();
        if error_str.contains("timed out") || error_str.contains("timeout") {
            Self::NatsTimeout(format!("NATS request timed out: {}", err))
        } else if error_str.contains("no responders") || error_str.contains("No responders") {
            Self::NatsDiscovery(format!("No NATS responders available: {}", err))
        } else {
            Self::NatsMessage(format!("NATS request failed: {}", err))
        }
    }
}

// Timeout error conversion for tokio operations
impl From<tokio::time::error::Elapsed> for QollectiveError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::Transport(format!("Operation timed out: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let err = QollectiveError::envelope("test error");
        assert!(matches!(err, QollectiveError::Envelope(_)));
    }

    #[test]
    fn test_error_conversion() {
        let result: Result<()> = Err(QollectiveError::config("test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_feature_not_enabled_error() {
        let err = QollectiveError::feature_not_enabled("NATS requires feature flag");
        assert!(matches!(err, QollectiveError::FeatureNotEnabled(_)));
        assert_eq!(
            err.to_string(),
            "feature not enabled: NATS requires feature flag"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_connection_error_construction() {
        let err = QollectiveError::nats_connection("Failed to connect to NATS server");
        assert!(matches!(err, QollectiveError::NatsConnection(_)));
        assert_eq!(
            err.to_string(),
            "NATS connection error: Failed to connect to NATS server"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_message_error_construction() {
        let err = QollectiveError::nats_message("Invalid message format");
        assert!(matches!(err, QollectiveError::NatsMessage(_)));
        assert_eq!(
            err.to_string(),
            "NATS message error: Invalid message format"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_timeout_error_construction() {
        let err = QollectiveError::nats_timeout("Request timed out after 30s");
        assert!(matches!(err, QollectiveError::NatsTimeout(_)));
        assert_eq!(
            err.to_string(),
            "NATS timeout error: Request timed out after 30s"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_discovery_error_construction() {
        let err = QollectiveError::nats_discovery("Agent not found in registry");
        assert!(matches!(err, QollectiveError::NatsDiscovery(_)));
        assert_eq!(
            err.to_string(),
            "NATS discovery error: Agent not found in registry"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_subject_error_construction() {
        let err = QollectiveError::nats_subject("Invalid subject pattern");
        assert!(matches!(err, QollectiveError::NatsSubject(_)));
        assert_eq!(
            err.to_string(),
            "NATS subject error: Invalid subject pattern"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_auth_error_construction() {
        let err = QollectiveError::nats_auth("Authentication failed with invalid credentials");
        assert!(matches!(err, QollectiveError::NatsAuth(_)));
        assert_eq!(
            err.to_string(),
            "NATS authentication error: Authentication failed with invalid credentials"
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_error_helper_methods() {
        // Test all NATS error creation methods
        let connection_err = QollectiveError::nats_connection("connection failed");
        let message_err = QollectiveError::nats_message("message failed");
        let timeout_err = QollectiveError::nats_timeout("timeout occurred");
        let discovery_err = QollectiveError::nats_discovery("discovery failed");
        let subject_err = QollectiveError::nats_subject("invalid subject");
        let auth_err = QollectiveError::nats_auth("auth failed");

        // Verify error types
        assert!(matches!(connection_err, QollectiveError::NatsConnection(_)));
        assert!(matches!(message_err, QollectiveError::NatsMessage(_)));
        assert!(matches!(timeout_err, QollectiveError::NatsTimeout(_)));
        assert!(matches!(discovery_err, QollectiveError::NatsDiscovery(_)));
        assert!(matches!(subject_err, QollectiveError::NatsSubject(_)));
        assert!(matches!(auth_err, QollectiveError::NatsAuth(_)));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_error_string_conversion() {
        let _err = QollectiveError::nats_connection("test");
        let err_string = String::from("connection issue");
        let converted_err = QollectiveError::nats_connection(err_string);

        assert!(matches!(converted_err, QollectiveError::NatsConnection(_)));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_error_chaining() {
        // Test error context preservation
        let root_cause = "underlying network error";
        let err = QollectiveError::nats_connection(format!("Connection failed: {}", root_cause));

        assert!(err.to_string().contains(root_cause));
        assert!(err.to_string().contains("Connection failed"));
    }

    #[test]
    fn test_error_categorization() {
        // Test that errors can be categorized properly
        let connection_err = QollectiveError::connection("generic connection error");
        let nats_connection_err = QollectiveError::feature_not_enabled("NATS not enabled");

        // Both should be distinguishable
        assert!(matches!(connection_err, QollectiveError::Connection(_)));
        assert!(matches!(
            nats_connection_err,
            QollectiveError::FeatureNotEnabled(_)
        ));
    }

    #[test]
    fn test_result_type_compatibility() {
        // Test that NATS errors work with Result<T>
        fn test_function() -> Result<String> {
            Err(QollectiveError::feature_not_enabled("NATS disabled"))
        }

        let result = test_function();
        assert!(result.is_err());

        match result {
            Err(QollectiveError::FeatureNotEnabled(msg)) => {
                assert_eq!(msg, "NATS disabled");
            }
            _ => panic!("Expected FeatureNotEnabled error"),
        }
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_result_type_compatibility() {
        // Test that NATS errors work with Result<T>
        fn test_nats_function() -> Result<String> {
            Err(QollectiveError::nats_connection("Connection refused"))
        }

        let result = test_nats_function();
        assert!(result.is_err());

        match result {
            Err(QollectiveError::NatsConnection(msg)) => {
                assert_eq!(msg, "Connection refused");
            }
            _ => panic!("Expected NatsConnection error"),
        }
    }

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[test]
    fn test_rmcp_error_integration() {
        // Test that rmcp::ErrorData can be converted to QollectiveError
        let rmcp_error = rmcp::ErrorData::method_not_found::<rmcp::model::CallToolRequestMethod>();
        let qollective_error: QollectiveError = rmcp_error.into();

        match qollective_error {
            QollectiveError::McpError(_) => {
                // Success - error was properly converted
                assert!(true);
            }
            _ => panic!("Expected McpError variant"),
        }
    }

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[test]
    fn test_rmcp_error_helper_method() {
        let rmcp_error = rmcp::ErrorData::internal_error("test error", None);
        let qollective_error = QollectiveError::rmcp_error(rmcp_error);

        assert!(matches!(qollective_error, QollectiveError::McpError(_)));
        assert!(qollective_error.to_string().contains("test error"));
    }

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[test]
    fn test_rmcp_error_result_compatibility() {
        // Test that rmcp errors work with Result<T>
        fn test_rmcp_function() -> Result<String> {
            let rmcp_error = rmcp::ErrorData::invalid_params("invalid parameters", None);
            Err(rmcp_error.into())
        }

        let result = test_rmcp_function();
        assert!(result.is_err());

        match result {
            Err(QollectiveError::McpError(_)) => {
                // Success
                assert!(true);
            }
            _ => panic!("Expected McpError variant"),
        }
    }
}

/// Enhanced Qollective Error with comprehensive OpenAPI support
///
/// This structure provides enterprise-grade error handling with structured details,
/// timestamps, and comprehensive OpenAPI schema generation for REST API documentation.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(title = "Enhanced Qollective Error"))]
pub struct EnhancedQollectiveError {
    /// The type of error that occurred within the Qollective framework
    #[cfg_attr(feature = "openapi", schema(example = "Validation"))]
    pub error_type: String,

    /// Human-readable description of the error for developers and support teams
    #[cfg_attr(feature = "openapi", schema(example = "Envelope validation failed: required field 'tenant_id' is missing"))]
    pub message: String,

    /// ISO 8601 timestamp when the error occurred for tracking and debugging
    #[cfg_attr(feature = "openapi", schema(example = "2025-08-23T10:30:45.123Z"))]
    pub occurred_at: DateTime<Utc>,

    /// Structured error details with additional context and debugging information
    #[cfg_attr(feature = "openapi", schema(example = "null"))]
    pub details: Option<Value>,

    /// Optional context information for enhanced error tracking
    #[cfg_attr(feature = "openapi", schema(example = "null"))]
    pub context: Option<Value>,
}

impl EnhancedQollectiveError {
    /// Create a new enhanced error from a QollectiveError
    pub fn from_error(error: QollectiveError) -> Self {
        Self {
            error_type: Self::classify_error(&error),
            message: error.to_string(),
            occurred_at: Utc::now(),
            details: None,
            context: None,
        }
    }

    /// Create a new enhanced error with custom details
    pub fn with_details(error: QollectiveError, details: Value) -> Self {
        Self {
            error_type: Self::classify_error(&error),
            message: error.to_string(),
            occurred_at: Utc::now(),
            details: Some(details),
            context: None,
        }
    }

    /// Create a new enhanced error with context information
    pub fn with_context(error: QollectiveError, context: Value) -> Self {
        Self {
            error_type: Self::classify_error(&error),
            message: error.to_string(),
            occurred_at: Utc::now(),
            details: None,
            context: Some(context),
        }
    }

    /// Classify the error type for API documentation
    fn classify_error(error: &QollectiveError) -> String {
        match error {
            QollectiveError::Envelope(_) => "Envelope".to_string(),
            QollectiveError::Config(_) => "Configuration".to_string(),
            QollectiveError::Transport(_) => "Transport".to_string(),
            QollectiveError::Serialization(_) => "Serialization".to_string(),
            QollectiveError::Validation(_) => "Validation".to_string(),
            QollectiveError::Security(_) => "Security".to_string(),
            QollectiveError::TenantExtraction(_) => "TenantExtraction".to_string(),
            QollectiveError::FeatureNotEnabled(_) => "FeatureNotEnabled".to_string(),
            QollectiveError::AgentNotFound(_) => "AgentNotFound".to_string(),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsConnection(_) => "NatsConnection".to_string(),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsMessage(_) => "NatsMessage".to_string(),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsTimeout(_) => "NatsTimeout".to_string(),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsDiscovery(_) => "NatsDiscovery".to_string(),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsSubject(_) => "NatsSubject".to_string(),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsAuth(_) => "NatsAuth".to_string(),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpProtocol(_) => "McpProtocol".to_string(),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpError(_) => "McpError".to_string(),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpToolExecution(_) => "McpToolExecution".to_string(),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpServerRegistration(_) => "McpServerRegistration".to_string(),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpClientConnection(_) => "McpClientConnection".to_string(),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpServerNotFound(_) => "McpServerNotFound".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

impl From<QollectiveError> for EnhancedQollectiveError {
    fn from(error: QollectiveError) -> Self {
        Self::from_error(error)
    }
}
