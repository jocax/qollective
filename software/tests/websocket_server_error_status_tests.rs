// Tests for WebSocket server using EnvelopeError status codes instead of hardcoded values
// Verifies WebSocket server extracts HTTP status codes from EnvelopeError.http_status_code

use qollective::envelope::builder::EnvelopeError;
use qollective::error::QollectiveError;
use qollective::client::websocket::WebSocketMessageType;
use serde_json::json;

/// Test that WebSocket server extracts status codes from EnvelopeError for handler errors
#[cfg(feature = "websocket-server")]
#[test]
fn test_websocket_server_extracts_status_code_from_envelope_error() {
    // Create validation error with HTTP 400 status
    let validation_error = QollectiveError::validation_error(
        "Invalid request parameters",
        Some(json!({"field": "user_id", "expected": "string"}))
    );
    
    // Verify the error has the expected HTTP status code
    assert_eq!(validation_error.code, "VALIDATION_FAILED");
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(validation_error.http_status_code, Some(400));
    
    // Create auth error with HTTP 401 status
    let auth_error = QollectiveError::auth_error(
        "Authentication token is invalid",
        Some(json!({"token": "expired"}))
    );
    
    assert_eq!(auth_error.code, "AUTHENTICATION_FAILED");
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(auth_error.http_status_code, Some(401));
}

/// Test helper function for converting EnvelopeError to WebSocket error message type
#[cfg(feature = "websocket-server")]
#[test]
fn test_envelope_error_to_websocket_error_message() {
    use qollective::server::websocket::envelope_error_to_websocket_message;
    
    // Test custom error with HTTP 422 status
    let custom_error = QollectiveError::custom_error(
        "VALIDATION_FAILED",
        "Request validation failed",
        Some(json!({"field": "email", "issue": "invalid_format"})),
        422
    );
    
    let websocket_message = envelope_error_to_websocket_message(&custom_error);
    
    match websocket_message {
        WebSocketMessageType::Error { message, code } => {
            assert!(message.contains("Request validation failed"));
            assert_eq!(code, Some(422));
        }
        _ => panic!("Expected WebSocketMessageType::Error"),
    }
}

/// Test that WebSocket server maintains backward compatibility for existing error patterns
#[cfg(feature = "websocket-server")]
#[test]
fn test_websocket_server_backward_compatibility() {
    use qollective::server::websocket::envelope_error_to_websocket_message;
    
    // Test legacy EnvelopeError without http_status_code
    let legacy_error = EnvelopeError {
        code: "LEGACY_ERROR".to_string(),
        message: "This is a legacy error".to_string(),
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
    
    let websocket_message = envelope_error_to_websocket_message(&legacy_error);
    
    match websocket_message {
        WebSocketMessageType::Error { message, code } => {
            assert!(message.contains("This is a legacy error"));
            // Should fallback to appropriate status code based on error pattern
            assert!(code.is_some());
            assert!(code.unwrap() >= 400);
        }
        _ => panic!("Expected WebSocketMessageType::Error"),
    }
}

/// Test that WebSocket server handles various error scenarios with proper status codes
#[cfg(feature = "websocket-server")]
#[test]
fn test_websocket_server_various_error_scenarios() {
    use qollective::server::websocket::envelope_error_to_websocket_message;
    
    let test_cases = vec![
        (
            QollectiveError::validation_error("Field missing", None),
            400,
            "validation error"
        ),
        (
            QollectiveError::auth_error("Token expired", None),
            401,
            "authentication error"
        ),
        (
            QollectiveError::not_found_error("Resource missing", None),
            404,
            "not found error"
        ),
        (
            QollectiveError::server_error("Internal failure", None),
            500,
            "server error"
        ),
    ];
    
    for (error, expected_status, description) in test_cases {
        let websocket_message = envelope_error_to_websocket_message(&error);
        
        match websocket_message {
            WebSocketMessageType::Error { message: _, code } => {
                assert_eq!(code, Some(expected_status), 
                    "Failed for {}: expected {}, got {:?}", description, expected_status, code);
            }
            _ => panic!("Expected WebSocketMessageType::Error for {}", description),
        }
    }
}

/// Test WebSocket server error response serialization
#[cfg(feature = "websocket-server")]
#[test] 
fn test_websocket_error_response_serialization() {
    let error = QollectiveError::custom_error(
        "RATE_LIMIT_EXCEEDED",
        "Too many requests from this connection",
        Some(json!({"limit": 100, "window": "60s"})),
        429
    );
    
    // Test that the error can be serialized to JSON for WebSocket transmission
    let json_string = serde_json::to_string(&error).expect("Should serialize");
    assert!(json_string.contains("RATE_LIMIT_EXCEEDED"));
    assert!(json_string.contains("429"));
    
    // Test deserialization roundtrip
    let deserialized: EnvelopeError = serde_json::from_str(&json_string)
        .expect("Should deserialize");
    
    assert_eq!(deserialized.code, "RATE_LIMIT_EXCEEDED");
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(deserialized.http_status_code, Some(429));
}

/// Test WebSocket server error message includes HTTP status equivalent for metadata
#[cfg(feature = "websocket-server")]
#[test]
fn test_websocket_error_includes_http_status_metadata() {
    use qollective::server::websocket::envelope_error_to_websocket_message;
    
    let server_error = QollectiveError::server_error(
        "Database connection timeout",
        Some(json!({"timeout": "30s", "retries": 3}))
    );
    
    let websocket_message = envelope_error_to_websocket_message(&server_error);
    
    match websocket_message {
        WebSocketMessageType::Error { message, code } => {
            // Verify the message includes meaningful error info
            assert!(message.contains("Database connection timeout"));
            
            // Verify the status code is set to 500 for server errors
            assert_eq!(code, Some(500));
        }
        _ => panic!("Expected WebSocketMessageType::Error"),
    }
}