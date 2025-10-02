// ABOUTME: Common test utilities shared across all test modules
// ABOUTME: Provides TLS crypto provider initialization and other shared test functionality

//! Common test utilities for qollective tests.
//!
//! This module provides shared functionality across different test files,
//! including TLS crypto provider initialization and other common setup.

use qollective::client::NatsClient;
use std::sync::Once;

/// NATS connection type for predictable test configurations
#[derive(Debug, Clone)]
pub enum NatsConnectionType {
    /// Plain NATS connection on port 4443 (for clients)
    PlainNats,
    /// TLS with insecure mode on port 4443 (for servers in trusted infrastructure)
    InsecureTls,
    /// TLS with proper certificates on port 4443 (for production)
    SecureTls,
    /// Try multiple configurations to find one that works (current behavior)
    Auto,
}

static CRYPTO_INIT: Once = Once::new();

/// Ensures that a crypto provider is installed for rustls.
///
/// This function is safe to call multiple times across different test files.
/// It uses `std::sync::Once` to ensure the crypto provider is only installed
/// once per test process, avoiding conflicts between test files.
pub fn ensure_crypto_provider() {
    CRYPTO_INIT.call_once(|| {
        #[cfg(feature = "tls")]
        {
            // Try to install default provider, but ignore error if already installed
            match rustls::crypto::aws_lc_rs::default_provider().install_default() {
                Ok(_) => {
                    // Successfully installed
                    eprintln!("Crypto provider installed successfully");
                }
                Err(_) => {
                    // Already installed or other error - this is fine for tests
                    // rustls will return an error if a provider is already installed
                    eprintln!("Crypto provider already installed or installation failed");
                }
            }
        }

        #[cfg(not(feature = "tls"))]
        {
            // No TLS feature enabled, nothing to do
        }
    });
}

/// Setup function that should be called at the beginning of any test
/// that might use TLS functionality.
pub fn setup_test_environment() {
    ensure_crypto_provider();

    // Add other common test setup here if needed
    // For example: tracing initialization, environment variable setup, etc.
}

/// Create test NATS config with predictable connection type
///
/// # Arguments
///
/// * `connection_type` - Optional connection type. If None, uses Auto behavior (try multiple configs)
///
/// # Connection Types
///
/// * `PlainNats` - Plain NATS on port 4443 (for clients)
/// * `InsecureTls` - TLS with insecure mode on port 4443 (for servers in trusted infrastructure)
/// * `SecureTls` - TLS with proper certificates on port 4443 (for production)
/// * `Auto` - Try multiple configurations to find one that works (original behavior)
///
/// # Returns
///
/// Returns the specified configuration, or the first working one for Auto mode.
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub async fn create_test_nats_config(
    connection_type: Option<NatsConnectionType>,
) -> qollective::error::Result<qollective::config::nats::NatsConfig> {
    use qollective::config::nats::NatsConfig;

    // Ensure crypto provider is initialized
    ensure_crypto_provider();

    // Handle connection type selection
    let connection_type = connection_type.unwrap_or(NatsConnectionType::Auto);

    match connection_type {
        NatsConnectionType::PlainNats => {
            // Plain NATS connection on 4443 (for clients)
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4443".to_string()];
            config.connection.tls.enabled = false;
            Ok(config)
        }
        NatsConnectionType::InsecureTls => {
            // TLS with insecure mode on 4443 (for servers in trusted infrastructure)
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4443".to_string()]; // Use nats:// scheme
            config.connection.tls.enabled = true;
            config.connection.tls.verification_mode =
                qollective::config::tls::VerificationMode::Skip;
            Ok(config)
        }
        NatsConnectionType::SecureTls => {
            // TLS with proper certificates on 4443 (for production)
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4443".to_string()];
            config.connection.tls.enabled = true;
            config.connection.tls.ca_cert_path =
                Some("/Users/ms/development/docker/nats/certs/server/ca.pem".into());
            config.connection.tls.cert_path =
                Some("/Users/ms/development/docker/nats/certs/server/client-cert.pem".into());
            config.connection.tls.key_path =
                Some("/Users/ms/development/docker/nats/certs/server/client-key.pem".into());
            config.connection.tls.verification_mode =
                qollective::config::tls::VerificationMode::MutualTls;
            Ok(config)
        }
        NatsConnectionType::Auto => {
            // Try different NATS configurations to find one that works (original behavior)
            try_multiple_nats_configs().await
        }
    }
}

