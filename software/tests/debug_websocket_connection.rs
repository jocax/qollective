// ABOUTME: Debug test to isolate WebSocket connection issues step by step
// ABOUTME: Tests each stage of WebSocket communication to identify exact failure point

use std::time::Duration;

mod common;
use common::websocket_test_utils::*;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use serde_json::Value;

#[tokio::test]
async fn test_websocket_server_startup_only() {
    println!("🔧 Testing WebSocket server startup only...");
    
    let config = WebSocketTestConfig {
        path: "/debug".to_string(),
        handler_name: "debug-handler".to_string(),
        ..Default::default()
    };

    println!("📋 Config: port={}, path={}, with_tls={}", config.port, config.path, config.with_tls);

    // Setup server
    let server_handle = match setup_test_websocket_server(config.clone()).await {
        Ok(handle) => {
            println!("✅ WebSocket server started successfully");
            handle
        }
        Err(e) => {
            println!("❌ Failed to start WebSocket server: {}", e);
            panic!("Server startup failed: {}", e);
        }
    };

    // Give server more time to fully initialize
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("✅ Server has been running for 2 seconds");

    // Cleanup
    server_handle.abort();
    println!("🧹 Server shut down");
}

#[tokio::test]
async fn test_websocket_client_creation_only() {
    println!("🔧 Testing WebSocket client creation only...");
    
    let config = WebSocketTestConfig {
        path: "/debug-client".to_string(),
        handler_name: "debug-client-handler".to_string(),
        ..Default::default()
    };

    println!("📋 Config: port={}, path={}, with_tls={}", config.port, config.path, config.with_tls);

    // Create client without connecting to any server
    match create_test_websocket_client(&config).await {
        Ok(_client) => {
            println!("✅ WebSocket client created successfully");
        }
        Err(e) => {
            println!("❌ Failed to create WebSocket client: {}", e);
            panic!("Client creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_websocket_server_and_client_separate() {
    println!("🔧 Testing WebSocket server and client creation separately...");
    
    let config = WebSocketTestConfig {
        path: "/debug-separate".to_string(),
        handler_name: "debug-separate-handler".to_string(),
        ..Default::default()
    };

    println!("📋 Config: port={}, path={}, with_tls={}", config.port, config.path, config.with_tls);

    // Setup server first
    let server_handle = match setup_test_websocket_server(config.clone()).await {
        Ok(handle) => {
            println!("✅ WebSocket server started successfully");
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
    let _client = match create_test_websocket_client(&config).await {
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

    println!("✅ Both server and client created successfully");
    
    // Cleanup
    server_handle.abort();
    println!("🧹 Server shut down");
}

#[tokio::test]
async fn test_websocket_envelope_creation() {
    println!("🔧 Testing WebSocket envelope creation...");
    
    let envelope = create_test_websocket_envelope("test message", "debug");
    
    println!("📋 Envelope created:");
    println!("  Request ID: {:?}", envelope.meta.request_id);
    println!("  Tenant: {:?}", envelope.meta.tenant);
    println!("  Version: {:?}", envelope.meta.version);
    println!("  Data message: {:?}", envelope.payload.get("message"));
    println!("  Data test_type: {:?}", envelope.payload.get("test_type"));

    assert!(envelope.meta.request_id.is_some());
    assert_eq!(envelope.meta.tenant, Some("debug-tenant".to_string()));
    assert_eq!(envelope.payload["message"], "test message");
    assert_eq!(envelope.payload["test_type"], "debug");
    
    println!("✅ Envelope creation test passed");
}

#[tokio::test]
async fn test_websocket_minimal_connection() {
    println!("🔧 Testing minimal WebSocket connection and immediate disconnect...");
    
    let config = WebSocketTestConfig {
        path: "/debug-minimal".to_string(),
        handler_name: "debug-minimal-handler".to_string(),
        ..Default::default()
    };

    println!("📋 Config: port={}, path={}, with_tls={}", config.port, config.path, config.with_tls);

    // Setup server
    let server_handle = match setup_test_websocket_server(config.clone()).await {
        Ok(handle) => {
            println!("✅ WebSocket server started successfully");
            handle
        }
        Err(e) => {
            println!("❌ Failed to start WebSocket server: {}", e);
            panic!("Server startup failed: {}", e);
        }
    };

    // Wait for server to be fully ready
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("⏰ Waited 3 seconds for server to fully initialize");

    // Try to create client (this should trigger connection)
    println!("🔗 Attempting to create client and establish connection...");
    let client_result = create_test_websocket_client(&config).await;
    
    match client_result {
        Ok(_client) => {
            println!("✅ WebSocket client connected successfully");
            println!("🔗 Connection established and ready for communication");
        }
        Err(e) => {
            println!("❌ Failed to establish WebSocket connection: {}", e);
            println!("🔍 Error details: {:#?}", e);
            server_handle.abort();
            panic!("Connection establishment failed: {}", e);
        }
    }

    // Cleanup
    server_handle.abort();
    println!("🧹 Server shut down");
}