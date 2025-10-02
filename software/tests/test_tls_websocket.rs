// ABOUTME: Comprehensive TLS tests for WebSocket transport protocol
// ABOUTME: Tests TLS builder methods, verification modes, and secure WebSocket communication

use qollective::config::tls::{TlsConfig, VerificationMode};
use qollective::config::websocket::{WebSocketClientConfig, WebSocketServerConfig};
use qollective::constants::{limits, network, timeouts};
use std::path::PathBuf;

mod common;
use common::{get_available_port, setup_test_environment};

/// Test WebSocket client TLS configuration creation with builder methods
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_client_tls_builder_methods() {
    setup_test_environment();

    // Test system CA verification
    let config = WebSocketClientConfig::builder()
        .with_tls_system_ca()
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
    assert!(config.tls.cert_path.is_none());
    assert!(config.tls.key_path.is_none());
    assert!(config.tls.ca_cert_path.is_none());

    // Test custom CA verification
    let config = WebSocketClientConfig::builder()
        .with_tls_custom_ca("./tests/certs/ca-cert.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca-cert.pem"))
    );

    // Test skip verification
    let config = WebSocketClientConfig::builder()
        .with_tls_skip_verify()
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test mutual TLS
    let config = WebSocketClientConfig::builder()
        .with_mutual_tls("./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));

    // Test mutual TLS with custom CA
    let config = WebSocketClientConfig::builder()
        .with_mutual_tls_with_ca("./tests/certs/ca-cert.pem", "./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca-cert.pem"))
    );
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
}

/// Test WebSocket server TLS configuration creation with builder methods
#[cfg(feature = "websocket-server")]
#[tokio::test]
async fn test_websocket_server_tls_builder_methods() {
    setup_test_environment();

    // Test system CA verification
    let config = WebSocketServerConfig::builder()
        .with_tls_system_ca()
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);

    // Test custom CA verification
    let config = WebSocketServerConfig::builder()
        .with_tls_custom_ca("./tests/certs/ca-cert.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca-cert.pem"))
    );

    // Test skip verification
    let config = WebSocketServerConfig::builder()
        .with_tls_skip_verify()
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test mutual TLS
    let config = WebSocketServerConfig::builder()
        .with_mutual_tls("./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));

    // Test mutual TLS with custom CA
    let config = WebSocketServerConfig::builder()
        .with_mutual_tls_with_ca("./tests/certs/ca-cert.pem", "./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca-cert.pem"))
    );
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
}

/// Test WebSocket client TLS configuration validation
#[cfg(feature = "websocket-client" )]
#[tokio::test]
async fn test_websocket_client_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = WebSocketClientConfig::default();
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = WebSocketClientConfig::builder()
        .with_tls_system_ca()
        .with_mutual_tls("./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = WebSocketClientConfig::builder()
        .with_tls_skip_verify()
        .with_mutual_tls("./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();
    assert!(config.tls.validate().is_ok());
}

/// Test WebSocket server TLS configuration validation
#[cfg(feature = "websocket-server")]
#[tokio::test]
async fn test_websocket_server_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = WebSocketServerConfig::default();
    
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = WebSocketServerConfig::builder()
        .with_tls_system_ca()
        .with_mutual_tls("./tests/certs/server-cert.pem", "./tests/certs/server-key.pem")
        .build();
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = WebSocketServerConfig::builder()
        .with_tls_skip_verify()
        .with_mutual_tls("./tests/certs/server-cert.pem", "./tests/certs/server-key.pem")
        .build();
    assert!(config.tls.validate().is_ok());
}

