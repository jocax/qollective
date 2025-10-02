// ABOUTME: A2A (Agent-to-Agent) server implementation following consistent module pattern
// ABOUTME: Provides server-side agent registry, discovery, and request handling functionality

//! A2A (Agent-to-Agent) server implementation for the Qollective framework.
//!
//! This module provides comprehensive server-side functionality for:
//! - Agent registration and deregistration handling
//! - Capability query processing and agent discovery
//! - Health monitoring and status management
//! - Request routing and response handling
//! - Integration with NATS server and transport layer

use crate::{
    client::a2a::{A2AClient, AgentCard, AgentMetadata, AgentRegistration, AgentRegistry},
    config::a2a::{A2AServerConfig, HealthConfig, RegistryConfig, RoutingConfig},
    constants::limits,
    envelope::Context,
    error::{QollectiveError, Result},
    server::nats::NatsServer,
    traits::handlers::{ContextDataHandler, DefaultEnvelopeHandler},
    types::a2a::{
        AgentId, AgentInfo, CapabilityQuery, DeregistrationRequest, HealthStatus, Heartbeat,
        RoutingDecision, RoutingPreferences,
    },
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;

// ============================================================================
// SERVER-SPECIFIC TYPES
// ============================================================================

// ============================================================================
// HANDLER REQUEST/RESPONSE TYPES
// ============================================================================

// Note: Using AgentRegistration from client::a2a instead of separate RegistrationRequest
// This ensures client and server use the same data structure

/// Registration response payload  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub message: String,
    pub agent_id: AgentId,
}

/// Capability query request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: CapabilityQuery,
    pub preferences: Option<RoutingPreferences>,
}

/// Health check request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub agent_id: AgentId,
}

/// Health check response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub agent_id: AgentId,
    pub status: HealthStatus,
    pub timestamp: SystemTime,
}

/// Registry statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub unique_capabilities: usize,
    pub avg_response_time_ms: f64,
    pub uptime: Duration,
    pub last_cleanup: Option<SystemTime>,
}

/// Capability router for handling routing decisions
#[derive(Debug)]
pub struct CapabilityRouter {
    config: RoutingConfig,
    routing_cache: HashMap<String, Vec<AgentId>>,
}

impl CapabilityRouter {
    pub fn new(config: RoutingConfig) -> Self {
        Self {
            config,
            routing_cache: HashMap::new(),
        }
    }

    pub async fn route_capability(
        &self,
        query: &CapabilityQuery,
        mut agents: Vec<AgentInfo>,
        preferences: &RoutingPreferences,
    ) -> Result<RoutingDecision> {
        use crate::constants::limits;
        use crate::types::a2a::LoadBalancingStrategy;

        // Filter out unhealthy agents if required
        if preferences.require_healthy_agents {
            agents.retain(|agent| agent.health_status == HealthStatus::Healthy);
        }

        // Filter out excluded agents
        agents.retain(|agent| !preferences.excluded_agents.contains(&agent.id));

        // If preferred agents exist and are available, prioritize them
        let mut preferred_agents = Vec::new();
        let mut other_agents = Vec::new();

        for agent in agents {
            if preferences.preferred_agents.contains(&agent.id) {
                preferred_agents.push(agent);
            } else {
                other_agents.push(agent);
            }
        }

        // Combine preferred agents first, then others
        let mut available_agents = preferred_agents;
        available_agents.extend(other_agents);

        // Apply max results limit
        let max_results = query
            .max_results
            .unwrap_or(limits::DEFAULT_MAX_AGENTS)
            .min(available_agents.len());

        // Apply load balancing strategy
        let selected_agents: Vec<AgentInfo> = match preferences.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                // For round robin, take agents in order up to max_results
                available_agents.into_iter().take(max_results).collect()
            }
            LoadBalancingStrategy::Random => {
                // For random, shuffle and take max_results
                use rand::seq::SliceRandom;
                let mut rng = rand::rng();
                available_agents.shuffle(&mut rng);
                available_agents.into_iter().take(max_results).collect()
            }
            LoadBalancingStrategy::HealthBased => {
                // Sort by health status (Healthy first, then Warning, then others)
                available_agents.sort_by(|a, b| {
                    use HealthStatus::*;
                    let health_priority = |status: &HealthStatus| match status {
                        Healthy => 0,
                        Warning => 1,
                        Unhealthy => 2,
                        Unknown => 3,
                    };
                    health_priority(&a.health_status).cmp(&health_priority(&b.health_status))
                });
                available_agents.into_iter().take(max_results).collect()
            }
            LoadBalancingStrategy::LeastConnections => {
                // For now, treat as round robin since we don't track connections yet
                available_agents.into_iter().take(max_results).collect()
            }
        };

        // Generate routing reason
        let routing_reason = format!(
            "Selected {} agents using {:?} strategy, healthy_only: {}, preferred_count: {}",
            selected_agents.len(),
            preferences.load_balancing_strategy,
            preferences.require_healthy_agents,
            preferences.preferred_agents.len()
        );

        // Estimate response time based on agent health
        let estimated_response_time_ms = if selected_agents.is_empty() {
            None
        } else {
            let avg_health_score = selected_agents
                .iter()
                .map(|agent| match agent.health_status {
                    HealthStatus::Healthy => 50,
                    HealthStatus::Warning => 100,
                    HealthStatus::Unhealthy => 200,
                    HealthStatus::Unknown => 150,
                })
                .sum::<u64>()
                / selected_agents.len() as u64;

            Some(avg_health_score)
        };

        // Set fallback agents (remaining available agents not selected)
        let fallback_agents = Vec::new(); // For now, keep simple without complex fallback logic

        Ok(RoutingDecision {
            selected_agents,
            routing_reason,
            estimated_response_time_ms,
            fallback_agents,
        })
    }
}

