// Tests for EnvelopeError http_status_code field functionality
// Tests serialization behavior with/without HTTP features

use qollective::envelope::builder::EnvelopeError;
use serde_json;

/// Test that http_status_code field is included when HTTP features are enabled
#[cfg(any(
    feature = "rest-server", 
    feature = "rest-client",
    feature = "websocket-server", 
    feature = "websocket-client",
    feature = "a2a"
))]
#[test]
fn test_envelope_error_with_http_status_code_included() {
    let error = EnvelopeError {
        code: "VALIDATION_FAILED".to_string(),
        message: "Required field missing".to_string(),
        details: Some(serde_json::json!({"field": "tenant_id"})),
        trace: Some("at validate_envelope (envelope.rs:123)".to_string()),
        http_status_code: Some(400),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&error).expect("Failed to serialize EnvelopeError");
    
    // Verify http_status_code is present in JSON
    assert!(json.contains("\"http_status_code\":400"));
    
    // Deserialize back to verify roundtrip
    let deserialized: EnvelopeError = serde_json::from_str(&json)
        .expect("Failed to deserialize EnvelopeError");
    
    assert_eq!(deserialized.http_status_code, Some(400));
    assert_eq!(deserialized.code, "VALIDATION_FAILED");
    assert_eq!(deserialized.message, "Required field missing");
}

/// Test that http_status_code field is omitted from JSON when None (skip_serializing_if)
#[cfg(any(
    feature = "rest-server", 
    feature = "rest-client",
    feature = "websocket-server", 
    feature = "websocket-client",
    feature = "a2a"
))]
#[test]
fn test_envelope_error_http_status_code_omitted_when_none() {
    let error = EnvelopeError {
        code: "GENERIC_ERROR".to_string(),
        message: "Something went wrong".to_string(),
        details: None,
        trace: None,
        http_status_code: None,
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&error).expect("Failed to serialize EnvelopeError");
    
    // Verify http_status_code is NOT present in JSON when None
    assert!(!json.contains("http_status_code"));
    
    // Deserialize back to verify roundtrip
    let deserialized: EnvelopeError = serde_json::from_str(&json)
        .expect("Failed to deserialize EnvelopeError");
    
    assert_eq!(deserialized.http_status_code, None);
    assert_eq!(deserialized.code, "GENERIC_ERROR");
}

/// Test that existing JSON without http_status_code field can still be deserialized
#[cfg(any(
    feature = "rest-server", 
    feature = "rest-client",
    feature = "websocket-server", 
    feature = "websocket-client",
    feature = "a2a"
))]
#[test]
fn test_envelope_error_backward_compatibility() {
    // JSON without http_status_code field (old format)
    let json_without_status = r#"{
        "code": "LEGACY_ERROR",
        "message": "This is from old format",
        "details": {"legacy": true},
        "trace": "old_function (old.rs:456)"
    }"#;
    
    // Should deserialize successfully with http_status_code as None
    let deserialized: EnvelopeError = serde_json::from_str(&json_without_status)
        .expect("Failed to deserialize legacy EnvelopeError");
    
    assert_eq!(deserialized.http_status_code, None);
    assert_eq!(deserialized.code, "LEGACY_ERROR");
    assert_eq!(deserialized.message, "This is from old format");
}

/// Test various HTTP status codes are handled correctly
#[cfg(any(
    feature = "rest-server", 
    feature = "rest-client",
    feature = "websocket-server", 
    feature = "websocket-client",
    feature = "a2a"
))]
#[test]
fn test_envelope_error_various_http_status_codes() {
    let test_cases = vec![
        (400, "Bad Request"),
        (401, "Unauthorized"),
        (403, "Forbidden"),
        (404, "Not Found"),
        (500, "Internal Server Error"),
        (503, "Service Unavailable"),
    ];
    
    for (status_code, message) in test_cases {
        let error = EnvelopeError {
            code: format!("HTTP_{}", status_code),
            message: message.to_string(),
            details: None,
            trace: None,
            http_status_code: Some(status_code),
        };
        
        // Test serialization roundtrip
        let json = serde_json::to_string(&error).expect("Failed to serialize");
        let deserialized: EnvelopeError = serde_json::from_str(&json)
            .expect("Failed to deserialize");
        
        assert_eq!(deserialized.http_status_code, Some(status_code));
        assert_eq!(deserialized.code, format!("HTTP_{}", status_code));
    }
}

/// Test that http_status_code field compilation is excluded for non-HTTP protocols
/// This test should only compile when HTTP features are NOT enabled
#[cfg(not(any(
    feature = "rest-server", 
    feature = "rest-client",
    feature = "websocket-server", 
    feature = "websocket-client",
    feature = "a2a"
)))]
#[test]
fn test_envelope_error_no_http_status_field_for_non_http_protocols() {
    // This test verifies that when only gRPC or NATS features are enabled,
    // the http_status_code field is not present in the struct
    let error = EnvelopeError {
        code: "GRPC_ERROR".to_string(),
        message: "gRPC specific error".to_string(),
        details: None,
        trace: None,
        // http_status_code field should not be available for non-HTTP protocols
    };
    
    // Serialize to JSON - should not contain http_status_code
    let json = serde_json::to_string(&error).expect("Failed to serialize EnvelopeError");
    assert!(!json.contains("http_status_code"));
    
    // Verify structure has expected fields
    assert_eq!(error.code, "GRPC_ERROR");
    assert_eq!(error.message, "gRPC specific error");
}

/// Test OpenAPI schema generation includes http_status_code field (if openapi feature enabled)
/// Note: This test verifies that the field compiles and is available when HTTP features are enabled
/// The actual OpenAPI schema integration is tested separately through the main OpenAPI utils
#[cfg(all(
    feature = "openapi",
    any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    )
))]
#[test]
fn test_envelope_error_openapi_schema_includes_http_status_code() {
    // Test that we can create an error with http_status_code when HTTP features are enabled
    let error = EnvelopeError {
        code: "API_ERROR".to_string(),
        message: "Test API error".to_string(),
        details: None,
        trace: None,
        http_status_code: Some(422),
    };
    
    // Verify the field is accessible
    assert_eq!(error.http_status_code, Some(422));
    
    // Test serialization to JSON includes the field
    let json = serde_json::to_string(&error).expect("Should serialize");
    assert!(json.contains("\"http_status_code\":422"));
    
    // Test deserialization roundtrip  
    let deserialized: EnvelopeError = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.http_status_code, Some(422));
}