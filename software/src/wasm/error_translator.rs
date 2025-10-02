// ABOUTME: Error translation layer for user-friendly JavaScript error messages
// ABOUTME: Converts QollectiveError types to user-friendly messages with retry policies

//! Error translation layer for WASM JavaScript interop.
//!
//! This module provides user-friendly error messages and retry policy
//! recommendations for different types of errors that occur in the system.

use crate::error::QollectiveError;
use crate::wasm::js_types::WasmEnvelopeError;
use wasm_bindgen::prelude::*;

/// Error translator for converting Rust errors to user-friendly messages
pub struct ErrorTranslator;

impl ErrorTranslator {
    /// Translate QollectiveError to user-friendly message
    pub fn translate_for_user(error: &QollectiveError) -> String {
        match error {
            QollectiveError::Envelope(msg) => {
                format!(
                    "Invalid data format: {}",
                    Self::sanitize_technical_message(msg)
                )
            }
            QollectiveError::Config(msg) => {
                format!(
                    "Configuration issue: {}",
                    Self::sanitize_technical_message(msg)
                )
            }
            QollectiveError::Serialization(msg) => {
                if msg.contains("expected") || msg.contains("invalid type") {
                    "The data format is not compatible. Please check your input structure."
                        .to_string()
                } else {
                    format!(
                        "Data processing error: {}",
                        Self::sanitize_technical_message(msg)
                    )
                }
            }
            QollectiveError::Transport(msg) => {
                if msg.contains("timeout") || msg.contains("timed out") {
                    "The request took too long to complete. Please try again.".to_string()
                } else if msg.contains("connection") || msg.contains("connect") {
                    "Unable to connect to the service. Please check your internet connection."
                        .to_string()
                } else if msg.contains("SSL") || msg.contains("TLS") || msg.contains("certificate")
                {
                    "Secure connection failed. This may be a temporary issue.".to_string()
                } else {
                    format!("Network error: {}", Self::sanitize_technical_message(msg))
                }
            }
            QollectiveError::Validation(msg) => {
                if msg.contains("required") {
                    "Required information is missing. Please check your input.".to_string()
                } else if msg.contains("invalid") {
                    "The provided information is not valid. Please verify your input.".to_string()
                } else {
                    format!(
                        "Input validation failed: {}",
                        Self::sanitize_technical_message(msg)
                    )
                }
            }
            QollectiveError::Security(msg) => {
                if msg.contains("authentication") || msg.contains("auth") {
                    "Authentication required. Please sign in again.".to_string()
                } else if msg.contains("authorization") || msg.contains("permission") {
                    "You don't have permission to perform this action.".to_string()
                } else if msg.contains("token") || msg.contains("expired") {
                    "Your session has expired. Please sign in again.".to_string()
                } else {
                    "Security verification failed. Please try signing in again.".to_string()
                }
            }
            QollectiveError::Internal(_) => {
                "An unexpected error occurred. Our team has been notified.".to_string()
            }
            QollectiveError::External(msg) => {
                if msg.contains("404") || msg.contains("not found") {
                    "The requested service is currently unavailable.".to_string()
                } else if msg.contains("503") || msg.contains("service unavailable") {
                    "The service is temporarily unavailable. Please try again in a few minutes."
                        .to_string()
                } else if msg.contains("500") || msg.contains("internal server error") {
                    "The service encountered an error. Please try again later.".to_string()
                } else {
                    format!(
                        "External service error: {}",
                        Self::sanitize_technical_message(msg)
                    )
                }
            }
            QollectiveError::Connection(msg) => {
                if msg.contains("refused") || msg.contains("unreachable") {
                    "Unable to reach the service. Please check your connection and try again."
                        .to_string()
                } else if msg.contains("timeout") {
                    "Connection timed out. Please try again.".to_string()
                } else {
                    format!(
                        "Connection error: {}",
                        Self::sanitize_technical_message(msg)
                    )
                }
            }
            QollectiveError::Deserialization(msg) => {
                if msg.contains("EOF") || msg.contains("unexpected end") {
                    "Incomplete data received. Please try again.".to_string()
                } else {
                    "Data format error. The response could not be processed.".to_string()
                }
            }
            QollectiveError::Remote(msg) => {
                format!("Service error: {}", Self::sanitize_technical_message(msg))
            }
            QollectiveError::Grpc(msg) => {
                if msg.contains("unavailable") {
                    "Service is temporarily unavailable. Please try again.".to_string()
                } else if msg.contains("deadline") || msg.contains("timeout") {
                    "Request timed out. Please try again.".to_string()
                } else {
                    format!(
                        "Service communication error: {}",
                        Self::sanitize_technical_message(msg)
                    )
                }
            }
            QollectiveError::TenantExtraction(_) => {
                "Account information could not be verified. Please sign in again.".to_string()
            }
            QollectiveError::FeatureNotEnabled(feature) => {
                format!(
                    "The {} feature is not available in this configuration.",
                    feature
                )
            }
            QollectiveError::AgentNotFound(agent_id) => {
                format!(
                    "Service '{}' is not available at this time.",
                    Self::sanitize_agent_id(agent_id)
                )
            }
            QollectiveError::ProtocolAdapter(msg) => {
                format!(
                    "Protocol communication error: {}",
                    Self::sanitize_technical_message(msg)
                )
            }

            // NATS-specific errors
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsConnection(_) => {
                "Unable to connect to the messaging service. Please try again.".to_string()
            }
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsMessage(_) => {
                "Message could not be delivered. Please try again.".to_string()
            }
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsTimeout(_) => {
                "Message delivery timed out. Please try again.".to_string()
            }
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsDiscovery(_) => {
                "Service discovery failed. The requested service may not be available.".to_string()
            }
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsSubject(_) => {
                "Invalid service address. Please check your request.".to_string()
            }
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsAuth(_) => {
                "Authentication with messaging service failed. Please try again.".to_string()
            }

            // MCP-specific errors
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpProtocol(_) => {
                "Tool communication protocol error. Please try again.".to_string()
            }
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpToolExecution(msg) => {
                if msg.contains("not found") {
                    "The requested tool is not available.".to_string()
                } else if msg.contains("timeout") {
                    "Tool execution timed out. Please try again.".to_string()
                } else {
                    format!(
                        "Tool execution failed: {}",
                        Self::sanitize_technical_message(msg)
                    )
                }
            }
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpServerRegistration(_) => {
                "Tool server registration failed. Please try again later.".to_string()
            }
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpClientConnection(_) => {
                "Unable to connect to tool server. Please try again.".to_string()
            }
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpServerNotFound(server) => {
                format!(
                    "Tool server '{}' is not available.",
                    Self::sanitize_server_name(server)
                )
            }
        }
    }

