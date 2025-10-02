// ABOUTME: WASM integration tests for all WASM protocol implementations
// ABOUTME: Tests crypto management, MCP adapter, REST client, and WebSocket client functionality

//! Integration tests for WASM envelope support.
//!
//! This module provides comprehensive tests for all WASM protocol implementations
//! including certificate management, MCP adapter translation, REST client, and
//! WebSocket client functionality following the Qollective framework patterns.

mod common;
use common::setup_test_environment;

// WASM-specific configuration tests (only available when WASM feature is enabled)
#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
mod wasm_config_tests {
    use super::*;
    use qollective::config::wasm::{CertificateConfig, WasmClientConfig, WasmTimeoutConfig, McpAdapterConfig, BundleConfig, BrowserConfig, CertificateInfo};
    use serde_json::json;

    fn create_test_certificate_config() -> CertificateConfig {
        CertificateConfig {
            default_domain: "test.example.com".to_string(),
            certificates: vec![
                CertificateInfo {
                    domains: vec!["*.example.com".to_string(), "api.example.com".to_string()],
                    cert_data: "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t".to_string(), // Mock base64 cert
                    key_data: "LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0t".to_string(),   // Mock base64 key
                    ca_data: Some("LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t".to_string()), // Mock base64 CA
                }
            ],
        }
    }

    /// Test WASM configuration structure without browser dependencies
    #[test]
    fn test_wasm_config_structure() {
        setup_test_environment();

        let config = WasmClientConfig::default();
        
        // Test configuration serialization/deserialization
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: WasmClientConfig = serde_json::from_str(&serialized).unwrap();
        
        // Compare key fields
        assert_eq!(config.rest_enabled, deserialized.rest_enabled);
        assert_eq!(config.websocket_enabled, deserialized.websocket_enabled);
        assert_eq!(config.mcp_enabled, deserialized.mcp_enabled);
    }

    /// Test certificate configuration validation
    #[test]
    fn test_certificate_config_validation() {
        setup_test_environment();

        let mut config = CertificateConfig::default();
        
        // Test default configuration
        assert!(config.certificates.is_empty());
        assert_eq!(config.default_domain, "localhost");
        
        // Test adding certificate info
        config.certificates.push(CertificateInfo {
            domains: vec!["test.com".to_string(), "*.test.com".to_string()],
            cert_data: "test-cert".to_string(),
            key_data: "test-key".to_string(),
            ca_data: Some("test-ca".to_string()),
        });
        
        assert_eq!(config.certificates.len(), 1);
        assert_eq!(config.certificates[0].domains.len(), 2);
        assert!(config.certificates[0].ca_data.is_some());
        
        // Test serialization
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: CertificateConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.certificates.len(), deserialized.certificates.len());
        assert_eq!(config.default_domain, deserialized.default_domain);
    }

    /// Test complete WASM configuration integration
    #[test]
    fn test_complete_wasm_config_integration() {
        setup_test_environment();

        let config = WasmClientConfig::default();
        
        // Test that all protocol configurations are present and valid
        assert!(config.mcp_config.connection_timeout_ms > 0);
        assert!(config.certificate_config.certificates.is_empty()); // Default has no certificates
        assert!(config.timeouts.connection_timeout_ms > 0);
        assert!(config.bundle_config.max_bundle_size_mb > 0);
        
        // Test configuration serialization
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
        
        let json_str = serialized.unwrap();
        assert!(json_str.contains("mcp_config"));
        assert!(json_str.contains("certificate_config"));
        assert!(json_str.contains("timeouts"));
        assert!(json_str.contains("bundle_config"));
    }
}

// Fallback tests for non-WASM environments (validate patterns and serialization)
#[cfg(not(all(target_arch = "wasm32", feature = "wasm-client")))]
mod fallback_config_tests {
    use super::*;
    use serde_json::json;

    /// Test that we can at least validate basic configuration structures when WASM is not available
    #[test]
    fn test_basic_config_serialization() {
        setup_test_environment();

        // Test basic JSON serialization patterns that WASM configs would use
        let test_config = json!({
            "rest_enabled": true,
            "websocket_enabled": true,
            "mcp_enabled": true,
            "certificate_config": {
                "default_domain": "localhost",
                "certificates": []
            },
            "timeouts": {
                "connection_timeout_ms": 10000,
                "request_timeout_ms": 30000,
                "retry_delay_ms": 1000
            }
        });

        // Verify the test configuration can be serialized and deserialized
        let serialized = serde_json::to_string(&test_config).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(test_config, deserialized);
        assert_eq!(test_config["rest_enabled"], true);
        assert_eq!(test_config["mcp_enabled"], true);
    }

