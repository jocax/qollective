// ABOUTME: NATS-specific configuration structures for client/server communication
// ABOUTME: Provides connection settings, discovery parameters, and authentication config

//! NATS configuration types for the Qollective framework.
//!
//! This module provides comprehensive configuration structures for NATS messaging,
//! including connection settings, client/server configuration, and service discovery
//! parameters following the existing Qollective configuration patterns.

use crate::constants::{circuit_breaker, limits, network, timeouts};
use crate::crypto::CryptoProviderStrategy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main NATS configuration structure combining all NATS-related settings.
///
/// This configuration provides a comprehensive setup for NATS messaging within
/// the Qollective framework, supporting both client and server operations,
/// service discovery, and various authentication methods.
///
/// # Examples
///
/// ```rust
/// use qollective::config::nats::NatsConfig;
///
/// // Create default configuration
/// let config = NatsConfig::default();
/// assert_eq!(config.connection.urls, vec!["nats://localhost:4222"]);
///
/// // Create custom configuration using builder
/// let custom_config = NatsConfig::builder()
///     .with_urls(vec!["nats://server1:4222".to_string()])
///     .with_tls(true)
///     .with_server_enabled(true)
///     .build()
///     .expect("Valid configuration");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsConfig {
    /// NATS connection settings including URLs, timeouts, and authentication
    pub connection: NatsConnectionConfig,
    /// Client-specific configuration for outbound messaging
    pub client: NatsClientBehaviorConfig,
    /// Server-specific configuration for inbound message handling
    pub server: NatsServerConfig,
    /// Service discovery configuration for agent registration
    pub discovery: NatsDiscoveryConfig,
}

/// NATS connection configuration managing how to connect to NATS servers.
///
/// This configuration handles connection URLs, authentication credentials,
/// TLS settings, and connection behavior like timeouts and retry logic.
///
/// # Examples
///
/// ```rust
/// use qollective::config::nats::NatsConnectionConfig;
/// use std::collections::HashMap;
///
/// let mut config = NatsConnectionConfig::default();
/// config.urls = vec!["nats://server1:4222".to_string(), "nats://server2:4222".to_string()];
/// config.tls.enabled = true;
/// config.username = Some("myuser".to_string());
/// config.password = Some("mypass".to_string());
///
/// assert!(config.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsConnectionConfig {
    /// List of NATS server URLs to connect to (supports failover)
    pub urls: Vec<String>,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Reconnection timeout in milliseconds
    pub reconnect_timeout_ms: u64,
    /// Maximum number of reconnection attempts (None = unlimited)
    pub max_reconnect_attempts: Option<u32>,
    /// Username for authentication (optional)
    pub username: Option<String>,
    /// Password for authentication (optional)
    pub password: Option<String>,
    /// Authentication token (alternative to username/password)
    pub token: Option<String>,
    /// NKey authentication - path to NKey seed file (.nk file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_file: Option<std::path::PathBuf>,
    /// NKey authentication - seed string directly (for programmatic use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_seed: Option<String>,
    /// TLS configuration for secure connections
    pub tls: crate::config::tls::TlsConfig,
    /// Strategy for crypto provider initialization (None = AutoInstall)
    pub crypto_provider_strategy: Option<CryptoProviderStrategy>,
    /// Custom headers to include in connection requests
    pub custom_headers: HashMap<String, String>,
    /// Client name to identify this connection in NATS server logs and monitoring
    pub client_name: Option<String>,
}

/// NATS client behavior configuration for outbound messaging behavior.
///
/// This configuration controls how the application sends messages to NATS,
/// including timeouts, retry logic, and connection pooling.
///
/// Note: This is the internal client behavior config, nested within NatsConfig.
/// For standalone client usage, use the main NatsClientConfig instead.
///
/// # Examples
///
/// ```rust
/// use qollective::config::nats::NatsClientBehaviorConfig;
///
/// let mut config = NatsClientBehaviorConfig::default();
/// config.request_timeout_ms = 60000; // 60 seconds
/// config.retry_attempts = 5;
/// config.connection_pool_size = 10;
///
/// assert!(config.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsClientBehaviorConfig {
    /// Request timeout in milliseconds for request-reply operations
    pub request_timeout_ms: u64,
    /// Maximum number of pending messages before backpressure
    pub max_pending_messages: usize,
    /// Number of retry attempts for failed requests
    pub retry_attempts: u32,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Size of the connection pool for concurrent operations
    pub connection_pool_size: usize,
}

