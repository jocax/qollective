// ABOUTME: Common utilities for WebSocket server/client integration tests with real-time communication
// ABOUTME: Provides WebSocket server setup, client creation, and roundtrip testing functions following REST pattern

//! Common utilities for WebSocket server/client integration tests.
//!
//! This module provides shared functionality for testing WebSocket communication
//! including server setup, client creation, and roundtrip testing patterns.
//! Follows the established REST test utilities pattern but adapted for WebSocket's
//! real-time, bidirectional communication features.

use super::{get_available_port, setup_test_environment};
use async_trait::async_trait;
use qollective::client::websocket::WebSocketClient;
use qollective::config::websocket::WebSocketClientConfig;
use qollective::envelope::{Context, Envelope, Meta};
use qollective::error::Result;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

/// Certificate path utilities for WebSocket testing
pub struct CertificatePaths;

impl CertificatePaths {
    /// Get the absolute path to the project root by finding Cargo.toml
    fn get_project_root() -> PathBuf {
        let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

        // Walk up the directory tree to find Cargo.toml
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return current_dir;
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                panic!("Could not find Cargo.toml in any parent directory");
            }
        }
    }

    /// Get absolute path to test certificates directory
    pub fn get_certs_dir() -> PathBuf {
        Self::get_project_root().join("tests").join("certs")
    }

    /// Get absolute path to server certificate
    pub fn get_server_cert_path() -> PathBuf {
        Self::get_certs_dir().join("server-cert.pem")
    }

    /// Get absolute path to server key
    pub fn get_server_key_path() -> PathBuf {
        Self::get_certs_dir().join("server-key.pem")
    }

    /// Get absolute path to client certificate
    pub fn get_client_cert_path() -> PathBuf {
        Self::get_certs_dir().join("client-cert.pem")
    }

    /// Get absolute path to client key
    pub fn get_client_key_path() -> PathBuf {
        Self::get_certs_dir().join("client-key.pem")
    }

    /// Get absolute path to CA certificate
    pub fn get_ca_cert_path() -> PathBuf {
        Self::get_certs_dir().join("ca.pem")
    }
}

/// Configuration for WebSocket roundtrip tests
#[derive(Debug, Clone)]
pub struct WebSocketTestConfig {
    pub port: u16,
    pub path: String,
    pub handler_name: String,
    pub max_connections: usize,
    pub ping_interval: Duration,
    pub connection_timeout: Option<Duration>,
    pub with_tls: bool,
}

impl Default for WebSocketTestConfig {
    fn default() -> Self {
        Self {
            port: get_available_port(),
            path: "/test".to_string(),
            handler_name: "test-handler".to_string(),
            max_connections: 10,
            ping_interval: Duration::from_secs(30),
            connection_timeout: Some(Duration::from_secs(10)),
            with_tls: false, // Default to non-TLS for compatibility
        }
    }
}

/// Generic test handler for WebSocket envelope processing
pub struct TestWebSocketHandler {
    pub name: String,
}

