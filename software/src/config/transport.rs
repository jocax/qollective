// ABOUTME: Unified transport configuration architecture for all protocol implementations
// ABOUTME: Provides seamless conversion between protocol-specific configs and unified transport layer

//! Unified transport configuration for the Qollective framework.
//!
//! This module provides a unified configuration interface that works seamlessly with
//! the HybridTransportClient while supporting all protocol-specific configurations.
//! It enables clean configuration hierarchy and automatic transport selection.

use super::presets::{LoggingConfig, PerformanceConfig, TenantClientConfig, TlsConfig};
use crate::error::{QollectiveError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Unified transport configuration that works with HybridTransportClient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport detection and selection configuration
    pub detection: TransportDetectionConfig,
    /// Protocol-specific configurations
    pub protocols: ProtocolConfigs,
    /// Global transport settings
    pub global: GlobalTransportConfig,
    /// Endpoint-specific configuration overrides
    pub endpoints: HashMap<String, EndpointTransportConfig>,
}

/// Transport detection and capability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportDetectionConfig {
    /// Whether to enable automatic protocol detection
    pub enable_auto_detection: bool,
    /// Timeout for capability detection
    pub detection_timeout_ms: u64,
    /// Cache TTL for detected capabilities
    pub capability_cache_ttl_ms: u64,
    /// Whether to retry failed detections
    pub retry_failed_detections: bool,
    /// Maximum retry attempts for detection
    pub max_detection_retries: u32,
    /// Preferred protocol order for selection
    pub preferred_protocols: Vec<String>,
}

/// Protocol-specific configurations container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfigs {
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub nats: Option<super::nats::NatsConfig>,

    #[cfg(feature = "grpc-client")]
    pub grpc_client: Option<super::grpc::GrpcClientConfig>,

    #[cfg(feature = "grpc-server")]
    pub grpc_server: Option<super::grpc::GrpcServerConfig>,

    pub rest: Option<super::presets::RestConfig>,

    #[cfg(feature = "websocket-client")]
    pub websocket: Option<super::websocket::WebSocketConfig>,

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub mcp: Option<super::mcp::McpClientConfig>,


    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub a2a: Option<super::a2a::A2AClientConfig>,
}

/// Global transport configuration applied to all protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalTransportConfig {
    /// Global timeout for all transport operations
    pub default_timeout_ms: u64,
    /// Maximum connections across all transports
    pub max_total_connections: usize,
    /// Global retry configuration
    pub retry_config: RetryConfig,
    /// Global TLS settings
    pub tls: TlsConfig,
    /// Global logging configuration
    pub logging: LoggingConfig,
    /// Global performance settings
    pub performance: PerformanceConfig,
    /// Global tenant configuration
    pub tenant: TenantClientConfig,
}

/// Endpoint-specific transport configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTransportConfig {
    /// Override timeout for this endpoint
    pub timeout_ms: Option<u64>,
    /// Override retry configuration
    pub retry_config: Option<RetryConfig>,
    /// Force specific protocol for this endpoint
    pub force_protocol: Option<String>,
    /// Endpoint-specific headers
    pub headers: HashMap<String, String>,
    /// Endpoint-specific TLS configuration
    pub tls: Option<TlsConfig>,
    /// Endpoint-specific performance settings
    pub performance: Option<PerformanceConfig>,
}

/// Retry configuration for transport operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to use jitter in retry delays
    pub use_jitter: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            detection: TransportDetectionConfig::default(),
            protocols: ProtocolConfigs::default(),
            global: GlobalTransportConfig::default(),
            endpoints: HashMap::new(),
        }
    }
}

impl Default for TransportDetectionConfig {
    fn default() -> Self {
        Self {
            enable_auto_detection: true,
            detection_timeout_ms: 5000,
            capability_cache_ttl_ms: 300000, // 5 minutes
            retry_failed_detections: true,
            max_detection_retries: 3,
            preferred_protocols: vec![
                "nats".to_string(),
                "grpc".to_string(),
                "rest".to_string(),
                "websocket".to_string(),
                "mcp".to_string(),
                "mcp-stdio".to_string(),
            ],
        }
    }
}