    /// Get retry policy for a given error
    pub fn get_retry_policy(error: &QollectiveError) -> &'static str {
        match error {
            QollectiveError::Transport(_) | QollectiveError::Connection(_) => "exponential_backoff",
            QollectiveError::External(_) | QollectiveError::Remote(_) => "linear_backoff",
            QollectiveError::Deserialization(_) | QollectiveError::Grpc(_) => "immediate_retry",
            QollectiveError::Security(_) | QollectiveError::Validation(_) => "none",
            QollectiveError::Internal(_) | QollectiveError::FeatureNotEnabled(_) => "none",

            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsTimeout(_) | QollectiveError::NatsConnection(_) => {
                "exponential_backoff"
            }
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsMessage(_) => "immediate_retry",
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsAuth(_) | QollectiveError::NatsSubject(_) => "none",
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsDiscovery(_) => "linear_backoff",

            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpToolExecution(_) => "immediate_retry",
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpClientConnection(_) => "exponential_backoff",
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpServerNotFound(_) | QollectiveError::McpServerRegistration(_) => {
                "linear_backoff"
            }
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpProtocol(_) => "none",

            _ => "none",
        }
    }

    /// Check if error should be shown to user or logged quietly
    pub fn is_user_error(error: &QollectiveError) -> bool {
        match error {
            QollectiveError::Validation(_) | QollectiveError::Security(_) => true,
            QollectiveError::FeatureNotEnabled(_) => true,
            QollectiveError::Transport(_) | QollectiveError::Connection(_) => true,
            QollectiveError::External(_) => true,
            QollectiveError::Internal(_) => false, // Log but don't show details
            _ => true,
        }
    }

    /// Check if error indicates a temporary issue that might resolve
    pub fn is_temporary_error(error: &QollectiveError) -> bool {
        match error {
            QollectiveError::Transport(_) | QollectiveError::Connection(_) => true,
            QollectiveError::External(_) | QollectiveError::Remote(_) => true,
            QollectiveError::Deserialization(_) => true,
            QollectiveError::Security(_) | QollectiveError::Validation(_) => false,
            QollectiveError::Internal(_) | QollectiveError::FeatureNotEnabled(_) => false,

            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsConnection(_) | QollectiveError::NatsTimeout(_) => true,
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsMessage(_) | QollectiveError::NatsDiscovery(_) => true,
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsAuth(_) | QollectiveError::NatsSubject(_) => false,

            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpClientConnection(_) | QollectiveError::McpServerNotFound(_) => true,
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpToolExecution(_) => true,
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpProtocol(_) | QollectiveError::McpServerRegistration(_) => false,

            _ => true,
        }
    }

    /// Sanitize technical error messages for user display
    fn sanitize_technical_message(msg: &str) -> String {
        // Remove file paths, stack traces, and overly technical details
        let msg = msg
            .replace("Error: ", "")
            .replace("Failed to ", "")
            .replace("Could not ", "")
            .replace("Unable to ", "");

        // Truncate very long messages
        if msg.len() > 100 {
            format!("{}...", &msg[..97])
        } else {
            msg
        }
    }

    /// Sanitize agent ID for user display
    fn sanitize_agent_id(agent_id: &str) -> String {
        // Remove UUID prefixes and technical identifiers
        if agent_id.contains('-') && agent_id.len() > 20 {
            agent_id.split('-').next().unwrap_or(agent_id).to_string()
        } else {
            agent_id.to_string()
        }
    }

    /// Sanitize server name for user display
    fn sanitize_server_name(server_name: &str) -> String {
        // Remove URL schemes and paths
        server_name
            .replace("http://", "")
            .replace("https://", "")
            .replace("ws://", "")
            .replace("wss://", "")
            .split('/')
            .next()
            .unwrap_or(server_name)
            .to_string()
    }
}

