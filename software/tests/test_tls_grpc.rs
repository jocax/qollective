// ABOUTME: Comprehensive TLS tests for gRPC transport protocol
// ABOUTME: Tests TLS builder methods, verification modes, and secure gRPC communication

use qollective::config::grpc::{GrpcClientConfig, GrpcServerConfig};
use qollective::config::tls::{TlsConfig, VerificationMode};
use qollective::constants::{network, timeouts};
use std::path::PathBuf;

mod common;
use common::{get_available_port, setup_test_environment};

/// Test gRPC client TLS configuration creation with builder methods
#[tokio::test]
async fn test_grpc_client_tls_builder_methods() {
    setup_test_environment();

    // Test system CA verification
    let config = GrpcClientConfig::builder().with_tls_system_ca().build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
    assert!(config.tls.cert_path.is_none());
    assert!(config.tls.key_path.is_none());
    assert!(config.tls.ca_cert_path.is_none());

    // Test custom CA verification
    let config = GrpcClientConfig::builder()
        .with_tls_custom_ca("./tests/certs/ca.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca.pem"))
    );

    // Test skip verification
    let config = GrpcClientConfig::builder().with_tls_skip_verify().build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test mutual TLS
    let config = GrpcClientConfig::builder()
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
    let config = GrpcClientConfig::builder()
        .with_mutual_tls_with_ca("./tests/certs/ca.pem", "./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca.pem"))
    );
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
}

/// Test gRPC server TLS configuration creation with builder methods
#[tokio::test]
async fn test_grpc_server_tls_builder_methods() {
    setup_test_environment();

    // Test system CA verification
    let config = GrpcServerConfig::builder().with_tls_system_ca().build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);

    // Test custom CA verification
    let config = GrpcServerConfig::builder()
        .with_tls_custom_ca("./tests/certs/ca.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca.pem"))
    );

    // Test skip verification
    let config = GrpcServerConfig::builder().with_tls_skip_verify().build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test mutual TLS
    let config = GrpcServerConfig::builder()
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
    let config = GrpcServerConfig::builder()
        .with_mutual_tls_with_ca("./tests/certs/ca.pem", "./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca.pem"))
    );
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
}

