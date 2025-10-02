// ABOUTME: WebSocket server implementation with unified envelope handling pattern
// ABOUTME: Provides WebSocket server functionality with automatic envelope parsing and metadata injection

//! WebSocket server implementation with unified envelope handling.
//!
//! This module provides a WebSocket server that integrates with the Qollective envelope system,
//! offering connection management, handler registration, and real-time bidirectional communication.

#[cfg(feature = "websocket-server")]
use crate::{
    client::websocket::WebSocketMessageType,
    envelope::EnvelopeError,
    error::{QollectiveError, Result},
    server::common::ServerConfig,
    traits::{handlers::ContextDataHandler, receivers::UnifiedEnvelopeReceiver},
};

#[cfg(feature = "websocket-server")]
use async_trait::async_trait;

#[cfg(feature = "websocket-server")]
use tokio::net::TcpListener;

#[cfg(feature = "websocket-server")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "websocket-server")]
use std::{collections::HashMap, sync::Arc, time::Duration};

#[cfg(feature = "websocket-server")]
use tokio::sync::RwLock;

#[cfg(feature = "websocket-server")]
use tokio_tungstenite::{accept_hdr_async, tungstenite::protocol::Message, WebSocketStream};

#[cfg(feature = "websocket-server")]
use tokio::net::TcpStream;

#[cfg(feature = "websocket-server")]
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "websocket-server")]
use futures_util::{SinkExt, StreamExt};
// =============================================================================
// CONFIGURATION TYPES
// =============================================================================

/// Configuration for WebSocket server behavior
#[cfg(feature = "websocket-server")]
#[derive(Debug, Clone)]
pub struct WebSocketServerConfig {
    /// Base server configuration (bind address, port, etc.)
    pub base: ServerConfig,
    /// TLS configuration for secure connections
    pub tls: crate::config::tls::TlsConfig,
    /// Maximum frame size for WebSocket messages
    pub max_frame_size: usize,
    /// Maximum message size for WebSocket messages
    pub max_message_size: usize,
    /// Ping interval for keep-alive
    pub ping_interval: Duration,
    /// Ping timeout
    pub ping_timeout: Duration,
    /// Supported subprotocols
    pub subprotocols: Vec<String>,
    /// Connection timeout
    pub connection_timeout: Option<Duration>,
}

#[cfg(feature = "websocket-server")]
impl Default for WebSocketServerConfig {
    fn default() -> Self {
        Self {
            base: ServerConfig::default(),
            tls: crate::config::tls::TlsConfig::default(),
            max_frame_size: 16 * 1024 * 1024,   // 16MB
            max_message_size: 64 * 1024 * 1024, // 64MB
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            subprotocols: vec!["qollective-v1".to_string()],
            connection_timeout: Some(Duration::from_secs(30)),
        }
    }
}

// =============================================================================
// CORE WEBSOCKET SERVER
// =============================================================================

/// Handler info for tracking registered handlers
#[cfg(feature = "websocket-server")]
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub route: String,
    pub description: String,
}

/// Type-erased handler function for WebSocket message processing
#[cfg(feature = "websocket-server")]
type BoxedHandler = Box<
    dyn Fn(
            serde_json::Value,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send>>
        + Send
        + Sync,
>;

/// WebSocket server for real-time communication following unified pattern
#[cfg(feature = "websocket-server")]
pub struct WebSocketServer {
    config: WebSocketServerConfig,
    handlers: HashMap<String, HandlerInfo>, // Path -> Handler info mapping
    handler_functions: Arc<RwLock<HashMap<String, BoxedHandler>>>, // Path -> Actual handler function
    listener: Option<TcpListener>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[cfg(feature = "websocket-server")]
impl std::fmt::Debug for WebSocketServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketServer")
            .field("config", &self.config)
            .field("handlers", &self.handlers)
            .field("listener", &"<TcpListener>")
            .field("shutdown_tx", &"<OneShot>")
            .finish()
    }
}

#[cfg(feature = "websocket-server")]
impl WebSocketServer {
    /// Create a new WebSocket server with the given configuration
    ///
    /// This is the single, obvious way to create a WebSocket server,
    /// following the unified pattern of simple construction.
    pub async fn new(config: WebSocketServerConfig) -> Result<Self> {
        // Validate configuration
        if config.base.port == 0 {
            return Err(QollectiveError::config("Port cannot be 0"));
        }

        if config.base.bind_address.is_empty() {
            return Err(QollectiveError::config("Bind address cannot be empty"));
        }

        if config.max_frame_size == 0 {
            return Err(QollectiveError::config("max_frame_size cannot be 0"));
        }

        if config.max_message_size == 0 {
            return Err(QollectiveError::config("max_message_size cannot be 0"));
        }

        // Validate TLS configuration using the unified config
        config.tls.validate()?;

        Ok(Self {
            config,
            handlers: HashMap::new(),
            handler_functions: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            shutdown_tx: None,
        })
    }