/// Circuit breaker state for agent health monitoring
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    failure_count: u32,
    last_failure_time: Option<SystemTime>,
    state: CircuitState,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,   // Normal operation
    Open,     // Circuit is open, rejecting requests
    HalfOpen, // Testing if agent has recovered
}

/// Agent health monitor for tracking agent health
#[derive(Debug)]
pub struct AgentHealthMonitor {
    config: HealthConfig,
    health_cache: HashMap<AgentId, (HealthStatus, SystemTime)>,
    circuit_breakers: HashMap<AgentId, CircuitBreakerState>,
}

impl AgentHealthMonitor {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            health_cache: HashMap::new(),
            circuit_breakers: HashMap::new(),
        }
    }

    pub async fn is_circuit_open(&mut self, agent_id: &AgentId) -> bool {
        use crate::constants::timeouts;

        let circuit_breaker =
            self.circuit_breakers
                .entry(*agent_id)
                .or_insert_with(|| CircuitBreakerState {
                    failure_count: 0,
                    last_failure_time: None,
                    state: CircuitState::Closed,
                });

        match circuit_breaker.state {
            CircuitState::Closed => false,
            CircuitState::Open => {
                // Check if enough time has passed to try half-open
                if let Some(last_failure) = circuit_breaker.last_failure_time {
                    let recovery_timeout = timeouts::DEFAULT_CIRCUIT_BREAKER_RECOVERY;
                    if SystemTime::now()
                        .duration_since(last_failure)
                        .unwrap_or_default()
                        >= recovery_timeout
                    {
                        circuit_breaker.state = CircuitState::HalfOpen;
                        tracing::info!(
                            "Circuit breaker for agent {} moved to half-open state",
                            agent_id
                        );
                        false // Allow one test request
                    } else {
                        true // Still in open state
                    }
                } else {
                    true // Open but no timestamp, keep open
                }
            }
            CircuitState::HalfOpen => false, // Allow the test request
        }
    }

    pub async fn record_health_result(&mut self, agent_id: &AgentId, is_healthy: bool) {
        let status = if is_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        // Update health cache
        self.health_cache
            .insert(*agent_id, (status, SystemTime::now()));

        // Update circuit breaker state
        let circuit_breaker =
            self.circuit_breakers
                .entry(*agent_id)
                .or_insert_with(|| CircuitBreakerState {
                    failure_count: 0,
                    last_failure_time: None,
                    state: CircuitState::Closed,
                });

        if is_healthy {
            // Reset circuit breaker on successful health check
            circuit_breaker.failure_count = 0;
            circuit_breaker.last_failure_time = None;
            if circuit_breaker.state != CircuitState::Closed {
                circuit_breaker.state = CircuitState::Closed;
                tracing::info!(
                    "Circuit breaker for agent {} reset to closed state",
                    agent_id
                );
            }
        } else {
            // Increment failure count
            circuit_breaker.failure_count += 1;
            circuit_breaker.last_failure_time = Some(SystemTime::now());

            // Open circuit if failure threshold exceeded
            const FAILURE_THRESHOLD: u32 = 5; // Could be made configurable
            if circuit_breaker.failure_count >= FAILURE_THRESHOLD
                && circuit_breaker.state != CircuitState::Open
            {
                circuit_breaker.state = CircuitState::Open;
                tracing::warn!(
                    "Circuit breaker for agent {} opened after {} failures",
                    agent_id,
                    circuit_breaker.failure_count
                );
            }
        }
    }
}

// ============================================================================
// SERVER STATE
// ============================================================================

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    /// Server uptime
    pub uptime: Duration,
    /// Total requests handled
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average request processing time
    pub avg_request_time_ms: f64,
    /// Current load (0.0-1.0)
    pub current_load: f64,
    /// Registry statistics
    pub registry_stats: RegistryStats,
}

// ============================================================================
// REQUEST HANDLERS
// ============================================================================

/// Rate limiting tracker for agent registrations
#[derive(Debug)]
struct RegistrationRateLimit {
    /// Per-agent registration counts with timestamps
    agent_registrations: HashMap<AgentId, Vec<SystemTime>>,
    /// Global registration count with timestamps
    global_registrations: Vec<SystemTime>,
    /// Maximum registrations per agent in time window
    max_per_agent: u32,
    /// Maximum global registrations in time window
    max_global: u32,
    /// Time window duration
    window_duration: Duration,
}

