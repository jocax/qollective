// ABOUTME: Comprehensive TLS tests for REST transport protocol
// ABOUTME: Tests TLS builder methods, verification modes, and secure HTTP communication

use qollective::config::presets::{RestClientConfig, RestServerConfig};
use qollective::config::tls::{TlsConfig, VerificationMode};
use qollective::constants::{network, timeouts};
use std::path::PathBuf;

mod common;
use common::{get_available_port, setup_test_environment};

/// Test REST client TLS configuration creation with builder methods
#[tokio::test]
async fn test_rest_client_tls_builder_methods() {
    setup_test_environment();

    // Test system CA verification
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
    assert_eq!(config.tls.cert_path, Some(PathBuf::from("./tests/certs/client-cert.pem")));
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
    assert!(config.tls.ca_cert_path.is_none());

    // Test custom CA verification
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::CustomCa)
            .ca_cert_path("./tests/certs/ca-cert.pem")
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca-cert.pem"))
    );
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(
        config.tls.key_path,
        Some(PathBuf::from("./tests/certs/client-key.pem"))
    );

    // Test skip verification
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::Skip)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);
    assert_eq!(config.tls.cert_path, Some(PathBuf::from("./tests/certs/client-cert.pem")));
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));

    // Test mutual TLS
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));

    // Test mutual TLS with custom CA
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .ca_cert_path("./tests/certs/ca-cert.pem")
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

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

/// Test REST server TLS configuration creation with builder methods
#[tokio::test]
async fn test_rest_server_tls_builder_methods() {
    setup_test_environment();

    // Test system CA verification
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
    assert_eq!(config.tls.cert_path, Some(PathBuf::from("./tests/certs/server-cert.pem")));
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/server-key.pem")));

    // Test custom CA verification
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::CustomCa)
            .ca_cert_path("./tests/certs/ca-cert.pem")
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
    assert_eq!(
        config.tls.ca_cert_path,
        Some(PathBuf::from("./tests/certs/ca-cert.pem"))
    );
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/server-cert.pem"))
    );
    assert_eq!(
        config.tls.key_path,
        Some(PathBuf::from("./tests/certs/server-key.pem"))
    );

    // Test skip verification
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::Skip)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);
    assert_eq!(config.tls.cert_path, Some(PathBuf::from("./tests/certs/server-cert.pem")));
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/server-key.pem")));

    // Test mutual TLS
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
    assert_eq!(
        config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(config.tls.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));

    // Test mutual TLS with custom CA
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .ca_cert_path("./tests/certs/ca-cert.pem")
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

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

/// Test REST client TLS configuration validation
#[tokio::test]
async fn test_rest_client_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = RestClientConfig::default();
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::Skip)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert!(config.tls.validate().is_ok());
}

/// Test REST server TLS configuration validation
#[tokio::test]
async fn test_rest_server_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = RestServerConfig::default();
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert!(config.tls.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::Skip)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert!(config.tls.validate().is_ok());
}

/// Test REST client TLS configuration with constants
#[tokio::test]
async fn test_rest_client_tls_with_constants() {
    setup_test_environment();

    let config = RestClientConfig {
        timeout_ms: timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS,
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert_eq!(config.timeout_ms, timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS);
    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
}

/// Test REST server TLS configuration with constants
#[tokio::test]
async fn test_rest_server_tls_with_constants() {
    setup_test_environment();

    let config = RestServerConfig {
        bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
        port: network::DEFAULT_REST_SERVER_PORT,
        request_timeout_ms: timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS,
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
        config.request_timeout_ms,
        timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS
    );
    assert!(config.tls.enabled);
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
}

/// Test REST TLS configuration serialization and deserialization
#[tokio::test]
async fn test_rest_tls_serialization() {
    setup_test_environment();

    let original_config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .ca_cert_path("./tests/certs/ca-cert.pem")
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };

    // Serialize configuration
    let serialized =
        serde_json::to_string(&original_config).expect("Failed to serialize REST client config");

    // Deserialize configuration
    let deserialized_config: RestClientConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize REST client config");

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

/// Test REST TLS configuration default values
#[tokio::test]
async fn test_rest_tls_defaults() {
    setup_test_environment();

    let client_config = RestClientConfig::default();
    assert!(!client_config.tls.enabled);
    assert_eq!(
        client_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(client_config.tls.cert_path.is_none());
    assert!(client_config.tls.key_path.is_none());
    assert!(client_config.tls.ca_cert_path.is_none());

    let server_config = RestServerConfig::default();
    assert!(!server_config.tls.enabled);
    assert_eq!(
        server_config.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(server_config.tls.cert_path.is_none());
    assert!(server_config.tls.key_path.is_none());
    assert!(server_config.tls.ca_cert_path.is_none());
}

/// Test REST TLS configuration with different verification modes
#[tokio::test]
async fn test_rest_tls_verification_modes() {
    setup_test_environment();

    // Test SystemCa mode
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);

    // Test CustomCa mode
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::CustomCa)
            .ca_cert_path("./tests/certs/ca-cert.pem")
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);

    // Test Skip mode
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::Skip)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert_eq!(config.tls.verification_mode, VerificationMode::Skip);

    // Test MutualTls mode
    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        ..Default::default()
    };
    assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
}