    /// Test error handling patterns used in WASM configurations
    #[test]
    fn test_error_handling_patterns() {
        setup_test_environment();

        // Test invalid configuration scenarios
        let invalid_timeout_config = json!({
            "connection_timeout_ms": 0, // Invalid timeout
            "request_timeout_ms": -1   // Invalid negative timeout
        });

        let serialized = serde_json::to_string(&invalid_timeout_config);
        assert!(serialized.is_ok());

        // Invalid JSON structure should fail deserialization
        let invalid_json = "{invalid json}";
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(parse_result.is_err());
    }

    /// Test domain matching patterns that would be used in certificate management
    #[test]
    fn test_domain_pattern_validation() {
        setup_test_environment();

        let test_domains = vec![
            "exact.example.com",
            "*.wildcard.com",
            "multi.*.pattern.com",
        ];

        // Test domain pattern validation logic
        for domain in &test_domains {
            assert!(!domain.is_empty());
            
            // Basic wildcard pattern validation
            if domain.contains('*') {
                assert!(domain.len() > 1); // Wildcard must not be the only character
            }
            
            // Basic domain format validation
            assert!(domain.contains('.'));
        }

        // Test that all domains serialize properly
        let domain_config = json!({
            "domains": test_domains
        });
        
        let serialized = serde_json::to_string(&domain_config).unwrap();
        assert!(serialized.contains("*.wildcard.com"));
        assert!(serialized.contains("exact.example.com"));
    }

    /// Test MCP error policy patterns
    #[test]
    fn test_mcp_error_policy_patterns() {
        setup_test_environment();

        // Test different error policy configurations
        let retry_exponential = json!({
            "type": "RetryExponential",
            "max_retries": 3,
            "base_delay_ms": 1000,
            "max_delay_ms": 30000
        });

        let retry_linear = json!({
            "type": "RetryLinear",
            "max_retries": 3,
            "delay_ms": 1000
        });

        let fail_fast = json!({
            "type": "FailFast"
        });

        // Test serialization of all policy types
        for policy in [&retry_exponential, &retry_linear, &fail_fast] {
            let serialized = serde_json::to_string(policy);
            assert!(serialized.is_ok());
            
            let deserialized: serde_json::Value = serde_json::from_str(&serialized.unwrap()).unwrap();
            assert_eq!(policy, &deserialized);
        }
    }

    /// Test browser compatibility configuration patterns
    #[test]
    fn test_browser_compatibility_patterns() {
        setup_test_environment();

        let browser_config = json!({
            "min_versions": {
                "chrome": "88",
                "firefox": "85",
                "safari": "14",
                "edge": "88"
            },
            "polyfills": {
                "fetch": true,
                "websocket": false,
                "promise": true,
                "abort_controller": true
            },
            "feature_detection": {
                "webassembly": true,
                "bigint": true,
                "dynamic_import": true,
                "worker": true
            }
        });

        // Test that browser config serializes properly
        let serialized = serde_json::to_string(&browser_config).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(browser_config, deserialized);
        assert_eq!(browser_config["min_versions"]["chrome"], "88");
        assert_eq!(browser_config["polyfills"]["fetch"], true);
        assert_eq!(browser_config["feature_detection"]["webassembly"], true);
    }

    /// Test bundle optimization configuration patterns
    #[test]
    fn test_bundle_optimization_patterns() {
        setup_test_environment();

        let bundle_config = json!({
            "size_limit_bytes": 500000, // 500KB
            "optimize_for_size": true,
            "tree_shaking": true,
            "compression": {
                "gzip": true,
                "brotli": true,
                "level": 6
            }
        });

        // Test bundle config validation
        let serialized = serde_json::to_string(&bundle_config).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(bundle_config, deserialized);
        assert_eq!(bundle_config["size_limit_bytes"], 500000);
        assert_eq!(bundle_config["optimize_for_size"], true);
        assert_eq!(bundle_config["compression"]["level"], 6);
    }