/// NATS server configuration for inbound message handling.
///
/// This configuration controls how the application receives and processes
/// messages from NATS, including subject patterns, queue groups, and
/// concurrency settings.
///
/// # Examples
///
/// ```rust
/// use qollective::config::nats::NatsServerConfig;
///
/// let mut config = NatsServerConfig::default();
/// config.enabled = true;
/// config.subject_prefix = "myapp".to_string();
/// config.queue_group = Some("workers".to_string());
/// config.max_concurrent_handlers = 100;
///
/// assert!(config.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsServerConfig {
    /// Whether the NATS server functionality is enabled
    pub enabled: bool,
    /// Subject prefix for all server subscriptions
    pub subject_prefix: String,
    /// Queue group name for load balancing (optional)
    pub queue_group: Option<String>,
    /// Maximum number of concurrent message handlers
    pub max_concurrent_handlers: usize,
    /// Timeout for individual message handlers in milliseconds
    pub handler_timeout_ms: u64,
    /// Whether to enable request-reply pattern support
    pub enable_request_reply: bool,
}

/// NATS discovery configuration for service registration and capabilities.
///
/// This configuration manages how agents register themselves and announce
/// their capabilities to other agents in the NATS network.
///
/// # Examples
///
/// ```rust
/// use qollective::config::nats::NatsDiscoveryConfig;
///
/// let mut config = NatsDiscoveryConfig::default();
/// config.enabled = true;
/// config.agent_registry_subject = "agents.registry".to_string();
/// config.announcement_interval_ms = 30000; // 30 seconds
/// config.ttl_ms = 90000; // 90 seconds
///
/// assert!(config.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsDiscoveryConfig {
    /// Whether service discovery is enabled
    pub enabled: bool,
    /// Subject pattern for agent registry announcements
    pub agent_registry_subject: String,
    /// Subject pattern for capability announcements
    pub capability_subject: String,
    /// Interval between agent announcements in milliseconds
    pub announcement_interval_ms: u64,
    /// Time-to-live for agent registrations in milliseconds
    pub ttl_ms: u64,
    /// Whether to automatically register this agent on startup
    pub auto_register: bool,
}

/// Standalone NATS client configuration for API consistency.
///
/// This configuration provides everything a NATS client needs in a single config,
/// following the same pattern as GrpcClientConfig, RestClientConfig, and McpClientConfig.
/// This ensures API consistency across all protocol clients.
///
/// # Examples
///
/// ```rust
/// use qollective::config::nats::NatsClientConfig;
/// use std::time::Duration;
///
/// let config = NatsClientConfig::builder()
///     .with_urls(vec!["nats://server1:4222".to_string()])
///     .with_request_timeout(Duration::from_secs(30))
///     .with_retry_attempts(3)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsClientConfig {
    /// NATS connection settings including URLs, timeouts, and authentication
    pub connection: NatsConnectionConfig,
    /// Client behavior configuration for outbound messaging
    pub client_behavior: NatsClientBehaviorConfig,
    /// Discovery cache TTL from discovery config (only the client-relevant part)
    pub discovery_cache_ttl_ms: u64,
}

impl Default for NatsClientConfig {
    fn default() -> Self {
        Self {
            connection: NatsConnectionConfig::default(),
            client_behavior: NatsClientBehaviorConfig::default(),
            discovery_cache_ttl_ms: 300000, // 5 minutes, extracted from NatsDiscoveryConfig::default()
        }
    }
}

/// Builder for NATS client configuration
pub struct NatsClientConfigBuilder {
    config: NatsClientConfig,
}

impl NatsClientConfigBuilder {
    /// Create a new NATS client config builder
    pub fn new() -> Self {
        Self {
            config: NatsClientConfig::default(),
        }
    }

    /// Set NATS server URLs
    pub fn with_urls(mut self, urls: Vec<String>) -> Self {
        self.config.connection.urls = urls;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.connection.connection_timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set request timeout  
    pub fn with_request_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.client_behavior.request_timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set retry attempts
    pub fn with_retry_attempts(mut self, attempts: u32) -> Self {
        self.config.client_behavior.retry_attempts = attempts;
        self
    }

    /// Set discovery cache TTL
    pub fn with_discovery_cache_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.config.discovery_cache_ttl_ms = ttl.as_millis() as u64;
        self
    }

    /// Enable TLS
    pub fn with_tls(mut self, enabled: bool) -> Self {
        self.config.connection.tls.enabled = enabled;
        self
    }

