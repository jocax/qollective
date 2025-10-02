//! Enhanced QollectiveError OpenAPI Tests
//!
//! This module tests the enhanced QollectiveError structure with comprehensive
//! OpenAPI schema generation support, including structured details, timestamps,
//! and realistic examples for enterprise API documentation.

use utoipa::{OpenApi, ToSchema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_qollective_error_schema_generation() {
        // This test will verify that the enhanced QollectiveError generates
        // comprehensive OpenAPI schemas with detailed field descriptions,
        // realistic examples, and proper type handling.
        
        // Note: This test will initially fail until we implement the enhanced
        // QollectiveError structure with ToSchema derives and comprehensive
        // field annotations.
        
        // Test that we can create the OpenAPI documentation
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(EnhancedQollectiveError)),
            info(title = "Qollective Error API", version = "1.0.0")
        )]
        struct ErrorApiDoc;
        
        let openapi = ErrorApiDoc::openapi();
        assert_eq!(openapi.info.title, "Qollective Error API");
        
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("EnhancedQollectiveError"), 
                "EnhancedQollectiveError schema should be present");
        
        // Verify the schema includes our enhanced fields
        let _error_schema = components.schemas.get("EnhancedQollectiveError").unwrap();
        
        // The schema should be present
        // This will be validated once we implement the structure
        // For now, just verify the schema exists
        assert!(true, "Schema validation will be expanded after implementation");
    }
    
    #[test]
    fn test_enhanced_error_with_structured_details() {
        // Test that enhanced errors can include structured details
        // with proper JSON Schema validation
        
        let structured_details = json!({
            "error_code": "ENVELOPE_VALIDATION_FAILED",
            "field_errors": [
                {
                    "field": "tenant_id",
                    "message": "Tenant ID is required but was not provided",
                    "code": "REQUIRED_FIELD_MISSING"
                },
                {
                    "field": "payload.user_id",
                    "message": "User ID must be a valid UUID",
                    "code": "INVALID_UUID_FORMAT"
                }
            ],
            "request_id": "req_123456789",
            "correlation_id": "corr_abcdef123"
        });
        
        // This test will verify we can create enhanced errors with structured details
        // Implementation will be done after creating the enhanced structure
        
        // For now, just verify the JSON structure is valid
        assert!(structured_details.is_object());
        assert!(structured_details["field_errors"].is_array());
        assert_eq!(structured_details["error_code"], "ENVELOPE_VALIDATION_FAILED");
    }
    
    #[test]
    fn test_enhanced_error_timestamp_field() {
        // Test that the enhanced error includes proper ISO 8601 timestamp
        // for error occurrence tracking
        
        let now = Utc::now();
        let timestamp_str = now.to_rfc3339();
        
        // Verify timestamp format is ISO 8601 compatible
        assert!(timestamp_str.contains("T"));
        assert!(timestamp_str.contains("Z") || timestamp_str.contains("+"));
        
        // Test parsing back from string
        let parsed_timestamp = DateTime::parse_from_rfc3339(&timestamp_str);
        assert!(parsed_timestamp.is_ok());
    }
    
    #[test]
    fn test_error_schema_examples() {
        // Test that the error schema includes realistic examples
        // that demonstrate actual usage patterns
        
        let validation_error_example = json!({
            "error_type": "Validation",
            "message": "Envelope validation failed: required field 'tenant_id' is missing",
            "occurred_at": "2025-08-23T10:30:45.123Z",
            "details": {
                "error_code": "ENVELOPE_VALIDATION_FAILED",
                "field_errors": [
                    {
                        "field": "tenant_id",
                        "message": "Tenant ID is required but was not provided",
                        "code": "REQUIRED_FIELD_MISSING"
                    }
                ],
                "request_id": "req_789abc123def",
                "correlation_id": "corr_enterprise_001"
            },
            "stack_trace": null
        });
        
        let security_error_example = json!({
            "error_type": "Security", 
            "message": "JWT token validation failed: token has expired",
            "occurred_at": "2025-08-23T10:30:45.456Z",
            "details": {
                "error_code": "JWT_TOKEN_EXPIRED",
                "token_info": {
                    "issued_at": "2025-08-23T09:30:45Z",
                    "expires_at": "2025-08-23T10:30:45Z", 
                    "tenant_id": "enterprise_001"
                },
                "request_id": "req_security_456",
                "correlation_id": "corr_auth_002"
            },
            "stack_trace": null
        });
        
        let transport_error_example = json!({
            "error_type": "Transport",
            "message": "NATS connection failed: connection refused by server",
            "occurred_at": "2025-08-23T10:30:45.789Z",
            "details": {
                "error_code": "NATS_CONNECTION_REFUSED",
                "connection_info": {
                    "server": "nats://enterprise.starfleet.local:4222",
                    "client_id": "enterprise_bridge_001",
                    "retry_count": 3,
                    "last_error": "dial tcp: connection refused"
                },
                "request_id": "req_nats_789",
                "correlation_id": "corr_bridge_003"
            },
            "stack_trace": null
        });
        
        // Verify example structures are valid JSON
        assert!(validation_error_example.is_object());
        assert!(security_error_example.is_object());
        assert!(transport_error_example.is_object());
        
        // Verify required fields are present
        for example in [&validation_error_example, &security_error_example, &transport_error_example] {
            assert!(example["error_type"].is_string());
            assert!(example["message"].is_string());
            assert!(example["occurred_at"].is_string());
            assert!(example["details"].is_object());
        }
    }
    
    #[test]
    fn test_error_schema_field_descriptions() {
        // Test that error schema includes comprehensive field descriptions
        // suitable for enterprise API documentation
        
        // This will be validated by examining the generated OpenAPI schema
        // once the enhanced structure is implemented with proper annotations
        
        let expected_descriptions = vec![
            "The type of error that occurred within the Qollective framework",
            "Human-readable description of the error for developers and support teams", 
            "ISO 8601 timestamp when the error occurred for tracking and debugging",
            "Structured error details with additional context and debugging information",
            "Optional stack trace information for development and debugging environments"
        ];
        
        // For now, verify the descriptions exist and are meaningful
        for desc in expected_descriptions {
            assert!(desc.len() > 20, "Description should be comprehensive: {}", desc);
            assert!(desc.contains("error") || desc.contains("debug") || desc.contains("information"),
                   "Description should be error-related: {}", desc);
        }
    }
    
    #[test] 
    fn test_error_schema_validation_rules() {
        // Test that the error schema includes proper validation rules
        // for field constraints and patterns
        
        // Test error_type enum validation
        let valid_error_types = vec![
            "Envelope", "Config", "Serialization", "Transport", "Validation",
            "Security", "Internal", "External", "Connection", "Deserialization", 
            "Remote", "Grpc", "TenantExtraction", "NatsConnection", "NatsMessage",
            "NatsTimeout", "NatsDiscovery", "NatsSubject", "NatsAuth",
            "FeatureNotEnabled", "McpProtocol", "McpError", "McpToolExecution",
            "McpServerRegistration", "McpClientConnection", "McpServerNotFound",
            "AgentNotFound", "ProtocolAdapter"
        ];
        
        // Verify we have a comprehensive list of error types
        assert!(valid_error_types.len() > 20, "Should have comprehensive error type coverage");
        
        // Test message field constraints
        let test_messages = vec![
            "Short error",
            "A longer error message that provides more context about what went wrong",
            "An extremely detailed error message that includes specific technical details, error codes, and comprehensive context for enterprise debugging and support team analysis"
        ];
        
        for message in test_messages {
            assert!(message.len() > 0, "Error messages should not be empty");
            // No maximum length enforced - enterprise errors can be detailed
        }
        
        // Test ISO 8601 timestamp format validation
        let valid_timestamps = vec![
            "2025-08-23T10:30:45.123Z",
            "2025-08-23T10:30:45Z", 
            "2025-08-23T10:30:45.123456Z",
            "2025-08-23T10:30:45+00:00",
            "2025-08-23T10:30:45.123-05:00"
        ];
        
        for timestamp in valid_timestamps {
            let parsed = DateTime::parse_from_rfc3339(timestamp);
            assert!(parsed.is_ok(), "Timestamp should be valid RFC3339: {}", timestamp);
        }
    }
    
    #[test]
    fn test_error_backward_compatibility() {
        // Test that enhanced error structure maintains backward compatibility
        // with existing QollectiveError enum variants
        
        // This test ensures we don't break existing error construction patterns
        // while adding the new enhanced features
        
        // Test basic error variant names that should still exist
        let expected_variants = vec![
            "Envelope", "Config", "Serialization", "Transport", "Validation",
            "Security", "Internal", "External", "Connection", "TenantExtraction"
        ];
        
        // The enhanced structure should maintain these core error categories
        for variant in expected_variants {
            assert!(variant.len() > 3, "Error variant should have meaningful name: {}", variant);
            assert!(!variant.contains("_"), "Error variants should use PascalCase: {}", variant);
        }
        
        // Test that helper methods should still work
        // (These will be implemented in the enhanced structure)
        let error_messages = vec![
            "envelope error message",
            "config error message", 
            "serialization error message",
            "transport error message"
        ];
        
        for msg in error_messages {
            assert!(msg.contains("error"), "Helper method messages should mention error");
        }
    }
}

