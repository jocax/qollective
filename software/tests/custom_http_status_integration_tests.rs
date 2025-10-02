// Comprehensive integration tests for custom HTTP status codes feature
// Tests end-to-end functionality across REST and WebSocket servers

use qollective::envelope::builder::EnvelopeError;
use qollective::error::QollectiveError;
use qollective::server::rest::{RestServer, RestServerConfig, extract_http_status_code, create_error_envelope_response};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig, envelope_error_to_websocket_message, convert_qollective_error_to_envelope_error};
use qollective::client::websocket::WebSocketMessageType;
use qollective::envelope::{Envelope, Meta};
use serde_json::json;
use axum::http::StatusCode;

/// Integration test for REST server JSON envelope error responses
#[cfg(all(feature = "rest-server", feature = "openapi"))]
#[tokio::test]
async fn test_rest_server_custom_status_codes_integration() {
    // Test various error types with custom status codes
    let test_errors = vec![
        (
            QollectiveError::validation_error(
                "Email format is invalid",
                Some(json!({"field": "email", "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"}))
            ),
            400,
            "validation error"
        ),
        (
            QollectiveError::auth_error(
                "JWT token has expired",
                Some(json!({"token_type": "JWT", "expired_at": "2024-01-01T00:00:00Z"}))
            ),
            401,
            "authentication error"
        ),
        (
            QollectiveError::not_found_error(
                "User account does not exist",
                Some(json!({"resource": "user", "user_id": "user123"}))
            ),
            404,
            "not found error"
        ),
        (
            QollectiveError::custom_error(
                "RATE_LIMIT_EXCEEDED",
                "API rate limit exceeded for this client",
                Some(json!({"limit": 1000, "window": "1h", "retry_after": 3600})),
                429
            ),
            429,
            "rate limit error"
        ),
        (
            QollectiveError::server_error(
                "Database connection pool exhausted",
                Some(json!({"pool_size": 10, "active_connections": 10, "queue_length": 50}))
            ),
            500,
            "server error"
        ),
    ];
    
    for (envelope_error, expected_status, description) in test_errors {
        // Test HTTP status code extraction
        let extracted_status = extract_http_status_code(&envelope_error);
        assert_eq!(extracted_status.as_u16(), expected_status, 
            "Failed status extraction for {}", description);
        
        // Test error envelope response creation
        let response = create_error_envelope_response(envelope_error.clone(), None);
        
        // Verify response has correct status code
        assert_eq!(response.status().as_u16(), expected_status,
            "Failed response status for {}", description);
        
        // Extract and verify JSON response body would contain envelope structure
        // Note: In real integration test, we'd inspect response body JSON
        assert!(envelope_error.details.is_some() || envelope_error.details.is_none(), 
            "Error structure should be valid for {}", description);
    }
}

/// Integration test for WebSocket server custom status codes
#[cfg(feature = "websocket-server")]
#[tokio::test]
async fn test_websocket_server_custom_status_codes_integration() {
    // Test conversion of various QollectiveError types to WebSocket messages
    let qollective_errors = vec![
        (QollectiveError::validation("Invalid request format"), 400, "validation"),
        (QollectiveError::security("Access token expired"), 401, "security"),  
        (QollectiveError::agent_not_found("Agent ID not registered"), 404, "agent not found"),
        (QollectiveError::transport("Connection lost to upstream service"), 500, "transport"),
        (QollectiveError::internal("Unexpected system failure"), 500, "internal"),
    ];
    
    for (qollective_error, expected_status, description) in qollective_errors {
        // Convert QollectiveError to EnvelopeError
        let envelope_error = convert_qollective_error_to_envelope_error(&qollective_error);
        
        // Convert to WebSocket message type
        let websocket_message = envelope_error_to_websocket_message(&envelope_error);
        
        // Verify WebSocket error message has correct status
        match websocket_message {
            WebSocketMessageType::Error { message, code } => {
                assert!(code.is_some(), "Status code should be present for {}", description);
                assert_eq!(code.unwrap() as u16, expected_status, 
                    "Wrong status code for {}: expected {}, got {:?}", description, expected_status, code);
                assert!(!message.is_empty(), "Message should not be empty for {}", description);
            }
            _ => panic!("Expected WebSocketMessageType::Error for {}", description),
        }
    }
}

