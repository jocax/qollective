// ABOUTME: MCP (Model Context Protocol) configuration structures for client, server, and transport
// ABOUTME: Provides centralized configuration following framework patterns with validation and defaults

//! MCP configuration management for the Qollective framework.
//!
//! This module provides configuration structures for MCP client, server, and transport
//! components, following the established patterns from other protocols (gRPC, REST, NATS).

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use crate::config::nats::NatsConfig;
// use crate::transport::{TransportCapabilities, TransportProtocol};
use crate::transport::{TransportCapabilities, TransportProtocol};
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for MCP client operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// NATS configuration for transport (fallback and chain/distributed execution)
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub nats_config: NatsConfig,
    /// Registry configuration
    pub registry_config: McpServerRegistryConfig,
    /// Tool chain executor configuration
    pub chain_executor_config: ToolChainExecutorConfig,
    /// Distributed executor configuration
    pub distributed_executor_config: DistributedExecutionConfig,
    /// Enable automatic tool discovery
    pub enable_auto_discovery: bool,
    /// Enable result caching
    pub enable_caching: bool,
    /// Cache TTL
    pub cache_ttl: Duration,
    /// Default timeout for operations
    pub default_timeout: Duration,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            nats_config: NatsConfig::default(),
            registry_config: McpServerRegistryConfig::default(),
            chain_executor_config: ToolChainExecutorConfig::default(),
            distributed_executor_config: DistributedExecutionConfig::default(),
            enable_auto_discovery: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(300),
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// Configuration for MCP server registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRegistryConfig {
    /// NATS configuration for transport
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub nats_config: NatsConfig,
    /// Registry name for identification
    pub registry_name: String,
    /// Enable async connectivity for MCP servers
    pub enable_async_connectivity: bool,
    /// Timeout for async operations
    pub async_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum number of servers to track
    pub max_servers: usize,
}

impl Default for McpServerRegistryConfig {
    fn default() -> Self {
        Self {
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            nats_config: NatsConfig::default(),
            registry_name: "qollective-mcp-registry".to_string(),
            enable_async_connectivity: true,
            async_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60),
            max_servers: 100,
        }
    }
}

/// Configuration for tool chain executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainExecutorConfig {
    /// Maximum concurrent executions
    pub max_concurrent_chains: usize,
    /// Default timeout for tool calls
    pub default_tool_timeout: Duration,
    /// Default timeout for chain execution
    pub default_chain_timeout: Duration,
    /// Enable result caching
    pub enable_caching: bool,
    /// Cache TTL
    pub cache_ttl: Duration,
}

impl Default for ToolChainExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_chains: 10,
            default_tool_timeout: Duration::from_secs(30),
            default_chain_timeout: Duration::from_secs(300), // 5 minutes
            enable_caching: true,
            cache_ttl: Duration::from_secs(300),
        }
    }
}

/// Configuration for distributed tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedExecutionConfig {
    /// Maximum concurrent executions per server
    pub max_concurrent_per_server: usize,
    /// Maximum total concurrent executions
    pub max_total_concurrent: usize,
    /// Default timeout for distributed operations
    pub default_timeout: Duration,
    /// Enable load balancing across servers
    pub enable_load_balancing: bool,
    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for DistributedExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_per_server: 5,
            max_total_concurrent: 50,
            default_timeout: Duration::from_secs(30),
            enable_load_balancing: true,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

/// Load balancing strategies for distributed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Choose server with lowest current load
    LeastConnections,
    /// Choose fastest responding server
    FastestResponse,
    /// Random server selection
    Random,
}

/// Configuration for MCP transport client - simplified for rmcp integration
#[derive(Debug, Clone)]
pub struct McpTransportConfig {
    /// Transport detection configuration
    pub detection_config: TransportDetectionConfig,
    /// Retry configuration  
    pub retry_config: TransportRetryConfig,
    /// Timeout configuration
    pub timeout_config: TransportTimeoutConfig,
}

