// ABOUTME: Comprehensive TLS tests for NATS transport protocol
// ABOUTME: Tests TLS builder methods, verification modes, and secure NATS messaging

use qollective::config::nats::{
    NatsClientBehaviorConfig, NatsClientConfig, NatsConfig, NatsConnectionConfig, NatsServerConfig,
};
use qollective::config::tls::{TlsConfig, VerificationMode};
use qollective::constants::timeouts;
use std::path::PathBuf;

mod common;
use common::setup_test_environment;

/// Test NATS client TLS configuration validation
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_client_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = NatsClientConfig::default();
    assert!(config.connection.tls.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(config.connection.tls.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::Skip)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(config.connection.tls.validate().is_ok());
}

/// Test NATS server TLS configuration validation
#[cfg(feature = "nats-server")]
#[tokio::test]
async fn test_nats_server_tls_validation() {
    setup_test_environment();

    // Test disabled TLS configuration - should pass validation
    let config = NatsConfig::default();
    assert!(config.connection.tls.validate().is_ok());

    // Test enabled TLS with system CA - should pass validation
    let config = NatsConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/server-cert.pem")
                .key_path("./tests/certs/server-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(config.connection.tls.validate().is_ok());

    // Test enabled TLS with skip verification - should pass validation
    let config = NatsConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::Skip)
                .cert_path("./tests/certs/server-cert.pem")
                .key_path("./tests/certs/server-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(config.connection.tls.validate().is_ok());
}

/// Test NATS client TLS configuration with constants
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_client_tls_with_constants() {
    setup_test_environment();

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            connection_timeout_ms: timeouts::DEFAULT_NATS_CONNECTION_TIMEOUT_MS,
            ..Default::default()
        },
        client_behavior: NatsClientBehaviorConfig {
            request_timeout_ms: timeouts::DEFAULT_NATS_REQUEST_TIMEOUT_MS,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        config.connection.connection_timeout_ms,
        timeouts::DEFAULT_NATS_CONNECTION_TIMEOUT_MS
    );
    assert_eq!(
        config.client_behavior.request_timeout_ms,
        timeouts::DEFAULT_NATS_REQUEST_TIMEOUT_MS
    );
    assert!(config.connection.tls.enabled);
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::SystemCa
    );
}

/// Test NATS server TLS configuration with constants
#[cfg(feature = "nats-server")]
#[tokio::test]
async fn test_nats_server_tls_with_constants() {
    setup_test_environment();

    let config = NatsConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/server-cert.pem")
                .key_path("./tests/certs/server-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        server: NatsServerConfig {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.server.enabled);
    assert!(config.connection.tls.enabled);
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::SystemCa
    );
}

/// Test NATS TLS configuration serialization and deserialization
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_serialization() {
    setup_test_environment();

    let original_config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::MutualTls)
                .ca_cert_path("./tests/certs/ca-cert.pem")
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Serialize configuration
    let serialized =
        serde_json::to_string(&original_config).expect("Failed to serialize NATS client config");

    // Deserialize configuration
    let deserialized_config: NatsClientConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize NATS client config");

    // Verify TLS configuration is preserved
    assert_eq!(
        original_config.connection.tls.enabled,
        deserialized_config.connection.tls.enabled
    );
    assert_eq!(
        original_config.connection.tls.verification_mode,
        deserialized_config.connection.tls.verification_mode
    );
    assert_eq!(
        original_config.connection.tls.cert_path,
        deserialized_config.connection.tls.cert_path
    );
    assert_eq!(
        original_config.connection.tls.key_path,
        deserialized_config.connection.tls.key_path
    );
    assert_eq!(
        original_config.connection.tls.ca_cert_path,
        deserialized_config.connection.tls.ca_cert_path
    );
}

