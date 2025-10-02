// ABOUTME: A2A (Agent-to-Agent) configuration following consistent module pattern
// ABOUTME: Contains all agent-related configuration structures consolidated from agent module

//! A2A (Agent-to-Agent) configuration.
//!
//! This module provides configuration structures for all aspects of A2A communication:
//! - Agent client configuration
//! - Agent registry configuration
//! - Routing and health monitoring configuration
//! - Transport and discovery configuration
//! Follows the consistent configuration pattern used by other protocols.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Agent client configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentClientConfig {
    /// Agent identifier
    pub agent_id: String,
    /// Agent name
    pub agent_name: String,
    /// List of capabilities this agent provides
    pub capabilities: Vec<String>,
    /// NATS connection URL
    pub nats_url: String,
    /// A2A HTTP endpoint for standard protocol (optional)
    pub endpoint: Option<String>,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Discovery cache TTL
    pub discovery_cache_ttl: Duration,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Subject configuration
    pub subject_config: A2ASubjectConfig,
    /// Enable performance metrics
    pub enable_metrics: bool,
    /// Metadata for the agent
    pub metadata: std::collections::HashMap<String, String>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

/// A2A subject configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2ASubjectConfig {
    /// Subject prefix for A2A messages
    pub prefix: String,
    /// Request subject pattern
    pub request_pattern: String,
    /// Notification subject pattern
    pub notification_pattern: String,
    /// Heartbeat subject pattern
    pub heartbeat_pattern: String,
    /// Discovery subject pattern
    pub discovery_pattern: String,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryConfig {
    /// TTL for agent entries before considered stale
    pub agent_ttl: Duration,
    /// Interval for cleanup of stale agents
    pub cleanup_interval: Duration,
    /// Maximum number of agents to store
    pub max_agents: usize,
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
    /// Enable agent registration/deregistration logging
    pub enable_agent_logging: bool,
    /// NATS subject for agent registration/deregistration logs
    pub agent_log_subject: String,
    /// Enable capability indexing for faster queries
    pub enable_capability_indexing: bool,
    /// Maximum capabilities per agent
    pub max_capabilities_per_agent: usize,
    /// Capability name for logging agents (used for routing log messages)
    pub logging_agent_capability: String,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingConfig {
    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// Enable sticky routing based on client affinity
    pub enable_sticky_routing: bool,
    /// TTL for routing cache entries
    pub routing_cache_ttl: Duration,
    /// Maximum routing cache size
    pub max_routing_cache_size: usize,
    /// Minimum capability match score (0.0-1.0)
    pub min_capability_match_score: f64,
    /// Enable routing metrics collection
    pub enable_routing_metrics: bool,
    /// Timeout for capability queries
    pub capability_query_timeout: Duration,
}

/// Load balancing strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Random selection
    Random,
    /// Least connections
    LeastConnections,
    /// Weighted round-robin based on performance
    WeightedRoundRobin,
    /// Capability-based scoring
    CapabilityScoring,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthConfig {
    /// Heartbeat interval for health checks
    pub heartbeat_interval: Duration,
    /// Timeout for individual health checks
    pub health_check_timeout: Duration,
    /// Number of failed checks before marking unhealthy
    pub failure_threshold: u32,
    /// Number of successful checks to recover from unhealthy state
    pub recovery_threshold: u32,
    /// Interval between health checks
    pub check_interval: Duration,
    /// Enable circuit breaker for unhealthy agents
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Enable health metrics collection
    pub enable_health_metrics: bool,
}

/// Agent transport configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentTransportConfig {
    /// Default timeout for agent communication
    pub default_timeout: Duration,
    /// Maximum retries for failed communications
    pub max_retries: u32,
    /// Whether to prefer envelope protocols
    pub prefer_envelopes: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Discovery configuration
    pub discovery_config: AgentDiscoveryConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Recovery timeout when circuit is open
    pub recovery_timeout: Duration,
    /// Whether circuit breaker is enabled
    pub enabled: bool,
    /// Half-open request count before fully closing
    pub half_open_max_calls: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
}