impl Default for McpTransportConfig {
    fn default() -> Self {
        Self {
            detection_config: TransportDetectionConfig::default(),
            retry_config: TransportRetryConfig::default(),
            timeout_config: TransportTimeoutConfig::default(),
        }
    }
}

/// Configuration for transport detection - simplified for rmcp integration
#[derive(Debug, Clone)]
pub struct TransportDetectionConfig {
    /// Whether to enable automatic detection
    pub enable_auto_detection: bool,
    /// Detection timeout
    pub detection_timeout: Duration,
    /// Whether to retry failed detections
    pub retry_failed_detections: bool,
    /// Maximum retry attempts for detection
    pub max_detection_retries: u32,
}

impl Default for TransportDetectionConfig {
    fn default() -> Self {
        Self {
            enable_auto_detection: true,
            detection_timeout: Duration::from_secs(5),
            retry_failed_detections: true,
            max_detection_retries: 3,
        }
    }
}


/// Configuration for transport retry behavior
#[derive(Debug, Clone)]
pub struct TransportRetryConfig {
    /// Enable automatic retries
    pub enable_retries: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for TransportRetryConfig {
    fn default() -> Self {
        Self {
            enable_retries: true,
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Configuration for transport timeouts
#[derive(Debug, Clone)]
pub struct TransportTimeoutConfig {
    /// Default connection timeout
    pub connection_timeout: Duration,
    /// Default request timeout
    pub request_timeout: Duration,
    /// Default discovery timeout
    pub discovery_timeout: Duration,
    /// Default health check timeout
    pub health_check_timeout: Duration,
}

impl Default for TransportTimeoutConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            discovery_timeout: Duration::from_secs(5),
            health_check_timeout: Duration::from_secs(5),
        }
    }
}

/// Configuration for MCP server
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Base server configuration
    pub base: McpBaseServerConfig,
    /// Registry configuration for tool/resource management
    pub registry_config: McpServerRegistryConfig,
    /// Server information for MCP protocol
    pub server_info: McpServerInfo,
    /// Available tools
    pub tools: Vec<McpToolConfig>,
    /// Available resources
    pub resources: Vec<McpResourceConfig>,
    /// Available prompts
    pub prompts: Vec<McpPromptConfig>,
    /// Enable envelope integration
    pub enable_envelope_integration: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            base: McpBaseServerConfig::default(),
            registry_config: McpServerRegistryConfig::default(),
            server_info: McpServerInfo::default(),
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            enable_envelope_integration: true,
        }
    }
}

/// Base server configuration
#[derive(Debug, Clone)]
pub struct McpBaseServerConfig {
    /// Server bind address
    pub bind_address: String,
    /// Server port
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS configuration
    pub tls_config: Option<McpTlsConfig>,
}

impl Default for McpBaseServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            enable_tls: false,
            tls_config: None,
        }
    }
}

/// Server information for MCP protocol
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server description
    pub description: Option<String>,
    /// Server metadata
    pub metadata: HashMap<String, String>,
}

impl Default for McpServerInfo {
    fn default() -> Self {
        Self {
            name: "Qollective MCP Server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Qollective MCP Server".to_string()),
            metadata: HashMap::new(),
        }
    }
}

/// Configuration for MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolConfig {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Input schema
    pub input_schema: serde_json::Value,
    /// Tool handler configuration
    pub handler_config: Option<HashMap<String, String>>,
}

/// Configuration for MCP resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceConfig {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Resource handler configuration
    pub handler_config: Option<HashMap<String, String>>,
}

/// Configuration for MCP prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptConfig {
    /// Prompt name
    pub name: String,
    /// Prompt description
    pub description: Option<String>,
    /// Prompt template
    pub template: String,
    /// Required arguments
    pub arguments: Vec<McpPromptArgumentConfig>,
}

/// Configuration for MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgumentConfig {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: Option<String>,
    /// Whether argument is required
    pub required: bool,
    /// Argument type
    pub argument_type: Option<String>,
}

