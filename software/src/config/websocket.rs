// ABOUTME: WebSocket-specific configuration structures for real-time bidirectional communication
// ABOUTME: Provides connection settings, message timeouts, and protocol negotiation config

//! WebSocket configuration types for the Qollective framework.
//!
//! This module provides comprehensive configuration structures for WebSocket messaging,
//! including connection settings, timeout configuration, and protocol negotiation
//! parameters following the existing Qollective configuration patterns.

use crate::constants::{limits, timeouts};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// WebSocket-specific configuration for transport layer
///
/// This configuration provides settings for WebSocket connections within
/// the Qollective framework, supporting real-time bidirectional communication
/// with proper timeout handling and protocol negotiation.
///
/// # Examples
///
/// ```rust
/// use qollective::config::websocket::WebSocketConfig;
///
/// // Create default configuration
/// let config = WebSocketConfig::default();
/// assert_eq!(config.ping_interval_ms, 30000);
/// assert_eq!(config.max_message_size, 16 * 1024 * 1024);
///
/// // Create custom configuration
/// let config = WebSocketConfig {
///     ping_interval_ms: 15000,
///     max_message_size: 8 * 1024 * 1024,
///     enable_compression: false,
///     subprotocols: vec!["custom-protocol".to_string()],
/// };
/// ```
#[cfg(feature = "websocket-client")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// WebSocket ping interval in milliseconds for keep-alive
    pub ping_interval_ms: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Whether to enable compression
    pub enable_compression: bool,
    /// Subprotocols to negotiate during handshake
    pub subprotocols: Vec<String>,
}

#[cfg(feature = "websocket-client")]
impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            ping_interval_ms: timeouts::DEFAULT_WEBSOCKET_PING_INTERVAL_MS,
            max_message_size: limits::DEFAULT_WEBSOCKET_MESSAGE_SIZE,
            enable_compression: true,
            subprotocols: vec!["qollective".to_string()],
        }
    }
}

/// WebSocket client-specific configuration
///
/// Comprehensive configuration for WebSocket client behavior, including connection
/// retry settings, protocol parameters, and client identification. Merged from
/// both framework config and client implementation config for centralized management.
#[cfg(feature = "websocket-client")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketClientConfig {
    /// Base client configuration for general client settings
    pub base: crate::client::common::ClientConfig,
    /// Base WebSocket configuration for protocol settings
    pub websocket: WebSocketConfig,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Message timeout in milliseconds
    pub message_timeout_ms: u64,
    /// Ping timeout in milliseconds
    pub ping_timeout_ms: u64,
    /// Maximum frame size in bytes
    pub max_frame_size: usize,
    /// Maximum number of connection retry attempts
    pub max_retry_attempts: u32,
    /// Client user agent string
    pub user_agent: String,
    /// Custom connection headers
    pub connection_headers: HashMap<String, String>,
    /// TLS configuration for secure connections
    pub tls: crate::config::tls::TlsConfig,
}

#[cfg(feature = "websocket-client")]
impl Default for WebSocketClientConfig {
    fn default() -> Self {
        Self {
            base: crate::client::common::ClientConfig::default(),
            websocket: WebSocketConfig::default(),
            connection_timeout_ms: timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS,
            message_timeout_ms: timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS,
            ping_timeout_ms: 10000,           // 10 seconds
            max_frame_size: 16 * 1024 * 1024, // 16MB
            max_retry_attempts: 3,
            user_agent: format!("qollective-rust/{}", env!("CARGO_PKG_VERSION")),
            connection_headers: HashMap::new(),
            tls: crate::config::tls::TlsConfig::default(),
        }
    }
}

#[cfg(feature = "websocket-client")]
impl WebSocketClientConfig {
    /// Get ping interval as Duration for compatibility with client code
    pub fn ping_interval(&self) -> Duration {
        Duration::from_millis(self.websocket.ping_interval_ms)
    }

    /// Get ping timeout as Duration for compatibility with client code
    pub fn ping_timeout(&self) -> Duration {
        Duration::from_millis(self.ping_timeout_ms)
    }

