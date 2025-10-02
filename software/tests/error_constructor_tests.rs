// Tests for QollectiveError convenience constructor methods with custom HTTP status codes
// Verifies that new error constructors create proper EnvelopeError instances

use qollective::envelope::builder::EnvelopeError;
use qollective::error::QollectiveError;
use serde_json;

/// Test convenience constructor for validation errors (HTTP 400)
#[test]
fn test_validation_error_constructor_with_custom_status() {
    let envelope_error = QollectiveError::validation_error(
        "Required field 'tenant_id' is missing",
        Some(serde_json::json!({"field": "tenant_id", "expected": "string"}))
    );
    
    assert_eq!(envelope_error.code, "VALIDATION_FAILED");
    assert_eq!(envelope_error.message, "Required field 'tenant_id' is missing");
    assert!(envelope_error.details.is_some());
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(envelope_error.http_status_code, Some(400));
}

/// Test convenience constructor for authentication errors (HTTP 401)
#[test]
fn test_auth_error_constructor_with_custom_status() {
    let envelope_error = QollectiveError::auth_error(
        "Invalid or expired authentication token",
        Some(serde_json::json!({"token_status": "expired", "expires_at": "2024-01-01T00:00:00Z"}))
    );
    
    assert_eq!(envelope_error.code, "AUTHENTICATION_FAILED");
    assert_eq!(envelope_error.message, "Invalid or expired authentication token");
    assert!(envelope_error.details.is_some());
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(envelope_error.http_status_code, Some(401));
}

/// Test convenience constructor for not found errors (HTTP 404)
#[test]
fn test_not_found_error_constructor_with_custom_status() {
    let envelope_error = QollectiveError::not_found_error(
        "Requested resource does not exist",
        Some(serde_json::json!({"resource": "user", "id": "12345"}))
    );
    
    assert_eq!(envelope_error.code, "RESOURCE_NOT_FOUND");
    assert_eq!(envelope_error.message, "Requested resource does not exist");
    assert!(envelope_error.details.is_some());
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(envelope_error.http_status_code, Some(404));
}

/// Test convenience constructor for server errors (HTTP 500)
#[test]
fn test_server_error_constructor_with_custom_status() {
    let envelope_error = QollectiveError::server_error(
        "Internal server error occurred during processing",
        Some(serde_json::json!({"component": "database", "operation": "query"}))
    );
    
    assert_eq!(envelope_error.code, "INTERNAL_SERVER_ERROR");
    assert_eq!(envelope_error.message, "Internal server error occurred during processing");
    assert!(envelope_error.details.is_some());
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(envelope_error.http_status_code, Some(500));
}

/// Test custom error constructor with arbitrary status code
#[test]
fn test_custom_error_constructor_with_arbitrary_status() {
    let envelope_error = QollectiveError::custom_error(
        "RATE_LIMIT_EXCEEDED",
        "Too many requests from this client",
        Some(serde_json::json!({"limit": 100, "window": "60s", "reset_time": "2025-09-02T12:01:00Z"})),
        429  // Too Many Requests
    );
    
    assert_eq!(envelope_error.code, "RATE_LIMIT_EXCEEDED");
    assert_eq!(envelope_error.message, "Too many requests from this client");
    assert!(envelope_error.details.is_some());
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(envelope_error.http_status_code, Some(429));
}

/// Test HTTP status code validation - invalid codes should be rejected
#[test]
fn test_custom_error_constructor_validates_status_codes() {
    // Valid status codes (400-599) should be accepted
    let valid_error = QollectiveError::custom_error(
        "TEST_ERROR",
        "Test message",
        None,
        422  // Unprocessable Entity
    );
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(valid_error.http_status_code, Some(422));
    
    // Invalid status codes (outside 400-599) should be normalized to 500
    let invalid_error = QollectiveError::custom_error(
        "INVALID_STATUS",
        "This should normalize to 500",
        None,
        200  // Success code - invalid for errors
    );
    
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(invalid_error.http_status_code, Some(500)); // Should be normalized
}

/// Test that convenience constructors create valid JSON-serializable errors
#[test]
fn test_convenience_constructors_json_serialization() {
    let errors = vec![
        QollectiveError::validation_error("Validation failed", None),
        QollectiveError::auth_error("Auth failed", None),
        QollectiveError::not_found_error("Not found", None),
        QollectiveError::server_error("Server error", None),
    ];
    
    for error in errors {
        // Test that each error can be serialized to JSON
        let json_string = serde_json::to_string(&error).expect("Should serialize to JSON");
        assert!(!json_string.is_empty());
        
        // Test that it can be deserialized back
        let deserialized: EnvelopeError = serde_json::from_str(&json_string)
            .expect("Should deserialize from JSON");
        
        assert!(!deserialized.code.is_empty());
        assert!(!deserialized.message.is_empty());
    }
}

/// Test backward compatibility - existing error creation patterns still work
#[test]
fn test_backward_compatibility_with_existing_error_patterns() {
    // Test existing QollectiveError constructors still work
    let transport_error = QollectiveError::transport("Connection failed");
    assert_eq!(transport_error.to_string(), "transport error: Connection failed");
    
    let validation_error = QollectiveError::validation("Invalid data");
    assert_eq!(validation_error.to_string(), "validation error: Invalid data");
    
    let config_error = QollectiveError::config("Missing config file");
    assert_eq!(config_error.to_string(), "configuration error: Missing config file");
}

/// Test that convenience constructors with None details work correctly
#[test]
fn test_convenience_constructors_with_none_details() {
    let error = QollectiveError::validation_error("Simple validation error", None);
    
    assert_eq!(error.code, "VALIDATION_FAILED");
    assert_eq!(error.message, "Simple validation error");
    assert!(error.details.is_none());
    assert!(error.trace.is_none());
}