impl RegistrationRateLimit {
    fn new() -> Self {
        tracing::info!(
            "Initializing agent registration rate limiter - all previous rate limits cleared"
        );
        tracing::debug!("Rate limits: {} registrations per agent per {} seconds, {} global registrations per {} seconds",
                      limits::DEFAULT_MAX_AGENT_REGISTRATIONS_PER_AGENT,
                      limits::DEFAULT_AGENT_RATE_LIMIT_WINDOW_SECS,
                      limits::DEFAULT_MAX_AGENT_GLOBAL_REGISTRATIONS,
                      limits::DEFAULT_AGENT_RATE_LIMIT_WINDOW_SECS);

        Self {
            agent_registrations: HashMap::new(),
            global_registrations: Vec::new(),
            max_per_agent: limits::DEFAULT_MAX_AGENT_REGISTRATIONS_PER_AGENT,
            max_global: limits::DEFAULT_MAX_AGENT_GLOBAL_REGISTRATIONS,
            window_duration: Duration::from_secs(limits::DEFAULT_AGENT_RATE_LIMIT_WINDOW_SECS),
        }
    }

    /// Check if agent registration is allowed and record it if so
    fn check_and_record(&mut self, agent_id: AgentId) -> Result<()> {
        let now = SystemTime::now();
        let window_start = now - self.window_duration;

        // Clean up old entries first
        self.cleanup_old_entries(window_start);

        // Check global limit
        if self.global_registrations.len() >= self.max_global as usize {
            return Err(QollectiveError::validation(format!(
                "Global registration rate limit exceeded: {} registrations in {} seconds",
                self.max_global,
                self.window_duration.as_secs()
            )));
        }

        // Check per-agent limit
        let agent_count = self
            .agent_registrations
            .get(&agent_id)
            .map(|v| v.len())
            .unwrap_or(0);

        if agent_count >= self.max_per_agent as usize {
            return Err(QollectiveError::validation(format!(
                "Agent registration rate limit exceeded: {} registrations in {} seconds for agent {}",
                self.max_per_agent, self.window_duration.as_secs(), agent_id
            )));
        }

        // Record the registration
        self.agent_registrations
            .entry(agent_id)
            .or_insert_with(Vec::new)
            .push(now);
        self.global_registrations.push(now);

        tracing::debug!("Agent registration rate limit check passed - Agent: {}, current count: {}/{}, global count: {}/{}", 
                       agent_id, agent_count + 1, self.max_per_agent, self.global_registrations.len(), self.max_global);

        Ok(())
    }

    /// Clean up registrations older than the time window
    fn cleanup_old_entries(&mut self, window_start: SystemTime) {
        // Clean up global registrations
        self.global_registrations
            .retain(|&time| time >= window_start);

        // Clean up per-agent registrations
        for (_, timestamps) in self.agent_registrations.iter_mut() {
            timestamps.retain(|&time| time >= window_start);
        }

        // Remove empty agent entries
        self.agent_registrations
            .retain(|_, timestamps| !timestamps.is_empty());
    }
}

/// Agent registration request handler
#[derive(Debug)]
struct RegistrationHandler {
    registry: Arc<RwLock<AgentRegistry>>,
    client: Arc<A2AClient>,
    config: RegistryConfig,
    rate_limiter: Arc<RwLock<RegistrationRateLimit>>,
}

/// Capability query request handler
#[derive(Debug)]
struct QueryHandler {
    registry: Arc<RwLock<AgentRegistry>>,
    client: Arc<A2AClient>,
    router: Arc<CapabilityRouter>,
    config: RoutingConfig,
}

/// Dedicated handler for agent discovery requests that returns Vec<AgentInfo>
#[derive(Debug)]
struct AgentDiscoveryHandler {
    registry: Arc<RwLock<AgentRegistry>>,
    config: RoutingConfig,
}

/// Health monitoring request handler
#[derive(Debug)]
struct HealthHandler {
    registry: Arc<RwLock<AgentRegistry>>,
    client: Arc<A2AClient>,
    health_monitor: Arc<tokio::sync::Mutex<AgentHealthMonitor>>,
    config: HealthConfig,
}

// ============================================================================
// MAIN A2A SERVER
// ============================================================================

/// Comprehensive A2A server with integrated registry, routing, and health monitoring
#[derive(Debug)]
pub struct A2AServer {
    /// NATS server
    nats_server: NatsServer,
    /// Server configuration
    config: A2AServerConfig,
    /// Agent registry for managing registered agents
    registry: Arc<RwLock<AgentRegistry>>,
    /// A2A client for communication
    client: Arc<A2AClient>,
    /// Capability router
    router: Arc<CapabilityRouter>,
    /// Health monitor
    health_monitor: Arc<tokio::sync::Mutex<AgentHealthMonitor>>,
    /// Request handlers
    registration_handler: Arc<RegistrationHandler>,
    query_handler: Arc<QueryHandler>,
    discovery_handler: Arc<AgentDiscoveryHandler>,
    health_handler: Arc<HealthHandler>,
    /// Server statistics
    server_stats: Arc<RwLock<ServerStats>>,
    /// Request queue
    request_queue: Arc<RwLock<Vec<PendingRequest>>>,
}

/// Pending request in the queue
#[derive(Debug, Clone)]
struct PendingRequest {
    /// Request ID
    pub id: String,
    /// Request type
    pub request_type: RequestType,
    /// Request payload
    pub payload: serde_json::Value,
    /// Reply subject
    pub reply_subject: Option<String>,
    /// Queued timestamp
    pub queued_at: SystemTime,
    /// Priority
    pub priority: RequestPriority,
}

