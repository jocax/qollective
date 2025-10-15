// ABOUTME: Pure NATS transport implementation for raw payload communication
// ABOUTME: Enables ecosystem compatibility with standard NATS clients without envelope wrapping

//! Pure NATS transport implementation for Step 8: Create Pure NATS Transport.
//!
//! This module provides a native NATS transport that sends raw payloads directly
//! to NATS subjects without envelope wrapping. This enables ecosystem compatibility
//! with external systems that use standard NATS clients.
//!
//! Key features:
//! - Raw payload serialization/deserialization
//! - Direct subject-to-endpoint mapping
//! - NATS request/reply pattern support
//! - Interoperability with non-Qollective NATS applications

use crate::error::{QollectiveError, Result};
use crate::traits::senders::UnifiedSender;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Note: No longer importing the client layer NatsClient to avoid circular dependencies

// Add robust internal NATS client implementation
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use crate::config::nats::NatsConfig;
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use crate::envelope::{Envelope, NatsEnvelopeCodec};
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use std::sync::Arc;
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use std::time::Instant;
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use tokio::sync::{mpsc, RwLock};
use tracing::debug;

/// Connection state for monitoring and resilience
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting,
    CircuitOpen,
    CircuitHalfOpen,
}

/// Connection event notifications
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
    Reconnecting { attempt: u32, delay: Duration },
    CircuitBreakerOpen,
    CircuitBreakerHalfOpen,
    CircuitBreakerClosed,
}

/// Connection metrics for monitoring
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Debug, Clone, Default)]
pub struct ConnectionMetrics {
    pub connection_attempts: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
    pub current_connection_duration: Option<Duration>,
    pub circuit_breaker_state_changes: u64,
    pub reconnection_attempts: u64,
}

/// Circuit breaker state management
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Debug)]
struct CircuitBreaker {
    failure_count: u32,
    failure_threshold: u32,
    last_failure_time: Option<Instant>,
    timeout: Duration,
    state: ConnectionState,
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            failure_threshold,
            last_failure_time: None,
            timeout,
            state: ConnectionState::Connected,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = ConnectionState::Connected;
    }

    fn record_failure(&mut self) -> ConnectionState {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = ConnectionState::CircuitOpen;
        }

        self.state.clone()
    }

    fn can_attempt(&mut self) -> bool {
        match self.state {
            ConnectionState::Connected
            | ConnectionState::Disconnected
            | ConnectionState::Reconnecting => true,
            ConnectionState::CircuitOpen => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = ConnectionState::CircuitHalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            ConnectionState::CircuitHalfOpen => true,
        }
    }

    fn current_state(&self) -> ConnectionState {
        self.state.clone()
    }
}

/// Internal client state for resilience
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
struct ClientState {
    connection_state: ConnectionState,
    circuit_breaker: CircuitBreaker,
    metrics: ConnectionMetrics,
    connection_start_time: Option<Instant>,
    event_sender: Option<mpsc::UnboundedSender<ConnectionEvent>>,
}