// WASM bindings for JavaScript access
#[wasm_bindgen]
pub struct JsErrorTranslator;

#[wasm_bindgen]
impl JsErrorTranslator {
    /// Translate error to user-friendly message
    #[wasm_bindgen]
    pub fn translate_error(error_type: &str, error_message: &str) -> String {
        // Create a QollectiveError from the string representation
        let error = match error_type.to_lowercase().as_str() {
            "validation" => QollectiveError::validation(error_message),
            "transport" => QollectiveError::transport(error_message),
            "connection" => QollectiveError::connection(error_message),
            "security" => QollectiveError::security(error_message),
            "external" => QollectiveError::external(error_message),
            "internal" => QollectiveError::internal(error_message),
            "serialization" => QollectiveError::serialization(error_message),
            "config" => QollectiveError::config(error_message),
            _ => QollectiveError::internal(error_message),
        };

        ErrorTranslator::translate_for_user(&error)
    }

    /// Get retry policy for error type
    #[wasm_bindgen]
    pub fn get_retry_policy_for_type(error_type: &str) -> String {
        let error = match error_type.to_lowercase().as_str() {
            "validation" => QollectiveError::validation(""),
            "transport" => QollectiveError::transport(""),
            "connection" => QollectiveError::connection(""),
            "security" => QollectiveError::security(""),
            "external" => QollectiveError::external(""),
            "internal" => QollectiveError::internal(""),
            "serialization" => QollectiveError::serialization(""),
            "config" => QollectiveError::config(""),
            _ => QollectiveError::internal(""),
        };

        ErrorTranslator::get_retry_policy(&error).to_string()
    }

