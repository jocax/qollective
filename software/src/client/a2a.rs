// ABOUTME: A2A (Agent-to-Agent) client implementation with envelope-first design
// ABOUTME: Provides clean, consistent agent-to-agent communication following standard envelope pattern

//! A2A (Agent-to-Agent) client implementation for the Qollective framework.
//!
//! This module provides a clean, envelope-first client for agent-to-agent communication:
//! - Standard `send_envelope()` method matching other clients
//! - Agent discovery and capability queries
//! - Broadcasting to capabilities
//! - A2A metadata integration via envelope extensions

use crate::{
    client::nats::NatsClient,
    config::a2a::A2AClientConfig,
    constants::subjects,
    envelope::{Envelope, Meta},
    error::Result,
    traits::senders::UnifiedEnvelopeSender,
    transport::{HybridTransportClient, TransportDetectionConfig},
    types::a2a::{AgentInfo, CapabilityQuery, RegistryEvent},
};

use chrono;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use uuid::Uuid;

// ============================================================================
// A2A METADATA AND PROTOCOL TYPES
// ============================================================================

/// A2A metadata that gets added to envelope extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMetadata {
    pub protocol_version: String,
    pub message_type: A2AMessageType,
    pub source_agent: String,
    pub target_agent: Option<String>,
    pub capability: Option<String>,
    pub timestamp: SystemTime,
}

/// Types of A2A messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum A2AMessageType {
    DirectMessage,
    CapabilityBroadcast,
    AgentDiscovery,
    AgentAnnounce,
    HealthCheck,
}

/// Official A2A Protocol Message Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub role: String,
    pub parts: Vec<A2APart>,
    pub message_id: String,
    pub task_id: Option<String>,
    pub context_id: Option<String>,
    pub kind: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A2A Protocol Part Union Type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum A2APart {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "data")]
    Data {
        data: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
}

/// Agent provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProviderInfo {
    pub name: String,
    pub url: Option<String>,
    pub contact: Option<String>,
}

/// AgentCard - A2A standard capability description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<AgentCapability>,
    pub endpoint_url: String,
    pub authentication: AuthenticationRequirement,
    pub protocol_version: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub provider: Option<AgentProviderInfo>,
}

/// Individual capability within an AgentCard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Authentication requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationRequirement {
    pub method: AuthMethod,
    pub parameters: HashMap<String, String>,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    None,
    ApiKey,
    OAuth2,
    JWT,
    Custom(String),
}

/// Agent metadata for registration and health tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub version: String,
    pub build_info: Option<String>,
    pub capabilities_metadata: HashMap<String, serde_json::Value>,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

/// Performance metrics for agent monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_tasks: u32,
    pub max_tasks: u32,
    pub average_response_time_ms: f64,
}

/// Agent registration data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub agent_info: AgentInfo,
    pub metadata: AgentMetadata,
}

/// Health status update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusUpdate {
    pub agent_id: uuid::Uuid,
    pub status: crate::types::a2a::HealthStatus,
    pub metadata: AgentMetadata,
    pub timestamp: std::time::SystemTime,
}

/// Agent provider trait for extensibility (simplified)
pub trait AgentProvider: Send + Sync {
    fn get_agent_info(&self) -> AgentInfo;
    fn get_capabilities(&self) -> Vec<String>;
}

/// Simplified agent registry for managing agent discovery
#[derive(Debug)]
pub struct AgentRegistry {
    nats_client: NatsClient,
    registered_agents: HashMap<uuid::Uuid, (AgentInfo, AgentMetadata)>,
    #[allow(dead_code)] // Stored for debugging and future configuration access
    config: crate::config::a2a::RegistryConfig,
    // Metrics tracking
    creation_time: SystemTime,
    response_times: Vec<Duration>,
    last_cleanup: Option<SystemTime>,
}