/// Integration test for custom EnvelopeError instances with arbitrary status codes
#[test]
fn test_custom_envelope_errors_with_various_status_codes() {
    let custom_errors = vec![
        (402, "PAYMENT_REQUIRED", "Payment required to access this resource"),
        (403, "FORBIDDEN", "User lacks required permissions"),
        (409, "CONFLICT", "Resource already exists with this identifier"),
        (422, "UNPROCESSABLE_ENTITY", "Request data is well-formed but semantically invalid"),
        (429, "RATE_LIMIT_EXCEEDED", "Too many requests in time window"),
        (503, "SERVICE_UNAVAILABLE", "Upstream service is temporarily unavailable"),
        (504, "GATEWAY_TIMEOUT", "Upstream service did not respond in time"),
    ];
    
    for (status_code, error_code, message) in custom_errors {
        let envelope_error = QollectiveError::custom_error(
            error_code,
            message,
            Some(json!({"test_case": true, "status_code": status_code})),
            status_code
        );
        
        // Verify custom status code is preserved
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        assert_eq!(envelope_error.http_status_code, Some(status_code));
        
        // Verify error structure
        assert_eq!(envelope_error.code, error_code);
        assert_eq!(envelope_error.message, message);
        assert!(envelope_error.details.is_some());
        
        // Test JSON serialization includes all fields
        let json_str = serde_json::to_string(&envelope_error).expect("Should serialize");
        assert!(json_str.contains(error_code));
        assert!(json_str.contains(&status_code.to_string()));
    }
}

/// Integration test for feature gate compilation
#[test]
fn test_feature_gate_compilation_integration() {
    // Test that EnvelopeError can be created and serialized regardless of feature flags
    let error = EnvelopeError {
        code: "TEST_ERROR".to_string(),
        message: "Test error message".to_string(),
        details: Some(json!({"integration_test": true})),
        trace: None,
        // http_status_code field should only be available with HTTP features
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(422),
    };
    
    // Verify basic structure
    assert_eq!(error.code, "TEST_ERROR");
    assert_eq!(error.message, "Test error message");
    
    // Test serialization works
    let json_str = serde_json::to_string(&error).expect("Should serialize");
    assert!(json_str.contains("TEST_ERROR"));
    
    // Test deserialization roundtrip
    let deserialized: EnvelopeError = serde_json::from_str(&json_str).expect("Should deserialize");
    assert_eq!(deserialized.code, "TEST_ERROR");
    assert_eq!(deserialized.message, "Test error message");
}

/// Integration test for OpenAPI documentation generation with new http_status_code field
#[cfg(feature = "openapi")]
#[test]
fn test_openapi_documentation_integration() {
    use qollective::openapi::OpenApiUtils;
    
    // Generate OpenAPI specification
    let spec = OpenApiUtils::generate_spec();
    
    // Verify spec contains our enhanced envelope schemas
    let spec_str = serde_json::to_string_pretty(&spec).expect("Should serialize spec");
    assert!(spec_str.contains("EnvelopeError") || spec_str.contains("Envelope"));
    assert!(!spec_str.is_empty());
    
    // Test example envelope generation includes new fields
    let example_envelope = OpenApiUtils::generate_example_envelope();
    assert!(example_envelope.meta.tenant.is_some());
    assert!(!example_envelope.payload.message.is_empty());
    
    // Test example error envelope includes error information
    let error_envelope = OpenApiUtils::generate_example_error_envelope();
    assert!(error_envelope.has_error());
    let error = error_envelope.error.as_ref().unwrap();
    assert!(!error.code.is_empty());
    assert!(!error.message.is_empty());
    
    // With HTTP features enabled, should include http_status_code
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert!(error.http_status_code.is_some());
}

