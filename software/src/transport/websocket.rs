// ABOUTME:SocketConfig transport implementation for envelope-to-WebSocket message conversion
// ABOUTME: Enables real-time bidirectional communication with WebSocket protocol support

//!SocketConfig transport implementation for Step 17: CreateSocketConfig Transport.
//!
//! This module provides a dedicated WebSocket transport that handles envelope ↔ WebSocket
//! message conversion, manages WebSocket connections, and supports WebSocket-specific
//! features like ping/pong frames and connection lifecycle management.
//!
//! Key features:
//! - Envelope ↔ WebSocket message conversion
//! - WebSocket protocol compliance (ping/pong, close frames)
//! - Connection lifecycle management
//! - Subprotocol and extension support
//! - Real-time bidirectional communication

use crate::envelope::Envelope;
use crate::error::{QollectiveError, Result};
use crate::traits::senders::UnifiedEnvelopeSender;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(feature = "websocket-client")]
use futures_util::{SinkExt, StreamExt};
#[cfg(feature = "websocket-client")]
use std::sync::Arc;
#[cfg(feature = "websocket-client")]
use tokio::net::TcpStream;
#[cfg(feature = "websocket-client")]
use tokio::sync::Mutex;
#[cfg(feature = "websocket-client")]
use tokio_tungstenite::tungstenite::{protocol::CloseFrame, Message};
#[cfg(feature = "websocket-client")]
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
#[cfg(feature = "websocket-client")]
use url::Url;

///SocketConfig transport configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Message timeout
    pub message_timeout: Duration,
    /// Ping interval for keep-alive
    pub ping_interval: Duration,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Supported subprotocols
    pub subprotocols: Vec<String>,
    /// Enable compression
    pub enable_compression: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            message_timeout: Duration::from_secs(10),
            ping_interval: Duration::from_secs(30),
            max_message_size: 16 * 1024 * 1024, // 16MB
            subprotocols: vec!["qollective.v1".to_string()],
            enable_compression: true,
        }
    }
}

///SocketConfig transport implementation
#[derive(Debug)]
pub struct WebSocketTransport {
    /// Transport configuration
    config: WebSocketConfig,
    /// TLS configuration for secure connections
    #[cfg(feature = "tls")]
    tls_config: Option<crate::config::tls::TlsConfig>,
    /// Active connections (endpoint -> connection)
    #[cfg(feature = "websocket-client")]
    connections: Arc<
        Mutex<
            std::collections::HashMap<
                String,
                Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
            >,
        >,
    >,
}