/// Request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    AgentRegistration,
    AgentDeregistration,
    CapabilityQuery,
    HealthCheck,
    HealthUpdate,
    AgentDiscovery,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl RegistrationHandler {
    /// Create a new registration handler
    pub fn new(
        registry: Arc<RwLock<AgentRegistry>>,
        client: Arc<A2AClient>,
        config: RegistryConfig,
    ) -> Self {
        Self {
            registry,
            client,
            config,
            rate_limiter: Arc::new(RwLock::new(RegistrationRateLimit::new())),
        }
    }

    /// Handle agent registration request using AgentRegistration
    pub async fn handle_registration(&self, registration: AgentRegistration) -> Result<()> {
        let agent_info = registration.agent_info;
        let metadata = registration.metadata;
        // Validate agent info
        self.validate_agent_info(&agent_info)?;

        // Check registry limits
        let stats = self.registry.read().await.get_stats().await?;
        if stats.total_agents >= self.config.max_agents {
            return Err(QollectiveError::validation(
                "Registry capacity exceeded".to_string(),
            ));
        }

        // Log agent registration with comprehensive details
        tracing::info!(
            "Agent registered: {} (ID: {})",
            agent_info.name,
            agent_info.id
        );
        tracing::debug!("Agent registration details: capabilities={:?}, health_status={:?}, last_heartbeat={:?}", 
                       agent_info.capabilities, agent_info.health_status, agent_info.last_heartbeat);

        // Log agent metadata if available
        if !agent_info.metadata.is_empty() {
            tracing::debug!("Agent metadata: {:?}", agent_info.metadata);
        }

        // Log additional metadata (metadata is always present in AgentRegistration)
        tracing::debug!("Agent version: {}", metadata.version);
        if let Some(ref build_info) = metadata.build_info {
            tracing::debug!("Agent build info: {}", build_info);
        }
        if !metadata.capabilities_metadata.is_empty() {
            tracing::debug!(
                "Agent capabilities metadata: {:?}",
                metadata.capabilities_metadata
            );
        }
        if let Some(ref perf_metrics) = metadata.performance_metrics {
            tracing::debug!(
                "Agent performance metrics: CPU: {:.1}%, Memory: {:.1}%, Active Tasks: {}/{}",
                perf_metrics.cpu_usage,
                perf_metrics.memory_usage,
                perf_metrics.active_tasks,
                perf_metrics.max_tasks
            );
        }
        if !metadata.custom_metadata.is_empty() {
            tracing::debug!("Agent custom metadata: {:?}", metadata.custom_metadata);
        }

        // Log NATS endpoint information (would need NATS config passed to handler for full details)
        tracing::debug!("NATS communication enabled for agent registration");

        // Log available subjects
        tracing::debug!(
            "NATS subjects - registration: {}, discovery: {}, health: {}, capabilities: {}",
            crate::constants::subjects::AGENT_REGISTRATION,
            crate::constants::subjects::AGENT_DISCOVERY,
            crate::constants::subjects::AGENT_HEALTH,
            crate::constants::subjects::AGENT_CAPABILITIES
        );

        // Log registry statistics
        if let Ok(stats) = self.registry.read().await.get_stats().await {
            tracing::debug!("Registry stats: {} total agents", stats.total_agents);
        }

        // Register agent in registry
        self.registry
            .write()
            .await
            .register_agent(agent_info.clone(), metadata.clone())
            .await?;

        tracing::info!("Agent registration announcement and event publishing skipped (transport configuration needed)");

        Ok(())
    }

    /// Handle agent deregistration request
    pub async fn handle_deregistration(&self, deregistration: DeregistrationRequest) -> Result<()> {
        tracing::info!(
            "Processing agent deregistration request - Agent ID: {}",
            deregistration.agent_id
        );
        if let Some(ref reason) = deregistration.reason {
            tracing::debug!("Deregistration reason: {}", reason);
        }

        // Validate the request
        if deregistration.agent_id.to_string().is_empty() {
            tracing::error!("Invalid deregistration request: empty agent ID");
            return Err(QollectiveError::validation("Invalid agent ID".to_string()));
        }

        // Get agent info before deregistering for the registry event
        let agent_info = {
            let registry = self.registry.read().await;
            if let Some(info) = registry.get_agent_by_id(&deregistration.agent_id) {
                tracing::debug!("Found agent in registry: '{}' (ID: {})", info.name, info.id);
                info.clone()
            } else {
                tracing::warn!(
                    "Agent not found in registry, using placeholder info for ID: {}",
                    deregistration.agent_id
                );
                AgentInfo {
                    id: deregistration.agent_id,
                    name: "Unknown".to_string(),
                    capabilities: vec![],
                    health_status: HealthStatus::Unknown,
                    last_heartbeat: std::time::SystemTime::now(),
                    metadata: std::collections::HashMap::new(),
                }
            }
        };

        // Deregister from registry
        tracing::debug!(
            "Removing agent from registry: '{}' (ID: {})",
            agent_info.name,
            agent_info.id
        );
        self.registry
            .write()
            .await
            .deregister_agent(deregistration.agent_id)
            .await?;

        tracing::info!("Agent deregistration announcement and event publishing skipped (transport configuration needed)");

        tracing::info!(
            "Agent deregistration completed - Agent: '{}' (ID: {})",
            agent_info.name,
            agent_info.id
        );
        Ok(())
    }

    /// Validate agent information
    fn validate_agent_info(&self, agent_info: &AgentInfo) -> Result<()> {
        if agent_info.name.trim().is_empty() {
            return Err(QollectiveError::validation(
                "Agent name cannot be empty".to_string(),
            ));
        }

        if agent_info.capabilities.len() > self.config.max_capabilities_per_agent {
            return Err(QollectiveError::validation(
                "Too many capabilities".to_string(),
            ));
        }

        if agent_info.capabilities.is_empty() {
            return Err(QollectiveError::validation(
                "Agent must have at least one capability".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl ContextDataHandler<AgentRegistration, RegistrationResponse> for RegistrationHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        registration: AgentRegistration,
    ) -> Result<RegistrationResponse> {
        let agent_id = registration.agent_info.id;
        let agent_name = registration.agent_info.name.clone();

        tracing::info!(
            "Processing agent registration request - Agent: '{}' (ID: {})",
            agent_name,
            agent_id
        );
        tracing::debug!(
            "Registration details: capabilities={:?}, health_status={:?}",
            registration.agent_info.capabilities,
            registration.agent_info.health_status
        );

        // Check rate limiting first
        match self.rate_limiter.write().await.check_and_record(agent_id) {
            Ok(()) => {
                tracing::debug!(
                    "Rate limit check passed for agent: '{}' (ID: {})",
                    agent_name,
                    agent_id
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Rate limit exceeded for agent registration - Agent: '{}' (ID: {}), Error: {}",
                    agent_name,
                    agent_id,
                    e
                );
                return Ok(RegistrationResponse {
                    success: false,
                    message: format!("Rate limit exceeded: {}", e),
                    agent_id,
                });
            }
        }

        // Handle the registration request using existing business logic
        match self.handle_registration(registration).await {
            Ok(()) => {
                tracing::info!(
                    "Agent registration successful - Agent: '{}' (ID: {})",
                    agent_name,
                    agent_id
                );
                Ok(RegistrationResponse {
                    success: true,
                    message: "Agent registered successfully".to_string(),
                    agent_id,
                })
            }
            Err(e) => {
                tracing::error!(
                    "Agent registration failed - Agent: '{}' (ID: {}), Error: {}",
                    agent_name,
                    agent_id,
                    e
                );
                Ok(RegistrationResponse {
                    success: false,
                    message: e.to_string(),
                    agent_id,
                })
            }
        }
    }
}

impl QueryHandler {
    /// Create a new query handler
    pub fn new(
        registry: Arc<RwLock<AgentRegistry>>,
        client: Arc<A2AClient>,
        router: Arc<CapabilityRouter>,
        config: RoutingConfig,
    ) -> Self {
        Self {
            registry,
            client,
            router,
            config,
        }
    }

    /// Handle capability query request
    pub async fn handle_capability_query(
        &self,
        query: CapabilityQuery,
        preferences: Option<RoutingPreferences>,
    ) -> Result<RoutingDecision> {
        // Validate query
        if query.required_capabilities.is_empty() && query.preferred_capabilities.is_empty() {
            return Err(QollectiveError::validation(
                "Query must specify at least one capability".to_string(),
            ));
        }

        // Find matching agents from local registry
        let query_result = self.registry.read().await.find_agents(&query).await?;

        if query_result.is_empty() {
            return Err(QollectiveError::validation(
                "No agents found matching query".to_string(),
            ));
        }

        // Route the request
        let routing_preferences = preferences.unwrap_or_default();
        self.router
            .route_capability(&query, query_result, &routing_preferences)
            .await
    }

    /// Handle agent discovery request
    pub async fn handle_agent_discovery(&self, query: CapabilityQuery) -> Result<Vec<AgentInfo>> {
        let result = self.registry.read().await.find_agents(&query).await?;

        // Apply server-side result limits
        let mut agents = result;
        if agents.len() > self.config.max_routing_cache_size {
            agents.truncate(self.config.max_routing_cache_size);
        }

        Ok(agents)
    }

    /// Handle agent card discovery request
    pub async fn handle_agent_card_discovery(
        &self,
        query: CapabilityQuery,
    ) -> Result<Vec<AgentCard>> {
        let agents = self.handle_agent_discovery(query).await?;

        // Convert agents to agent cards
        let agent_cards = agents
            .into_iter()
            .map(|agent| self.create_agent_card_from_info(&agent))
            .collect();

        Ok(agent_cards)
    }

    /// Create agent card from agent info
    fn create_agent_card_from_info(&self, agent_info: &AgentInfo) -> AgentCard {
        use crate::client::a2a::{AgentCapability, AuthMethod, AuthenticationRequirement};

        let capabilities = agent_info
            .capabilities
            .iter()
            .map(|cap| AgentCapability {
                name: cap.clone(),
                description: format!("Capability: {}", cap),
                input_types: vec!["application/json".to_string()],
                output_types: vec!["application/json".to_string()],
                parameters: HashMap::new(),
            })
            .collect();

        let mut metadata = HashMap::new();
        for (key, value) in &agent_info.metadata {
            metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        metadata.insert(
            "agent_id".to_string(),
            serde_json::Value::String(agent_info.id.to_string()),
        );

        AgentCard {
            name: agent_info.name.clone(),
            description: "A Qollective agent".to_string(),
            version: "1.0.0".to_string(),
            capabilities,
            endpoint_url: format!("qollective://agent/{}", agent_info.id),
            authentication: AuthenticationRequirement {
                method: AuthMethod::None,
                parameters: HashMap::new(),
            },
            protocol_version: "1.0".to_string(),
            metadata,
            tags: vec!["qollective".to_string(), "a2a".to_string()],
            provider: Some(crate::client::a2a::AgentProviderInfo {
                name: "Qollective Framework".to_string(),
                url: Some("https://github.com/qollective/qollective".to_string()),
                contact: Some("support@qollective.com".to_string()),
            }),
        }
    }
}

#[async_trait]
impl ContextDataHandler<QueryRequest, RoutingDecision> for QueryHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        request: QueryRequest,
    ) -> Result<RoutingDecision> {
        // Handle the capability query using existing business logic
        self.handle_capability_query(request.query, request.preferences)
            .await
    }
}

impl AgentDiscoveryHandler {
    /// Create a new agent discovery handler
    pub fn new(registry: Arc<RwLock<AgentRegistry>>, config: RoutingConfig) -> Self {
        Self { registry, config }
    }
}

#[async_trait]
impl ContextDataHandler<CapabilityQuery, Vec<AgentInfo>> for AgentDiscoveryHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        query: CapabilityQuery,
    ) -> Result<Vec<AgentInfo>> {
        // Use the registry to find agents matching the query
        let result = self.registry.read().await.find_agents(&query).await?;

        // Apply server-side result limits
        let mut agents = result;
        if agents.len() > self.config.max_routing_cache_size {
            agents.truncate(self.config.max_routing_cache_size);
        }

        Ok(agents)
    }
}

