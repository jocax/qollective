// ABOUTME: Schema validation error handling tests for the Qollective Framework
// ABOUTME: Tests comprehensive error reporting and detailed validation failure messages

use jsonschema::{ValidationError, Validator};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Test suite for comprehensive schema validation error handling and reporting
#[cfg(test)]
mod validation_error_handling_tests {
    use super::*;

    #[test]
    fn test_envelope_validation_provides_detailed_error_messages() {
        // ARRANGE: Load schema and create invalid envelope
        let schema = load_envelope_schema();
        let invalid_envelope = json!({
            "meta": {
                "timestamp": "invalid-timestamp",
                "requestId": "invalid-uuid",
                "version": "invalid-version"
            },
            "data": {"test": "data"}
        });

        // ACT: Validate and collect errors
        assert!(
            !schema.is_valid(&invalid_envelope),
            "Invalid envelope should fail validation"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&invalid_envelope).collect();
        assert!(!errors.is_empty(), "Should have validation errors");

        // Check that we get specific error messages for each invalid field
        let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();

        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("timestamp") || msg.contains("date-time")),
            "Should have timestamp format error"
        );
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("requestId") || msg.contains("uuid")),
            "Should have requestId format error"
        );
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("version") || msg.contains("pattern")),
            "Should have version format error"
        );
    }

    #[test]
    fn test_envelope_validation_reports_missing_required_fields() {
        // ARRANGE: Load schema and create envelope missing required fields
        let schema = load_envelope_schema();
        let incomplete_envelope = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z"
                // Missing requestId and version
            },
            "data": {"test": "data"}
        });

        // ACT: Validate and collect errors
        assert!(
            !schema.is_valid(&incomplete_envelope),
            "Incomplete envelope should fail validation"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&incomplete_envelope).collect();
        let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();

        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("requestId") || msg.contains("required")),
            "Should report missing requestId field"
        );
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("version") || msg.contains("required")),
            "Should report missing version field"
        );
    }

    #[test]
    fn test_service_config_validation_provides_clear_field_errors() {
        // ARRANGE: Load schema and create invalid service config
        let schema = load_service_config_schema();
        let invalid_config = json!({
            "service": {
                "name": "Invalid-Service-Name",
                "version": "invalid-version",
                "contact": {
                    "email": "not-an-email",
                    "slack": "invalid-channel"
                }
            },
            "qollective": {
                "meta": {
                    "performance": {
                        "sampling_rate": 1.5 // Invalid range
                    }
                }
            }
        });

        // ACT: Validate and collect errors
        assert!(
            !schema.is_valid(&invalid_config),
            "Invalid service config should fail validation"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&invalid_config).collect();
        let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();

        // Check for specific validation errors
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("Invalid-Service-Name") && msg.contains("match")),
            "Should report service name pattern error"
        );
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("invalid-version") && msg.contains("match")),
            "Should report version pattern error"
        );
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("email") || msg.contains("format")),
            "Should report email format error"
        );
        assert!(
            error_messages
                .iter()
                .any(|msg| msg.contains("sampling_rate") || msg.contains("maximum")),
            "Should report sampling rate range error"
        );
    }

    #[test]
    fn test_security_meta_validation_reports_permission_format_errors() {
        // ARRANGE: Load schema and create invalid security meta
        let schema = load_security_meta_schema();
        let invalid_security = json!({
            "user_id": "valid_user",
            "permissions": [
                "valid:permission",
                "invalid-permission-format",
                "INVALID:FORMAT",
                ":missing_resource",
                "missing_action:"
            ]
        });

        // ACT: Validate and collect errors
        assert!(
            !schema.is_valid(&invalid_security),
            "Invalid security meta should fail validation"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&invalid_security).collect();
        assert!(!errors.is_empty(), "Should have permission format errors");

        // Each invalid permission should generate an error
        let error_count = errors.len();
        assert!(
            error_count >= 4,
            "Should have at least 4 permission format errors, got {}",
            error_count
        );
    }

    #[test]
    fn test_validation_error_contains_instance_path() {
        // ARRANGE: Load schema and create nested invalid data
        let schema = load_envelope_schema();
        let nested_invalid = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "serviceChain": [
                    {
                        "serviceName": "valid-service",
                        "serviceVersion": "1.0.0",
                        "requestId": "550e8400-e29b-41d4-a716-446655440001",
                        "timestamp": "2024-06-06T14:23:14.000Z"
                    },
                    {
                        "serviceName": "Invalid-Service-Name", // Invalid pattern
                        "serviceVersion": "invalid-version", // Invalid pattern
                        "requestId": "invalid-uuid", // Invalid format
                        "timestamp": "invalid-timestamp" // Invalid format
                    }
                ]
            },
            "data": {"test": "data"}
        });

        // ACT: Validate and collect errors
        // ASSERT: Should provide path information for nested errors
        assert!(
            !schema.is_valid(&nested_invalid),
            "Nested invalid data should fail validation"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&nested_invalid).collect();
        assert!(!errors.is_empty(), "Should have nested validation errors");

        // Check that error paths point to the correct nested locations
        let has_service_chain_errors = errors
            .iter()
            .any(|error| error.instance_path.to_string().contains("serviceChain"));

        assert!(
            has_service_chain_errors,
            "Should have errors with serviceChain path information"
        );
    }

    #[test]
    fn test_validation_error_schema_path_information() {
        // ARRANGE: Load schema and create data that violates specific schema constraints
        let schema = load_envelope_schema();
        let constraint_violation = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "duration": -5.0 // Violates minimum constraint
            },
            "data": {"test": "data"}
        });

        // ACT: Validate and collect errors
        // ASSERT: Should provide schema path information
        assert!(
            !schema.is_valid(&constraint_violation),
            "Constraint violation should fail validation"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&constraint_violation).collect();
        assert!(
            !errors.is_empty(),
            "Should have constraint violation errors"
        );

        // Check that errors include schema path information
        let has_schema_path = errors
            .iter()
            .any(|error| !error.schema_path.to_string().is_empty());

        assert!(
            has_schema_path,
            "Errors should include schema path information"
        );
    }

    #[test]
    fn test_multiple_schema_validation_error_aggregation() {
        // ARRANGE: Load multiple schemas and test error aggregation
        let envelope_schema = load_envelope_schema();
        let service_config_schema = load_service_config_schema();
        let security_meta_schema = load_security_meta_schema();

        let test_data = vec![
            ("envelope", &envelope_schema, json!({"invalid": "data"})),
            (
                "service_config",
                &service_config_schema,
                json!({"invalid": "config"}),
            ),
            (
                "security_meta",
                &security_meta_schema,
                json!({"invalid": "security"}),
            ),
        ];

        // ACT & ASSERT: Validate each and collect error information
        for (schema_name, schema, data) in test_data {
            assert!(
                !schema.is_valid(&data),
                "{} schema should reject invalid data",
                schema_name
            );

            let errors: Vec<ValidationError> = schema.iter_errors(&data).collect();
            assert!(
                !errors.is_empty(),
                "{} schema should produce validation errors",
                schema_name
            );

            // Ensure errors are meaningful and not just generic
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();

            assert!(
                error_messages.iter().all(|msg| !msg.is_empty()),
                "{} schema errors should have non-empty messages",
                schema_name
            );
        }
    }

    #[test]
    fn test_validation_error_provides_helpful_suggestions() {
        // ARRANGE: Load schema and create common mistake scenarios
        let schema = load_envelope_schema();

        let common_mistakes = vec![
            // Wrong property name (case sensitivity)
            json!({
                "meta": {
                    "Timestamp": "2024-06-06T14:23:15.456Z", // Wrong case
                    "requestId": "550e8400-e29b-41d4-a716-446655440000",
                    "version": "1.2.3"
                },
                "data": {"test": "data"}
            }),
            // Both data and error present
            json!({
                "meta": {
                    "timestamp": "2024-06-06T14:23:15.456Z",
                    "requestId": "550e8400-e29b-41d4-a716-446655440000",
                    "version": "1.2.3"
                },
                "data": {"test": "data"},
                "error": {"code": "ERROR", "message": "Error"}
            }),
        ];

        for (i, mistake_data) in common_mistakes.iter().enumerate() {
            // ACT: Validate the mistake
            // ASSERT: Should fail with informative errors
            assert!(
                !schema.is_valid(mistake_data),
                "Common mistake {} should fail validation",
                i
            );

            let errors: Vec<ValidationError> = schema.iter_errors(mistake_data).collect();
            assert!(
                !errors.is_empty(),
                "Common mistake {} should have errors",
                i
            );

            // Errors should provide some context about what went wrong
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();

            assert!(
                error_messages.iter().any(|msg| msg.len() > 10),
                "Common mistake {} should have detailed error messages",
                i
            );
        }
    }

    #[test]
    fn test_validation_performance_with_large_errors() {
        // ARRANGE: Load schema and create data with many validation errors
        let schema = load_envelope_schema();
        let many_errors_data = json!({
            "meta": {
                "timestamp": "invalid",
                "requestId": "invalid",
                "version": "invalid",
                "duration": -1,
                "serviceChain": (0..10).map(|i| json!({
                    "serviceName": format!("Invalid-Service-{}", i),
                    "serviceVersion": "invalid",
                    "requestId": "invalid",
                    "timestamp": "invalid",
                    "duration": -1
                })).collect::<Vec<_>>()
            },
            "data": {"test": "data"},
            "error": {"code": "ERROR", "message": "Error"} // Also invalid (both data and error)
        });

        // ACT: Validate and measure error collection
        let start = std::time::Instant::now();
        let is_valid = schema.is_valid(&many_errors_data);
        let validation_duration = start.elapsed();

        // ASSERT: Should complete validation in reasonable time
        assert!(!is_valid, "Data with many errors should fail validation");
        assert!(
            validation_duration.as_millis() < 1000,
            "Validation should complete within 1 second"
        );

        let errors: Vec<ValidationError> = schema.iter_errors(&many_errors_data).collect();
        assert!(
            errors.len() > 10,
            "Should collect multiple validation errors"
        );
    }

    // Helper functions to load schemas
    fn load_envelope_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/envelope.json");
        let schema_content =
            fs::read_to_string(schema_path).expect("Failed to read envelope schema file");

        let schema_value: Value =
            serde_json::from_str(&schema_content).expect("Failed to parse envelope schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile envelope schema")
    }

    fn load_service_config_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/service-config.json");
        let schema_content =
            fs::read_to_string(schema_path).expect("Failed to read service config schema file");

        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse service config schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile service config schema")
    }

    fn load_security_meta_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/metadata/security.json");
        let schema_content =
            fs::read_to_string(schema_path).expect("Failed to read security meta schema file");

        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse security meta schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile security meta schema")
    }
}
