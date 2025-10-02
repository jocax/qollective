//! Debug test to isolate the GET roundtrip issue

use serde_json::Value;

mod common;
use common::rest_test_utils::*;

#[tokio::test]
async fn debug_get_roundtrip_error() {
    println!("🔍 Debug: Starting isolated GET roundtrip test");

    // Use the exact same config as the failing test
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/get".to_string(),
        handler_name: "get-handler".to_string(),
        ..Default::default()
    };

    println!(
        "🔍 Debug: Config - endpoint: {}, port: {}",
        config.endpoint, config.port
    );

    // Setup server
    println!("🔍 Debug: Setting up test server...");
    let server_handle = match setup_test_rest_server(config.clone()).await {
        Ok(handle) => {
            println!("✅ Debug: Server setup successful");
            handle
        }
        Err(e) => {
            println!("❌ Debug: Server setup failed: {}", e);
            panic!("Server setup failed: {}", e);
        }
    };

    // Create client
    println!("🔍 Debug: Creating test client...");
    let client = match create_test_rest_client(&config).await {
        Ok(client) => {
            println!("✅ Debug: Client creation successful");
            client
        }
        Err(e) => {
            println!("❌ Debug: Client creation failed: {}", e);
            server_handle.abort();
            panic!("Client creation failed: {}", e);
        }
    };

    // Print client configuration for debugging
    if let Some(client_config) = client.config() {
        println!("🔍 Debug: Client base URL: {}", client_config.base.base_url);
    }

    // Create test envelope
    println!("🔍 Debug: Creating test envelope...");
    let request_envelope = create_test_envelope("test GET roundtrip", "get");
    println!(
        "🔍 Debug: Request envelope meta: tenant={:?}, version={:?}, request_id={:?}",
        request_envelope.meta.tenant,
        request_envelope.meta.version,
        request_envelope.meta.request_id
    );

    // Execute GET request with detailed error handling
    println!(
        "🔍 Debug: Executing GET request to path: {}",
        config.endpoint
    );
    let timeout_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.get(&config.endpoint, request_envelope.clone()),
    )
    .await;

    match timeout_result {
        Ok(Ok(response_envelope)) => {
            println!("✅ Debug: GET request successful!");
            println!(
                "🔍 Debug: Response meta: tenant={:?}, version={:?}, request_id={:?}",
                response_envelope.meta.tenant,
                response_envelope.meta.version,
                response_envelope.meta.request_id
            );
            println!(
                "🔍 Debug: Response data: {}",
                serde_json::to_string_pretty::<Value>(&response_envelope.payload).unwrap_or_default()
            );

            // Now test the verification step that the original test does
            println!("🔍 Debug: Testing verification step...");
            match std::panic::catch_unwind(|| {
                verify_roundtrip_response(
                    &request_envelope,
                    &response_envelope,
                    "get",
                    &config.handler_name,
                );
            }) {
                Ok(_) => {
                    println!("✅ Debug: Verification passed!");
                }
                Err(panic_payload) => {
                    if let Some(msg) = panic_payload.downcast_ref::<String>() {
                        println!("❌ Debug: Verification panicked with: {}", msg);
                    } else if let Some(msg) = panic_payload.downcast_ref::<&str>() {
                        println!("❌ Debug: Verification panicked with: {}", msg);
                    } else {
                        println!("❌ Debug: Verification panicked with unknown error");
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ Debug: GET request failed with error: {}", e);
            println!("🔍 Debug: Error type: {:?}", e);
            // This is the key - let's see the actual error message
        }
        Err(_timeout) => {
            println!("❌ Debug: GET request timed out after 5 seconds");
        }
    }

    // Cleanup
    server_handle.abort();

    println!("🔍 Debug: Test completed");
}