impl HealthHandler {
    /// Create a new health handler
    pub fn new(
        registry: Arc<RwLock<AgentRegistry>>,
        client: Arc<A2AClient>,
        health_monitor: Arc<tokio::sync::Mutex<AgentHealthMonitor>>,
        config: HealthConfig,
    ) -> Self {
        Self {
            registry,
            client,
            health_monitor,
            config,
        }
    }

    /// Handle health check request
    pub async fn handle_health_check(&self, agent_id: &AgentId) -> Result<HealthStatus> {
        // Check if circuit breaker is open
        let is_circuit_open = self
            .health_monitor
            .lock()
            .await
            .is_circuit_open(agent_id)
            .await;

        if is_circuit_open {
            return Ok(HealthStatus::Unhealthy);
        }

        // Would implement actual health check logic
        // For now, return healthy
        Ok(HealthStatus::Healthy)
    }

    /// Handle health status update
    pub async fn handle_health_update(&self, heartbeat: Heartbeat) -> Result<()> {
        // Record health result
        let is_healthy = heartbeat.health_status == HealthStatus::Healthy;
        self.health_monitor
            .lock()
            .await
            .record_health_result(&heartbeat.agent_id, is_healthy)
            .await;

        // Publish health update via A2A client
        let metadata = AgentMetadata::default();
        self.client
            .publish_health_status(heartbeat.agent_id, heartbeat.health_status, Some(metadata))
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ContextDataHandler<HealthCheckRequest, HealthCheckResponse> for HealthHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        request: HealthCheckRequest,
    ) -> Result<HealthCheckResponse> {
        // Handle the health check using existing business logic
        let status = self.handle_health_check(&request.agent_id).await?;

        Ok(HealthCheckResponse {
            agent_id: request.agent_id,
            status,
            timestamp: SystemTime::now(),
        })
    }
}

