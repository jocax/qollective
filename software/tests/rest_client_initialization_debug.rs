// Debug test to verify RestClient initialization works correctly
use qollective::client::common::ClientConfig;
use qollective::client::rest::{RestClient, RestClientConfig};

#[tokio::test]
async fn test_rest_client_initialization_debug() {
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

    println!("🔧 Creating RestClient with config...");

    // Try to create a REST client
    let client = RestClient::new(config)
        .await
        .expect("Failed to create RestClient");

    println!("✅ RestClient created successfully!");

    // Check if internal client is configured
    if let Some(client_config) = client.config() {
        println!("✅ Internal REST client is configured:");
        println!("   - Base URL: {}", client_config.base.base_url);
        println!(
            "   - Timeout: {} seconds",
            client_config.base.timeout_seconds
        );

        // This should pass if initialization worked
        assert_eq!(client_config.base.base_url, "http://127.0.0.1:8080");
        assert_eq!(client_config.base.timeout_seconds, 10);
    } else {
        panic!("❌ Internal REST client is NOT configured!");
    }

    println!("🎯 RestClient initialization test passed!");
}