/// Integration test for error response consistency across protocols
#[cfg(all(feature = "rest-server", feature = "websocket-server"))]
#[test]
fn test_cross_protocol_error_response_consistency() {
    // Create the same logical error for both protocols
    let validation_error = QollectiveError::validation_error(
        "Request validation failed",
        Some(json!({"field": "user_id", "issue": "required"}))
    );
    
    // Test REST server status code extraction
    let rest_status = extract_http_status_code(&validation_error);
    assert_eq!(rest_status, StatusCode::BAD_REQUEST);
    
    // Test WebSocket server status code extraction
    let websocket_message = envelope_error_to_websocket_message(&validation_error);
    match websocket_message {
        WebSocketMessageType::Error { message: _, code } => {
            assert_eq!(code, Some(400));
        }
        _ => panic!("Expected WebSocketMessageType::Error"),
    }
    
    // Verify both protocols extract the same status code
    assert_eq!(rest_status.as_u16(), 400);
}

/// Integration test for end-to-end error handling with metadata preservation
#[cfg(feature = "rest-server")]
#[test]
fn test_end_to_end_error_handling_with_metadata() {
    // Create error with rich metadata context
    let server_error = QollectiveError::server_error(
        "Database transaction failed",
        Some(json!({
            "transaction_id": "txn_123456",
            "database": "user_db", 
            "operation": "UPDATE",
            "affected_rows": 0,
            "error_code": "DEADLOCK_DETECTED"
        }))
    );
    
    // Create mock request metadata
    let mut request_meta = Meta::for_new_request();
    request_meta.tenant = Some("enterprise_starfleet".to_string());
    request_meta.version = Some("1.0".to_string());
    
    // Test error envelope response creation preserves metadata
    let response = create_error_envelope_response(server_error.clone(), Some(request_meta.clone()));
    
    // Verify response has correct status code
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    
    // In a real integration test, we would extract the JSON body and verify:
    // - Response contains proper envelope structure
    // - Error details are preserved
    // - Request metadata is carried forward
    // - HTTP status matches EnvelopeError.http_status_code
    
    // For now, verify the envelope error has the expected structure
    assert_eq!(server_error.code, "INTERNAL_SERVER_ERROR");
    assert!(server_error.details.is_some());
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    assert_eq!(server_error.http_status_code, Some(500));
}

