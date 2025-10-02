// ABOUTME: Common utilities for REST server/client integration tests with TLS support
// ABOUTME: Provides TLS configuration, server setup, and roundtrip testing functions

//! Common utilities for REST server/client integration tests.
//!
//! This module provides shared functionality for testing REST communication
//! including TLS configuration, server setup, and roundtrip testing patterns.

use super::{get_available_port, setup_test_environment};
use async_trait::async_trait;
use qollective::client::common::ClientConfig;
use qollective::client::rest::{RestClient, RestClientConfig};
use qollective::config::tls::TlsConfig;
use qollective::envelope::{Context, Envelope, Meta};
use qollective::error::Result;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::server::common::ServerConfig;
use qollective::server::rest::{
    MetadataEncoding, MetadataHandlingConfig, RestServer, RestServerConfig,
};
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

/// Get test certificates directory with environment variable override
///
/// Supports the following environment variable:
/// - `QOLLECTIVE_TEST_CERTS_DIR`: Override the default certificate directory
///
/// Falls back to checking common certificate locations if environment variable is not set.
pub fn get_test_certs_dir() -> String {
    std::env::var("QOLLECTIVE_TEST_CERTS_DIR").unwrap_or_else(|_| {
        // Try common certificate locations
        let possible_dirs = [
            "./tests/certs",              // Test certs directory (primary)
            "./certs",                    // Local certs directory
            "/tmp/qollective-test-certs", // Temporary directory
            "/etc/ssl/qollective",        // System directory
        ];

        for dir in &possible_dirs {
            if std::path::Path::new(dir).exists() {
                return dir.to_string();
            }
        }

        // Default to codebase certs directory
        "./tests/certs".to_string()
    })
}

/// TLS configuration for testing
pub struct TestTlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: String,
}

impl Default for TestTlsConfig {
    fn default() -> Self {
        let certs_dir = get_test_certs_dir();
        Self {
            cert_path: format!("{}/server-cert.pem", certs_dir),
            key_path: format!("{}/server-key.pem", certs_dir),
            ca_path: format!("{}/ca.pem", certs_dir),
        }
    }
}

impl TestTlsConfig {
    /// Check if TLS certificates exist
    pub fn certs_available(&self) -> bool {
        std::path::Path::new(&self.cert_path).exists()
            && std::path::Path::new(&self.key_path).exists()
            && std::path::Path::new(&self.ca_path).exists()
    }

    /// Create TLS configuration for server
    pub fn to_server_tls_config(&self) -> TlsConfig {
        TlsConfig {
            enabled: true,
            cert_path: Some(std::path::PathBuf::from(&self.cert_path)),
            key_path: Some(std::path::PathBuf::from(&self.key_path)),
            ca_cert_path: Some(std::path::PathBuf::from(&self.ca_path)),
            verification_mode: qollective::config::tls::VerificationMode::SystemCa,
        }
    }
}

/// Generic test handler for envelope processing
pub struct TestEnvelopeHandler {
    pub name: String,
}

impl TestEnvelopeHandler {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl ContextDataHandler<Value, Value> for TestEnvelopeHandler {
    async fn handle(&self, context: Option<Context>, data: Value) -> Result<Value> {
        // Extract context information
        let has_context = context.is_some();
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
            "processed_at": chrono::Utc::now()
        }))
    }
}

/// Configuration for roundtrip tests
#[derive(Debug, Clone)]
pub struct RoundtripTestConfig {
    pub use_tls: bool,
    pub port: u16,
    pub endpoint: String,
    pub handler_name: String,
}

impl Default for RoundtripTestConfig {
    fn default() -> Self {
        Self {
            use_tls: false,
            port: get_available_port(),
            endpoint: "/test".to_string(),
            handler_name: "test-handler".to_string(),
        }
    }
}

/// Setup REST server for testing
pub async fn setup_test_rest_server(
    config: RoundtripTestConfig,
) -> Result<tokio::task::JoinHandle<()>> {
    setup_test_environment();

    let tls_config = TestTlsConfig::default();

    // Create server configuration
    let server_config = RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: config.port,
            ..Default::default()
        },
        tls: if config.use_tls && tls_config.certs_available() {
            Some(tls_config.to_server_tls_config())
        } else {
            None
        },
        metadata: MetadataHandlingConfig {
            max_header_size: 4096,
            max_total_headers: 16384,
            encoding: MetadataEncoding::Base64,
        },
        ..Default::default()
    };

    let mut server = RestServer::new(server_config).await?;

    // Register test handler
    let handler = TestEnvelopeHandler::new(&config.handler_name);
    server
        .receive_envelope_at(&config.endpoint, handler)
        .await?;

    // Configure OPTIONS behavior for endpoints that need it
    if config.endpoint.contains("options") || config.handler_name.contains("options") {
        server
            .set_options_behavior(
                &config.endpoint,
                qollective::server::rest::OptionsBehavior::Application,
            )
            .await?;
    }

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("🔍 Server should be running on 127.0.0.1:{}", config.port);

    // Try to verify server is responsive with simple health check
    let health_url = format!("http://127.0.0.1:{}/health", config.port);
    match reqwest::get(&health_url).await {
        Ok(response) => println!("✅ Health check successful: {}", response.status()),
        Err(e) => println!("❌ Health check failed: {}", e),
    }

    Ok(server_handle)
}