    /// Get max message size for compatibility
    pub fn max_message_size(&self) -> usize {
        self.websocket.max_message_size
    }

    /// Get compression enabled setting for compatibility
    pub fn compression_enabled(&self) -> bool {
        self.websocket.enable_compression
    }

    /// Get subprotocols for compatibility
    pub fn subprotocols(&self) -> &Vec<String> {
        &self.websocket.subprotocols
    }

    /// Create a builder for WebSocket client configuration
    pub fn builder() -> WebSocketClientConfigBuilder {
        WebSocketClientConfigBuilder::new()
    }
}

/// WebSocket server-specific configuration
///
/// Configuration for WebSocket server instances,
/// including binding settings and connection limits.
#[cfg(feature = "websocket-server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketServerConfig {
    /// Base WebSocket configuration
    pub base: WebSocketConfig,
    /// Server bind address
    pub bind_address: String,
    /// Server port
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// TLS configuration for secure connections
    pub tls: crate::config::tls::TlsConfig,
}

#[cfg(feature = "websocket-server")]
impl Default for WebSocketServerConfig {
    fn default() -> Self {
        Self {
            base: WebSocketConfig::default(),
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: limits::DEFAULT_MAX_WEBSOCKET_CONNECTIONS,
            tls: crate::config::tls::TlsConfig::default(),
        }
    }
}

#[cfg(feature = "websocket-server")]
impl WebSocketServerConfig {
    /// Create a builder for WebSocket server configuration
    pub fn builder() -> WebSocketServerConfigBuilder {
        WebSocketServerConfigBuilder::new()
    }
}

/// WebSocket client configuration builder
#[cfg(feature = "websocket-client")]
pub struct WebSocketClientConfigBuilder {
    config: WebSocketClientConfig,
}

#[cfg(feature = "websocket-client")]
impl WebSocketClientConfigBuilder {
    /// Create a new WebSocket client configuration builder
    pub fn new() -> Self {
        Self {
            config: WebSocketClientConfig::default(),
        }
    }

