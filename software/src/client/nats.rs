// ABOUTME: NATS client implementation for Qollective envelope-based messaging (transformed for dependency injection)
// ABOUTME: Provides request/reply and publish patterns through transport layer delegation

//! NATS client implementation for Qollective framework.
//!
//! This module provides a NATS client that integrates with the Qollective envelope system,
//! offering request/reply patterns, publish functionality, and connection management through
//! the transport layer.

use crate::error::{QollectiveError, Result};

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use crate::config::nats::NatsConfig;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use crate::envelope::Envelope;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use std::sync::Arc;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use std::time::Duration;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use tokio::sync::mpsc;

// Connection types from the transport layer
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use crate::transport::nats::{ConnectionEvent, ConnectionMetrics, ConnectionState};

#[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
#[derive(Clone)]
pub struct NatsClient;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Clone)]
pub struct NatsClient {
    transport: Arc<crate::transport::HybridTransportClient>,
}

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl std::fmt::Debug for NatsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NatsClient")
            .field("transport", &self.transport)
            .finish_non_exhaustive()
    }
}

impl NatsClient {
    /// Create a NATS client with dependency injection for testing
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_transport(transport: Arc<crate::transport::HybridTransportClient>) -> Result<Self> {
        Ok(Self { transport })
    }

    /// Create a NATS client with its own transport layer (NEW API - preferred)
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn new(config: NatsConfig) -> Result<Self> {
        // Create transport configuration from NATS client config (CONFIG FIRST PRINCIPLE)
        let transport_config = crate::transport::TransportDetectionConfig {
            enable_auto_detection: true,
            detection_timeout: std::time::Duration::from_millis(config.client.request_timeout_ms),
            capability_cache_ttl: std::time::Duration::from_millis(config.discovery.ttl_ms),
            retry_failed_detections: config.client.retry_attempts > 0,
            max_detection_retries: config.client.retry_attempts,
        };

        // Create transport with NATS client injected
        let mut transport = crate::transport::HybridTransportClient::new(transport_config);

        // Convert client config to full config for internal NATS client
        let full_config = NatsConfig {
            connection: config.connection,
            client: config.client,
            server: crate::config::nats::NatsServerConfig::default(), // Not needed for client
            discovery: crate::config::nats::NatsDiscoveryConfig {
                enabled: true,
                ttl_ms: config.discovery.ttl_ms,
                ..Default::default()
            },
        };

        // Create the actual internal NATS client that the transport will use
        let internal_nats_client =
            crate::transport::nats::InternalNatsClient::new(full_config).await?;
        transport = transport.with_internal_nats_client(Arc::new(internal_nats_client));

        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn new(_config: ()) -> Result<Self> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    /// Create a mock NATS client for testing
    #[cfg(test)]
    pub fn new_mock() -> Self {
        // For now, just create the basic struct for testing interfaces
        #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
        {
            Self
        }

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            // For feature-enabled builds, we can't easily mock the connection
            // This will be used only for interface testing
            panic!("Mock NATS client not implemented for feature-enabled builds")
        }
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn send_envelope<T, R>(
        &self,
        subject: &str,
        envelope: Envelope<T>,
    ) -> Result<Envelope<R>>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        // Delegate to transport layer - get internal NATS client and call its method
        if let Some(nats_client) = self.transport.internal_nats_client() {
            nats_client.send_envelope(subject, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No NATS client configured in transport layer",
            ))
        }
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn send_envelope<T, R>(&self, _subject: &str, _envelope: T) -> Result<R> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn publish<T>(&self, subject: &str, envelope: Envelope<T>) -> Result<()>
    where
        T: serde::Serialize,
    {
        // Delegate to transport layer - use the new publish_envelope method
        if let Some(nats_client) = self.transport.internal_nats_client() {
            nats_client.publish_envelope(subject, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No NATS client configured in transport layer",
            ))
        }
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn publish<T>(&self, _subject: &str, _envelope: T) -> Result<()> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    /// Publish message to queue group (automatically load balanced by NATS)
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn publish_to_queue_group<T>(
        &self,
        subject: &str,
        envelope: Envelope<T>,
    ) -> Result<()>
    where
        T: serde::Serialize,
    {
        // Queue groups work at subscription level, so this is the same as regular publish
        // NATS will automatically load balance to subscribers in the queue group
        self.publish(subject, envelope).await
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn publish_to_queue_group<T>(
        &self,
        _subject: &str,
        _envelope: Envelope<T>,
    ) -> Result<()> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    /// Publish raw bytes to a NATS subject (for ecosystem compatibility)
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn publish_raw(&self, subject: &str, payload: &[u8]) -> Result<()> {
        // Delegate to transport layer - use the new publish_raw method
        if let Some(nats_client) = self.transport.internal_nats_client() {
            nats_client.publish_raw(subject, payload).await
        } else {
            Err(QollectiveError::transport(
                "No NATS client configured in transport layer",
            ))
        }
    }

    /// Send raw bytes to a NATS subject and wait for response
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn request_raw(
        &self,
        subject: &str,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>> {
        // Delegate to transport layer - use internal NATS client
        if let Some(nats_client) = self.transport.internal_nats_client() {
            nats_client.request_raw(subject, payload, timeout).await
        } else {
            Err(QollectiveError::transport(
                "No NATS client configured in transport layer",
            ))
        }
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn publish_raw(&self, _subject: &str, _payload: &[u8]) -> Result<()> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn request_raw(
        &self,
        _subject: &str,
        _payload: &[u8],
        _timeout: Duration,
    ) -> Result<Vec<u8>> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    /// Get current connection state
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn connection_state(&self) -> ConnectionState {
        // Delegate to transport layer
        ConnectionState::Disconnected // Temporary stub
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn connection_state(&self) -> ConnectionState {
        ConnectionState::Disconnected
    }

    /// Check if connection is healthy
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn is_healthy(&self) -> bool {
        // Delegate to transport layer
        false // Temporary stub
    }

    /// Subscribe to a NATS subject
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn subscribe(
        &self,
        subject: &str,
        queue_group: Option<&str>,
    ) -> Result<async_nats::Subscriber> {
        // Delegate to transport layer - use internal NATS client
        if let Some(nats_client) = self.transport.internal_nats_client() {
            nats_client.subscribe(subject, queue_group).await
        } else {
            Err(QollectiveError::transport(
                "No NATS client configured in transport layer",
            ))
        }
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn subscribe(&self, _subject: &str, _queue_group: Option<&str>) -> Result<()> {
        Err(QollectiveError::feature_not_enabled(
            "NATS client requires nats-client or nats-server feature",
        ))
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn is_healthy(&self) -> bool {
        false
    }

    /// Get connection metrics
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn connection_metrics(&self) -> ConnectionMetrics {
        // Delegate to transport layer
        ConnectionMetrics::default() // Temporary stub
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn connection_metrics(&self) -> ConnectionMetrics {
        ConnectionMetrics::default()
    }

    /// Subscribe to connection events
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn connection_events(&self) -> mpsc::UnboundedReceiver<ConnectionEvent> {
        // Delegate to transport layer
        let (_, receiver) = mpsc::unbounded_channel();
        receiver // Temporary stub
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    pub async fn connection_events(&self) -> mpsc::UnboundedReceiver<ConnectionEvent> {
        let (_, receiver) = mpsc::unbounded_channel();
        receiver
    }

    pub async fn disconnect(&self) -> Result<()> {
        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            // Delegate to transport layer
            Err(QollectiveError::transport(
                "NATS client disconnect not yet implemented with transport delegation",
            ))
        }
        #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
        {
            Err(QollectiveError::feature_not_enabled(
                "NATS client requires nats-client or nats-server feature",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::nats::NatsConfig;
    use crate::envelope::{Envelope, Meta};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestRequest {
        message: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestResponse {
        reply: String,
    }

    fn create_test_envelope<T>(data: T) -> Envelope<T> {
        Envelope {
            meta: Meta {
                timestamp: Some(chrono::Utc::now()),
                request_id: Some(uuid::Uuid::now_v7()),
                version: Some("1.0.0".to_string()),
                duration: None,
                tenant: Some("test-tenant".to_string()),
                on_behalf_of: None,
                security: None,
                debug: None,
                performance: None,
                monitoring: None,
                tracing: None,
                extensions: None,
            },
            payload: data,
            error: None,
        }
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_client_creation_with_valid_config() {
        // ARRANGE: Create valid NATS config
        let config = NatsConfig::default();

        // ACT: Create NATS client (will fail without running NATS server)
        let result = NatsClient::new(config).await;

        // ASSERT: Should fail connecting to NATS server (not running)
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to connect to NATS"));
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    #[tokio::test]
    async fn test_nats_client_requires_feature_flags() {
        // ACT: Attempt to create client without feature flags
        let result = NatsClient::new(()).await;

        // ASSERT: Should fail with feature not enabled error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("feature"));
    }

    // NEW DEPENDENCY INJECTION TESTS (TDD - These should now pass with the dependency injection pattern)

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_client_with_transport_constructor() {
        // ARRANGE: Create mock transport
        use crate::transport::{HybridTransportClient, TransportDetectionConfig};
        let transport = std::sync::Arc::new(HybridTransportClient::new(
            TransportDetectionConfig::default(),
        ));

        // ACT: Create NATS client with injected transport
        let result = NatsClient::with_transport(transport);

        // ASSERT: Client should be created successfully
        assert!(result.is_ok());
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_client_send_envelope_uses_transport() {
        // ARRANGE: Create mock transport and client
        use crate::transport::{HybridTransportClient, TransportDetectionConfig};
        let transport = std::sync::Arc::new(HybridTransportClient::new(
            TransportDetectionConfig::default(),
        ));
        let client = NatsClient::with_transport(transport).unwrap();

        // Create test envelope
        let request = TestRequest {
            message: "test".to_string(),
        };
        let envelope = create_test_envelope(request);

        // ACT: Try to send envelope (should return error saying no NATS client configured)
        let result: Result<Envelope<TestResponse>> =
            client.send_envelope("test.subject", envelope).await;

        // ASSERT: Should return our delegation error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No NATS client configured"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_client_request_raw_uses_transport() {
        // ARRANGE: Create mock transport and client
        use crate::transport::{HybridTransportClient, TransportDetectionConfig};
        use std::time::Duration;
        let transport = std::sync::Arc::new(HybridTransportClient::new(
            TransportDetectionConfig::default(),
        ));
        let client = NatsClient::with_transport(transport).unwrap();

        // ACT: Try to send raw request (should return error saying no NATS client configured)
        let result = client
            .request_raw("test.subject", b"test data", Duration::from_secs(5))
            .await;

        // ASSERT: Should return our delegation error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No NATS client configured"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[tokio::test]
    async fn test_nats_client_new_creates_own_transport() {
        // ARRANGE: Create config
        let config = NatsConfig::default();

        // ACT: Create client with new() method (should still work after refactor)
        let result = NatsClient::new(config).await;

        // ASSERT: Should fail connecting but method signature should work
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to connect to NATS"));
    }
}
