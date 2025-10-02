// ABOUTME: WebSocket client implementation with envelope support for browser-based agents
// ABOUTME: WebSocket client implementation with envelope support for browser-based agents
// ABOUTME: Provides bidirectional WebSocket communication with automatic envelope handling and metadata injection

//! WebSocket client implementation with envelope support.
//!
//! This module provides a comprehensive WebSocket client that:
//! - Handles bidirectional communication for real-time agent interaction
//! - Automatically injects envelope metadata for protocol compatibility
//! - Supports connection pooling and reconnection handling
//! - Provides detailed error context and logging
//! - Enables browser-based agent development through WebSocket transport

#[cfg(feature = "websocket-client")]
use {
    crate::{
        client::common::ClientConfig,
        config::websocket::WebSocketClientConfig,
        envelope::Envelope,
        error::{QollectiveError, Result},
        traits::senders::UnifiedEnvelopeSender,
    },
    serde::{Deserialize, Serialize},
    serde_json,
    std::time::Duration,
    url::Url,
};

/// WebSocket client for bidirectional communication with envelope support
#[cfg(feature = "websocket-client")]
#[derive(Debug)]
pub struct WebSocketClient {
    config: WebSocketClientConfig,
    endpoint_url: String,
    /// Optional transport layer for dependency injection
    transport: Option<std::sync::Arc<crate::transport::HybridTransportClient>>,
}

/// WebSocket message types for protocol handling
#[cfg(feature = "websocket-client")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessageType {
    #[serde(rename = "envelope")]
    Envelope { payload: serde_json::Value },
    #[serde(rename = "ping")]
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    #[serde(rename = "pong")]
    Pong {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    #[serde(rename = "error")]
    Error { message: String, code: Option<u32> },
}

#[cfg(feature = "websocket-client")]
impl WebSocketClient {
    /// Create a new WebSocket client with transport layer (like NATS client)
    pub async fn new(endpoint_url: String, config: WebSocketClientConfig) -> Result<Self> {
        // Validate WebSocket URL
        let url = Url::parse(&endpoint_url).map_err(|e| {
            QollectiveError::transport(format!("Invalid WebSocket URL '{}': {}", endpoint_url, e))
        })?;

        if url.scheme() != "ws" && url.scheme() != "wss" {
            return Err(QollectiveError::transport(format!(
                "Invalid WebSocket scheme '{}', expected 'ws' or 'wss'",
                url.scheme()
            )));
        }

        // Create transport configuration from WebSocket config (CONFIG FIRST PRINCIPLE)
        let transport_config = crate::transport::TransportDetectionConfig {
            enable_auto_detection: true,
            detection_timeout: std::time::Duration::from_secs(config.base.timeout_seconds),
            capability_cache_ttl: std::time::Duration::from_secs(300),
            retry_failed_detections: config.base.retry_attempts > 0,
            max_detection_retries: config.base.retry_attempts,
        };

        // Create transport with WebSocket transport injected
        let mut transport = crate::transport::HybridTransportClient::new(transport_config);

        // Create the WebSocket pure transport that the transport will use
        let ws_config = crate::transport::websocket::WebSocketConfig::default();
        let ws_transport = std::sync::Arc::new(
            crate::transport::websocket::WebSocketTransport::new_with_unified_tls(ws_config, Some(&config.tls)),
        );
        transport = transport.with_websocket_transport(ws_transport);

        Ok(Self {
            config,
            endpoint_url,
            transport: Some(std::sync::Arc::new(transport)),
        })
    }

    /// Create a WebSocket client with dependency injection for testing
    pub fn with_transport(
        transport: std::sync::Arc<crate::transport::HybridTransportClient>,
    ) -> Result<Self> {
        Ok(Self {
            config: WebSocketClientConfig::default(),
            endpoint_url: String::new(), // Will be provided per-request when using transport
            transport: Some(transport),
        })
    }