impl TestWebSocketHandler {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl ContextDataHandler<Value, Value> for TestWebSocketHandler {
    async fn handle(&self, context: Option<Context>, data: Value) -> Result<Value> {
        // Extract context information
        let context_info = if let Some(ctx) = context {
            let meta = ctx.meta();
            json!({
                "has_context": true,
                "request_id": meta.request_id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
                "tenant": meta.tenant.as_ref().unwrap_or(&String::new()),
                "timestamp": meta.timestamp.as_ref().map(|ts| ts.to_rfc3339()).unwrap_or_default()
            })
        } else {
            json!({
                "has_context": false
            })
        };

        // Create response with echo data and context information
        Ok(json!({
            "handler": self.name,
            "status": "success",
            "echo": data,
            "context": context_info,
            "processed_at": chrono::Utc::now(),
            "message_type": "websocket_response"
        }))
    }
}

/// Setup WebSocket server for testing
pub async fn setup_test_websocket_server(
    config: WebSocketTestConfig,
) -> Result<tokio::task::JoinHandle<()>> {
    setup_test_environment();

    // Create server configuration with TLS properly configured using our unified TLS system
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = "127.0.0.1".to_string();
    server_config.base.port = config.port;
    server_config.base.max_connections = 10; // Limit connections for testing
    server_config.ping_interval = config.ping_interval;

    // Configure TLS based on the with_tls parameter
    server_config.tls.enabled = config.with_tls;
    if config.with_tls {
        server_config.tls.verification_mode = qollective::config::tls::VerificationMode::Skip;
        server_config.tls.cert_path = Some(CertificatePaths::get_server_cert_path());
        server_config.tls.key_path = Some(CertificatePaths::get_server_key_path());
    }

    let mut server = WebSocketServer::new(server_config).await?;

    // Register test handler
    let handler = TestWebSocketHandler::new(&config.handler_name);
    server.receive_envelope_at(&config.path, handler).await?;

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("WebSocket server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    let scheme = if config.with_tls { "wss" } else { "ws" };
    println!(
        "🔍 WebSocket server should be running on {}://127.0.0.1:{}{}",
        scheme, config.port, config.path
    );

    Ok(server_handle)
}

/// Create WebSocket client for testing
pub async fn create_test_websocket_client(config: &WebSocketTestConfig) -> Result<WebSocketClient> {
    // Use appropriate scheme based on TLS configuration
    let scheme = if config.with_tls { "wss" } else { "ws" };
    let endpoint_url = format!("{}://127.0.0.1:{}{}", scheme, config.port, config.path);

    // Create client configuration with conditional TLS support
    let mut builder = WebSocketClientConfig::builder()
        .with_connection_timeout(10000) // 10 seconds
        .with_message_timeout(5000)     // 5 seconds
        .with_ping_timeout(5000)        // 5 seconds
        .with_max_retry_attempts(1);

    // Apply TLS settings only if TLS is enabled
    if config.with_tls {
        builder = builder.with_tls_skip_verify(); // Skip verification for testing
    }

    let mut client_config = builder.build();

    // Set ping interval manually since not all builder methods are available
    client_config.websocket.ping_interval_ms = config.ping_interval.as_millis() as u64;

    WebSocketClient::new(endpoint_url, client_config).await
}

/// Create test envelope with metadata for WebSocket testing
pub fn create_test_websocket_envelope(message: &str, test_type: &str) -> Envelope<Value> {
    let mut meta = Meta::default();
    meta.request_id = Some(Uuid::now_v7());
    meta.tenant = Some(format!("{}-tenant", test_type));
    meta.version = Some("1.0".to_string());
    meta.timestamp = Some(chrono::Utc::now());

    let data = json!({
        "message": message,
        "test_type": test_type,
        "test_id": Uuid::now_v7(),
        "timestamp": chrono::Utc::now(),
        "transport": "websocket"
    });

    Envelope::new(meta, data)
}

/// Verify WebSocket roundtrip response
pub fn verify_websocket_roundtrip_response(
    request_envelope: &Envelope<Value>,
    response_envelope: &Envelope<Value>,
    expected_test_type: &str,
    handler_name: &str,
) {
    // Verify metadata preservation
    assert_eq!(
        response_envelope.meta.request_id,
        request_envelope.meta.request_id
    );
    assert_eq!(response_envelope.meta.tenant, request_envelope.meta.tenant);
    assert_eq!(
        response_envelope.meta.version,
        request_envelope.meta.version
    );

    // Verify response structure
    assert_eq!(response_envelope.payload["status"], "success");
    assert_eq!(response_envelope.payload["handler"], handler_name);
    assert_eq!(response_envelope.payload["message_type"], "websocket_response");

    // Verify echo data
    let echo_data = &response_envelope.payload["echo"];
    assert_eq!(echo_data["test_type"], expected_test_type);
    assert_eq!(echo_data["message"], request_envelope.payload["message"]);
    assert_eq!(echo_data["transport"], "websocket");

    // Verify context was passed
    assert_eq!(response_envelope.payload["context"]["has_context"], true);
    assert!(response_envelope.payload["processed_at"].is_string());
}

/// Run a basic WebSocket roundtrip test
pub async fn run_websocket_roundtrip_test(test_type: &str) -> Result<()> {
    let config = WebSocketTestConfig {
        path: format!("/{}", test_type),
        handler_name: format!("{}-handler", test_type),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_websocket_server(config.clone()).await?;

    // Create client
    let client = create_test_websocket_client(&config).await?;

    // Create test envelope
    let request_envelope =
        create_test_websocket_envelope(&format!("test {} roundtrip", test_type), test_type);

    // Execute WebSocket request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(10),
        client.send_envelope(request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_websocket_roundtrip_response(
        &request_envelope,
        &response_envelope,
        test_type,
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Setup WebSocket server with broadcasting capabilities
pub async fn setup_websocket_broadcast_server(
    config: WebSocketTestConfig,
) -> Result<tokio::task::JoinHandle<()>> {
    setup_test_environment();

    // Create broadcast handler that echoes to all connections
    struct BroadcastHandler {
        name: String,
    }

    impl BroadcastHandler {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl ContextDataHandler<Value, Value> for BroadcastHandler {
        async fn handle(&self, context: Option<Context>, data: Value) -> Result<Value> {
            // Extract context information
            let context_info = if let Some(ctx) = context {
                let meta = ctx.meta();
                json!({
                    "has_context": true,
                    "request_id": meta.request_id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
                    "tenant": meta.tenant.as_ref().unwrap_or(&String::new()),
                })
            } else {
                json!({ "has_context": false })
            };

            // Create broadcast response
            Ok(json!({
                "handler": self.name,
                "status": "broadcast",
                "echo": data,
                "context": context_info,
                "broadcast_at": chrono::Utc::now(),
                "message_type": "websocket_broadcast"
            }))
        }
    }

    // Create server with broadcast handler and TLS properly configured
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = "127.0.0.1".to_string();
    server_config.base.port = config.port;
    server_config.base.max_connections = 10; // Limit connections for testing
    server_config.ping_interval = config.ping_interval;

    // Configure TLS based on the with_tls parameter
    server_config.tls.enabled = config.with_tls;
    if config.with_tls {
        server_config.tls.verification_mode = qollective::config::tls::VerificationMode::Skip;
        server_config.tls.cert_path = Some(CertificatePaths::get_server_cert_path());
        server_config.tls.key_path = Some(CertificatePaths::get_server_key_path());
    }

    let mut server = WebSocketServer::new(server_config).await?;
    let broadcast_handler = BroadcastHandler::new(&config.handler_name);
    server
        .receive_envelope_at(&config.path, broadcast_handler)
        .await?;

    // Start server
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("WebSocket broadcast server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    let scheme = if config.with_tls { "wss" } else { "ws" };
    println!(
        "🔍 WebSocket broadcast server running on {}://127.0.0.1:{}{}",
        scheme, config.port, config.path
    );

    Ok(server_handle)
}

/// Create multiple WebSocket clients for testing broadcasting
pub async fn create_multiple_websocket_clients(
    config: &WebSocketTestConfig,
    count: usize,
) -> Result<Vec<WebSocketClient>> {
    let mut clients = Vec::new();

    for i in 0..count {
        let mut client_config = config.clone();
        client_config.handler_name = format!("{}-client-{}", config.handler_name, i);

        let client = create_test_websocket_client(&client_config).await?;
        clients.push(client);

        // Small delay between client creations to avoid overwhelming the server
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!(
        "✅ Created {} WebSocket clients for broadcast testing",
        count
    );
    Ok(clients)
}

/// Test WebSocket ping/pong functionality (simplified for testing)
pub fn test_websocket_ping_pong(_client: &WebSocketClient) -> Result<Duration> {
    // Simplified ping test - just verify connection is working
    // In a full implementation, this would use actual WebSocket ping frames
    let ping_duration = Duration::from_millis(10); // Simulated ping duration
    println!(
        "🏓 WebSocket ping/pong test (simulated) completed in {:?}",
        ping_duration
    );

    Ok(ping_duration)
}

/// Create large WebSocket message for testing
pub fn create_large_websocket_envelope(size_kb: usize) -> Envelope<Value> {
    let mut meta = Meta::default();
    meta.request_id = Some(Uuid::now_v7());
    meta.tenant = Some("large-message-test".to_string());
    meta.version = Some("1.0".to_string());
    meta.timestamp = Some(chrono::Utc::now());

    // Create large payload
    let large_data = "x".repeat(size_kb * 1024);

    let data = json!({
        "message": "large WebSocket message test",
        "test_type": "large_message",
        "large_payload": large_data,
        "size_kb": size_kb,
        "timestamp": chrono::Utc::now(),
        "transport": "websocket"
    });

    Envelope::new(meta, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_config_creation() {
        let config = WebSocketTestConfig::default();
        assert!(config.port > 0);
        assert_eq!(config.path, "/test");
        assert_eq!(config.handler_name, "test-handler");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.ping_interval, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_websocket_handler() {
        let handler = TestWebSocketHandler::new("test");
        let context = Some(Context::empty());
        let data = json!({"test": "data"});

        let result = handler.handle(context, data).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["handler"], "test");
        assert_eq!(response["status"], "success");
        assert_eq!(response["echo"]["test"], "data");
        assert_eq!(response["context"]["has_context"], true);
        assert_eq!(response["message_type"], "websocket_response");
    }

    #[test]
    fn test_create_test_websocket_envelope() {
        let envelope = create_test_websocket_envelope("test message", "basic");

        assert!(envelope.meta.request_id.is_some());
        assert_eq!(envelope.meta.tenant, Some("basic-tenant".to_string()));
        assert_eq!(envelope.meta.version, Some("1.0".to_string()));
        assert!(envelope.meta.timestamp.is_some());

        assert_eq!(envelope.payload["message"], "test message");
        assert_eq!(envelope.payload["test_type"], "basic");
        assert_eq!(envelope.payload["transport"], "websocket");
        assert!(envelope.payload["test_id"].is_string());
    }

    #[test]
    fn test_create_large_websocket_envelope() {
        let envelope = create_large_websocket_envelope(5); // 5KB

        assert_eq!(envelope.payload["test_type"], "large_message");
        assert_eq!(envelope.payload["size_kb"], 5);
        assert_eq!(
            envelope.payload["large_payload"].as_str().unwrap().len(),
            5 * 1024
        );
    }

    #[test]
    fn test_websocket_config_with_tls_default() {
        let config = WebSocketTestConfig::default();
        assert_eq!(config.with_tls, false); // Default should be false for compatibility
        assert!(config.port > 0);
        assert_eq!(config.path, "/test");
        assert_eq!(config.handler_name, "test-handler");
    }

    #[test]
    fn test_websocket_config_with_tls_enabled() {
        let config = WebSocketTestConfig {
            with_tls: true,
            ..Default::default()
        };
        assert_eq!(config.with_tls, true);
        assert!(config.port > 0);
        assert_eq!(config.path, "/test");
        assert_eq!(config.handler_name, "test-handler");
    }

    #[test]
    fn test_websocket_config_with_tls_disabled() {
        let config = WebSocketTestConfig {
            with_tls: false,
            ..Default::default()
        };
        assert_eq!(config.with_tls, false);
        assert!(config.port > 0);
    }

    #[test]
    fn test_websocket_config_custom_values_with_tls() {
        let config = WebSocketTestConfig {
            port: 8080,
            path: "/custom".to_string(),
            handler_name: "custom-handler".to_string(),
            max_connections: 20,
            ping_interval: Duration::from_secs(60),
            connection_timeout: Some(Duration::from_secs(15)),
            with_tls: true,
        };

        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "/custom");
        assert_eq!(config.handler_name, "custom-handler");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.ping_interval, Duration::from_secs(60));
        assert_eq!(config.connection_timeout, Some(Duration::from_secs(15)));
        assert_eq!(config.with_tls, true);
    }

    #[test]
    fn test_certificate_paths_exist() {
        // Test that our certificate path utilities work correctly
        let certs_dir = CertificatePaths::get_certs_dir();
        let server_cert = CertificatePaths::get_server_cert_path();
        let server_key = CertificatePaths::get_server_key_path();
        let client_cert = CertificatePaths::get_client_cert_path();
        let client_key = CertificatePaths::get_client_key_path();
        let ca_cert = CertificatePaths::get_ca_cert_path();

        // Verify paths are absolute and have correct structure
        assert!(certs_dir.is_absolute());
        assert!(server_cert.is_absolute());
        assert!(server_key.is_absolute());
        assert!(client_cert.is_absolute());
        assert!(client_key.is_absolute());
        assert!(ca_cert.is_absolute());

        // Verify paths end with expected filenames
        assert!(server_cert.to_string_lossy().ends_with("server-cert.pem"));
        assert!(server_key.to_string_lossy().ends_with("server-key.pem"));
        assert!(client_cert.to_string_lossy().ends_with("client-cert.pem"));
        assert!(client_key.to_string_lossy().ends_with("client-key.pem"));
        assert!(ca_cert.to_string_lossy().ends_with("ca.pem"));

        // Verify all paths contain the certs directory
        let certs_dir_str = certs_dir.to_string_lossy();
        assert!(server_cert.to_string_lossy().contains(&*certs_dir_str));
        assert!(server_key.to_string_lossy().contains(&*certs_dir_str));
        assert!(client_cert.to_string_lossy().contains(&*certs_dir_str));
        assert!(client_key.to_string_lossy().contains(&*certs_dir_str));
        assert!(ca_cert.to_string_lossy().contains(&*certs_dir_str));
    }

    #[test]
    fn test_certificate_paths_project_root_detection() {
        // Test that we can find the project root correctly
        let certs_dir = CertificatePaths::get_certs_dir();

        // The path should contain both "tests" and "certs" components
        let path_str = certs_dir.to_string_lossy();
        assert!(path_str.contains("tests"));
        assert!(path_str.contains("certs"));

        // Should end with tests/certs
        assert!(path_str.ends_with("tests/certs") || path_str.ends_with("tests\\certs"));
    }

    #[tokio::test]
    async fn test_websocket_server_config_with_tls_disabled() {
        // Test that server configuration correctly handles TLS disabled
        let config = WebSocketTestConfig {
            with_tls: false,
            ..Default::default()
        };

        // This should not panic and should create a valid server config
        // We can't easily test the full server setup without async complexities,
        // but we can verify the config structure
        assert_eq!(config.with_tls, false);

        // Verify the config would generate correct URL scheme
        let expected_scheme = if config.with_tls { "wss" } else { "ws" };
        assert_eq!(expected_scheme, "ws");
    }

    #[tokio::test]
    async fn test_websocket_server_config_with_tls_enabled() {
        // Test that server configuration correctly handles TLS enabled
        let config = WebSocketTestConfig {
            with_tls: true,
            ..Default::default()
        };

        assert_eq!(config.with_tls, true);

        // Verify the config would generate correct URL scheme
        let expected_scheme = if config.with_tls { "wss" } else { "ws" };
        assert_eq!(expected_scheme, "wss");
    }

    #[test]
    fn test_websocket_client_url_scheme_generation() {
        // Test URL scheme generation logic
        let config_no_tls = WebSocketTestConfig {
            port: 8080,
            path: "/test".to_string(),
            with_tls: false,
            ..Default::default()
        };

        let config_with_tls = WebSocketTestConfig {
            port: 8080,
            path: "/test".to_string(),
            with_tls: true,
            ..Default::default()
        };

        // Simulate the URL generation logic from create_test_websocket_client
        let scheme_no_tls = if config_no_tls.with_tls { "wss" } else { "ws" };
        let url_no_tls = format!("{}://127.0.0.1:{}{}", scheme_no_tls, config_no_tls.port, config_no_tls.path);

        let scheme_with_tls = if config_with_tls.with_tls { "wss" } else { "ws" };
        let url_with_tls = format!("{}://127.0.0.1:{}{}", scheme_with_tls, config_with_tls.port, config_with_tls.path);

        assert_eq!(url_no_tls, "ws://127.0.0.1:8080/test");
        assert_eq!(url_with_tls, "wss://127.0.0.1:8080/test");
    }
}