impl WebSocketTransport {
    /// Create a newSocketConfig transport
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "tls")]
            tls_config: None,
            #[cfg(feature = "websocket-client")]
            connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Create a new WebSocket transport with unified TLS configuration
    pub fn new_with_unified_tls(
        config: WebSocketConfig,
        tls_config: Option<&crate::config::tls::TlsConfig>,
    ) -> Self {
        Self {
            config,
            #[cfg(feature = "tls")]
            tls_config: tls_config.cloned(),
            #[cfg(feature = "websocket-client")]
            connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Parse WebSocket URL and validate scheme
    #[cfg(feature = "websocket-client")]
    pub fn parse_websocket_url(&self, endpoint: &str) -> Result<Url> {
        let url = Url::parse(endpoint)
            .map_err(|e| QollectiveError::validation(format!("Invalid WebSocket URL: {}", e)))?;

        match url.scheme() {
            "ws" | "wss" => Ok(url),
            "http" => {
                let ws_url = endpoint.replace("http://", "ws://");
                Url::parse(&ws_url).map_err(|e| {
                    QollectiveError::validation(format!("Invalid converted WebSocket URL: {}", e))
                })
            }
            "https" => {
                let wss_url = endpoint.replace("https://", "wss://");
                Url::parse(&wss_url).map_err(|e| {
                    QollectiveError::validation(format!("Invalid converted WebSocket URL: {}", e))
                })
            }
            _ => Err(QollectiveError::validation(format!(
                "Unsupported scheme for WebSocket: {}. Expected ws, wss, http, or https",
                url.scheme()
            ))),
        }
    }

    /// Establish WebSocket connection
    #[cfg(feature = "websocket-client")]
    async fn establish_connection(
        &self,
        endpoint: &str,
    ) -> Result<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>> {
        let url = self.parse_websocket_url(endpoint)?;

        // Check if connection already exists
        {
            let connections = self.connections.lock().await;
            if let Some(connection) = connections.get(endpoint) {
                return Ok(connection.clone());
            }
        }

        // Determine if TLS is enabled and create connector
        let connector = self.create_tls_connector().await?;

        // Establish new connection with timeout
        let connection_result = tokio::time::timeout(
            self.config.connection_timeout,
            self.connect_with_tls(url.as_str(), connector),
        )
        .await;

        let (ws_stream, _response) = connection_result
            .map_err(|_| QollectiveError::transport("WebSocket connection timeout".to_string()))?
            .map_err(|e| {
                QollectiveError::connection(format!("WebSocket connection failed: {}", e))
            })?;

        let connection = Arc::new(Mutex::new(ws_stream));

        // Cache the connection
        {
            let mut connections = self.connections.lock().await;
            connections.insert(endpoint.to_string(), connection.clone());
        }

        Ok(connection)
    }

    /// Create TLS connector based on configuration
    #[cfg(feature = "websocket-client")]
    async fn create_tls_connector(&self) -> Result<Option<Connector>> {
        #[cfg(feature = "tls")]
        {
            if let Some(tls_config) = &self.tls_config {
                if !tls_config.enabled {
                    return Ok(None);
                }

                // Create rustls ClientConfig using our unified TLS configuration
                let client_config = tls_config.create_client_config().await?;

                // Convert to Arc<rustls::ClientConfig> for tokio-tungstenite
                let connector = Connector::Rustls(client_config);
                return Ok(Some(connector));
            }
        }

        // No TLS config or TLS feature disabled - use default behavior
        Ok(None)
    }

    /// Connect with TLS support
    #[cfg(feature = "websocket-client")]
    async fn connect_with_tls(
        &self,
        url: &str,
        connector: Option<Connector>,
    ) -> Result<(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::handshake::client::Response,
    )> {
        use tokio_tungstenite::{connect_async, connect_async_tls_with_config};

        match connector {
            Some(connector) => {
                // Use custom TLS connector
                connect_async_tls_with_config(url, None, false, Some(connector))
                    .await
                    .map_err(|e| {
                        QollectiveError::connection(format!(
                            "WebSocket TLS connection failed: {}",
                            e
                        ))
                    })
            }
            None => {
                // Use default connection (may still be TLS if wss:// scheme)
                connect_async(url).await.map_err(|e| {
                    QollectiveError::connection(format!("WebSocket connection failed: {}", e))
                })
            }
        }
    }

    /// Convert envelope to WebSocket message
    fn envelope_to_websocket_message<T>(&self, envelope: Envelope<T>) -> Result<Message>
    where
        T: Serialize,
    {
        // First convert envelope to JSON Value
        let envelope_value = serde_json::to_value(&envelope).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize envelope: {}", e))
        })?;

        // Wrap the envelope data in WebSocketMessageType::Envelope format
        let websocket_message = crate::client::websocket::WebSocketMessageType::Envelope {
            payload: envelope_value,
        };

        // Serialize the wrapped message
        let json_data = serde_json::to_string(&websocket_message).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize WebSocket message: {}", e))
        })?;

        // Check message size
        if json_data.len() > self.config.max_message_size {
            return Err(QollectiveError::validation(format!(
                "Message size {} exceeds maximum {}",
                json_data.len(),
                self.config.max_message_size
            )));
        }

        Ok(Message::Text(json_data.into()))
    }

    /// Convert WebSocket message to envelope
    fn websocket_message_to_envelope<R>(&self, message: Message) -> Result<Envelope<R>>
    where
        R: for<'de> Deserialize<'de>,
    {
        let text_data = match message {
            Message::Text(text) => text.to_string(),
            Message::Binary(data) => String::from_utf8(data.to_vec()).map_err(|e| {
                QollectiveError::deserialization(format!("Invalid UTF-8 in binary message: {}", e))
            })?,
            Message::Close(_) => {
                return Err(QollectiveError::connection(
                    "WebSocket connection closed by remote",
                ));
            }
            Message::Ping(_) | Message::Pong(_) => {
                return Err(QollectiveError::transport(
                    "Received ping/pong frame instead of data".to_string(),
                ));
            }
            Message::Frame(_) => {
                return Err(QollectiveError::transport(
                    "Received raw frame instead of data".to_string(),
                ));
            }
        };

        // First parse as WebSocketMessageType to unwrap the envelope
        let websocket_message: crate::client::websocket::WebSocketMessageType =
            serde_json::from_str(&text_data).map_err(|e| {
                QollectiveError::deserialization(format!(
                    "Failed to deserialize WebSocket message: {}",
                    e
                ))
            })?;

        // Extract the envelope data from the WebSocketMessageType
        let websocket_message_type_value = match websocket_message {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => payload,
            crate::client::websocket::WebSocketMessageType::Ping { .. } => {
                return Err(QollectiveError::transport(
                    "Received ping message instead of envelope".to_string(),
                ));
            }
            crate::client::websocket::WebSocketMessageType::Pong { .. } => {
                return Err(QollectiveError::transport(
                    "Received pong message instead of envelope".to_string(),
                ));
            }
            crate::client::websocket::WebSocketMessageType::Error { message, code } => {
                return Err(QollectiveError::transport(format!(
                    "Received WebSocket error message: {} (code: {:?})",
                    message, code
                )));
            }
        };

        // Now deserialize the envelope data as a Qollective envelope
        let envelope: Envelope<R> = serde_json::from_value(websocket_message_type_value).map_err(|e| {
            QollectiveError::deserialization(format!("Failed to deserialize envelope: {}", e))
        })?;

        Ok(envelope)
    }

    /// Send ping frame to keep connection alive
    #[cfg(feature = "websocket-client")]
    async fn send_ping(
        &self,
        connection: &Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    ) -> Result<()> {
        let mut ws_stream = connection.lock().await;
        ws_stream
            .send(Message::Ping(vec![].into()))
            .await
            .map_err(|e| QollectiveError::connection(format!("Failed to send ping: {}", e)))?;
        Ok(())
    }

    /// Close WebSocket connection
    #[cfg(feature = "websocket-client")]
    pub async fn close_connection(&self, endpoint: &str) -> Result<()> {
        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.remove(endpoint) {
            let mut ws_stream = connection.lock().await;
            let close_frame = CloseFrame {
                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                reason: "Closing connection".into(),
            };
            ws_stream
                .send(Message::Close(Some(close_frame)))
                .await
                .map_err(|e| {
                    QollectiveError::connection(format!("Failed to close connection: {}", e))
                })?;
        }
        Ok(())
    }
}

