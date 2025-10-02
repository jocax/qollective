// Test for smart CORS middleware with per-route OPTIONS configuration

use qollective::client::rest::RestClient;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use qollective::prelude::*;
use qollective::server::rest::{
    MetadataEncoding, MetadataHandlingConfig, OptionsBehavior, RestServer, RestServerConfig,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

mod common;
use common::{get_available_port, rest_test_utils::*, setup_test_environment};

#[tokio::test]
async fn test_smart_cors_application_options() {
    setup_test_environment();

    let port = get_available_port();
    println!("🔍 Using port: {}", port);

    // Setup server with CORS enabled
    let server_config = qollective::server::rest::RestServerConfig {
        base: qollective::server::common::ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        cors: Some(qollective::server::rest::CorsConfig::permissive()), // Enable CORS
        metadata: MetadataHandlingConfig {
            max_header_size: 4096,
            max_total_headers: 16384,
            encoding: MetadataEncoding::Base64,
        },
        ..Default::default()
    };

    let mut server = RestServer::new(server_config)
        .await
        .expect("Failed to create server");

    // Register a handler
    let handler = TestEnvelopeHandler::new("smart-cors-test");
    server
        .receive_envelope_at("/test", handler)
        .await
        .expect("Failed to register handler");

    // Configure this route to allow application OPTIONS
    server
        .set_options_behavior("/test", OptionsBehavior::Application)
        .await
        .expect("Failed to set OPTIONS behavior");

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("🔍 Server started with smart CORS");

    // Create client
    let client_config = qollective::client::rest::RestClientConfig {
        base: qollective::client::common::ClientConfig {
            base_url: format!("http://127.0.0.1:{}", port),
            timeout_seconds: 10,
            retry_attempts: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    let client = RestClient::new(client_config)
        .await
        .expect("Failed to create client");
    println!("🔍 Client created");

    // Create test envelope
    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.tenant = Some("smart-cors-tenant".to_string());
    meta.version = Some("1.0".to_string());

    let data = json!({
        "message": "test smart CORS OPTIONS",
        "method": "options",
        "test_id": "smart-cors-123"
    });

    let envelope = Envelope::new(meta.clone(), data);
    println!("🔍 Created envelope: {:?}", envelope);

    // Try the OPTIONS request - should work with smart CORS
    println!("🔍 Making OPTIONS request...");
    let result: Result<Envelope<Value>> = timeout(
        Duration::from_secs(10),
        client.options("/test", envelope.clone()),
    )
    .await
    .expect("Request timed out");

    match result {
        Ok(response) => {
            println!("✅ Smart CORS OPTIONS request successful!");
            println!("📦 Response: {:?}", response);

            // Verify response structure
            assert_eq!(response.payload["status"], "success");
            assert_eq!(response.payload["handler"], "smart-cors-test");
            assert_eq!(response.payload["echo"]["method"], "options");
        }
        Err(e) => {
            println!("❌ Smart CORS OPTIONS request failed: {}", e);
            panic!("Smart CORS OPTIONS request failed: {}", e);
        }
    }

    // Cleanup
    server_handle.abort();
    println!("✅ Smart CORS test completed successfully");
}

#[tokio::test]
async fn test_smart_cors_preflight_detection() {
    setup_test_environment();

    let port = get_available_port();
    println!("🔍 Using port: {} for preflight test", port);

    // Setup server with CORS enabled
    let server_config = qollective::server::rest::RestServerConfig {
        base: qollective::server::common::ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        cors: Some(qollective::server::rest::CorsConfig::permissive()), // Enable CORS
        metadata: MetadataHandlingConfig {
            max_header_size: 4096,
            max_total_headers: 16384,
            encoding: MetadataEncoding::Base64,
        },
        ..Default::default()
    };

    let mut server = RestServer::new(server_config)
        .await
        .expect("Failed to create server");

    // Register a handler
    let handler = TestEnvelopeHandler::new("preflight-test");
    server
        .receive_envelope_at("/preflight", handler)
        .await
        .expect("Failed to register handler");

    // Keep default behavior (CorsOnly) for this route

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("🔍 Server started for preflight test");

    // Make a raw OPTIONS request that looks like CORS preflight
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/preflight", port);

    let response = client
        .request(reqwest::Method::OPTIONS, &url)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .header("origin", "https://example.com")
        .send()
        .await
        .expect("Failed to send preflight request");

    println!("✅ Preflight request successful!");
    println!("📊 Status: {}", response.status());
    println!("📋 Headers: {:?}", response.headers());

    // Verify it's a proper CORS response (not our application response)
    assert_eq!(response.status(), 200);
    assert!(response
        .headers()
        .contains_key("access-control-allow-origin"));
    assert!(response
        .headers()
        .contains_key("access-control-allow-methods"));

    let body = response.text().await.expect("Failed to read response body");
    println!("📦 Response body: '{}'", body);

    // Should be empty (CORS preflight response, not application response)
    assert!(body.is_empty(), "Preflight response should have empty body");

    // Cleanup
    server_handle.abort();
    println!("✅ Preflight detection test completed successfully");
}
