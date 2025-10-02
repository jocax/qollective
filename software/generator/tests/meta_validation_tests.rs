// ABOUTME: Meta schema validation tests for the Qollective Framework
// ABOUTME: Tests JSON schema validation for all meta sections (security, performance, tracing, debug, monitoring)

use jsonschema::Validator;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Test suite for validating meta schema collection
#[cfg(test)]
mod meta_validation_tests {
    use super::*;

    #[test]
    fn test_security_meta_schema_is_valid() {
        // ARRANGE: Load the security meta schema file
        let schema_path = Path::new("../schemas/core/metadata/security.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read security meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse security meta schema as JSON");

        // ACT: Create Validator validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Security meta schema should be valid JSON Schema");
    }

    #[test]
    fn test_performance_meta_schema_is_valid() {
        // ARRANGE: Load the performance meta schema file
        let schema_path = Path::new("../schemas/core/metadata/performance.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read performance meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse performance meta schema as JSON");

        // ACT: Create Validator validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Performance meta schema should be valid JSON Schema");
    }

    #[test]
    fn test_tracing_meta_schema_is_valid() {
        // ARRANGE: Load the tracing meta schema file
        let schema_path = Path::new("../schemas/core/metadata/tracing.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read tracing meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse tracing meta schema as JSON");

        // ACT: Create Validator validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Tracing meta schema should be valid JSON Schema");
    }

    #[test]
    fn test_debug_meta_schema_is_valid() {
        // ARRANGE: Load the debug meta schema file
        let schema_path = Path::new("../schemas/core/metadata/debug.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read debug meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse debug meta schema as JSON");

        // ACT: Create JSONSchema validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Debug meta schema should be valid JSON Schema");
    }

    #[test]
    fn test_monitoring_meta_schema_is_valid() {
        // ARRANGE: Load the monitoring meta schema file
        let schema_path = Path::new("../schemas/core/metadata/monitoring.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read monitoring meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse monitoring meta schema as JSON");

        // ACT: Create JSONSchema validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Monitoring meta schema should be valid JSON Schema");
    }

    #[test]
    fn test_security_meta_validates_complete_example() {
        // ARRANGE: Load schema and create valid security meta data
        let schema = load_security_meta_schema();
        let valid_security_meta = json!({
            "user_id": "user_12345",
            "session_id": "sess_1NXWnCWxLdhI2RMIxaXYL7vtS3c",
            "auth_method": "jwt",
            "permissions": ["user:read", "user:write"],
            "ip_address": "192.168.1.100",
            "tenant_id": "tenant-123",
            "roles": ["admin", "user"],
            "token_expires_at": "2024-06-07T14:23:15.456Z",
            "security_level": "medium",
            "mfa_verified": true,
            "compliance_flags": ["gdpr"]
        });

        // ACT: Validate the meta data
        let validation_result = schema.validate(&valid_security_meta);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Valid security meta should pass validation");
    }