impl Default for ProtocolConfigs {
    fn default() -> Self {
        Self {
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            nats: None,

            #[cfg(feature = "grpc-client")]
            grpc_client: None,

            #[cfg(feature = "grpc-server")]
            grpc_server: None,

            rest: None,

            #[cfg(feature = "websocket-client")]
            websocket: None,

            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            mcp: None,


            #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
            a2a: None,
        }
    }
}

impl Default for GlobalTransportConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            max_total_connections: 1000,
            retry_config: RetryConfig::default(),
            tls: TlsConfig {
                enabled: false,
                cert_path: None,
                key_path: None,
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::SystemCa,
            },
            logging: LoggingConfig {
                enabled: true,
                log_requests: true,
                log_responses: false,
                log_headers: false,
                log_body: false,
                log_level: "info".to_string(),
                structured_logging: true,
            },
            performance: PerformanceConfig {
                enabled: true,
                track_request_duration: true,
                track_response_size: true,
                track_connection_pool: true,
                benchmarking_enabled: false,
                metrics_collection: true,
            },
            tenant: TenantClientConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl TransportConfig {
    /// Create a new transport configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to HybridTransportClient detection config
    pub fn to_detection_config(&self) -> crate::transport::TransportDetectionConfig {
        crate::transport::TransportDetectionConfig {
            enable_auto_detection: self.detection.enable_auto_detection,
            detection_timeout: Duration::from_millis(self.detection.detection_timeout_ms),
            capability_cache_ttl: Duration::from_millis(self.detection.capability_cache_ttl_ms),
            retry_failed_detections: self.detection.retry_failed_detections,
            max_detection_retries: self.detection.max_detection_retries,
        }
    }

    /// Get timeout for specific endpoint
    pub fn get_endpoint_timeout(&self, endpoint: &str) -> u64 {
        self.endpoints
            .get(endpoint)
            .and_then(|config| config.timeout_ms)
            .unwrap_or(self.global.default_timeout_ms)
    }

    /// Get retry config for specific endpoint
    pub fn get_endpoint_retry_config(&self, endpoint: &str) -> &RetryConfig {
        self.endpoints
            .get(endpoint)
            .and_then(|config| config.retry_config.as_ref())
            .unwrap_or(&self.global.retry_config)
    }

    /// Get forced protocol for endpoint
    pub fn get_forced_protocol(&self, endpoint: &str) -> Option<&str> {
        self.endpoints
            .get(endpoint)
            .and_then(|config| config.force_protocol.as_deref())
    }

    /// Validate the transport configuration
    pub fn validate(&self) -> Result<()> {
        // Validate detection config
        if self.detection.detection_timeout_ms == 0 {
            return Err(QollectiveError::config(
                "Detection timeout must be greater than 0",
            ));
        }

        if self.detection.capability_cache_ttl_ms == 0 {
            return Err(QollectiveError::config(
                "Capability cache TTL must be greater than 0",
            ));
        }

        // Validate global config
        if self.global.default_timeout_ms == 0 {
            return Err(QollectiveError::config(
                "Default timeout must be greater than 0",
            ));
        }

        if self.global.max_total_connections == 0 {
            return Err(QollectiveError::config(
                "Max total connections must be greater than 0",
            ));
        }

        // Validate retry config
        if self.global.retry_config.max_attempts == 0 {
            return Err(QollectiveError::config(
                "Max retry attempts must be greater than 0",
            ));
        }

        if self.global.retry_config.initial_delay_ms == 0 {
            return Err(QollectiveError::config(
                "Initial retry delay must be greater than 0",
            ));
        }

        if self.global.retry_config.backoff_multiplier <= 0.0 {
            return Err(QollectiveError::config(
                "Backoff multiplier must be greater than 0",
            ));
        }

        // Validate protocol configs
        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        if let Some(ref nats_config) = self.protocols.nats {
            nats_config
                .connection
                .validate()
                .map_err(|e| QollectiveError::config(e))?;
            nats_config
                .client
                .validate()
                .map_err(|e| QollectiveError::config(e))?;
            nats_config
                .server
                .validate()
                .map_err(|e| QollectiveError::config(e))?;
            nats_config
                .discovery
                .validate()
                .map_err(|e| QollectiveError::config(e))?;
        }

        #[cfg(feature = "grpc-client")]
        if let Some(ref grpc_client_config) = self.protocols.grpc_client {
            grpc_client_config.validate()?;
        }

        #[cfg(feature = "grpc-server")]
        if let Some(ref grpc_server_config) = self.protocols.grpc_server {
            grpc_server_config.validate()?;
        }

        Ok(())
    }
}