/// Try multiple NATS configurations to find one that works (legacy behavior)
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
async fn try_multiple_nats_configs(
) -> qollective::error::Result<qollective::config::nats::NatsConfig> {
    use qollective::config::nats::NatsConfig;

    let configs_to_try = vec![
        // Option 1: Plain NATS connection on 4443 (preferred for trusted infrastructure)
        {
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4443".to_string()];
            config.connection.tls.enabled = false;
            config
        },
        // Option 2: TLS with insecure mode for localhost:4443 (trusted infrastructure)
        {
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4443".to_string()];
            config.connection.tls.enabled = true;
            config.connection.tls.verification_mode =
                qollective::config::tls::VerificationMode::Skip; // Skip certificate verification for trusted infrastructure
            config
        },
        // Option 3: TLS with proper certificates for localhost:4443 (mTLS required)
        {
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4443".to_string()];
            config.connection.tls.enabled = true;
            config.connection.tls.ca_cert_path =
                Some("/Users/ms/development/docker/nats/certs/server/ca.pem".into());
            config.connection.tls.cert_path =
                Some("/Users/ms/development/docker/nats/certs/server/client-cert.pem".into());
            config.connection.tls.key_path =
                Some("/Users/ms/development/docker/nats/certs/server/client-key.pem".into());
            config.connection.tls.verification_mode =
                qollective::config::tls::VerificationMode::MutualTls;
            config
        },
        // Option 5: Plain NATS connection on 4222 (final fallback)
        {
            let mut config = NatsConfig::default();
            config.connection.urls = vec!["nats://localhost:4222".to_string()];
            config.connection.tls.enabled = false;
            config
        },
    ];

    for config in &configs_to_try {
        match NatsClient::new(config.clone()).await {
            Ok(_) => return Ok(config.clone()),
            Err(_) => continue,
        }
    }

    // If all configurations fail, return the first one for the error message
    Ok(configs_to_try.into_iter().next().unwrap())
}

/// Create a test MCP server registry configuration with NATS
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub async fn create_test_mcp_registry_config(
    connection_type: Option<NatsConnectionType>,
) -> qollective::error::Result<qollective::config::mcp::McpTransportClientConfig> {
    // Use the correct MCP config type from config module
    use qollective::config::mcp::McpTransportClientConfig;
    use std::time::Duration;

    let mut config = McpTransportClientConfig::default();
    config.discovery_timeout = Duration::from_secs(10);
    Ok(config)
}

/// Test helper to skip tests gracefully when NATS is not available
pub fn skip_test_if_nats_unavailable<T>(
    result: qollective::error::Result<T>,
    test_name: &str,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(e) => {
            println!(
                "⚠️  Skipping {} due to NATS connection issue: {}",
                test_name, e
            );
            None
        }
    }
}

/// Assert that an error contains expected keywords for MCP operations
pub fn assert_expected_mcp_error(
    result: &qollective::error::Result<impl std::fmt::Debug>,
    expected_keywords: &[&str],
) {
    match result {
        Ok(_) => println!("✅ Operation executed successfully"),
        Err(e) => {
            let error_str = e.to_string().to_lowercase();
            let has_expected_keyword = expected_keywords
                .iter()
                .any(|keyword| error_str.contains(&keyword.to_lowercase()));

            if has_expected_keyword {
                println!("✅ Operation handled correctly with expected error: {}", e);
            } else {
                panic!(
                    "Unexpected error (should contain one of {:?}): {}",
                    expected_keywords, e
                );
            }
        }
    }
}