/// Agent discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDiscoveryConfig {
    /// Enable external agent discovery
    pub enable_external_discovery: bool,
    /// External registry URLs
    pub external_registries: Vec<String>,
    /// Discovery cache TTL
    pub cache_ttl: Duration,
    /// Discovery timeout
    pub discovery_timeout: Duration,
    /// Batch size for bulk discovery operations
    pub discovery_batch_size: usize,
    /// Enable discovery metrics
    pub enable_discovery_metrics: bool,
}

/// Standalone A2A client configuration for API consistency.
///
/// This configuration provides everything an A2A client needs in a single config,
/// following the same pattern as NatsClientConfig, GrpcClientConfig, RestClientConfig, and McpClientConfig.
/// This ensures API consistency across all protocol clients.
///
/// # Examples
///
/// ```rust
/// use qollective::config::a2a::A2AClientConfig;
/// use std::time::Duration;
///
/// let config = A2AClientConfig::builder()
///     .with_agent_id("my-agent".to_string())
///     .with_capabilities(vec!["logging".to_string(), "analytics".to_string()])
///     .with_nats_url("nats://server:4222".to_string())
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2AClientConfig {
    /// Agent client configuration for the client instance
    pub client: AgentClientConfig,
    /// Transport configuration for client communication
    pub transport: AgentTransportConfig,
    /// NATS client configuration (extracted from full NATS config)
    pub nats_client: crate::config::nats::NatsClientConfig,
    /// Discovery cache TTL (client-relevant part from discovery config)
    pub discovery_cache_ttl_ms: u64,
}

impl Default for A2AClientConfig {
    fn default() -> Self {
        Self {
            client: AgentClientConfig::default(),
            transport: AgentTransportConfig::default(),
            nats_client: crate::config::nats::NatsClientConfig::default(),
            discovery_cache_ttl_ms: 300000, // 5 minutes, extracted from A2AConfig::default()
        }
    }
}

/// Builder for A2A client configuration
pub struct A2AClientConfigBuilder {
    config: A2AClientConfig,
}

impl A2AClientConfigBuilder {
    /// Create a new A2A client config builder
    pub fn new() -> Self {
        Self {
            config: A2AClientConfig::default(),
        }
    }

    /// Set agent ID
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.config.client.agent_id = agent_id;
        self
    }

    /// Set agent name
    pub fn with_agent_name(mut self, agent_name: String) -> Self {
        self.config.client.agent_name = agent_name;
        self
    }

    /// Set agent capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.config.client.capabilities = capabilities;
        self
    }

    /// Set NATS URL
    pub fn with_nats_url(mut self, nats_url: String) -> Self {
        self.config.client.nats_url = nats_url.clone();
        self.config.nats_client.connection.urls = vec![nats_url];
        self
    }

    /// Set NATS URLs
    pub fn with_nats_urls(mut self, urls: Vec<String>) -> Self {
        if !urls.is_empty() {
            self.config.client.nats_url = urls[0].clone(); // For backwards compatibility
            self.config.nats_client.connection.urls = urls;
        }
        self
    }

    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.config.client.heartbeat_interval = interval;
        self
    }

    /// Set discovery cache TTL
    pub fn with_discovery_cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.client.discovery_cache_ttl = ttl;
        self.config.discovery_cache_ttl_ms = ttl.as_millis() as u64;
        self.config.nats_client.discovery_cache_ttl_ms = ttl.as_millis() as u64;
        self
    }

    /// Set NATS client configuration
    pub fn with_nats_client_config(
        mut self,
        nats_config: crate::config::nats::NatsClientConfig,
    ) -> Self {
        // Keep nats_url in sync for backwards compatibility
        if !nats_config.connection.urls.is_empty() {
            self.config.client.nats_url = nats_config.connection.urls[0].clone();
        }
        self.config.nats_client = nats_config;
        self
    }

    /// Set transport configuration
    pub fn with_transport_config(mut self, transport_config: AgentTransportConfig) -> Self {
        self.config.transport = transport_config;
        self
    }

    /// Set maximum retries for transport
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.transport.max_retries = max_retries;
        self.config.client.retry_config.max_retries = max_retries;
        self
    }

    /// Set default timeout for transport
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.config.transport.default_timeout = timeout;
        self
    }

    /// Set agent metadata
    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.config.client.metadata = metadata;
        self
    }

    /// Build the configuration
    pub fn build(self) -> A2AClientConfig {
        self.config
    }
}