#[cfg(feature = "websocket-client")]
#[async_trait]
impl<T, R> UnifiedEnvelopeSender<T, R> for WebSocketTransport
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    async fn send_envelope(&self, endpoint: &str, envelope: Envelope<T>) -> Result<Envelope<R>> {
        // Establish connection
        let connection = self.establish_connection(endpoint).await?;

        // Convert envelope to WebSocket message
        let request_message = self.envelope_to_websocket_message(envelope)?;

        // Send message with timeout
        {
            let mut ws_stream = connection.lock().await;
            tokio::time::timeout(self.config.message_timeout, ws_stream.send(request_message))
                .await
                .map_err(|_| QollectiveError::transport("WebSocket send timeout".to_string()))?
                .map_err(|e| {
                    QollectiveError::connection(format!("Failed to send WebSocket message: {}", e))
                })?;
        }

        // Wait for response with timeout
        let response_message = tokio::time::timeout(self.config.message_timeout, async {
            let mut ws_stream = connection.lock().await;
            ws_stream.next().await
        })
        .await
        .map_err(|_| QollectiveError::transport("WebSocket receive timeout".to_string()))?
        .ok_or_else(|| QollectiveError::connection("WebSocket stream ended unexpectedly"))?
        .map_err(|e| QollectiveError::connection(format!("WebSocket receive error: {}", e)))?;

        // Convert response message to envelope
        let response_envelope = self.websocket_message_to_envelope(response_message)?;

        Ok(response_envelope)
    }
}

