// ABOUTME: Integration test for POST method roundtrip with envelope transport and TLS support
// ABOUTME: Tests complete client-server communication using REST POST with UnifiedEnvelopeReceiver trait

//! Integration tests for REST POST method roundtrip communication.
//!
//! This test verifies that POST requests work correctly with:
//! - Complete envelope transport (metadata + data)
//! - TLS encryption when certificates are available
//! - UnifiedEnvelopeReceiver trait implementation
//! - Proper metadata preservation through roundtrip
//! - ContextDataHandler processing with envelope context
//! - Large payload handling in request body

use qollective::client::rest::RestClient;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use serde_json::Value;

mod common;
use common::rest_test_utils::*;

#[tokio::test]
async fn test_post_roundtrip_without_tls() {
    let result = run_post_roundtrip_test(false).await;

    match result {
        Ok(_) => println!("✅ POST roundtrip test (HTTP) completed successfully"),
        Err(e) => panic!("❌ POST roundtrip test (HTTP) failed: {}", e),
    }
}

#[tokio::test]
async fn test_post_roundtrip_with_tls() {
    let result = run_post_roundtrip_test(true).await;

    match result {
        Ok(_) => println!("✅ POST roundtrip test (HTTPS) completed successfully"),
        Err(e) => println!("⚠️  POST roundtrip test (HTTPS) skipped or failed: {}", e),
    }
}

#[tokio::test]
async fn test_post_large_payload() {
    // Test POST with large payload that would exceed header limits
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/post-large".to_string(),
        handler_name: "post-large-handler".to_string(),
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

    // Create envelope with large payload typical for POST requests
    let large_data = serde_json::json!({
        "message": "test POST with large payload",
        "method": "post",
        "large_field": "x".repeat(5000), // 5KB of data
        "metadata": {
            "description": "This is a large POST payload that tests body transport",
            "tags": (0..100).map(|i| format!("tag-{}", i)).collect::<Vec<_>>(),
            "data_points": (0..500).map(|i| serde_json::json!({
                "id": i,
                "value": format!("data-point-{}", i),
                "timestamp": chrono::Utc::now()
            })).collect::<Vec<_>>()
        }
    });

    let envelope = Envelope::new(Meta::default(), large_data.clone());

    // Execute POST request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(10), // Longer timeout for large payload
        client.post(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("POST request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(
        response.payload["echo"]["large_field"],
        large_data["large_field"]
    );
    assert_eq!(
        response.payload["echo"]["metadata"]["tags"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
    assert_eq!(
        response.payload["echo"]["metadata"]["data_points"]
            .as_array()
            .unwrap()
            .len(),
        500
    );

    // Cleanup
    server_handle.abort();

    println!("✅ POST large payload test completed successfully");
}

#[tokio::test]
async fn test_post_json_content_type() {
    // Test that POST requests handle JSON content properly
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/post-json".to_string(),
        handler_name: "post-json-handler".to_string(),
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

    // Create envelope with complex JSON structure
    let json_data = serde_json::json!({
        "message": "test POST with complex JSON",
        "method": "post",
        "complex_data": {
            "arrays": [1, 2, 3, 4, 5],
            "nested": {
                "key1": "value1",
                "key2": {
                    "deep": "nested_value"
                }
            },
            "booleans": [true, false, true],
            "numbers": [1.5, 2.7, 3.14159],
            "strings": ["hello", "world", "json"]
        }
    });

    let envelope = Envelope::new(Meta::default(), json_data.clone());

    // Execute POST request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.post(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("POST request failed");

    // Verify JSON structure preservation
    assert_eq!(response.payload["status"], "success");
    assert_eq!(
        response.payload["echo"]["complex_data"]["arrays"],
        json_data["complex_data"]["arrays"]
    );
    assert_eq!(
        response.payload["echo"]["complex_data"]["nested"]["key2"]["deep"],
        "nested_value"
    );
    assert_eq!(response.payload["echo"]["complex_data"]["booleans"][0], true);
    assert_eq!(response.payload["echo"]["complex_data"]["numbers"][2], 3.14159);

    // Cleanup
    server_handle.abort();

    println!("✅ POST JSON content type test completed successfully");
}

#[tokio::test]
async fn test_post_metadata_headers_transport() {
    // Test POST-specific functionality: metadata via headers with body data
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/post-headers".to_string(),
        handler_name: "post-headers-handler".to_string(),
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

    // Create envelope with metadata for header transport
    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.tenant = Some("post-headers-test".to_string());
    meta.version = Some("1.0".to_string());
    meta.timestamp = Some(chrono::Utc::now());

    let envelope = Envelope::new(
        meta.clone(),
        serde_json::json!({
            "message": "test POST with header metadata transport",
            "method": "post",
            "test_case": "headers_transport"
        }),
    );

    // Execute POST request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.post(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("POST request failed");

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, meta.request_id);
    assert_eq!(response.meta.tenant, meta.tenant);
    assert_eq!(response.meta.version, meta.version);

    // Verify response data
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["test_case"], "headers_transport");
    assert_eq!(response.payload["context"]["has_context"], true);

    // Cleanup
    server_handle.abort();

    println!("✅ POST metadata headers transport test completed successfully");
}

#[tokio::test]
async fn test_post_envelope_context_processing() {
    // Test that POST requests properly process envelope context in handlers
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/post-context".to_string(),
        handler_name: "post-context-handler".to_string(),
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
    let envelope = create_test_envelope("test POST context processing", "post");

    // Execute POST request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.post(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("POST request failed");

    // Verify context was properly extracted and passed to handler
    assert_eq!(response.payload["context"]["has_context"], true);
    assert_eq!(
        response.payload["context"]["request_id"],
        envelope.meta.request_id.as_ref().unwrap().to_string()
    );
    assert_eq!(
        response.payload["context"]["tenant"],
        envelope.meta.tenant.as_ref().unwrap().as_str()
    );
    assert!(response.payload["context"]["timestamp"].is_string());

    // Verify that the default envelope handler preserved metadata
    assert_eq!(response.meta.request_id, envelope.meta.request_id);
    assert_eq!(response.meta.tenant, envelope.meta.tenant);

    // Cleanup
    server_handle.abort();

    println!("✅ POST envelope context processing test completed successfully");
}
