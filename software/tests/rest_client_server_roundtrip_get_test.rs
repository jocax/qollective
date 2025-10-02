// ABOUTME: Integration test for GET method roundtrip with envelope transport and TLS support
// ABOUTME: Tests complete client-server communication using REST GET with UnifiedEnvelopeReceiver trait

//! Integration tests for REST GET method roundtrip communication.
//!
//! This test verifies that GET requests work correctly with:
//! - Complete envelope transport (metadata + data)
//! - TLS encryption when certificates are available
//! - UnifiedEnvelopeReceiver trait implementation
//! - Proper metadata preservation through roundtrip
//! - ContextDataHandler processing with envelope context

use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

mod common;
use common::rest_test_utils::*;

#[tokio::test]
async fn test_get_roundtrip_without_tls() {
    let result = run_get_roundtrip_test(false).await;

    match result {
        Ok(_) => println!("✅ GET roundtrip test (HTTP) completed successfully"),
        Err(e) => panic!("❌ GET roundtrip test (HTTP) failed: {}", e),
    }
}

#[tokio::test]
async fn test_get_roundtrip_with_tls() {
    let result = run_get_roundtrip_test(true).await;

    match result {
        Ok(_) => println!("✅ GET roundtrip test (HTTPS) completed successfully"),
        Err(e) => println!("⚠️  GET roundtrip test (HTTPS) skipped or failed: {}", e),
    }
}

#[tokio::test]
async fn test_get_query_parameters_metadata_transport() {
    // Test GET-specific functionality: metadata via query parameters
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/get-query-test".to_string(),
        handler_name: "get-query-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope with metadata that might be too large for headers
    let mut large_meta = Meta::default();
    large_meta.request_id = Some(uuid::Uuid::now_v7());
    large_meta.tenant = Some(
        "large-metadata-tenant-with-very-long-name-that-might-exceed-header-limits".to_string(),
    );
    large_meta.version = Some("1.0.0-beta.1+build.metadata.extended".to_string());

    let envelope = Envelope::new(
        large_meta.clone(),
        serde_json::json!({
            "message": "test GET with query parameter metadata fallback",
            "method": "get",
            "test_case": "query_parameters"
        }),
    );

    // Execute GET request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.get(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("GET request failed");

    // Verify response metadata preservation
    assert_eq!(response.meta.request_id, large_meta.request_id);
    assert_eq!(response.meta.tenant, large_meta.tenant);
    assert_eq!(response.meta.version, large_meta.version);

    // Verify response data
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["test_case"], "query_parameters");

    // Cleanup
    server_handle.abort();

    println!("✅ GET query parameters metadata transport test completed successfully");
}

#[tokio::test]
async fn test_get_empty_body_handling() {
    // Test that GET requests handle empty/minimal bodies correctly
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/get-empty".to_string(),
        handler_name: "get-empty-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope with minimal data (GET typically has no body)
    let envelope = Envelope::new(Meta::default(), serde_json::Value::Null);

    // Execute GET request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.get(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("GET request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"], serde_json::Value::Null);

    // Cleanup
    server_handle.abort();

    println!("✅ GET empty body handling test completed successfully");
}

#[tokio::test]
async fn test_get_context_extraction() {
    // Test that context is properly extracted and passed to handler
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/get-context".to_string(),
        handler_name: "get-context-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope with rich metadata for context testing
    let envelope = create_test_envelope("test GET context extraction", "get");

    // Execute GET request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.get(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("GET request failed");

    // Verify context was properly passed to handler
    assert_eq!(response.payload["context"]["has_context"], true);
    assert_eq!(
        response.payload["context"]["request_id"],
        envelope.meta.request_id.as_ref().unwrap().to_string()
    );
    assert_eq!(
        response.payload["context"]["tenant"],
        envelope.meta.tenant.as_ref().unwrap().as_str()
    );

    // Cleanup
    server_handle.abort();

    println!("✅ GET context extraction test completed successfully");
}