/// Test gRPC client TLS configuration validation
#[tokio::test]
async fn test_grpc_client_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = GrpcClientConfig::default();
    assert!(config.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = GrpcClientConfig::builder().with_tls_system_ca().build();
    assert!(config.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = GrpcClientConfig::builder().with_tls_skip_verify().build();
    assert!(config.validate().is_ok());
}

/// Test gRPC server TLS configuration validation
#[tokio::test]
async fn test_grpc_server_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = GrpcServerConfig::default();
    assert!(config.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = GrpcServerConfig::builder().with_tls_system_ca().build();
    assert!(config.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = GrpcServerConfig::builder().with_tls_skip_verify().build();
    assert!(config.validate().is_ok());
}

/// Test gRPC client TLS configuration with constants
#[tokio::test]
async fn test_grpc_client_tls_with_constants() {
    setup_test_environment();

    let config = GrpcClientConfig {
        timeout_ms: timeouts::DEFAULT_GRPC_TIMEOUT_MS,
        tls: TlsConfig::builder()
            .enabled(true)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .verification_mode(VerificationMode::SystemCa)
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert_eq!(config.timeout_ms, timeouts::DEFAULT_GRPC_TIMEOUT_MS);
    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
}

/// Test gRPC server TLS configuration with constants
#[tokio::test]
async fn test_grpc_server_tls_with_constants() {
    setup_test_environment();

    let config = GrpcServerConfig {
        bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
        port: network::DEFAULT_GRPC_SERVER_PORT,
        request_timeout_ms: timeouts::DEFAULT_GRPC_TIMEOUT_MS,
        tls: TlsConfig::builder()
            .enabled(true)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .verification_mode(VerificationMode::SystemCa)
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert_eq!(config.bind_address, network::DEFAULT_BIND_LOCALHOST);
    assert_eq!(config.port, network::DEFAULT_GRPC_SERVER_PORT);
    assert_eq!(config.request_timeout_ms, timeouts::DEFAULT_GRPC_TIMEOUT_MS);
    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
}

/// Test gRPC TLS configuration serialization and deserialization
#[tokio::test]
async fn test_grpc_tls_serialization() {
    setup_test_environment();

    let original_config = GrpcClientConfig::builder()
        .with_mutual_tls_with_ca("./tests/certs/ca.pem", "./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();

    // Serialize configuration
    let serialized =
        serde_json::to_string(&original_config).expect("Failed to serialize gRPC client config");

    // Deserialize configuration
    let deserialized_config: GrpcClientConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize gRPC client config");

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

/// Test gRPC TLS configuration builder method chaining
#[tokio::test]
async fn test_grpc_tls_builder_chaining() {
    setup_test_environment();

    let port = get_available_port();

    // Test complex builder chain for client
    let client_config = GrpcClientConfig::builder()
        .with_base_url("https://example.com:443")
        .with_timeout(5000)
        .with_max_connections(50)
        .with_retry_attempts(5)
        .with_tls_system_ca()
        .build();

    assert_eq!(
        client_config.base_url,
        Some("https://example.com:443".to_string())
    );
    assert_eq!(client_config.timeout_ms, 5000);
    assert_eq!(client_config.max_connections, 50);
    assert_eq!(client_config.retry_attempts, 5);
    assert!(client_config.tls.enabled);
    assert_eq!(
        client_config.tls.verification_mode,
        VerificationMode::SystemCa
    );

    // Test complex builder chain for server
    let server_config = GrpcServerConfig::builder()
        .with_bind_address("0.0.0.0")
        .with_port(port)
        .with_max_connections(1000)
        .with_request_timeout(10000)
        .with_tls_skip_verify()
        .build();

    assert_eq!(server_config.bind_address, "0.0.0.0");
    assert_eq!(server_config.port, port);
    assert_eq!(server_config.max_connections, 1000);
    assert_eq!(server_config.request_timeout_ms, 10000);
    assert!(server_config.tls.enabled);
    assert_eq!(server_config.tls.verification_mode, VerificationMode::Skip);
}

/// Test gRPC TLS configuration default values
#[tokio::test]
async fn test_grpc_tls_defaults() {
    setup_test_environment();

    let client_config = GrpcClientConfig::default();
    assert!(!client_config.tls.enabled);
    assert_eq!(
        client_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(client_config.tls.cert_path.is_none());
    assert!(client_config.tls.key_path.is_none());
    assert!(client_config.tls.ca_cert_path.is_none());

    let server_config = GrpcServerConfig::default();
    assert!(!server_config.tls.enabled);
    assert_eq!(
        server_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(server_config.tls.cert_path.is_none());
    assert!(server_config.tls.key_path.is_none());
    assert!(server_config.tls.ca_cert_path.is_none());
}

/// Test gRPC TLS configuration with different verification modes
#[tokio::test]
async fn test_grpc_tls_verification_modes() {
    setup_test_environment();

    // Test SystemCa mode
    let config = GrpcClientConfig::builder().with_tls_system_ca().build();
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);

    // Test CustomCa mode
    let config = GrpcClientConfig::builder()
        .with_tls_custom_ca("./tests/certs/ca.pem")
        .build();
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);

    // Test Skip mode
    let config = GrpcClientConfig::builder().with_tls_skip_verify().build();
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test MutualTls mode
    let config = GrpcClientConfig::builder()
        .with_mutual_tls("./tests/certs/client-cert.pem", "./tests/certs/client-key.pem")
        .build();
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
}

/// Test gRPC TLS configuration path handling
#[tokio::test]
async fn test_grpc_tls_path_handling() {
    setup_test_environment();

    let cert_path = "/etc/ssl/certs/grpc-cert.pem";
    let key_path = "/etc/ssl/private/grpc-key.pem";
    let ca_path = "/etc/ssl/certs/ca.pem";

    let config = GrpcClientConfig::builder()
        .with_mutual_tls_with_ca(ca_path, cert_path, key_path)
        .build();

    assert_eq!(config.tls.cert_path, Some(PathBuf::from(cert_path)));
    assert_eq!(config.tls.key_path, Some(PathBuf::from(key_path)));
    assert_eq!(config.tls.ca_cert_path, Some(PathBuf::from(ca_path)));
}

/// Test gRPC TLS configuration with environment variables
#[tokio::test]
async fn test_grpc_tls_with_env_vars() {
    setup_test_environment();

    // Set environment variables
    std::env::set_var("QOLLECTIVE_TLS_ENABLED", "true");
    std::env::set_var("QOLLECTIVE_TLS_CERT_PATH", "./tests/certs/client-cert.pem");
    std::env::set_var("QOLLECTIVE_TLS_KEY_PATH", "./tests/certs/client-key.pem");
    std::env::set_var("QOLLECTIVE_TLS_VERIFY_MODE", "skip");

    let tls_config = TlsConfig::from_env().expect("Failed to create TLS config from env");

    let grpc_config = GrpcClientConfig {
        tls: tls_config,
        ..Default::default()
    };

    assert!(grpc_config.tls.enabled);
    assert_eq!(
        grpc_config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(
        grpc_config.tls.key_path,
        Some(PathBuf::from("./tests/certs/client-key.pem"))
    );
    assert_eq!(grpc_config.tls.verification_mode, VerificationMode::Skip);

    // Clean up
    std::env::remove_var("QOLLECTIVE_TLS_ENABLED");
    std::env::remove_var("QOLLECTIVE_TLS_CERT_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_KEY_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_VERIFY_MODE");
}