/// Create a raw NATS echo responder for testing NativeNats transport
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub async fn setup_raw_nats_echo_responder(
    connection_type: Option<NatsConnectionType>,
    subject: &str,
) -> qollective::error::Result<tokio::task::JoinHandle<()>> {
    use serde_json;

    // Create raw NATS client directly (bypassing Qollective wrapper for raw access)
    let nats_config = create_test_nats_config(connection_type).await?;

    // Create a second NATS client using the same configuration for the responder
    let responder_nats_client = qollective::client::nats::NatsClient::new(nats_config.clone())
        .await
        .map_err(|e| {
            qollective::error::QollectiveError::transport(format!(
                "Failed to create responder NATS client: {}",
                e
            ))
        })?;

    let subject = subject.to_string();

    // Spawn the echo responder task
    let responder_handle = tokio::spawn(async move {
        println!(
            "🔧 Starting RAW NATS echo responder on subject: {}",
            subject
        );

        // Subscribe to the subject using Qollective NATS client
        let mut subscriber = match responder_nats_client.subscribe(&subject, None).await {
            Ok(sub) => {
                println!("✅ Successfully subscribed to subject: {}", subject);
                sub
            }
            Err(e) => {
                println!("❌ Failed to subscribe to subject {}: {}", subject, e);
                return;
            }
        };

        // Handle incoming messages
        loop {
            use futures_util::StreamExt;

            if let Some(message) = subscriber.next().await {
                println!(
                    "📨 Received message on {}: {} bytes",
                    subject,
                    message.payload.len()
                );

                // Only reply if there's a reply subject
                if let Some(reply_subject) = &message.reply {
                    // Parse the incoming request to extract the message
                    let request_text = match std::str::from_utf8(&message.payload) {
                        Ok(text) => {
                            println!("📄 Request payload: {}", text);
                            text
                        }
                        Err(_) => "binary data",
                    };

                    // Create echo response as raw JSON (no envelope for raw NATS)
                    let response = serde_json::json!({
                        "result": format!("echo: {}", request_text),
                        "status": 200
                    });

                    let response_bytes = response.to_string().into_bytes();

                    // Send reply using raw NATS publish
                    if let Err(e) = responder_nats_client
                        .publish_raw(reply_subject, &response_bytes)
                        .await
                    {
                        println!("❌ Failed to send reply: {}", e);
                    } else {
                        println!("✅ Sent echo reply: {}", response);
                    }
                } else {
                    println!("⚠️  Message has no reply subject - cannot echo");
                }
            } else {
                break;
            }
        }

        println!("🔧 NATS echo responder stopped");
    });

    // Give the responder a moment to start and subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(responder_handle)
}