/// Create REST client for testing
pub async fn create_test_rest_client(config: &RoundtripTestConfig) -> Result<RestClient> {
    let scheme = if config.use_tls { "https" } else { "http" };
    let base_url = format!("{}://127.0.0.1:{}", scheme, config.port);

    let client_config = RestClientConfig {
        base: ClientConfig {
            base_url,
            timeout_seconds: 10,
            retry_attempts: 1,
            ..Default::default()
        },
        ..Default::default()
    };

    RestClient::new(client_config).await
}

/// Create test envelope with metadata
pub fn create_test_envelope(message: &str, method: &str) -> Envelope<Value> {
    let mut meta = Meta::default();
    meta.request_id = Some(Uuid::now_v7());
    meta.tenant = Some(format!("{}-tenant", method));
    meta.version = Some("1.0".to_string());
    meta.timestamp = Some(chrono::Utc::now());

    let payload = json!({
        "message": message,
        "method": method,
        "test_id": Uuid::now_v7(),
        "timestamp": chrono::Utc::now()
    });

    Envelope::new(meta, payload)
}

/// Verify envelope roundtrip response
pub fn verify_roundtrip_response(
    request_envelope: &Envelope<Value>,
    response_envelope: &Envelope<Value>,
    expected_method: &str,
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

    // Verify echo data
    let echo_data = &response_envelope.payload["echo"];
    assert_eq!(echo_data["method"], expected_method);
    assert_eq!(echo_data["message"], request_envelope.payload["message"]);

    // Verify context was passed
    assert_eq!(response_envelope.payload["context"]["has_context"], true);
    assert!(response_envelope.payload["processed_at"].is_string());
}

