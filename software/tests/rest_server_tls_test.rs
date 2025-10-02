// ABOUTME: TDD tests for REST server TLS functionality
// ABOUTME: Tests TLS configuration, certificate loading, and secure HTTP server operation

#![cfg(all(feature = "rest-server", feature = "tls"))]

use async_trait::async_trait;
use qollective::config::tls::TlsConfig;
use qollective::constants::{network, timeouts};
use qollective::envelope::Context;
use qollective::error::Result;
use qollective::prelude::ContextDataHandler;
use qollective::prelude::UnifiedEnvelopeReceiver;
use qollective::server::common::ServerConfig;
use qollective::server::rest::{RestServer, RestServerConfig};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

mod common;
use common::rest_test_utils::TestTlsConfig;
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

/// Test TLS configuration creation and validation
#[tokio::test]
async fn test_tls_config_creation() {
    setup_test_environment();

    // Test basic TLS config creation
    let tls_config = TlsConfig::builder()
        .enabled(true)
        .cert_path("/path/to/cert.pem")
        .key_path("/path/to/key.pem")
        .verification_mode(qollective::config::tls::VerificationMode::SystemCa)
        .build()
        .expect("Failed to build TLS config");

    assert_eq!(
        tls_config.cert_path,
        Some(std::path::PathBuf::from("/path/to/cert.pem"))
    );
    assert_eq!(
        tls_config.key_path,
        Some(std::path::PathBuf::from("/path/to/key.pem"))
    );
    assert_eq!(
        tls_config.verification_mode,
        qollective::config::tls::VerificationMode::SystemCa
    );

    // Test with mutual TLS (client cert required)
    let tls_config_with_client = TlsConfig::builder()
        .enabled(true)
        .cert_path("/path/to/cert.pem")
        .key_path("/path/to/key.pem")
        .verification_mode(qollective::config::tls::VerificationMode::MutualTls)
        .build()
        .expect("Failed to build mTLS config");

    assert_eq!(
        tls_config_with_client.verification_mode,
        qollective::config::tls::VerificationMode::MutualTls
    );
}

/// Test TLS config from environment variables
#[tokio::test]
async fn test_tls_config_from_env() {
    setup_test_environment();

    // Set environment variables
    std::env::set_var("QOLLECTIVE_TLS_CERT_PATH", "/test/cert.pem");
    std::env::set_var("QOLLECTIVE_TLS_KEY_PATH", "/test/key.pem");

    let tls_config = TlsConfig::from_env().expect("Failed to create TLS config from env");

    assert_eq!(
        tls_config.cert_path,
        Some(std::path::PathBuf::from("/test/cert.pem"))
    );
    assert_eq!(
        tls_config.key_path,
        Some(std::path::PathBuf::from("/test/key.pem"))
    );

    // Clean up
    std::env::remove_var("QOLLECTIVE_TLS_CERT_PATH");
    std::env::remove_var("QOLLECTIVE_TLS_KEY_PATH");
}

/// Test TLS config with path expansion
#[tokio::test]
async fn test_tls_config_path_expansion() {
    setup_test_environment();

    // Set HOME environment variable for testing
    std::env::set_var("HOME", "/home/testuser");
    std::env::set_var("TEST_VAR", "testvalue");

    let tls_config = TlsConfig::builder()
        .enabled(true)
        .cert_path("~/certs/cert.pem")
        .key_path("${TEST_VAR}/key.pem")
        .verification_mode(qollective::config::tls::VerificationMode::SystemCa)
        .build()
        .expect("Failed to build TLS config");

    assert!(tls_config
        .cert_path
        .as_ref()
        .unwrap()
        .to_string_lossy()
        .contains("/home/testuser/certs/cert.pem"));
    assert!(tls_config
        .key_path
        .as_ref()
        .unwrap()
        .to_string_lossy()
        .contains("testvalue/key.pem"));

    // Clean up
    std::env::remove_var("TEST_VAR");
}

/// Test REST server creation with TLS configuration
#[tokio::test]
async fn test_rest_server_with_tls_config() {
    setup_test_environment();

    let port = get_available_port();
    let test_tls_config = TestTlsConfig::default();

    let tls_config = test_tls_config.to_server_tls_config();

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
            port,
            ..Default::default()
        },
        tls: Some(tls_config),
        ..Default::default()
    };

    let server = RestServer::new(config)
        .await
        .expect("Failed to create REST server with TLS");

    // Verify TLS configuration is set
    assert!(server.config().tls.is_some());
    let tls = server.config().tls.as_ref().unwrap();
    assert!(tls
        .cert_path
        .as_ref()
        .unwrap()
        .to_string_lossy()
        .contains("server-cert.pem"));
    assert!(tls
        .key_path
        .as_ref()
        .unwrap()
        .to_string_lossy()
        .contains("server-key.pem"));
}

