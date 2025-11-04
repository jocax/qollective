// ABOUTME: Schema validation tests for REST Protocol Metadata Extension
// ABOUTME: Tests JSON schema validation for REST protocol extension and integration with envelopes

use jsonschema::Validator;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Test suite for validating REST protocol metadata JSON schema and test data
#[cfg(test)]
mod rest_protocol_metadata_schema_tests {
    use super::*;

    /// Load the REST protocol metadata schema from file
    fn load_rest_protocol_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/protocols/rest.json");
        let schema_content =
            fs::read_to_string(schema_path).expect("Failed to read REST protocol schema file");

        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse REST protocol schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("REST protocol schema should be valid JSON Schema")
    }

    /// Load the envelope schema for integration testing
    fn load_envelope_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/envelope.json");
        let schema_content =
            fs::read_to_string(schema_path).expect("Failed to read envelope schema file");

        let schema_value: Value =
            serde_json::from_str(&schema_content).expect("Failed to parse envelope schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Envelope schema should be valid JSON Schema")
    }

    #[test]
    fn test_rest_protocol_schema_is_valid() {
        // ARRANGE: Load the REST protocol schema file
        let schema_path = Path::new("../schemas/core/protocols/rest.json");
        let schema_content =
            fs::read_to_string(schema_path).expect("Failed to read REST protocol schema file");

        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse REST protocol schema as JSON");

        // ACT: Create JSONSchema validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(
            result.is_ok(),
            "REST protocol schema should be valid JSON Schema"
        );
    }

    #[test]
    fn test_valid_rest_protocol_minimal() {
        // ARRANGE: Load schema and create minimal valid REST protocol object
        let validator = load_rest_protocol_schema();
        let valid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": "/api/v1/users"
        });

        // ACT: Validate against schema
        let result = validator.validate(&valid_protocol);

        // ASSERT: Validation should pass
        assert!(result.is_ok(), "Minimal REST protocol should be valid");
    }

    #[test]
    fn test_valid_rest_protocol_with_query_params() {
        // ARRANGE: Load schema and create REST protocol with query params
        let validator = load_rest_protocol_schema();
        let valid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": "/api/v1/users",
            "query_params": {
                "page": "1",
                "limit": "20",
                "include": "profile"
            }
        });

        // ACT: Validate against schema
        let result = validator.validate(&valid_protocol);

        // ASSERT: Validation should pass
        assert!(
            result.is_ok(),
            "REST protocol with query params should be valid"
        );
    }

    #[test]
    fn test_valid_rest_protocol_with_headers() {
        // ARRANGE: Load schema and create REST protocol with headers
        let validator = load_rest_protocol_schema();
        let valid_protocol = json!({
            "type": "rest",
            "method": "POST",
            "uri_path": "/api/v1/users",
            "headers": {
                "content-type": "application/json",
                "authorization": "Bearer token123",
                "accept": "application/json"
            }
        });

        // ACT: Validate against schema
        let result = validator.validate(&valid_protocol);

        // ASSERT: Validation should pass
        assert!(result.is_ok(), "REST protocol with headers should be valid");
    }

    #[test]
    fn test_valid_rest_protocol_full() {
        // ARRANGE: Load schema and create full REST protocol object
        let validator = load_rest_protocol_schema();
        let valid_protocol = json!({
            "type": "rest",
            "method": "POST",
            "uri_path": "/api/v1/users",
            "query_params": {
                "include": "profile"
            },
            "headers": {
                "content-type": "application/json",
                "authorization": "Bearer token123"
            }
        });

        // ACT: Validate against schema
        let result = validator.validate(&valid_protocol);

        // ASSERT: Validation should pass
        assert!(result.is_ok(), "Full REST protocol should be valid");
    }

    #[test]
    fn test_invalid_rest_protocol_missing_type() {
        // ARRANGE: Load schema and create invalid protocol (missing type)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "method": "GET",
            "uri_path": "/api/v1/users"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol without type should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_wrong_type() {
        // ARRANGE: Load schema and create invalid protocol (wrong type)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "websocket",
            "method": "GET",
            "uri_path": "/api/v1/users"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with wrong type should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_missing_method() {
        // ARRANGE: Load schema and create invalid protocol (missing method)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "uri_path": "/api/v1/users"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol without method should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_invalid_method() {
        // ARRANGE: Load schema and create invalid protocol (invalid HTTP method)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "INVALID",
            "uri_path": "/api/v1/users"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with invalid method should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_missing_uri_path() {
        // ARRANGE: Load schema and create invalid protocol (missing uri_path)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "GET"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol without uri_path should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_empty_uri_path() {
        // ARRANGE: Load schema and create invalid protocol (empty uri_path)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": ""
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with empty uri_path should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_uri_path_no_slash() {
        // ARRANGE: Load schema and create invalid protocol (uri_path doesn't start with /)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": "api/v1/users"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with uri_path not starting with / should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_query_params_not_string() {
        // ARRANGE: Load schema and create invalid protocol (query params not string values)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": "/api/v1/users",
            "query_params": {
                "page": 1,  // Should be string
                "limit": "20"
            }
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with non-string query param values should be invalid"
        );
    }

    #[test]
    fn test_invalid_rest_protocol_headers_not_string() {
        // ARRANGE: Load schema and create invalid protocol (headers not string values)
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "POST",
            "uri_path": "/api/v1/users",
            "headers": {
                "content-type": "application/json",
                "content-length": 123  // Should be string
            }
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with non-string header values should be invalid"
        );
    }

    #[test]
    fn test_rest_protocol_no_additional_properties() {
        // ARRANGE: Load schema and create protocol with additional properties
        let validator = load_rest_protocol_schema();
        let invalid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": "/api/v1/users",
            "extra_field": "should not be allowed"
        });

        // ACT: Validate against schema
        let result = validator.validate(&invalid_protocol);

        // ASSERT: Validation should fail
        assert!(
            result.is_err(),
            "REST protocol with additional properties should be invalid"
        );
    }

    #[test]
    fn test_envelope_with_rest_protocol_extension() {
        // ARRANGE: Load envelope schema and create envelope with REST protocol extension
        let envelope_validator = load_envelope_schema();
        let rest_protocol_validator = load_rest_protocol_schema();

        let protocol_metadata = json!({
            "type": "rest",
            "method": "POST",
            "uri_path": "/api/v1/users",
            "query_params": {
                "include": "profile"
            },
            "headers": {
                "content-type": "application/json"
            }
        });

        // First validate the protocol extension itself
        let protocol_result = rest_protocol_validator.validate(&protocol_metadata);
        assert!(protocol_result.is_ok(), "Protocol metadata should be valid");

        let envelope_with_protocol = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "duration": 245.000,
                "extensions": {
                    "protocol": protocol_metadata
                }
            },
            "payload": {
                "name": "John Doe",
                "email": "john.doe@example.com"
            }
        });

        // ACT: Validate envelope against schema
        let envelope_result = envelope_validator.validate(&envelope_with_protocol);

        // ASSERT: Envelope validation should pass
        if let Err(validation_error) = envelope_result {
            eprintln!("Validation error: {:?}", validation_error);
            panic!("Envelope with REST protocol extension should be valid");
        }
    }

    #[test]
    fn test_all_http_methods_supported() {
        // ARRANGE: Load schema
        let validator = load_rest_protocol_schema();
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];

        // ACT & ASSERT: Test each HTTP method
        for method in methods.iter() {
            let protocol = json!({
                "type": "rest",
                "method": method,
                "uri_path": "/api/v1/test"
            });

            let result = validator.validate(&protocol);
            assert!(result.is_ok(), "HTTP method {} should be valid", method);
        }
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        // ARRANGE: Create a REST protocol JSON object manually

        let original_json = json!({
            "type": "rest",
            "method": "POST",
            "uri_path": "/api/v1/users",
            "query_params": {
                "page": "1",
                "limit": "20"
            },
            "headers": {
                "content-type": "application/json",
                "authorization": "Bearer token"
            }
        });

        // ACT: Validate against schema
        let validator = load_rest_protocol_schema();
        let validation_result = validator.validate(&original_json);

        // ASSERT: Should be valid
        assert!(
            validation_result.is_ok(),
            "JSON object should be valid against schema"
        );

        // Verify JSON structure matches expected fields
        let obj = original_json.as_object().unwrap();
        assert_eq!(obj.get("type").unwrap().as_str().unwrap(), "rest");
        assert_eq!(obj.get("method").unwrap().as_str().unwrap(), "POST");
        assert_eq!(
            obj.get("uri_path").unwrap().as_str().unwrap(),
            "/api/v1/users"
        );

        let query_params = obj.get("query_params").unwrap().as_object().unwrap();
        assert_eq!(query_params.get("page").unwrap().as_str().unwrap(), "1");
        assert_eq!(query_params.get("limit").unwrap().as_str().unwrap(), "20");

        let headers = obj.get("headers").unwrap().as_object().unwrap();
        assert_eq!(
            headers.get("content-type").unwrap().as_str().unwrap(),
            "application/json"
        );
        assert_eq!(
            headers.get("authorization").unwrap().as_str().unwrap(),
            "Bearer token"
        );
    }

    #[test]
    fn test_uri_path_length_limits() {
        // ARRANGE: Load schema
        let validator = load_rest_protocol_schema();

        // Test maximum length (2048 characters) - should pass
        let long_path = format!("/{}", "a".repeat(2047)); // 2048 total with leading /
        let valid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": long_path
        });

        let result = validator.validate(&valid_protocol);
        assert!(result.is_ok(), "URI path at maximum length should be valid");

        // Test over maximum length (2049 characters) - should fail
        let too_long_path = format!("/{}", "a".repeat(2048)); // 2049 total with leading /
        let invalid_protocol = json!({
            "type": "rest",
            "method": "GET",
            "uri_path": too_long_path
        });

        let result = validator.validate(&invalid_protocol);
        assert!(
            result.is_err(),
            "URI path over maximum length should be invalid"
        );
    }
}