impl A2AServer {
    /// Create a new A2A server
    pub async fn new(config: A2AServerConfig) -> Result<Self> {
        let nats_server = NatsServer::new(crate::config::nats::NatsConfig {
            connection: config.nats_client.connection.clone(),
            client: config.nats_client.client_behavior.clone(),
            server: config.nats_server.clone(),
            discovery: crate::config::nats::NatsDiscoveryConfig {
                enabled: true, // Enable discovery for A2A server operation
                ..Default::default()
            },
        })
        .await?;

        // Initialize components with proper config inheritance (CONFIG FIRST PRINCIPLE)
        let registry = Arc::new(RwLock::new(
            AgentRegistry::new(
                config.registry.clone(),
                crate::config::nats::NatsConfig {
                    connection: config.nats_client.connection.clone(),
                    client: config.nats_client.client_behavior.clone(),
                    server: config.nats_server.clone(),
                    discovery: crate::config::nats::NatsDiscoveryConfig {
                        enabled: true, // Enable discovery for agent registry operation
                        ..Default::default()
                    },
                },
            )
            .await?,
        ));
        // Create A2A client config from server config (CONFIG FIRST PRINCIPLE)
        let client_config = crate::config::a2a::A2AClientConfig {
            client: crate::config::a2a::AgentClientConfig {
                agent_id: config.server_id.clone(),
                agent_name: config.server_name.clone(),
                capabilities: vec!["a2a_server".to_string()],
                // Use server's NATS config instead of hard-coded localhost
                nats_url: config
                    .nats_client
                    .connection
                    .urls
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "nats://localhost:4222".to_string()),
                ..Default::default()
            },
            transport: config.transport.clone(),
            nats_client: config.nats_client.clone(),
            discovery_cache_ttl_ms: 300000, // 5 minutes default
        };
        let client = Arc::new(A2AClient::new(client_config).await?);
        let router = Arc::new(CapabilityRouter::new(config.routing.clone()));
        let health_monitor = Arc::new(tokio::sync::Mutex::new(AgentHealthMonitor::new(
            config.health.clone(),
        )));

