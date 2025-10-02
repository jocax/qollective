// ABOUTME: Comprehensive tests for unified TLS configuration across all transports
// ABOUTME: Tests TLS config builder, verification modes, path expansion, and cross-transport compatibility

use qollective::config::tls::{TlsConfig, TlsConfigBuilder, VerificationMode};
use qollective::constants::tls as tls_constants;
use std::path::PathBuf;

mod common;
use common::setup_test_environment;

/// Test TLS configuration builder pattern
#[tokio::test]
async fn test_tls_config_builder() {
    setup_test_environment();

    // Test basic builder
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();

    assert!(config.enabled);
    assert_eq!(config.cert_path, Some(PathBuf::from("./tests/certs/client-cert.pem")));
    assert_eq!(config.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
    assert_eq!(config.verification_mode, VerificationMode::SystemCa);

    // Test builder with CA certificate
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .ca_cert_path("./tests/certs/ca-cert.pem")
        .verification_mode(VerificationMode::CustomCa)
        .build()
        .unwrap();

    assert!(config.enabled);
    assert_eq!(config.ca_cert_path, Some(PathBuf::from("./tests/certs/ca-cert.pem")));
    assert_eq!(config.verification_mode, VerificationMode::CustomCa);

    // Test builder with verify_certificates method
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verify_certificates(false)
        .build()
        .unwrap();

    assert!(config.enabled);
    assert_eq!(config.verification_mode, VerificationMode::Skip);
}

/// Test TLS configuration default values
#[tokio::test]
async fn test_tls_config_defaults() {
    setup_test_environment();

    let config = TlsConfig::default();
    assert!(!config.enabled);
    assert!(config.cert_path.is_none());
    assert!(config.key_path.is_none());
    assert!(config.ca_cert_path.is_none());
    assert_eq!(config.verification_mode, VerificationMode::SystemCa);
}

/// Test TLS configuration validation
#[tokio::test]
async fn test_tls_config_validation() {
    setup_test_environment();

    // Test disabled TLS - should pass validation
    let config = TlsConfig::default();
    assert!(config.validate().is_ok());

    // Test enabled TLS without cert path - should fail
    let config = TlsConfig::builder().enabled(true).build();
    assert!(config.is_err());

    // Test enabled TLS without key path - should fail
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .build();
    assert!(config.is_err());

    // Test CustomCa mode without CA path - should fail
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::CustomCa)
        .build();
    assert!(config.is_err());

    // Test valid configuration
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();
    assert!(config.validate().is_ok());
}

/// Test TLS configuration from environment variables
#[tokio::test]
async fn test_tls_config_from_env() {
    setup_test_environment();

    // Test with basic environment variables
    std::env::set_var(tls_constants::env_vars::QOLLECTIVE_TLS_ENABLED, "true");
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_CERT_PATH,
        "./tests/certs/client-cert.pem",
    );
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_KEY_PATH,
        "./tests/certs/client-key.pem",
    );
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "system_ca",
    );

    let config = TlsConfig::from_env().unwrap();
    assert!(config.enabled);
    assert_eq!(config.cert_path, Some(PathBuf::from("./tests/certs/client-cert.pem")));
    assert_eq!(config.key_path, Some(PathBuf::from("./tests/certs/client-key.pem")));
    assert_eq!(config.verification_mode, VerificationMode::SystemCa);

    // Test with CA certificate
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_CA_PATH,
        "./tests/certs/ca-cert.pem",
    );
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "custom_ca",
    );

    let config = TlsConfig::from_env().unwrap();
    assert_eq!(config.ca_cert_path, Some(PathBuf::from("./tests/certs/ca-cert.pem")));
    assert_eq!(config.verification_mode, VerificationMode::CustomCa);

    // Test with different verification modes
    std::env::set_var(tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE, "skip");
    let config = TlsConfig::from_env().unwrap();
    assert_eq!(config.verification_mode, VerificationMode::Skip);

    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "mutual_tls",
    );
    let config = TlsConfig::from_env().unwrap();
    assert_eq!(config.verification_mode, VerificationMode::MutualTls);

    // Test with alternative verification mode formats
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "systemca",
    );
    let config = TlsConfig::from_env().unwrap();
    assert_eq!(config.verification_mode, VerificationMode::SystemCa);

    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "customca",
    );
    let config = TlsConfig::from_env().unwrap();
    assert_eq!(config.verification_mode, VerificationMode::CustomCa);

    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "mutualtls",
    );
    let config = TlsConfig::from_env().unwrap();
    assert_eq!(config.verification_mode, VerificationMode::MutualTls);

    // Test with invalid verification mode
    std::env::set_var(
        tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE,
        "invalid",
    );
    let config = TlsConfig::from_env();
    assert!(config.is_err());

    // Clean up
    std::env::remove_var(tls_constants::env_vars::QOLLECTIVE_TLS_ENABLED);
    std::env::remove_var(tls_constants::env_vars::QOLLECTIVE_TLS_CERT_PATH);
    std::env::remove_var(tls_constants::env_vars::QOLLECTIVE_TLS_KEY_PATH);
    std::env::remove_var(tls_constants::env_vars::QOLLECTIVE_TLS_CA_PATH);
    std::env::remove_var(tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE);
}

