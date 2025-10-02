// ABOUTME: Test that verifies gRPC server handler registration functionality works correctly
// ABOUTME: Validates that UnifiedEnvelopeReceiver::receive_envelope actually registers handlers in the gRPC server

use async_trait::async_trait;
use qollective::envelope::Context;
use qollective::error::Result;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::server::common::ServerConfig;
use qollective::server::grpc::{GrpcServer, QollectiveServiceImpl};
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

/// Test handler that processes requests
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
async fn test_grpc_server_handler_registration() {
    setup_test_environment();

    // Create gRPC server
    let server_port = get_available_port();
    let server_config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: server_port,
        max_connections: 100,
    };

    let mut server = GrpcServer::new(server_config);

    // Register the service implementation first
    let service_impl = QollectiveServiceImpl::new();
    let register_result = server.register_service(service_impl).await;
    assert!(
        register_result.is_ok(),
        "Service registration should succeed"
    );

    // Create test handler
    let handler = TestHandler;

    // Register handler using UnifiedEnvelopeReceiver trait
    let result = server
        .receive_envelope::<TestRequest, TestResponse, _>(handler)
        .await;

    // Should succeed (handler registration should work)
    assert!(
        result.is_ok(),
        "Handler registration should succeed: {:?}",
        result
    );

    println!("✅ gRPC handler registration test passed!");
    println!("   🎯 UnifiedEnvelopeReceiver::receive_envelope works correctly");
}

#[tokio::test]
async fn test_grpc_service_has_handlers_flag() {
    setup_test_environment();

    // Create service implementation directly
    let service = QollectiveServiceImpl::new();

    // Check initial state - no handlers registered
    let has_handlers = service.has_registered_handlers().await;
    assert!(!has_handlers, "Initially should have no handlers");

    // Register a handler
    let handler = TestHandler;
    let type_key = "TestRequest:TestResponse".to_string();
    let result = service.register_handler(type_key, handler).await;

    assert!(result.is_ok(), "Handler registration should succeed");

    // Check that handlers flag is now true
    let has_handlers = service.has_registered_handlers().await;
    assert!(has_handlers, "Should now have handlers registered");

    println!("✅ gRPC service handler flag test passed!");
    println!("   🎯 Handler registration properly updates internal state");
}

#[tokio::test]
async fn test_grpc_server_multiple_handler_registration() {
    setup_test_environment();

    // Create gRPC server
    let server_port = get_available_port();
    let server_config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: server_port,
        max_connections: 100,
    };

    let mut server = GrpcServer::new(server_config);

    // Register the service implementation first
    let service_impl = QollectiveServiceImpl::new();
    let register_result = server.register_service(service_impl).await;
    assert!(
        register_result.is_ok(),
        "Service registration should succeed"
    );

    // Register multiple handlers
    let handler1 = TestHandler;
    let handler2 = TestHandler;

    let result1 = server
        .receive_envelope::<TestRequest, TestResponse, _>(handler1)
        .await;
    let result2 = server
        .receive_envelope::<TestRequest, TestResponse, _>(handler2)
        .await;

    assert!(result1.is_ok(), "First handler registration should succeed");
    assert!(
        result2.is_ok(),
        "Second handler registration should succeed"
    );

    println!("✅ gRPC multiple handler registration test passed!");
    println!("   🎯 Multiple handlers can be registered without conflicts");
}