    /// Check if error type is user-facing
    #[wasm_bindgen]
    pub fn is_user_facing_error(error_type: &str) -> bool {
        let error = match error_type.to_lowercase().as_str() {
            "validation" => QollectiveError::validation(""),
            "transport" => QollectiveError::transport(""),
            "connection" => QollectiveError::connection(""),
            "security" => QollectiveError::security(""),
            "external" => QollectiveError::external(""),
            "internal" => QollectiveError::internal(""),
            "serialization" => QollectiveError::serialization(""),
            "config" => QollectiveError::config(""),
            _ => QollectiveError::internal(""),
        };

        ErrorTranslator::is_user_error(&error)
    }

    /// Check if error type is temporary
    #[wasm_bindgen]
    pub fn is_temporary_error_type(error_type: &str) -> bool {
        let error = match error_type.to_lowercase().as_str() {
            "validation" => QollectiveError::validation(""),
            "transport" => QollectiveError::transport(""),
            "connection" => QollectiveError::connection(""),
            "security" => QollectiveError::security(""),
            "external" => QollectiveError::external(""),
            "internal" => QollectiveError::internal(""),
            "serialization" => QollectiveError::serialization(""),
            "config" => QollectiveError::config(""),
            _ => QollectiveError::internal(""),
        };

        ErrorTranslator::is_temporary_error(&error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_translation() {
        let error = QollectiveError::validation("field 'name' is required");
        let translated = ErrorTranslator::translate_for_user(&error);
        assert!(translated.contains("Required information is missing"));
        assert_eq!(ErrorTranslator::get_retry_policy(&error), "none");
        assert!(ErrorTranslator::is_user_error(&error));
        assert!(!ErrorTranslator::is_temporary_error(&error));
    }

    #[test]
    fn test_transport_error_translation() {
        let error = QollectiveError::transport("connection timed out");
        let translated = ErrorTranslator::translate_for_user(&error);
        assert!(translated.contains("took too long"));
        assert_eq!(
            ErrorTranslator::get_retry_policy(&error),
            "exponential_backoff"
        );
        assert!(ErrorTranslator::is_user_error(&error));
        assert!(ErrorTranslator::is_temporary_error(&error));
    }

    #[test]
    fn test_security_error_translation() {
        let error = QollectiveError::security("invalid token");
        let translated = ErrorTranslator::translate_for_user(&error);
        assert!(translated.contains("session has expired"));
        assert_eq!(ErrorTranslator::get_retry_policy(&error), "none");
        assert!(ErrorTranslator::is_user_error(&error));
        assert!(!ErrorTranslator::is_temporary_error(&error));
    }

    #[test]
    fn test_technical_message_sanitization() {
        let long_msg = "Failed to connect to server at 192.168.1.1:8080 with SSL certificate validation error: unable to verify the first certificate in chain";
        let sanitized = ErrorTranslator::sanitize_technical_message(long_msg);
        assert!(sanitized.len() <= 100);
        assert!(!sanitized.starts_with("Failed to"));
    }

    #[test]
    fn test_agent_id_sanitization() {
        let uuid_agent = "test-agent-12345-67890-abcdef";
        let sanitized = ErrorTranslator::sanitize_agent_id(uuid_agent);
        assert_eq!(sanitized, "test");
    }
}
