// ABOUTME: A2A (Agent-to-Agent) protocol types and data structures
// ABOUTME: Contains agent identification, health monitoring, and capability definitions

//! A2A (Agent-to-Agent) protocol types and data structures.
//!
//! This module defines the fundamental types used throughout the A2A system:
//! - Agent identification and metadata
//! - Health status monitoring  
//! - Capability definitions and queries
//! - Communication envelope structures

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};
use uuid::Uuid;

/// Agent identifier type
pub type AgentId = Uuid;

/// Agent information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub name: String,
    pub capabilities: Vec<String>,
    pub health_status: HealthStatus,
    pub last_heartbeat: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Agent health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Unhealthy,
    Unknown,
}

/// Capability query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityQuery {
    pub required_capabilities: Vec<String>,
    pub preferred_capabilities: Vec<String>,
    pub exclude_agents: Vec<AgentId>,
    pub max_results: Option<usize>,
}

/// Agent heartbeat structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: AgentId,
    pub health_status: HealthStatus,
    pub timestamp: SystemTime,
    pub metadata: Option<HashMap<String, String>>,
}

/// Registry event structure for agent registration/deregistration events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    pub event_type: String,
    pub agent_id: AgentId,
    pub agent_name: String,
    pub capabilities: Vec<String>,
    pub timestamp: SystemTime,
    pub metadata: Option<HashMap<String, String>>,
}

/// Agent deregistration request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeregistrationRequest {
    pub agent_id: AgentId,
    pub reason: Option<String>,
}

/// Queue group configuration for capability-based load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueGroupConfig {
    pub queue_name: String,
    pub capability: String,
    pub version: String,
    pub auto_scale: bool,
}

impl QueueGroupConfig {
    /// Create a new queue group config for a capability
    pub fn new(capability: &str, version: &str) -> Self {
        Self {
            queue_name: format!("qollective.capability.{}.{}", capability, version),
            capability: capability.to_string(),
            version: version.to_string(),
            auto_scale: true,
        }
    }

    /// Generate queue group name based on capability
    pub fn queue_name(&self) -> &str {
        &self.queue_name
    }

    /// Validate queue group configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.capability.trim().is_empty() {
            return Err("Capability name cannot be empty".to_string());
        }

        if self.version.trim().is_empty() {
            return Err("Version cannot be empty".to_string());
        }

        if self.queue_name.contains("..") || self.queue_name.ends_with('.') {
            return Err("Invalid queue name format".to_string());
        }

        if !self.queue_name.starts_with("qollective.") {
            return Err("Queue name must follow shared subject pattern".to_string());
        }

        Ok(())
    }
}

/// Routing preferences for capability queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingPreferences {
    pub preferred_agents: Vec<AgentId>,
    pub excluded_agents: Vec<AgentId>,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub max_response_time_ms: Option<u64>,
    pub require_healthy_agents: bool,
}

/// Load balancing strategies for agent selection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoadBalancingStrategy {
    #[default]
    RoundRobin,
    LeastConnections,
    Random,
    HealthBased,
}

/// Routing decision result from capability query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_agents: Vec<AgentInfo>,
    pub routing_reason: String,
    pub estimated_response_time_ms: Option<u64>,
    pub fallback_agents: Vec<AgentInfo>,
}
