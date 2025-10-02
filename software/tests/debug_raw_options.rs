// Debug test for raw OPTIONS HTTP request

use qollective::prelude::UnifiedEnvelopeReceiver;
use qollective::server::rest::{MetadataEncoding, MetadataHandlingConfig, RestServer};
use tokio::time::Duration;

mod common;
use common::{get_available_port, rest_test_utils::*, setup_test_environment};

#[tokio::test]
async fn debug_raw_options_request() {
    setup_test_environment();

    let port = get_available_port();
    println!("🔍 Using port: {}", port);

    // Setup server without CORS to test OPTIONS handling
    let server_config = qollective::server::rest::RestServerConfig {
        base: qollective::server::common::ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        cors: None, // Disable CORS for this test
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
    let handler = TestEnvelopeHandler::new("debug-raw-options");
    server
        .receive_envelope_at("/debug", handler)
        .await
        .expect("Failed to register handler");

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("🔍 Server started");

    // Make raw HTTP OPTIONS request using reqwest directly
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/debug", port);

    // Create test data
    let test_data = serde_json::json!({
        "message": "debug raw OPTIONS test",
        "method": "options",
        "test_id": "debug-raw-123"
    });

    let data_json = serde_json::to_string(&test_data).expect("Failed to serialize test data");
    println!("🔍 Sending envelope_data: {}", data_json);

    let response = client
        .request(reqwest::Method::OPTIONS, &url)
        .query(&[("envelope_data", data_json)])
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("✅ Raw OPTIONS request successful!");
            println!("📊 Status: {}", resp.status());
            println!("📋 Headers: {:?}", resp.headers());

            let body = resp.text().await.expect("Failed to read response body");
            println!("📦 Response body: {}", body);

            if body.is_empty() {
                println!("❌ ERROR: Response body is empty!");
            } else {
                println!("✅ Response body is not empty");

                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(&body) {
                    Ok(json) => println!("✅ Response parsed as JSON: {:?}", json),
                    Err(e) => println!("❌ Failed to parse response as JSON: {}", e),
                }
            }
        }
        Err(e) => {
            println!("❌ Raw OPTIONS request failed: {}", e);
            panic!("Raw OPTIONS request failed: {}", e);
        }
    }

    // Cleanup
    server_handle.abort();
    println!("✅ Debug raw test completed");
}
