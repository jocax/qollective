// Debug test to isolate the GET call issue
use qollective::client::common::ClientConfig;
use qollective::client::rest::{RestClient, RestClientConfig};
use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

#[tokio::test]
async fn test_debug_rest_client_get_call() {
    // Create a basic config
    let config = RestClientConfig {
        base: ClientConfig {
            base_url: "http://127.0.0.1:8080".to_string(),
            timeout_seconds: 10,
            retry_attempts: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    println!("🔧 Creating RestClient...");
    let client = RestClient::new(config)
        .await
        .expect("Failed to create RestClient");

    println!("✅ RestClient created successfully!");

    // Check if internal client is configured
    if let Some(client_config) = client.config() {
        println!("✅ Internal REST client is configured");
        println!("   - Base URL: {}", client_config.base.base_url);
    } else {
        println!("❌ Internal REST client is NOT configured!");
        panic!("Internal REST client not configured");
    }

    // Create a simple envelope
    let test_data = serde_json::json!({"test": "data"});
    let envelope = Envelope::new(Meta::default(), test_data);

    println!("🚀 Attempting GET call (expected to fail due to no server)...");

    // Try the GET call - this should fail with a connection error, not "No REST client configured"
    let result: Result<Envelope<Value>, _> = client.get("/test", envelope).await;

    match result {
        Ok(_) => println!("✅ Unexpected success - server must be running!"),
        Err(e) => {
            let error_str = e.to_string();
            println!("❌ GET call failed with error: {}", error_str);

            // Check the specific error - if it's "No REST client configured", that's our bug
            if error_str.contains("No REST client configured") {
                panic!("🐛 BUG CONFIRMED: RestClient delegation not working properly");
            } else {
                println!("✅ Expected error (connection/network issue, not delegation issue)");
            }
        }
    }
}
