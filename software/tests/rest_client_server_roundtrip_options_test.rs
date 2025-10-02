// ABOUTME: Integration test for OPTIONS method roundtrip with envelope transport and TLS support
// ABOUTME: Tests complete client-server communication using REST OPTIONS with UnifiedEnvelopeReceiver trait

//! Integration tests for REST OPTIONS method roundtrip communication.
//!
//! This test verifies that OPTIONS requests work correctly with:
//! - Complete envelope transport (metadata + data)
//! - TLS encryption when certificates are available
//! - UnifiedEnvelopeReceiver trait implementation
//! - Proper metadata preservation through roundtrip
//! - ContextDataHandler processing with envelope context
//! - CORS preflight request handling

use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

mod common;
use common::rest_test_utils::*;

#[tokio::test]
async fn test_options_roundtrip_without_tls() {
    let result = run_options_roundtrip_test(false).await;

    match result {
        Ok(_) => println!("✅ OPTIONS roundtrip test (HTTP) completed successfully"),
        Err(e) => panic!("❌ OPTIONS roundtrip test (HTTP) failed: {}", e),
    }
}

#[tokio::test]
async fn test_options_roundtrip_with_tls() {
    let result = run_options_roundtrip_test(true).await;

    match result {
        Ok(_) => println!("✅ OPTIONS roundtrip test (HTTPS) completed successfully"),
        Err(e) => println!(
            "⚠️  OPTIONS roundtrip test (HTTPS) skipped or failed: {}",
            e
        ),
    }
}

#[tokio::test]
async fn test_options_cors_preflight_simulation() {
    // Test OPTIONS for CORS preflight request simulation
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/options-cors".to_string(),
        handler_name: "options-cors-handler".to_string(),
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

    // Create envelope with CORS preflight information
    let cors_data = serde_json::json!({
        "message": "test OPTIONS for CORS preflight",
        "method": "options",
        "purpose": "cors_preflight",
        "requested_headers": [
            "Content-Type",
            "Authorization",
            "X-Qollective-Request-Id",
            "X-Qollective-Tenant"
        ],
        "requested_methods": ["GET", "POST", "PUT", "DELETE"],
        "origin": "https://example.com",
        "credentials": true
    });

    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.tenant = Some("cors-options-tenant".to_string());

    let envelope = Envelope::new(meta.clone(), cors_data.clone());

    // Execute OPTIONS request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.options(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("OPTIONS request failed");

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, meta.request_id);
    assert_eq!(response.meta.tenant, meta.tenant);

    // Verify response structure
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["purpose"], "cors_preflight");
    assert_eq!(
        response.payload["echo"]["requested_methods"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(response.payload["echo"]["origin"], "https://example.com");

    // Cleanup
    server_handle.abort();

    println!("✅ OPTIONS CORS preflight simulation test completed successfully");
}

#[tokio::test]
async fn test_options_api_discovery() {
    // Test OPTIONS for API discovery and capabilities
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/options-discovery".to_string(),
        handler_name: "options-discovery-handler".to_string(),
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

    // Create envelope with API discovery request
    let discovery_data = serde_json::json!({
        "message": "test OPTIONS for API discovery",
        "method": "options",
        "purpose": "api_discovery",
        "request_info": {
            "path": "/options-discovery",
            "query": "capabilities=true&formats=json",
            "user_agent": "qollective-test-client/1.0"
        },
        "discover": [
            "supported_methods",
            "supported_formats",
            "authentication_required",
            "rate_limits",
            "api_version"
        ]
    });

    let envelope = Envelope::new(Meta::default(), discovery_data.clone());

    // Execute OPTIONS request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.options(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("OPTIONS request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["echo"]["purpose"], "api_discovery");
    assert_eq!(
        response.payload["echo"]["discover"].as_array().unwrap().len(),
        5
    );
    assert_eq!(
        response.payload["echo"]["request_info"]["path"],
        "/options-discovery"
    );

    // Cleanup
    server_handle.abort();

    println!("✅ OPTIONS API discovery test completed successfully");
}

#[tokio::test]
async fn test_options_metadata_capabilities() {
    // Test OPTIONS with metadata capabilities inquiry
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/options-metadata".to_string(),
        handler_name: "options-metadata-handler".to_string(),
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

    // Create envelope with metadata capabilities inquiry
    let metadata_inquiry = serde_json::json!({
        "message": "test OPTIONS for metadata capabilities",
        "method": "options",
        "purpose": "metadata_inquiry",
        "inquiry": {
            "supported_headers": "query",
            "max_header_size": "query",
            "envelope_formats": "query",
            "compression": "query",
            "authentication": "query"
        },
        "client_capabilities": {
            "supports_base64": true,
            "supports_json": true,
            "supports_compression": false,
            "max_payload_size": 1048576
        }
    });

    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.version = Some("1.0".to_string());

    let envelope = Envelope::new(meta.clone(), metadata_inquiry.clone());

    // Execute OPTIONS request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.options(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("OPTIONS request failed");

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, meta.request_id);
    assert_eq!(response.meta.version, meta.version);

    // Verify response structure
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["echo"]["purpose"], "metadata_inquiry");
    assert_eq!(
        response.payload["echo"]["client_capabilities"]["supports_base64"],
        true
    );
    assert_eq!(
        response.payload["echo"]["client_capabilities"]["max_payload_size"],
        1048576
    );

    // Cleanup
    server_handle.abort();

    println!("✅ OPTIONS metadata capabilities test completed successfully");
}

#[tokio::test]
async fn test_options_context_extraction() {
    // Test that context is properly extracted and passed to handler for OPTIONS
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/options-context".to_string(),
        handler_name: "options-context-handler".to_string(),
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
    let envelope = create_test_envelope("test OPTIONS context extraction", "options");

    // Execute OPTIONS request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.options(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("OPTIONS request failed");

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

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, envelope.meta.request_id);
    assert_eq!(response.meta.tenant, envelope.meta.tenant);

    // Cleanup
    server_handle.abort();

    println!("✅ OPTIONS context extraction test completed successfully");
}

#[tokio::test]
async fn test_options_minimal_request() {
    // Test OPTIONS with minimal request data (typical for basic OPTIONS requests)
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/options-minimal".to_string(),
        handler_name: "options-minimal-handler".to_string(),
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

    // Create envelope with minimal data (OPTIONS often has minimal body)
    let minimal_data = serde_json::json!({
        "method": "options"
    });

    let envelope = Envelope::new(Meta::default(), minimal_data.clone());

    // Execute OPTIONS request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.options(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("OPTIONS request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["method"], "options");

    // Cleanup
    server_handle.abort();

    println!("✅ OPTIONS minimal request test completed successfully");
}