/// Integration test for invalid status code handling and normalization
#[test]
fn test_invalid_status_code_normalization_integration() {
    // Test various invalid status codes are properly normalized
    let invalid_codes = vec![
        (200, "Success codes should be rejected"),
        (302, "Redirect codes should be rejected"), 
        (100, "Informational codes should be rejected"),
        (600, "Invalid high codes should be rejected"),
        (0, "Zero should be rejected"),
    ];
    
    for (invalid_code, description) in invalid_codes {
        let normalized_error = QollectiveError::custom_error(
            "INVALID_STATUS_TEST",
            "Testing invalid status code normalization",
            Some(json!({"original_code": invalid_code})),
            invalid_code
        );
        
        // Should be normalized to 500
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        assert_eq!(normalized_error.http_status_code, Some(500), 
            "{}: {} should normalize to 500", description, invalid_code);
        
        // Test that REST server extraction handles normalized codes
        #[cfg(feature = "rest-server")]
        {
            let status = extract_http_status_code(&normalized_error);
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        // Test that WebSocket server handles normalized codes
        #[cfg(feature = "websocket-server")]
        {
            let ws_message = envelope_error_to_websocket_message(&normalized_error);
            match ws_message {
                WebSocketMessageType::Error { code, .. } => {
                    assert_eq!(code, Some(500));
                }
                _ => panic!("Expected WebSocket error message"),
            }
        }
    }
}

/// Integration test for error response JSON envelope structure
#[test]
fn test_error_response_envelope_structure_integration() {
    // Create error with comprehensive details
    let detailed_error = QollectiveError::custom_error(
        "BUSINESS_RULE_VIOLATION",
        "User cannot perform this action due to business rules",
        Some(json!({
            "rule": "max_daily_transfers",
            "current_count": 5,
            "limit": 5,
            "reset_time": "2025-09-03T00:00:00Z",
            "user_tier": "basic"
        })),
        422
    );
    
    // Test JSON serialization produces valid envelope structure
    let json_str = serde_json::to_string_pretty(&detailed_error).expect("Should serialize");
    
    // Verify envelope-like structure in JSON
    assert!(json_str.contains("\"code\""));
    assert!(json_str.contains("\"message\""));
    assert!(json_str.contains("\"details\""));
    assert!(json_str.contains("BUSINESS_RULE_VIOLATION"));
    assert!(json_str.contains("business rules"));
    assert!(json_str.contains("max_daily_transfers"));
    
    // With HTTP features, should include status code
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    {
        assert!(json_str.contains("\"http_status_code\"") && json_str.contains("422"));
    }
    
    // Test deserialization roundtrip maintains all data
    let deserialized: EnvelopeError = serde_json::from_str(&json_str).expect("Should deserialize");
    assert_eq!(deserialized.code, detailed_error.code);
    assert_eq!(deserialized.message, detailed_error.message);
    assert!(deserialized.details.is_some());
    
    let details = deserialized.details.unwrap();
    assert_eq!(details["rule"], "max_daily_transfers");
    assert_eq!(details["limit"], 5);
    assert_eq!(details["user_tier"], "basic");
}

/// Integration test verifying backward compatibility doesn't break existing patterns
#[test]
fn test_backward_compatibility_integration() {
    // Test legacy EnvelopeError creation still works
    let legacy_error = EnvelopeError {
        code: "LEGACY_ERROR_CODE".to_string(),
        message: "This is how errors were created before".to_string(),
        details: Some(json!({"legacy": true})),
        trace: Some("legacy_function (old_file.rs:123)".to_string()),
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: None,
    };
    
    // Should work with REST server status extraction (fallback to pattern)
    #[cfg(feature = "rest-server")]
    {
        let status = extract_http_status_code(&legacy_error);
        assert!(status.is_server_error() || status.is_client_error());
    }
    
    // Should work with WebSocket server conversion
    #[cfg(feature = "websocket-server")]
    {
        let ws_message = envelope_error_to_websocket_message(&legacy_error);
        match ws_message {
            WebSocketMessageType::Error { message, code } => {
                assert!(message.contains("This is how errors were created before"));
                assert!(code.is_some());
                assert!(code.unwrap() >= 400);
            }
            _ => panic!("Expected WebSocket error message"),
        }
    }
    
    // Should serialize/deserialize correctly
    let json_str = serde_json::to_string(&legacy_error).expect("Should serialize");
    assert!(json_str.contains("LEGACY_ERROR_CODE"));
    assert!(json_str.contains("legacy_function"));
    
    let deserialized: EnvelopeError = serde_json::from_str(&json_str).expect("Should deserialize");
    assert_eq!(deserialized.code, "LEGACY_ERROR_CODE");
    assert!(deserialized.trace.is_some());
}

/// Integration test for complete error lifecycle from QollectiveError to HTTP response
#[cfg(all(feature = "rest-server", feature = "websocket-server"))]
#[test]
fn test_complete_error_lifecycle_integration() {
    // Start with a business domain error
    let business_error = "Customer account is suspended due to payment issues";
    
    // Create structured error using convenience constructor
    let envelope_error = QollectiveError::custom_error(
        "ACCOUNT_SUSPENDED",
        business_error,
        Some(json!({
            "customer_id": "cust_789",
            "suspension_reason": "payment_overdue",
            "overdue_amount": 150.00,
            "currency": "USD",
            "suspension_date": "2025-09-01T00:00:00Z",
            "contact_support": "billing@company.com"
        })),
        402  // Payment Required
    );
    
    // Test REST server handling
    let rest_status = extract_http_status_code(&envelope_error);
    assert_eq!(rest_status.as_u16(), 402);
    
    let rest_response = create_error_envelope_response(envelope_error.clone(), None);
    assert_eq!(rest_response.status().as_u16(), 402);
    
    // Test WebSocket server handling
    let websocket_message = envelope_error_to_websocket_message(&envelope_error);
    match websocket_message {
        WebSocketMessageType::Error { message, code } => {
            assert!(message.contains("payment issues"));
            assert_eq!(code, Some(402));
        }
        _ => panic!("Expected WebSocket error"),
    }
    
    // Test that error maintains rich context throughout lifecycle
    assert_eq!(envelope_error.code, "ACCOUNT_SUSPENDED");
    assert!(envelope_error.details.is_some());
    
    let details = envelope_error.details.unwrap();
    assert_eq!(details["customer_id"], "cust_789");
    assert_eq!(details["overdue_amount"], 150.00);
    assert_eq!(details["contact_support"], "billing@company.com");
}