    /// Set TLS certificate files
    pub fn with_tls_files(
        mut self,
        ca_file: Option<String>,
        cert_file: Option<String>,
        key_file: Option<String>,
    ) -> Self {
        self.config.connection.tls.ca_cert_path = ca_file.map(std::path::PathBuf::from);
        self.config.connection.tls.cert_path = cert_file.map(std::path::PathBuf::from);
        self.config.connection.tls.key_path = key_file.map(std::path::PathBuf::from);
        self
    }

    /// Set authentication credentials
    pub fn with_credentials(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.config.connection.username = username;
        self.config.connection.password = password;
        self
    }

    /// Set authentication token
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.config.connection.token = token;
        self
    }

    /// Set NKey authentication using seed file
    pub fn with_nkey_file(mut self, nkey_file: std::path::PathBuf) -> Self {
        self.config.connection.nkey_file = Some(nkey_file);
        self
    }

    /// Set NKey authentication using seed string directly
    pub fn with_nkey_seed(mut self, nkey_seed: String) -> Self {
        self.config.connection.nkey_seed = Some(nkey_seed);
        self
    }

    /// Set TLS insecure mode (skip certificate verification)
    pub fn with_tls_insecure(mut self, insecure: bool) -> Self {
        self.config.connection.tls.verification_mode = if insecure {
            crate::config::tls::VerificationMode::Skip
        } else {
            crate::config::tls::VerificationMode::SystemCa
        };
        self
    }

    /// Set the client name for NATS connection identification
    pub fn with_client_name(mut self, name: impl Into<String>) -> Self {
        self.config.connection.client_name = Some(name.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> NatsClientConfig {
        self.config
    }
}

impl Default for NatsClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NatsClientConfig {
    /// Create a new builder for NATS client configuration
    pub fn builder() -> NatsClientConfigBuilder {
        NatsClientConfigBuilder::new()
    }
}

/// Main NATS configuration combining all sub-configurations
impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            connection: NatsConnectionConfig::default(),
            client: NatsClientBehaviorConfig::default(),
            server: NatsServerConfig::default(),
            discovery: NatsDiscoveryConfig::default(),
        }
    }
}

/// Default NATS connection configuration with sensible defaults
impl Default for NatsConnectionConfig {
    fn default() -> Self {
        Self {
            urls: vec![network::DEFAULT_NATS_URL.to_string()],
            connection_timeout_ms: timeouts::DEFAULT_NATS_CONNECTION_TIMEOUT_MS,
            reconnect_timeout_ms: timeouts::DEFAULT_NATS_RECONNECT_TIMEOUT_MS,
            max_reconnect_attempts: Some(limits::DEFAULT_NATS_MAX_RECONNECT_ATTEMPTS),
            username: None,
            password: None,
            token: None,
            nkey_file: None,
            nkey_seed: None,
            tls: crate::config::tls::TlsConfig::default(),
            crypto_provider_strategy: None, // Default to AutoInstall
            custom_headers: HashMap::new(),
            client_name: None,
        }
    }
}

/// Default NATS client behavior configuration optimized for typical usage
impl Default for NatsClientBehaviorConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: timeouts::DEFAULT_NATS_REQUEST_TIMEOUT_MS,
            max_pending_messages: limits::DEFAULT_NATS_MAX_PENDING_MESSAGES,
            retry_attempts: circuit_breaker::DEFAULT_MAX_RETRIES,
            retry_delay_ms: timeouts::DEFAULT_NATS_RETRY_DELAY_MS,
            connection_pool_size: circuit_breaker::DEFAULT_FAILURE_THRESHOLD as usize,
        }
    }
}

/// Default NATS server configuration with conservative settings
impl Default for NatsServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            subject_prefix: "qollective".to_string(),
            queue_group: None,
            max_concurrent_handlers: limits::DEFAULT_NATS_MAX_CONCURRENT_HANDLERS,
            handler_timeout_ms: timeouts::DEFAULT_NATS_REQUEST_TIMEOUT_MS,
            enable_request_reply: true,
        }
    }
}

/// Default NATS discovery configuration for agent registration
impl Default for NatsDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent_registry_subject: "qollective.agents".to_string(),
            capability_subject: "qollective.capabilities".to_string(),
            announcement_interval_ms: timeouts::DEFAULT_NATS_ANNOUNCEMENT_INTERVAL_MS,
            ttl_ms: timeouts::DEFAULT_NATS_TTL_MS,
            auto_register: true,
        }
    }
}

