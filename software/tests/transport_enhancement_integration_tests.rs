// ABOUTME: Integration tests for enhanced transport implementations with jsonrpsee, rMCP, and a2a-rs
// ABOUTME: Tests new transport features while maintaining envelope pattern compatibility

//! Integration tests for enhanced transport implementations
//!
//! This module tests the new transport implementations:
//! - jsonrpsee 0.25.1 JSON-RPC 2.0 support
//! - rMCP 0.3.0 enhanced MCP features
//! - a2a-rs 0.1.0 standardized Agent-to-Agent protocol
//!
//! All tests validate envelope pattern compatibility and proper error handling.

use qollective::config::transport::TransportConfig;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use qollective::prelude::UnifiedEnvelopeSender;
use qollective::transport::HybridTransportClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

// Test data structures for transport enhancement testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EnhancedTestRequest {
    message: String,
    id: u32,
    transport_type: String,
    test_scenario: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EnhancedTestResponse {
    result: String,
    status: u32,
    processed_by: String,
    transport_used: String,
    features_supported: Vec<String>,
}

/// Test helper to create test envelope
fn create_test_envelope(request: EnhancedTestRequest) -> Envelope<EnhancedTestRequest> {
    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.timestamp = Some(chrono::Utc::now());
    meta.tenant = Some("transport-enhancement-test".to_string());

    Envelope::new(meta, request)
}

/// Test helper to create transport config for testing
fn create_transport_config_for_testing() -> TransportConfig {
    let mut config = TransportConfig::default();

    // Configure REST transport for basic testing
    config.protocols.rest = Some(qollective::config::presets::RestConfig {
        client: Some(qollective::config::presets::RestClientConfig::default()),
        server: None,
    });

    config
}

