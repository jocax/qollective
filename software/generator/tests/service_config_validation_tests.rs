// ABOUTME: Service configuration schema validation tests for the Qollective Framework
// ABOUTME: Tests JSON schema validation for service configuration and presets

use jsonschema::Validator;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Test suite for validating service configuration schema
#[cfg(test)]
mod service_config_validation_tests {
    use super::*;

    #[test]
    fn test_service_config_schema_is_valid() {
        // ARRANGE: Load the service config schema file
        let schema_path = Path::new("../schemas/core/service-config.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read service config schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse service config schema as JSON");

        // ACT: Create JSONSchema validator
        let result = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value);

        // ASSERT: Schema compilation should succeed
        assert!(result.is_ok(), "Service config schema should be valid JSON Schema");
    }

    #[test]
    fn test_minimal_service_config_validates() {
        // ARRANGE: Load schema and create minimal valid config
        let schema = load_service_config_schema();
        let minimal_config = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0"
            },
            "qollective": {}
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&minimal_config);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Minimal service config should pass validation");
    }

    #[test]
    fn test_complete_service_config_validates() {
        // ARRANGE: Load schema and create complete valid config
        let schema = load_service_config_schema();
        let complete_config = json!({
            "service": {
                "name": "user-service",
                "version": "1.2.3",
                "description": "User management and authentication service",
                "team": "platform-team",
                "repository": "https://github.com/company/user-service",
                "contact": {
                    "email": "platform-team@company.com",
                    "slack": "#platform-support",
                    "pagerduty": "P7XYZ12"
                }
            },
            "qollective": {
                "meta": {
                    "security": {
                        "enabled": true,
                        "properties": {
                            "user_id": true,
                            "session_id": true,
                            "*": false
                        }
                    },
                    "performance": {
                        "enabled": true,
                        "properties": "*",
                        "sampling_rate": 0.1
                    }
                },
                "protocols": {
                    "rest": {
                        "enabled": true,
                        "headers": {
                            "prefix": "X-User-Service"
                        }
                    },
                    "grpc": {
                        "enabled": false
                    }
                }
            }
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&complete_config);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Complete service config should pass validation");
    }

    #[test]
    fn test_service_name_validation() {
        // ARRANGE: Load schema and create configs with various service names
        let schema = load_service_config_schema();
        
        let test_cases = vec![
            ("valid-service", true),
            ("user-api", true),
            ("payment-processor-v2", true),
            ("a", false), // too short
            ("Invalid-Service", false), // uppercase
            ("invalid_service", false), // underscore
            ("123-service", false), // starts with number
            ("-invalid", false), // starts with dash
            ("invalid-", false), // ends with dash
        ];

        for (service_name, should_be_valid) in test_cases {
            let config = json!({
                "service": {
                    "name": service_name,
                    "version": "1.0.0"
                },
                "qollective": {}
            });

            // ACT: Validate the config
            let validation_result = schema.validate(&config);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "Service name '{}' should be valid", service_name);
            } else {
                assert!(validation_result.is_err(), "Service name '{}' should be invalid", service_name);
            }
        }
    }

    #[test]
    fn test_version_format_validation() {
        // ARRANGE: Load schema and create configs with various version formats
        let schema = load_service_config_schema();
        
        let test_cases = vec![
            ("1.0.0", true),
            ("2.1.3", true),
            ("0.5.0", true),
            ("10.20.30", true),
            ("1.0", false), // missing patch
            ("v1.0.0", false), // has prefix
            ("1.0.0-beta", false), // has suffix
            ("1.0.0.1", false), // too many parts
        ];

        for (version, should_be_valid) in test_cases {
            let config = json!({
                "service": {
                    "name": "test-service",
                    "version": version
                },
                "qollective": {}
            });

            // ACT: Validate the config
            let validation_result = schema.validate(&config);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "Version '{}' should be valid", version);
            } else {
                assert!(validation_result.is_err(), "Version '{}' should be invalid", version);
            }
        }
    }

    #[test]
    fn test_meta_section_config_validation() {
        // ARRANGE: Load schema and create config with meta section configurations
        let schema = load_service_config_schema();
        let config_with_meta = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0"
            },
            "qollective": {
                "meta": {
                    "security": {
                        "enabled": true,
                        "properties": "*",
                        "sampling_rate": 1.0
                    },
                    "performance": {
                        "enabled": false
                    },
                    "debug": {
                        "enabled": true,
                        "properties": {
                            "trace_enabled": true,
                            "log_level": true,
                            "*": false
                        },
                        "sampling_rate": 0.1,
                        "max_size": 4096
                    }
                }
            }
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&config_with_meta);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Config with meta sections should pass validation");
    }

    #[test]
    fn test_sampling_rate_validation() {
        // ARRANGE: Load schema and test various sampling rates
        let schema = load_service_config_schema();
        
        let test_cases = vec![
            (0.0, true),
            (0.1, true),
            (0.5, true),
            (1.0, true),
            (-0.1, false), // negative
            (1.1, false), // greater than 1
        ];

        for (sampling_rate, should_be_valid) in test_cases {
            let config = json!({
                "service": {
                    "name": "test-service",
                    "version": "1.0.0"
                },
                "qollective": {
                    "meta": {
                        "performance": {
                            "enabled": true,
                            "sampling_rate": sampling_rate
                        }
                    }
                }
            });

            // ACT: Validate the config
            let validation_result = schema.validate(&config);

            // ASSERT: Check validation result matches expectation
            if should_be_valid {
                assert!(validation_result.is_ok(), "Sampling rate {} should be valid", sampling_rate);
            } else {
                assert!(validation_result.is_err(), "Sampling rate {} should be invalid", sampling_rate);
            }
        }
    }

    #[test]
    fn test_protocol_configuration_validation() {
        // ARRANGE: Load schema and create config with protocol settings
        let schema = load_service_config_schema();
        let config_with_protocols = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0"
            },
            "qollective": {
                "protocols": {
                    "rest": {
                        "enabled": true,
                        "headers": {
                            "prefix": "X-Test",
                            "request_id": "X-Request-ID",
                            "trace_id": "X-Trace-ID"
                        },
                        "compression": {
                            "enabled": true,
                            "algorithms": ["gzip", "deflate"],
                            "min_size": 1024
                        },
                        "timeouts": {
                            "request": 30000,
                            "connection": 5000
                        }
                    },
                    "grpc": {
                        "enabled": true,
                        "reflection": false,
                        "health_check": true,
                        "compression": {
                            "enabled": true,
                            "algorithm": "gzip"
                        },
                        "keepalive": {
                            "time": 30,
                            "timeout": 5
                        },
                        "max_message_size": 4194304
                    }
                }
            }
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&config_with_protocols);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Config with protocol settings should pass validation");
    }

    #[test]
    fn test_presets_configuration_validation() {
        // ARRANGE: Load schema and create config with presets
        let schema = load_service_config_schema();
        let config_with_presets = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0"
            },
            "qollective": {
                "presets": {
                    "development": {
                        "description": "Development environment with full meta",
                        "meta": {
                            "security": {
                                "enabled": true,
                                "properties": "*"
                            },
                            "debug": {
                                "enabled": true,
                                "properties": "*"
                            }
                        }
                    },
                    "production": {
                        "description": "Production environment with minimal meta",
                        "meta": {
                            "security": {
                                "enabled": true,
                                "properties": {
                                    "user_id": true,
                                    "*": false
                                }
                            },
                            "debug": {
                                "enabled": false
                            }
                        }
                    }
                }
            }
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&config_with_presets);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Config with presets should pass validation");
    }

    #[test]
    fn test_environments_configuration_validation() {
        // ARRANGE: Load schema and create config with environment overrides
        let schema = load_service_config_schema();
        let config_with_environments = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0"
            },
            "qollective": {
                "environments": {
                    "development": {
                        "preset": "development"
                    },
                    "staging": {
                        "meta": {
                            "debug": {
                                "enabled": true,
                                "sampling_rate": 0.1
                            }
                        }
                    },
                    "production": {
                        "preset": "production",
                        "protocols": {
                            "rest": {
                                "compression": {
                                    "enabled": true,
                                    "min_size": 2048
                                }
                            }
                        }
                    }
                }
            }
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&config_with_environments);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Config with environments should pass validation");
    }

    #[test]
    fn test_contact_information_validation() {
        // ARRANGE: Load schema and test contact information
        let schema = load_service_config_schema();
        let config_with_contact = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0",
                "contact": {
                    "email": "team@example.com",
                    "slack": "#team-channel",
                    "pagerduty": "PXXXXXX"
                }
            },
            "qollective": {}
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&config_with_contact);

        // ASSERT: Validation should succeed
        assert!(validation_result.is_ok(), "Config with contact info should pass validation");
    }

    #[test]
    fn test_invalid_email_format() {
        // ARRANGE: Load schema and test invalid email
        let schema = load_service_config_schema();
        let config_with_invalid_email = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0",
                "contact": {
                    "email": "invalid-email"
                }
            },
            "qollective": {}
        });

        // ACT: Validate the config
        let validation_result = schema.validate(&config_with_invalid_email);

        // ASSERT: Validation should fail
        assert!(validation_result.is_err(), "Config with invalid email should fail validation");
    }

    #[test]
    fn test_missing_required_fields() {
        // ARRANGE: Load schema and test configs missing required fields
        let schema = load_service_config_schema();
        
        // Missing service name
        let config_missing_name = json!({
            "service": {
                "version": "1.0.0"
            },
            "qollective": {}
        });

        // Missing service version
        let config_missing_version = json!({
            "service": {
                "name": "test-service"
            },
            "qollective": {}
        });

        // Missing qollective section
        let config_missing_qollective = json!({
            "service": {
                "name": "test-service",
                "version": "1.0.0"
            }
        });

        // ACT & ASSERT: All should fail validation
        assert!(schema.validate(&config_missing_name).is_err(), "Config missing service name should fail");
        assert!(schema.validate(&config_missing_version).is_err(), "Config missing service version should fail");
        assert!(schema.validate(&config_missing_qollective).is_err(), "Config missing qollective section should fail");
    }

    // Helper function to load and compile the service config schema
    fn load_service_config_schema() -> Validator {
        let schema_path = Path::new("../schemas/core/service-config.json");
        let schema_content = fs::read_to_string(schema_path)
            .expect("Failed to read service config schema file");
        
        let schema_value: Value = serde_json::from_str(&schema_content)
            .expect("Failed to parse service config schema as JSON");

        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)
            .expect("Failed to compile service config schema")
    }
}