/// Builder for creating transport configurations
pub struct TransportConfigBuilder {
    config: TransportConfig,
}

impl TransportConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: TransportConfig::default(),
        }
    }

    /// Set detection configuration
    pub fn with_detection_config(mut self, detection: TransportDetectionConfig) -> Self {
        self.config.detection = detection;
        self
    }

    /// Set global timeout
    pub fn with_global_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.global.default_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum total connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.config.global.max_total_connections = max_connections;
        self
    }

    /// Set preferred protocols
    pub fn with_preferred_protocols(mut self, protocols: Vec<String>) -> Self {
        self.config.detection.preferred_protocols = protocols;
        self
    }

    /// Add endpoint-specific configuration
    pub fn with_endpoint_config(
        mut self,
        endpoint: String,
        config: EndpointTransportConfig,
    ) -> Self {
        self.config.endpoints.insert(endpoint, config);
        self
    }

    /// Set NATS configuration
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_config(mut self, nats_config: super::nats::NatsConfig) -> Self {
        self.config.protocols.nats = Some(nats_config);
        self
    }

    /// Set gRPC client configuration
    #[cfg(feature = "grpc-client")]
    pub fn with_grpc_client_config(
        mut self,
        grpc_client_config: super::grpc::GrpcClientConfig,
    ) -> Self {
        self.config.protocols.grpc_client = Some(grpc_client_config);
        self
    }

    /// Set gRPC server configuration
    #[cfg(feature = "grpc-server")]
    pub fn with_grpc_server_config(
        mut self,
        grpc_server_config: super::grpc::GrpcServerConfig,
    ) -> Self {
        self.config.protocols.grpc_server = Some(grpc_server_config);
        self
    }

    /// Set REST configuration
    pub fn with_rest_config(mut self, rest_config: super::presets::RestConfig) -> Self {
        self.config.protocols.rest = Some(rest_config);
        self
    }

    /// Build and validate the configuration
    pub fn build(self) -> Result<TransportConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for TransportConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::presets;

    // TDD: Write failing tests FIRST for unified transport configuration

    #[test]
    fn test_transport_config_fails_without_implementation() {
        // This test should initially fail until we implement the configuration properly
        let config = TransportConfig::new();

        // Should have detection config
        assert!(config.detection.enable_auto_detection);
        assert_eq!(config.detection.detection_timeout_ms, 5000);

        // Should have global config
        assert_eq!(config.global.default_timeout_ms, 30000);
        assert_eq!(config.global.max_total_connections, 1000);

        // Should have NO protocol configs by default (all None)
        assert!(config.protocols.rest.is_none());

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        assert!(config.protocols.nats.is_none());

        #[cfg(feature = "grpc-client")]
        assert!(config.protocols.grpc_client.is_none());

        #[cfg(feature = "grpc-server")]
        assert!(config.protocols.grpc_server.is_none());
    }

    #[test]
    fn test_transport_config_validation_fails_for_invalid_values() {
        let mut config = TransportConfig::default();

        // Test invalid detection timeout
        config.detection.detection_timeout_ms = 0;
        assert!(config.validate().is_err());

        // Reset and test invalid global timeout
        config.detection.detection_timeout_ms = 5000;
        config.global.default_timeout_ms = 0;
        assert!(config.validate().is_err());

        // Reset and test invalid max connections
        config.global.default_timeout_ms = 30000;
        config.global.max_total_connections = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_transport_config_to_detection_config_conversion_fails() {
        let transport_config = TransportConfig::default();

        // This should convert to HybridTransportClient's detection config
        let detection_config = transport_config.to_detection_config();

        assert_eq!(detection_config.enable_auto_detection, true);
        assert_eq!(
            detection_config.detection_timeout,
            Duration::from_millis(5000)
        );
        assert_eq!(
            detection_config.capability_cache_ttl,
            Duration::from_millis(300000)
        );
        assert_eq!(detection_config.retry_failed_detections, true);
        assert_eq!(detection_config.max_detection_retries, 3);
    }

    #[test]
    fn test_endpoint_specific_configuration_fails() {
        let mut config = TransportConfig::default();

        // Add endpoint-specific config
        let endpoint_config = EndpointTransportConfig {
            timeout_ms: Some(60000),
            retry_config: Some(RetryConfig {
                max_attempts: 5,
                initial_delay_ms: 2000,
                max_delay_ms: 60000,
                backoff_multiplier: 3.0,
                use_jitter: false,
            }),
            force_protocol: Some("grpc".to_string()),
            headers: [("X-Custom".to_string(), "value".to_string())].into(),
            tls: None,
            performance: None,
        };

        config
            .endpoints
            .insert("https://special.example.com".to_string(), endpoint_config);

        // Test endpoint timeout override
        assert_eq!(
            config.get_endpoint_timeout("https://special.example.com"),
            60000
        );
        assert_eq!(
            config.get_endpoint_timeout("https://other.example.com"),
            30000
        ); // default

        // Test forced protocol
        assert_eq!(
            config.get_forced_protocol("https://special.example.com"),
            Some("grpc")
        );
        assert_eq!(
            config.get_forced_protocol("https://other.example.com"),
            None
        );

        // Test retry config override
        let retry_config = config.get_endpoint_retry_config("https://special.example.com");
        assert_eq!(retry_config.max_attempts, 5);
        assert_eq!(retry_config.initial_delay_ms, 2000);
    }

    #[test]
    fn test_transport_config_builder_fails() {
        let config = TransportConfigBuilder::new()
            .with_global_timeout(45000)
            .with_max_connections(2000)
            .with_preferred_protocols(vec![
                "grpc".to_string(),
                "nats".to_string(),
                "rest".to_string(),
            ])
            .build()
            .expect("Should build valid config");

        assert_eq!(config.global.default_timeout_ms, 45000);
        assert_eq!(config.global.max_total_connections, 2000);
        assert_eq!(
            config.detection.preferred_protocols,
            vec!["grpc", "nats", "rest"]
        );
    }

    #[test]
    fn test_protocol_specific_config_integration_fails() {
        let config = TransportConfig::default();

        // Test that protocol configs are None by default (explicit configuration required)
        assert!(config.protocols.rest.is_none());

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            assert!(config.protocols.nats.is_none());
        }

        #[cfg(feature = "grpc-client")]
        {
            assert!(config.protocols.grpc_client.is_none());
        }

        #[cfg(feature = "grpc-server")]
        {
            assert!(config.protocols.grpc_server.is_none());
        }

        // Test that we can explicitly configure protocols
        let mut config_with_protocols = TransportConfig::default();
        config_with_protocols.protocols.rest = Some(presets::RestConfig {
            client: Some(presets::RestClientConfig::default()),
            server: None,
        });
        assert!(config_with_protocols.protocols.rest.is_some());
    }

    #[test]
    fn test_global_config_overrides_fail() {
        let mut config = TransportConfig::default();

        // Modify global settings
        config.global.default_timeout_ms = 15000;
        config.global.retry_config.max_attempts = 1;
        config.global.tls.enabled = true;

        // Global settings should affect all protocols
        assert_eq!(config.global.default_timeout_ms, 15000);
        assert_eq!(config.global.retry_config.max_attempts, 1);
        assert!(config.global.tls.enabled);

        // Validation should still pass
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serialization_roundtrip_fails() {
        let config = TransportConfig::default();

        // Test JSON serialization
        let json = serde_json::to_string(&config).expect("Should serialize to JSON");
        assert!(json.contains("detection"));
        assert!(json.contains("protocols"));
        assert!(json.contains("global"));

        // Test deserialization
        let deserialized: TransportConfig =
            serde_json::from_str(&json).expect("Should deserialize from JSON");

        assert_eq!(
            deserialized.detection.enable_auto_detection,
            config.detection.enable_auto_detection
        );
        assert_eq!(
            deserialized.global.default_timeout_ms,
            config.global.default_timeout_ms
        );
        assert_eq!(
            deserialized.global.max_total_connections,
            config.global.max_total_connections
        );
    }
}