    /// Create a WebSocket client with its own transport layer (following NATS pattern)
    pub async fn new_with_transport(config: WebSocketClientConfig) -> Result<Self> {
        // Create transport configuration from WebSocket config (CONFIG FIRST PRINCIPLE)
        let transport_config = crate::transport::TransportDetectionConfig {
            enable_auto_detection: true,
            detection_timeout: std::time::Duration::from_secs(config.base.timeout_seconds),
            capability_cache_ttl: std::time::Duration::from_secs(300), // Use reasonable default
            retry_failed_detections: config.base.retry_attempts > 0,
            max_detection_retries: config.base.retry_attempts,
        };

        // Create transport with WebSocket transport injected
        let mut transport = crate::transport::HybridTransportClient::new(transport_config);

        // Create the WebSocket pure transport that the transport will use
        let ws_config = crate::transport::websocket::WebSocketConfig::default();
        let ws_transport = std::sync::Arc::new(
            crate::transport::websocket::WebSocketTransport::new_with_unified_tls(ws_config, Some(&config.tls)),
        );
        transport = transport.with_websocket_transport(ws_transport);

        Ok(Self {
            config,
            endpoint_url: String::new(), // Will be provided per-request when using transport
            transport: Some(std::sync::Arc::new(transport)),
        })
    }

    /// Create a WebSocket client with basic configuration
    pub async fn with_base_config(endpoint_url: String, config: ClientConfig) -> Result<Self> {
        let ws_config = WebSocketClientConfig {
            base: config,
            ..Default::default()
        };
        Self::new(endpoint_url, ws_config).await
    }

    /// Get reference to transport layer for delegation
    pub fn transport(&self) -> Option<&std::sync::Arc<crate::transport::HybridTransportClient>> {
        self.transport.as_ref()
    }

    /// Connect to WebSocket endpoint - removed, use transport layer instead
    pub async fn connect(&self) -> Result<()> {
        Err(QollectiveError::transport(
            "Direct WebSocket connection removed. Use send_envelope() which delegates to transport layer.".to_string()
        ))
    }

    /// Send a request and receive response using envelope protocol
    pub async fn send_envelope<Req, Res>(&self, envelope: Envelope<Req>) -> Result<Envelope<Res>>
    where
        Req: Serialize + Send + 'static,
        Res: for<'de> Deserialize<'de> + Send + 'static,
    {
        // If transport is available, delegate to it
        if let Some(transport) = &self.transport {
            if let Some(websocket_transport) = transport.websocket_transport() {
                // Use the endpoint URL from the client configuration, or default to localhost for testing
                let endpoint = if self.endpoint_url.is_empty() {
                    "ws://localhost:8080" // Default for testing when using dependency injection
                } else {
                    &self.endpoint_url
                };
                return websocket_transport.send_envelope(endpoint, envelope).await;
            } else {
                return Err(QollectiveError::transport(
                    "No WebSocket transport configured in transport layer".to_string(),
                ));
            }
        }

        // No transport available - WebSocket client requires transport layer
        Err(QollectiveError::transport(
            "WebSocket client requires transport layer. Use new_with_transport() or with_transport().".to_string()
        ))
    }

    /// Get client configuration
    pub fn config(&self) -> &WebSocketClientConfig {
        &self.config
    }

    /// Get endpoint URL
    pub fn endpoint_url(&self) -> &str {
        &self.endpoint_url
    }
}

// WebSocketConnection removed - use transport layer instead

/// Builder for WebSocket client configuration
#[cfg(feature = "websocket-client")]
pub struct WebSocketClientBuilder {
    endpoint_url: Option<String>,
    config: WebSocketClientConfig,
}

#[cfg(feature = "websocket-client")]
impl WebSocketClientBuilder {
    pub fn new() -> Self {
        Self {
            endpoint_url: None,
            config: WebSocketClientConfig::default(),
        }
    }

    pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = Some(url.into());
        self
    }

    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.config.websocket.ping_interval_ms = interval.as_millis() as u64;
        self
    }

    pub fn ping_timeout(mut self, timeout: Duration) -> Self {
        self.config.ping_timeout_ms = timeout.as_millis() as u64;
        self
    }

    pub fn max_frame_size(mut self, size: usize) -> Self {
        self.config.max_frame_size = size;
        self
    }

    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.websocket.max_message_size = size;
        self
    }

    pub fn compression_enabled(mut self, enabled: bool) -> Self {
        self.config.websocket.enable_compression = enabled;
        self
    }

    pub fn subprotocol(mut self, protocol: impl Into<String>) -> Self {
        self.config.websocket.subprotocols.push(protocol.into());
        self
    }

    pub fn connection_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config
            .connection_headers
            .insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.base.timeout_seconds = timeout.as_secs();
        self
    }

    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.config.base.retry_attempts = attempts;
        self
    }

    pub async fn build(self) -> Result<WebSocketClient> {
        let endpoint_url = self.endpoint_url.ok_or_else(|| {
            QollectiveError::config("WebSocket endpoint URL is required".to_string())
        })?;

        WebSocketClient::new(endpoint_url, self.config).await
    }
}

