// ABOUTME: A2A (Agent-to-Agent) transport implementation with dual architecture support
// ABOUTME: Supports both HTTP-based standard protocol and NATS-only communication for Qollective framework

//! A2A (Agent-to-Agent) Transport Implementation
//!
//! ## Architecture Overview
//!
//! This module supports two distinct A2A communication approaches:
//!
//! ### 1. A2A Standard (HTTP) - `a2a-standard` feature
//! - Uses external `a2a-rs` crate with HTTP transport
//! - Requires HTTP server endpoints for agent communication
//! - For interoperability with external A2A protocol implementations
//! - Example: `HttpClient::new("http://agent:8080")`
//! - Use case: Integration with external A2A systems that expect standard HTTP endpoints
//!
//! ### 2. A2A NATS - Default behavior without `a2a-standard`
//! - Direct NATS subject-based communication
//! - No HTTP servers required
//! - Optimized for internal Enterprise agent architectures
//! - Example: NATS subjects like `enterprise.bridge.challenge`
//! - Use case: High-performance internal agent coordination via NATS messaging
//!
//! ## Feature Flag Selection
//!
//! - `a2a` - Core A2A with NATS-only transport (no HTTP endpoints required)
//! - `a2a-full` - Complete A2A with both NATS and HTTP transport options
//! - `a2a-standard` - HTTP-only transport using `a2a-rs` crate
//!
//! ## Usage Guidelines
//!
//! **For Internal Agent Communication (Enterprise Examples):**
//! ```toml
//! qollective = { features = ["nats", "a2a"] }  # NATS-only, no HTTP overhead
//! ```
//!
//! **For External A2A Interoperability:**
//! ```toml
//! qollective = { features = ["nats", "a2a-full"] }  # Includes HTTP endpoints
//! ```

use crate::config::a2a::A2AClientConfig;
use crate::envelope::{Envelope, Meta};
use crate::error::{QollectiveError, Result};
use crate::traits::senders::{UnifiedEnvelopeSender, UnifiedSender};
use crate::types::a2a::AgentInfo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Use a2a-rs standard types and client
#[cfg(feature = "a2a-standard")]
use a2a_rs::{HttpClient, Message, Task};
#[cfg(feature = "a2a-standard")]
use a2a_rs::services::client::AsyncA2AClient;

/// A2A transport client using a2a-rs standard implementation
pub struct InternalA2AClient {
    /// A2A configuration
    config: A2AClientConfig,
    /// Standard a2a-rs HTTP client
    #[cfg(feature = "a2a-standard")]
    a2a_client: Option<HttpClient>,
    /// Agent registry for discovered agents
    agent_registry: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// Local agent identifier
    local_agent_id: String,
    /// Transport connection status
    is_connected: bool,
}

impl InternalA2AClient {
    /// Create A2A client with configuration (only way to instantiate)
    pub async fn new(config: A2AClientConfig) -> Result<Self> {
        let local_agent_id = config.client.agent_id.clone();

        // Initialize a2a-rs HTTP client using the configured endpoint (only if standard feature is enabled)
        #[cfg(feature = "a2a-standard")]
        let a2a_client = if let Some(ref endpoint) = config.client.endpoint {
            Some(HttpClient::new(endpoint.clone()))
        } else {
            return Err(QollectiveError::config(
                "A2A endpoint is required for standard protocol".to_string()
            ));
        };

        // For NATS-only A2A (no standard HTTP), we don't need an HTTP client
        #[cfg(not(feature = "a2a-standard"))]
        let _unused_endpoint = &config.client.endpoint; // Acknowledge but don't require endpoint

        Ok(Self {
            config,
            #[cfg(feature = "a2a-standard")]
            a2a_client,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            local_agent_id,
            is_connected: true,
        })
    }

    /// Create A2A client for testing (no HTTP client)
    pub fn mock() -> Self {
        Self {
            config: A2AClientConfig::default(),
            #[cfg(feature = "a2a-standard")]
            a2a_client: None,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            local_agent_id: "mock-agent".to_string(),
            is_connected: true, // Set to true for testing so mock transport can handle requests
        }
    }

