// ABOUTME: WASM WebSocket client implementation
// ABOUTME: Provides WebSocket communication for real-time envelope messaging in browsers

//! WASM WebSocket client implementation.
//!
//! This module provides WebSocket communication capabilities for WASM applications
//! while maintaining envelope patterns and connection management.

use crate::config::websocket::WebSocketClientConfig;
use crate::constants::{limits, timeouts};
use crate::error::{QollectiveError, Result};
use crate::wasm::crypto::{WasmCertificateManager, CertificateBundle};
use crate::wasm::js_types::{WasmEnvelope, WasmMeta};
use crate::wasm::rest::ConnectivityResult;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{WebSocket, MessageEvent, Event, CloseEvent, ErrorEvent};

/// WASM WebSocket client
#[derive(Debug, Clone)]
pub struct WasmWebSocketClient {
    config: WebSocketClientConfig,
    cert_manager: WasmCertificateManager,
}

/// WebSocket connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

/// WebSocket connection wrapper for WASM
#[derive(Debug)]
pub struct WasmWebSocketConnection {
    websocket: WebSocket,
    state: ConnectionState,
    url: String,
}

impl WasmWebSocketClient {
    /// Create new WASM WebSocket client
    pub fn new(
        config: WebSocketClientConfig,
        cert_manager: WasmCertificateManager,
    ) -> Result<Self> {
        // CONFIG FIRST PRINCIPLE - validate config
        if config.base.base_url.is_empty() {
            return Err(QollectiveError::validation("Base URL cannot be empty"));
        }

        Ok(Self {
            config,
            cert_manager,
        })
    }