/// Test TLS configuration path expansion
#[tokio::test]
async fn test_tls_config_path_expansion() {
    setup_test_environment();

    // Test basic path (no expansion needed)
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("/absolute/path/to/cert.pem")
        .key_path("/absolute/path/to/key.pem")
        .build()
        .unwrap();

    assert_eq!(
        config.cert_path,
        Some(PathBuf::from("/absolute/path/to/cert.pem"))
    );
    assert_eq!(
        config.key_path,
        Some(PathBuf::from("/absolute/path/to/key.pem"))
    );

    // Test home directory expansion
    std::env::set_var("HOME", "/home/user");

    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("~/certs/cert.pem")
        .key_path("~/certs/key.pem")
        .build()
        .unwrap();

    let cert_path = config.cert_path.as_ref().unwrap();
    let key_path = config.key_path.as_ref().unwrap();
    assert!(cert_path
        .to_string_lossy()
        .contains("/home/user/certs/cert.pem"));
    assert!(key_path
        .to_string_lossy()
        .contains("/home/user/certs/key.pem"));

    // Test environment variable expansion
    std::env::set_var("CERT_DIR", "/etc/ssl/certs");
    std::env::set_var("KEY_DIR", "/etc/ssl/private");

    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("${CERT_DIR}/cert.pem")
        .key_path("${KEY_DIR}/key.pem")
        .build()
        .unwrap();

    let cert_path = config.cert_path.as_ref().unwrap();
    let key_path = config.key_path.as_ref().unwrap();
    assert!(cert_path
        .to_string_lossy()
        .contains("/etc/ssl/certs/cert.pem"));
    assert!(key_path
        .to_string_lossy()
        .contains("/etc/ssl/private/key.pem"));

    // Clean up
    std::env::remove_var("HOME");
    std::env::remove_var("CERT_DIR");
    std::env::remove_var("KEY_DIR");
}

/// Test TLS configuration serialization and deserialization
#[tokio::test]
async fn test_tls_config_serialization() {
    setup_test_environment();

    let original_config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .ca_cert_path("./tests/certs/ca-cert.pem")
        .verification_mode(VerificationMode::MutualTls)
        .build()
        .unwrap();

    // Serialize to JSON
    let serialized =
        serde_json::to_string(&original_config).expect("Failed to serialize TLS config");

    // Deserialize from JSON
    let deserialized_config: TlsConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize TLS config");

    // Verify all fields are preserved
    assert_eq!(original_config.enabled, deserialized_config.enabled);
    assert_eq!(original_config.cert_path, deserialized_config.cert_path);
    assert_eq!(original_config.key_path, deserialized_config.key_path);
    assert_eq!(
        original_config.ca_cert_path,
        deserialized_config.ca_cert_path
    );
    assert_eq!(
        original_config.verification_mode,
        deserialized_config.verification_mode
    );
}

/// Test TLS verification mode serialization
#[tokio::test]
async fn test_verification_mode_serialization() {
    setup_test_environment();

    let modes = vec![
        VerificationMode::SystemCa,
        VerificationMode::CustomCa,
        VerificationMode::Skip,
        VerificationMode::MutualTls,
    ];

    for mode in modes {
        let serialized =
            serde_json::to_string(&mode).expect("Failed to serialize verification mode");
        let deserialized: VerificationMode =
            serde_json::from_str(&serialized).expect("Failed to deserialize verification mode");
        assert_eq!(mode, deserialized);
    }
}

/// Test TLS configuration preset configurations
#[tokio::test]
async fn test_tls_config_presets() {
    setup_test_environment();

    // Test production preset
    let prod_config = TlsConfig::production();
    assert!(prod_config.enabled);
    assert_eq!(prod_config.verification_mode, VerificationMode::SystemCa);
    assert!(prod_config.cert_path.is_some());
    assert!(prod_config.key_path.is_some());
    assert!(prod_config.ca_cert_path.is_none());

    // Test development preset
    let dev_config = TlsConfig::development();
    assert!(dev_config.enabled);
    assert_eq!(dev_config.verification_mode, VerificationMode::Skip);
    assert!(dev_config.cert_path.is_some());
    assert!(dev_config.key_path.is_some());

    // Test high-performance preset
    let perf_config = TlsConfig::high_performance();
    assert!(perf_config.enabled);
    assert_eq!(perf_config.verification_mode, VerificationMode::SystemCa);
    assert!(perf_config.cert_path.is_some());
    assert!(perf_config.key_path.is_some());
}