    /// Create disconnected A2A client for testing error handling
    pub fn mock_disconnected() -> Self {
        Self {
            config: A2AClientConfig::default(),
            #[cfg(feature = "a2a-standard")]
            a2a_client: None,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            local_agent_id: "mock-agent".to_string(),
            is_connected: false, // Disconnected for error testing
        }
    }

    /// Simple agent discovery using a2a-rs (no NATS required)
    pub async fn discover_agents(&self, _capabilities: &[String]) -> Result<Vec<AgentInfo>> {
        if !self.is_connected {
            return Err(QollectiveError::transport(
                "A2A transport not connected".to_string(),
            ));
        }

        // With a2a-rs, we don't need complex agent discovery
        // The HTTP client handles endpoint routing directly
        let registry = self.agent_registry.read().await;
        Ok(registry.values().cloned().collect())
    }

    /// Convert qollective envelope to a2a-rs message format
    #[cfg(feature = "a2a-standard")]
    fn envelope_to_a2a_message<T: Serialize>(&self, envelope: &Envelope<T>) -> Result<Message> {
        // Extract envelope data for conversion
        let data_json = serde_json::to_value(&envelope.payload).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize envelope data: {}", e))
        })?;

        // Create a2a-rs message with user role (most common case)
        let message_id = envelope.meta.request_id.map(|id| id.to_string())
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let mut message = Message::user_text(data_json.to_string(), message_id);

        // Add metadata from envelope
        if let Some(ref meta) = envelope.meta.tenant {
            message.metadata.get_or_insert_with(|| serde_json::Map::new())
                .insert("tenant".to_string(), serde_json::Value::String(meta.clone()));
        }

        if let Some(ref request_id) = envelope.meta.request_id {
            message.metadata.get_or_insert_with(|| serde_json::Map::new())
                .insert("request_id".to_string(), serde_json::Value::String(request_id.to_string()));
        }

        Ok(message)
    }

    /// Convert a2a-rs task to qollective envelope
    #[cfg(feature = "a2a-standard")]
    fn a2a_task_to_envelope<R: for<'de> serde::Deserialize<'de>>(&self, task: Task, meta: Meta) -> Result<Envelope<R>> {
        // Extract the relevant data from the task
        let task_data = serde_json::to_value(&task).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize task: {}", e))
        })?;

        // Deserialize to the target type
        let response_data: R = serde_json::from_value(task_data).map_err(|e| {
            QollectiveError::deserialization(format!("Failed to deserialize task to target type: {}", e))
        })?;

        // Create envelope with metadata
        Ok(Envelope::new(meta, response_data))
    }

    /// Send envelope using a2a-rs standard protocol
    #[cfg(feature = "a2a-standard")]
    async fn send_envelope_a2a<T, R>(&self, endpoint: &str, envelope: Envelope<T>) -> Result<Envelope<R>>
    where
        T: Serialize + Send + 'static,
        R: for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        // Check if a2a client is available
        let a2a_client = self.a2a_client.as_ref().ok_or_else(|| {
            QollectiveError::transport("A2A standard client not configured".to_string())
        })?;

        // Convert envelope to a2a-rs message
        let message = self.envelope_to_a2a_message(&envelope)?;

        // Extract task ID from endpoint or generate one
        let task_id = endpoint.strip_prefix("a2a://").unwrap_or(endpoint);

        // Send message using a2a-rs client
        let task = a2a_client.send_task_message(task_id, &message, None, None).await
            .map_err(|e| QollectiveError::transport(format!("A2A send failed: {}", e)))?;

        // Convert response back to envelope
        self.a2a_task_to_envelope(task, envelope.meta.clone())
    }

    /// Get agent information by ID
    pub async fn get_agent_info(&self, agent_id: &str) -> Option<AgentInfo> {
        let registry = self.agent_registry.read().await;
        registry.get(agent_id).cloned()
    }

    /// List all registered agents
    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let registry = self.agent_registry.read().await;
        registry.values().cloned().collect()
    }

    /// Route message to agents with specified capability (simplified for a2a-rs)
    pub async fn route_to_capability(
        &self,
        capability: &str,
        envelope: Envelope<serde_json::Value>,
    ) -> Result<Envelope<serde_json::Value>> {
        // For a2a-rs, we route to the configured endpoint
        // The capability is used as part of the task identification
        let endpoint = format!("capability:{}", capability);
        self.send_envelope(&endpoint, envelope).await
    }
}