/// TLS configuration for MCP server
#[derive(Debug, Clone)]
pub struct McpTlsConfig {
    /// Path to certificate file
    pub cert_file: String,
    /// Path to private key file
    pub key_file: String,
    /// Path to CA certificate file (for client authentication)
    pub ca_file: Option<String>,
    /// Require client certificates
    pub require_client_cert: bool,
}

/// Builder for MCP client configuration
pub struct McpClientConfigBuilder {
    config: McpClientConfig,
}

impl McpClientConfigBuilder {
    /// Create a new MCP client config builder
    pub fn new() -> Self {
        Self {
            config: McpClientConfig::default(),
        }
    }

    /// Set NATS configuration
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_config(mut self, nats_config: NatsConfig) -> Self {
        self.config.nats_config = nats_config;
        self
    }

    /// Set registry configuration
    pub fn with_registry_config(mut self, registry_config: McpServerRegistryConfig) -> Self {
        self.config.registry_config = registry_config;
        self
    }

    /// Set tool chain executor configuration
    pub fn with_chain_executor_config(mut self, chain_config: ToolChainExecutorConfig) -> Self {
        self.config.chain_executor_config = chain_config;
        self
    }

    /// Set distributed executor configuration
    pub fn with_distributed_config(
        mut self,
        distributed_config: DistributedExecutionConfig,
    ) -> Self {
        self.config.distributed_executor_config = distributed_config;
        self
    }

    /// Enable/disable auto discovery
    pub fn with_auto_discovery(mut self, enabled: bool) -> Self {
        self.config.enable_auto_discovery = enabled;
        self
    }

    /// Enable/disable caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.config.enable_caching = enabled;
        self
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.cache_ttl = ttl;
        self
    }

    /// Set default timeout
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    /// Build the configuration
    pub fn build(self) -> McpClientConfig {
        self.config
    }
}

impl Default for McpClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TRANSPORT CONFIGURATION TYPES (moved from src/transport/mcp.rs)
// ============================================================================

/// MCP transport client configuration - unified transport settings
#[derive(Debug, Clone)]
pub struct McpTransportClientConfig {
    /// Connection timeout for discovery
    pub discovery_timeout: Duration,
    /// Maximum number of rmcp clients to cache
    pub max_rmcp_clients: usize,
    /// Enable connection pooling for rmcp clients
    pub enable_connection_pooling: bool,
    /// Maximum connections per rmcp client
    pub max_connections_per_client: u32,
    /// Keep-alive timeout for connections
    pub keep_alive_timeout: Duration,
}

impl Default for McpTransportClientConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: Duration::from_millis(
                crate::constants::timeouts::DEFAULT_MCP_DISCOVERY_TIMEOUT_MS,
            ),
            max_rmcp_clients: crate::constants::limits::DEFAULT_MCP_MAX_CACHED_CLIENTS,
            enable_connection_pooling: true,
            max_connections_per_client:
                crate::constants::limits::DEFAULT_MCP_MAX_CONNECTIONS_PER_CLIENT,
            keep_alive_timeout: Duration::from_secs(60),
        }
    }
}

/// MCP server endpoint information with transport capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEndpoint {
    /// Server unique identifier
    pub server_id: String,
    /// Server endpoint URL
    pub endpoint_url: String,
    /// Transport capabilities for this endpoint
    pub capabilities: TransportCapabilities,
    /// MCP protocol version
    pub mcp_version: String,
    /// Tools supported by this server
    pub supported_tools: Vec<Tool>,
    /// Preferred transport protocol
    pub preferred_transport: TransportProtocol,
    /// Whether this is a Qollective-native MCP server
    pub is_qollective_native: bool,
}

/// MCP transport statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTransportStats {
    /// Number of rmcp clients currently cached
    pub rmcp_clients_cached: usize,
    /// Maximum number of rmcp clients that can be cached
    pub max_rmcp_clients: usize,
    /// Whether connection pooling is enabled
    pub connection_pooling_enabled: bool,
}