impl AgentRegistry {
    /// Create a new agent registry with provided NATS configuration (CONFIG FIRST PRINCIPLE)
    pub async fn new(
        config: crate::config::a2a::RegistryConfig,
        nats_config: crate::config::nats::NatsConfig,
    ) -> crate::error::Result<Self> {
        // Use provided NATS config instead of creating defaults (following CONFIG FIRST PRINCIPLE)
        // Convert full config to client config for API consistency
        let nats_client = NatsClient::new(nats_config).await?;

        Ok(Self {
            nats_client,
            registered_agents: HashMap::new(),
            config,
            // Initialize metrics tracking
            creation_time: SystemTime::now(),
            response_times: Vec::new(),
            last_cleanup: None,
        })
    }

    /// Register an agent
    pub async fn register_agent(
        &mut self,
        agent_info: AgentInfo,
        metadata: AgentMetadata,
    ) -> crate::error::Result<()> {
        // Store locally
        self.registered_agents
            .insert(agent_info.id, (agent_info.clone(), metadata.clone()));

        // Create envelope for registration announcement
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let registration = AgentRegistration {
            agent_info,
            metadata,
        };

        let envelope = Envelope {
            meta,
            payload: registration,
            error: None,
        };

        // Announce via NATS using proper subject constant
        self.nats_client
            .publish(subjects::AGENT_REGISTRATION, envelope)
            .await
    }

    /// Deregister an agent
    pub async fn deregister_agent(&mut self, agent_id: uuid::Uuid) -> crate::error::Result<()> {
        // Remove locally
        self.registered_agents.remove(&agent_id);

        // Create envelope for deregistration
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let deregistration = crate::types::a2a::DeregistrationRequest {
            agent_id,
            reason: Some("manual_deregistration".to_string()),
        };

        let envelope = Envelope {
            meta,
            payload: deregistration,
            error: None,
        };

        // Announce via NATS using proper subject constant
        self.nats_client
            .publish(subjects::AGENT_DEREGISTRATION, envelope)
            .await
    }

    /// Get all registered agents
    pub fn get_agents(&self) -> Vec<&AgentInfo> {
        self.registered_agents
            .values()
            .map(|(info, _)| info)
            .collect()
    }

    /// Get agent by ID
    pub fn get_agent_by_id(&self, agent_id: &uuid::Uuid) -> Option<&AgentInfo> {
        self.registered_agents.get(agent_id).map(|(info, _)| info)
    }

    /// Find agents by capability
    pub fn find_agents_by_capability(&self, capability: &str) -> Vec<&AgentInfo> {
        self.registered_agents
            .values()
            .filter(|(info, _)| info.capabilities.contains(&capability.to_string()))
            .map(|(info, _)| info)
            .collect()
    }

    /// Find agents based on capability query
    pub async fn find_agents(
        &self,
        query: &crate::types::a2a::CapabilityQuery,
    ) -> crate::error::Result<Vec<AgentInfo>> {
        let mut matching_agents = Vec::new();

        for (agent_info, _) in self.registered_agents.values() {
            // Check if agent has all required capabilities
            let has_required = query
                .required_capabilities
                .iter()
                .all(|cap| agent_info.capabilities.contains(cap));

            // Skip if doesn't have required capabilities
            if !has_required {
                continue;
            }

            // Skip if agent is in exclude list
            if query.exclude_agents.contains(&agent_info.id) {
                continue;
            }

            matching_agents.push(agent_info.clone());
        }

        // Limit results if max_results is specified
        if let Some(max) = query.max_results {
            matching_agents.truncate(max);
        }

        Ok(matching_agents)
    }

    /// Record a response time for metrics tracking
    pub fn record_response_time(&mut self, response_time: Duration) {
        self.response_times.push(response_time);

        // Keep only the last 1000 response times to prevent unbounded growth
        if self.response_times.len() > 1000 {
            self.response_times.drain(0..500); // Remove oldest 500 entries
        }
    }

    /// Mark that a cleanup operation has been performed
    pub fn mark_cleanup_performed(&mut self) {
        self.last_cleanup = Some(SystemTime::now());
    }

    /// Calculate average response time in milliseconds
    fn calculate_avg_response_time_ms(&self) -> f64 {
        if self.response_times.is_empty() {
            return 0.0;
        }

        let total_ms: u128 = self
            .response_times
            .iter()
            .map(|duration| duration.as_millis())
            .sum();

        total_ms as f64 / self.response_times.len() as f64
    }