impl std::fmt::Debug for InternalA2AClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalA2AClient")
            .field("config", &self.config)
            .field("local_agent_id", &self.local_agent_id)
            .field("is_connected", &self.is_connected)
            .field("agent_registry", &"<RwLock<HashMap<String, AgentInfo>>>")
            .finish()
    }
}

#[async_trait]
impl<T, R> UnifiedEnvelopeSender<T, R> for InternalA2AClient
where
    T: Serialize + Send + Sync + 'static,
    R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    async fn send_envelope(&self, endpoint: &str, envelope: Envelope<T>) -> Result<Envelope<R>> {
        if !self.is_connected {
            return Err(QollectiveError::transport(
                "A2A transport not connected".to_string(),
            ));
        }

        // Use a2a-rs standard client if available
        #[cfg(feature = "a2a-standard")]
        {
            if self.a2a_client.is_some() {
                return self.send_envelope_a2a(endpoint, envelope).await;
            }
        }

        // For NATS-only A2A, we don't support direct envelope sending via HTTP
        // The Enterprise agents use NATS subjects directly for communication
        #[cfg(not(feature = "a2a-standard"))]
        {
            return Err(QollectiveError::transport(
                "A2A envelope sending requires NATS subject-based communication for NATS-only transport".to_string(),
            ));
        }

        // If no a2a client configured and standard feature is enabled, return error
        #[cfg(feature = "a2a-standard")]
        Err(QollectiveError::transport(
            "A2A standard client not configured".to_string(),
        ))
    }
}

#[async_trait]
impl<T, R> UnifiedSender<T, R> for InternalA2AClient
where
    T: Serialize + Send + Sync + 'static,
    R: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    async fn send(&self, endpoint: &str, data: T) -> Result<R> {
        let envelope = Envelope::new(Meta::default(), data);
        let response_envelope = self.send_envelope(endpoint, envelope).await?;
        let (_, response_data) = response_envelope.extract();
        Ok(response_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Meta;
    use serde_json::json;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        result: String,
    }

    #[test]
    fn test_a2a_client_creation() {
        let client = InternalA2AClient::mock();
        assert_eq!(client.local_agent_id, "mock-agent");
        assert!(client.is_connected);
    }

    #[test]
    fn test_a2a_client_disconnected() {
        let client = InternalA2AClient::mock_disconnected();
        assert_eq!(client.local_agent_id, "mock-agent");
        assert!(!client.is_connected);
    }

    #[tokio::test]
    async fn test_agent_discovery() {
        let client = InternalA2AClient::mock();
        let agents = client.discover_agents(&["test_capability".to_string()]).await;
        assert!(agents.is_ok());
        assert_eq!(agents.unwrap().len(), 0); // No agents in mock registry
    }

    #[tokio::test]
    async fn test_agent_discovery_disconnected() {
        let client = InternalA2AClient::mock_disconnected();
        let result = client.discover_agents(&["test_capability".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_envelope_disconnected() {
        let client = InternalA2AClient::mock_disconnected();
        let envelope = Envelope::new(Meta::default(), TestRequest {
            message: "test".to_string(),
        });

        let result: Result<Envelope<TestResponse>> = client.send_envelope("test-endpoint", envelope).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_envelope_no_client() {
        let client = InternalA2AClient::mock(); // Connected but no a2a client
        let envelope = Envelope::new(Meta::default(), TestRequest {
            message: "test".to_string(),
        });

        let result: Result<Envelope<TestResponse>> = client.send_envelope("test-endpoint", envelope).await;
        assert!(result.is_err());
    }
}