impl Default for A2AClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl A2AClientConfig {
    /// Create a new builder for A2A client configuration
    pub fn builder() -> A2AClientConfigBuilder {
        A2AClientConfigBuilder::new()
    }

    /// Validates the A2A client configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate client config
        if self.client.agent_id.trim().is_empty() {
            return Err("Agent ID cannot be empty".to_string());
        }

        if self.client.agent_name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }

        // Validate NATS client config
        self.nats_client.validate()?;

        // Validate transport config
        if self.transport.max_retries == 0 {
            return Err("Transport max retries must be greater than 0".to_string());
        }

        if self.discovery_cache_ttl_ms == 0 {
            return Err("Discovery cache TTL must be greater than 0".to_string());
        }

        // Validate consistency between nats_url and nats_client.connection.urls
        if !self.nats_client.connection.urls.is_empty() {
            let first_url = &self.nats_client.connection.urls[0];
            if self.client.nats_url != *first_url {
                return Err(
                    "NATS URL in client config must match first URL in NATS client config"
                        .to_string(),
                );
            }
        }

        Ok(())
    }
}

/// Standalone A2A server configuration for API consistency.
///
/// This configuration provides everything an A2A server needs in a single config,
/// following the same pattern as A2AClientConfig and other protocol server configs.
/// This ensures API consistency across all protocol servers.
///
/// # Examples
///
/// ```rust
/// use qollective::config::a2a::A2AServerConfig;
/// use std::time::Duration;
///
/// let config = A2AServerConfig::builder()
///     .with_registry_enabled(true)
///     .with_max_agents(1000)
///     .with_heartbeat_interval(Duration::from_secs(30))
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2AServerConfig {
    /// Server identifier
    pub server_id: String,
    /// Server name
    pub server_name: String,
    /// Agent registry configuration for managing registered agents
    pub registry: RegistryConfig,
    /// Routing configuration for capability-based routing
    pub routing: RoutingConfig,
    /// Health monitoring configuration for agent health tracking
    pub health: HealthConfig,
    /// Transport configuration for server communication
    pub transport: AgentTransportConfig,
    /// NATS server configuration (extracted from full NATS config)
    pub nats_server: crate::config::nats::NatsServerConfig,
    /// NATS client configuration for server's client operations
    pub nats_client: crate::config::nats::NatsClientConfig,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Request timeout
    pub request_timeout: Duration,
    /// Enable request queuing
    pub enable_request_queuing: bool,
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Requests per second limit
    pub requests_per_second: u32,
}

impl Default for A2AServerConfig {
    fn default() -> Self {
        Self {
            server_id: "default-a2a-server".to_string(),
            server_name: "Default A2A Server".to_string(),
            registry: RegistryConfig::default(),
            routing: RoutingConfig::default(),
            health: HealthConfig::default(),
            transport: AgentTransportConfig::default(),
            nats_server: crate::config::nats::NatsServerConfig::default(),
            nats_client: crate::config::nats::NatsClientConfig::default(),
            max_concurrent_requests: 1000,
            request_timeout: Duration::from_secs(30),
            enable_request_queuing: true,
            max_queue_size: 10000,
            enable_rate_limiting: false,
            requests_per_second: 1000,
        }
    }
}

/// Builder for A2A server configuration
pub struct A2AServerConfigBuilder {
    config: A2AServerConfig,
}

