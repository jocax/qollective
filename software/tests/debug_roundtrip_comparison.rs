// ABOUTME: Debug test comparing working debug path test vs failing run_websocket_roundtrip_test
// ABOUTME: Isolates the specific difference between working and failing WebSocket test patterns

use std::time::Duration;

mod common;
use common::websocket_test_utils::*;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use serde_json::Value;

#[tokio::test]
async fn test_exact_roundtrip_comparison() {
    println!("🔍 Testing exact comparison between working and failing patterns...");
    
    // Test 1: Use the exact same configuration as run_websocket_roundtrip_test("basic")
    let config = WebSocketTestConfig {
        path: format!("/{}", "basic"),
        handler_name: format!("{}-handler", "basic"),
        ..Default::default()
    };

    println!("📋 Config: port={}, path={}, handler_name={}", 
        config.port, config.path, config.handler_name);

    // Setup server - exact same as run_websocket_roundtrip_test
    let server_handle = match setup_test_websocket_server(config.clone()).await {
        Ok(handle) => {
            println!("✅ WebSocket server started successfully on path: {}", config.path);
            handle
        }
        Err(e) => {
            println!("❌ Failed to start WebSocket server: {}", e);
            panic!("Server startup failed: {}", e);
        }
    };

    // Wait for server to be fully ready (same as debug test that worked)
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("⏰ Waited 2 seconds for server to fully initialize");

    // Create client - exact same as run_websocket_roundtrip_test
    let client = match create_test_websocket_client(&config).await {
        Ok(client) => {
            println!("✅ WebSocket client created successfully");
            client
        }
        Err(e) => {
            println!("❌ Failed to create WebSocket client: {}", e);
            server_handle.abort();
            panic!("Client creation failed: {}", e);
        }
    };

    // Create test envelope - exact same as run_websocket_roundtrip_test
    let request_envelope = create_test_websocket_envelope(&format!("test {} roundtrip", "basic"), "basic");
    println!("📧 Created envelope with data: {:?}", request_envelope.payload.get("message"));

    // Execute WebSocket request - exact same timeout and pattern as run_websocket_roundtrip_test
    println!("🔗 Attempting to send envelope using timeout wrapper...");
    let timeout_result = tokio::time::timeout(
        Duration::from_secs(10),
        client.send_envelope(request_envelope.clone()),
    ).await;

    let response_result: Result<Envelope<Value>> = match timeout_result {
        Ok(result) => result,
        Err(_) => {
            server_handle.abort();
            panic!("Request timed out");
        }
    };

    match response_result {
        Ok(response_envelope) => {
            println!("✅ Response received successfully!");
            println!("📥 Response data: {:?}", response_envelope.payload);
            
            // Check specific verification details instead of using the function
            println!("🔍 Verifying response manually...");
            
            // Verify basic response structure
            assert_eq!(response_envelope.payload["status"], "success");
            assert_eq!(response_envelope.payload["handler"], config.handler_name);
            assert_eq!(response_envelope.payload["message_type"], "websocket_response");
            
            // Verify echo data
            let echo_data = &response_envelope.payload["echo"];
            assert_eq!(echo_data["test_type"], "basic");
            assert_eq!(echo_data["message"], request_envelope.payload["message"]);
            assert_eq!(echo_data["transport"], "websocket");
            
            // Verify context was passed
            assert_eq!(response_envelope.payload["context"]["has_context"], true);
            assert!(response_envelope.payload["processed_at"].is_string());
            
            println!("✅ Manual verification passed!");
            
            // Now try the verify function to see which exact assertion fails
            match std::panic::catch_unwind(|| {
                verify_websocket_roundtrip_response(
                    &request_envelope,
                    &response_envelope,
                    "basic",
                    &config.handler_name,
                );
            }) {
                Ok(_) => {
                    println!("✅ verify_websocket_roundtrip_response passed too!");
                }
                Err(_) => {
                    println!("❌ verify_websocket_roundtrip_response failed");
                    println!("🔍 Request metadata: {:?}", request_envelope.meta);
                    println!("🔍 Response metadata: {:?}", response_envelope.meta);
                }
            }
        }
        Err(e) => {
            println!("❌ Send envelope failed: {}", e);
            println!("🔍 Error details: {:#?}", e);
            
            // This will help us see if it's the same "No handler found" error
            if e.to_string().contains("No handler found") {
                println!("🚨 CONFIRMED: This is the path routing issue!");
                println!("🎯 The server is not finding the handler for path: {}", config.path);
                println!("❓ Handler name should be: {}", config.handler_name);
            }
        }
    }

    // Cleanup
    server_handle.abort();
    println!("🧹 Server shut down");
}

#[tokio::test] 
async fn test_direct_run_websocket_roundtrip_test() {
    println!("🔍 Testing direct call to run_websocket_roundtrip_test...");
    
    // Call the exact function that's failing
    let result = run_websocket_roundtrip_test("basic").await;
    
    match result {
        Ok(_) => {
            println!("✅ run_websocket_roundtrip_test succeeded!");
        }
        Err(e) => {
            println!("❌ run_websocket_roundtrip_test failed: {}", e);
            println!("🔍 Error details: {:#?}", e);
            
            if e.to_string().contains("No handler found") {
                println!("🚨 CONFIRMED: run_websocket_roundtrip_test has the routing issue!");
            }
        }
    }
}