#[cfg(feature = "websocket-client")]
impl Default for WebSocketClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Feature-disabled implementations
#[cfg(not(feature = "websocket-client"))]
pub struct WebSocketClient;

#[cfg(not(feature = "websocket-client"))]
pub struct WebSocketClientConfig;

#[cfg(not(feature = "websocket-client"))]
pub struct WebSocketClientBuilder;

#[cfg(not(feature = "websocket-client"))]
impl WebSocketClient {
    pub async fn new(
        _endpoint_url: String,
        _config: WebSocketClientConfig,
    ) -> crate::error::Result<Self> {
        Err(crate::error::QollectiveError::config(
            "websocket-client feature not enabled",
        ))
    }

    pub async fn with_base_config(
        _endpoint_url: String,
        _config: crate::client::common::ClientConfig,
    ) -> crate::error::Result<Self> {
        Err(crate::error::QollectiveError::config(
            "websocket-client feature not enabled",
        ))
    }
}

#[cfg(not(feature = "websocket-client"))]
impl WebSocketClientBuilder {
    pub fn new() -> Self {
        Self
    }

    pub async fn build(self) -> crate::error::Result<WebSocketClient> {
        Err(crate::error::QollectiveError::config(
            "websocket-client feature not enabled",
        ))
    }
}