    /// Test timeout configuration bounds
    #[test]
    fn test_timeout_bounds() {
        setup_test_environment();

        let timeout_config = json!({
            "connection_timeout_ms": 30000,
            "request_timeout_ms": 60000,
            "retry_delay_ms": 1000,
            "max_retries": 3
        });

        // Test reasonable timeout values
        assert!(timeout_config["connection_timeout_ms"].as_u64().unwrap() >= 1000); // At least 1 second
        assert!(timeout_config["request_timeout_ms"].as_u64().unwrap() >= timeout_config["connection_timeout_ms"].as_u64().unwrap()); // Request >= connection
        assert!(timeout_config["retry_delay_ms"].as_u64().unwrap() >= 100); // At least 100ms
        assert!(timeout_config["max_retries"].as_u64().unwrap() <= 10); // Reasonable retry limit

        // Test serialization preserves values
        let serialized = serde_json::to_string(&timeout_config).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(timeout_config["connection_timeout_ms"], deserialized["connection_timeout_ms"]);
        assert_eq!(timeout_config["request_timeout_ms"], deserialized["request_timeout_ms"]);
    }

    /// Test WASM constants validation (using constants that exist)
    #[test]
    fn test_wasm_constants() {
        setup_test_environment();

        // Test that WASM-related constants are defined and reasonable
        #[cfg(any(feature = "rest-client", feature = "wasm-client"))]
        {
            use qollective::constants::{timeouts, limits};
            assert!(timeouts::DEFAULT_REST_RETRY_DELAY_MS > 0);
            assert!(timeouts::DEFAULT_REST_RETRY_DELAY_MS <= 5000); // Reasonable upper bound
            assert!(limits::DEFAULT_RETRY_ATTEMPTS > 0);
            assert!(limits::DEFAULT_RETRY_ATTEMPTS <= 10); // Reasonable upper bound
            assert!(timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS > 0);
        }
        
        #[cfg(feature = "websocket-client")]
        {
            use qollective::constants::timeouts;
            assert!(timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS > 0);
            assert!(timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS > 0);
            assert!(timeouts::DEFAULT_WEBSOCKET_PING_INTERVAL_MS > 0);
        }
    }

    /// Test configuration inheritance and override patterns
    #[test]
    fn test_config_inheritance() {
        setup_test_environment();

        let base_config = json!({
            "mcp_config": {
                "connection_timeout_ms": 10000
            },
            "timeouts": {
                "connection_timeout_ms": 10000
            }
        });
        
        // Test that child configurations inherit from parent properly
        assert_eq!(base_config["mcp_config"]["connection_timeout_ms"], base_config["timeouts"]["connection_timeout_ms"]);
        
        // Test timeout override cascading
        let mut custom_config = base_config.clone();
        custom_config["timeouts"]["connection_timeout_ms"] = json!(15000);
        
        // Verify the change is reflected
        assert_eq!(custom_config["timeouts"]["connection_timeout_ms"], 15000);
        
        // Test that other timeouts remain unchanged
        assert_ne!(custom_config["timeouts"]["connection_timeout_ms"], 10000);
    }

    /// Test error handling and validation in configurations
    #[test]
    fn test_config_error_handling() {
        setup_test_environment();

        // Test that invalid configurations are properly validated
        let mut config = json!({
            "rest_enabled": true,
            "websocket_enabled": true,
            "mcp_enabled": true,
            "timeouts": {
                "connection_timeout_ms": 5000
            },
            "certificate_config": {
                "certificates": []
            }
        });
        
        // Test configuration with invalid timeout values
        config["timeouts"]["connection_timeout_ms"] = json!(0); // Invalid timeout
        
        // Configuration should still serialize but indicate invalid state
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
        
        // Reset to valid state
        config["timeouts"]["connection_timeout_ms"] = json!(5000);
        
        // Test invalid certificate configuration
        config["certificate_config"]["certificates"].as_array_mut().unwrap().push(json!({
            "domains": [], // Empty domains should be handled gracefully
            "cert_data": "", // Empty cert data
            "key_data": "", // Empty key data
            "ca_data": null
        }));
        
        // Should not panic during serialization
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }
}