    /// Send envelope via WebSocket
    pub async fn send_envelope(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        // Validate URL
        if url.is_empty() {
            return Err(QollectiveError::validation("URL cannot be empty"));
        }

        // Build full WebSocket URL
        let full_url = self.build_websocket_url(url)?;

        // Retry loop for connection and message sending
        let mut last_error = QollectiveError::transport("No attempts made".to_string());
        
        for attempt in 0..self.config.max_retry_attempts {
            match self.attempt_send_envelope(&full_url, &envelope).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = e;
                    
                    // If not the last attempt, wait before retrying
                    if attempt < self.config.max_retry_attempts - 1 {
                        let delay = self.calculate_retry_delay(attempt);
                        self.sleep_ms(delay).await;
                        
                        web_sys::console::warn_1(&format!(
                            "WebSocket request attempt {} failed, retrying in {}ms", 
                            attempt + 1, delay
                        ).into());
                    }
                }
            }
        }

        Err(last_error)
    }

    /// Attempt to send envelope via WebSocket (single attempt)
    async fn attempt_send_envelope(&self, url: &str, envelope: &WasmEnvelope) -> Result<WasmEnvelope> {
        // Create WebSocket connection
        let connection = self.create_connection(url).await?;
        
        // Create a unique request ID for this message
        let request_id = envelope.meta().request_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| format!("req_{}", js_sys::Date::now() as u64));

        // Send envelope as JSON message
        let json_message = serde_json::to_string(envelope)
            .map_err(|e| QollectiveError::serialization(format!("Failed to serialize envelope: {}", e)))?;

        connection.websocket.send_with_str(&json_message)
            .map_err(|_| QollectiveError::transport("Failed to send WebSocket message".to_string()))?;

        // Wait for response with timeout using JavaScript Promise
        let response = self.wait_for_response(&connection, &request_id).await?;
        
        Ok(response)
    }

    /// Create WebSocket connection
    async fn create_connection(&self, url: &str) -> Result<WasmWebSocketConnection> {
        // Create WebSocket with protocols
        let protocols = js_sys::Array::new();
        for protocol in self.config.subprotocols() {
            protocols.push(&JsValue::from_str(protocol));
        }

        let websocket = if protocols.length() > 0 {
            WebSocket::new_with_str_sequence(url, &protocols)
        } else {
            WebSocket::new(url)
        }.map_err(|_| QollectiveError::transport("Failed to create WebSocket".to_string()))?;

        // Set binary type to handle envelope messages
        websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let connection = WasmWebSocketConnection {
            websocket,
            state: ConnectionState::Connecting,
            url: url.to_string(),
            pending_messages: HashMap::new(),
        };

        // Wait for connection to open
        self.wait_for_connection(&connection).await?;

        Ok(connection)
    }

    /// Wait for WebSocket connection to open
    async fn wait_for_connection(&self, connection: &WasmWebSocketConnection) -> Result<()> {
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let resolve = Rc::new(RefCell::new(Some(resolve)));
            let reject = Rc::new(RefCell::new(Some(reject)));

            // Set up onopen handler
            let onopen_resolve = resolve.clone();
            let onopen = Closure::wrap(Box::new(move |_: Event| {
                if let Some(resolver) = onopen_resolve.borrow_mut().take() {
                    resolver.call0(&JsValue::NULL).unwrap();
                }
            }) as Box<dyn FnMut(Event)>);

            connection.websocket.set_onopen(Some(onopen.as_ref().unchecked_ref()));

            // Set up onerror handler
            let onerror_reject = reject.clone();
            let onerror = Closure::wrap(Box::new(move |_: ErrorEvent| {
                if let Some(rejector) = onerror_reject.borrow_mut().take() {
                    rejector.call1(&JsValue::NULL, &JsValue::from_str("WebSocket connection error")).unwrap();
                }
            }) as Box<dyn FnMut(ErrorEvent)>);

            connection.websocket.set_onerror(Some(onerror.as_ref().unchecked_ref()));

            // Set up timeout
            let timeout_reject = reject.clone();
            let timeout = self.config.connection_timeout_ms as i32;
            
            web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                &Closure::once_into_js(move || {
                    if let Some(rejector) = timeout_reject.borrow_mut().take() {
                        rejector.call1(&JsValue::NULL, &JsValue::from_str("WebSocket connection timeout")).unwrap();
                    }
                }),
                timeout,
            ).unwrap();

            // Keep closures alive
            onopen.forget();
            onerror.forget();
        });

        match JsFuture::from(promise).await {
            Ok(_) => Ok(()),
            Err(_) => Err(QollectiveError::transport("WebSocket connection failed".to_string())),
        }
    }

    /// Wait for response message with specified request ID
    async fn wait_for_response(&self, connection: &WasmWebSocketConnection, request_id: &str) -> Result<WasmEnvelope> {
        let request_id = request_id.to_string();
        
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let resolve = Rc::new(RefCell::new(Some(resolve)));
            let reject = Rc::new(RefCell::new(Some(reject)));

            // Set up message handler
            let onmessage_resolve = resolve.clone();
            let onmessage_reject = reject.clone();
            let msg_request_id = request_id.clone();
            
            let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                // Parse message data
                let message_data = if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                    text.as_string().unwrap_or_default()
                } else {
                    web_sys::console::error_1(&"Received non-text WebSocket message".into());
                    return;
                };

                // Try to parse as envelope
                match serde_json::from_str::<WasmEnvelope>(&message_data) {
                    Ok(envelope) => {
                        // Check if this response matches our request
                        if let Some(env_request_id) = envelope.meta().request_id() {
                            if env_request_id.to_string() == msg_request_id {
                                if let Some(resolver) = onmessage_resolve.borrow_mut().take() {
                                    let js_envelope = serde_wasm_bindgen::to_value(&envelope).unwrap();
                                    resolver.call1(&JsValue::NULL, &js_envelope).unwrap();
                                }
                            }
                        }
                    },
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to parse envelope: {}", e).into());
                        if let Some(rejector) = onmessage_reject.borrow_mut().take() {
                            rejector.call1(&JsValue::NULL, &JsValue::from_str(&format!("Parse error: {}", e))).unwrap();
                        }
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            connection.websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

            // Set up timeout
            let timeout_reject = reject.clone();
            let timeout = self.config.message_timeout_ms as i32;
            
            web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                &Closure::once_into_js(move || {
                    if let Some(rejector) = timeout_reject.borrow_mut().take() {
                        rejector.call1(&JsValue::NULL, &JsValue::from_str("WebSocket message timeout")).unwrap();
                    }
                }),
                timeout,
            ).unwrap();

            // Keep closure alive
            onmessage.forget();
        });

        match JsFuture::from(promise).await {
            Ok(js_value) => {
                let envelope: WasmEnvelope = serde_wasm_bindgen::from_value(js_value)
                    .map_err(|e| QollectiveError::deserialization(format!("Failed to deserialize response: {}", e)))?;
                Ok(envelope)
            },
            Err(_) => Err(QollectiveError::transport("WebSocket response failed".to_string())),
        }
    }

    /// Build full WebSocket URL from base URL and path
    fn build_websocket_url(&self, url: &str) -> Result<String> {
        if url.starts_with("ws://") || url.starts_with("wss://") {
            Ok(url.to_string())
        } else if url.starts_with("http://") {
            Ok(url.replace("http://", "ws://"))
        } else if url.starts_with("https://") {
            Ok(url.replace("https://", "wss://"))
        } else {
            // Build from base URL
            let base_url = &self.config.base.base_url;
            let websocket_url = if base_url.starts_with("https://") {
                base_url.replace("https://", "wss://")
            } else if base_url.starts_with("http://") {
                base_url.replace("http://", "ws://")
            } else {
                format!("wss://{}", base_url)
            };
            
            Ok(format!("{}{}", websocket_url.trim_end_matches('/'), url))
        }
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> u64 {
        let base_delay = 1000u64; // 1 second base delay
        let max_delay = 30000u64; // 30 seconds max delay
        
        let delay = base_delay * (2_u64.pow(attempt));
        delay.min(max_delay)
    }

    /// Sleep for specified milliseconds using setTimeout
    async fn sleep_ms(&self, ms: u64) {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    ms as i32,
                )
                .unwrap();
        });

        JsFuture::from(promise).await.unwrap();
    }

    /// Test connectivity to WebSocket endpoint
    pub async fn test_connectivity(&self, url: &str) -> Result<ConnectivityResult> {
        let start_time = js_sys::Date::now() as u64;

        // Build WebSocket URL
        let full_url = self.build_websocket_url(url)?;

        // Create a simple health check envelope
        let meta = WasmMeta::with_auto_fields();
        let envelope = WasmEnvelope::new(meta, serde_wasm_bindgen::to_value(&serde_json::json!({
            "action": "ping",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?)?;

        match self.attempt_send_envelope(&full_url, &envelope).await {
            Ok(_) => {
                let end_time = js_sys::Date::now() as u64;
                Ok(ConnectivityResult {
                    response_time_ms: end_time - start_time,
                    status_code: 101, // WebSocket upgrade status
                    success: true,
                })
            }
            Err(_) => {
                let end_time = js_sys::Date::now() as u64;
                
                // For connectivity test, we don't fail completely - we return info about the failure
                Ok(ConnectivityResult {
                    response_time_ms: end_time - start_time,
                    status_code: 0, // Unknown status
                    success: false,
                })
            }
        }
    }

    /// Subscribe to WebSocket messages for real-time communication
    pub async fn subscribe(&self, url: &str, topic: &str) -> Result<WasmWebSocketSubscription> {
        let full_url = self.build_websocket_url(url)?;
        let connection = self.create_connection(&full_url).await?;

        // Send subscription message
        let meta = WasmMeta::with_auto_fields();
        let subscription_envelope = WasmEnvelope::new(meta, serde_wasm_bindgen::to_value(&serde_json::json!({
            "action": "subscribe",
            "topic": topic,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?)?;

        let json_message = serde_json::to_string(&subscription_envelope)
            .map_err(|e| QollectiveError::serialization(format!("Failed to serialize subscription: {}", e)))?;

        connection.websocket.send_with_str(&json_message)
            .map_err(|_| QollectiveError::transport("Failed to send subscription message".to_string()))?;

        Ok(WasmWebSocketSubscription {
            connection,
            topic: topic.to_string(),
        })
    }

    /// Get client configuration
    pub fn config(&self) -> &WebSocketClientConfig {
        &self.config
    }

    /// Get certificate manager
    pub fn cert_manager(&self) -> &WasmCertificateManager {
        &self.cert_manager
    }
}

/// WebSocket subscription for real-time message handling
#[derive(Debug)]
pub struct WasmWebSocketSubscription {
    connection: WasmWebSocketConnection,
    topic: String,
}

impl WasmWebSocketSubscription {
    /// Get next message from subscription
    pub async fn next_message(&self) -> Result<WasmEnvelope> {
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let resolve = Rc::new(RefCell::new(Some(resolve)));
            let reject = Rc::new(RefCell::new(Some(reject)));

            let onmessage_resolve = resolve.clone();
            let onmessage_reject = reject.clone();

            let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                let message_data = if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                    text.as_string().unwrap_or_default()
                } else {
                    return;
                };

                match serde_json::from_str::<WasmEnvelope>(&message_data) {
                    Ok(envelope) => {
                        if let Some(resolver) = onmessage_resolve.borrow_mut().take() {
                            let js_envelope = serde_wasm_bindgen::to_value(&envelope).unwrap();
                            resolver.call1(&JsValue::NULL, &js_envelope).unwrap();
                        }
                    },
                    Err(e) => {
                        if let Some(rejector) = onmessage_reject.borrow_mut().take() {
                            rejector.call1(&JsValue::NULL, &JsValue::from_str(&format!("Parse error: {}", e))).unwrap();
                        }
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            self.connection.websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        });

        match JsFuture::from(promise).await {
            Ok(js_value) => {
                let envelope: WasmEnvelope = serde_wasm_bindgen::from_value(js_value)
                    .map_err(|e| QollectiveError::deserialization(format!("Failed to deserialize message: {}", e)))?;
                Ok(envelope)
            },
            Err(_) => Err(QollectiveError::transport("Message reception failed".to_string())),
        }
    }

    /// Unsubscribe from the topic
    pub async fn unsubscribe(&self) -> Result<()> {
        let meta = WasmMeta::with_auto_fields();
        let unsubscribe_envelope = WasmEnvelope::new(meta, serde_wasm_bindgen::to_value(&serde_json::json!({
            "action": "unsubscribe",
            "topic": &self.topic,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?)?;

        let json_message = serde_json::to_string(&unsubscribe_envelope)
            .map_err(|e| QollectiveError::serialization(format!("Failed to serialize unsubscribe: {}", e)))?;

        self.connection.websocket.send_with_str(&json_message)
            .map_err(|_| QollectiveError::transport("Failed to send unsubscribe message".to_string()))?;

        Ok(())
    }

    /// Close the subscription
    pub fn close(&self) -> Result<()> {
        self.connection.websocket.close()
            .map_err(|_| QollectiveError::transport("Failed to close WebSocket".to_string()))?;
        Ok(())
    }

    /// Get topic name
    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::wasm::CertificateConfig;
    use crate::wasm::js_types::WasmMeta;

    fn create_test_config() -> WebSocketClientConfig {
        use crate::client::common::ClientConfig;
        
        WebSocketClientConfig {
            base: ClientConfig {
                base_url: "wss://api.example.com".to_string(),
                timeout_seconds: timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS / 1000,
                retry_attempts: 3,
                ..ClientConfig::default()
            },
            websocket: crate::config::websocket::WebSocketConfig {
                ping_interval_ms: timeouts::DEFAULT_WEBSOCKET_PING_INTERVAL_MS,
                max_message_size: limits::DEFAULT_WEBSOCKET_MESSAGE_SIZE,
                enable_compression: true,
                subprotocols: vec!["qollective".to_string()],
            },
            connection_timeout_ms: timeouts::DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS,
            message_timeout_ms: timeouts::DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS,
            ping_timeout_ms: 10000,
            max_frame_size: 16 * 1024 * 1024,
            max_retry_attempts: 3,
            user_agent: "qollective-wasm-test/1.0".to_string(),
            connection_headers: std::collections::HashMap::new(),
            tls: crate::config::tls::TlsConfig::default(),
        }
    }

    fn create_test_cert_manager() -> WasmCertificateManager {
        let cert_config = CertificateConfig::default();
        WasmCertificateManager::new(&cert_config).unwrap()
    }

    #[test]
    fn test_wasm_websocket_client_creation() {
        let config = create_test_config();
        let cert_manager = create_test_cert_manager();
        
        let client = WasmWebSocketClient::new(config, cert_manager);
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.config().base.base_url, "wss://api.example.com");
        assert_eq!(client.config().max_retry_attempts, 3);
        assert_eq!(client.config().websocket.subprotocols, vec!["qollective".to_string()]);
    }

    #[test]
    fn test_websocket_url_building() {
        let config = create_test_config();
        let cert_manager = create_test_cert_manager();
        let client = WasmWebSocketClient::new(config, cert_manager).unwrap();

        // Test full WebSocket URLs
        assert_eq!(
            client.build_websocket_url("wss://test.com/api").unwrap(),
            "wss://test.com/api"
        );
        assert_eq!(
            client.build_websocket_url("ws://test.com/api").unwrap(),
            "ws://test.com/api"
        );

        // Test HTTP to WebSocket conversion
        assert_eq!(
            client.build_websocket_url("https://test.com/api").unwrap(),
            "wss://test.com/api"
        );
        assert_eq!(
            client.build_websocket_url("http://test.com/api").unwrap(),
            "ws://test.com/api"
        );

        // Test relative URL building from base
        assert_eq!(
            client.build_websocket_url("/api/ws").unwrap(),
            "wss://api.example.com/api/ws"
        );
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = create_test_config();
        let cert_manager = create_test_cert_manager();
        let client = WasmWebSocketClient::new(config, cert_manager).unwrap();

        // Test exponential backoff
        assert_eq!(client.calculate_retry_delay(0), 1000);  // 1000 * 2^0
        assert_eq!(client.calculate_retry_delay(1), 2000);  // 1000 * 2^1
        assert_eq!(client.calculate_retry_delay(2), 4000);  // 1000 * 2^2
        assert_eq!(client.calculate_retry_delay(3), 8000);  // 1000 * 2^3
        assert_eq!(client.calculate_retry_delay(4), 16000); // 1000 * 2^4
        assert_eq!(client.calculate_retry_delay(5), 30000); // Capped at 30 seconds
    }

    #[test]
    fn test_config_validation() {
        let mut config = create_test_config();
        config.base.base_url = "".to_string(); // Invalid empty URL
        
        let cert_manager = create_test_cert_manager();
        let result = WasmWebSocketClient::new(config, cert_manager);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Base URL cannot be empty"));
    }

    #[test]
    fn test_connection_state() {
        let states = vec![
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Disconnecting,
        ];

        // Test Debug trait
        for state in states {
            let debug_str = format!("{:?}", state);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_websocket_subscription() {
        let connection = WasmWebSocketConnection {
            websocket: WebSocket::new("wss://test.com").unwrap(),
            state: ConnectionState::Connected,
            url: "wss://test.com".to_string(),
        };

        let subscription = WasmWebSocketSubscription {
            connection,
            topic: "test-topic".to_string(),
        };

        assert_eq!(subscription.topic(), "test-topic");
    }

    // Note: Browser-based tests would require wasm-bindgen-test framework
    // These tests verify the structure and configuration handling
}