// Compile stub for when websocket-client feature is not enabled
#[cfg(not(feature = "websocket-client"))]
#[async_trait]
impl<T, R> UnifiedEnvelopeSender<T, R> for WebSocketTransport
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    async fn send_envelope(&self, _endpoint: &str, _envelope: Envelope<T>) -> Result<Envelope<R>> {
        Err(QollectiveError::configuration(
            "WebSocket transport not available: websocket-client feature not enabled",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Meta;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        result: String,
    }

    #[test]
    fn test_websocket_transport_creation() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);
        assert_eq!(transport.config.connection_timeout, Duration::from_secs(30));
        assert_eq!(transport.config.message_timeout, Duration::from_secs(10));
        assert_eq!(transport.config.max_message_size, 16 * 1024 * 1024);
    }

    // TDD Step 2: Write failing test for WebSocket URL parsing
    #[cfg(feature = "websocket-client")]
    #[test]
    fn test_websocket_url_parsing() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);

        // Test valid WebSocket URLs
        assert!(transport.parse_websocket_url("ws://localhost:8080").is_ok());
        assert!(transport
            .parse_websocket_url("wss://example.com/websocket")
            .is_ok());

        // Test HTTP to WebSocket conversion
        let http_result = transport.parse_websocket_url("http://localhost:8080");
        assert!(http_result.is_ok());

        // Test HTTPS to WebSocket conversion
        let https_result = transport.parse_websocket_url("https://example.com");
        assert!(https_result.is_ok());

        // Test invalid URLs
        assert!(transport.parse_websocket_url("ftp://example.com").is_err());
        assert!(transport.parse_websocket_url("invalid-url").is_err());
    }

    // TDD Step 3: Write failing test for envelope to WebSocket message conversion
    #[test]
    fn test_envelope_to_websocket_message_conversion() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);

        let request = TestRequest {
            message: "Hello WebSocket".to_string(),
        };
        let envelope = Envelope::new(Meta::default(), request);

        let result = transport.envelope_to_websocket_message(envelope);
        assert!(result.is_ok());

        let message = result.unwrap();
        match message {
            Message::Text(text) => {
                assert!(text.contains("Hello WebSocket"));
                assert!(text.contains("\"payload\""));
                assert!(text.contains("\"meta\""));
            }
            _ => panic!("Expected text message"),
        }
    }

    // TDD Step 4: Write failing test for WebSocket message to envelope conversion
    #[test]
    fn test_websocket_message_to_envelope_conversion() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);

        // Create a JSON message that represents a WebSocketMessageType::Envelope containing an envelope
        // This is the format that the websocket_message_to_envelope method expects (tagged enum with "type" field)
        let envelope_data = r#"{"meta":{"id":"test-id","timestamp":"2023-01-01T00:00:00Z","version":"1.0","context":{},"routing":{"source":"test","destination":"test","protocol":"websocket"}},"payload":{"result":"Hello from WebSocket"}}"#;
        let websocket_message_json = format!(r#"{{"type":"envelope","payload":{}}}"#, envelope_data);
        let message = Message::Text(websocket_message_json.into());

        let result: Result<Envelope<TestResponse>> =
            transport.websocket_message_to_envelope(message);
        assert!(result.is_ok());

        let envelope = result.unwrap();
        let (_, response) = envelope.extract();
        assert_eq!(response.result, "Hello from WebSocket");
    }

    // TDD Step 5: Write failing test for large message rejection
    #[test]
    fn test_large_message_rejection() {
        let mut config = WebSocketConfig::default();
        config.max_message_size = 100; // Very small limit
        let transport = WebSocketTransport::new(config);

        let large_request = TestRequest {
            message: "x".repeat(1000), // Large message
        };
        let envelope = Envelope::new(Meta::default(), large_request);

        let result = transport.envelope_to_websocket_message(envelope);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    // TDD Step 6: Write failing test for WebSocket protocol message handling
    #[test]
    fn test_websocket_protocol_message_handling() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);

        // Test close frame handling
        let close_message = Message::Close(None);
        let result: Result<Envelope<TestResponse>> =
            transport.websocket_message_to_envelope(close_message);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("connection closed"));

        // Test ping frame handling
        let ping_message = Message::Ping(vec![].into());
        let result: Result<Envelope<TestResponse>> =
            transport.websocket_message_to_envelope(ping_message);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ping/pong frame"));

        // Test pong frame handling
        let pong_message = Message::Pong(vec![].into());
        let result: Result<Envelope<TestResponse>> =
            transport.websocket_message_to_envelope(pong_message);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ping/pong frame"));
    }

    // TDD Step 7: Write failing test for UnifiedEnvelopeSender trait implementation
    #[tokio::test]
    async fn test_unified_envelope_sender_trait_implementation() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);

        let request = TestRequest {
            message: "test message".to_string(),
        };
        let envelope = Envelope::new(Meta::default(), request);

        // This test will fail initially because we don't have a real WebSocket server
        // But it validates the trait implementation compiles correctly
        let result: Result<Envelope<TestResponse>> = transport
            .send_envelope("ws://localhost:8080", envelope)
            .await;

        // For now, we expect this to fail with a connection error since no server is running
        assert!(result.is_err());

        // Verify it's a connection error (not a compilation error)
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("timeout")
                || error.to_string().contains("Connection refused")
        );
    }

    // TDD Step 8: Test TLS integration with unified configuration
    #[tokio::test]
    async fn test_tls_integration_with_unified_config() {
        use crate::config::tls::{TlsConfig, VerificationMode};

        let config = WebSocketConfig::default();

        // Test with TLS disabled
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = false;
        tls_config.verification_mode = VerificationMode::SystemCa;

        let transport_disabled =
            WebSocketTransport::new_with_unified_tls(config.clone(), Some(&tls_config));

        // Test TLS connector creation (should return None for disabled TLS)
        let connector = transport_disabled.create_tls_connector().await;
        assert!(connector.is_ok());
        assert!(connector.unwrap().is_none());

        // Test with TLS enabled but Skip verification mode
        let mut tls_config_skip = TlsConfig::default();
        tls_config_skip.enabled = true;
        tls_config_skip.verification_mode = VerificationMode::Skip;

        let transport_skip =
            WebSocketTransport::new_with_unified_tls(config.clone(), Some(&tls_config_skip));

        // Test TLS connector creation with Skip mode
        let connector_skip = transport_skip.create_tls_connector().await;
        assert!(connector_skip.is_ok());
        assert!(connector_skip.unwrap().is_some());

        // Test with TLS enabled and SystemCa verification mode
        let mut tls_config_system = TlsConfig::default();
        tls_config_system.enabled = true;
        tls_config_system.verification_mode = VerificationMode::SystemCa;

        let transport_system =
            WebSocketTransport::new_with_unified_tls(config.clone(), Some(&tls_config_system));

        // Test TLS connector creation with SystemCa mode
        let connector_system = transport_system.create_tls_connector().await;
        assert!(connector_system.is_ok());
        assert!(connector_system.unwrap().is_some());
    }

    // TDD Step 9: Test WebSocket with TLS connections (will fail without server)
    #[tokio::test]
    async fn test_websocket_with_tls_connections() {
        use crate::config::tls::{TlsConfig, VerificationMode};
        use std::time::Duration;

        // Use shorter timeout for testing
        let mut config = WebSocketConfig::default();
        config.connection_timeout = Duration::from_millis(100);
        config.message_timeout = Duration::from_millis(100);

        // Test with Skip verification mode for self-signed certificates
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = true;
        tls_config.verification_mode = VerificationMode::Skip;

        let transport = WebSocketTransport::new_with_unified_tls(config, Some(&tls_config));

        let request = TestRequest {
            message: "test TLS message".to_string(),
        };
        let envelope = Envelope::new(Meta::default(), request);

        // Test WSS connection (will fail without server but should use TLS)
        let result: Result<Envelope<TestResponse>> = transport
            .send_envelope("wss://localhost:19999", envelope)
            .await;

        // Expect connection error since no server is running
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        // The test is successful if we get any of these expected error conditions:
        // - Connection refused (no server running)
        // - Connection timeout
        // - WebSocket error (if some server is running but not handling our request properly)
        assert!(
            error_str.contains("connection")
                || error_str.contains("timeout")
                || error_str.contains("Connection refused")
                || error_str.contains("os error 61") // Connection refused on macOS
                || error_str.contains("os error 111") // Connection refused on Linux
                || error_str.contains("No handler found") // WebSocket connection established but no handler
                || error_str.contains("transport error") // General transport error
        );
    }
}
