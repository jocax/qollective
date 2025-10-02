// ABOUTME: Schema validation test suite for the Qollective Framework
// ABOUTME: Tests JSON schema validation for envelope, service config, and metadata schemas

use jsonschema::Validator;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Test suite for validating JSON schemas and test data against the Qollective envelope schema
#[cfg(test)]
mod schema_validation_tests {
    use super::*;

    #[test]
    fn test_envelope_schema_is_valid() {
        // ARRANGE: Load the envelope schema file
        let schema_path = Path::new("../schemas/core/envelope.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read envelope schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse envelope schema as JSON");

        // ACT: Create JSONSchema validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Envelope schema should be valid JSON Schema");
    }

    #[test]
    fn test_envelope_schema_validates_success_response() {
        // ARRANGE: Load schema and create a valid success response
        let schema = load_envelope_schema();
        let valid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "duration": 245.000
            },
            "data": {
                "user_id": 12345,
                "name": "John Doe",
                "email": "john.doe@example.com"
            }
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&valid_response);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Valid success response should pass validation");
    }

    #[test]
    fn test_envelope_schema_validates_error_response() {
        // ARRANGE: Load schema and create a valid error response
        let schema = load_envelope_schema();
        let valid_error_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:25:30.789Z",
                "requestId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                "version": "1.2.3",
                "duration": 12.500
            },
            "error": {
                "code": "USER_NOT_FOUND",
                "message": "The requested user was not found",
                "status": 404,
                "path": "/api/v1/users/99999",
                "method": "GET"
            }
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&valid_error_response);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Valid error response should pass validation");
    }

    #[test]
    fn test_envelope_schema_rejects_both_data_and_error() {
        // ARRANGE: Load schema and create an invalid response with both data and error
        let schema = load_envelope_schema();
        let invalid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3"
            },
            "data": {"user_id": 123},
            "error": {"code": "ERROR", "message": "Error"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&invalid_response);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Response with both data and error should fail validation");
    }

    #[test]
    fn test_envelope_schema_rejects_neither_data_nor_error() {
        // ARRANGE: Load schema and create an invalid response with neither data nor error
        let schema = load_envelope_schema();
        let invalid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3"
            }
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&invalid_response);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Response with neither data nor error should fail validation");
    }

    #[test]
    fn test_envelope_schema_validates_required_meta_fields() {
        // ARRANGE: Load schema and create response missing required meta fields
        let schema = load_envelope_schema();
        let invalid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z"
                // Missing requestId and version
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&invalid_response);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Response missing required meta fields should fail validation");
    }

    #[test]
    fn test_envelope_schema_validates_timestamp_format() {
        // ARRANGE: Load schema and create response with invalid timestamp
        let schema = load_envelope_schema();
        let invalid_response = json!({
            "meta": {
                "timestamp": "invalid-timestamp",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3"
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&invalid_response);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Response with invalid timestamp format should fail validation");
    }

    #[test]
    fn test_envelope_schema_validates_uuid_format() {
        // ARRANGE: Load schema and create response with invalid UUID
        let schema = load_envelope_schema();
        let invalid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "invalid-uuid",
                "version": "1.2.3"
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&invalid_response);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Response with invalid UUID format should fail validation");
    }

    #[test]
    fn test_envelope_schema_validates_version_format() {
        // ARRANGE: Load schema and create response with invalid version format
        let schema = load_envelope_schema();
        let invalid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "invalid-version"
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&invalid_response);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Response with invalid version format should fail validation");
    }

    #[test]
    fn test_envelope_schema_validates_service_chain() {
        // ARRANGE: Load schema and create response with valid service chain
        let schema = load_envelope_schema();
        let valid_response = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "serviceChain": [
                    {
                        "serviceName": "api-gateway",
                        "serviceVersion": "2.1.0",
                        "requestId": "550e8400-e29b-41d4-a716-446655440001",
                        "timestamp": "2024-06-06T14:23:14.000Z",
                        "duration": 12.300
                    }
                ]
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&valid_response);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Response with valid service chain should pass validation");
    }

    #[test]
    fn test_envelope_schema_validates_optional_meta_sections() {
        // ARRANGE: Load schema and create response with all optional meta sections
        let schema = load_envelope_schema();
        let response_with_all_meta = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "security": {
                    "user_id": "user_12345",
                    "auth_method": "jwt",
                    "permissions": ["user:read", "user:write"]
                },
                "performance": {
                    "db_query_time": 45.123,
                    "db_query_count": 3,
                    "cache_hit_ratio": 0.85
                },
                "monitoring": {
                    "server_id": "web-server-01",
                    "environment": "production"
                },
                "tracing": {
                    "trace_id": "550e8400e29b41d4a716446655440000",
                    "span_id": "b7ad6b7169203331",
                    "sampled": true
                },
                "debug": {
                    "trace_enabled": true,
                    "log_level": "debug"
                }
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&response_with_all_meta);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Response with all optional meta sections should pass validation");
    }

    #[test]
    fn test_envelope_schema_validates_extensions() {
        // ARRANGE: Load schema and create response with extensions
        let schema = load_envelope_schema();
        let response_with_extensions = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "extensions": {
                    "pagination": {
                        "total_items": 1247,
                        "current_page": 2,
                        "total_pages": 125,
                        "page_size": 10,
                        "has_next_page": true,
                        "has_previous_page": true,
                        "items_on_page": 10
                    },
                    "user_service": {
                        "account_tier": "premium",
                        "feature_flags": ["new_ui", "advanced_search"]
                    }
                }
            },
            "data": {"test": "data"}
        });

        // ACT: Validate the response
        let validation_result = schema.validate(&response_with_extensions);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Response with valid extensions should pass validation");
    }

    // Helper function to load and compile the envelope schema
    fn load_envelope_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/envelope.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read envelope schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse envelope schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile envelope schema")
    }
}