/// Test NATS TLS configuration default values
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_defaults() {
    setup_test_environment();

    let client_config = NatsClientConfig::default();
    assert!(!client_config.connection.tls.enabled);
    assert_eq!(
        client_config.connection.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(client_config.connection.tls.cert_path.is_none());
    assert!(client_config.connection.tls.key_path.is_none());
    assert!(client_config.connection.tls.ca_cert_path.is_none());
}

/// Test NATS server TLS configuration default values
#[cfg(feature = "nats-server")]
#[tokio::test]
async fn test_nats_server_tls_defaults() {
    setup_test_environment();

    let server_config = NatsConfig::default();
    assert!(!server_config.connection.tls.enabled);
    assert_eq!(
        server_config.connection.tls.verification_mode,
        VerificationMode::SystemCa
    );
    assert!(server_config.connection.tls.cert_path.is_none());
    assert!(server_config.connection.tls.key_path.is_none());
    assert!(server_config.connection.tls.ca_cert_path.is_none());
}

/// Test NATS TLS configuration with different verification modes
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_verification_modes() {
    setup_test_environment();

    // Test SystemCa mode
    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::SystemCa
    );

    // Test CustomCa mode
    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::CustomCa)
                .ca_cert_path("./tests/certs/ca-cert.pem")
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::CustomCa
    );

    // Test Skip mode
    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::Skip)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::Skip
    );

    // Test MutualTls mode
    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::MutualTls)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::MutualTls
    );
}

/// Test NATS TLS configuration path handling
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_path_handling() {
    setup_test_environment();

    let cert_path = "/etc/ssl/certs/nats-cert.pem";
    let key_path = "/etc/ssl/private/nats-key.pem";
    let ca_path = "/etc/ssl/certs/ca.pem";

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::MutualTls)
                .ca_cert_path(ca_path)
                .cert_path(cert_path)
                .key_path(key_path)
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        config.connection.tls.cert_path,
        Some(PathBuf::from(cert_path))
    );
    assert_eq!(
        config.connection.tls.key_path,
        Some(PathBuf::from(key_path))
    );
    assert_eq!(
        config.connection.tls.ca_cert_path,
        Some(PathBuf::from(ca_path))
    );
}

/// Test NATS TLS configuration with environment variables
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_with_env_vars() {
    setup_test_environment();

    // Set environment variables
    std::env::set_var("QOLLECTIVE_TLS_ENABLED", "true");
    std::env::set_var("QOLLECTIVE_TLS_CERT_PATH", "./tests/certs/client-cert.pem");
    std::env::set_var("QOLLECTIVE_TLS_KEY_PATH", "./tests/certs/client-key.pem");
    std::env::set_var("QOLLECTIVE_TLS_VERIFY_MODE", "skip");

    let tls_config = TlsConfig::from_env().expect("Failed to create TLS config from env");

    let nats_config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: tls_config,
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(nats_config.connection.tls.enabled);
    assert_eq!(
        nats_config.connection.tls.cert_path,
        Some(PathBuf::from("./tests/certs/client-cert.pem"))
    );
    assert_eq!(
        nats_config.connection.tls.key_path,
        Some(PathBuf::from("./tests/certs/client-key.pem"))
    );
    assert_eq!(
        nats_config.connection.tls.verification_mode,
        VerificationMode::Skip
    );

    // Clean up
    std::env::remove_var("QOLLECTIVE_TLS_ENABLED");
    std::env::remove_var("QOLLECTIVE_TLS_CERT_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_KEY_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_VERIFY_MODE");
}

/// Test NATS TLS configuration with connection parameters
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_with_connection_params() {
    setup_test_environment();

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            connection_timeout_ms: timeouts::DEFAULT_NATS_CONNECTION_TIMEOUT_MS,
            max_reconnect_attempts: Some(5),
            reconnect_timeout_ms: 1000,
            ..Default::default()
        },
        client_behavior: NatsClientBehaviorConfig {
            request_timeout_ms: timeouts::DEFAULT_NATS_REQUEST_TIMEOUT_MS,
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    assert_eq!(
        config.connection.connection_timeout_ms,
        timeouts::DEFAULT_NATS_CONNECTION_TIMEOUT_MS
    );
    assert_eq!(
        config.client_behavior.request_timeout_ms,
        timeouts::DEFAULT_NATS_REQUEST_TIMEOUT_MS
    );
    assert_eq!(config.connection.max_reconnect_attempts, Some(5));
    assert_eq!(config.connection.reconnect_timeout_ms, 1000);
}