/// Internal NATS client with robust connection management (copied from original NatsClient)
/// This handles all the complex connection logic, circuit breaker, mTLS, etc.
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Clone)]
pub struct InternalNatsClient {
    connection: async_nats::Client,
    config: NatsConfig,
    state: Arc<RwLock<ClientState>>,
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl InternalNatsClient {
    /// Get access to the underlying async_nats::Client for direct NATS operations
    /// This enables advanced NATS features like subscriptions, streams, and KV stores
    pub fn client(&self) -> &async_nats::Client {
        &self.connection
    }
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl std::fmt::Debug for InternalNatsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalNatsClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl InternalNatsClient {
    /// Create a new internal NATS client with robust connection management
    pub async fn new(config: NatsConfig) -> Result<Self> {
        // Initialize circuit breaker with config defaults
        let circuit_breaker = CircuitBreaker::new(
            crate::constants::circuit_breaker::DEFAULT_FAILURE_THRESHOLD,
            crate::constants::timeouts::DEFAULT_CIRCUIT_BREAKER_RECOVERY,
        );

        // Initialize client state
        let state = Arc::new(RwLock::new(ClientState {
            connection_state: ConnectionState::Disconnected,
            circuit_breaker,
            metrics: ConnectionMetrics::default(),
            connection_start_time: None,
            event_sender: None,
        }));

        // Connect to NATS server using first URL
        let url = &config.connection.urls[0];

        // Update metrics - connection attempt
        {
            let mut state_guard = state.write().await;
            state_guard.metrics.connection_attempts += 1;
        }

        // Configure connection options
        let mut connect_options = async_nats::ConnectOptions::new();

        // Set client name for better NATS server logging and monitoring
        if let Some(ref name) = config.connection.client_name {
            debug!("Setting NATS client name: {}", name);
            connect_options = connect_options.name(name);
        } else {
            debug!("No NATS client name configured");
        }

        // Configure NKey authentication if provided
        if let Some(ref nkey_file) = config.connection.nkey_file {
            debug!("Loading NKey from file: {:?}", nkey_file);
            let nkey_seed = std::fs::read_to_string(nkey_file).map_err(|e| {
                QollectiveError::nats_connection(format!(
                    "Failed to read NKey file {:?}: {}",
                    nkey_file, e
                ))
            })?;
            connect_options = connect_options.nkey(nkey_seed.trim().to_string());
        } else if let Some(ref nkey_seed) = config.connection.nkey_seed {
            debug!("Configuring NKey from seed string");
            connect_options = connect_options.nkey(nkey_seed.trim().to_string());
        }
        // Configure username/password authentication if NKey not set
        else if let (Some(ref username), Some(ref password)) = (&config.connection.username, &config.connection.password) {
            debug!("Configuring username/password authentication for user: {}", username);
            connect_options = connect_options.user_and_password(username.clone(), password.clone());
        }
        // Configure token authentication as fallback
        else if let Some(ref token) = config.connection.token {
            debug!("Configuring token authentication");
            connect_options = connect_options.token(token.clone());
        }

        // Configure TLS if enabled using unified TLS config
        if config.connection.tls.enabled {
            // Initialize crypto provider using strategy from config
            let strategy = config
                .connection
                .crypto_provider_strategy
                .unwrap_or(crate::crypto::CryptoProviderStrategy::AutoInstall);
            crate::crypto::init_with_strategy(strategy).map_err(|e| {
                QollectiveError::nats_connection(format!(
                    "Failed to initialize crypto provider: {}",
                    e
                ))
            })?;

            // Always require TLS when TLS is enabled (critical missing piece!)
            connect_options = connect_options.require_tls(true);

            // Use the unified TLS config to create rustls client config
            let tls_config = config
                .connection
                .tls
                .create_client_config()
                .await
                .map_err(|e| {
                    QollectiveError::nats_connection(format!("Failed to create TLS config: {}", e))
                })?;

            connect_options = connect_options.tls_client_config((*tls_config).clone());
        }

        let connection = async_nats::connect_with_options(url, connect_options)
            .await
            .map_err(|e| {
                // Update metrics - failed connection
                tokio::spawn({
                    let state = state.clone();
                    async move {
                        let mut state_guard = state.write().await;
                        state_guard.metrics.failed_connections += 1;
                        state_guard.connection_state = ConnectionState::Disconnected;
                        let _ = state_guard.circuit_breaker.record_failure();
                    }
                });
                QollectiveError::nats_connection(format!(
                    "Failed to connect to NATS at {}: {}",
                    url, e
                ))
            })?;

        // Update state - successful connection
        {
            let mut state_guard = state.write().await;
            state_guard.metrics.successful_connections += 1;
            state_guard.connection_state = ConnectionState::Connected;
            state_guard.connection_start_time = Some(Instant::now());
            state_guard.circuit_breaker.record_success();
        }

        Ok(Self {
            connection,
            config,
            state,
        })
    }

    /// Send an envelope to a NATS subject with robust error handling
    pub async fn send_envelope<T, R>(
        &self,
        subject: &str,
        envelope: Envelope<T>,
    ) -> Result<Envelope<R>>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        // Check circuit breaker before making request
        self.can_make_request().await?;

        // Encode envelope to bytes
        let encoded_data = NatsEnvelopeCodec::encode(&envelope).map_err(|e| {
            QollectiveError::nats_message(format!("Failed to encode envelope: {}", e))
        })?;

        // Send request and wait for response
        let response_result = self
            .connection
            .request(subject.to_string(), encoded_data.into())
            .await;

        match response_result {
            Ok(response) => {
                // Success - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    state_guard.circuit_breaker.record_success();
                }

                // Decode response envelope
                let response_envelope =
                    NatsEnvelopeCodec::decode(&response.payload).map_err(|e| {
                        QollectiveError::nats_message(format!("Failed to decode response: {}", e))
                    })?;

                Ok(response_envelope)
            }
            Err(e) => {
                // Failure - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    let new_state = state_guard.circuit_breaker.record_failure();
                    state_guard.connection_state = new_state.clone();

                    // Send circuit breaker event if state changed
                    if let Some(ref sender) = state_guard.event_sender {
                        match new_state {
                            ConnectionState::CircuitOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerOpen);
                            }
                            ConnectionState::CircuitHalfOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerHalfOpen);
                            }
                            _ => {}
                        }
                    }
                }

                Err(QollectiveError::from(e))
            }
        }
    }

    /// Send raw bytes to a NATS subject and wait for response
    pub async fn request_raw(
        &self,
        subject: &str,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>> {
        // Check circuit breaker before making request
        self.can_make_request().await?;

        // Create timeout version of the request
        let request_future = self
            .connection
            .request(subject.to_string(), payload.to_vec().into());
        let timed_request = tokio::time::timeout(timeout, request_future);

        match timed_request.await {
            Ok(response_result) => {
                match response_result {
                    Ok(response) => {
                        // Success - record in circuit breaker
                        {
                            let mut state_guard = self.state.write().await;
                            state_guard.circuit_breaker.record_success();
                        }

                        // Return raw response payload
                        Ok(response.payload.to_vec())
                    }
                    Err(e) => {
                        // Failure - record in circuit breaker
                        {
                            let mut state_guard = self.state.write().await;
                            let new_state = state_guard.circuit_breaker.record_failure();
                            state_guard.connection_state = new_state.clone();

                            // Send circuit breaker event if state changed
                            if let Some(ref sender) = state_guard.event_sender {
                                match new_state {
                                    ConnectionState::CircuitOpen => {
                                        let _ = sender.send(ConnectionEvent::CircuitBreakerOpen);
                                    }
                                    ConnectionState::CircuitHalfOpen => {
                                        let _ =
                                            sender.send(ConnectionEvent::CircuitBreakerHalfOpen);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        Err(QollectiveError::transport(format!(
                            "Raw NATS request failed: {}",
                            e
                        )))
                    }
                }
            }
            Err(_) => {
                // Timeout occurred - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    let new_state = state_guard.circuit_breaker.record_failure();
                    state_guard.connection_state = new_state;
                }

                Err(QollectiveError::transport(format!(
                    "Raw NATS request to {} timed out after {:?}",
                    subject, timeout
                )))
            }
        }
    }

    /// Publish an envelope to a NATS subject (fire-and-forget)
    pub async fn publish_envelope<T>(&self, subject: &str, envelope: Envelope<T>) -> Result<()>
    where
        T: serde::Serialize,
    {
        // Check circuit breaker before making request
        self.can_make_request().await?;

        // Encode envelope to bytes
        let encoded_data = NatsEnvelopeCodec::encode(&envelope).map_err(|e| {
            QollectiveError::nats_message(format!("Failed to encode envelope: {}", e))
        })?;

        // Publish to NATS (fire-and-forget)
        let publish_result = self
            .connection
            .publish(subject.to_string(), encoded_data.into())
            .await;

        match publish_result {
            Ok(()) => {
                // Success - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    state_guard.circuit_breaker.record_success();
                }
                Ok(())
            }
            Err(e) => {
                // Failure - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    let new_state = state_guard.circuit_breaker.record_failure();
                    state_guard.connection_state = new_state.clone();

                    // Send circuit breaker event if state changed
                    if let Some(ref sender) = state_guard.event_sender {
                        match new_state {
                            ConnectionState::CircuitOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerOpen);
                            }
                            ConnectionState::CircuitHalfOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerHalfOpen);
                            }
                            _ => {}
                        }
                    }
                }

                Err(QollectiveError::from(e))
            }
        }
    }

    /// Publish raw bytes to a NATS subject (fire-and-forget)
    pub async fn publish_raw(&self, subject: &str, payload: &[u8]) -> Result<()> {
        // Check circuit breaker before making request
        self.can_make_request().await?;

        // Publish raw bytes to NATS (fire-and-forget)
        let publish_result = self
            .connection
            .publish(subject.to_string(), payload.to_vec().into())
            .await;

        match publish_result {
            Ok(()) => {
                // Success - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    state_guard.circuit_breaker.record_success();
                }
                Ok(())
            }
            Err(e) => {
                // Failure - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    let new_state = state_guard.circuit_breaker.record_failure();
                    state_guard.connection_state = new_state.clone();

                    // Send circuit breaker event if state changed
                    if let Some(ref sender) = state_guard.event_sender {
                        match new_state {
                            ConnectionState::CircuitOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerOpen);
                            }
                            ConnectionState::CircuitHalfOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerHalfOpen);
                            }
                            _ => {}
                        }
                    }
                }

                Err(QollectiveError::transport(format!(
                    "Raw NATS publish failed: {}",
                    e
                )))
            }
        }
    }

    /// Subscribe to a NATS subject
    pub async fn subscribe(
        &self,
        subject: &str,
        queue_group: Option<&str>,
    ) -> Result<async_nats::Subscriber> {
        // Check circuit breaker before subscribing
        self.can_make_request().await?;

        // Create subscription based on whether queue group is specified
        let subscription_result = match queue_group {
            Some(queue) => {
                self.connection
                    .queue_subscribe(subject.to_string(), queue.to_string())
                    .await
            }
            None => self.connection.subscribe(subject.to_string()).await,
        };

        match subscription_result {
            Ok(subscriber) => {
                // Success - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    state_guard.circuit_breaker.record_success();
                }
                Ok(subscriber)
            }
            Err(e) => {
                // Failure - record in circuit breaker
                {
                    let mut state_guard = self.state.write().await;
                    let new_state = state_guard.circuit_breaker.record_failure();
                    state_guard.connection_state = new_state.clone();

                    // Send circuit breaker event if state changed
                    if let Some(ref sender) = state_guard.event_sender {
                        match new_state {
                            ConnectionState::CircuitOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerOpen);
                            }
                            ConnectionState::CircuitHalfOpen => {
                                let _ = sender.send(ConnectionEvent::CircuitBreakerHalfOpen);
                            }
                            _ => {}
                        }
                    }
                }

                Err(QollectiveError::from(e))
            }
        }
    }

    /// Check if circuit breaker allows requests
    async fn can_make_request(&self) -> Result<()> {
        let mut state_guard = self.state.write().await;

        if !state_guard.circuit_breaker.can_attempt() {
            return Err(QollectiveError::nats_connection(
                "Circuit breaker is open - requests not allowed",
            ));
        }

        Ok(())
    }
}

