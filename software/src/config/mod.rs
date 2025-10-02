// ABOUTME: Configuration management for metadata inclusion and framework behavior
// ABOUTME: Provides preset configurations and granular control over metadata sections

//! Configuration management for the Qollective framework.
//!
//! This module provides configuration utilities for controlling metadata
//! inclusion, transport options, and framework behavior with support for
//! global, endpoint, and request-level configuration scopes.

pub mod loader;
pub mod masking;
pub mod meta;
pub mod presets;
pub mod rest;
pub mod tls;
pub mod transport;
pub mod validator;

#[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
pub mod grpc;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub mod nats;

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub mod mcp;

#[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
pub mod a2a;

#[cfg(feature = "websocket-client")]
pub mod websocket;


#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod wasm;

pub use masking::{
    FieldMasker, MaskType, Maskable, MaskingConfig, MaskingError, MaskingLevel, MaskingRule,
};
pub use meta::{MetaConfig, MetaSectionConfig, PropertyConfig};
pub use presets::{
    ConfigPreset, CorsConfig, LoggingConfig, PerformanceConfig, QollectiveConfig,
    QollectiveConfigBuilder, RestClientConfig, RestConfig, RestServerConfig, TenantClientConfig,
};

#[cfg(feature = "tls")]
pub use tls::{TlsConfig, TlsConfigBuilder, VerificationMode};

#[cfg(feature = "tenant-extraction")]
pub use crate::tenant::extraction::ExtractionConfig;
pub use loader::{ConfigLoader, ConfigSource, EnvironmentMapper};
pub use rest::{HeaderManager, PerformanceBenchmark, RequestLogger, RestConfigBuilder, UrlManager};
pub use transport::{
    EndpointTransportConfig, GlobalTransportConfig, ProtocolConfigs, RetryConfig, TransportConfig,
    TransportConfigBuilder, TransportDetectionConfig,
};
pub use validator::{
    ConfigValidator, ValidationError, ValidationErrorType, ValidationResult, ValidationWarning,
};

#[cfg(feature = "websocket-client")]
pub use websocket::WebSocketConfig;


#[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
pub use grpc::{
    ConcurrencyConfig, ConnectionPoolConfig, GrpcClientConfig, GrpcServerConfig, HealthCheckConfig,
    ReflectionConfig,
};

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use nats::{
    NatsClientConfig, NatsConfig, NatsConnectionConfig, NatsDiscoveryConfig, NatsServerConfig,
};

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub use mcp::{
    DistributedExecutionConfig, LoadBalancingStrategy, McpBaseServerConfig, McpClientConfig,
    McpClientConfigBuilder, McpPromptArgumentConfig, McpPromptConfig, McpResourceConfig,
    McpServerConfig, McpServerEndpoint, McpServerInfo, McpServerRegistryConfig, McpTlsConfig,
    McpToolConfig, McpTransportClientConfig, McpTransportConfig, McpTransportStats,
    ToolChainExecutorConfig, TransportRetryConfig, TransportTimeoutConfig,
};

#[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
pub use a2a::{
    A2AClientConfig, A2AServerConfig, A2ASubjectConfig, AgentClientConfig, AgentDiscoveryConfig,
    AgentTransportConfig, CircuitBreakerConfig, HealthConfig,
    LoadBalancingStrategy as A2ALoadBalancingStrategy, QueueGroupConfig, RegistryConfig,
    RetryConfig as A2ARetryConfig, RoutingConfig,
};