/// Test TLS configuration with different verification modes
#[tokio::test]
async fn test_tls_verification_modes() {
    setup_test_environment();

    // Test SystemCa mode
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();
    assert_eq!(config.verification_mode, VerificationMode::SystemCa);
    assert!(config.validate().is_ok());

    // Test CustomCa mode
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .ca_cert_path("./tests/certs/ca-cert.pem")
        .verification_mode(VerificationMode::CustomCa)
        .build()
        .unwrap();
    assert_eq!(config.verification_mode, VerificationMode::CustomCa);
    assert!(config.validate().is_ok());

    // Test Skip mode
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::Skip)
        .build()
        .unwrap();
    assert_eq!(config.verification_mode, VerificationMode::Skip);
    assert!(config.validate().is_ok());

    // Test MutualTls mode
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::MutualTls)
        .build()
        .unwrap();
    assert_eq!(config.verification_mode, VerificationMode::MutualTls);
    assert!(config.validate().is_ok());

    // Test MutualTls mode with CA
    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .ca_cert_path("./tests/certs/ca-cert.pem")
        .verification_mode(VerificationMode::MutualTls)
        .build()
        .unwrap();
    assert_eq!(config.verification_mode, VerificationMode::MutualTls);
    assert!(config.validate().is_ok());
}

/// Test TLS configuration builder error handling
#[tokio::test]
async fn test_tls_config_builder_errors() {
    setup_test_environment();

    // Test enabled TLS without cert path
    let result = TlsConfig::builder()
        .enabled(true)
        .key_path("./tests/certs/client-key.pem")
        .build();
    assert!(result.is_err());

    // Test enabled TLS without key path
    let result = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .build();
    assert!(result.is_err());

    // Test CustomCa mode without CA path
    let result = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::CustomCa)
        .build();
    assert!(result.is_err());
}

/// Test TLS configuration compatibility across transports
#[tokio::test]
async fn test_tls_config_transport_compatibility() {
    setup_test_environment();

    // Create a common TLS configuration
    let tls_config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();

    // Test that the same TLS config can be used across different transports
    let grpc_config = qollective::config::grpc::GrpcClientConfig {
        tls: tls_config.clone(),
        ..Default::default()
    };

    let rest_config = qollective::config::presets::RestClientConfig {
        tls: tls_config.clone(),
        ..Default::default()
    };

    #[cfg(feature = "websocket-client")]
    let websocket_config = qollective::config::websocket::WebSocketClientConfig {
        tls: tls_config.clone(),
        ..Default::default()
    };

    #[cfg(feature = "nats-client")]
    let nats_config = qollective::config::nats::NatsClientConfig {
        connection: qollective::config::nats::NatsConnectionConfig {
            tls: tls_config.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Verify all configurations have the same TLS settings
    assert_eq!(grpc_config.tls.enabled, tls_config.enabled);
    assert_eq!(
        grpc_config.tls.verification_mode,
        tls_config.verification_mode
    );
    assert_eq!(grpc_config.tls.cert_path, tls_config.cert_path);
    assert_eq!(grpc_config.tls.key_path, tls_config.key_path);

    assert_eq!(rest_config.tls.enabled, tls_config.enabled);
    assert_eq!(
        rest_config.tls.verification_mode,
        tls_config.verification_mode
    );
    assert_eq!(rest_config.tls.cert_path, tls_config.cert_path);
    assert_eq!(rest_config.tls.key_path, tls_config.key_path);

    #[cfg(feature = "websocket-client")]
    {
        assert_eq!(websocket_config.tls.enabled, tls_config.enabled);
        assert_eq!(
            websocket_config.tls.verification_mode,
            tls_config.verification_mode
        );
        assert_eq!(websocket_config.tls.cert_path, tls_config.cert_path);
        assert_eq!(websocket_config.tls.key_path, tls_config.key_path);
    }

    #[cfg(feature = "nats-client")]
    {
        assert_eq!(nats_config.connection.tls.enabled, tls_config.enabled);
        assert_eq!(
            nats_config.connection.tls.verification_mode,
            tls_config.verification_mode
        );
        assert_eq!(nats_config.connection.tls.cert_path, tls_config.cert_path);
        assert_eq!(nats_config.connection.tls.key_path, tls_config.key_path);
    }
}

/// Test TLS configuration equality and cloning
#[tokio::test]
async fn test_tls_config_equality_and_cloning() {
    setup_test_environment();

    let config1 = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();

    let config2 = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();

    let config3 = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/server-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();

    // Test equality
    assert_eq!(config1, config2);
    assert_ne!(config1, config3);

    // Test cloning
    let cloned_config = config1.clone();
    assert_eq!(config1, cloned_config);
    assert_eq!(config1.enabled, cloned_config.enabled);
    assert_eq!(config1.cert_path, cloned_config.cert_path);
    assert_eq!(config1.key_path, cloned_config.key_path);
    assert_eq!(config1.verification_mode, cloned_config.verification_mode);
}

/// Test TLS configuration debug output
#[tokio::test]
async fn test_tls_config_debug_output() {
    setup_test_environment();

    let config = TlsConfig::builder()
        .enabled(true)
        .cert_path("./tests/certs/client-cert.pem")
        .key_path("./tests/certs/client-key.pem")
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .unwrap();

    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("enabled: true"));
    assert!(debug_output.contains("cert.pem"));
    assert!(debug_output.contains("key.pem"));
    assert!(debug_output.contains("SystemCa"));
}