/// Test NATS TLS configuration with server cluster settings
#[cfg(feature = "nats-server")]
#[tokio::test]
async fn test_nats_tls_with_cluster_settings() {
    setup_test_environment();

    let config = NatsConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/server-cert.pem")
                .key_path("./tests/certs/server-key.pem")
                .build()
                .unwrap(),
            urls: vec![
                "nats://node1:6222".to_string(),
                "nats://node2:6222".to_string(),
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    assert_eq!(config.connection.urls.len(), 2);
    assert_eq!(config.connection.urls[0], "nats://node1:6222");
    assert_eq!(config.connection.urls[1], "nats://node2:6222");
}

/// Test NATS TLS configuration with JetStream settings
#[cfg(feature = "nats-server")]
#[tokio::test]
async fn test_nats_tls_with_jetstream_settings() {
    setup_test_environment();

    let config = NatsConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/server-cert.pem")
                .key_path("./tests/certs/server-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    // Note: JetStream configuration would be handled by the NATS server itself
    // and not directly in our client configuration structure
}

/// Test NATS TLS configuration with authentication settings
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_with_auth_settings() {
    setup_test_environment();

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            username: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            token: Some("testtoken123".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    assert_eq!(config.connection.username, Some("testuser".to_string()));
    assert_eq!(config.connection.password, Some("testpass".to_string()));
    assert_eq!(config.connection.token, Some("testtoken123".to_string()));
}

/// Test NATS TLS configuration with URL handling
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_with_url_handling() {
    setup_test_environment();

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            urls: vec![
                "nats://localhost:4222".to_string(),
                "tls://localhost:4443".to_string(),
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    assert_eq!(config.connection.urls.len(), 2);
    assert_eq!(config.connection.urls[0], "nats://localhost:4222");
    assert_eq!(config.connection.urls[1], "tls://localhost:4443");

    // Verify TLS URL is properly handled
    assert!(config.connection.urls[1].starts_with("tls://"));
}

/// Test NATS TLS configuration with performance settings
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_with_performance_settings() {
    setup_test_environment();

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    // Note: Performance settings like ping_interval, max_pings_out, etc. would be
    // configured through the NATS client library itself, not our config structure
}

/// Test NATS TLS configuration with connection pooling
#[cfg(feature = "nats-client")]
#[tokio::test]
async fn test_nats_tls_with_connection_pooling() {
    setup_test_environment();

    let config = NatsClientConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/client-cert.pem")
                .key_path("./tests/certs/client-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    // Note: Connection pooling is typically handled by the NATS client library, not our config
    assert_eq!(
        config.connection.tls.verification_mode,
        VerificationMode::SystemCa
    );
}

/// Test NATS TLS configuration with subject permissions
#[cfg(feature = "nats-server")]
#[tokio::test]
async fn test_nats_tls_with_subject_permissions() {
    setup_test_environment();

    let config = NatsConfig {
        connection: NatsConnectionConfig {
            tls: TlsConfig::builder()
                .enabled(true)
                .verification_mode(VerificationMode::SystemCa)
                .cert_path("./tests/certs/server-cert.pem")
                .key_path("./tests/certs/server-key.pem")
                .build()
                .unwrap(),
            ..Default::default()
        },
        server: NatsServerConfig {
            enabled: true,
            subject_prefix: "qollective.test".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(config.connection.tls.enabled);
    assert!(config.server.enabled);
    assert_eq!(config.server.subject_prefix, "qollective.test");
}