impl A2AServerConfigBuilder {
    /// Create a new A2A server config builder
    pub fn new() -> Self {
        Self {
            config: A2AServerConfig::default(),
        }
    }

    /// Set server ID
    pub fn with_server_id(mut self, server_id: String) -> Self {
        self.config.server_id = server_id;
        self
    }

    /// Set server name
    pub fn with_server_name(mut self, server_name: String) -> Self {
        self.config.server_name = server_name;
        self
    }

    /// Set registry configuration
    pub fn with_registry_config(mut self, registry_config: RegistryConfig) -> Self {
        self.config.registry = registry_config;
        self
    }

    /// Enable registry
    pub fn with_registry_enabled(mut self, enabled: bool) -> Self {
        self.config.registry.enable_health_monitoring = enabled;
        self.config.registry.enable_agent_logging = enabled;
        self
    }

    /// Set maximum number of agents
    pub fn with_max_agents(mut self, max_agents: usize) -> Self {
        self.config.registry.max_agents = max_agents;
        self
    }

    /// Set routing configuration
    pub fn with_routing_config(mut self, routing_config: RoutingConfig) -> Self {
        self.config.routing = routing_config;
        self
    }

    /// Set load balancing strategy
    pub fn with_load_balancing_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.config.routing.load_balancing_strategy = strategy;
        self
    }

    /// Set health configuration
    pub fn with_health_config(mut self, health_config: HealthConfig) -> Self {
        self.config.health = health_config;
        self
    }

    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.config.health.heartbeat_interval = interval;
        self
    }

    /// Set transport configuration
    pub fn with_transport_config(mut self, transport_config: AgentTransportConfig) -> Self {
        self.config.transport = transport_config;
        self
    }

    /// Set NATS server configuration
    pub fn with_nats_server_config(
        mut self,
        nats_server_config: crate::config::nats::NatsServerConfig,
    ) -> Self {
        self.config.nats_server = nats_server_config;
        self
    }

    /// Set NATS client configuration
    pub fn with_nats_client_config(
        mut self,
        nats_client_config: crate::config::nats::NatsClientConfig,
    ) -> Self {
        self.config.nats_client = nats_client_config;
        self
    }

    /// Enable NATS server
    pub fn with_nats_server_enabled(mut self, enabled: bool) -> Self {
        self.config.nats_server.enabled = enabled;
        self
    }

    /// Set NATS subject prefix
    pub fn with_nats_subject_prefix(mut self, prefix: String) -> Self {
        self.config.nats_server.subject_prefix = prefix;
        self
    }

    /// Set NATS URLs for client operations
    pub fn with_nats_urls(mut self, urls: Vec<String>) -> Self {
        self.config.nats_client.connection.urls = urls;
        self
    }

    /// Set maximum concurrent requests
    pub fn with_max_concurrent_requests(mut self, max_requests: usize) -> Self {
        self.config.max_concurrent_requests = max_requests;
        self
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Enable request queuing
    pub fn with_request_queuing(mut self, enabled: bool) -> Self {
        self.config.enable_request_queuing = enabled;
        self
    }

    /// Set maximum queue size
    pub fn with_max_queue_size(mut self, max_size: usize) -> Self {
        self.config.max_queue_size = max_size;
        self
    }

    /// Enable rate limiting
    pub fn with_rate_limiting(mut self, enabled: bool, requests_per_second: u32) -> Self {
        self.config.enable_rate_limiting = enabled;
        self.config.requests_per_second = requests_per_second;
        self
    }

    /// Build the configuration
    pub fn build(self) -> A2AServerConfig {
        self.config
    }
}

impl Default for A2AServerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl A2AServerConfig {
    /// Create a new builder for A2A server configuration
    pub fn builder() -> A2AServerConfigBuilder {
        A2AServerConfigBuilder::new()
    }

    /// Validates the A2A server configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server identity
        if self.server_id.trim().is_empty() {
            return Err("Server ID cannot be empty".to_string());
        }

        if self.server_name.trim().is_empty() {
            return Err("Server name cannot be empty".to_string());
        }