/// Test REST TLS configuration path handling
#[tokio::test]
async fn test_rest_tls_path_handling() {
    setup_test_environment();

    let cert_path = "/etc/ssl/certs/rest-cert.pem";
    let key_path = "/etc/ssl/private/rest-key.pem";
    let ca_path = "/etc/ssl/certs/ca.pem";

    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::MutualTls)
            .ca_cert_path(ca_path)
            .cert_path(cert_path)
            .key_path(key_path)
            .build()
            .unwrap(),
        ..Default::default()
    };

    assert_eq!(config.tls.cert_path, Some(PathBuf::from(cert_path)));
    assert_eq!(config.tls.key_path, Some(PathBuf::from(key_path)));
    assert_eq!(config.tls.ca_cert_path, Some(PathBuf::from(ca_path)));
}

/// Test REST TLS configuration with environment variables
#[tokio::test]
async fn test_rest_tls_with_env_vars() {
    setup_test_environment();

    // Set environment variables
    std::env::set_var("QOLLECTIVE_TLS_ENABLED", "true");
    std::env::set_var("QOLLECTIVE_TLS_CERT_PATH", "./tests/certs/client-cert.pem");
    std::env::set_var("QOLLECTIVE_TLS_KEY_PATH", "./tests/certs/client-key.pem");
    std::env::set_var("QOLLECTIVE_TLS_VERIFY_MODE", "skip");

    let tls_config = TlsConfig::from_env().expect("Failed to create TLS config from env");

    let rest_config = RestClientConfig {
        tls: tls_config,
        ..Default::default()
    };

    assert!(rest_config.tls.enabled);
    assert_eq!(
        rest_config.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(
        rest_config.tls.key_path,
        Some(PathBuf::from("./tests/certs/client-key.pem"))
    );
    assert_eq!(rest_config.tls.verification_mode, VerificationMode::Skip);

    // Clean up
    std::env::remove_var("QOLLECTIVE_TLS_ENABLED");
    std::env::remove_var("QOLLECTIVE_TLS_CERT_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_KEY_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_VERIFY_MODE");
}

/// Test REST TLS configuration with CORS
#[tokio::test]
async fn test_rest_tls_with_cors() {
    setup_test_environment();

    let config = RestServerConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/server-cert.pem")
            .key_path("./tests/certs/server-key.pem")
            .build()
            .unwrap(),
        cors: qollective::config::presets::CorsConfig {
            enabled: true,
            allowed_origins: vec!["https://example.com".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age_seconds: 3600,
        },
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert!(config.cors.enabled);
    assert_eq!(config.cors.allowed_origins, vec!["https://example.com"]);
    assert_eq!(config.cors.allowed_methods, vec!["GET", "POST"]);
}

/// Test REST TLS configuration with performance settings
#[tokio::test]
async fn test_rest_tls_with_performance() {
    setup_test_environment();

    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        performance: qollective::config::presets::PerformanceConfig {
            enabled: true,
            track_request_duration: true,
            track_response_size: true,
            track_connection_pool: true,
            benchmarking_enabled: false,
            metrics_collection: true,
        },
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert!(config.performance.enabled);
    assert!(config.performance.track_request_duration);
    assert!(config.performance.track_response_size);
    assert!(config.performance.track_connection_pool);
    assert!(config.performance.metrics_collection);
}

/// Test REST TLS configuration with logging
#[tokio::test]
async fn test_rest_tls_with_logging() {
    setup_test_environment();

    let config = RestClientConfig {
        tls: TlsConfig::builder()
            .enabled(true)
            .verification_mode(VerificationMode::SystemCa)
            .cert_path("./tests/certs/client-cert.pem")
            .key_path("./tests/certs/client-key.pem")
            .build()
            .unwrap(),
        logging: qollective::config::presets::LoggingConfig {
            enabled: true,
            log_requests: true,
            log_responses: false,
            log_headers: true,
            log_body: false,
            log_level: "info".to_string(),
            structured_logging: true,
        },
        ..Default::default()
    };

    assert!(config.tls.enabled);
    assert!(config.logging.enabled);
    assert!(config.logging.log_requests);
    assert!(!config.logging.log_responses);
    assert!(config.logging.log_headers);
    assert!(!config.logging.log_body);
    assert_eq!(config.logging.log_level, "info");
    assert!(config.logging.structured_logging);
}
