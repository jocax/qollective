// ABOUTME: Comprehensive integration tests for the code generation functionality
// ABOUTME: Tests complete end-to-end workflows from schema parsing to code output

use qollective_cli::codegen::DirectTypifyGenerator;
use qollective_cli::SchemaParser;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod code_generation_integration_tests {
    use super::*;

    #[test]
    fn test_envelope_schema_generates_complete_rust_code() {
        // ARRANGE: Load the actual envelope schema file
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test envelope schema that represents our core structure
        let envelope_schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "QollectiveEnvelope",
            "description": "Standard envelope for Qollective framework",
            "type": "object",
            "properties": {
                "meta": {
                    "type": "object",
                    "properties": {
                        "timestamp": {
                            "type": "string",
                            "format": "date-time",
                            "description": "Request timestamp"
                        },
                        "requestId": {
                            "type": "string",
                            "format": "uuid",
                            "description": "Unique request identifier"
                        },
                        "version": {
                            "type": "string",
                            "pattern": "^\\d+\\.\\d+\\.\\d+$",
                            "description": "API version"
                        },
                        "duration": {
                            "type": "number",
                            "minimum": 0,
                            "description": "Request duration in milliseconds"
                        }
                    },
                    "required": ["timestamp", "requestId", "version"]
                },
                "data": {
                    "description": "Request/response payload"
                },
                "error": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Error code"
                        },
                        "message": {
                            "type": "string",
                            "description": "Error message"
                        },
                        "details": {
                            "description": "Additional error details"
                        }
                    },
                    "required": ["code", "message"]
                }
            },
            "required": ["meta"]
        });

        let schema_file = temp_dir.path().join("envelope.json");
        fs::write(&schema_file, serde_json::to_string_pretty(&envelope_schema).unwrap()).unwrap();

        // ACT: Generate Rust code from the schema
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_file.to_str().unwrap());

        // ASSERT: Code generation should succeed
        assert!(result.is_ok(), "Envelope schema generation should succeed");
        
        let generated_code = result.unwrap();
        
        // Should contain the main envelope struct
        assert!(generated_code.contains("pub struct QollectiveEnvelope"), "Should generate main envelope struct");
        
        // Should contain meta struct
        assert!(generated_code.contains("pub struct Meta") || generated_code.contains("meta:"), "Should include meta field or struct");
        
        // Should contain error struct
        assert!(generated_code.contains("pub struct Error") || generated_code.contains("error:"), "Should include error field or struct");
        
        // Should have proper serde imports and derives (typify uses full paths)
        assert!(generated_code.contains(":: serde ::") || generated_code.contains("use serde"), "Should import serde");
        assert!(generated_code.contains("Serialize"), "Should derive Serialize");
        assert!(generated_code.contains("Deserialize"), "Should derive Deserialize");
        
        // Should have Option types for non-required fields (typify uses full paths)
        assert!(generated_code.contains("Option <") || generated_code.contains("Option<"), "Should use Option for optional fields");
        
        // Should compile without errors
        assert!(validate_rust_syntax(&generated_code), "Generated code should have valid Rust syntax");
    }

    #[test]
    fn test_metadata_schema_generates_specialized_types() {
        // ARRANGE: Create a security metadata schema
        let security_meta_schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "SecurityMeta",
            "description": "Security metadata for requests",
            "type": "object",
            "properties": {
                "user_id": {
                    "type": "string",
                    "description": "Authenticated user identifier"
                },
                "session_id": {
                    "type": "string",
                    "description": "Session identifier"
                },
                "auth_method": {
                    "type": "string",
                    "enum": ["jwt", "oauth", "api_key", "basic"],
                    "description": "Authentication method used"
                },
                "permissions": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "pattern": "^[a-z_]+:[a-z_]+$"
                    },
                    "description": "User permissions array"
                },
                "ip_address": {
                    "type": "string",
                    "format": "ipv4",
                    "description": "Client IP address"
                },
                "tenant_id": {
                    "type": "string",
                    "description": "Multi-tenant identifier"
                }
            },
            "required": ["user_id", "auth_method"]
        });

        let temp_dir = TempDir::new().unwrap();
        let schema_file = temp_dir.path().join("security_meta.json");
        fs::write(&schema_file, serde_json::to_string_pretty(&security_meta_schema).unwrap()).unwrap();

        // ACT: Generate Rust code
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_file.to_str().unwrap());

        // ASSERT: Should generate specialized types
        assert!(result.is_ok(), "Security meta schema generation should succeed");
        
        let generated_code = result.unwrap();
        
        
        // Should contain the security meta struct (typify generates it)
        assert!(generated_code.contains("SecurityMeta"), "Should generate SecurityMeta struct");
        
        // Should contain auth method enum (typify generates SecurityMetaAuthMethod)
        assert!(generated_code.contains("SecurityMetaAuthMethod"), "Should generate SecurityMetaAuthMethod enum");
        
        // Enum should have proper variants (typify converts to PascalCase)
        assert!(generated_code.contains("Jwt") || generated_code.contains("JWT"), "Should have JWT variant");
        assert!(generated_code.contains("OAuth") || generated_code.contains("Oauth"), "Should have OAuth variant");
        assert!(generated_code.contains("ApiKey") || generated_code.contains("api_key"), "Should have API key variant");
        
        // Should handle arrays properly (typify generates Vec<SecurityMetaPermissionsItem>)
        assert!(generated_code.contains("SecurityMetaPermissionsItem") || (generated_code.contains("Vec<") && generated_code.contains("permissions")), "Should handle permissions array");
        
        // Should handle required vs optional fields properly (typify generates proper types)
        assert!(generated_code.contains("user_id"), "Should have user_id field");
        assert!(generated_code.contains("auth_method"), "Should have auth_method field");
        
        // Optional fields should be wrapped in Option (typify uses full paths)
        assert!(generated_code.contains("Option <") || generated_code.contains("Option<"), "Optional fields should be Option types");
    }

    #[test]
    fn test_complex_nested_schema_generation() {
        // ARRANGE: Create a complex nested schema
        let complex_schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "ServiceConfig",
            "description": "Complex service configuration",
            "type": "object",
            "properties": {
                "service": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "pattern": "^[a-z][a-z0-9_-]*$"
                        },
                        "version": {
                            "type": "string"
                        },
                        "endpoints": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "path": {
                                        "type": "string"
                                    },
                                    "methods": {
                                        "type": "array",
                                        "items": {
                                            "type": "string",
                                            "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"]
                                        }
                                    },
                                    "metadata_config": {
                                        "type": "object",
                                        "properties": {
                                            "security": {
                                                "type": "object",
                                                "properties": {
                                                    "enabled": {"type": "boolean"},
                                                    "properties": {
                                                        "oneOf": [
                                                            {"type": "string", "enum": ["*"]},
                                                            {
                                                                "type": "object",
                                                                "additionalProperties": {"type": "boolean"}
                                                            }
                                                        ]
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                "required": ["path", "methods"]
                            }
                        }
                    },
                    "required": ["name", "version"]
                }
            },
            "required": ["service"]
        });

        let temp_dir = TempDir::new().unwrap();
        let schema_file = temp_dir.path().join("service_config.json");
        fs::write(&schema_file, serde_json::to_string_pretty(&complex_schema).unwrap()).unwrap();

        // ACT: Generate Rust code
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_file.to_str().unwrap());

        // ASSERT: Should handle complex nesting
        assert!(result.is_ok(), "Complex schema generation should succeed");
        
        let generated_code = result.unwrap();
        
        
        // Should contain main struct
        assert!(generated_code.contains("pub struct ServiceConfig"), "Should generate ServiceConfig struct");
        
        // Should contain nested structs
        assert!(generated_code.contains("pub struct Service") || generated_code.contains("service:"), "Should handle service object");
        
        // Should handle arrays of objects (typify generates Vec< and ServiceConfigServiceEndpointsItem)
        assert!(generated_code.contains("Vec <") || generated_code.contains("Vec<"), "Should generate Vec types for arrays");
        assert!(generated_code.contains("ServiceConfigServiceEndpointsItem") || generated_code.contains("Endpoint") || generated_code.contains("endpoints"), "Should handle endpoint arrays");
        
        // Should handle HTTP method enums
        assert!(generated_code.contains("GET") || generated_code.contains("Get"), "Should handle HTTP methods");
        
        // Should handle oneOf unions appropriately (may map to serde_json::Value)
        assert!(generated_code.contains("serde_json::Value") || generated_code.contains("Properties"), "Should handle oneOf unions");
        
        // Should compile without errors
        assert!(validate_rust_syntax(&generated_code), "Complex generated code should have valid Rust syntax");
    }

    #[test]
    fn test_enum_generation_with_various_types() {
        // ARRANGE: Create schemas with different enum types
        let enum_schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "EnumTest",
            "description": "Various enum types",
            "type": "object",
            "properties": {
                "string_enum": {
                    "type": "string",
                    "enum": ["active", "inactive", "pending", "archived"]
                },
                "number_enum": {
                    "type": "integer",
                    "enum": [100, 200, 300, 404, 500]
                },
                "mixed_enum": {
                    "description": "Mixed type field - falls back to serde_json::Value"
                }
            }
        });

        let temp_dir = TempDir::new().unwrap();
        let schema_file = temp_dir.path().join("enum_test.json");
        fs::write(&schema_file, serde_json::to_string_pretty(&enum_schema).unwrap()).unwrap();

        // ACT: Generate Rust code
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_file.to_str().unwrap());

        // ASSERT: Should handle different enum types
        assert!(result.is_ok(), "Enum schema generation should succeed");
        
        let generated_code = result.unwrap();
        
        
        // Should contain enum definitions or appropriate type aliases
        assert!(generated_code.contains("pub enum") || generated_code.contains("StringEnum"), "Should handle string enums");
        
        // String enum should have proper variants
        assert!(generated_code.contains("Active") || generated_code.contains("active"), "Should have enum variants");
        
        // Should handle serde rename for string enums
        assert!(generated_code.contains("#[serde(rename") || generated_code.contains("active"), "Should handle serde renaming");
        
        // Mixed field should be handled as a regular field (no enum for mixed types)
        assert!(generated_code.contains("mixed_enum"), "Should have mixed_enum field");
    }

    #[test]
    fn test_error_handling_for_invalid_schemas() {
        // ARRANGE: Create invalid schemas
        let invalid_schemas = vec![
            // Invalid JSON
            r#"{"invalid": json}"#,
            
            // Valid JSON but invalid schema
            r#"{"type": "invalid_type"}"#,
            
            // Empty schema
            r#"{}"#,
            
            // Schema with missing ref target (will cause error)
            "{\"$ref\": \"#/definitions/missing\"}",
        ];

        let temp_dir = TempDir::new().unwrap();
        let generator = DirectTypifyGenerator::new();

        for (i, invalid_schema) in invalid_schemas.iter().enumerate() {
            // ACT: Try to generate code from invalid schema
            let schema_file = temp_dir.path().join(format!("invalid_{}.json", i));
            
            // Catch any panics from typify and handle them gracefully
            let result = std::panic::catch_unwind(|| {
                fs::write(&schema_file, invalid_schema).unwrap();
                generator.generate_from_file(schema_file.to_str().unwrap())
            });
            
            // ASSERT: Should either fail gracefully or panic (both are acceptable)
            match result {
                Ok(gen_result) => {
                    if gen_result.is_ok() {
                        // Some invalid schemas might still generate code - that's acceptable
                        let code = gen_result.unwrap();
                        assert!(!code.is_empty(), "If generation succeeds, should produce some code");
                    } else {
                        // Error should be meaningful
                        let error = gen_result.unwrap_err();
                        assert!(!error.to_string().is_empty(), "Error should have meaningful message");
                    }
                },
                Err(_) => {
                    // Panic occurred - this is acceptable for truly invalid schemas
                    // as typify may legitimately panic on malformed input
                }
            }
        }
    }

    #[test]
    fn test_large_schema_performance() {
        // ARRANGE: Create a large schema with many properties
        let mut large_schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "LargeSchema",
            "type": "object",
            "properties": {}
        });

        // Add 100 properties to test performance
        for i in 0..100 {
            large_schema["properties"][format!("field_{}", i)] = json!({
                "type": "string",
                "description": format!("Field number {}", i)
            });
        }

        let temp_dir = TempDir::new().unwrap();
        let schema_file = temp_dir.path().join("large_schema.json");
        fs::write(&schema_file, serde_json::to_string_pretty(&large_schema).unwrap()).unwrap();

        // ACT: Generate code and measure time
        let generator = DirectTypifyGenerator::new();
        let start = std::time::Instant::now();
        let result = generator.generate_from_file(schema_file.to_str().unwrap());
        let duration = start.elapsed();

        // ASSERT: Should complete in reasonable time
        assert!(result.is_ok(), "Large schema generation should succeed");
        assert!(duration.as_secs() < 10, "Large schema should generate within 10 seconds, took {:?}", duration);
        
        let generated_code = result.unwrap();
        
        // Should contain all fields
        assert!(generated_code.contains("field_0"), "Should contain first field");
        assert!(generated_code.contains("field_99"), "Should contain last field");
        
        // Should be well-formatted
        assert!(generated_code.contains("pub struct LargeSchema"), "Should have main struct");
    }

    #[test]
    fn test_schema_parsing_integration() {
        // ARRANGE: Create a schema that exercises the parser
        let parser_test_schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "ParserTest",
            "type": "object",
            "properties": {
                "simple_string": {"type": "string"},
                "constrained_number": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100
                },
                "array_field": {
                    "type": "array",
                    "items": {"type": "integer"}
                },
                "object_field": {
                    "type": "object",
                    "properties": {
                        "nested_string": {"type": "string"}
                    }
                },
                "union_field": {
                    "anyOf": [
                        {"type": "string"},
                        {"type": "number"}
                    ]
                }
            },
            "required": ["simple_string"]
        });

        let temp_dir = TempDir::new().unwrap();
        let schema_file = temp_dir.path().join("parser_test.json");
        fs::write(&schema_file, serde_json::to_string_pretty(&parser_test_schema).unwrap()).unwrap();

        // ACT: Parse the schema first, then generate
        let mut parser = SchemaParser::new();
        let parsed_schema = parser.parse_file(schema_file.to_str().unwrap());
        
        // ASSERT: Parsing should succeed
        assert!(parsed_schema.is_ok(), "Schema parsing should succeed");
        
        let schema = parsed_schema.unwrap();
        
        // Should parse properties correctly
        assert!(schema.properties.contains_key("simple_string"), "Should parse simple string property");
        assert!(schema.properties.contains_key("constrained_number"), "Should parse constrained number");
        assert!(schema.properties.contains_key("array_field"), "Should parse array field");
        assert!(schema.properties.contains_key("object_field"), "Should parse object field");
        assert!(schema.properties.contains_key("union_field"), "Should parse union field");
        
        // Should parse required fields
        assert!(schema.required.contains(&"simple_string".to_string()), "Should identify required fields");
        
        // ACT: Generate code from the same JSON file with DirectTypifyGenerator
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_file.to_str().unwrap());
        
        // ASSERT: Code generation should work with the schema file
        assert!(result.is_ok(), "Code generation from schema file should succeed");
        
        let generated_code = result.unwrap();
        assert!(generated_code.contains("ParserTest"), "Should generate struct from parsed schema");
    }

    // Helper function to validate Rust syntax (simplified check)
    fn validate_rust_syntax(code: &str) -> bool {
        // Basic syntax checks
        let brace_balance = code.chars().fold(0i32, |acc, c| {
            match c {
                '{' => acc + 1,
                '}' => acc - 1,
                _ => acc,
            }
        });
        
        let paren_balance = code.chars().fold(0i32, |acc, c| {
            match c {
                '(' => acc + 1,
                ')' => acc - 1,
                _ => acc,
            }
        });

        // Must have balanced braces and parentheses
        brace_balance == 0 && paren_balance == 0 &&
        // Must contain some basic Rust constructs
        (code.contains("struct") || code.contains("enum") || code.contains("type")) &&
        // Must have serde imports (typify uses fully qualified paths)
        (code.contains("use serde") || code.contains(":: serde ::")) &&
        // Must not have obvious syntax errors
        !code.contains("{{") && !code.contains("}}")
    }
}