impl NatsConnectionConfig {
    /// Validates the connection configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.urls.is_empty() {
            return Err("At least one NATS URL must be specified".to_string());
        }

        if self.connection_timeout_ms == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }

        if self.reconnect_timeout_ms == 0 {
            return Err("Reconnect timeout must be greater than 0".to_string());
        }

        // Validate URLs have proper format
        for url in &self.urls {
            if url.is_empty() {
                return Err("NATS URL cannot be empty".to_string());
            }
            if !url.starts_with("nats://")
                && !url.starts_with("tls://")
                && !url.starts_with("ws://")
                && !url.starts_with("wss://")
            {
                return Err(format!("Invalid NATS URL scheme: {}", url));
            }
        }

        // Validate NKey configuration
        if self.nkey_file.is_some() && self.nkey_seed.is_some() {
            return Err("Cannot specify both nkey_file and nkey_seed - choose one".to_string());
        }

        // TLS configuration validation is handled by the unified TLS config
        self.tls
            .validate()
            .map_err(|e| format!("TLS configuration error: {}", e))?;

        Ok(())
    }
}

impl NatsClientBehaviorConfig {
    /// Validates the client behavior configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.request_timeout_ms == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        if self.max_pending_messages == 0 {
            return Err("Max pending messages must be greater than 0".to_string());
        }

        if self.retry_delay_ms == 0 {
            return Err("Retry delay must be greater than 0".to_string());
        }

        if self.connection_pool_size == 0 {
            return Err("Connection pool size must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl NatsClientConfig {
    /// Validates the standalone client configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate connection config
        self.connection.validate()?;

        // Validate client behavior config
        self.client_behavior.validate()?;

        if self.discovery_cache_ttl_ms == 0 {
            return Err("Discovery cache TTL must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl NatsServerConfig {
    /// Validates the server configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.subject_prefix.is_empty() {
            return Err("Subject prefix cannot be empty".to_string());
        }

        if self.max_concurrent_handlers == 0 {
            return Err("Max concurrent handlers must be greater than 0".to_string());
        }

        if self.handler_timeout_ms == 0 {
            return Err("Handler timeout must be greater than 0".to_string());
        }

        // Validate subject prefix format (basic check)
        if self.subject_prefix.contains(' ') || self.subject_prefix.contains('\t') {
            return Err("Subject prefix cannot contain whitespace".to_string());
        }

        Ok(())
    }
}

impl NatsDiscoveryConfig {
    /// Validates the discovery configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.agent_registry_subject.is_empty() {
            return Err("Agent registry subject cannot be empty".to_string());
        }

        if self.capability_subject.is_empty() {
            return Err("Capability subject cannot be empty".to_string());
        }

        if self.announcement_interval_ms == 0 {
            return Err("Announcement interval must be greater than 0".to_string());
        }

        if self.ttl_ms == 0 {
            return Err("TTL must be greater than 0".to_string());
        }

        if self.ttl_ms < self.announcement_interval_ms {
            return Err("TTL must be greater than or equal to announcement interval".to_string());
        }

        Ok(())
    }
}

/// Builder for creating NATS configurations with fluent API
pub struct NatsConfigBuilder {
    config: NatsConfig,
}

impl NatsConfig {
    /// Creates a new builder with default configuration
    pub fn builder() -> NatsConfigBuilder {
        NatsConfigBuilder {
            config: NatsConfig::default(),
        }
    }
}

impl NatsConfigBuilder {
    /// Sets the NATS server URLs
    pub fn with_urls(mut self, urls: Vec<String>) -> Self {
        self.config.connection.urls = urls;
        self
    }

    /// Enables or disables TLS
    pub fn with_tls(mut self, enabled: bool) -> Self {
        self.config.connection.tls.enabled = enabled;
        self
    }

    /// Enables or disables the NATS server functionality
    pub fn with_server_enabled(mut self, enabled: bool) -> Self {
        self.config.server.enabled = enabled;
        self
    }

    /// Enables or disables the discovery functionality
    pub fn with_discovery_enabled(mut self, enabled: bool) -> Self {
        self.config.discovery.enabled = enabled;
        self
    }