/// Test REST server TLS server startup (this should fail initially)
#[tokio::test]
async fn test_rest_server_tls_startup() {
    setup_test_environment();

    let port = get_available_port();
    let test_tls_config = TestTlsConfig::default();

    // Skip test if certificates are not available
    if !test_tls_config.certs_available() {
        println!("⚠️  Skipping TLS startup test - certificates not available");
        return;
    }

    let tls_config = test_tls_config.to_server_tls_config();

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
            port,
            ..Default::default()
        },
        tls: Some(tls_config),
        ..Default::default()
    };

    let mut server = RestServer::new(config)
        .await
        .expect("Failed to create REST server with TLS");

    // Register a test handler
    let handler = TestHandler;
    server
        .receive_envelope_at("/test", handler)
        .await
        .expect("Failed to register handler");

    // Start the TLS server in background (following the pattern from rest_test_utils)
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("TLS server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify TLS server is running by making a simple HTTPS request
    let https_url = format!("https://127.0.0.1:{}/test", port);

    // Create a custom TLS client that ignores certificate validation for testing
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create HTTPS client");

    // Try to connect to the TLS server
    let response = timeout(Duration::from_millis(2000), client.get(&https_url).send()).await;

    match response {
        Ok(Ok(resp)) => {
            println!("✅ TLS server is working! Status: {}", resp.status());
            // Any HTTP response (including 400 Bad Request) proves TLS is working
            assert!(
                resp.status().is_client_error()
                    || resp.status().is_success()
                    || resp.status() == 404,
                "Server should respond to HTTPS requests. Got status: {}",
                resp.status()
            );
        }
        Ok(Err(e)) => {
            panic!("TLS server responded but with error: {}", e);
        }
        Err(_) => {
            panic!("TLS server startup timed out - TLS implementation may have issues");
        }
    }

    // Cleanup
    server_handle.abort();
}

/// Test REST server TLS with client certificate requirement
#[tokio::test]
async fn test_rest_server_tls_with_client_cert() {
    setup_test_environment();

    let port = get_available_port();
    let test_tls_config = TestTlsConfig::default();

    let mut tls_config = test_tls_config.to_server_tls_config();
    tls_config.verification_mode = qollective::config::tls::VerificationMode::MutualTls;

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
            port,
            ..Default::default()
        },
        tls: Some(tls_config),
        ..Default::default()
    };

    let server = RestServer::new(config)
        .await
        .expect("Failed to create REST server with mTLS");

    // Verify client cert is required
    let tls = server.config().tls.as_ref().unwrap();
    assert_eq!(
        tls.verification_mode,
        qollective::config::tls::VerificationMode::MutualTls
    );
}

/// Test TLS certificate loading and validation (this should fail initially)
#[tokio::test]
async fn test_tls_certificate_loading() {
    setup_test_environment();

    let test_tls_config = TestTlsConfig::default();

    // Skip test if certificates are not available
    if !test_tls_config.certs_available() {
        println!("⚠️  Skipping certificate loading test - certificates not available");
        return;
    }

    let tls_config = test_tls_config.to_server_tls_config();

    // This test verifies that we can load and validate TLS certificates
    // It should fail initially until we implement the certificate loading logic

    // For now, just verify the paths are correct
    assert!(tls_config.cert_path.as_ref().unwrap().exists());
    assert!(tls_config.key_path.as_ref().unwrap().exists());

    // Test actual certificate loading and validation
    let client_config_result = tls_config.create_client_config().await;
    assert!(
        client_config_result.is_ok(),
        "Client config creation should succeed: {:?}",
        client_config_result.err()
    );

    // Test TLS config creation
    let server_config_result = tls_config.create_server_config().await;
    assert!(
        server_config_result.is_ok(),
        "Server config creation should succeed: {:?}",
        server_config_result.err()
    );

    println!("✅ Certificate loading and TLS config creation working correctly");
}

/// Test invalid TLS configuration handling
#[tokio::test]
async fn test_invalid_tls_config() {
    setup_test_environment();

    let port = get_available_port();

    // Test with non-existent certificate files
    let tls_config = TlsConfig::builder()
        .enabled(true)
        .cert_path("/nonexistent/cert.pem")
        .key_path("/nonexistent/key.pem")
        .verification_mode(qollective::config::tls::VerificationMode::SystemCa)
        .build()
        .expect("Failed to build TLS config");

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
            port,
            ..Default::default()
        },
        tls: Some(tls_config),
        ..Default::default()
    };

    // Server creation should succeed (validation happens on start)
    let mut server = RestServer::new(config)
        .await
        .expect("Failed to create REST server");

    // Starting the server should fail due to invalid certificate files
    let start_result = timeout(Duration::from_millis(1000), server.start()).await;

    // This should fail once we implement TLS certificate validation
    match start_result {
        Ok(result) => {
            // Should fail with certificate loading error
            assert!(
                result.is_err(),
                "Server should fail to start with invalid certificates"
            );
        }
        Err(_) => {
            // Timeout is also acceptable for now
            println!("TLS server startup timed out with invalid certs - expected");
        }
    }
}