/// Create a Qollective envelope echo responder for testing QollectiveNats transport
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub async fn setup_envelope_nats_echo_responder(
    connection_type: Option<NatsConnectionType>,
    subject: &str,
) -> qollective::error::Result<tokio::task::JoinHandle<()>> {
    use qollective::envelope::{Envelope, Meta, NatsEnvelopeCodec};
    use serde_json;

    // Create NATS client using Qollective configuration for envelope handling
    let nats_config = create_test_nats_config(connection_type).await?;

    // Create a NATS client for the envelope responder
    let responder_nats_client = qollective::client::nats::NatsClient::new(nats_config.clone())
        .await
        .map_err(|e| {
            qollective::error::QollectiveError::transport(format!(
                "Failed to create envelope responder NATS client: {}",
                e
            ))
        })?;

    let subject = subject.to_string();

    // Spawn the envelope echo responder task
    let responder_handle = tokio::spawn(async move {
        println!(
            "🔧 Starting ENVELOPE NATS echo responder on subject: {}",
            subject
        );

        // Subscribe to the subject using Qollective NATS client
        let mut subscriber = match responder_nats_client.subscribe(&subject, None).await {
            Ok(sub) => {
                println!(
                    "✅ Successfully subscribed to envelope subject: {}",
                    subject
                );
                sub
            }
            Err(e) => {
                println!(
                    "❌ Failed to subscribe to envelope subject {}: {}",
                    subject, e
                );
                return;
            }
        };

        // Handle incoming envelope messages
        loop {
            use futures_util::StreamExt;

            if let Some(message) = subscriber.next().await {
                println!(
                    "📨 Received envelope message on {}: {} bytes",
                    subject,
                    message.payload.len()
                );

                // Only reply if there's a reply subject
                if let Some(reply_subject) = &message.reply {
                    // Decode the incoming envelope
                    let request_envelope: Result<Envelope<serde_json::Value>, _> =
                        NatsEnvelopeCodec::decode(&message.payload);

                    match request_envelope {
                        Ok(envelope) => {
                            println!("📄 Decoded envelope payload: {:?}", envelope.payload);

                            // Create echo response envelope
                            let response_data = serde_json::json!({
                                "result": format!("envelope echo: {:?}", envelope.payload),
                                "status": 200
                            });

                            // Create response envelope with metadata
                            let mut response_meta = Meta::default();
                            response_meta.timestamp = Some(chrono::Utc::now());
                            response_meta.request_id = envelope.meta.request_id; // Preserve request ID
                            response_meta.tenant = envelope.meta.tenant; // Preserve tenant

                            let response_envelope = Envelope::new(response_meta, response_data);

                            // Encode response envelope
                            match NatsEnvelopeCodec::encode(&response_envelope) {
                                Ok(encoded_response) => {
                                    // Send reply using raw NATS publish (since we have encoded envelope bytes)
                                    if let Err(e) = responder_nats_client
                                        .publish_raw(reply_subject, &encoded_response)
                                        .await
                                    {
                                        println!("❌ Failed to send envelope reply: {}", e);
                                    } else {
                                        println!("✅ Sent envelope echo reply with metadata");
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Failed to encode response envelope: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to decode request envelope: {}", e);
                            // Send error response
                            let error_response = serde_json::json!({
                                "error": "Failed to decode envelope",
                                "status": 400
                            });
                            let error_bytes = error_response.to_string().into_bytes();

                            if let Err(e) = responder_nats_client
                                .publish_raw(reply_subject, &error_bytes)
                                .await
                            {
                                println!("❌ Failed to send error reply: {}", e);
                            }
                        }
                    }
                } else {
                    println!("⚠️  Envelope message has no reply subject - cannot echo");
                }
            } else {
                break;
            }
        }

        println!("🔧 Envelope NATS echo responder stopped");
    });

    // Give the responder a moment to start and subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(responder_handle)
}

/// Create test agent info with standard metadata
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub fn create_test_agent(name: &str, capabilities: Vec<&str>) -> qollective::types::a2a::AgentInfo {
    use qollective::types::a2a::{AgentInfo, HealthStatus};
    use std::{collections::HashMap, time::SystemTime};
    use uuid::Uuid;

    AgentInfo {
        id: Uuid::now_v7(),
        name: name.to_string(),
        capabilities: capabilities.into_iter().map(String::from).collect(),
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::new(),
    }
}

/// Create test envelope with tenant metadata
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub fn create_test_envelope<T>(data: T, tenant: &str) -> qollective::envelope::Envelope<T> {
    use qollective::envelope::{Envelope, Meta};
    use uuid::Uuid;

    let mut meta = Meta::default();
    meta.tenant = Some(tenant.to_string());
    meta.request_id = Some(Uuid::now_v7());
    Envelope::new(meta, data)
}

/// Setup NATS server with discovery service for integration tests
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub async fn setup_nats_server_with_discovery(
    connection_type: Option<NatsConnectionType>,
    enable_health_monitoring: bool,
) -> qollective::error::Result<(
    qollective::server::nats::NatsServer,
    std::sync::Arc<qollective::client::a2a::AgentRegistry>,
)> {
    use qollective::{
        client::a2a::AgentRegistry, config::a2a::RegistryConfig, server::nats::NatsServer,
    };
    use std::{sync::Arc, time::Duration};

    // Initialize TLS crypto provider
    ensure_crypto_provider();

    // Create registry with configuration
    let config = RegistryConfig {
        agent_ttl: Duration::from_secs(60),
        cleanup_interval: Duration::from_secs(10),
        max_agents: 100,
        enable_health_monitoring,
        enable_agent_logging: false,
        agent_log_subject: "test.logs".to_string(),
        logging_agent_capability: "logging".to_string(),
        enable_capability_indexing: true,
        max_capabilities_per_agent: 50,
    };

    // Create NATS server
    let nats_config = create_test_nats_config(connection_type).await?;
    let registry = Arc::new(AgentRegistry::new(config, nats_config.clone()).await?);
    let mut server = NatsServer::new(nats_config).await?;

    // Enable discovery service
    server.enable_discovery(Arc::clone(&registry)).await?;

    Ok((server, registry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_provider_setup() {
        // This should not panic
        ensure_crypto_provider();

        // Calling it again should also not panic
        ensure_crypto_provider();
    }

    #[test]
    fn test_setup_test_environment() {
        // This should not panic
        setup_test_environment();

        // Calling it again should also not panic
        setup_test_environment();
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_create_test_nats_config() {
        // Test that the config creation doesn't panic (auto mode)
        let config = create_test_nats_config(None).await;
        assert!(config.is_ok());

        let nats_config = config.unwrap();
        assert!(!nats_config.connection.urls.is_empty());

        // Test specific connection types
        let plain_config = create_test_nats_config(Some(NatsConnectionType::PlainNats)).await;
        assert!(plain_config.is_ok());
        assert!(!plain_config.unwrap().connection.tls.enabled);

        let insecure_config = create_test_nats_config(Some(NatsConnectionType::InsecureTls)).await;
        assert!(insecure_config.is_ok());
        let insecure = insecure_config.unwrap();
        assert!(insecure.connection.tls.enabled);
        assert_eq!(
            insecure.connection.tls.verification_mode,
            qollective::config::tls::VerificationMode::Skip
        );
    }

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[tokio::test]
    async fn test_create_test_mcp_registry_config() {
        // Test that the MCP registry config creation works
        let config = create_test_mcp_registry_config(None).await;
        assert!(config.is_ok());

        let registry_config = config.unwrap();
        assert_eq!(
            registry_config.discovery_timeout,
            std::time::Duration::from_secs(10)
        );
        assert!(registry_config.enable_connection_pooling);
    }

    #[test]
    fn test_skip_test_if_nats_unavailable() {
        use qollective::error::QollectiveError;

        // Test successful case
        let success_result: qollective::error::Result<String> = Ok("success".to_string());
        let value = skip_test_if_nats_unavailable(success_result, "test");
        assert_eq!(value, Some("success".to_string()));

        // Test error case - use feature_not_enabled since it's always available
        let error_result: qollective::error::Result<String> =
            Err(QollectiveError::feature_not_enabled("connection failed"));
        let value = skip_test_if_nats_unavailable(error_result, "test");
        assert_eq!(value, None);
    }

    #[test]
    fn test_assert_expected_mcp_error() {
        use qollective::error::QollectiveError;

        // Test expected error
        let error_result: qollective::error::Result<()> =
            Err(QollectiveError::feature_not_enabled("server not found"));
        assert_expected_mcp_error(&error_result, &["server", "not found"]);

        // Test success case
        let success_result: qollective::error::Result<()> = Ok(());
        assert_expected_mcp_error(&success_result, &["any"]);
    }
}

/// Get a random available port for testing
///
/// This function finds an available port by binding to port 0, which tells the OS
/// to assign any available port, then returns that port number for use in tests.
/// This prevents test conflicts when multiple tests run concurrently.
pub fn get_available_port() -> u16 {
    use std::net::TcpListener;

    // Bind to port 0 to get any available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to an available port");

    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();

    // Drop the listener to free the port for actual use
    drop(listener);

    port
}

// Re-export REST test utilities
pub mod rest_test_utils;
pub use rest_test_utils::*;

// Re-export WebSocket test utilities
pub mod websocket_test_utils;
pub use websocket_test_utils::*;

// Re-export test constants
pub mod test_constants;
pub use test_constants::*;