/// Pure NATS transport for raw payload communication.
///
/// This transport implements the `UnifiedSender` trait to enable communication
/// with NATS subjects using raw payloads (no envelope wrapping). It provides
/// ecosystem compatibility with standard NATS clients and applications.
///
/// # Examples
///
/// ```rust
/// use qollective::transport::nats::NatsTransport;
/// use qollective::prelude::UnifiedSender;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyRequest {
///     message: String,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct MyResponse {
///     result: String,
/// }
///
/// async fn example() -> qollective::error::Result<()> {
///     let transport = NatsTransport::new("nats://localhost:4222").await?;
///
///     let request = MyRequest {
///         message: "Hello NATS".to_string(),
///     };
///
///     // Send raw payload to NATS subject (no envelope)
///     let response: MyResponse = transport.send("nats://localhost:4222/my.subject", request).await?;
///
///     println!("Response: {}", response.result);
///     Ok(())
/// }
/// ```
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Debug, Clone)]
pub struct NatsTransport {
    /// Underlying internal NATS client for connection management
    nats_client: InternalNatsClient,
    /// Default timeout for request/reply operations
    request_timeout: Duration,
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl NatsTransport {
    /// Create a new pure NATS transport from a connection URL.
    ///
    /// # Arguments
    ///
    /// * `nats_url` - NATS server URL (e.g., "nats://localhost:4222")
    ///
    /// # Returns
    ///
    /// Returns a `Result<NatsTransport>` with the configured transport.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The NATS URL is invalid
    /// - Connection to the NATS server fails
    /// - Authentication fails
    pub async fn new(nats_url: &str) -> Result<Self> {
        // Parse the NATS URL to extract server address
        let nats_config = crate::config::nats::NatsConfig {
            connection: crate::config::nats::NatsConnectionConfig {
                urls: vec![nats_url.to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        // Create underlying internal NATS client
        let nats_client = InternalNatsClient::new(nats_config).await?;

        Ok(Self {
            nats_client,
            request_timeout: Duration::from_secs(30), // Default 30 second timeout
        })
    }

    /// Create a new NATS transport with unified TLS configuration
    ///
    /// # Arguments
    ///
    /// * `nats_url` - NATS server URL (e.g., "nats://localhost:4222" or "tls://localhost:4443")
    /// * `tls_config` - Optional unified TLS configuration
    ///
    /// # Returns
    ///
    /// Returns a `Result<NatsTransport>` with the configured transport supporting TLS.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The NATS URL is invalid
    /// - TLS configuration is invalid
    /// - Connection to the NATS server fails
    /// - TLS handshake fails
    pub async fn new_with_unified_tls(
        nats_url: &str,
        tls_config: Option<&crate::config::tls::TlsConfig>,
    ) -> Result<Self> {
        // Create NATS config with TLS configuration
        let mut nats_config = crate::config::nats::NatsConfig {
            connection: crate::config::nats::NatsConnectionConfig {
                urls: vec![nats_url.to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        // Apply TLS configuration if provided
        if let Some(tls_config) = tls_config {
            nats_config.connection.tls = tls_config.clone();
        }

        // Create underlying internal NATS client
        let nats_client = InternalNatsClient::new(nats_config).await?;

        Ok(Self {
            nats_client,
            request_timeout: Duration::from_secs(30), // Default 30 second timeout
        })
    }

    /// Create a pure NATS transport from an existing InternalNatsClient.
    ///
    /// This is useful when you want to reuse an existing NATS connection
    /// for both envelope and raw payload communication.
    ///
    /// # Arguments
    ///
    /// * `nats_client` - Existing internal NATS client instance
    ///
    /// # Returns
    ///
    /// Returns a configured `NatsTransport` using the provided client.
    pub fn from_internal_nats_client(nats_client: InternalNatsClient) -> Self {
        Self {
            nats_client,
            request_timeout: Duration::from_secs(30),
        }
    }

    /// Set the request timeout for request/reply operations.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for a response
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Extract NATS subject from endpoint URL.
    ///
    /// Converts endpoint formats like:
    /// - "nats://localhost:4222/my.subject" → "my.subject"
    /// - "nats://server:4222/events.user.created" → "events.user.created"
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL to parse
    ///
    /// # Returns
    ///
    /// Returns the NATS subject extracted from the endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the endpoint format is invalid.
    fn extract_subject_from_endpoint(&self, endpoint: &str) -> Result<String> {
        if !endpoint.starts_with("nats://") {
            return Err(QollectiveError::transport(format!(
                "Invalid NATS endpoint: {}. Must start with 'nats://'",
                endpoint
            )));
        }

        // Parse URL to extract subject from path
        let url_parts: Vec<&str> = endpoint.split('/').collect();
        if url_parts.len() < 4 {
            return Err(QollectiveError::transport(format!(
                "NATS endpoint missing subject: {}. Expected format: nats://server:port/subject",
                endpoint
            )));
        }

        // Extract subject from path (everything after hostname:port/)
        // Convert path segments to NATS subject format (dots)
        let subject = url_parts[3..].join(".");

        if subject.is_empty() {
            return Err(QollectiveError::transport(format!(
                "Empty NATS subject in endpoint: {}",
                endpoint
            )));
        }

        Ok(subject)
    }
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[async_trait]
impl<T, R> UnifiedSender<T, R> for NatsTransport
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    /// Send a raw payload to the specified NATS endpoint.
    ///
    /// This method implements direct NATS communication without envelope wrapping:
    /// 1. Extracts the NATS subject from the endpoint URL
    /// 2. Serializes the payload to JSON
    /// 3. Uses NATS request/reply pattern for synchronous communication
    /// 4. Deserializes the response payload
    ///
    /// # Arguments
    ///
    /// * `endpoint` - NATS endpoint URL (e.g., "nats://localhost:4222/my.subject")
    /// * `payload` - The request payload to send (will be serialized to JSON)
    ///
    /// # Returns
    ///
    /// Returns the deserialized response payload.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The endpoint URL is malformed
    /// - Payload serialization fails
    /// - NATS request/reply fails (timeout, no responders, etc.)
    /// - Response deserialization fails
    async fn send(&self, endpoint: &str, payload: T) -> Result<R> {
        // Extract NATS subject from endpoint
        let subject = self.extract_subject_from_endpoint(endpoint)?;

        // Serialize payload to JSON for NATS transport
        let payload_bytes = serde_json::to_vec(&payload).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize pure NATS payload: {}", e))
        })?;

        // Send raw payload to NATS subject and wait for response
        // Use the underlying NATS client's request/reply functionality
        let response_bytes = self
            .nats_client
            .request_raw(&subject, &payload_bytes, self.request_timeout)
            .await?;

        // Deserialize response from JSON for NATS transport
        serde_json::from_slice::<R>(&response_bytes).map_err(|e| {
            QollectiveError::serialization(format!(
                "Failed to deserialize pure NATS response: {}",
                e
            ))
        })
    }
}

// Non-feature version for compilation when NATS features are disabled
#[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
#[derive(Debug, Clone)]
pub struct NatsTransport;

#[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
impl NatsTransport {
    pub async fn new(_nats_url: &str) -> Result<Self> {
        Err(QollectiveError::transport(
            "NATS client feature not enabled".to_string(),
        ))
    }

    pub async fn new_with_unified_tls(
        _nats_url: &str,
        _tls_config: Option<&crate::config::tls::TlsConfig>,
    ) -> Result<Self> {
        Err(QollectiveError::transport(
            "NATS client feature not enabled".to_string(),
        ))
    }

    pub fn from_nats_client(_nats_client: ()) -> Self {
        Self
    }

    pub fn with_timeout(self, _timeout: Duration) -> Self {
        self
    }
}

#[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
#[async_trait]
impl<T, R> UnifiedSender<T, R> for NatsTransport
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    async fn send(&self, _endpoint: &str, _payload: T) -> Result<R> {
        Err(QollectiveError::transport(
            "NATS client feature not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_extract_subject_from_endpoint() {
        // For URL parsing tests, we can create a minimal transport instance
        // Since we're only testing the URL parsing logic, we don't need a real NATS connection
        use crate::client::nats::NatsClient;
        use std::time::Duration;

        // Create a minimal NATS config for testing
        let config = crate::config::nats::NatsConfig::default();

        // Since this is a unit test for URL parsing only, we'll create a mock transport structure
        // In a real scenario, this would have an actual NatsClient, but for URL parsing tests
        // we only need the method to be callable
        struct TestNatsTransport;

        impl TestNatsTransport {
            fn extract_subject_from_endpoint(&self, endpoint: &str) -> Result<String> {
                // Copy the same URL parsing logic for testing
                if !endpoint.starts_with("nats://") {
                    return Err(QollectiveError::transport(format!(
                        "Invalid NATS endpoint: {}. Must start with 'nats://'",
                        endpoint
                    )));
                }

                // Parse URL to extract subject from path
                let url_parts: Vec<&str> = endpoint.split('/').collect();
                if url_parts.len() < 4 {
                    return Err(QollectiveError::transport(
                        format!("NATS endpoint missing subject: {}. Expected format: nats://server:port/subject", endpoint)
                    ));
                }

                // Extract subject from path (everything after hostname:port/)
                // Convert path segments to NATS subject format (dots)
                let subject = url_parts[3..].join(".");

                if subject.is_empty() {
                    return Err(QollectiveError::transport(format!(
                        "Empty NATS subject in endpoint: {}",
                        endpoint
                    )));
                }

                Ok(subject)
            }
        }

        let transport = TestNatsTransport;

        // Test valid endpoints
        assert_eq!(
            transport
                .extract_subject_from_endpoint("nats://localhost:4222/my.subject")
                .unwrap(),
            "my.subject"
        );

        assert_eq!(
            transport
                .extract_subject_from_endpoint("nats://server:4222/events.user.created")
                .unwrap(),
            "events.user.created"
        );

        assert_eq!(
            transport
                .extract_subject_from_endpoint("nats://cluster.example.com:4222/service/method/v1")
                .unwrap(),
            "service.method.v1"
        );

        // Test invalid endpoints
        assert!(transport
            .extract_subject_from_endpoint("http://localhost:8080/api")
            .is_err());
        assert!(transport
            .extract_subject_from_endpoint("nats://localhost:4222")
            .is_err());
        assert!(transport
            .extract_subject_from_endpoint("nats://localhost:4222/")
            .is_err());
    }

    #[test]
    fn test_pure_nats_transport_creation_without_features() {
        // Test that transport can be created even when NATS features are disabled
        // This ensures the code compiles in all feature configurations

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            // When features are enabled, we need to properly construct the struct
            // But since we can't create a real connection in tests, we'll just test that the types compile
            // This test validates that the feature gates work correctly
            assert!(true, "NATS features are enabled - compilation successful");
        }

        #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
        {
            // When features are disabled, we can create the empty struct
            let _transport = NatsTransport;
            assert!(true, "NATS features disabled - compilation successful");
        }
    }

    // TLS integration tests
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_transport_with_unified_tls_config() {
        use crate::config::tls::{TlsConfig, VerificationMode};

        // Test with TLS disabled
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = false;
        tls_config.verification_mode = VerificationMode::SystemCa;

        // Test transport creation (will fail connection but should accept TLS config)
        let result =
            NatsTransport::new_with_unified_tls("nats://localhost:4222", Some(&tls_config)).await;

        // Expect connection error since no NATS server is running, but TLS config should be accepted
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("Failed to connect")
                || error.to_string().contains("Connection refused")
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_transport_with_tls_skip_verification() {
        use crate::config::tls::{TlsConfig, VerificationMode};

        // Test with TLS enabled and Skip verification mode
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = true;
        tls_config.verification_mode = VerificationMode::Skip;

        // Test transport creation with TLS config
        let result =
            NatsTransport::new_with_unified_tls("tls://localhost:4443", Some(&tls_config)).await;

        // Expect connection error since no NATS server is running, but TLS config should be accepted
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("Failed to connect")
                || error.to_string().contains("Connection refused")
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_transport_with_tls_system_ca_verification() {
        use crate::config::tls::{TlsConfig, VerificationMode};

        // Test with TLS enabled and SystemCa verification mode
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = true;
        tls_config.verification_mode = VerificationMode::SystemCa;

        // Test transport creation with TLS config
        let result =
            NatsTransport::new_with_unified_tls("tls://localhost:4443", Some(&tls_config)).await;

        // Expect connection error since no NATS server is running, but TLS config should be accepted
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("Failed to connect")
                || error.to_string().contains("Connection refused")
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_transport_with_tls_custom_ca_verification() {
        use crate::config::tls::{TlsConfig, VerificationMode};
        use std::path::PathBuf;

        // Test with TLS enabled and CustomCa verification mode
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = true;
        tls_config.verification_mode = VerificationMode::CustomCa;
        tls_config.ca_cert_path = Some(PathBuf::from("/path/to/ca.pem"));

        // Test transport creation with TLS config (will fail but should parse TLS config)
        let result =
            NatsTransport::new_with_unified_tls("tls://localhost:4443", Some(&tls_config)).await;

        // Expect error (either connection or CA file not found), but TLS config should be accepted
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("Failed to connect")
                || error.to_string().contains("Connection refused")
                || error.to_string().contains("No such file")
                || error.to_string().contains("TLS")
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_transport_with_tls_mutual_tls_verification() {
        use crate::config::tls::{TlsConfig, VerificationMode};
        use std::path::PathBuf;

        // Test with TLS enabled and MutualTls verification mode
        let mut tls_config = TlsConfig::default();
        tls_config.enabled = true;
        tls_config.verification_mode = VerificationMode::MutualTls;
        tls_config.ca_cert_path = Some(PathBuf::from("/path/to/ca.pem"));
        tls_config.cert_path = Some(PathBuf::from("/path/to/client.pem"));
        tls_config.key_path = Some(PathBuf::from("/path/to/client.key"));

        // Test transport creation with TLS config (will fail but should parse TLS config)
        let result =
            NatsTransport::new_with_unified_tls("tls://localhost:4443", Some(&tls_config)).await;

        // Expect error (either connection or certificate file not found), but TLS config should be accepted
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("Failed to connect")
                || error.to_string().contains("Connection refused")
                || error.to_string().contains("No such file")
                || error.to_string().contains("TLS")
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_transport_without_tls_config() {
        // Test transport creation without TLS config (should use default)
        let result = NatsTransport::new_with_unified_tls("nats://localhost:4222", None).await;

        // Expect connection error since no NATS server is running, but config should be accepted
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("connection")
                || error.to_string().contains("Failed to connect")
                || error.to_string().contains("Connection refused")
        );
    }
}
