// ABOUTME: Debug test to check what path is being sent/received in WebSocket connections
// ABOUTME: Isolates the path handling issue in WebSocket client-server communication

use std::time::Duration;

mod common;
use common::websocket_test_utils::*;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use serde_json::Value;

#[tokio::test]
async fn test_websocket_path_debugging() {
    println!("🔧 Testing WebSocket path handling...");
    
    let config = WebSocketTestConfig {
        path: "/debug-path".to_string(),
        handler_name: "debug-path-handler".to_string(),
        ..Default::default()
    };

    println!("📋 Config: port={}, path={}, handler_name={}", 
        config.port, config.path, config.handler_name);

    // Setup server
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

    // Wait for server to be fully ready
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("⏰ Waited 2 seconds for server to fully initialize");

    // Create client
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

    // Create simple envelope
    let envelope = create_test_websocket_envelope("path debugging test", "debug_path");
    println!("📧 Created envelope with data: {:?}", envelope.payload.get("message"));

    // Try to send envelope
    println!("🔗 Attempting to send envelope...");
    let response_result: Result<Envelope<Value>> = client.send_envelope(envelope.clone()).await;
    match response_result {
        Ok(response) => {
            println!("✅ Response received successfully!");
            println!("📥 Response data: {:?}", response.payload);
        }
        Err(e) => {
            println!("❌ Send envelope failed: {}", e);
            println!("🔍 Error details: {:#?}", e);
            
            // This will help us see if it's the same "No handler found" error
            if e.to_string().contains("No handler found") {
                println!("🚨 CONFIRMED: This is the path routing issue!");
                println!("🎯 The server is not finding the handler for path: {}", config.path);
            }
        }
    }

    // Cleanup
    server_handle.abort();
    println!("🧹 Server shut down");
}