    /// Calculate uptime since registry creation
    fn calculate_uptime(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.creation_time)
            .unwrap_or(Duration::from_secs(0))
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> crate::error::Result<crate::server::a2a::RegistryStats> {
        let total_agents = self.registered_agents.len();
        let healthy_agents = self
            .registered_agents
            .values()
            .filter(|(info, _)| {
                matches!(info.health_status, crate::types::a2a::HealthStatus::Healthy)
            })
            .count();

        let mut unique_capabilities = std::collections::HashSet::new();
        for (agent_info, _) in self.registered_agents.values() {
            for cap in &agent_info.capabilities {
                unique_capabilities.insert(cap);
            }
        }

        Ok(crate::server::a2a::RegistryStats {
            total_agents,
            healthy_agents,
            unique_capabilities: unique_capabilities.len(),
            avg_response_time_ms: self.calculate_avg_response_time_ms(),
            uptime: self.calculate_uptime(),
            last_cleanup: self.last_cleanup,
        })
    }

    /// Clean up stale agents based on last heartbeat time
    pub async fn cleanup_stale_agents(&mut self) -> crate::error::Result<usize> {
        use crate::constants::timeouts;

        let now = SystemTime::now();
        let stale_threshold = timeouts::DEFAULT_AGENT_TIMEOUT;
        let mut stale_agents = Vec::new();

        // Find agents that haven't sent a heartbeat within the threshold
        for (agent_id, (agent_info, _metadata)) in &self.registered_agents {
            if let Ok(duration_since_heartbeat) = now.duration_since(agent_info.last_heartbeat) {
                if duration_since_heartbeat > stale_threshold {
                    stale_agents.push(*agent_id);
                }
            }
        }

        // Remove stale agents
        let removed_count = stale_agents.len();
        for agent_id in stale_agents {
            if let Some((agent_info, _)) = self.registered_agents.remove(&agent_id) {
                tracing::info!(
                    "Cleaned up stale agent: {} (ID: {})",
                    agent_info.name,
                    agent_id
                );
            }
        }

        // Mark cleanup as performed
        self.mark_cleanup_performed();

        Ok(removed_count)
    }
}

// ============================================================================
// CLEAN A2A CLIENT IMPLEMENTATION
// ============================================================================

/// Clean, envelope-first A2A client following standard pattern
#[derive(Debug)]
pub struct A2AClient {
    transport: std::sync::Arc<crate::transport::HybridTransportClient>,
    agent_id: String,
    #[allow(dead_code)] // Stored for debugging and future configuration access
    config: A2AClientConfig,
}

impl A2AClient {
    /// Create a new A2A client with its own transport layer (NEW API - preferred)
    pub async fn new(config: A2AClientConfig) -> Result<Self> {
        // Create transport configuration from A2A client config (CONFIG FIRST PRINCIPLE)
        let transport_config = TransportDetectionConfig {
            enable_auto_detection: true,
            detection_timeout: config.transport.default_timeout,
            capability_cache_ttl: config.transport.discovery_config.cache_ttl,
            retry_failed_detections: config.transport.max_retries > 0,
            max_detection_retries: config.transport.max_retries,
        };

        // Create transport with A2A transport injected
        let mut transport = HybridTransportClient::new(transport_config);

        // Create the actual A2A transport that the transport will use
        let a2a_transport = crate::transport::a2a::InternalA2AClient::new(config.clone()).await?;
        transport = transport.with_a2a_transport(Arc::new(a2a_transport));

        Ok(Self {
            transport: Arc::new(transport),
            agent_id: config.client.agent_id.clone(),
            config,
        })
    }

    /// Create A2A client with transport dependency injection (for testing)
    pub async fn with_transport(
        config: A2AClientConfig,
        transport: std::sync::Arc<HybridTransportClient>,
    ) -> Result<Self> {
        Ok(Self {
            transport,
            agent_id: config.client.agent_id.clone(),
            config,
        })
    }