        // Create request handlers
        let registration_handler = Arc::new(RegistrationHandler::new(
            registry.clone(),
            client.clone(),
            config.registry.clone(),
        ));
        let query_handler = Arc::new(QueryHandler::new(
            registry.clone(),
            client.clone(),
            router.clone(),
            config.routing.clone(),
        ));
        let health_handler = Arc::new(HealthHandler::new(
            registry.clone(),
            client.clone(),
            health_monitor.clone(),
            config.health.clone(),
        ));
        let discovery_handler = Arc::new(AgentDiscoveryHandler::new(
            registry.clone(),
            config.routing.clone(),
        ));

        Ok(Self {
            nats_server,
            config: config.clone(),
            registry,
            client,
            router,
            health_monitor,
            registration_handler,
            query_handler,
            discovery_handler,
            health_handler,
            server_stats: Arc::new(RwLock::new(ServerStats {
                uptime: Duration::from_secs(0),
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_request_time_ms: 0.0,
                current_load: 0.0,
                registry_stats: RegistryStats {
                    total_agents: 0,
                    healthy_agents: 0,
                    unique_capabilities: 0,
                    avg_response_time_ms: 0.0,
                    uptime: Duration::from_secs(0),
                    last_cleanup: None,
                },
            })),
            request_queue: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Start the A2A server
    pub async fn start(&mut self) -> Result<()> {
        // A2A server connects to existing NATS infrastructure, never starts its own
        // The NATS connections are already established during construction

        // Set up NATS request handlers for A2A communication
        self.setup_request_handlers().await?;

        // Start the NATS server to begin processing messages
        tracing::info!("Starting NATS message processing");
        match self.nats_server.start().await {
            Ok(()) => {
                tracing::info!("NATS message processing started successfully");
            }
            Err(e) => {
                tracing::error!("Failed to start NATS message processing: {}", e);
                return Err(e);
            }
        }

        // Start background tasks for agent management
        self.start_background_tasks().await?;

        Ok(())
    }

    /// Set up NATS request handlers
    async fn setup_request_handlers(&mut self) -> Result<()> {
        tracing::info!("Setting up A2A server NATS request handlers");

        // Wrap handlers with DefaultEnvelopeHandler to bridge between
        // envelope processing and context-data processing

        // Agent discovery and registration
        let registration_envelope_handler =
            DefaultEnvelopeHandler::new(self.registration_handler.clone());
        // Handle both the standard registration subject and discovery announcement
        tracing::info!(
            "Registering handler for subject: {}",
            crate::constants::subjects::AGENT_REGISTRATION
        );
        match self
            .nats_server
            .handle(
                crate::constants::subjects::AGENT_REGISTRATION,
                registration_envelope_handler.clone(),
            )
            .await
        {
            Ok(()) => tracing::info!(
                "Successfully registered handler for: {}",
                crate::constants::subjects::AGENT_REGISTRATION
            ),
            Err(e) => {
                tracing::error!(
                    "Failed to register handler for {}: {}",
                    crate::constants::subjects::AGENT_REGISTRATION,
                    e
                );
                return Err(e);
            }
        }
        tracing::info!(
            "Registering handler for subject: {}",
            crate::constants::subjects::AGENT_REGISTRY_ANNOUNCE
        );
        match self
            .nats_server
            .handle(
                crate::constants::subjects::AGENT_REGISTRY_ANNOUNCE,
                registration_envelope_handler,
            )
            .await
        {
            Ok(()) => tracing::info!(
                "Successfully registered handler for: {}",
                crate::constants::subjects::AGENT_REGISTRY_ANNOUNCE
            ),
            Err(e) => {
                tracing::error!(
                    "Failed to register handler for {}: {}",
                    crate::constants::subjects::AGENT_REGISTRY_ANNOUNCE,
                    e
                );
                return Err(e);
            }
        }

        // Agent discovery requests
        let discovery_envelope_handler =
            DefaultEnvelopeHandler::new(self.discovery_handler.clone());
        tracing::info!(
            "Registering handler for subject: {}",
            crate::constants::subjects::AGENT_DISCOVERY
        );
        self.nats_server
            .handle(
                crate::constants::subjects::AGENT_DISCOVERY,
                discovery_envelope_handler,
            )
            .await?;

        // Agent capability announcements
        let query_envelope_handler = DefaultEnvelopeHandler::new(self.query_handler.clone());
        tracing::info!(
            "Registering handler for subject: {}",
            crate::constants::subjects::AGENT_CAPABILITIES
        );
        self.nats_server
            .handle(
                crate::constants::subjects::AGENT_CAPABILITIES,
                query_envelope_handler,
            )
            .await?;

        // Health check and heartbeat
        let health_envelope_handler = DefaultEnvelopeHandler::new(self.health_handler.clone());
        tracing::info!(
            "Registering handler for subject: {}",
            crate::constants::subjects::AGENT_HEALTH
        );
        self.nats_server
            .handle(
                crate::constants::subjects::AGENT_HEALTH,
                health_envelope_handler,
            )
            .await?;

        tracing::info!("All A2A server NATS handlers registered successfully");
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        tracing::info!("Starting A2A server background tasks");

        // Start registry cleanup task
        tracing::info!(
            "Starting registry cleanup task (interval: {:?})",
            self.config.registry.cleanup_interval
        );
        self.start_registry_cleanup_task().await;

        // Start health monitoring task
        tracing::info!(
            "Starting health monitoring task (interval: {:?})",
            self.config.health.check_interval
        );
        self.start_health_monitoring_task().await;

        // Start request queue processing task
        tracing::info!("Starting request queue processing task");
        self.start_request_processing_task().await;

        tracing::info!("All A2A server background tasks started successfully");
        Ok(())
    }

    /// Start registry cleanup background task
    async fn start_registry_cleanup_task(&self) {
        let registry = self.registry.clone();
        let cleanup_interval = self.config.registry.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                // Perform registry cleanup - remove stale agents
                match registry.write().await.cleanup_stale_agents().await {
                    Ok(removed_count) => {
                        if removed_count > 0 {
                            tracing::info!(
                                "Registry cleanup completed: removed {} stale agents",
                                removed_count
                            );
                        } else {
                            tracing::debug!("Registry cleanup completed: no stale agents found");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Registry cleanup failed: {}", e);
                    }
                }
            }
        });
    }