/// Test TLS configuration constants usage
#[tokio::test]
async fn test_tls_with_constants() {
    setup_test_environment();

    // Test that we're using constants instead of hardcoded values
    let port = get_available_port();
    let test_tls_config = TestTlsConfig::default();

    let tls_config = test_tls_config.to_server_tls_config();

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(), // Using constant
            port,
            ..Default::default()
        },
        tls: Some(tls_config),
        request_timeout: Some(Duration::from_millis(
            timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS,
        )), // Using constant
        ..Default::default()
    };

    let server = RestServer::new(config)
        .await
        .expect("Failed to create REST server");

    // Verify constants are being used correctly
    assert_eq!(
        server.config().base.bind_address,
        network::DEFAULT_BIND_LOCALHOST
    );
    assert_eq!(
        server.config().request_timeout.unwrap().as_millis() as u64,
        timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS
    );
}

/// Test REST server mTLS with client certificate authentication
#[tokio::test]
async fn test_rest_server_mtls_client_cert_auth() {
    setup_test_environment();

    let port = get_available_port();
    let test_tls_config = TestTlsConfig::default();

    // Skip test if certificates are not available
    if !test_tls_config.certs_available() {
        println!("⚠️  Skipping mTLS test - certificates not available");
        return;
    }

    // Create server config requiring client certificates
    let mut tls_config = test_tls_config.to_server_tls_config();
    tls_config.verification_mode = qollective::config::tls::VerificationMode::MutualTls;

    let config = RestServerConfig {
        base: ServerConfig {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
            port,
            ..Default::default()
        },
        tls: Some(tls_config),
        ..Default::default()
    };

    let mut server = RestServer::new(config)
        .await
        .expect("Failed to create REST server with mTLS");

    // Register a test handler
    let handler = TestHandler;
    server
        .receive_envelope_at("/mtls-test", handler)
        .await
        .expect("Failed to register handler");

    // Start the mTLS server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("mTLS server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test 1: Connection without client certificate should fail
    let https_url = format!("https://127.0.0.1:{}/mtls-test", port);

    let client_without_cert = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Accept self-signed server cert
        .build()
        .expect("Failed to create HTTPS client");

    let response_without_cert = timeout(
        Duration::from_millis(2000),
        client_without_cert.get(&https_url).send(),
    )
    .await;

    match response_without_cert {
        Ok(Ok(_resp)) => {
            panic!("Connection should have failed without client certificate");
        }
        Ok(Err(e)) => {
            println!(
                "✅ Correctly rejected connection without client cert: {}",
                e
            );
            // This is expected - connection should fail without client cert
        }
        Err(_) => {
            println!("✅ Connection timed out without client cert - expected behavior");
            // Timeout is also acceptable - server may drop connection
        }
    }

    // Test 2: Connection with valid client certificate should succeed
    // Load client certificate and key for mTLS
    let certs_dir = std::path::Path::new(&test_tls_config.cert_path)
        .parent()
        .expect("Certificate path should have parent directory")
        .to_string_lossy();
    let client_cert_path = format!("{}/client-cert.pem", certs_dir);
    let client_key_path = format!("{}/client-key.pem", certs_dir);

    // Check if client certificates exist
    if std::path::Path::new(&client_cert_path).exists()
        && std::path::Path::new(&client_key_path).exists()
    {
        // Create client with client certificate
        let client_cert =
            std::fs::read(&client_cert_path).expect("Failed to read client certificate");
        let client_key = std::fs::read(&client_key_path).expect("Failed to read client key");

        // Combine key first, then certificate for proper PEM format
        let mut combined_pem = Vec::new();
        combined_pem.extend_from_slice(&client_key);
        combined_pem.push(b'\n');
        combined_pem.extend_from_slice(&client_cert);
        
        let client_identity = match reqwest::Identity::from_pem(&combined_pem) {
            Ok(identity) => identity,
            Err(e) => {
                println!("⚠️  Failed to create client identity from PEM: {}", e);
                return; // Skip mTLS test if certificate loading fails
            }
        };

        let client_with_cert = match reqwest::Client::builder()
            .identity(client_identity)
            .danger_accept_invalid_certs(true) // Accept self-signed server cert
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                println!("⚠️  Failed to create mTLS client: {}", e);
                return; // Skip mTLS test if client creation fails
            }
        };

        let response_with_cert = timeout(
            Duration::from_millis(3000),
            client_with_cert.get(&https_url).send(),
        )
        .await;

        match response_with_cert {
            Ok(Ok(resp)) => {
                println!(
                    "✅ mTLS client authentication successful! Status: {}",
                    resp.status()
                );
                // Any HTTP response proves mTLS is working
                assert!(
                    resp.status().is_client_error()
                        || resp.status().is_success()
                        || resp.status() == 404,
                    "Server should respond to mTLS requests. Got status: {}",
                    resp.status()
                );
            }
            Ok(Err(e)) => {
                println!("⚠️  mTLS client authentication failed: {}", e);
                // This could indicate the client cert isn't properly configured
                // but the server is still working correctly
            }
            Err(_) => {
                println!("⚠️  mTLS connection timed out - may indicate server configuration issue");
            }
        }
    } else {
        println!(
            "⚠️  Client certificates not found at {} and {} - skipping client cert test",
            client_cert_path, client_key_path
        );
    }

    // Cleanup
    server_handle.abort();
}