        // Validate registry config
        if self.registry.max_agents == 0 {
            return Err("Max agents must be greater than 0".to_string());
        }

        // Validate routing config
        if self.routing.min_capability_match_score < 0.0
            || self.routing.min_capability_match_score > 1.0
        {
            return Err("Min capability match score must be between 0.0 and 1.0".to_string());
        }

        // Validate health config
        if self.health.failure_threshold == 0 {
            return Err("Health failure threshold must be greater than 0".to_string());
        }

        if self.health.recovery_threshold == 0 {
            return Err("Health recovery threshold must be greater than 0".to_string());
        }

        // Validate transport config
        if self.transport.max_retries == 0 {
            return Err("Transport max retries must be greater than 0".to_string());
        }

        // Validate request handling
        if self.max_concurrent_requests == 0 {
            return Err("Max concurrent requests must be greater than 0".to_string());
        }

        if self.max_queue_size == 0 {
            return Err("Max queue size must be greater than 0".to_string());
        }

        if self.enable_rate_limiting && self.requests_per_second == 0 {
            return Err(
                "Requests per second must be greater than 0 when rate limiting is enabled"
                    .to_string(),
            );
        }

        // Validate NATS client config
        self.nats_client.validate()?;

        Ok(())
    }
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

// Default implementations

impl Default for AgentClientConfig {
    fn default() -> Self {
        Self {
            agent_id: "default-agent".to_string(),
            agent_name: "Default Agent".to_string(),
            capabilities: vec![],
            nats_url: "nats://localhost:4222".to_string(),
            endpoint: None, // No default A2A endpoint
            heartbeat_interval: Duration::from_secs(30),
            discovery_cache_ttl: Duration::from_secs(300),
            retry_config: RetryConfig::default(),
            subject_config: A2ASubjectConfig::default(),
            enable_metrics: true,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for A2ASubjectConfig {
    fn default() -> Self {
        Self {
            prefix: "qollective.agent".to_string(),
            request_pattern: "qollective.agent.{agent_id}.request.v1".to_string(),
            notification_pattern: "qollective.agent.{agent_id}.notify.v1".to_string(),
            heartbeat_pattern: "qollective.agent.heartbeat.v1".to_string(),
            discovery_pattern: "qollective.agent.discovery.v1".to_string(),
        }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            agent_ttl: Duration::from_secs(300),
            cleanup_interval: Duration::from_secs(60),
            max_agents: 10000,
            enable_health_monitoring: true,
            enable_agent_logging: true,
            agent_log_subject: "qollective.agent.registry.log.v1".to_string(),
            enable_capability_indexing: true,
            max_capabilities_per_agent: 100,
            logging_agent_capability: "logging".to_string(),
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            load_balancing_strategy: LoadBalancingStrategy::CapabilityScoring,
            enable_sticky_routing: true,
            routing_cache_ttl: Duration::from_secs(300),
            max_routing_cache_size: 1000,
            min_capability_match_score: 0.7,
            enable_routing_metrics: true,
            capability_query_timeout: Duration::from_secs(10),
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            check_interval: Duration::from_secs(60),
            enable_circuit_breaker: true,
            circuit_breaker: CircuitBreakerConfig::default(),
            enable_health_metrics: true,
        }
    }
}

impl Default for AgentTransportConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            prefer_envelopes: true,
            circuit_breaker_config: CircuitBreakerConfig::default(),
            discovery_config: AgentDiscoveryConfig::default(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            enabled: true,
            half_open_max_calls: 3,
            success_threshold: 2,
        }
    }
}

