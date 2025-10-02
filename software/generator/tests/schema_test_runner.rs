// ABOUTME: Schema validation test runner for the Qollective Framework
// ABOUTME: Standalone binary to run schema validation tests and report results

use jsonschema::{Validator, ValidationError};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct TestResult {
    schema_name: String,
    test_name: String,
    passed: bool,
    error_message: Option<String>,
    execution_time_ms: u128,
}

#[derive(Debug)]
struct TestSuite {
    name: String,
    results: Vec<TestResult>,
}

impl TestSuite {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            results: Vec::new(),
        }
    }

    fn add_result(&mut self, result: TestResult) {
        self.results.push(result);
    }

    fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    fn total_count(&self) -> usize {
        self.results.len()
    }

    fn total_execution_time_ms(&self) -> u128 {
        self.results.iter().map(|r| r.execution_time_ms).sum()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Qollective Schema Validation Test Runner");
    println!("============================================");

    let schema_dir = Path::new("../schemas/core");
    if !schema_dir.exists() {
        eprintln!("❌ Schema directory not found: {:?}", schema_dir);
        std::process::exit(1);
    }

    let mut test_suites = Vec::new();

    // Run envelope schema tests
    test_suites.push(run_envelope_schema_tests(schema_dir)?);

    // Run service config schema tests
    test_suites.push(run_service_config_schema_tests(schema_dir)?);

    // Run meta schema tests
    test_suites.push(run_meta_schema_tests(schema_dir)?);

    // Run cross-schema integration tests
    test_suites.push(run_integration_tests(schema_dir)?);

    // Print summary
    print_test_summary(&test_suites);

    // Exit with appropriate code
    let total_failures: usize = test_suites.iter().map(|s| s.failed_count()).sum();
    if total_failures > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_envelope_schema_tests(schema_dir: &Path) -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite::new("Envelope Schema");
    
    let schema_path = schema_dir.join("envelope.json");
    let validator = load_validator(&schema_path)?;

    // Test 1: Schema compilation
    let start = std::time::Instant::now();
    let result = TestResult {
        schema_name: "envelope".to_string(),
        test_name: "Schema compilation".to_string(),
        passed: true, // If we got here, it compiled
        error_message: None,
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    // Test 2: Valid success response
    let start = std::time::Instant::now();
    let valid_response = json!({
        "meta": {
            "timestamp": "2024-06-06T14:23:15.456Z",
            "requestId": "550e8400-e29b-41d4-a716-446655440000",
            "version": "1.2.3"
        },
        "data": {"user_id": 123, "name": "Test User"}
    });
    
    let validation_result = validator.validate(&valid_response);
    let result = TestResult {
        schema_name: "envelope".to_string(),
        test_name: "Valid success response".to_string(),
        passed: validation_result.is_ok(),
        error_message: if validation_result.is_err() {
            Some(format_validation_errors(validation_result.unwrap_err()))
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    // Test 3: Valid error response
    let start = std::time::Instant::now();
    let valid_error = json!({
        "meta": {
            "timestamp": "2024-06-06T14:23:15.456Z",
            "requestId": "550e8400-e29b-41d4-a716-446655440000",
            "version": "1.2.3"
        },
        "error": {
            "code": "USER_NOT_FOUND",
            "message": "User not found"
        }
    });
    
    let validation_result = validator.validate(&valid_error);
    let result = TestResult {
        schema_name: "envelope".to_string(),
        test_name: "Valid error response".to_string(),
        passed: validation_result.is_ok(),
        error_message: if validation_result.is_err() {
            Some(format_validation_errors(validation_result.unwrap_err()))
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    // Test 4: Invalid - both data and error
    let start = std::time::Instant::now();
    let invalid_both = json!({
        "meta": {
            "timestamp": "2024-06-06T14:23:15.456Z",
            "requestId": "550e8400-e29b-41d4-a716-446655440000",
            "version": "1.2.3"
        },
        "data": {"test": "data"},
        "error": {"code": "ERROR", "message": "Error"}
    });
    
    let validation_result = validator.validate(&invalid_both);
    let result = TestResult {
        schema_name: "envelope".to_string(),
        test_name: "Rejects both data and error".to_string(),
        passed: validation_result.is_err(),
        error_message: if validation_result.is_ok() {
            Some("Expected validation to fail but it passed".to_string())
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    Ok(suite)
}

fn run_service_config_schema_tests(schema_dir: &Path) -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite::new("Service Config Schema");
    
    let schema_path = schema_dir.join("service-config.json");
    let validator = load_validator(&schema_path)?;

    // Test 1: Schema compilation
    let start = std::time::Instant::now();
    let result = TestResult {
        schema_name: "service-config".to_string(),
        test_name: "Schema compilation".to_string(),
        passed: true,
        error_message: None,
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    // Test 2: Minimal valid config
    let start = std::time::Instant::now();
    let minimal_config = json!({
        "service": {
            "name": "test-service",
            "version": "1.0.0"
        },
        "qollective": {}
    });
    
    let validation_result = validator.validate(&minimal_config);
    let result = TestResult {
        schema_name: "service-config".to_string(),
        test_name: "Minimal valid config".to_string(),
        passed: validation_result.is_ok(),
        error_message: if validation_result.is_err() {
            Some(format_validation_errors(validation_result.unwrap_err()))
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    // Test 3: Invalid service name
    let start = std::time::Instant::now();
    let invalid_name = json!({
        "service": {
            "name": "Invalid-Service-Name",
            "version": "1.0.0"
        },
        "qollective": {}
    });
    
    let validation_result = validator.validate(&invalid_name);
    let result = TestResult {
        schema_name: "service-config".to_string(),
        test_name: "Rejects invalid service name".to_string(),
        passed: validation_result.is_err(),
        error_message: if validation_result.is_ok() {
            Some("Expected validation to fail but it passed".to_string())
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    Ok(suite)
}

fn run_meta_schema_tests(schema_dir: &Path) -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite::new("Meta Schemas");
    
    let meta_schemas = vec![
        "meta-security.json",
        "meta-performance.json",
        "meta-tracing.json",
        "meta-debug.json",
        "meta-monitoring.json",
    ];

    for schema_file in meta_schemas {
        let start = std::time::Instant::now();
        let schema_path = schema_dir.join(schema_file);
        
        match load_validator(&schema_path) {
            Ok(_schema) => {
                let result = TestResult {
                    schema_name: schema_file.to_string(),
                    test_name: "Schema compilation".to_string(),
                    passed: true,
                    error_message: None,
                    execution_time_ms: start.elapsed().as_millis(),
                };
                suite.add_result(result);
            },
            Err(e) => {
                let result = TestResult {
                    schema_name: schema_file.to_string(),
                    test_name: "Schema compilation".to_string(),
                    passed: false,
                    error_message: Some(e.to_string()),
                    execution_time_ms: start.elapsed().as_millis(),
                };
                suite.add_result(result);
            }
        }
    }

    // Test security meta validation
    let start = std::time::Instant::now();
    let security_validator = load_validator(&schema_dir.join("meta-security.json"))?;
    let valid_security = json!({
        "user_id": "test_user",
        "auth_method": "jwt",
        "permissions": ["user:read", "user:write"]
    });
    
    let validation_result = security_validator.validate(&valid_security);
    let result = TestResult {
        schema_name: "meta-security".to_string(),
        test_name: "Valid security meta".to_string(),
        passed: validation_result.is_ok(),
        error_message: if validation_result.is_err() {
            Some(format_validation_errors(validation_result.unwrap_err()))
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    Ok(suite)
}

fn run_integration_tests(schema_dir: &Path) -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite::new("Integration Tests");
    
    // Test that envelope schema accepts all meta sections
    let start = std::time::Instant::now();
    let envelope_validator = load_validator(&schema_dir.join("envelope.json"))?;
    
    let envelope_with_all_meta = json!({
        "meta": {
            "timestamp": "2024-06-06T14:23:15.456Z",
            "requestId": "550e8400-e29b-41d4-a716-446655440000",
            "version": "1.2.3",
            "security": {
                "user_id": "test_user",
                "auth_method": "jwt"
            },
            "performance": {
                "db_query_time": 45.123,
                "cache_hit_ratio": 0.85
            },
            "monitoring": {
                "server_id": "web-01",
                "environment": "production"
            },
            "tracing": {
                "trace_id": "550e8400e29b41d4a716446655440000",
                "span_id": "b7ad6b7169203331"
            },
            "debug": {
                "trace_enabled": true,
                "log_level": "debug"
            }
        },
        "data": {"test": "integration"}
    });
    
    let validation_result = envelope_validator.validate(&envelope_with_all_meta);
    let result = TestResult {
        schema_name: "envelope".to_string(),
        test_name: "Envelope with all meta sections".to_string(),
        passed: validation_result.is_ok(),
        error_message: if validation_result.is_err() {
            Some(format_validation_errors(validation_result.unwrap_err()))
        } else {
            None
        },
        execution_time_ms: start.elapsed().as_millis(),
    };
    suite.add_result(result);

    Ok(suite)
}

fn load_validator(schema_path: &Path) -> Result<Validator, Box<dyn std::error::Error>> {
    let schema_content = fs::read_to_string(schema_path)?;
    let schema_value: Value = serde_json::from_str(&schema_content)?;
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema_value)?;
    Ok(validator)
}

fn format_validation_errors(error: ValidationError) -> String {
    format!("  • {}", error)
}

fn print_test_summary(test_suites: &[TestSuite]) {
    println!("\n📊 Test Summary");
    println!("===============");

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_time = 0;

    for suite in test_suites {
        let passed = suite.passed_count();
        let failed = suite.failed_count();
        let total = suite.total_count();
        let time = suite.total_execution_time_ms();

        total_passed += passed;
        total_failed += failed;
        total_time += time;

        let status_emoji = if failed == 0 { "✅" } else { "❌" };
        println!(
            "{} {} - {}/{} passed ({}ms)",
            status_emoji, suite.name, passed, total, time
        );

        // Print failed tests
        for result in &suite.results {
            if !result.passed {
                println!("   ❌ {} - {}", result.test_name, 
                    result.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
        }
    }

    println!("\n🎯 Overall Results");
    println!("==================");
    println!("Total Passed: {}", total_passed);
    println!("Total Failed: {}", total_failed);
    println!("Total Time: {}ms", total_time);

    if total_failed == 0 {
        println!("🎉 All tests passed!");
    } else {
        println!("⚠️  {} test(s) failed", total_failed);
    }
}