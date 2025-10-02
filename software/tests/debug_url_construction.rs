//! Debug test to check URL construction in REST client

use qollective::client::common::ClientConfig;
use qollective::client::rest::{RestClient, RestClientConfig};
use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

#[tokio::test]
async fn debug_url_construction() {
    println!("🔍 Debug: Checking REST client URL construction");

    // Create a simple test config similar to the failing test
    let config = RestClientConfig {
        base: ClientConfig {
            base_url: "http://127.0.0.1:12345".to_string(),
            timeout_seconds: 10,
            retry_attempts: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    println!("🔍 Debug: Config base_url = {}", config.base.base_url);

    // Create the client
    let client = RestClient::new(config)
        .await
        .expect("Failed to create client");

    // Print the final config
    if let Some(client_config) = client.config() {
        println!(
            "🔍 Debug: Final client config base_url = {}",
            client_config.base.base_url
        );
    } else {
        println!("❌ Debug: No client config available");
    }

    // Create a test envelope
    let envelope = Envelope::new(Meta::default(), Value::Null);

    // Try to make a GET request (this will fail but we can see the URL in the error)
    let path = "/get";
    println!("🔍 Debug: Attempting GET request to path: {}", path);
    println!(
        "🔍 Debug: Expected final URL should be: {}{}",
        if let Some(config) = client.config() {
            &config.base.base_url
        } else {
            "unknown"
        },
        path
    );

    let result: Result<Envelope<Value>, _> = client.get(path, envelope).await;
    match result {
        Ok(_) => println!("✅ Debug: GET request succeeded (unexpected)"),
        Err(e) => {
            let error_msg = e.to_string();
            println!("❌ Debug: GET request failed (expected): {}", error_msg);

            // Check if the error contains any URL information
            if error_msg.contains("http") {
                println!("🔍 Debug: Error contains URL information");
            }
            if error_msg.contains("roundtrip") {
                println!("🚨 Debug: Error contains 'roundtrip' - this is the bug!");
            }
        }
    }

    println!("🔍 Debug: URL construction test completed");
}
