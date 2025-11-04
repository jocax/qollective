// ABOUTME: Integration tests for schema parser with real-world schema files
// ABOUTME: Tests end-to-end parsing and validation of actual Qollective schemas

use qollective_tools_lib::schema::{SchemaParser, SchemaValidator};
use std::path::Path;

#[test]
fn test_parse_qollective_extension_schema() {
    let schema_path = "../schemas/core/config.json";
    if !Path::new(schema_path).exists() {
        // Skip test if schema file doesn't exist in test environment
        return;
    }

    let mut parser = SchemaParser::new();
    let schema = parser
        .parse_file(schema_path)
        .expect("Failed to parse qollective schema");

    // Basic structure validation
    assert_eq!(
        schema.title,
        Some("Qollective Extension Schema".to_string())
    );
    assert!(schema.description.is_some());
    assert!(schema.properties.contains_key("qollective"));

    // Validate schema correctness
    let validator = SchemaValidator::new();
    validator
        .validate_schema(&schema)
        .expect("Schema validation failed");
}

#[test]
fn test_parse_envelope_schema() {
    let schema_path = "../schemas/core/envelope.json";
    if !Path::new(schema_path).exists() {
        // Skip test if schema file doesn't exist in test environment
        return;
    }

    let mut parser = SchemaParser::new();
    let schema = parser
        .parse_file(schema_path)
        .expect("Failed to parse envelope schema");

    // Basic structure validation
    assert_eq!(schema.title, Some("Qollective Envelope Schema".to_string()));
    assert!(schema.description.is_some());
    assert!(schema.properties.contains_key("meta"));

    // Check that data and error are in anyOf/properties structure
    assert!(schema.properties.contains_key("data") || !schema.any_of.is_empty());
}

#[test]
fn test_parse_minimal_service_example() {
    let schema_path = "../schemas/examples/minimal-service.json";
    if !Path::new(schema_path).exists() {
        // Skip test if schema file doesn't exist in test environment
        return;
    }

    let mut parser = SchemaParser::new();
    let schema = parser
        .parse_file(schema_path)
        .expect("Failed to parse minimal service schema");

    // Basic structure validation
    assert_eq!(schema.title, Some("Minimal Service Example".to_string()));
    assert!(schema.description.is_some());

    // Validate schema correctness
    let validator = SchemaValidator::new();
    validator
        .validate_schema(&schema)
        .expect("Schema validation failed");
}

#[test]
fn test_schema_with_complex_types() {
    let mut parser = SchemaParser::new();

    // Test complex schema with nested objects, arrays, and references
    let complex_schema = r##"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Complex Test Schema",
        "type": "object",
        "properties": {
            "users": {
                "type": "array",
                "items": {
                    "$ref": "#/definitions/User"
                }
            },
            "metadata": {
                "type": "object",
                "properties": {
                    "version": {
                        "type": "string",
                        "pattern": "^\\d+\\.\\d+\\.\\d+$"
                    },
                    "tags": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["version"]
            }
        },
        "definitions": {
            "User": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "minimum": 1
                    },
                    "name": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 100
                    },
                    "email": {
                        "type": "string",
                        "format": "email"
                    },
                    "role": {
                        "type": "string",
                        "enum": ["admin", "user", "guest"]
                    }
                },
                "required": ["id", "name", "email"]
            }
        },
        "required": ["users"]
    }"##;

    let schema = parser
        .parse_string(complex_schema)
        .expect("Failed to parse complex schema");

    // Validate structure
    assert_eq!(schema.title, Some("Complex Test Schema".to_string()));
    assert!(schema.properties.contains_key("users"));
    assert!(schema.properties.contains_key("metadata"));
    assert!(schema.definitions.contains_key("User"));
    assert_eq!(schema.required, vec!["users"]);

    // Check array with reference
    let users_prop = &schema.properties["users"];
    assert!(users_prop.is_array());
    assert!(users_prop.items.is_some());
    let items_schema = users_prop.items.as_ref().unwrap();
    assert!(items_schema.is_reference());

    // Check user definition
    let user_def = &schema.definitions["User"];
    assert!(user_def.is_object());
    assert_eq!(user_def.required, vec!["id", "name", "email"]);

    // Check enum property
    let role_prop = &user_def.properties["role"];
    assert!(role_prop.is_enum());
    assert_eq!(role_prop.enum_values.len(), 3);

    // Validate schema correctness
    let validator = SchemaValidator::new();
    validator
        .validate_schema(&schema)
        .expect("Complex schema validation failed");
}

#[test]
fn test_schema_validation_errors() {
    let mut parser = SchemaParser::new();

    // Test schema with validation errors
    let invalid_schema = r#"{
        "type": "object",
        "properties": {
            "name": {
                "type": "string"
            }
        },
        "required": ["name", "missing_property"]
    }"#;

    let schema = parser
        .parse_string(invalid_schema)
        .expect("Failed to parse schema");

    let validator = SchemaValidator::new();
    let result = validator.validate_schema(&schema);

    // Should fail validation because required field is not in properties
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("missing_property"));
}

#[test]
fn test_schema_linting() {
    let mut parser = SchemaParser::new();

    // Test schema that should generate linting warnings
    let permissive_schema = r#"{
        "type": "object",
        "properties": {
            "unconstrained_string": {
                "type": "string"
            },
            "unconstrained_object": {
                "type": "object"
            },
            "unconstrained_array": {
                "type": "array"
            }
        }
    }"#;

    let schema = parser
        .parse_string(permissive_schema)
        .expect("Failed to parse schema");

    let validator = SchemaValidator::new();
    let warnings = validator.lint_schema(&schema);

    // Should have warnings for overly permissive types
    assert!(!warnings.is_empty());
    assert!(warnings.iter().any(|w| w.contains("very permissive")));
    assert!(warnings.iter().any(|w| w.contains("without properties")));
    assert!(warnings.iter().any(|w| w.contains("without items")));
}
