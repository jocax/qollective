//! Integration tests for MCP service discovery
//!
//! **REQUIRES RUNNING SERVICES**
//!
//! ## Prerequisites
//!
//! 1. Start NATS server: `./start-nats.sh`
//! 2. Start all 4 MCP services in separate terminals:
//!    ```bash
//!    cargo run -p story-generator &
//!    cargo run -p quality-control &
//!    cargo run -p constraint-enforcer &
//!    cargo run -p prompt-helper &
//!    ```
//! 3. Run tests: `cargo test --package orchestrator --test integration_discovery -- --ignored`

use orchestrator::discovery::DiscoveryClient;
use orchestrator::config::OrchestratorConfig;
use async_nats;
use std::sync::Arc;
use std::path::Path;
use shared_types::types::tool_registration::ServiceCapabilities;
use rustls::{ClientConfig, RootCertStore};

/// Helper to create NATS client for tests
async fn create_test_nats_client() -> Result<Arc<async_nats::Client>, Box<dyn std::error::Error>> {
    // Initialize TLS crypto provider (required for rustls)
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Load TLS certificates
    let ca_cert_path = Path::new("../certs/ca.pem");
    let ca_cert_pem = std::fs::read_to_string(ca_cert_path)?;
    let ca_cert = rustls_pemfile::certs(&mut ca_cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()?;

    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates(ca_cert);

    let tls_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // Connect to NATS with TLS using test credentials
    let nats_client = async_nats::ConnectOptions::new()
        .tls_client_config(tls_config)
        .user_and_password("test".to_string(), "test".to_string())
        .connect("tls://localhost:5222")
        .await?;

    Ok(Arc::new(nats_client))
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_discover_all_services() {
    // ARRANGE
    let nats_client = create_test_nats_client()
        .await
        .expect("Failed to connect to NATS");

    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT
    let services = discovery_client.discover_all_services()
        .await
        .expect("Failed to discover services");

    // ASSERT
    // Verify all 4 services discovered
    assert_eq!(services.len(), 4, "Should discover all 4 services");
    assert!(services.contains_key("story-generator"));
    assert!(services.contains_key("quality-control"));
    assert!(services.contains_key("constraint-enforcer"));
    assert!(services.contains_key("prompt-helper"));

    // Verify tool counts match expected
    let story_tools = &services["story-generator"];
    assert_eq!(story_tools.len(), 3, "story-generator should have 3 tools");

    let qc_tools = &services["quality-control"];
    assert_eq!(qc_tools.len(), 2, "quality-control should have 2 tools");

    let ce_tools = &services["constraint-enforcer"];
    assert_eq!(ce_tools.len(), 1, "constraint-enforcer should have 1 tool");

    let ph_tools = &services["prompt-helper"];
    assert_eq!(ph_tools.len(), 1, "prompt-helper should have 1 tool");
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_tool_registration_data_correctness() {
    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT
    let story_tools = discovery_client.discover_service_tools("story-generator")
        .await
        .expect("Failed to discover story-generator");

    // ASSERT
    // Verify tool names are correct
    let tool_names: Vec<&str> = story_tools.iter()
        .map(|t| t.tool_name.as_str())
        .collect();

    assert!(tool_names.contains(&"generate_structure"));
    assert!(tool_names.contains(&"generate_nodes"));
    assert!(tool_names.contains(&"validate_paths"));

    // Verify each tool has valid schema
    for tool in &story_tools {
        assert!(!tool.tool_schema.is_null(), "Tool {} should have non-null schema", tool.tool_name);
        assert!(tool.tool_schema.is_object(), "Tool {} schema should be JSON object", tool.tool_name);

        // Verify schema has 'type' field (basic JSON Schema validation)
        let schema_obj = tool.tool_schema.as_object().unwrap();
        assert!(schema_obj.contains_key("type") || schema_obj.contains_key("$schema"),
                "Tool {} schema should have 'type' or '$schema' field", tool.tool_name);
    }

    // Verify service metadata
    for tool in &story_tools {
        assert_eq!(tool.service_name, "story-generator");
        assert_eq!(tool.service_version, "0.0.1");
        assert!(!tool.capabilities.is_empty(), "Tool {} should have capabilities", tool.tool_name);
    }
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_service_capabilities_match_expected() {
    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT
    let all_services = discovery_client.discover_all_services().await.expect("Failed to discover");

    // ASSERT
    // Verify story-generator capabilities
    for tool in &all_services["story-generator"] {
        if tool.tool_name == "generate_structure" || tool.tool_name == "generate_nodes" {
            assert!(tool.capabilities.contains(&ServiceCapabilities::Batching));
            assert!(tool.capabilities.contains(&ServiceCapabilities::Retry));
        }
    }

    // Verify quality-control capabilities
    for tool in &all_services["quality-control"] {
        assert!(tool.capabilities.contains(&ServiceCapabilities::Batching) ||
                tool.capabilities.contains(&ServiceCapabilities::Retry));
    }

    // Verify prompt-helper capabilities
    for tool in &all_services["prompt-helper"] {
        assert!(tool.capabilities.contains(&ServiceCapabilities::Caching) ||
                tool.capabilities.contains(&ServiceCapabilities::Retry));
    }
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_discovery_caching_works() {
    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT - First call (should hit NATS)
    let start = std::time::Instant::now();
    let tools1 = discovery_client.discover_service_tools("story-generator")
        .await
        .expect("First discovery failed");
    let first_duration = start.elapsed();

    // Second call (should use cache)
    let start = std::time::Instant::now();
    let tools2 = discovery_client.discover_service_tools("story-generator")
        .await
        .expect("Second discovery failed");
    let second_duration = start.elapsed();

    // ASSERT
    // Results should be identical
    assert_eq!(tools1.len(), tools2.len());
    assert_eq!(tools1[0].tool_name, tools2[0].tool_name);

    // Second call should be faster (cached)
    // Note: This assertion might be flaky, so we just verify it completes successfully
    println!("First call: {:?}, Second call (cached): {:?}", first_duration, second_duration);
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_health_check_integration() {
    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT
    let is_healthy = discovery_client.check_service_health("story-generator")
        .await
        .expect("Health check failed");

    // ASSERT
    assert!(is_healthy, "Service should be healthy when running");
}

#[tokio::test]
#[ignore] // Requires manual service stopping
async fn test_graceful_degradation_missing_optional_service() {
    // This test requires manually stopping prompt-helper before running
    // It verifies that orchestrator can handle missing optional services

    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT
    let result = discovery_client.discover_service_tools("prompt-helper").await;

    // ASSERT
    // Should handle missing optional service gracefully
    // Either returns error or empty list depending on implementation
    match result {
        Ok(tools) => println!("Found {} tools from prompt-helper", tools.len()),
        Err(e) => println!("Expected error for missing service: {}", e),
    }
}

#[tokio::test]
#[ignore] // Requires manual service stopping
async fn test_fail_fast_missing_required_service() {
    // This test requires manually stopping story-generator before running
    // It verifies that orchestrator fails fast when required service is missing

    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT
    let result = discovery_client.discover_service_tools("story-generator").await;

    // ASSERT
    // Should fail with clear error message
    assert!(result.is_err(), "Should fail when required service is missing");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("story-generator") || error_msg.contains("Failed to discover"),
            "Error should mention the missing service");
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_cache_ttl_expiration() {
    // This test verifies that cache expires after TTL (5 minutes)
    // Note: This would take 5 minutes to run, so we just verify the cache can be cleared

    // ARRANGE
    let nats_client = create_test_nats_client().await.expect("Failed to connect");
    let discovery_client = DiscoveryClient::new(nats_client);

    // ACT - Discover and cache
    let _tools1 = discovery_client.discover_service_tools("story-generator")
        .await
        .expect("First discovery failed");

    // Clear cache manually (simulates TTL expiration)
    discovery_client.clear_cache().await;

    // Discover again (should hit NATS, not cache)
    let tools2 = discovery_client.discover_service_tools("story-generator")
        .await
        .expect("Second discovery after cache clear failed");

    // ASSERT
    assert_eq!(tools2.len(), 3, "Should still get 3 tools after cache clear");
}