/// Run a simple GET roundtrip test
pub async fn run_get_roundtrip_test(use_tls: bool) -> Result<()> {
    let config = RoundtripTestConfig {
        use_tls,
        endpoint: "/get".to_string(),
        handler_name: "get-handler".to_string(),
        ..Default::default()
    };

    // Check TLS availability
    if use_tls {
        let tls_config = TestTlsConfig::default();
        if !tls_config.certs_available() {
            println!(
                "⚠️  Skipping TLS test - certificates not available at {}",
                get_test_certs_dir()
            );
            return Ok(());
        }
    }

    // Setup server
    let server_handle = setup_test_rest_server(config.clone()).await?;

    // Create client
    let client = create_test_rest_client(&config).await?;

    // Create test envelope for GET (typically minimal body)
    let request_envelope = create_test_envelope("test GET roundtrip", "get");

    // Execute GET request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(5),
        client.get(&config.endpoint, request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_roundtrip_response(
        &request_envelope,
        &response_envelope,
        "get",
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Run a simple POST roundtrip test
pub async fn run_post_roundtrip_test(use_tls: bool) -> Result<()> {
    let config = RoundtripTestConfig {
        use_tls,
        endpoint: "/post".to_string(),
        handler_name: "post-handler".to_string(),
        ..Default::default()
    };

    // Check TLS availability
    if use_tls {
        let tls_config = TestTlsConfig::default();
        if !tls_config.certs_available() {
            println!(
                "⚠️  Skipping TLS test - certificates not available at {}",
                get_test_certs_dir()
            );
            return Ok(());
        }
    }

    // Setup server
    let server_handle = setup_test_rest_server(config.clone()).await?;

    // Create client
    let client = create_test_rest_client(&config).await?;

    // Create test envelope for POST
    let request_envelope = create_test_envelope("test POST roundtrip", "post");

    // Execute POST request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(5),
        client.post(&config.endpoint, request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_roundtrip_response(
        &request_envelope,
        &response_envelope,
        "post",
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Run a simple PUT roundtrip test
pub async fn run_put_roundtrip_test(use_tls: bool) -> Result<()> {
    let config = RoundtripTestConfig {
        use_tls,
        endpoint: "/put".to_string(),
        handler_name: "put-handler".to_string(),
        ..Default::default()
    };

    // Check TLS availability
    if use_tls {
        let tls_config = TestTlsConfig::default();
        if !tls_config.certs_available() {
            println!(
                "⚠️  Skipping TLS test - certificates not available at {}",
                get_test_certs_dir()
            );
            return Ok(());
        }
    }

    // Setup server
    let server_handle = setup_test_rest_server(config.clone()).await?;

    // Create client
    let client = create_test_rest_client(&config).await?;

    // Create test envelope for PUT
    let request_envelope = create_test_envelope("test PUT roundtrip", "put");

    // Execute PUT request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(5),
        client.put(&config.endpoint, request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_roundtrip_response(
        &request_envelope,
        &response_envelope,
        "put",
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Run a simple DELETE roundtrip test
pub async fn run_delete_roundtrip_test(use_tls: bool) -> Result<()> {
    let config = RoundtripTestConfig {
        use_tls,
        endpoint: "/delete".to_string(),
        handler_name: "delete-handler".to_string(),
        ..Default::default()
    };

    // Check TLS availability
    if use_tls {
        let tls_config = TestTlsConfig::default();
        if !tls_config.certs_available() {
            println!(
                "⚠️  Skipping TLS test - certificates not available at {}",
                get_test_certs_dir()
            );
            return Ok(());
        }
    }

    // Setup server
    let server_handle = setup_test_rest_server(config.clone()).await?;

    // Create client
    let client = create_test_rest_client(&config).await?;

    // Create test envelope for DELETE
    let request_envelope = create_test_envelope("test DELETE roundtrip", "delete");

    // Execute DELETE request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(5),
        client.delete(&config.endpoint, request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_roundtrip_response(
        &request_envelope,
        &response_envelope,
        "delete",
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Run a simple OPTIONS roundtrip test
pub async fn run_options_roundtrip_test(use_tls: bool) -> Result<()> {
    let config = RoundtripTestConfig {
        use_tls,
        endpoint: "/options".to_string(),
        handler_name: "options-handler".to_string(),
        ..Default::default()
    };

    // Check TLS availability
    if use_tls {
        let tls_config = TestTlsConfig::default();
        if !tls_config.certs_available() {
            println!(
                "⚠️  Skipping TLS test - certificates not available at {}",
                get_test_certs_dir()
            );
            return Ok(());
        }
    }

    // Setup server
    let server_handle = setup_test_rest_server(config.clone()).await?;

    // Create client
    let client = create_test_rest_client(&config).await?;

    // Create test envelope for OPTIONS
    let request_envelope = create_test_envelope("test OPTIONS roundtrip", "options");

    // Execute OPTIONS request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(5),
        client.options(&config.endpoint, request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_roundtrip_response(
        &request_envelope,
        &response_envelope,
        "options",
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

/// Run a simple PATCH roundtrip test
pub async fn run_patch_roundtrip_test(use_tls: bool) -> Result<()> {
    let config = RoundtripTestConfig {
        use_tls,
        endpoint: "/patch".to_string(),
        handler_name: "patch-handler".to_string(),
        ..Default::default()
    };

    // Check TLS availability
    if use_tls {
        let tls_config = TestTlsConfig::default();
        if !tls_config.certs_available() {
            println!(
                "⚠️  Skipping TLS test - certificates not available at {}",
                get_test_certs_dir()
            );
            return Ok(());
        }
    }

    // Setup server
    let server_handle = setup_test_rest_server(config.clone()).await?;

    // Create client
    let client = create_test_rest_client(&config).await?;

    // Create test envelope for PATCH
    let request_envelope = create_test_envelope("test PATCH roundtrip", "patch");

    // Execute PATCH request
    let response_envelope: Envelope<Value> = timeout(
        Duration::from_secs(5),
        client.patch(&config.endpoint, request_envelope.clone()),
    )
    .await
    .map_err(|_| qollective::error::QollectiveError::transport("Request timed out"))??;

    // Verify response
    verify_roundtrip_response(
        &request_envelope,
        &response_envelope,
        "patch",
        &config.handler_name,
    );

    // Cleanup
    server_handle.abort();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_creation() {
        let tls_config = TestTlsConfig::default();
        assert!(!tls_config.cert_path.is_empty());
        assert!(!tls_config.key_path.is_empty());
        assert!(!tls_config.ca_path.is_empty());

        let server_tls = tls_config.to_server_tls_config();
        assert_eq!(
            server_tls.cert_path,
            Some(std::path::PathBuf::from(&tls_config.cert_path))
        );
        assert_eq!(
            server_tls.key_path,
            Some(std::path::PathBuf::from(&tls_config.key_path))
        );
        assert_eq!(
            server_tls.verification_mode,
            qollective::config::tls::VerificationMode::SystemCa
        );
    }

    #[test]
    fn test_roundtrip_config_creation() {
        let config = RoundtripTestConfig::default();
        assert!(!config.use_tls);
        assert!(config.port > 0);
        assert_eq!(config.endpoint, "/test");
        assert_eq!(config.handler_name, "test-handler");
    }

    #[tokio::test]
    async fn test_envelope_handler() {
        let handler = TestEnvelopeHandler::new("test");
        let context = Some(Context::empty());
        let data = json!({"test": "data"});

        let result = handler.handle(context, data).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["handler"], "test");
        assert_eq!(response["status"], "success");
        assert_eq!(response["echo"]["test"], "data");
        assert_eq!(response["context"]["has_context"], true);
    }

    #[test]
    fn test_create_test_envelope() {
        let envelope = create_test_envelope("test message", "GET");

        assert!(envelope.meta.request_id.is_some());
        assert_eq!(envelope.meta.tenant, Some("GET-tenant".to_string()));
        assert_eq!(envelope.meta.version, Some("1.0".to_string()));
        assert!(envelope.meta.timestamp.is_some());

        assert_eq!(envelope.payload["message"], "test message");
        assert_eq!(envelope.payload["method"], "GET");
        assert!(envelope.payload["test_id"].is_string());
    }
}