    /// Start the WebSocket server
    ///
    /// Binds to the configured address and port, then starts accepting WebSocket connections.
    /// This method will block until the server is shut down.
    pub async fn start(&mut self) -> Result<()> {
        let bind_addr = format!(
            "{}:{}",
            self.config.base.bind_address, self.config.base.port
        );

        // Create TCP listener
        let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
            QollectiveError::transport(format!("Failed to bind to {}: {}", bind_addr, e))
        })?;

        let protocol = if self.config.tls.enabled { "wss" } else { "ws" };
        tracing::info!("WebSocket server starting on {}://{}", protocol, bind_addr);
        self.listener = Some(listener);

        // Create TLS acceptor if TLS is enabled
        let tls_acceptor = if self.config.tls.enabled {
            let server_config = self.config.tls.create_server_config().await?;
            Some(TlsAcceptor::from(server_config))
        } else {
            None
        };

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Get the listener (we know it's Some because we just set it)
        let listener = self.listener.take().unwrap();

        // Accept connections in a loop
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = &mut shutdown_rx => {
                    tracing::info!("WebSocket server shutting down gracefully");
                    break;
                }
                // Accept new connections
                connection_result = listener.accept() => {
                    match connection_result {
                        Ok((stream, addr)) => {
                            tracing::info!("New WebSocket connection from {}", addr);
                            let config = self.config.clone();
                            let handler_functions = Arc::clone(&self.handler_functions);
                            let tls_acceptor = tls_acceptor.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_websocket_connection(stream, config, handler_functions, tls_acceptor).await {
                                    tracing::error!("WebSocket connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }

        tracing::info!("WebSocket server stopped");
        Ok(())
    }

    /// Shutdown the WebSocket server gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        Ok(())
    }

    /// Get the server configuration
    pub fn config(&self) -> &WebSocketServerConfig {
        &self.config
    }

    /// Register a route with handler info
    ///
    /// This is an internal method used by the UnifiedEnvelopeReceiver implementation.
    fn register_route_with_handler(&mut self, path: &str, handler_info: HandlerInfo) -> Result<()> {
        // Validate path
        if path.is_empty() {
            return Err(QollectiveError::config("Path cannot be empty"));
        }

        // Check for duplicate paths
        if self.handlers.contains_key(path) {
            return Err(QollectiveError::config(format!(
                "Path '{}' is already registered",
                path
            )));
        }

        // Store handler info
        self.handlers.insert(path.to_string(), handler_info);
        tracing::info!("Registered WebSocket handler: {}", path);
        Ok(())
    }

    /// Get handler info for a path
    pub fn get_handler_info(&self, path: &str) -> Option<&HandlerInfo> {
        self.handlers.get(path)
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Check if a path has a handler
    pub fn has_handler(&self, path: &str) -> bool {
        self.handlers.contains_key(path)
    }

    /// List all registered paths
    pub fn paths(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

/// Handle individual WebSocket connection
#[cfg(feature = "websocket-server")]
async fn handle_websocket_connection(
    stream: TcpStream,
    config: WebSocketServerConfig,
    handler_functions: Arc<RwLock<HashMap<String, BoxedHandler>>>,
    tls_acceptor: Option<TlsAcceptor>,
) -> Result<()> {
    // Extract path from HTTP request during WebSocket handshake
    let mut request_path = String::from("/"); // Default path

    // Handle TLS handshake if TLS is enabled and create WebSocket stream
    if let Some(tls_acceptor) = tls_acceptor {
        // TLS connection
        let tls_stream = tls_acceptor
            .accept(stream)
            .await
            .map_err(|e| QollectiveError::transport(format!("TLS handshake failed: {}", e)))?;

        let ws_stream = accept_hdr_async(
            tls_stream,
            |req: &tokio_tungstenite::tungstenite::handshake::server::Request,
             response: tokio_tungstenite::tungstenite::handshake::server::Response| {
                // Extract the path from the HTTP request
                request_path = req.uri().path().to_string();
                tracing::debug!("WebSocket request path: {}", request_path);
                Ok(response)
            },
        )
        .await
        .map_err(|e| QollectiveError::transport(format!("WebSocket handshake failed: {}", e)))?;

        tracing::info!("WebSocket handshake completed for path: {}", request_path);
        handle_websocket_messages(ws_stream, config, handler_functions, &request_path).await?;
    } else {
        // Plain TCP connection
        let ws_stream = accept_hdr_async(
            stream,
            |req: &tokio_tungstenite::tungstenite::handshake::server::Request,
             response: tokio_tungstenite::tungstenite::handshake::server::Response| {
                // Extract the path from the HTTP request
                request_path = req.uri().path().to_string();
                tracing::debug!("WebSocket request path: {}", request_path);
                Ok(response)
            },
        )
        .await
        .map_err(|e| QollectiveError::transport(format!("WebSocket handshake failed: {}", e)))?;

        tracing::info!("WebSocket handshake completed for path: {}", request_path);
        handle_websocket_messages(ws_stream, config, handler_functions, &request_path).await?;
    }

    Ok(())
}

/// Handle WebSocket messages for any stream type
#[cfg(feature = "websocket-server")]
async fn handle_websocket_messages<S>(
    ws_stream: WebSocketStream<S>,
    config: WebSocketServerConfig,
    handler_functions: Arc<RwLock<HashMap<String, BoxedHandler>>>,
    request_path: &str,
) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // Handle messages
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                // Parse WebSocket message
                match serde_json::from_str::<WebSocketMessageType>(&text) {
                    Ok(WebSocketMessageType::Envelope { payload }) => {
                        // Process envelope message using registered handlers with extracted path
                        let response = process_envelope_message(
                            payload,
                            &config,
                            Arc::clone(&handler_functions),
                            &request_path,
                        )
                        .await;

                        // Send response
                        let response_text = serde_json::to_string(&response).map_err(|e| {
                            QollectiveError::serialization(format!(
                                "Failed to serialize response: {}",
                                e
                            ))
                        })?;

                        if let Err(e) = ws_sender.send(Message::Text(response_text.into())).await {
                            tracing::error!("Failed to send WebSocket response: {}", e);
                            break;
                        }
                        
                        tracing::debug!("Response sent successfully to client");
                    }
                    Ok(WebSocketMessageType::Ping { timestamp: _ }) => {
                        // Respond with pong
                        let pong = WebSocketMessageType::Pong {
                            timestamp: chrono::Utc::now(),
                        };
                        let pong_text = serde_json::to_string(&pong).map_err(|e| {
                            QollectiveError::serialization(format!(
                                "Failed to serialize pong: {}",
                                e
                            ))
                        })?;

                        if let Err(e) = ws_sender.send(Message::Text(pong_text.into())).await {
                            tracing::error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Ok(_) => {
                        // Handle other message types as needed
                        tracing::debug!("Received other WebSocket message type");
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse WebSocket message: {}", e);
                        // Send error response
                        let error_msg = WebSocketMessageType::Error {
                            message: format!("Invalid message format: {}", e),
                            code: Some(400),
                        };
                        let error_text = serde_json::to_string(&error_msg).map_err(|e| {
                            QollectiveError::serialization(format!(
                                "Failed to serialize error: {}",
                                e
                            ))
                        })?;
                        
                        if let Err(e) = ws_sender.send(Message::Text(error_text.into())).await {
                            tracing::error!("Failed to send error response: {}", e);
                        }
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                tracing::debug!("Received binary WebSocket message (not supported)");
            }
            Ok(Message::Close(close_frame)) => {
                tracing::info!("WebSocket connection close frame received from client: {:?}", close_frame);
                
                // Send close frame response (WebSocket close handshake protocol)
                if let Err(e) = ws_sender.send(Message::Close(close_frame.clone())).await {
                    tracing::error!("Failed to send close frame response: {}", e);
                } else {
                    tracing::info!("Close frame response sent to client - handshake complete");
                }
                
                // Break from message loop to close connection gracefully
                tracing::info!("Exiting WebSocket message loop after close handshake");
                break;
            }
            Ok(Message::Ping(data)) => {
                // Respond with pong
                if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                    tracing::error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Handle pong response
                tracing::debug!("Received pong from client");
            }
            Ok(Message::Frame(_)) => {
                tracing::debug!("Received raw frame (not supported)");
            }
            Err(e) => {
                let error_msg = e.to_string();
                
                // Handle "Connection reset without closing handshake" more gracefully
                // This commonly happens when clients complete their work and disconnect immediately
                if error_msg.contains("Connection reset without closing handshake") {
                    tracing::info!("Client disconnected without close handshake (normal for short-lived connections): {}", e);
                } else {
                    tracing::error!("WebSocket error occurred: {}", e);
                }
                
                // Connection is likely already closed, so don't attempt to send close frame
                tracing::debug!("Exiting WebSocket message loop due to connection termination");
                break;
            }
        }
    }

    tracing::info!("WebSocket connection closed for path: {}", request_path);
    Ok(())
}

/// Process envelope message and return response
#[cfg(feature = "websocket-server")]
pub(crate) async fn process_envelope_message(
    data: serde_json::Value,
    _config: &WebSocketServerConfig,
    handler_functions: Arc<RwLock<HashMap<String, BoxedHandler>>>,
    path: &str,
) -> WebSocketMessageType {
    // Extract original metadata if present for preservation in response
    let original_meta = if data.is_object() && data.as_object().unwrap().contains_key("meta") {
        data.as_object().unwrap().get("meta").and_then(|meta_value| {
            serde_json::from_value::<crate::envelope::Meta>(meta_value.clone()).ok()
        })
    } else {
        None
    };

    // Try to find a handler for the specified path
    let handlers = handler_functions.read().await;

    if let Some(handler) = handlers.get(path) {
        // Call the actual handler
        match handler(data).await {
            Ok(result) => {
                // Wrap handler response in proper Qollective envelope using framework types
                let envelope_response = create_response_envelope(result, original_meta);
                WebSocketMessageType::Envelope {
                    payload: envelope_response,
                }
            }
            Err(e) => {
                // Convert QollectiveError to EnvelopeError and extract status code
                let envelope_error = convert_qollective_error_to_envelope_error(&e);
                envelope_error_to_websocket_message(&envelope_error)
            },
        }
    } else {
        // No handler found for this path - try default path "/"
        if path != "/" {
            if let Some(default_handler) = handlers.get("/") {
                match default_handler(data).await {
                    Ok(result) => {
                        // Wrap handler response in proper Qollective envelope using framework types
                        let envelope_response = create_response_envelope(result, original_meta);
                        WebSocketMessageType::Envelope {
                            payload: envelope_response,
                        }
                    }
                    Err(e) => {
                        // Convert QollectiveError to EnvelopeError and extract status code
                        let envelope_error = convert_qollective_error_to_envelope_error(&e);
                        envelope_error_to_websocket_message(&envelope_error)
                    },
                }
            } else {
                let not_found_error = QollectiveError::not_found_error(
                    format!("No handler found for path: {}", path),
                    Some(serde_json::json!({"path": path, "operation": "handler_lookup"}))
                );
                envelope_error_to_websocket_message(&not_found_error)
            }
        } else {
            let not_found_error = QollectiveError::not_found_error(
                format!("No handler found for path: {}", path),
                Some(serde_json::json!({"path": path, "operation": "handler_lookup"}))
            );
            envelope_error_to_websocket_message(&not_found_error)
        }
    }
}

/// Create response envelope using framework Envelope structure, preserving original metadata
#[cfg(feature = "websocket-server")]
fn create_response_envelope(handler_result: serde_json::Value, original_meta: Option<crate::envelope::Meta>) -> serde_json::Value {
    use crate::envelope::{Envelope, Meta};

    // Use the proper metadata preservation utility following the same pattern as gRPC server
    let meta = Meta::preserve_for_response(original_meta.as_ref());

    let envelope = Envelope::new(meta, handler_result);

    // Convert envelope to JSON Value for WebSocket transport
    serde_json::to_value(envelope).unwrap_or_else(|e| {
        tracing::error!("Failed to serialize response envelope: {}", e);
        
        // Use the same preservation pattern for error fallback
        let error_meta = Meta::preserve_for_response(original_meta.as_ref());
        
        // Create proper error envelope using builder pattern
        use crate::envelope::EnvelopeError;
        let envelope_error = EnvelopeError {
            code: "SERIALIZATION_ERROR".to_string(),
            message: format!("Failed to serialize response: {}", e),
            details: None,
            trace: None,
            #[cfg(any(
                feature = "rest-server", 
                feature = "rest-client",
                feature = "websocket-server", 
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(500),
        };
        
        let error_envelope = Envelope::error(error_meta, (), envelope_error);
        serde_json::to_value(error_envelope).unwrap_or_else(|_| {
            // Ultimate fallback if even error envelope serialization fails
            serde_json::json!({
                "meta": {"timestamp": chrono::Utc::now()},
                "payload": null,
                "error": {"code": "CRITICAL_SERIALIZATION_ERROR", "message": "Cannot serialize any response"}
            })
        })
    })
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Convert QollectiveError to EnvelopeError for consistent error handling
/// 
/// This helper function converts framework QollectiveError instances to EnvelopeError
/// instances, allowing the WebSocket server to use the same error handling patterns
/// as other protocols while extracting appropriate HTTP status codes.
#[cfg(feature = "websocket-server")]
pub fn convert_qollective_error_to_envelope_error(error: &QollectiveError) -> EnvelopeError {
    // Map QollectiveError variants to appropriate EnvelopeError instances with status codes
    match error {
        QollectiveError::Validation(msg) => QollectiveError::validation_error(msg.clone(), None),
        QollectiveError::Security(msg) => QollectiveError::auth_error(msg.clone(), None),
        QollectiveError::Config(msg) => QollectiveError::validation_error(
            format!("Configuration error: {}", msg), 
            Some(serde_json::json!({"category": "configuration"}))
        ),
        QollectiveError::Transport(msg) => QollectiveError::server_error(
            format!("Transport error: {}", msg),
            Some(serde_json::json!({"category": "transport"}))
        ),
        QollectiveError::Connection(msg) => QollectiveError::server_error(
            format!("Connection error: {}", msg),
            Some(serde_json::json!({"category": "connection"}))
        ),
        QollectiveError::AgentNotFound(msg) => QollectiveError::not_found_error(
            format!("Agent not found: {}", msg),
            Some(serde_json::json!({"category": "agent", "resource": "agent"}))
        ),
        _ => QollectiveError::server_error(
            error.to_string(),
            Some(serde_json::json!({"category": "general", "error_type": std::any::type_name::<QollectiveError>()}))
        ),
    }
}

/// Convert EnvelopeError to WebSocketMessageType::Error with proper status code extraction
/// 
/// This helper function extracts the HTTP status code from EnvelopeError.http_status_code
/// and creates a WebSocketMessageType::Error with the appropriate status code. If no
/// custom status code is set, it falls back to pattern-based mapping.
#[cfg(feature = "websocket-server")]
pub fn envelope_error_to_websocket_message(error: &EnvelopeError) -> WebSocketMessageType {
    // Extract HTTP status code using same logic as REST server
    let status_code = extract_http_status_code_for_websocket(error);
    
    WebSocketMessageType::Error {
        message: error.message.clone(),
        code: Some(status_code as u32),
    }
}

/// Extract HTTP status code from EnvelopeError for WebSocket error responses
/// 
/// This function mirrors the REST server logic for extracting HTTP status codes,
/// providing consistent behavior across protocols while respecting the envelope-first architecture.
#[cfg(feature = "websocket-server")]
fn extract_http_status_code_for_websocket(error: &EnvelopeError) -> u16 {
    // Check if custom status code is available and valid
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    if let Some(status_code) = error.http_status_code {
        // Validate status code is in error range (400-599)
        if status_code >= 400 && status_code < 600 {
            return status_code;
        }
        // Log warning for invalid status code but continue with fallback
        #[cfg(feature = "tracing")]
        tracing::warn!("Invalid HTTP status code {} in EnvelopeError for WebSocket, using fallback mapping", status_code);
    }
    
    // Fallback to pattern-based status code mapping (same logic as REST server)
    let error_code = error.code.to_uppercase();
    match error_code.as_str() {
        // Authentication and authorization errors
        code if code.contains("AUTH") || code.contains("UNAUTHORIZED") => 401,
        code if code.contains("FORBIDDEN") || code.contains("PERMISSION") => 403,
        
        // Client errors  
        code if code.contains("VALIDATION") || code.contains("INVALID") => 400,
        code if code.contains("NOT_FOUND") || code.contains("MISSING") => 404,
        code if code.contains("CONFLICT") || code.contains("EXISTS") => 409,
        code if code.contains("TOO_LARGE") || code.contains("SIZE") => 413,
        code if code.contains("RATE_LIMIT") || code.contains("THROTTLE") => 429,
        
        // Server errors
        code if code.contains("TIMEOUT") || code.contains("DEADLINE") => 504,
        code if code.contains("UNAVAILABLE") || code.contains("SERVICE") => 503,
        code if code.contains("NOT_IMPLEMENTED") => 501,
        
        // Default to 500 for any unrecognized error patterns
        _ => 500,
    }
}

/// Implementation of UnifiedEnvelopeReceiver trait
///
/// This provides native integration with the unified pattern,
/// built into the core architecture rather than bolted on.
#[cfg(feature = "websocket-server")]
#[async_trait]
impl UnifiedEnvelopeReceiver for WebSocketServer {
    /// Register a handler for envelopes at the default path ("/")
    async fn receive_envelope<T, R, H>(&mut self, handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        self.receive_envelope_at("/", handler).await
    }

    /// Register a handler for envelopes at the specified path
    async fn receive_envelope_at<T, R, H>(&mut self, path: &str, handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        // Create handler info for demonstration
        let handler_info = HandlerInfo {
            route: path.to_string(),
            description: format!("WebSocket handler registered for path: {}", path),
        };

        // Wrap handler in Arc to allow sharing across multiple calls
        let handler_arc = Arc::new(handler);

        // Create type-erased handler function that wraps the typed handler
        let boxed_handler: BoxedHandler = Box::new(move |data: serde_json::Value| {
            let handler_ref = Arc::clone(&handler_arc);
            Box::pin(async move {
                // Check if data is a full envelope structure with meta and payload fields
                let (typed_data, context): (T, Option<crate::envelope::Context>) = if data.is_object() && data.as_object().unwrap().contains_key("payload") {
                    let envelope_obj = data.as_object().unwrap();
                    
                    // Extract the payload field from the envelope
                    let envelope_data = envelope_obj.get("payload").unwrap();
                    let typed_data: T = serde_json::from_value(envelope_data.clone()).map_err(|e| {
                        QollectiveError::envelope(format!("Failed to deserialize envelope data: {}", e))
                    })?;
                    
                    // Extract the meta field and construct Context if present
                    let context = if let Some(meta_value) = envelope_obj.get("meta") {
                        let meta: crate::envelope::Meta = serde_json::from_value(meta_value.clone()).map_err(|e| {
                            QollectiveError::envelope(format!("Failed to deserialize envelope metadata: {}", e))
                        })?;
                        Some(crate::envelope::Context::new(meta))
                    } else {
                        None
                    };
                    
                    (typed_data, context)
                } else {
                    // Assume it's already the data portion (for backward compatibility)
                    let typed_data: T = serde_json::from_value(data).map_err(|e| {
                        QollectiveError::envelope(format!("Failed to deserialize input: {}", e))
                    })?;
                    (typed_data, None)
                };

                // Call the actual handler with proper context
                let result: R = handler_ref.handle(context, typed_data).await?;

                // Serialize the result back to Value
                let value_result = serde_json::to_value(result).map_err(|e| {
                    QollectiveError::envelope(format!("Failed to serialize result: {}", e))
                })?;

                Ok(value_result)
            })
        });

        // Store the actual handler function
        {
            let mut handlers = self.handler_functions.write().await;
            handlers.insert(path.to_string(), boxed_handler);
        }

        // Register the path with handler info
        self.register_route_with_handler(path, handler_info)
    }
}

// Feature-disabled implementations
#[cfg(not(feature = "websocket-server"))]
pub struct WebSocketServer;

#[cfg(not(feature = "websocket-server"))]
pub struct WebSocketServerConfig;

#[cfg(not(feature = "websocket-server"))]
impl WebSocketServer {
    pub async fn new(_config: WebSocketServerConfig) -> crate::error::Result<Self> {
        Err(crate::error::QollectiveError::config(
            "websocket-server feature not enabled",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        echo: String,
    }

    // TDD Step 1: Write failing test for WebSocket server creation
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    pub async fn test_websocket_server_creation() {
        // ARRANGE: Create server configuration
        let config = WebSocketServerConfig::default();

        // ACT: Create WebSocket server
        let result = WebSocketServer::new(config).await;

        // ASSERT: Server should be created successfully
        assert!(result.is_ok());

        let server = result.unwrap();
        assert_eq!(server.handler_count(), 0);
        assert_eq!(server.paths().len(), 0);
    }

    // TDD Step 2: Write failing test for configuration validation
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_websocket_server_config_validation() {
        // ARRANGE: Create invalid configurations
        let mut config = WebSocketServerConfig::default();
        config.base.port = 0;

        // ACT & ASSERT: Should fail with port validation error
        let result = WebSocketServer::new(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Port cannot be 0"));

        // ARRANGE: Empty bind address
        let mut config = WebSocketServerConfig::default();
        config.base.bind_address = String::new();

        // ACT & ASSERT: Should fail with bind address validation error
        let result = WebSocketServer::new(config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Bind address cannot be empty"));

        // ARRANGE: Zero frame size
        let mut config = WebSocketServerConfig::default();
        config.max_frame_size = 0;

        // ACT & ASSERT: Should fail with frame size validation error
        let result = WebSocketServer::new(config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("max_frame_size cannot be 0"));
    }

    // TDD Step 3: Write failing test for UnifiedEnvelopeReceiver implementation
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_websocket_server_unified_envelope_receiver() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and mock handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        // Create mock handler
        struct MockHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for MockHandler {
            async fn handle(
                &self,
                _context: Option<Context>,
                _data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: "test response".to_string(),
                })
            }
        }

        let handler = MockHandler;

        // ACT: Register handler using UnifiedEnvelopeReceiver trait
        let result = server.receive_envelope(handler).await;

        // ASSERT: Handler should be registered successfully
        assert!(result.is_ok());
        assert_eq!(server.handler_count(), 1);
        assert!(server.has_handler("/"));

        let handler_info = server.get_handler_info("/").unwrap();
        assert_eq!(handler_info.route, "/");
        assert!(handler_info.description.contains("WebSocket handler"));
    }

    // TDD Step 4: Write failing test for receive_envelope_at method
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_websocket_server_receive_envelope_at() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and mock handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        // Create mock handler
        struct MockHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for MockHandler {
            async fn handle(
                &self,
                _context: Option<Context>,
                _data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: "test response".to_string(),
                })
            }
        }

        let handler = MockHandler;

        // ACT: Register handler at specific path
        let result = server.receive_envelope_at("/test", handler).await;

        // ASSERT: Handler should be registered at specified path
        assert!(result.is_ok());
        assert_eq!(server.handler_count(), 1);
        assert!(server.has_handler("/test"));
        assert!(!server.has_handler("/"));

        let handler_info = server.get_handler_info("/test").unwrap();
        assert_eq!(handler_info.route, "/test");
        assert!(handler_info.description.contains("WebSocket handler"));
        assert!(handler_info.description.contains("/test"));
    }

    // TDD Step 5: Write failing test for duplicate handler registration
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_websocket_server_duplicate_handler_registration() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and mock handlers
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        // Create mock handler
        struct MockHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for MockHandler {
            async fn handle(
                &self,
                _context: Option<Context>,
                _data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: "test response".to_string(),
                })
            }
        }

        let handler1 = MockHandler;
        let handler2 = MockHandler;

        // ACT: Register first handler
        let result1 = server.receive_envelope_at("/test", handler1).await;
        assert!(result1.is_ok());

        // ACT: Try to register second handler at same path
        let result2 = server.receive_envelope_at("/test", handler2).await;

        // ASSERT: Second registration should fail
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("already registered"));
        assert_eq!(server.handler_count(), 1);
    }

    // TDD Step 6: Write failing test for configuration defaults
    #[cfg(feature = "websocket-server")]
    #[test]
    fn test_websocket_server_config_defaults() {
        // ARRANGE & ACT: Create default configuration
        let config = WebSocketServerConfig::default();

        // ASSERT: Verify default values
        assert_eq!(config.max_frame_size, 16 * 1024 * 1024); // 16MB
        assert_eq!(config.max_message_size, 64 * 1024 * 1024); // 64MB
        assert_eq!(config.ping_interval, Duration::from_secs(30));
        assert_eq!(config.ping_timeout, Duration::from_secs(10));
        assert_eq!(config.subprotocols, vec!["qollective-v1".to_string()]);
        assert_eq!(config.connection_timeout, Some(Duration::from_secs(30)));
    }

    // TDD Step 7: Write failing test for feature-disabled behavior
    #[tokio::test]
    async fn test_websocket_server_feature_disabled() {
        #[cfg(not(feature = "websocket-server"))]
        {
            let config = WebSocketServerConfig;
            let result = WebSocketServer::new(config).await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("feature not enabled"));
        }
    }

    // TDD Step 8: CRITICAL MISSING TEST - Test handler function execution
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_handler_function_execution() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and mock handler that tracks calls
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        // Create mock handler that we can verify was actually called
        struct TrackingHandler {
            call_count: Arc<std::sync::atomic::AtomicUsize>,
        }

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for TrackingHandler {
            async fn handle(
                &self,
                _context: Option<Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                // Increment call counter to prove handler was executed
                self.call_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(TestResponse {
                    echo: format!("Handler called with: {}", data.message),
                })
            }
        }

        let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let handler = TrackingHandler {
            call_count: Arc::clone(&call_count),
        };

        // Register handler at specific path
        server.receive_envelope_at("/test", handler).await.unwrap();

        // ACT: Simulate message processing (what happens during actual WebSocket communication)
        let test_request = TestRequest {
            message: "test message".to_string(),
        };
        let request_data = serde_json::to_value(test_request).unwrap();

        // Get handler functions for testing
        let handler_functions = Arc::clone(&server.handler_functions);

        // Call process_envelope_message directly (this is what was broken)
        let response =
            process_envelope_message(request_data, &server.config, handler_functions, "/test")
                .await;

        // ASSERT: Handler should have been called and returned correct response
        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Handler should have been called exactly once"
        );

        // Verify response contains expected data
        match response {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                // The data is a full envelope with meta and data fields
                let envelope: crate::envelope::Envelope<TestResponse> =
                    serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "Handler called with: test message");
            }
            _ => panic!("Expected Envelope response, got error: {:?}", response),
        }
    }

    // TDD Step 9: CRITICAL MISSING TEST - Test path-based routing
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_path_based_message_routing() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and multiple handlers for different paths
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        // Handler for /path1
        struct Path1Handler;
        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for Path1Handler {
            async fn handle(
                &self,
                _context: Option<Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: format!("PATH1: {}", data.message),
                })
            }
        }

        // Handler for /path2
        struct Path2Handler;
        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for Path2Handler {
            async fn handle(
                &self,
                _context: Option<Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: format!("PATH2: {}", data.message),
                })
            }
        }

        // Register handlers at different paths
        server
            .receive_envelope_at("/path1", Path1Handler)
            .await
            .unwrap();
        server
            .receive_envelope_at("/path2", Path2Handler)
            .await
            .unwrap();

        let test_request = TestRequest {
            message: "routing test".to_string(),
        };
        let request_data = serde_json::to_value(test_request).unwrap();
        let handler_functions = Arc::clone(&server.handler_functions);

        // ACT & ASSERT: Test routing to /path1
        let response1 = process_envelope_message(
            request_data.clone(),
            &server.config,
            Arc::clone(&handler_functions),
            "/path1",
        )
        .await;

        match response1 {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                let envelope: crate::envelope::Envelope<TestResponse> =
                    serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "PATH1: routing test");
            }
            _ => panic!(
                "Expected Envelope response for path1, got error: {:?}",
                response1
            ),
        }

        // ACT & ASSERT: Test routing to /path2
        let response2 =
            process_envelope_message(request_data, &server.config, handler_functions, "/path2")
                .await;

        match response2 {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                let envelope: crate::envelope::Envelope<TestResponse> =
                    serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "PATH2: routing test");
            }
            _ => panic!(
                "Expected Envelope response for path2, got error: {:?}",
                response2
            ),
        }
    }

    // TDD Step 10: CRITICAL MISSING TEST - Test handler not found scenario
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_handler_not_found() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server with no handlers
        let config = WebSocketServerConfig::default();
        let server = WebSocketServer::new(config).await.unwrap();

        let test_request = TestRequest {
            message: "test message".to_string(),
        };
        let request_data = serde_json::to_value(test_request).unwrap();
        let handler_functions = Arc::clone(&server.handler_functions);

        // ACT: Try to process message for non-existent path
        let response = process_envelope_message(
            request_data,
            &server.config,
            handler_functions,
            "/nonexistent",
        )
        .await;

        // ASSERT: Should return error response
        match response {
            crate::client::websocket::WebSocketMessageType::Error { message, code } => {
                assert!(message.contains("No handler found for path: /nonexistent"));
                assert_eq!(code, Some(404));
            }
            _ => panic!("Expected Error response, got: {:?}", response),
        }
    }

    // TDD Step 11: NEW TEST - Test envelope wrapping functionality
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_response_envelope_wrapping() {
        use crate::envelope::{Envelope, Meta};

        // ARRANGE: Create test handler result
        let handler_result = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"test": "data"},
            "id": "123"
        });

        // ACT: Create response envelope
        let envelope_response = create_response_envelope(handler_result.clone(), None);

        // ASSERT: Should be wrapped in proper envelope format
        assert!(envelope_response.is_object());

        let envelope_obj = envelope_response.as_object().unwrap();

        // Check meta field exists and has required fields
        assert!(envelope_obj.contains_key("meta"));
        let meta = envelope_obj.get("meta").unwrap();
        assert!(meta.get("version").is_some());
        assert!(meta.get("timestamp").is_some());
        // Note: tenant is None when no original metadata provided (fallback behavior)

        // Check payload field contains original handler result
        assert!(envelope_obj.contains_key("payload"));
        let payload = envelope_obj.get("payload").unwrap();
        assert_eq!(payload, &handler_result);

        // Verify it can be deserialized as proper Envelope
        let envelope: Envelope<serde_json::Value> =
            serde_json::from_value(envelope_response).unwrap();
        assert_eq!(envelope.payload, handler_result);
        assert_eq!(envelope.meta.tenant, None); // No tenant when no original metadata
        assert_eq!(envelope.meta.version, Some("1.0".to_string()));
        assert!(envelope.meta.timestamp.is_some());
    }

    // TDD Step 12: NEW TEST - Test envelope wrapping in message processing
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_envelope_wrapping_in_message_processing() {
        use crate::envelope::{Context, Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        struct TestHandler;
        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for TestHandler {
            async fn handle(
                &self,
                _context: Option<Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: format!("Processed: {}", data.message),
                })
            }
        }

        server
            .receive_envelope_at("/test", TestHandler)
            .await
            .unwrap();

        let test_request = TestRequest {
            message: "envelope test".to_string(),
        };
        let request_data = serde_json::to_value(test_request).unwrap();
        let handler_functions = Arc::clone(&server.handler_functions);

        // ACT: Process message (should wrap response in envelope)
        let response =
            process_envelope_message(request_data, &server.config, handler_functions, "/test")
                .await;

        // ASSERT: Response should be wrapped envelope
        match response {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                // Verify envelope structure
                assert!(payload.is_object());
                let envelope_obj = payload.as_object().unwrap();

                // Check envelope has meta and payload fields
                assert!(envelope_obj.contains_key("meta"));
                assert!(envelope_obj.contains_key("payload"));

                // Verify meta fields
                let meta = envelope_obj.get("meta").unwrap();
                assert_eq!(meta.get("version").unwrap().as_str().unwrap(), "1.0");
                assert!(meta.get("timestamp").is_some());
                // Note: tenant is None when no original metadata provided

                // Verify payload contains handler response
                let response_data = envelope_obj.get("payload").unwrap();
                let test_response: TestResponse =
                    serde_json::from_value(response_data.clone()).unwrap();
                assert_eq!(test_response.echo, "Processed: envelope test");

                // Verify can be deserialized as proper Envelope
                let envelope: Envelope<TestResponse> = serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "Processed: envelope test");
                assert_eq!(envelope.meta.tenant, None); // No tenant when no original metadata
            }
            _ => panic!("Expected Envelope response, got: {:?}", response),
        }
    }

    // TDD Step 13: NEW TEST - Test envelope structure deserialization (full envelope with meta and data)
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_envelope_structure_deserialization() {
        use crate::envelope::{Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        struct EnvelopeHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for EnvelopeHandler {
            async fn handle(
                &self,
                _context: Option<crate::envelope::Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: format!("Envelope handled: {}", data.message),
                })
            }
        }

        server.receive_envelope_at("/envelope_test", EnvelopeHandler).await.unwrap();

        // ACT: Create full envelope structure (what WebSocket client sends)
        let envelope_request = Envelope::new(
            Meta::default(),
            TestRequest {
                message: "envelope test".to_string(),
            },
        );
        let envelope_data = serde_json::to_value(envelope_request).unwrap();

        let handler_functions = Arc::clone(&server.handler_functions);
        let response = process_envelope_message(envelope_data, &server.config, handler_functions, "/envelope_test").await;

        // ASSERT: Handler should successfully process the envelope structure
        match response {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                let envelope: Envelope<TestResponse> = serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "Envelope handled: envelope test");
            }
            _ => panic!("Expected Envelope response, got: {:?}", response),
        }
    }

    // TDD Step 14: NEW TEST - Test backward compatibility with raw data (no envelope structure)
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_backward_compatibility_raw_data() {
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        struct RawDataHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for RawDataHandler {
            async fn handle(
                &self,
                _context: Option<crate::envelope::Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: format!("Raw handled: {}", data.message),
                })
            }
        }

        server.receive_envelope_at("/raw_test", RawDataHandler).await.unwrap();

        // ACT: Send raw data (no envelope structure - for backward compatibility)
        let raw_request = TestRequest {
            message: "raw test".to_string(),
        };
        let raw_data = serde_json::to_value(raw_request).unwrap();

        let handler_functions = Arc::clone(&server.handler_functions);
        let response = process_envelope_message(raw_data, &server.config, handler_functions, "/raw_test").await;

        // ASSERT: Handler should successfully process raw data
        match response {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                let envelope: crate::envelope::Envelope<TestResponse> = serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "Raw handled: raw test");
            }
            _ => panic!("Expected Envelope response, got: {:?}", response),
        }
    }

    // TDD Step 15: NEW TEST - Test envelope deserialization error handling
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_envelope_deserialization_error_handling() {
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        struct StrictHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for StrictHandler {
            async fn handle(
                &self,
                _context: Option<crate::envelope::Context>,
                _data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: "Should not reach here".to_string(),
                })
            }
        }

        server.receive_envelope_at("/strict_test", StrictHandler).await.unwrap();

        // ACT: Send malformed envelope structure (missing required field)
        let malformed_envelope = serde_json::json!({
            "meta": {},
            "payload": {
                "wrong_field": "this should fail"
            }
        });

        let handler_functions = Arc::clone(&server.handler_functions);
        let response = process_envelope_message(malformed_envelope, &server.config, handler_functions, "/strict_test").await;

        // ASSERT: Should return error response
        match response {
            crate::client::websocket::WebSocketMessageType::Error { message, code } => {
                assert!(message.contains("Failed to deserialize envelope data"));
                assert_eq!(code, Some(500));
            }
            _ => panic!("Expected Error response, got: {:?}", response),
        }
    }

    // TDD Step 16: NEW TEST - Test envelope with complex nested data
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_envelope_with_complex_data() {
        use crate::envelope::{Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;
        use serde::{Deserialize, Serialize};

        // ARRANGE: Define complex request/response types
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ComplexRequest {
            id: String,
            data: serde_json::Value,
            metadata: std::collections::HashMap<String, String>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ComplexResponse {
            processed_id: String,
            result: serde_json::Value,
        }

        struct ComplexHandler;

        #[async_trait]
        impl ContextDataHandler<ComplexRequest, ComplexResponse> for ComplexHandler {
            async fn handle(
                &self,
                _context: Option<crate::envelope::Context>,
                data: ComplexRequest,
            ) -> crate::error::Result<ComplexResponse> {
                Ok(ComplexResponse {
                    processed_id: data.id,
                    result: serde_json::json!({"processed": true, "original": data.data}),
                })
            }
        }

        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();
        server.receive_envelope_at("/complex_test", ComplexHandler).await.unwrap();

        // ACT: Send complex envelope structure
        let complex_request = ComplexRequest {
            id: "test-123".to_string(),
            data: serde_json::json!({"nested": {"value": 42}}),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map
            },
        };

        let envelope_request = Envelope::new(Meta::default(), complex_request);
        let envelope_data = serde_json::to_value(envelope_request).unwrap();

        let handler_functions = Arc::clone(&server.handler_functions);
        let response = process_envelope_message(envelope_data, &server.config, handler_functions, "/complex_test").await;

        // ASSERT: Handler should successfully process complex envelope
        match response {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                let envelope: Envelope<ComplexResponse> = serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.processed_id, "test-123");
                assert_eq!(envelope.payload.result["processed"], true);
                assert_eq!(envelope.payload.result["original"]["nested"]["value"], 42);
            }
            _ => panic!("Expected Envelope response, got: {:?}", response),
        }
    }

    // TDD Step 17: NEW TEST - Test envelope meta field extraction
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_envelope_meta_field_ignored() {
        use crate::envelope::{Envelope, Meta};
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        struct MetaIgnoreHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for MetaIgnoreHandler {
            async fn handle(
                &self,
                _context: Option<crate::envelope::Context>,
                data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: format!("Meta ignored: {}", data.message),
                })
            }
        }

        server.receive_envelope_at("/meta_test", MetaIgnoreHandler).await.unwrap();

        // ACT: Send envelope with complex meta information
        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.version = Some("2.0".to_string());
        
        let envelope_request = Envelope::new(
            meta,
            TestRequest {
                message: "meta test".to_string(),
            },
        );
        let envelope_data = serde_json::to_value(envelope_request).unwrap();

        let handler_functions = Arc::clone(&server.handler_functions);
        let response = process_envelope_message(envelope_data, &server.config, handler_functions, "/meta_test").await;

        // ASSERT: Handler should process data correctly, ignoring meta
        match response {
            crate::client::websocket::WebSocketMessageType::Envelope { payload } => {
                let envelope: Envelope<TestResponse> = serde_json::from_value(payload).unwrap();
                assert_eq!(envelope.payload.echo, "Meta ignored: meta test");
                // Meta should preserve original metadata when provided (test-tenant should be preserved)
                assert_eq!(envelope.meta.tenant, Some("test-tenant".to_string()));
            }
            _ => panic!("Expected Envelope response, got: {:?}", response),
        }
    }

    // TDD Step 18: NEW TEST - Test empty envelope structure handling
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_empty_envelope_structure() {
        use crate::traits::handlers::ContextDataHandler;
        use async_trait::async_trait;

        // ARRANGE: Create server and handler
        let config = WebSocketServerConfig::default();
        let mut server = WebSocketServer::new(config).await.unwrap();

        struct EmptyHandler;

        #[async_trait]
        impl ContextDataHandler<TestRequest, TestResponse> for EmptyHandler {
            async fn handle(
                &self,
                _context: Option<crate::envelope::Context>,
                _data: TestRequest,
            ) -> crate::error::Result<TestResponse> {
                Ok(TestResponse {
                    echo: "Should not reach here".to_string(),
                })
            }
        }

        server.receive_envelope_at("/empty_test", EmptyHandler).await.unwrap();

        // ACT: Send envelope with empty payload field
        let empty_envelope = serde_json::json!({
            "meta": {},
            "payload": {}
        });

        let handler_functions = Arc::clone(&server.handler_functions);
        let response = process_envelope_message(empty_envelope, &server.config, handler_functions, "/empty_test").await;

        // ASSERT: Should return error response for missing required fields
        match response {
            crate::client::websocket::WebSocketMessageType::Error { message, code } => {
                assert!(message.contains("Failed to deserialize envelope data"));
                assert_eq!(code, Some(500));
            }
            _ => panic!("Expected Error response, got: {:?}", response),
        }
    }

    // TDD Step 19: NEW TEST - Test envelope deserialization in bridge context
    #[cfg(feature = "websocket-server")]
    #[tokio::test]
    async fn test_envelope_bridge_compatibility() {
        use crate::envelope::{Envelope, Meta};

        // ARRANGE: Create handler result like MCP response
        let mcp_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "tools": [
                    {
                        "name": "test_tool",
                        "description": "A test tool"
                    }
                ]
            },
            "id": "request-123"
        });

        // ACT: Wrap in envelope (what server does)
        let envelope_response = create_response_envelope(mcp_response.clone(), None);

        // ASSERT: Bridge should be able to extract data field
        let envelope: Envelope<serde_json::Value> =
            serde_json::from_value(envelope_response).unwrap();

        // Verify bridge can access the original MCP response
        assert_eq!(envelope.payload, mcp_response);

        // Verify bridge can extract specific fields
        assert_eq!(
            envelope.payload.get("jsonrpc").unwrap().as_str().unwrap(),
            "2.0"
        );
        assert_eq!(
            envelope.payload.get("id").unwrap().as_str().unwrap(),
            "request-123"
        );

        // Verify meta fields are present for bridge processing
        assert!(envelope.meta.timestamp.is_some());
        assert_eq!(envelope.meta.tenant, None); // No tenant when no original metadata
        assert_eq!(envelope.meta.version, Some("1.0".to_string()));

        // This is the format the bridge expects and should now receive
        assert!(envelope.error.is_none()); // No error in successful response
    }
}