    /// Start health monitoring background task
    async fn start_health_monitoring_task(&self) {
        let _health_monitor = self.health_monitor.clone();
        let check_interval = self.config.health.check_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            loop {
                interval.tick().await;
                // Would implement periodic health checks
            }
        });
    }

    /// Start request processing background task
    async fn start_request_processing_task(&self) {
        let request_queue = self.request_queue.clone();
        let _registration_handler = self.registration_handler.clone();
        let _query_handler = self.query_handler.clone();
        let _health_handler = self.health_handler.clone();

        tokio::spawn(async move {
            loop {
                // Process queued requests
                let requests = {
                    let mut queue = request_queue.write().await;
                    let mut processed = Vec::new();

                    // Take up to 10 requests for processing
                    for _ in 0..10.min(queue.len()) {
                        if let Some(request) = queue.pop() {
                            processed.push(request);
                        }
                    }

                    processed
                };

                for request in requests {
                    // Process each request based on its type
                    match request.request_type {
                        RequestType::AgentRegistration => {
                            // Would deserialize and process registration
                        }
                        RequestType::CapabilityQuery => {
                            // Would deserialize and process query
                        }
                        RequestType::HealthCheck => {
                            // Would process health check
                        }
                        _ => {
                            // Handle other request types
                        }
                    }
                }

                // Sleep briefly before next iteration
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }

    /// Get comprehensive server statistics
    pub async fn get_server_stats(&self) -> Result<ServerStats> {
        let mut stats = self.server_stats.read().await.clone();

        // Update registry stats from AgentRegistry
        stats.registry_stats = self.registry.read().await.get_stats().await?;

        // Update current load (would be calculated based on various factors)
        stats.current_load = self.calculate_current_load().await;

        Ok(stats)
    }

    /// Calculate current server load
    async fn calculate_current_load(&self) -> f64 {
        // Would calculate based on request queue size, processing time, etc.
        let queue_size = self.request_queue.read().await.len();
        let max_queue_size = self.config.max_queue_size;

        (queue_size as f64 / max_queue_size as f64).min(1.0)
    }

    /// Handle agent registration request
    pub async fn handle_registration_request(
        &self,
        agent_info: AgentInfo,
        metadata: AgentMetadata,
    ) -> Result<()> {
        let start_time = SystemTime::now();

        let registration = AgentRegistration {
            agent_info,
            metadata,
        };
        let result = self
            .registration_handler
            .handle_registration(registration)
            .await;

        // Update statistics
        self.update_request_stats(start_time, result.is_ok()).await;

        result
    }

    /// Handle capability query request
    pub async fn handle_query_request(
        &self,
        query: CapabilityQuery,
        preferences: Option<RoutingPreferences>,
    ) -> Result<RoutingDecision> {
        let start_time = SystemTime::now();

        let result = self
            .query_handler
            .handle_capability_query(query, preferences)
            .await;

        // Update statistics
        self.update_request_stats(start_time, result.is_ok()).await;

        result
    }

    /// Update request statistics
    async fn update_request_stats(&self, start_time: SystemTime, success: bool) {
        let mut stats = self.server_stats.write().await;

        stats.total_requests += 1;
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        // Update average request time
        if let Ok(duration) = start_time.elapsed() {
            let request_time_ms = duration.as_millis() as f64;
            stats.avg_request_time_ms =
                (stats.avg_request_time_ms * (stats.total_requests - 1) as f64 + request_time_ms)
                    / stats.total_requests as f64;
        }
    }
}

// Default implementation for ServerStats
impl Default for ServerStats {
    fn default() -> Self {
        Self {
            uptime: Duration::from_secs(0),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_request_time_ms: 0.0,
            current_load: 0.0,
            registry_stats: RegistryStats {
                total_agents: 0,
                healthy_agents: 0,
                unique_capabilities: 0,
                avg_response_time_ms: 0.0,
                uptime: Duration::from_secs(0),
                last_cleanup: None,
            },
        }
    }
}
