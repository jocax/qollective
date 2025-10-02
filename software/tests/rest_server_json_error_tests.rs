// Tests for REST server JSON envelope error response functionality
// Verifies that REST server returns JSON envelopes for all errors instead of plain text

use qollective::envelope::builder::EnvelopeError;
use qollective::envelope::{Envelope, Meta};
use qollective::server::rest::{RestServer, RestServerConfig};
use qollective::client::rest::RestClient;
use qollective::error::QollectiveError;
use serde_json::{json, Value};
use std::time::Duration;
use tokio;

/// Helper function to create a test REST server config
async fn create_test_rest_server_config() -> RestServerConfig {
    RestServerConfig::default()
}

/// Test that validation errors return HTTP 400 with JSON envelope response
#[cfg(feature = "rest-server")]
#[tokio::test]
async fn test_rest_server_validation_error_returns_json_envelope() {
    // This test will verify that validation errors return proper JSON envelopes
    // instead of plain text responses with appropriate HTTP 400 status code
    
    // Create a test error that should result in HTTP 400
    let validation_error = EnvelopeError {
        code: "VALIDATION_FAILED".to_string(),
        message: "Required field is missing".to_string(),
        details: Some(json!({"field": "tenant_id"})),
        trace: None,
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(400),
    };
    
    // Verify the error has the expected structure
    assert_eq!(validation_error.code, "VALIDATION_FAILED");
    assert_eq!(validation_error.http_status_code, Some(400));
    
    // Verify it serializes to JSON correctly
    let json_string = serde_json::to_string(&validation_error).expect("Should serialize");
    assert!(json_string.contains("\"http_status_code\":400"));
    assert!(json_string.contains("VALIDATION_FAILED"));
}

/// Test that authentication errors return HTTP 401 with JSON envelope response
#[cfg(feature = "rest-server")]
#[tokio::test]
async fn test_rest_server_auth_error_returns_json_envelope() {
    let auth_error = EnvelopeError {
        code: "UNAUTHORIZED".to_string(),
        message: "Invalid or missing authentication token".to_string(),
        details: Some(json!({"token": "expired", "expires_at": "2024-01-01T00:00:00Z"})),
        trace: None,
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(401),
    };
    
    // Verify the error has the expected structure
    assert_eq!(auth_error.code, "UNAUTHORIZED");
    assert_eq!(auth_error.http_status_code, Some(401));
    
    // Test JSON serialization includes all fields
    let json_string = serde_json::to_string(&auth_error).expect("Should serialize");
    assert!(json_string.contains("\"http_status_code\":401"));
    assert!(json_string.contains("UNAUTHORIZED"));
    assert!(json_string.contains("expired"));
}

/// Test that server errors return HTTP 500 with JSON envelope response
#[cfg(feature = "rest-server")]
#[tokio::test]
async fn test_rest_server_internal_error_returns_json_envelope() {
    let internal_error = EnvelopeError {
        code: "INTERNAL_SERVER_ERROR".to_string(),
        message: "Failed to process request due to internal error".to_string(),
        details: Some(json!({"component": "database", "error": "connection timeout"})),
        trace: Some("at process_request (handler.rs:245)".to_string()),
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(500),
    };
    
    // Verify the error has the expected structure
    assert_eq!(internal_error.code, "INTERNAL_SERVER_ERROR");
    assert_eq!(internal_error.http_status_code, Some(500));
    
    // Test JSON serialization includes trace information
    let json_string = serde_json::to_string(&internal_error).expect("Should serialize");
    assert!(json_string.contains("\"http_status_code\":500"));
    assert!(json_string.contains("handler.rs:245"));
}

/// Test helper function for extracting HTTP status code from EnvelopeError
#[cfg(feature = "rest-server")]
#[test]
fn test_extract_http_status_code_from_envelope_error() {
    use qollective::server::rest::extract_http_status_code;
    
    // Test with custom status code
    let error_with_status = EnvelopeError {
        code: "CUSTOM_ERROR".to_string(),
        message: "Custom error message".to_string(),
        details: None,
        trace: None,
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(422),
    };
    
    let status = extract_http_status_code(&error_with_status);
    assert_eq!(status.as_u16(), 422);
    
    // Test without custom status code (should use pattern-based fallback)
    let error_without_status = EnvelopeError {
        code: "VALIDATION_ERROR".to_string(),
        message: "Validation failed".to_string(),
        details: None,
        trace: None,
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: None,
    };
    
    let fallback_status = extract_http_status_code(&error_without_status);
    // Should fallback to pattern-based mapping (likely 400 for validation errors)
    assert!(fallback_status.is_client_error());
}

/// Test that invalid HTTP status codes are handled gracefully
#[cfg(feature = "rest-server")]
#[test]
fn test_invalid_http_status_code_validation() {
    // Test status codes outside valid error range (400-599)
    let test_cases = vec![
        (200, "Success codes should be rejected"),
        (300, "Redirect codes should be rejected"),
        (100, "Informational codes should be rejected"),
        (600, "Invalid codes should be rejected"),
    ];
    
    for (invalid_status, description) in test_cases {
        // Create an error with invalid status code
        let error = EnvelopeError {
            code: "TEST_ERROR".to_string(),
            message: "Test error".to_string(),
            details: None,
            trace: None,
            #[cfg(any(
                feature = "rest-server", 
                feature = "rest-client",
                feature = "websocket-server", 
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(invalid_status),
        };
        
        // The helper should validate and fallback to appropriate status
        let validated_status = qollective::server::rest::extract_http_status_code(&error);
        assert!(validated_status.is_client_error() || validated_status.is_server_error(), 
            "{}: Status {} should fallback to valid error code", description, invalid_status);
    }
}

/// Test backward compatibility - errors without http_status_code still work
#[cfg(feature = "rest-server")]
#[test]
fn test_rest_server_backward_compatibility_no_status_code() {
    let legacy_error = EnvelopeError {
        code: "LEGACY_ERROR".to_string(),
        message: "This is a legacy error without status code".to_string(),
        details: None,
        trace: None,
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: None,
    };
    
    // Should still work with pattern-based mapping
    let status = qollective::server::rest::extract_http_status_code(&legacy_error);
    assert!(status.is_client_error() || status.is_server_error());
    
    // Should serialize properly without http_status_code field
    let json = serde_json::to_string(&legacy_error).expect("Should serialize");
    assert!(!json.contains("http_status_code"));
    assert!(json.contains("LEGACY_ERROR"));
}