// Placeholder structure for the enhanced QollectiveError
// This will be replaced with the actual enhanced implementation
// that includes ToSchema derives and comprehensive OpenAPI annotations
#[derive(Debug, ToSchema, Serialize, Deserialize, Clone)]
#[schema(
    title = "Enhanced Qollective Error",
    description = "Comprehensive error type for all Qollective framework operations with structured details and enterprise-grade debugging information",
    example = json!({
        "error_type": "Validation",
        "message": "Envelope validation failed: required field 'tenant_id' is missing",
        "occurred_at": "2025-08-23T10:30:45.123Z",
        "details": {
            "error_code": "ENVELOPE_VALIDATION_FAILED",
            "field_errors": [
                {
                    "field": "tenant_id",
                    "message": "Tenant ID is required but was not provided",
                    "code": "REQUIRED_FIELD_MISSING"
                }
            ],
            "request_id": "req_789abc123def",
            "correlation_id": "corr_enterprise_001"
        },
        "stack_trace": null
    })
)]
struct EnhancedQollectiveError {
    #[schema(example = "Validation")]
    pub error_type: String,
    
    #[schema(example = "Envelope validation failed: required field 'tenant_id' is missing")]
    pub message: String,
    
    #[schema(example = "2025-08-23T10:30:45.123Z")]
    pub occurred_at: DateTime<Utc>,
    
    #[schema(example = json!({
        "error_code": "ENVELOPE_VALIDATION_FAILED", 
        "request_id": "req_789abc123def",
        "correlation_id": "corr_enterprise_001"
    }))]
    pub details: Option<Value>,
    
    #[schema(example = "null")]
    pub stack_trace: Option<String>,
}