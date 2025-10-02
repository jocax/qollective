// ABOUTME: Test module declarations for schema validation tests
// ABOUTME: Includes all schema validation test modules and integration helpers

pub mod schema_validation_tests;
pub mod service_config_validation_tests;
pub mod meta_validation_tests;
pub mod validation_error_handling_tests;

use jsonschema::Validator;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Common test utilities for schema validation
pub mod test_utils {
    use super::*;

    /// Load and compile a JSON schema from file path
    pub fn load_schema(schema_path: &Path) -> Result<Validator, Box<dyn std::error::Error>> {
        let schema_content = fs::read_to_string(schema_path)?;
        let schema_value: Value = serde_json::from_str(&schema_content)?;
        let schema = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema_value)?;
        Ok(schema)
    }

    /// Validate data against schema and return detailed error information
    pub fn validate_with_details(
        schema: &Validator,
        data: &Value,
    ) -> Result<(), Vec<String>> {
        if schema.is_valid(data) {
            Ok(())
        } else {
            let error_messages: Vec<String> = schema.iter_errors(data)
                .map(|e| format!("{} (at {})", e, e.instance_path))
                .collect();
            Err(error_messages)
        }
    }

    /// Run a validation test case and return standardized result
    pub fn run_validation_test(
        test_name: &str,
        schema: &Validator,
        data: &Value,
        should_pass: bool,
    ) -> TestResult {
        let start = std::time::Instant::now();
        let is_valid = schema.is_valid(data);
        let duration = start.elapsed();

        let passed = if should_pass {
            is_valid
        } else {
            !is_valid
        };

        let error_message = if !passed {
            if should_pass {
                let errors: Vec<String> = schema.iter_errors(data)
                    .map(|e| e.to_string())
                    .collect();
                Some(format!(
                    "Expected validation to pass but failed: {}",
                    errors.join(", ")
                ))
            } else {
                Some("Expected validation to fail but it passed".to_string())
            }
        } else {
            None
        };

        TestResult {
            test_name: test_name.to_string(),
            passed,
            error_message,
            duration_ms: duration.as_millis(),
        }
    }

    #[derive(Debug, Clone)]
    pub struct TestResult {
        pub test_name: String,
        pub passed: bool,
        pub error_message: Option<String>,
        pub duration_ms: u128,
    }

    /// Print test results in a formatted way
    pub fn print_test_results(results: &[TestResult], suite_name: &str) {
        println!("\n🧪 {} Test Results", suite_name);
        println!("{}", "=".repeat(suite_name.len() + 14));

        let passed_count = results.iter().filter(|r| r.passed).count();
        let total_count = results.len();

        for result in results {
            let status = if result.passed { "✅" } else { "❌" };
            println!("{} {} ({}ms)", status, result.test_name, result.duration_ms);

            if let Some(error) = &result.error_message {
                println!("   └─ {}", error);
            }
        }

        println!("\n📊 Summary: {}/{} passed", passed_count, total_count);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::test_utils::*;
    use serde_json::json;

    #[test]
    fn test_all_schemas_compile_successfully() {
        let schema_dir = Path::new("../schemas/core");
        
        let schema_files = vec![
            "envelope.json",
            "service-config.json",
            "meta-security.json",
            "meta-performance.json",
            "meta-tracing.json",
            "meta-debug.json",
            "meta-monitoring.json",
        ];

        for schema_file in schema_files {
            let schema_path = schema_dir.join(schema_file);
            let result = load_schema(&schema_path);
            
            assert!(
                result.is_ok(),
                "Schema {} should compile successfully: {:?}",
                schema_file,
                result.err()
            );
        }
    }

    #[test]
    fn test_envelope_integrates_with_all_meta_schemas() {
        let schema_dir = Path::new("../schemas/core");
        let envelope_schema = load_schema(&schema_dir.join("envelope.json"))
            .expect("Failed to load envelope schema");

        // Create envelope with data from all meta sections
        let comprehensive_envelope = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "duration": 245.000,
                "serviceChain": [
                    {
                        "serviceName": "test-service",
                        "serviceVersion": "1.0.0",
                        "requestId": "550e8400-e29b-41d4-a716-446655440001",
                        "timestamp": "2024-06-06T14:23:14.000Z",
                        "duration": 233.000
                    }
                ],
                "security": {
                    "user_id": "test_user",
                    "session_id": "sess_12345",
                    "auth_method": "jwt",
                    "permissions": ["user:read", "admin:write"],
                    "tenant_id": "tenant-123",
                    "roles": ["admin", "user"]
                },
                "performance": {
                    "db_query_time": 45.123,
                    "db_query_count": 3,
                    "cache_hit_ratio": 0.85,
                    "memory_allocated": 1048576,
                    "cpu_usage": 0.15
                },
                "monitoring": {
                    "server_id": "web-server-01",
                    "datacenter": "us-west-2",
                    "environment": "production",
                    "build_version": "v1.2.3-build.456"
                },
                "tracing": {
                    "trace_id": "550e8400e29b41d4a716446655440000",
                    "span_id": "b7ad6b7169203331",
                    "operation_name": "test_operation",
                    "span_kind": "server",
                    "sampled": true
                },
                "debug": {
                    "trace_enabled": true,
                    "log_level": "debug",
                    "memory_usage": {
                        "heap_used": 1024,
                        "heap_total": 2048
                    }
                },
                "extensions": {
                    "pagination": {
                        "total_items": 100,
                        "current_page": 1,
                        "page_size": 25,
                        "has_next_page": true,
                        "has_previous_page": false,
                        "items_on_page": 25
                    }
                }
            },
            "data": {
                "test": "comprehensive integration test",
                "items": [1, 2, 3]
            }
        });

        assert!(
            envelope_schema.is_valid(&comprehensive_envelope),
            "Comprehensive envelope should validate successfully: {:?}",
            envelope_schema.iter_errors(&comprehensive_envelope).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cross_schema_consistency() {
        let schema_dir = Path::new("../schemas/core");
        
        // Load all schemas
        let envelope_schema = load_schema(&schema_dir.join("envelope.json")).unwrap();
        let security_schema = load_schema(&schema_dir.join("meta-security.json")).unwrap();
        
        // Create security data that should be valid in both contexts
        let security_data = json!({
            "user_id": "test_user",
            "session_id": "sess_12345",
            "auth_method": "jwt",
            "permissions": ["user:read", "user:write"],
            "ip_address": "192.168.1.100",
            "tenant_id": "tenant-123"
        });

        // Validate against security schema
        assert!(
            security_schema.is_valid(&security_data),
            "Security data should be valid against security schema"
        );

        // Validate envelope containing the same security data
        let envelope_with_security = json!({
            "meta": {
                "timestamp": "2024-06-06T14:23:15.456Z",
                "requestId": "550e8400-e29b-41d4-a716-446655440000",
                "version": "1.2.3",
                "security": security_data
            },
            "data": {"test": "security integration"}
        });

        assert!(
            envelope_schema.is_valid(&envelope_with_security),
            "Envelope with security data should be valid: {:?}",
            envelope_schema.iter_errors(&envelope_with_security).collect::<Vec<_>>()
        );
    }
}