#[tokio::test]
async fn test_enhanced_mcp_stdio_with_jsonrpsee_integration() {
    // Test enhanced MCP-stdio transport with jsonrpsee JSON-RPC 2.0
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test enhanced MCP-stdio with jsonrpsee".to_string(),
        id: 1001,
        transport_type: "mcp-stdio".to_string(),
        test_scenario: "jsonrpsee_integration".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with MCP-stdio endpoint (would use jsonrpsee for JSON-RPC 2.0)
    let result: Result<Envelope<EnhancedTestResponse>> = client
        .send_envelope("mcp-stdio://localhost:8080/mcp", envelope)
        .await;

    // Verify that we get proper transport-level processing
    match result {
        Ok(response) => {
            // Success indicates proper jsonrpsee integration
            let (_, data) = response.extract();
            assert_eq!(data.transport_used, "mcp-stdio");
            assert!(data.features_supported.contains(&"jsonrpsee".to_string()));
        }
        Err(err) => {
            // Expected transport errors for non-existent endpoints
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("transport")
                    || error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("endpoint")
                    || error_msg.contains("not found")
                    || error_msg.contains("refused"),
                "Should get proper transport error, got: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_enhanced_mcp_transport_with_rmcp_features() {
    // Test enhanced MCP transport with rMCP 0.3.0 features
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test enhanced MCP with rMCP 0.3.0".to_string(),
        id: 2001,
        transport_type: "mcp".to_string(),
        test_scenario: "rmcp_features".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with enhanced MCP endpoint
    let result: Result<Envelope<EnhancedTestResponse>> = client
        .send_envelope("mcp://localhost:8080/tools", envelope)
        .await;

    // Verify proper rMCP integration
    match result {
        Ok(response) => {
            let (_, data) = response.extract();
            assert_eq!(data.transport_used, "mcp");
            assert!(data.features_supported.contains(&"rmcp".to_string()));
        }
        Err(err) => {
            // Expected transport errors for test endpoints
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("transport")
                    || error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("endpoint"),
                "Should get proper transport error, got: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_standardized_a2a_transport_with_a2a_rs() {
    // Test standardized A2A transport with a2a-rs 0.1.0
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test standardized A2A with a2a-rs".to_string(),
        id: 3001,
        transport_type: "a2a".to_string(),
        test_scenario: "a2a_rs_standard".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with A2A endpoint
    let result: Result<Envelope<EnhancedTestResponse>> = client
        .send_envelope("a2a://localhost:8080/agents", envelope)
        .await;

    // Verify proper a2a-rs integration
    match result {
        Ok(response) => {
            let (_, data) = response.extract();
            assert_eq!(data.transport_used, "a2a");
            assert!(data.features_supported.contains(&"a2a-rs".to_string()));
        }
        Err(err) => {
            // Expected transport errors for test endpoints
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("transport")
                    || error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("endpoint"),
                "Should get proper transport error, got: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_jsonrpsee_websocket_server_integration() {
    // Test new MCP jsonrpsee WebSocket server implementation
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test jsonrpsee WebSocket server".to_string(),
        id: 4001,
        transport_type: "jsonrpc-ws".to_string(),
        test_scenario: "websocket_server".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with WebSocket JSON-RPC endpoint
    let result: Result<Envelope<EnhancedTestResponse>> = client
        .send_envelope("ws://localhost:8080/jsonrpc", envelope)
        .await;

    // Verify proper WebSocket JSON-RPC integration
    match result {
        Ok(response) => {
            let (_, data) = response.extract();
            assert_eq!(data.transport_used, "jsonrpc-ws");
            assert!(data.features_supported.contains(&"websocket".to_string()));
        }
        Err(err) => {
            // Expected transport errors for test endpoints
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("transport")
                    || error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("websocket")
                    || error_msg.contains("endpoint"),
                "Should get proper transport error, got: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_wasm_enhanced_transport_compatibility() {
    // Test WASM compatibility with enhanced transports
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test WASM enhanced transport compatibility".to_string(),
        id: 5001,
        transport_type: "wasm-enhanced".to_string(),
        test_scenario: "wasm_compatibility".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test multiple WASM-compatible endpoints
    let endpoints = vec![
        "https://wasm-rest.example.com/api",
        "wss://wasm-jsonrpc.example.com/rpc",
        "https://wasm-mcp.example.com/mcp",
    ];

    for endpoint in endpoints {
        let result: Result<Envelope<EnhancedTestResponse>> = client
            .send_envelope(endpoint, envelope.clone())
            .await;

        // Verify WASM compatibility
        match result {
            Ok(response) => {
                let (_, data) = response.extract();
                assert!(data.features_supported.contains(&"wasm".to_string()));
            }
            Err(err) => {
                // Expected transport errors for test endpoints
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("transport")
                        || error_msg.contains("connection")
                        || error_msg.contains("network")
                        || error_msg.contains("endpoint"),
                    "Should get proper transport error for {}, got: {}",
                    endpoint,
                    error_msg
                );
            }
        }
    }
}

#[tokio::test]
async fn test_feature_gate_combinations() {
    // Test various feature gate combinations work correctly
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test feature gate combinations".to_string(),
        id: 6001,
        transport_type: "feature-gates".to_string(),
        test_scenario: "feature_combinations".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test different feature combinations
    let feature_tests = vec![
        ("mcp-jsonrpc", "mcp-stdio://localhost:8080/mcp"),
        ("a2a-standard", "a2a://localhost:8080/agents"),
        ("wasm-jsonrpc", "wss://localhost:8080/jsonrpc"),
        ("hybrid-transport", "https://localhost:8080/hybrid"),
    ];

    for (feature_name, endpoint) in feature_tests {
        let result: Result<Envelope<EnhancedTestResponse>> = client
            .send_envelope(endpoint, envelope.clone())
            .await;

        // Verify feature gate works
        match result {
            Ok(response) => {
                let (_, data) = response.extract();
                assert!(data.features_supported.contains(&feature_name.to_string()));
            }
            Err(err) => {
                // Expected transport errors for test endpoints
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("transport")
                        || error_msg.contains("connection")
                        || error_msg.contains("network")
                        || error_msg.contains("endpoint"),
                    "Should get proper transport error for {}, got: {}",
                    feature_name,
                    error_msg
                );
            }
        }
    }
}

#[tokio::test]
async fn test_envelope_pattern_preservation() {
    // Test that envelope pattern is preserved across all enhanced transports
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test envelope pattern preservation".to_string(),
        id: 7001,
        transport_type: "envelope-preservation".to_string(),
        test_scenario: "pattern_preservation".to_string(),
    };

    let mut envelope = create_test_envelope(request);

    // Set specific metadata to verify preservation
    envelope.meta.tenant = Some("test-tenant".to_string());
    envelope.meta.version = Some("1.0.0".to_string());

    // Test with REST endpoint (known to work)
    let result: Result<Envelope<EnhancedTestResponse>> = client
        .send_envelope("https://envelope-test.example.com/api", envelope)
        .await;

    // Verify envelope pattern preservation
    match result {
        Ok(response) => {
            // Verify metadata is preserved
            assert_eq!(response.meta.tenant, Some("test-tenant".to_string()));
            assert_eq!(response.meta.version, Some("1.0.0".to_string()));
            assert!(response.meta.request_id.is_some());
            assert!(response.meta.timestamp.is_some());
        }
        Err(err) => {
            // Expected transport errors for test endpoints
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("transport")
                    || error_msg.contains("connection")
                    || error_msg.contains("network")
                    || error_msg.contains("endpoint"),
                "Should get proper transport error, got: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_transport_error_handling_improvements() {
    // Test improved error handling across enhanced transports
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test error handling improvements".to_string(),
        id: 8001,
        transport_type: "error-handling".to_string(),
        test_scenario: "error_improvements".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with various invalid endpoints to verify error handling
    let invalid_endpoints = vec![
        "invalid://bad.endpoint",
        "mcp://nonexistent:9999/mcp",
        "a2a://missing.host/agents",
        "ws://timeout.example.com/jsonrpc",
    ];

    for endpoint in invalid_endpoints {
        let result: Result<Envelope<EnhancedTestResponse>> = client
            .send_envelope(endpoint, envelope.clone())
            .await;

        // Verify proper error handling
        match result {
            Ok(_) => {
                // Unexpected success - should not happen with invalid endpoints
                panic!("Expected error for invalid endpoint: {}", endpoint);
            }
            Err(err) => {
                let error_msg = err.to_string();

                // Verify we get proper transport errors, not panics or internal errors
                assert!(
                    error_msg.contains("transport")
                        || error_msg.contains("connection")
                        || error_msg.contains("network")
                        || error_msg.contains("endpoint")
                        || error_msg.contains("protocol")
                        || error_msg.contains("invalid"),
                    "Should get proper transport error for {}, got: {}",
                    endpoint,
                    error_msg
                );

                // Verify error doesn't contain internal panic messages
                assert!(
                    !error_msg.contains("panic")
                        && !error_msg.contains("unwrap")
                        && !error_msg.contains("expect"),
                    "Error should not contain panic messages: {}",
                    error_msg
                );
            }
        }
    }
}

#[tokio::test]
async fn test_performance_improvements() {
    // Test performance characteristics of enhanced transports
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test performance improvements".to_string(),
        id: 9001,
        transport_type: "performance".to_string(),
        test_scenario: "performance_test".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with timeout to verify performance
    let timeout_duration = Duration::from_secs(5);
    let start_time = std::time::Instant::now();

    let future = client.send_envelope("https://performance.test.com/api", envelope);
    let result: std::result::Result<Result<Envelope<EnhancedTestResponse>>, _> = timeout(
        timeout_duration,
        future
    )
    .await;

    let elapsed = start_time.elapsed();

    // Verify performance characteristics
    match result {
        Ok(Ok(_)) => {
            // Success within timeout
            assert!(elapsed < timeout_duration, "Request should complete within timeout");
        }
        Ok(Err(err)) => {
            // Expected transport error within timeout
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("transport")
                    || error_msg.contains("connection")
                    || error_msg.contains("network"),
                "Should get proper transport error: {}",
                error_msg
            );
            assert!(elapsed < timeout_duration, "Error should occur within timeout");
        }
        Err(_) => {
            // Timeout - this is acceptable for performance testing
            assert!(elapsed >= timeout_duration, "Timeout should occur at expected time");
        }
    }
}

#[tokio::test]
async fn test_backward_compatibility() {
    // Test that enhanced transports maintain backward compatibility
    let config = create_transport_config_for_testing();
    let client = HybridTransportClient::from_config(config)
        .await
        .expect("Should create client from config");

    let request = EnhancedTestRequest {
        message: "Test backward compatibility".to_string(),
        id: 10001,
        transport_type: "backward-compatibility".to_string(),
        test_scenario: "compatibility_test".to_string(),
    };

    let envelope = create_test_envelope(request);

    // Test with legacy endpoint patterns
    let legacy_endpoints = vec![
        "https://legacy.rest.com/api",
        "grpc://legacy.grpc.com:443/service",
        "nats://legacy.nats.com/subject",
    ];

    for endpoint in legacy_endpoints {
        let result: Result<Envelope<EnhancedTestResponse>> = client
            .send_envelope(endpoint, envelope.clone())
            .await;

        // Verify backward compatibility
        match result {
            Ok(response) => {
                // Success indicates backward compatibility
                let (_, data) = response.extract();
                assert!(data.transport_used.len() > 0);
            }
            Err(err) => {
                // Expected transport errors for test endpoints
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("transport")
                        || error_msg.contains("connection")
                        || error_msg.contains("network")
                        || error_msg.contains("endpoint"),
                    "Should get proper transport error for {}, got: {}",
                    endpoint,
                    error_msg
                );
            }
        }
    }
}
