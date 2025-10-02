// ABOUTME: Basic integration test for new REST server architecture
// ABOUTME: Verifies that the clean NATS-style REST server can be created and configured

use async_trait::async_trait;
use qollective::envelope::Context;
use qollective::error::Result;
use qollective::prelude::ContextDataHandler;
use qollective::prelude::UnifiedEnvelopeReceiver;
use qollective::server::common::ServerConfig;
use qollective::server::rest::{RestServer, RestServerConfig};
use serde::{Deserialize, Serialize};

mod common;
use common::{get_available_port, setup_test_environment};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestRequest {
    message: String,
    id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestResponse {
    result: String,
    status: u32,
}

struct TestHandler;

#[async_trait]
impl ContextDataHandler<TestRequest, TestResponse> for TestHandler {
    async fn handle(&self, _context: Option<Context>, data: TestRequest) -> Result<TestResponse> {
        Ok(TestResponse {
            result: format!("Processed: {}", data.message),
            status: 200,
        })
    }
}

#[tokio::test]
async fn test_new_rest_server_creation_and_configuration() {
    setup_test_environment();

    let port = get_available_port();

    // Test single construction pattern
    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        ..Default::default()
    };

    let server = RestServer::new(config)
        .await
        .expect("Failed to create REST server");

    // Verify configuration
    assert_eq!(server.config().base.port, port);
    assert_eq!(server.config().base.bind_address, "127.0.0.1");
    assert_eq!(server.route_count(), 0);

    println!("✅ REST server created successfully with single construction pattern");
}

#[tokio::test]
async fn test_new_rest_server_unified_envelope_receiver() {
    setup_test_environment();

    let port = get_available_port();

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut server = RestServer::new(config)
        .await
        .expect("Failed to create REST server");

    // Test UnifiedEnvelopeReceiver implementation
    let handler = TestHandler;

    // Test default route
    server
        .receive_envelope(handler)
        .await
        .expect("Failed to register default envelope handler");
    assert!(server.has_route("/envelope"));
    assert_eq!(server.route_count(), 1);

    // Test custom route
    let handler2 = TestHandler;
    server
        .receive_envelope_at("/api/v1/test", handler2)
        .await
        .expect("Failed to register custom envelope handler");
    assert!(server.has_route("/api/v1/test"));
    assert_eq!(server.route_count(), 2);

    // Test duplicate route detection
    let handler3 = TestHandler;
    let result = server.receive_envelope_at("/api/v1/test", handler3).await;
    assert!(result.is_err());
    assert_eq!(server.route_count(), 2); // Should remain unchanged

    // Verify route listing
    let routes = server.routes();
    assert!(routes.contains(&"/envelope".to_string()));
    assert!(routes.contains(&"/api/v1/test".to_string()));

    println!("✅ UnifiedEnvelopeReceiver implementation working correctly");
}

#[tokio::test]
async fn test_new_rest_server_configuration_validation() {
    setup_test_environment();

    // Test port validation
    let invalid_config1 = RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Invalid port
            ..Default::default()
        },
        ..Default::default()
    };

    let result1 = RestServer::new(invalid_config1).await;
    assert!(result1.is_err());

    // Test bind address validation
    let invalid_config2 = RestServerConfig {
        base: ServerConfig {
            bind_address: "".to_string(), // Invalid address
            port: 8080,
            ..Default::default()
        },
        ..Default::default()
    };

    let result2 = RestServer::new(invalid_config2).await;
    assert!(result2.is_err());

    // Test metadata configuration validation
    let mut invalid_config3 = RestServerConfig::default();
    invalid_config3.metadata.max_header_size = 0; // Invalid header size

    let result3 = RestServer::new(invalid_config3).await;
    assert!(result3.is_err());

    println!("✅ Configuration validation working correctly");
}

#[tokio::test]
async fn test_new_rest_server_nats_style_simplicity() {
    setup_test_environment();

    let port = get_available_port();

    // Test NATS-style simplicity: one line to create server
    let server = RestServer::new(RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .expect("Failed to create server with one-line configuration");

    // Verify clean API
    assert_eq!(server.route_count(), 0);
    assert_eq!(server.routes().len(), 0);
    assert!(!server.has_route("/nonexistent"));

    // Verify configuration access
    let config = server.config();
    assert_eq!(config.base.port, port);
    assert_eq!(config.metadata.max_header_size, 4_096); // Default value
    assert_eq!(config.metadata.max_total_headers, 65_536); // Default value

    println!("✅ NATS-style simplicity achieved");
}

#[tokio::test]
async fn test_new_rest_server_architecture_comparison() {
    setup_test_environment();

    // This test documents the architectural improvements:

    // OLD: Multiple ways to create REST server (5 different patterns)
    // NEW: Single construction pattern - RestServer::new(config)

    // OLD: 50+ builder methods, complex configuration
    // NEW: Simple config struct with sensible defaults

    // OLD: UnifiedEnvelopeReceiver bolted on as afterthought
    // NEW: Native UnifiedEnvelopeReceiver integration from day 1

    // OLD: 1,253 lines of fragmented code
    // NEW: ~200 lines of clean, focused code

    let port = get_available_port();
    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        ..Default::default()
    };

    // Single construction pattern (NATS-style)
    let mut server = RestServer::new(config)
        .await
        .expect("Failed to create server");

    // Native UnifiedEnvelopeReceiver (built-in, not bolted on)
    let handler = TestHandler;
    server
        .receive_envelope(handler)
        .await
        .expect("Failed to register handler");

    // Clean, simple API
    assert_eq!(server.route_count(), 1);
    assert!(server.has_route("/envelope"));

    println!("✅ New REST server architecture follows NATS principles");
    println!("   🎯 Single construction pattern");
    println!("   🎯 Native unified envelope support");
    println!("   🎯 Configuration-based design");
    println!("   🎯 Clean, focused architecture");
}
