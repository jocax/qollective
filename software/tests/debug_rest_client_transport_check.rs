// Debug test to check exactly when the transport loses the internal client
use qollective::client::common::ClientConfig;
use qollective::client::rest::{RestClient, RestClientConfig};
use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

#[tokio::test]
async fn test_transport_internal_client_persistence() {
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

    // Check internal client immediately after creation
    println!("🔍 Checking internal client after creation...");
    if let Some(_) = client.config() {
        println!("✅ Internal REST client configured after creation");
    } else {
        println!("❌ Internal REST client NOT configured after creation");
        panic!("Internal REST client lost after creation");
    }

    // RestClient doesn't implement Clone, so skip this test section

    // Move the client (simulate what happens when passing to functions)
    println!("➡️  Moving client to function...");
    let moved_client = simulate_client_move(client).await;

    println!("🔍 Checking client after move...");
    if let Some(_) = moved_client.config() {
        println!("✅ Moved client has internal REST client");
    } else {
        println!("❌ Moved client does NOT have internal REST client");
        panic!("Moved client lost internal REST client");
    }

    // Try a simple envelope operation with the moved client
    println!("🚀 Testing envelope operation with moved client...");
    let envelope = Envelope::new(Meta::default(), serde_json::json!({"test": "data"}));

    let result: Result<Envelope<Value>, _> = moved_client.get("/test", envelope).await;
    match result {
        Ok(_) => println!("✅ Unexpected success - envelope operation worked"),
        Err(e) => {
            let error_str = e.to_string();
            println!("❌ Envelope operation failed: {}", error_str);

            if error_str.contains("No REST client configured") {
                panic!("🐛 BUG: Internal REST client lost somewhere in the chain!");
            } else {
                println!("✅ Expected error (network/connection issue, not delegation issue)");
            }
        }
    }
}

async fn simulate_client_move(client: RestClient) -> RestClient {
    println!("   🔍 Checking client inside function...");
    if let Some(_) = client.config() {
        println!("   ✅ Client has internal REST client inside function");
    } else {
        println!("   ❌ Client does NOT have internal REST client inside function");
    }
    client
}

// RestClient doesn't implement Clone, so remove the struct cloning test