    /// Get reference to transport for testing
    pub fn transport(&self) -> Option<&std::sync::Arc<HybridTransportClient>> {
        Some(&self.transport)
    }

    /// Get access to internal NATS client for subscriptions
    pub fn get_nats_client(&self) -> Result<&async_nats::Client> {
        if let Some(nats_client) = self.transport.internal_nats_client() {
            // Access the underlying async_nats::Client from InternalNatsClient
            Ok(nats_client.client())
        } else {
            Err(crate::error::QollectiveError::transport(
                "NATS client not available in transport layer - ensure NATS transport is configured".to_string()
            ))
        }
    }

    /// Primary envelope method - send message to specific agent
    pub async fn send_envelope<T, R>(
        &self,
        target_agent: &str,
        envelope: Envelope<T>,
    ) -> Result<Envelope<R>>
    where
        T: Serialize + Send + Sync + 'static,
        R: for<'de> Deserialize<'de> + Send + Sync + 'static,
    {
        // Add A2A metadata to envelope extensions
        let enhanced_envelope = self.add_a2a_metadata(
            envelope,
            Some(target_agent.to_string()),
            A2AMessageType::DirectMessage,
        )?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // Send directly to target agent using agent ID
            let result: Result<Envelope<R>> = a2a_transport
                .send_envelope(target_agent, json_envelope)
                .await;
            result
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Broadcast envelope to capability
    pub async fn broadcast_envelope<T>(&self, capability: &str, envelope: Envelope<T>) -> Result<()>
    where
        T: Serialize + Send + Sync + 'static,
    {
        // Add A2A metadata for capability broadcast
        let enhanced_envelope =
            self.add_a2a_metadata(envelope, None, A2AMessageType::CapabilityBroadcast)?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // Use route_to_capability for broadcasting to all agents with the capability
            match a2a_transport
                .route_to_capability(capability, json_envelope)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Discover agents with capability query
    pub async fn discover_agents(
        &self,
        query: CapabilityQuery,
    ) -> Result<Envelope<Vec<AgentInfo>>> {
        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert CapabilityQuery to capability list for A2A transport
            let agents = a2a_transport
                .discover_agents(&query.required_capabilities)
                .await?;

            // Create envelope response
            let mut meta = Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("agents".to_string());

            Ok(Envelope {
                meta,
                payload: agents,
                error: None,
            })
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Discover agent cards with capability query (rich metadata)
    pub async fn discover_agent_cards(&self, query: CapabilityQuery) -> Result<Vec<AgentCard>> {
        // Get agent info via normal discovery
        let agents_envelope = self.discover_agents(query).await?;
        let agents = agents_envelope.payload;

        // Convert AgentInfo to AgentCard with rich metadata
        let agent_cards = agents
            .into_iter()
            .map(|agent_info| self.agent_info_to_agent_card(&agent_info))
            .collect();

        Ok(agent_cards)
    }

    /// Announce agent presence
    pub async fn announce_agent(&self, agent_info: AgentInfo) -> Result<()> {
        // Create envelope with agent info
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let envelope = Envelope {
            meta,
            payload: agent_info,
            error: None,
        };

        // Add A2A metadata for announcement
        let enhanced_envelope =
            self.add_a2a_metadata(envelope, None, A2AMessageType::AgentAnnounce)?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // For announcements, we can use the local agent ID or a broadcast endpoint
            let endpoint = subjects::AGENT_REGISTRY_ANNOUNCE;
            let result: Result<Envelope<serde_json::Value>> =
                a2a_transport.send_envelope(endpoint, json_envelope).await;
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Subscribe to messages for this agent - note: requires direct NATS client access
    /// For now, use the NATS client directly in examples for subscriptions
    pub async fn subscribe_to_messages(&self) -> Result<String> {
        let subject = format!("agent.{}.direct", self.agent_id);
        Ok(subject) // Return subject for manual subscription
    }

    /// Subscribe to capability broadcasts (modern method name)
    /// Returns subject name for manual subscription with NATS client
    pub async fn subscribe_to_capability_broadcasts(&self, capability: String) -> Result<String> {
        let subject = format!("capability.{}.broadcast", capability);
        Ok(subject) // Return subject for manual subscription
    }

    /// Create A2A text message
    pub fn create_a2a_text_message(&self, text: String, task_id: Option<String>) -> A2AMessage {
        A2AMessage {
            role: "user".to_string(),
            parts: vec![A2APart::Text {
                text,
                metadata: None,
            }],
            message_id: Uuid::now_v7().to_string(),
            task_id,
            context_id: Some(Uuid::now_v7().to_string()),
            kind: "message".to_string(),
            metadata: HashMap::from([
                (
                    "source_agent".to_string(),
                    serde_json::Value::String(self.agent_id.clone()),
                ),
                (
                    "protocol".to_string(),
                    serde_json::Value::String("a2a-in-qollective".to_string()),
                ),
            ]),
        }
    }

    /// Create A2A data message with structured data payload
    pub fn create_a2a_data_message<T: Serialize>(
        &self,
        data: T,
        task_id: Option<String>,
    ) -> Result<A2AMessage> {
        let data_value = serde_json::to_value(data)?;
        Ok(A2AMessage {
            role: "user".to_string(),
            parts: vec![A2APart::Data {
                data: data_value,
                metadata: None,
            }],
            message_id: Uuid::now_v7().to_string(),
            task_id,
            context_id: Some(Uuid::now_v7().to_string()),
            kind: "message".to_string(),
            metadata: HashMap::from([
                (
                    "source_agent".to_string(),
                    serde_json::Value::String(self.agent_id.clone()),
                ),
                (
                    "protocol".to_string(),
                    serde_json::Value::String("a2a-in-qollective".to_string()),
                ),
                (
                    "message_type".to_string(),
                    serde_json::Value::String("data".to_string()),
                ),
            ]),
        })
    }

    /// Register agent with discovery system
    pub async fn register_agent(
        &self,
        agent_info: AgentInfo,
        metadata: AgentMetadata,
    ) -> Result<()> {
        // Create envelope with agent registration
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let registration_data = AgentRegistration {
            agent_info,
            metadata,
        };

        let envelope = Envelope {
            meta,
            payload: registration_data,
            error: None,
        };

        // Add A2A metadata for agent registration
        let enhanced_envelope =
            self.add_a2a_metadata(envelope, None, A2AMessageType::AgentAnnounce)?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // Send registration announcement - we ignore the response since this is publish-like
            let result: Result<Envelope<serde_json::Value>> = a2a_transport
                .send_envelope(subjects::AGENT_REGISTRATION, json_envelope)
                .await;
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Deregister agent from discovery system
    pub async fn deregister_agent(&self, agent_id: uuid::Uuid) -> Result<()> {
        // Create envelope with deregistration request
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let deregistration_request = crate::types::a2a::DeregistrationRequest {
            agent_id,
            reason: Some("manual_deregistration".to_string()),
        };

        let envelope = Envelope {
            meta,
            payload: deregistration_request,
            error: None,
        };

        // Add A2A metadata for deregistration
        let enhanced_envelope =
            self.add_a2a_metadata(envelope, None, A2AMessageType::AgentAnnounce)?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // Send deregistration request - ignore response since this is publish-like
            let result: Result<Envelope<serde_json::Value>> = a2a_transport
                .send_envelope("discovery.agent.deregister", json_envelope)
                .await;
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Publish health status update
    pub async fn publish_health_status(
        &self,
        agent_id: uuid::Uuid,
        status: crate::types::a2a::HealthStatus,
        metadata: Option<AgentMetadata>,
    ) -> Result<()> {
        // Create envelope with health status
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let health_update = HealthStatusUpdate {
            agent_id,
            status,
            metadata: metadata.unwrap_or_default(),
            timestamp: std::time::SystemTime::now(),
        };

        let envelope = Envelope {
            meta,
            payload: health_update,
            error: None,
        };

        // Add A2A metadata for health update
        let enhanced_envelope =
            self.add_a2a_metadata(envelope, None, A2AMessageType::HealthCheck)?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // Send health status update - ignore response since this is publish-like
            let result: Result<Envelope<serde_json::Value>> = a2a_transport
                .send_envelope("agent.health.update", json_envelope)
                .await;
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Publish registry event
    pub async fn publish_registry_event(
        &self,
        event_type: String,
        agent_id: uuid::Uuid,
        agent_name: String,
        capabilities: Vec<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        // Create envelope with registry event
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let registry_event = RegistryEvent {
            event_type,
            agent_id,
            agent_name,
            capabilities,
            timestamp: std::time::SystemTime::now(),
            metadata,
        };

        let envelope = Envelope {
            meta,
            payload: registry_event,
            error: None,
        };

        // Add A2A metadata for registry event
        let enhanced_envelope =
            self.add_a2a_metadata(envelope, None, A2AMessageType::AgentAnnounce)?;

        // Use A2A transport delegation pattern
        if let Some(a2a_transport) = self.transport.internal_a2a_client() {
            // Convert to JSON envelope for internal routing
            let (meta, data) = enhanced_envelope.extract();
            let json_data = serde_json::to_value(data).map_err(|e| {
                crate::error::QollectiveError::transport(format!("Serialization failed: {}", e))
            })?;
            let json_envelope = Envelope::new(meta, json_data);

            // Send registry event - ignore response since this is publish-like
            let result: Result<Envelope<serde_json::Value>> = a2a_transport
                .send_envelope("registry.event.publish", json_envelope)
                .await;
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(crate::error::QollectiveError::transport(
                "A2A transport not available in transport layer".to_string(),
            ))
        }
    }

    /// Publish registry event directly to NATS subject (bypassing request/reply)
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub async fn publish_registry_event_to_subject(
        &self,
        event_type: String,
        agent_id: uuid::Uuid,
        agent_name: String,
        capabilities: Vec<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        use crate::constants::subjects;

        // Create registry event
        let registry_event = RegistryEvent {
            event_type,
            agent_id,
            agent_name,
            capabilities,
            timestamp: std::time::SystemTime::now(),
            metadata,
        };

        // Create envelope with registry event
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());
        meta.tenant = Some("agents".to_string());

        let envelope = Envelope {
            meta,
            payload: registry_event,
            error: None,
        };

        // Get internal NATS client from transport and publish directly to subject
        if let Some(internal_nats_client) = self.transport.internal_nats_client() {
            let async_nats_client = internal_nats_client.client();

            // Serialize the envelope to bytes
            let payload = serde_json::to_vec(&envelope).map_err(|e| {
                crate::error::QollectiveError::transport(format!(
                    "Failed to serialize registry event: {}",
                    e
                ))
            })?;

            // Publish directly to the NATS subject
            async_nats_client
                .publish(subjects::AGENT_REGISTRY_EVENTS, payload.into())
                .await
                .map_err(|e| {
                    crate::error::QollectiveError::transport(format!(
                        "Failed to publish registry event to NATS: {}",
                        e
                    ))
                })?;

            Ok(())
        } else {
            Err(crate::error::QollectiveError::transport(
                "NATS client not available in transport layer".to_string(),
            ))
        }
    }

    /// Subscribe to direct A2A messages (modern method name)
    /// Returns subject name for manual subscription with NATS client
    pub async fn subscribe_to_direct_a2a_messages(&self) -> Result<String> {
        let subject = format!("agent.{}.direct", self.agent_id);
        Ok(subject) // Return subject for manual subscription
    }

    /// Helper method to add A2A metadata to envelope extensions
    fn add_a2a_metadata<T>(
        &self,
        mut envelope: Envelope<T>,
        target_agent: Option<String>,
        message_type: A2AMessageType,
    ) -> Result<Envelope<T>> {
        let a2a_metadata = A2AMetadata {
            protocol_version: "1.0".to_string(),
            message_type,
            source_agent: self.agent_id.clone(),
            target_agent,
            capability: None,
            timestamp: SystemTime::now(),
        };

        // Add A2A metadata to envelope extensions
        if envelope.meta.extensions.is_none() {
            envelope.meta.extensions = Some(crate::envelope::meta::ExtensionsMeta {
                sections: HashMap::new(),
            });
        }

        if let Some(ref mut extensions) = envelope.meta.extensions {
            extensions.sections.insert(
                "a2a_metadata".to_string(),
                serde_json::to_value(a2a_metadata)?,
            );
        }

        Ok(envelope)
    }

    /// Convert AgentInfo to AgentCard with rich metadata
    fn agent_info_to_agent_card(&self, agent_info: &AgentInfo) -> AgentCard {
        // Convert capabilities to AgentCapability structs
        let capabilities = agent_info
            .capabilities
            .iter()
            .map(|cap| AgentCapability {
                name: cap.clone(),
                description: format!("Agent capability: {}", cap),
                input_types: vec!["application/json".to_string()],
                output_types: vec!["application/json".to_string()],
                parameters: HashMap::new(),
            })
            .collect();

        // Convert metadata to JSON values
        let mut card_metadata = HashMap::new();
        for (key, value) in &agent_info.metadata {
            card_metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        card_metadata.insert(
            "agent_id".to_string(),
            serde_json::Value::String(agent_info.id.to_string()),
        );
        card_metadata.insert(
            "last_heartbeat".to_string(),
            serde_json::Value::String(format!("{:?}", agent_info.last_heartbeat)),
        );

        AgentCard {
            name: agent_info.name.clone(),
            description: format!("Qollective agent: {}", agent_info.name),
            version: "1.0.0".to_string(),
            capabilities,
            endpoint_url: format!("qollective://agent/{}", agent_info.id),
            authentication: AuthenticationRequirement {
                method: AuthMethod::None,
                parameters: HashMap::new(),
            },
            protocol_version: "1.0".to_string(),
            metadata: card_metadata,
            tags: vec!["qollective".to_string(), "a2a".to_string()],
            provider: Some(AgentProviderInfo {
                name: "Qollective Framework".to_string(),
                url: Some("https://qollective.io".to_string()),
                contact: None,
            }),
        }
    }

    /// Create A2A response message
    pub fn create_a2a_response_message(
        &self,
        text: String,
        task_id: Option<String>,
        in_reply_to: Option<String>,
    ) -> Result<A2AMessage> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "source_agent".to_string(),
            serde_json::Value::String(self.agent_id.clone()),
        );
        if let Some(reply_to) = in_reply_to {
            metadata.insert(
                "in_reply_to".to_string(),
                serde_json::Value::String(reply_to),
            );
        }

        Ok(A2AMessage {
            role: "assistant".to_string(),
            parts: vec![A2APart::Text {
                text,
                metadata: None,
            }],
            message_id: Uuid::now_v7().to_string(),
            task_id,
            context_id: None,
            kind: "response".to_string(),
            metadata,
        })
    }
}

// Default implementations
impl Default for AuthenticationRequirement {
    fn default() -> Self {
        Self {
            method: AuthMethod::None,
            parameters: HashMap::new(),
        }
    }
}

impl Default for AgentMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            build_info: None,
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::new(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_tasks: 0,
            max_tasks: 10,
            average_response_time_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::a2a::HealthStatus;
    use uuid::Uuid;

    fn create_test_agent_info() -> AgentInfo {
        AgentInfo {
            id: Uuid::now_v7(),
            name: "test-agent".to_string(),
            capabilities: vec!["test-capability".to_string()],
            health_status: HealthStatus::Healthy,
            last_heartbeat: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_envelope<T>(data: T) -> Envelope<T> {
        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.timestamp = Some(chrono::Utc::now());

        Envelope {
            meta,
            payload: data,
            error: None,
        }
    }

    #[test]
    fn test_a2a_metadata_enhancement() {
        let config = A2AClientConfig::default();
        let agent_id = config.client.agent_id.clone();

        // Create mock client (can't create real one without NATS in test)
        let envelope = create_test_envelope("test message".to_string());

        // Test metadata structure
        let a2a_metadata = A2AMetadata {
            protocol_version: "1.0".to_string(),
            message_type: A2AMessageType::DirectMessage,
            source_agent: agent_id,
            target_agent: Some("target-agent".to_string()),
            capability: None,
            timestamp: SystemTime::now(),
        };

        // Verify serialization
        let serialized = serde_json::to_value(&a2a_metadata).unwrap();
        assert!(serialized.get("protocol_version").is_some());
        assert!(serialized.get("message_type").is_some());
        assert!(serialized.get("source_agent").is_some());
    }

    #[test]
    fn test_a2a_message_creation() {
        let config = A2AClientConfig::default();

        // Create mock client structure for testing message creation
        let agent_id = config.client.agent_id.clone();

        // Test A2A text message creation
        let text_message = A2AMessage {
            role: "user".to_string(),
            parts: vec![A2APart::Text {
                text: "Hello World".to_string(),
                metadata: None,
            }],
            message_id: Uuid::now_v7().to_string(),
            task_id: None,
            context_id: Some(Uuid::now_v7().to_string()),
            kind: "message".to_string(),
            metadata: HashMap::from([
                (
                    "source_agent".to_string(),
                    serde_json::Value::String(agent_id.clone()),
                ),
                (
                    "protocol".to_string(),
                    serde_json::Value::String("a2a-in-qollective".to_string()),
                ),
            ]),
        };

        assert_eq!(text_message.role, "user");
        assert_eq!(text_message.parts.len(), 1);
        assert_eq!(text_message.kind, "message");

        // Test data message
        let data_value = serde_json::json!({"key": "value"});
        let data_message = A2AMessage {
            role: "user".to_string(),
            parts: vec![A2APart::Data {
                data: data_value.clone(),
                metadata: None,
            }],
            message_id: Uuid::now_v7().to_string(),
            task_id: None,
            context_id: Some(Uuid::now_v7().to_string()),
            kind: "message".to_string(),
            metadata: HashMap::from([
                (
                    "source_agent".to_string(),
                    serde_json::Value::String(agent_id),
                ),
                (
                    "protocol".to_string(),
                    serde_json::Value::String("a2a-in-qollective".to_string()),
                ),
                (
                    "message_type".to_string(),
                    serde_json::Value::String("data".to_string()),
                ),
            ]),
        };

        if let A2APart::Data { data, .. } = &data_message.parts[0] {
            assert_eq!(data, &data_value);
        } else {
            panic!("Expected data part");
        }
    }

    #[test]
    fn test_capability_query_structure() {
        let query = CapabilityQuery {
            required_capabilities: vec!["capability1".to_string(), "capability2".to_string()],
            preferred_capabilities: vec!["capability3".to_string()],
            exclude_agents: vec![Uuid::now_v7()],
            max_results: Some(10),
        };

        assert_eq!(query.required_capabilities.len(), 2);
        assert_eq!(query.preferred_capabilities.len(), 1);
        assert_eq!(query.exclude_agents.len(), 1);
        assert_eq!(query.max_results, Some(10));
    }

    #[test]
    fn test_agent_card_structure() {
        let capability = AgentCapability {
            name: "test-capability".to_string(),
            description: "Test capability".to_string(),
            input_types: vec!["application/json".to_string()],
            output_types: vec!["application/json".to_string()],
            parameters: HashMap::new(),
        };

        let agent_card = AgentCard {
            name: "test-agent".to_string(),
            description: "Test agent".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![capability],
            endpoint_url: "nats://agent.test".to_string(),
            authentication: AuthenticationRequirement::default(),
            protocol_version: "1.0".to_string(),
            metadata: HashMap::new(),
            tags: vec!["test".to_string()],
            provider: None,
        };

        assert_eq!(agent_card.name, "test-agent");
        assert_eq!(agent_card.capabilities.len(), 1);
        assert_eq!(agent_card.capabilities[0].name, "test-capability");
        assert!(matches!(agent_card.authentication.method, AuthMethod::None));
    }

    #[test]
    fn test_envelope_creation() {
        let agent_info = create_test_agent_info();
        let envelope = create_test_envelope(agent_info.clone());

        assert_eq!(envelope.payload.name, agent_info.name);
        assert!(envelope.meta.request_id.is_some());
        assert!(envelope.meta.timestamp.is_some());
        assert!(envelope.error.is_none());
    }
}