    /// Set the connection timeout
    pub fn with_connection_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.connection_timeout_ms = timeout_ms;
        self
    }

    /// Set the message timeout
    pub fn with_message_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.message_timeout_ms = timeout_ms;
        self
    }

    /// Set the ping timeout
    pub fn with_ping_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.ping_timeout_ms = timeout_ms;
        self
    }

    /// Set the max frame size
    pub fn with_max_frame_size(mut self, size: usize) -> Self {
        self.config.max_frame_size = size;
        self
    }

    /// Set the max retry attempts
    pub fn with_max_retry_attempts(mut self, attempts: u32) -> Self {
        self.config.max_retry_attempts = attempts;
        self
    }

    /// Set the user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Add a connection header
    pub fn with_connection_header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.config
            .connection_headers
            .insert(key.into(), value.into());
        self
    }

    /// Set multiple connection headers
    pub fn with_connection_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.config.connection_headers = headers;
        self
    }

    /// Enable TLS with system CA verification
    pub fn with_tls_system_ca(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::SystemCa;
        self
    }

    /// Enable TLS with custom CA certificate
    pub fn with_tls_custom_ca(mut self, ca_cert_path: impl Into<std::path::PathBuf>) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::CustomCa;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self
    }

    /// Enable TLS with verification skipped (insecure)
    pub fn with_tls_skip_verify(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::Skip;
        self
    }

    /// Enable mutual TLS with client certificate and key
    pub fn with_mutual_tls(
        mut self,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Enable mutual TLS with custom CA, client certificate and key
    pub fn with_mutual_tls_with_ca(
        mut self,
        ca_cert_path: impl Into<std::path::PathBuf>,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> WebSocketClientConfig {
        self.config
    }
}

/// WebSocket server configuration builder
#[cfg(feature = "websocket-server")]
pub struct WebSocketServerConfigBuilder {
    config: WebSocketServerConfig,
}

#[cfg(feature = "websocket-server")]
impl WebSocketServerConfigBuilder {
    /// Create a new WebSocket server configuration builder
    pub fn new() -> Self {
        Self {
            config: WebSocketServerConfig::default(),
        }
    }

    /// Set the bind address
    pub fn with_bind_address(mut self, address: impl Into<String>) -> Self {
        self.config.bind_address = address.into();
        self
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Set the maximum connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.config.max_connections = max_connections;
        self
    }

    /// Enable TLS with system CA verification
    pub fn with_tls_system_ca(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::SystemCa;
        self
    }

    /// Enable TLS with custom CA certificate
    pub fn with_tls_custom_ca(mut self, ca_cert_path: impl Into<std::path::PathBuf>) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::CustomCa;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self
    }

    /// Enable TLS with verification skipped (insecure)
    pub fn with_tls_skip_verify(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::Skip;
        self
    }

    /// Enable mutual TLS with client certificate and key
    pub fn with_mutual_tls(
        mut self,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Enable mutual TLS with custom CA, client certificate and key
    pub fn with_mutual_tls_with_ca(
        mut self,
        ca_cert_path: impl Into<std::path::PathBuf>,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> WebSocketServerConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tls::VerificationMode;
    use std::path::PathBuf;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(
            config.ping_interval_ms,
            timeouts::DEFAULT_WEBSOCKET_PING_INTERVAL_MS
        );
        assert_eq!(
            config.max_message_size,
            limits::DEFAULT_WEBSOCKET_MESSAGE_SIZE
        );
        assert!(config.enable_compression);
        assert_eq!(config.subprotocols, vec!["qollective".to_string()]);
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_config_default() {
        let config = WebSocketClientConfig::default();
        assert_eq!(
            config.connection_timeout_ms,
            timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS
        );
        assert_eq!(
            config.message_timeout_ms,
            timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS
        );
        assert_eq!(config.ping_timeout_ms, 10000);
        assert_eq!(config.max_frame_size, 16 * 1024 * 1024);
        assert_eq!(config.max_retry_attempts, 3);
        assert!(config.user_agent.contains("qollective-rust"));
        assert!(config.connection_headers.is_empty());
        assert!(!config.tls.enabled);
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_config_default() {
        let config = WebSocketServerConfig::default();
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(
            config.max_connections,
            limits::DEFAULT_MAX_WEBSOCKET_CONNECTIONS
        );
        assert!(!config.tls.enabled);
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_config_methods() {
        let config = WebSocketClientConfig::default();

        // Test duration methods
        assert_eq!(
            config.ping_interval().as_millis(),
            config.websocket.ping_interval_ms as u128
        );
        assert_eq!(
            config.ping_timeout().as_millis(),
            config.ping_timeout_ms as u128
        );

        // Test compatibility methods
        assert_eq!(config.max_message_size(), config.websocket.max_message_size);
        assert_eq!(
            config.compression_enabled(),
            config.websocket.enable_compression
        );
        assert_eq!(config.subprotocols(), &config.websocket.subprotocols);
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_basic() {
        let config = WebSocketClientConfig::builder()
            .with_connection_timeout(5000)
            .with_message_timeout(3000)
            .with_ping_timeout(1000)
            .with_max_frame_size(1024 * 1024)
            .with_max_retry_attempts(5)
            .with_user_agent("test-client")
            .build();

        assert_eq!(config.connection_timeout_ms, 5000);
        assert_eq!(config.message_timeout_ms, 3000);
        assert_eq!(config.ping_timeout_ms, 1000);
        assert_eq!(config.max_frame_size, 1024 * 1024);
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.user_agent, "test-client");
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_connection_headers() {
        let config = WebSocketClientConfig::builder()
            .with_connection_header("Authorization", "Bearer token123")
            .with_connection_header("X-Custom", "value")
            .build();

        assert_eq!(config.connection_headers.len(), 2);
        assert_eq!(
            config.connection_headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            config.connection_headers.get("X-Custom"),
            Some(&"value".to_string())
        );

        // Test with_connection_headers
        let mut headers = std::collections::HashMap::new();
        headers.insert("Auth".to_string(), "token".to_string());
        headers.insert("Version".to_string(), "1.0".to_string());

        let config2 = WebSocketClientConfig::builder()
            .with_connection_headers(headers.clone())
            .build();

        assert_eq!(config2.connection_headers, headers);
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_tls_system_ca() {
        let config = WebSocketClientConfig::builder()
            .with_tls_system_ca()
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_tls_custom_ca() {
        let config = WebSocketClientConfig::builder()
            .with_tls_custom_ca("/path/to/ca.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
        assert_eq!(
            config.tls.ca_cert_path,
            Some(PathBuf::from("/path/to/ca.pem"))
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_tls_skip_verify() {
        let config = WebSocketClientConfig::builder()
            .with_tls_skip_verify()
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::Skip);
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_mutual_tls() {
        let config = WebSocketClientConfig::builder()
            .with_mutual_tls("/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
        assert_eq!(
            config.tls.cert_path,
            Some(PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(config.tls.key_path, Some(PathBuf::from("/path/to/key.pem")));
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_mutual_tls_with_ca() {
        let config = WebSocketClientConfig::builder()
            .with_mutual_tls_with_ca("/path/to/ca.pem", "/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
        assert_eq!(
            config.tls.ca_cert_path,
            Some(PathBuf::from("/path/to/ca.pem"))
        );
        assert_eq!(
            config.tls.cert_path,
            Some(PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(config.tls.key_path, Some(PathBuf::from("/path/to/key.pem")));
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_basic() {
        let config = WebSocketServerConfig::builder()
            .with_bind_address("0.0.0.0")
            .with_port(9090)
            .with_max_connections(1000)
            .build();

        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 9090);
        assert_eq!(config.max_connections, 1000);
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_tls_system_ca() {
        let config = WebSocketServerConfig::builder()
            .with_tls_system_ca()
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_tls_custom_ca() {
        let config = WebSocketServerConfig::builder()
            .with_tls_custom_ca("/path/to/ca.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::CustomCa);
        assert_eq!(
            config.tls.ca_cert_path,
            Some(PathBuf::from("/path/to/ca.pem"))
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_tls_skip_verify() {
        let config = WebSocketServerConfig::builder()
            .with_tls_skip_verify()
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::Skip);
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_mutual_tls() {
        let config = WebSocketServerConfig::builder()
            .with_mutual_tls("/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
        assert_eq!(
            config.tls.cert_path,
            Some(PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(config.tls.key_path, Some(PathBuf::from("/path/to/key.pem")));
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_mutual_tls_with_ca() {
        let config = WebSocketServerConfig::builder()
            .with_mutual_tls_with_ca("/path/to/ca.pem", "/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::MutualTls);
        assert_eq!(
            config.tls.ca_cert_path,
            Some(PathBuf::from("/path/to/ca.pem"))
        );
        assert_eq!(
            config.tls.cert_path,
            Some(PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(config.tls.key_path, Some(PathBuf::from("/path/to/key.pem")));
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_builder_chaining() {
        let config = WebSocketClientConfig::builder()
            .with_connection_timeout(5000)
            .with_tls_system_ca()
            .with_user_agent("test-client")
            .with_max_retry_attempts(10)
            .build();

        assert_eq!(config.connection_timeout_ms, 5000);
        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::SystemCa);
        assert_eq!(config.user_agent, "test-client");
        assert_eq!(config.max_retry_attempts, 10);
    }

    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_builder_chaining() {
        let config = WebSocketServerConfig::builder()
            .with_bind_address("0.0.0.0")
            .with_port(8443)
            .with_tls_skip_verify()
            .with_max_connections(500)
            .build();

        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 8443);
        assert!(config.tls.enabled);
        assert_eq!(config.tls.verification_mode, VerificationMode::Skip);
        assert_eq!(config.max_connections, 500);
    }
}