    #[test]
    fn test_security_meta_validates_user_id_patterns() {
        // ARRANGE: Load schema and test various user ID patterns
        let schema = load_security_meta_schema();
        
        let test_cases = vec![
            ("user_12345", true),
            ("auth0|507f1f77bcf86cd799439011", true),
            ("admin@example.com", true),
            ("simple-user", true),
            ("user.name", true),
            ("user with space", false), // spaces not allowed
            ("", false), // empty string
        ];

        for (user_id, should_be_valid) in test_cases {
            let meta_data = json!({
                "user_id": user_id,
                "auth_method": "jwt"
            });

            // ACT: Validate the meta data
            let validation_result = schema.validate(&meta_data);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "User ID '{}' should be valid", user_id);
            } else {
                assert!(validation_result.is_err(), "User ID '{}' should be invalid", user_id);
            }
        }
    }

    #[test]
    fn test_security_meta_validates_permissions_format() {
        // ARRANGE: Load schema and test permission format validation
        let schema = load_security_meta_schema();
        
        let test_cases = vec![
            (vec!["user:read", "user:write"], true),
            (vec!["admin:manage", "product:delete"], true),
            (vec!["user_service:read"], true),
            (vec!["invalid"], false), // missing colon
            (vec!["User:Read"], false), // uppercase not allowed
            (vec!["user:"], false), // missing action
            (vec![":read"], false), // missing resource
        ];

        for (permissions, should_be_valid) in test_cases {
            let meta_data = json!({
                "user_id": "test_user",
                "permissions": permissions
            });

            // ACT: Validate the meta data
            let validation_result = schema.validate(&meta_data);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "Permissions {:?} should be valid", permissions);
            } else {
                assert!(validation_result.is_err(), "Permissions {:?} should be invalid", permissions);
            }
        }
    }

    #[test]
    fn test_security_meta_validates_ip_addresses() {
        // ARRANGE: Load schema and test IP address validation
        let schema = load_security_meta_schema();
        
        let test_cases = vec![
            ("192.168.1.100", true), // IPv4
            ("10.0.0.1", true), // IPv4
            ("2001:db8::1", true), // IPv6
            ("::1", true), // IPv6 localhost
            ("256.1.1.1", false), // Invalid IPv4
            ("192.168.1", false), // Incomplete IPv4
            ("not-an-ip", false), // Not an IP
        ];

        for (ip_address, should_be_valid) in test_cases {
            let meta_data = json!({
                "user_id": "test_user",
                "ip_address": ip_address
            });

            // ACT: Validate the meta data
            let validation_result = schema.validate(&meta_data);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "IP address '{}' should be valid", ip_address);
            } else {
                assert!(validation_result.is_err(), "IP address '{}' should be invalid", ip_address);
            }
        }
    }

    #[test]
    fn test_security_meta_validates_geo_location() {
        // ARRANGE: Load schema and test geo location validation
        let schema = load_security_meta_schema();
        let meta_with_geo = json!({
            "user_id": "test_user",
            "geo_location": {
                "country": "US",
                "region": "California",
                "city": "San Francisco",
                "coordinates": {
                    "latitude": 37.7749,
                    "longitude": -122.4194
                }
            }
        });

        // ACT: Validate the meta data
        let validation_result = schema.validate(&meta_with_geo);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Meta with geo location should pass validation");
    }

    #[test]
    fn test_security_meta_validates_risk_score_range() {
        // ARRANGE: Load schema and test risk score validation
        let schema = load_security_meta_schema();
        
        let test_cases = vec![
            (0.0, true),
            (50.5, true),
            (100.0, true),
            (-1.0, false), // below minimum
            (101.0, false), // above maximum
        ];

        for (risk_score, should_be_valid) in test_cases {
            let meta_data = json!({
                "user_id": "test_user",
                "risk_score": risk_score
            });

            // ACT: Validate the meta data
            let validation_result = schema.validate(&meta_data);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "Risk score {} should be valid", risk_score);
            } else {
                assert!(validation_result.is_err(), "Risk score {} should be invalid", risk_score);
            }
        }
    }

    #[test]
    fn test_security_meta_validates_compliance_flags() {
        // ARRANGE: Load schema and test compliance flags validation
        let schema = load_security_meta_schema();
        
        let test_cases = vec![
            (vec!["gdpr", "hipaa"], true),
            (vec!["pci_dss"], true),
            (vec!["iso27001", "fips"], true),
            (vec!["invalid_flag"], false), // not in enum
            (vec!["GDPR"], false), // uppercase not allowed
        ];

        for (compliance_flags, should_be_valid) in test_cases {
            let meta_data = json!({
                "user_id": "test_user",
                "compliance_flags": compliance_flags
            });

            // ACT: Validate the meta data
            let validation_result = schema.validate(&meta_data);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "Compliance flags {:?} should be valid", compliance_flags);
            } else {
                assert!(validation_result.is_err(), "Compliance flags {:?} should be invalid", compliance_flags);
            }
        }
    }

    #[test]
    fn test_security_meta_allows_minimal_data() {
        // ARRANGE: Load schema and create minimal security meta
        let schema = load_security_meta_schema();
        let minimal_meta = json!({
            "user_id": "test_user",
            "auth_method": "api_key"
        });

        // ACT: Validate the meta data
        let validation_result = schema.validate(&minimal_meta);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Minimal security meta should pass validation");
    }

    #[test]
    fn test_security_meta_rejects_additional_properties() {
        // ARRANGE: Load schema and create meta with additional properties
        let schema = load_security_meta_schema();
        let meta_with_extra = json!({
            "user_id": "test_user",
            "auth_method": "jwt",
            "unknown_property": "should_not_be_allowed"
        });

        // ACT: Validate the meta data
        let validation_result = schema.validate(&meta_with_extra);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Security meta with additional properties should fail validation");
    }

    #[test]
    fn test_all_meta_schemas_exist_and_compile() {
        // ARRANGE: List of all expected meta schema files
        let meta_schema_files = vec![
            "metadata/security.json",
            "metadata/performance.json",
            "metadata/tracing.json",
            "metadata/debug.json",
            "metadata/monitoring.json",
        ];

        for schema_file in meta_schema_files {
            // ARRANGE: Load each schema file
            let schema_path = Path::new("../schemas/core").join(schema_file);
            let schema_content = fs::read_to_string(&schema_path)
                .expect(&format!("Failed to read {} schema file", schema_file));
            
            let schema_value: Value = serde_json::from_str(&schema_content)
                .expect(&format!("Failed to parse {} schema as JSON", schema_file));

            // ACT: Create Validator validator
            let result = jsonschema::options()
                .should_validate_formats(true)
                .build(&schema_value);

            // ASSERT: Schema compilation should succeed
            assert!(result.is_ok(), "{} schema should be valid JSON Schema", schema_file);
        }
    }

    #[test]
    fn test_meta_schemas_have_required_properties() {
        // ARRANGE: List of meta schema files with their expected properties
        let schema_requirements = vec![
            ("metadata/security.json", vec!["$schema", "$id", "title", "description", "version", "type", "properties"]),
            ("metadata/performance.json", vec!["$schema", "$id", "title", "description", "version", "type", "properties"]),
            ("metadata/tracing.json", vec!["$schema", "$id", "title", "description", "version", "type", "properties"]),
            ("metadata/debug.json", vec!["$schema", "$id", "title", "description", "version", "type", "properties"]),
            ("metadata/monitoring.json", vec!["$schema", "$id", "title", "description", "version", "type", "properties"]),
        ];

        for (schema_file, required_props) in schema_requirements {
            // ARRANGE: Load schema file
            let schema_path = Path::new("../schemas/core").join(schema_file);
            let schema_content = fs::read_to_string(&schema_path)
                .expect(&format!("Failed to read {} schema file", schema_file));
            
            let schema_value: Value = serde_json::from_str(&schema_content)
                .expect(&format!("Failed to parse {} schema as JSON", schema_file));

            // ACT & ASSERT: Check each required property exists
            for prop in required_props {
                assert!(
                    schema_value.get(prop).is_some(),
                    "{} schema should have '{}' property",
                    schema_file,
                    prop
                );
            }
        }
    }

    // Helper functions to load meta schemas
    fn load_security_meta_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/metadata/security.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read security meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse security meta schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile security meta schema")
    }

    fn load_performance_meta_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/metadata/performance.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read performance meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse performance meta schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile performance meta schema")
    }

    fn load_tracing_meta_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/metadata/tracing.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read tracing meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse tracing meta schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile tracing meta schema")
    }

    fn load_debug_meta_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/metadata/debug.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read debug meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse debug meta schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile debug meta schema")
    }

    fn load_monitoring_meta_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/metadata/monitoring.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read monitoring meta schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse monitoring meta schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile monitoring meta schema")
    }
}