impl Default for AgentDiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_external_discovery: true,
            external_registries: vec![],
            cache_ttl: Duration::from_secs(300),
            discovery_timeout: Duration::from_secs(10),
            discovery_batch_size: 50,
            enable_discovery_metrics: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_group_config_validation() {
        let valid_config = QueueGroupConfig::new("test-capability", "v1");
        assert!(valid_config.validate().is_ok());

        let invalid_config = QueueGroupConfig {
            queue_name: "invalid..name".to_string(),
            capability: "test".to_string(),
            version: "v1".to_string(),
            auto_scale: true,
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_load_balancing_strategies() {
        assert_eq!(
            LoadBalancingStrategy::CapabilityScoring,
            LoadBalancingStrategy::CapabilityScoring
        );
        assert_ne!(
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::Random
        );
    }

    #[test]
    fn test_a2a_client_config_default() {
        let config = A2AClientConfig::default();
        assert_eq!(config.client.agent_id, "default-agent");
        assert_eq!(config.client.agent_name, "Default Agent");
        assert_eq!(
            config.nats_client.connection.urls,
            vec!["nats://localhost:4222"]
        );
        assert_eq!(config.discovery_cache_ttl_ms, 300000);
        assert!(config.transport.prefer_envelopes);
    }

    #[test]
    fn test_a2a_client_config_builder() {
        let config = A2AClientConfig::builder()
            .with_agent_id("test-agent".to_string())
            .with_agent_name("Test Agent".to_string())
            .with_capabilities(vec!["logging".to_string(), "analytics".to_string()])
            .with_nats_url("nats://server:4222".to_string())
            .with_heartbeat_interval(std::time::Duration::from_secs(60))
            .with_max_retries(5)
            .build();

        assert_eq!(config.client.agent_id, "test-agent");
        assert_eq!(config.client.agent_name, "Test Agent");
        assert_eq!(config.client.capabilities, vec!["logging", "analytics"]);
        assert_eq!(config.client.nats_url, "nats://server:4222");
        assert_eq!(
            config.nats_client.connection.urls,
            vec!["nats://server:4222"]
        );
        assert_eq!(
            config.client.heartbeat_interval,
            std::time::Duration::from_secs(60)
        );
        assert_eq!(config.transport.max_retries, 5);
        assert_eq!(config.client.retry_config.max_retries, 5);
    }

    #[test]
    fn test_a2a_client_config_validation() {
        let valid_config = A2AClientConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = A2AClientConfig {
            client: AgentClientConfig {
                agent_id: "".to_string(), // Empty agent ID should be invalid
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
        assert!(invalid_config
            .validate()
            .unwrap_err()
            .contains("Agent ID cannot be empty"));

        let invalid_config2 = A2AClientConfig {
            transport: AgentTransportConfig {
                max_retries: 0, // Zero retries should be invalid
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config2.validate().is_err());
        assert!(invalid_config2
            .validate()
            .unwrap_err()
            .contains("Transport max retries must be greater than 0"));
    }

    #[test]
    fn test_a2a_client_config_nats_url_consistency() {
        let config = A2AClientConfig::builder()
            .with_nats_urls(vec![
                "nats://server1:4222".to_string(),
                "nats://server2:4222".to_string(),
            ])
            .build();

        // Should keep nats_url in sync with first URL in NATS client config
        assert_eq!(config.client.nats_url, "nats://server1:4222");
        assert_eq!(
            config.nats_client.connection.urls,
            vec!["nats://server1:4222", "nats://server2:4222"]
        );
        assert!(config.validate().is_ok());

        // Test validation catches inconsistency
        let inconsistent_config = A2AClientConfig {
            client: AgentClientConfig {
                nats_url: "nats://different:4222".to_string(),
                ..Default::default()
            },
            nats_client: crate::config::nats::NatsClientConfig {
                connection: crate::config::nats::NatsConnectionConfig {
                    urls: vec!["nats://server:4222".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(inconsistent_config.validate().is_err());
        assert!(inconsistent_config
            .validate()
            .unwrap_err()
            .contains("NATS URL in client config must match first URL in NATS client config"));
    }

    #[test]
    fn test_a2a_client_config_serialization() {
        let config = A2AClientConfig::builder()
            .with_agent_id("test-agent".to_string())
            .with_nats_url("nats://test:4222".to_string())
            .build();

        // Test serialization
        let serialized = serde_json::to_string(&config).expect("Should serialize");
        assert!(serialized.contains("test-agent"));
        assert!(serialized.contains("nats://test:4222"));
        assert!(serialized.contains("nats_client"));

        // Test deserialization
        let deserialized: A2AClientConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(deserialized.client.agent_id, "test-agent");
        assert_eq!(deserialized.client.nats_url, "nats://test:4222");
        assert_eq!(
            deserialized.nats_client.connection.urls,
            vec!["nats://test:4222"]
        );
    }

    #[test]
    fn test_a2a_server_config_default() {
        let config = A2AServerConfig::default();
        assert_eq!(config.registry.max_agents, 10000);
        assert_eq!(
            config.routing.load_balancing_strategy,
            LoadBalancingStrategy::CapabilityScoring
        );
        assert!(config.health.enable_circuit_breaker);
        assert!(!config.nats_server.enabled);
        assert_eq!(
            config.nats_client.connection.urls,
            vec!["nats://localhost:4222"]
        );
    }

    #[test]
    fn test_a2a_server_config_builder() {
        let config = A2AServerConfig::builder()
            .with_registry_enabled(true)
            .with_max_agents(5000)
            .with_load_balancing_strategy(LoadBalancingStrategy::RoundRobin)
            .with_heartbeat_interval(std::time::Duration::from_secs(45))
            .with_nats_server_enabled(true)
            .with_nats_subject_prefix("test-agents".to_string())
            .with_nats_urls(vec!["nats://server:4222".to_string()])
            .build();

        assert_eq!(config.registry.max_agents, 5000);
        assert!(config.registry.enable_health_monitoring);
        assert!(config.registry.enable_agent_logging);
        assert_eq!(
            config.routing.load_balancing_strategy,
            LoadBalancingStrategy::RoundRobin
        );
        assert_eq!(
            config.health.heartbeat_interval,
            std::time::Duration::from_secs(45)
        );
        assert!(config.nats_server.enabled);
        assert_eq!(config.nats_server.subject_prefix, "test-agents");
        assert_eq!(
            config.nats_client.connection.urls,
            vec!["nats://server:4222"]
        );
    }

    #[test]
    fn test_a2a_server_config_validation() {
        let valid_config = A2AServerConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = A2AServerConfig {
            registry: RegistryConfig {
                max_agents: 0, // Zero agents should be invalid
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
        assert!(invalid_config
            .validate()
            .unwrap_err()
            .contains("Max agents must be greater than 0"));

        let invalid_config2 = A2AServerConfig {
            routing: RoutingConfig {
                min_capability_match_score: 1.5, // Score > 1.0 should be invalid
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config2.validate().is_err());
        assert!(invalid_config2
            .validate()
            .unwrap_err()
            .contains("Min capability match score must be between 0.0 and 1.0"));

        let invalid_config3 = A2AServerConfig {
            health: HealthConfig {
                failure_threshold: 0, // Zero threshold should be invalid
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config3.validate().is_err());
        assert!(invalid_config3
            .validate()
            .unwrap_err()
            .contains("Health failure threshold must be greater than 0"));
    }

    #[test]
    fn test_a2a_server_config_serialization() {
        let config = A2AServerConfig::builder()
            .with_registry_enabled(true)
            .with_nats_server_enabled(true)
            .with_nats_urls(vec!["nats://test:4222".to_string()])
            .build();

        // Test serialization
        let serialized = serde_json::to_string(&config).expect("Should serialize");
        assert!(serialized.contains("nats://test:4222"));
        assert!(serialized.contains("nats_server"));
        assert!(serialized.contains("nats_client"));
        assert!(serialized.contains("registry"));

        // Test deserialization
        let deserialized: A2AServerConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert!(deserialized.registry.enable_health_monitoring);
        assert!(deserialized.nats_server.enabled);
        assert_eq!(
            deserialized.nats_client.connection.urls,
            vec!["nats://test:4222"]
        );
    }
}