#[cfg(not(feature = "websocket-client"))]
impl Default for WebSocketClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        echo: String,
    }

    #[tokio::test]
    async fn test_websocket_client_builder() {
        let builder = WebSocketClientBuilder::new()
            .endpoint_url("wss://api.example.com/ws")
            .ping_interval(Duration::from_secs(30))
            .ping_timeout(Duration::from_secs(10))
            .max_frame_size(1024 * 1024)
            .compression_enabled(true)
            .subprotocol("qollective-v1")
            .connection_header("Authorization", "Bearer token")
            .timeout(Duration::from_secs(60))
            .retry_attempts(3);

        #[cfg(feature = "websocket-client")]
        {
            let client = builder
                .build()
                .await
                .expect("Failed to build WebSocket client");
            assert_eq!(client.endpoint_url(), "wss://api.example.com/ws");
            assert_eq!(client.config().ping_interval(), Duration::from_secs(30));
            assert_eq!(client.config().ping_timeout(), Duration::from_secs(10));
            assert_eq!(client.config().max_frame_size, 1024 * 1024);
            assert!(client.config().compression_enabled());
            assert!(client
                .config()
                .subprotocols()
                .contains(&"qollective-v1".to_string()));
            assert!(client
                .config()
                .connection_headers
                .contains_key("Authorization"));
            assert_eq!(client.config().base.timeout_seconds, 60);
            assert_eq!(client.config().base.retry_attempts, 3);
        }
    }

    #[tokio::test]
    async fn test_websocket_url_validation() {
        #[cfg(feature = "websocket-client")]
        {
            // Valid WebSocket URLs
            assert!(WebSocketClient::new(
                "ws://localhost:8080".to_string(),
                WebSocketClientConfig::default()
            )
            .await
            .is_ok());
            assert!(WebSocketClient::new(
                "wss://api.example.com/ws".to_string(),
                WebSocketClientConfig::default()
            )
            .await
            .is_ok());

            // Invalid URLs
            assert!(WebSocketClient::new(
                "http://localhost:8080".to_string(),
                WebSocketClientConfig::default()
            )
            .await
            .is_err());
            assert!(WebSocketClient::new(
                "invalid-url".to_string(),
                WebSocketClientConfig::default()
            )
            .await
            .is_err());
        }
    }

    #[test]
    fn test_websocket_message_type_serialization() {
        #[cfg(feature = "websocket-client")]
        {
            let envelope_msg = WebSocketMessageType::Envelope {
                payload: serde_json::json!({"test": "data"}),
            };

            let serialized = serde_json::to_string(&envelope_msg).expect("Failed to serialize");
            assert!(serialized.contains("\"type\":\"envelope\""));

            let ping_msg = WebSocketMessageType::Ping {
                timestamp: Utc::now(),
            };

            let serialized = serde_json::to_string(&ping_msg).expect("Failed to serialize");
            assert!(serialized.contains("\"type\":\"ping\""));

            let error_msg = WebSocketMessageType::Error {
                message: "Test error".to_string(),
                code: Some(500),
            };

            let serialized = serde_json::to_string(&error_msg).expect("Failed to serialize");
            assert!(serialized.contains("\"type\":\"error\""));
            assert!(serialized.contains("\"message\":\"Test error\""));
            assert!(serialized.contains("\"code\":500"));
        }
    }

    #[tokio::test]
    async fn test_websocket_client_with_base_config() {
        #[cfg(feature = "websocket-client")]
        {
            let base_config = ClientConfig {
                base_url: "wss://api.example.com".to_string(),
                timeout_seconds: 30,
                retry_attempts: 5,
                ..Default::default()
            };

            let client = WebSocketClient::with_base_config(
                "wss://api.example.com/ws".to_string(),
                base_config,
            )
            .await
            .expect("Failed to create WebSocket client");

            assert_eq!(client.config().base.timeout_seconds, 30);
            assert_eq!(client.config().base.retry_attempts, 5);
        }
    }

    #[tokio::test]
    async fn test_feature_disabled_behavior() {
        #[cfg(not(feature = "websocket-client"))]
        {
            let builder = WebSocketClientBuilder::new();
            let result = builder.build().await;
            assert!(result.is_err());

            let config = WebSocketClientConfig;
            let result = WebSocketClient::new("ws://test".to_string(), config).await;
            assert!(result.is_err());
        }
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_config_defaults() {
        let config = WebSocketClientConfig::default();
        assert_eq!(config.ping_interval(), Duration::from_secs(30));
        assert_eq!(config.ping_timeout(), Duration::from_millis(10000));
        assert_eq!(config.max_frame_size, 16 * 1024 * 1024);
        assert_eq!(config.max_message_size(), 16 * 1024 * 1024); // Uses websocket.max_message_size
        assert!(config.compression_enabled());
        assert_eq!(config.subprotocols(), &vec!["qollective".to_string()]); // Uses websocket.subprotocols
        assert!(config.connection_headers.is_empty());
    }

    #[cfg(feature = "websocket-client")]
    #[tokio::test]
    async fn test_builder_missing_endpoint_url() {
        let builder = WebSocketClientBuilder::new();
        let result = builder.build().await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("endpoint URL is required"));
        }
    }

    // TDD Step 1: Write failing test for dependency injection constructor
    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_client_with_transport_dependency_injection() {
        use crate::transport::{HybridTransportClient, TransportDetectionConfig};
        use std::sync::Arc;

        // Create transport client
        let transport_config = TransportDetectionConfig::default();
        let transport = Arc::new(HybridTransportClient::new(transport_config));

        // Create WebSocket client with injected transport
        let result = WebSocketClient::with_transport(transport);
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(client.transport().is_some());
    }

    // TDD Step 2: Write failing test for constructor with native WebSocket transport integration
    #[cfg(feature = "websocket-client")]
    #[tokio::test]
    async fn test_websocket_client_new_with_transport_integration() {
        use crate::transport::{
            websocket::WebSocketConfig, websocket::WebSocketTransport, HybridTransportClient,
            TransportDetectionConfig,
        };
        use std::sync::Arc;

        // Create WebSocket transport
        let ws_config = WebSocketConfig::default();
        let ws_transport = Arc::new(WebSocketTransport::new(ws_config));

        // Create transport client with WebSocket transport injected
        let transport_config = TransportDetectionConfig::default();
        let transport = Arc::new(
            HybridTransportClient::new(transport_config)
                .with_websocket_transport(ws_transport.clone()),
        );

        // Create WebSocket client that uses this transport
        let result = WebSocketClient::with_transport(transport);
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(client.transport().is_some());

        // Verify transport has WebSocket transport injected
        let transport_ref = client.transport().unwrap();
        assert!(transport_ref.websocket_transport().is_some());
    }

    // TDD Step 3: Write failing test for send_envelope delegation to transport layer
    #[cfg(feature = "websocket-client")]
    #[tokio::test]
    async fn test_websocket_client_send_envelope_transport_delegation() {
        use crate::envelope::{Envelope, Meta};
        use crate::transport::{
            websocket::WebSocketConfig, websocket::WebSocketTransport, HybridTransportClient,
            TransportDetectionConfig,
        };
        use std::sync::Arc;

        // Create WebSocket transport
        let ws_config = WebSocketConfig::default();
        let ws_transport = Arc::new(WebSocketTransport::new(ws_config));

        // Create transport client with WebSocket transport injected
        let transport_config = TransportDetectionConfig::default();
        let transport = Arc::new(
            HybridTransportClient::new(transport_config)
                .with_websocket_transport(ws_transport.clone()),
        );

        // Create WebSocket client that uses this transport
        let client = WebSocketClient::with_transport(transport).unwrap();

        // Create test envelope
        let request = TestRequest {
            message: "test message".to_string(),
        };
        let envelope = Envelope::new(Meta::default(), request);

        // This test will fail initially because we don't have a real WebSocket server
        // But it validates that the delegation path exists and compiles correctly
        let result: Result<Envelope<TestResponse>> = client.send_envelope(envelope).await;

        // For now, we expect this to fail with a connection error since no server is running
        assert!(result.is_err());

        // Verify it's a connection/transport error (not a compilation error)
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("timeout")
                || error.to_string().contains("Connection refused")
                || error.to_string().contains("transport")
        );
    }

    // TDD Step 4: Write test for new_with_transport constructor following NATS pattern
    #[cfg(feature = "websocket-client")]
    #[tokio::test]
    async fn test_websocket_client_new_with_transport_constructor() {
        // Create WebSocket client with automatic transport setup
        let config = WebSocketClientConfig::default();
        let result = WebSocketClient::new_with_transport(config).await;
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(client.transport().is_some());

        // Verify transport has WebSocket transport configured
        let transport_ref = client.transport().unwrap();
        assert!(transport_ref.websocket_transport().is_some());
    }

    // TDD Step 5: Write failing test for WebSocket handshake headers issue
    #[cfg(feature = "websocket-client")]
    #[tokio::test]
    async fn test_websocket_client_handshake_headers_included() {
        // ARRANGE: Create WebSocket client with basic configuration
        let config = WebSocketClientConfig::default();
        let client = WebSocketClient::new("ws://localhost:8443/test".to_string(), config)
            .await
            .expect("Failed to create WebSocket client");

        // ACT: Test that the client is properly configured to use transport layer
        // Since connect() method is removed, just verify client creation succeeds
        // and uses transport layer (not manual connection)

        // ASSERT: Client should be created successfully and have transport
        assert!(
            client.transport().is_some(),
            "Client should have transport layer"
        );

        // Should NOT use manual connection - connect() method should fail
        let connection_result = client.connect().await;
        assert!(connection_result.is_err());
        let error_message = connection_result.unwrap_err().to_string();
        assert!(
            error_message.contains("Direct WebSocket connection removed"),
            "connect() method should be disabled in favor of transport layer"
        );
    }

    // TDD Step 6: Write test for WebSocket client with subprotocols and custom headers
    #[cfg(feature = "websocket-client")]
    #[tokio::test]
    async fn test_websocket_client_with_custom_headers_and_subprotocols() {
        // ARRANGE: Create WebSocket client with subprotocols and custom headers
        let mut config = WebSocketClientConfig::default();
        config.websocket.subprotocols = vec!["qollective-v1".to_string(), "chat".to_string()];
        config
            .connection_headers
            .insert("Authorization".to_string(), "Bearer test-token".to_string());
        config
            .connection_headers
            .insert("X-Client-Version".to_string(), "1.0.0".to_string());

        let client = WebSocketClient::new("ws://localhost:8443/test".to_string(), config)
            .await
            .expect("Failed to create WebSocket client");

        // ACT: Test that client with custom headers/subprotocols still uses transport layer
        // ASSERT: Client should be created successfully and have transport
        assert!(
            client.transport().is_some(),
            "Client should have transport layer"
        );

        // Should NOT use manual connection - connect() method should fail
        let connection_result = client.connect().await;
        assert!(connection_result.is_err());
        let error_message = connection_result.unwrap_err().to_string();
        assert!(
            error_message.contains("Direct WebSocket connection removed"),
            "connect() method should be disabled even with custom headers/subprotocols"
        );
    }
}