    /// Sets authentication credentials
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.config.connection.username = Some(username);
        self.config.connection.password = Some(password);
        self
    }

    /// Sets authentication token
    pub fn with_token(mut self, token: String) -> Self {
        self.config.connection.token = Some(token);
        self
    }

    /// Sets NKey authentication using seed file
    pub fn with_nkey_file(mut self, nkey_file: std::path::PathBuf) -> Self {
        self.config.connection.nkey_file = Some(nkey_file);
        self
    }

    /// Sets NKey authentication using seed string directly
    pub fn with_nkey_seed(mut self, nkey_seed: String) -> Self {
        self.config.connection.nkey_seed = Some(nkey_seed);
        self
    }

    /// Sets TLS certificate and key files
    pub fn with_tls_files(mut self, cert_file: String, key_file: String) -> Self {
        self.config.connection.tls.cert_path = Some(std::path::PathBuf::from(cert_file));
        self.config.connection.tls.key_path = Some(std::path::PathBuf::from(key_file));
        self
    }

    /// Sets TLS CA file for certificate verification
    pub fn with_ca_file(mut self, ca_file: String) -> Self {
        self.config.connection.tls.ca_cert_path = Some(std::path::PathBuf::from(ca_file));
        self
    }

    /// Sets the subject prefix for server operations
    pub fn with_subject_prefix(mut self, prefix: String) -> Self {
        self.config.server.subject_prefix = prefix;
        self
    }

    /// Sets client timeout settings
    pub fn with_client_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.client.request_timeout_ms = timeout_ms;
        self
    }

    /// Sets TLS insecure mode (skip certificate verification)
    pub fn with_tls_insecure(mut self, insecure: bool) -> Self {
        self.config.connection.tls.verification_mode = if insecure {
            crate::config::tls::VerificationMode::Skip
        } else {
            crate::config::tls::VerificationMode::SystemCa
        };
        self
    }

    /// Set the client name for NATS connection identification
    pub fn with_client_name(mut self, name: impl Into<String>) -> Self {
        self.config.connection.client_name = Some(name.into());
        self
    }

    /// Builds and validates the configuration
    pub fn build(self) -> Result<NatsConfig, String> {
        // Validate all sub-configurations
        self.config.connection.validate()?;
        self.config.client.validate()?;
        self.config.server.validate()?;
        self.config.discovery.validate()?;

        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();

        assert!(config
            .connection
            .urls
            .contains(&"nats://localhost:4222".to_string()));
        assert_eq!(config.connection.connection_timeout_ms, 5000);
        assert_eq!(config.client.request_timeout_ms, 30000);
        assert!(!config.server.enabled);
        assert_eq!(config.discovery.announcement_interval_ms, 30000);
    }

    #[test]
    fn test_nats_config_serialization_roundtrip() {
        let config = NatsConfig {
            connection: NatsConnectionConfig {
                urls: vec![
                    "nats://server1:4222".to_string(),
                    "nats://server2:4222".to_string(),
                ],
                connection_timeout_ms: 10000,
                reconnect_timeout_ms: 2000,
                max_reconnect_attempts: Some(10),
                username: Some("test_user".to_string()),
                password: Some("test_pass".to_string()),
                token: None,
                nkey_file: None,
                nkey_seed: None,
                tls: crate::config::tls::TlsConfig {
                    enabled: true,
                    verification_mode: crate::config::tls::VerificationMode::CustomCa,
                    ca_cert_path: Some(std::path::PathBuf::from("/path/to/ca.pem")),
                    cert_path: Some(std::path::PathBuf::from("/path/to/cert.pem")),
                    key_path: Some(std::path::PathBuf::from("/path/to/key.pem")),
                    ..Default::default()
                },
                crypto_provider_strategy: None,
                custom_headers: HashMap::new(),
                client_name: Some("test_client".to_string()),
            },
            client: NatsClientBehaviorConfig {
                request_timeout_ms: 60000,
                max_pending_messages: 1000,
                retry_attempts: 5,
                retry_delay_ms: 1000,
                connection_pool_size: 10,
            },
            server: NatsServerConfig {
                enabled: true,
                subject_prefix: "qollective".to_string(),
                queue_group: Some("qollective-workers".to_string()),
                max_concurrent_handlers: 100,
                handler_timeout_ms: 30000,
                enable_request_reply: true,
            },
            discovery: NatsDiscoveryConfig {
                enabled: true,
                agent_registry_subject: "qollective.agents".to_string(),
                capability_subject: "qollective.capabilities".to_string(),
                announcement_interval_ms: 60000,
                ttl_ms: 180000,
                auto_register: true,
            },
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&config).expect("Should serialize to JSON");
        assert!(json.contains("\"urls\":[\"nats://server1:4222\",\"nats://server2:4222\"]"));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"enabled\":true"));

        // Test deserialization from JSON
        let deserialized: NatsConfig =
            serde_json::from_str(&json).expect("Should deserialize from JSON");

        assert_eq!(deserialized.connection.urls, config.connection.urls);
        assert_eq!(
            deserialized.connection.tls.enabled,
            config.connection.tls.enabled
        );
        assert_eq!(deserialized.server.enabled, config.server.enabled);
        assert_eq!(deserialized.discovery.ttl_ms, config.discovery.ttl_ms);
    }

    #[test]
    fn test_nats_connection_config_validation() {
        let valid_config = NatsConnectionConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = NatsConnectionConfig {
            urls: vec![],             // Empty URLs should be invalid
            connection_timeout_ms: 0, // Zero timeout should be invalid
            crypto_provider_strategy: None,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_nats_client_config_validation() {
        let valid_config = NatsClientConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = NatsClientConfig {
            client_behavior: NatsClientBehaviorConfig {
                request_timeout_ms: 0,   // Zero timeout should be invalid
                max_pending_messages: 0, // Zero max pending should be invalid
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_nats_server_config_validation() {
        let valid_config = NatsServerConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = NatsServerConfig {
            subject_prefix: "".to_string(), // Empty prefix should be invalid
            max_concurrent_handlers: 0,     // Zero handlers should be invalid
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_nats_discovery_config_validation() {
        let valid_config = NatsDiscoveryConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = NatsDiscoveryConfig {
            agent_registry_subject: "".to_string(), // Empty subject should be invalid
            announcement_interval_ms: 0,            // Zero interval should be invalid
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_nats_config_builder_pattern() {
        let config = NatsConfig::builder()
            .with_urls(vec!["nats://test:4222".to_string()])
            .with_tls(true)
            .with_tls_files(
                "/path/to/cert.pem".to_string(),
                "/path/to/key.pem".to_string(),
            )
            .with_server_enabled(true)
            .with_discovery_enabled(false)
            .build()
            .expect("Builder should create valid config");

        assert_eq!(config.connection.urls, vec!["nats://test:4222".to_string()]);
        assert!(config.connection.tls.enabled);
        assert_eq!(
            config.connection.tls.cert_path,
            Some(std::path::PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(
            config.connection.tls.key_path,
            Some(std::path::PathBuf::from("/path/to/key.pem"))
        );
        assert!(config.server.enabled);
        assert!(!config.discovery.enabled);
    }

    #[test]
    fn test_nats_config_builder_with_tls_files() {
        let config = NatsConfig::builder()
            .with_urls(vec!["nats://secure:4443".to_string()])
            .with_tls(true)
            .with_tls_files(
                "/path/to/client-cert.pem".to_string(),
                "/path/to/client-key.pem".to_string(),
            )
            .with_ca_file("/path/to/ca.pem".to_string())
            .build()
            .expect("Builder should create valid TLS config");

        assert_eq!(
            config.connection.urls,
            vec!["nats://secure:4443".to_string()]
        );
        assert!(config.connection.tls.enabled);
        assert_eq!(
            config.connection.tls.cert_path,
            Some(std::path::PathBuf::from("/path/to/client-cert.pem"))
        );
        assert_eq!(
            config.connection.tls.key_path,
            Some(std::path::PathBuf::from("/path/to/client-key.pem"))
        );
        assert_eq!(
            config.connection.tls.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/ca.pem"))
        );
    }

    #[test]
    fn test_auth_config_serialization() {
        let config = NatsConnectionConfig {
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            token: Some("token123".to_string()),
            crypto_provider_strategy: None,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: NatsConnectionConfig =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.username, Some("user".to_string()));
        assert_eq!(deserialized.password, Some("pass".to_string()));
        assert_eq!(deserialized.token, Some("token123".to_string()));
    }

    #[test]
    fn test_custom_headers_serialization() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let config = NatsConnectionConfig {
            custom_headers: headers.clone(),
            crypto_provider_strategy: None,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: NatsConnectionConfig =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.custom_headers, headers);
    }

    #[test]
    fn test_client_name_configuration() {
        let config = NatsClientConfig::builder()
            .with_client_name("test-client")
            .build();

        assert_eq!(
            config.connection.client_name,
            Some("test-client".to_string())
        );
    }

    #[test]
    fn test_client_name_serialization() {
        let config = NatsConnectionConfig {
            client_name: Some("my-app-component".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: NatsConnectionConfig =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(
            deserialized.client_name,
            Some("my-app-component".to_string())
        );
    }
}
