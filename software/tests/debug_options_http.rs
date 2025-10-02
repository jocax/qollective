// Debug test for actual OPTIONS HTTP roundtrip

use qollective::client::rest::RestClient;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use qollective::prelude::*;
use qollective::server::rest::{
    MetadataEncoding, MetadataHandlingConfig, RestServer, RestServerConfig,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

mod common;
use common::{get_available_port, rest_test_utils::*, setup_test_environment};

#[tokio::test]
async fn debug_options_http_request() {
    setup_test_environment();

    let port = get_available_port();
    println!("🔍 Using port: {}", port);

    // Setup server
    let server_config = qollective::server::rest::RestServerConfig {
        base: qollective::server::common::ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
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

    // Register a simple handler
    let handler = TestEnvelopeHandler::new("debug-options");
    server
        .receive_envelope_at("/debug", handler)
        .await
        .expect("Failed to register handler");

    // Configure this route to allow application OPTIONS (smart CORS)
    server
        .set_options_behavior(
            "/debug",
            qollective::server::rest::OptionsBehavior::Application,
        )
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
    println!("🔍 Server started");

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
    meta.tenant = Some("debug-tenant".to_string());
    meta.version = Some("1.0".to_string());

    let data = json!({
        "message": "debug OPTIONS test",
        "method": "options",
        "test_id": "debug-123"
    });

    let envelope = Envelope::new(meta.clone(), data);
    println!("🔍 Created envelope: {:?}", envelope);

    // Try the OPTIONS request
    println!("🔍 Making OPTIONS request...");
    let result: Result<Envelope<Value>> = timeout(
        Duration::from_secs(10),
        client.options("/debug", envelope.clone()),
    )
    .await
    .expect("Request timed out");

    match result {
        Ok(response) => {
            println!("✅ OPTIONS request successful!");
            println!("📦 Response: {:?}", response);

            // Verify response structure
            assert_eq!(response.payload["status"], "success");
            assert_eq!(response.payload["handler"], "debug-options");
            assert_eq!(response.payload["echo"]["method"], "options");
        }
        Err(e) => {
            println!("❌ OPTIONS request failed: {}", e);
            panic!("OPTIONS request failed: {}", e);
        }
    }

    // Cleanup
    server_handle.abort();
    println!("✅ Debug test completed");
}