/// Test WebSocket client TLS configuration with constants
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_client_tls_with_constants() {
    setup_test_environment();

    let config = WebSocketClientConfig {
        connection_timeout_ms: timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS,
        message_timeout_ms: timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS,
        ping_timeout_ms: timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS,
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert_eq!(
        config.connection_timeout_ms,
        timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS
    );
    assert_eq!(
        config.message_timeout_ms,
        timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS
    );
    assert_eq!(
        config.ping_timeout_ms,
        timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS
    );
    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
}

/// Test WebSocket server TLS configuration with constants
#[cfg(feature = "websocket-server")]
#[tokio::test]
async fn test_websocket_server_tls_with_constants() {
    setup_test_environment();

    let config = WebSocketServerConfig {
        bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
        port: network::DEFAULT_REST_SERVER_PORT,
        max_connections: limits::DEFAULT_MAX_WEBSOCKET_CONNECTIONS,
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert_eq!(config.bind_address, network::DEFAULT_BIND_LOCALHOST);
    assert_eq!(config.port, network::DEFAULT_REST_SERVER_PORT);
    assert_eq!(
        config.max_connections,
        limits::DEFAULT_MAX_WEBSOCKET_CONNECTIONS
    );
    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
}

/// Test WebSocket TLS configuration serialization and deserialization
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_serialization() {
    setup_test_environment();

    let original_config = WebSocketClientConfig::builder()
        .with_mutual_tls_with_ca("./tests/certs/ca-cert.pem", "./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    // Serialize configuration
    let serialized = serde_json::to_string(&original_config)
        .expect("Failed to serialize WebSocket client config");

    // Deserialize configuration
    let deserialized_config: WebSocketClientConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize WebSocket client config");

    // Verify TLS configuration is preserved
    assert_eq!(original_config.tls.enabled, deserialized_config.tls.enabled);
    assert_eq!(
        original_config.tls.verification_mode,
        deserialized_config.tls.verification_mode
    );
    assert_eq!(
        original_config.tls.cert_path,
        deserialized_config.tls.cert_path
    );
    assert_eq!(
        original_config.tls.key_path,
        deserialized_config.tls.key_path
    );
    assert_eq!(
        original_config.tls.ca_cert_path,
        deserialized_config.tls.ca_cert_path
    );
}

/// Test WebSocket TLS configuration builder method chaining
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_builder_chaining() {
    setup_test_environment();

    // Test complex builder chain for client
    let client_config = WebSocketClientConfig::builder()
        .with_connection_timeout(5000)
        .with_message_timeout(3000)
        .with_ping_timeout(1000)
        .with_max_frame_size(1024 * 1024)
        .with_max_retry_attempts(5)
        .with_user_agent("test-websocket-client")
        .with_tls_system_ca()
        .build();

    assert_eq!(client_config.connection_timeout_ms, 5000);
    assert_eq!(client_config.message_timeout_ms, 3000);
    assert_eq!(client_config.ping_timeout_ms, 1000);
    assert_eq!(client_config.max_frame_size, 1024 * 1024);
    assert_eq!(client_config.max_retry_attempts, 5);
    assert_eq!(client_config.user_agent, "test-websocket-client");
    assert!(client_config.tls.enabled);
    assert_eq!(
        client_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
}

/// Test WebSocket server TLS configuration builder method chaining
#[cfg(feature = "websocket-server")]
#[tokio::test]
async fn test_websocket_server_tls_builder_chaining() {
    setup_test_environment();

    let port = get_available_port();

    // Test complex builder chain for server
    let server_config = WebSocketServerConfig::builder()
        .with_bind_address("0.0.0.0")
        .with_port(port)
        .with_max_connections(1000)
        .with_tls_skip_verify()
        .build();

    assert_eq!(server_config.bind_address, "0.0.0.0");
    assert_eq!(server_config.port, port);
    assert_eq!(server_config.max_connections, 1000);
    assert!(server_config.tls.enabled);
    assert_eq!(server_config.tls.verification_mode, VerificationMode::Skip);
}

/// Test WebSocket TLS configuration default values
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_defaults() {
    setup_test_environment();

    let client_config = WebSocketClientConfig::default();
    assert!(!client_config.tls.enabled);
    assert_eq!(
        client_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(client_config.tls.cert_path.is_none());
    assert!(client_config.tls.key_path.is_none());
    assert!(client_config.tls.ca_cert_path.is_none());
}

/// Test WebSocket server TLS configuration default values
#[cfg(feature = "websocket-server")]
#[tokio::test]
async fn test_websocket_server_tls_defaults() {
    setup_test_environment();

    let server_config = WebSocketServerConfig::default();
    assert!(!server_config.tls.enabled);
    assert_eq!(
        server_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(server_config.tls.cert_path.is_none());
    assert!(server_config.tls.key_path.is_none());
    assert!(server_config.tls.ca_cert_path.is_none());
}

/// Test WebSocket TLS configuration with different verification modes
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_verification_modes() {
    setup_test_environment();

    // Test SystemCa mode
    let config = WebSocketClientConfig::builder()
        .with_tls_system_ca()
        .build();
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);

    // Test CustomCa mode
    let config = WebSocketClientConfig::builder()
        .with_tls_custom_ca("./tests/certs/ca-cert.pem")
        .build();
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);

    // Test Skip mode
    let config = WebSocketClientConfig::builder()
        .with_tls_skip_verify()
        .build();
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test MutualTls mode
    let config = WebSocketClientConfig::builder()
        .with_mutual_tls("./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
}

/// Test WebSocket TLS configuration path handling
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_path_handling() {
    setup_test_environment();

    let cert_path = "/etc/ssl/certs/websocket-cert.pem";
    let key_path = "/etc/ssl/private/websocket-key.pem";
    let ca_path = "/etc/ssl/certs/ca.pem";

    let config = WebSocketClientConfig::builder()
        .with_mutual_tls_with_ca(ca_path, cert_path, key_path)
        .build();

    assert_eq!(config.tls.cert_path, Some(PathBuf::from(cert_path)));
    assert_eq!(config.tls.key_path, Some(PathBuf::from(key_path)));
    assert_eq!(config.tls.ca_cert_path, Some(PathBuf::from(ca_path)));
}

/// Test WebSocket TLS configuration with environment variables
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_with_env_vars() {
    setup_test_environment();

    // Set environment variables
    std::env::set_var("QOLLECTIVE_TLS_ENABLED", "true");
    std::env::set_var("QOLLECTIVE_TLS_CERT_PATH", "./tests/certs/client-cert.pem");
    std::env::set_var("QOLLECTIVE_TLS_KEY_PATH", "./tests/certs/client-key.pem");
    std::env::set_var("QOLLECTIVE_TLS_VERIFY_MODE", "skip");

    let tls_config = TlsConfig::from_env().expect("Failed to create TLS config from env");

    let websocket_config = WebSocketClientConfig {
        tls: tls_config,
        ..Default::default()
    };

    assert!(websocket_config.tls.enabled);
    assert_eq!(
        websocket_config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(
        websocket_config.tls.key_path,
        Some(PathBuf::from("./tests/certs/client-key.pem"))
    );
    assert_eq!(
        websocket_config.tls.verification_mode,
        VerificationMode::Skip
    );

    // Clean up
    std::env::remove_var("QOLLECTIVE_TLS_ENABLED");
    std::env::remove_var("QOLLECTIVE_TLS_CERT_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_KEY_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_VERIFY_MODE");
}

/// Test WebSocket TLS configuration with connection headers
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_with_connection_headers() {
    setup_test_environment();

    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let config = WebSocketClientConfig::builder()
        .with_tls_system_ca()
        .with_connection_headers(headers.clone())
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.connection_headers, headers);
    assert_eq!(
        config.connection_headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert_eq!(
        config.connection_headers.get("X-Custom-Header"),
        Some(&"custom-value".to_string())
    );
}

/// Test WebSocket TLS configuration with connection parameters
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_with_connection_params() {
    setup_test_environment();

    let config = WebSocketClientConfig::builder()
        .with_tls_system_ca()
        .with_connection_timeout(10000)
        .with_message_timeout(5000)
        .with_ping_timeout(2000)
        .with_max_frame_size(2 * 1024 * 1024)
        .with_max_retry_attempts(3)
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.connection_timeout_ms, 10000);
    assert_eq!(config.message_timeout_ms, 5000);
    assert_eq!(config.ping_timeout_ms, 2000);
    assert_eq!(config.max_frame_size, 2 * 1024 * 1024);
    assert_eq!(config.max_retry_attempts, 3);
}

/// Test WebSocket TLS configuration with WebSocket-specific settings
#[cfg(feature = "websocket-client")]
#[tokio::test]
async fn test_websocket_tls_with_websocket_settings() {
    setup_test_environment();

    let config = WebSocketClientConfig::builder()
        .with_tls_system_ca()
        .build();

    assert!(config.tls.enabled);

    // Test WebSocket-specific configuration access methods
    let ping_interval = config.ping_interval();
    let ping_timeout = config.ping_timeout();
    let max_message_size = config.max_message_size();
    let compression_enabled = config.compression_enabled();
    let subprotocols = config.subprotocols();

    assert!(ping_interval.as_millis() > 0);
    assert!(ping_timeout.as_millis() > 0);
    assert!(max_message_size > 0);
    assert!(compression_enabled); // Default is true
    assert!(!subprotocols.is_empty());
    assert_eq!(subprotocols[0], "qollective");
}
