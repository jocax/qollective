// ABOUTME: Common client traits and utilities for protocol abstraction
// ABOUTME: Provides shared client functionality across different transport protocols

//! Common client traits and utilities for protocol abstraction.

use crate::constants::{circuit_breaker, network};

/// Client configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub tenant_config: TenantClientConfig,
}

/// Tenant-specific client configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TenantClientConfig {
    /// Whether to automatically propagate tenant context from current context
    pub auto_propagate_tenant: bool,
    /// Override tenant ID for all outgoing requests (overrides context-based tenant)
    pub override_tenant_id: Option<String>,
    /// Whether to propagate onBehalfOf metadata
    pub propagate_on_behalf_of: bool,
    /// Fallback tenant ID when no context is available
    pub fallback_tenant_id: Option<String>,
}

impl Default for TenantClientConfig {
    fn default() -> Self {
        Self {
            auto_propagate_tenant: true,
            override_tenant_id: None,
            propagate_on_behalf_of: true,
            fallback_tenant_id: None,
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: {
                #[cfg(any(feature = "rest-server", feature = "rest-client"))]
                let port = network::DEFAULT_REST_SERVER_PORT;
                #[cfg(not(any(feature = "rest-server", feature = "rest-client")))]
                let port = 8080;
                
                format!("http://{}:{}", network::DEFAULT_LOCALHOST, port)
            },
            timeout_seconds: 30,
            retry_attempts: circuit_breaker::DEFAULT_MAX_RETRIES,
            tenant_config: TenantClientConfig::default(),
        }
    }
}

/// Builder for client configuration
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }

    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.config.retry_attempts = attempts;
        self
    }

    pub fn tenant_config(mut self, tenant_config: TenantClientConfig) -> Self {
        self.config.tenant_config = tenant_config;
        self
    }

    pub fn auto_propagate_tenant(mut self, enabled: bool) -> Self {
        self.config.tenant_config.auto_propagate_tenant = enabled;
        self
    }

    pub fn override_tenant_id(mut self, tenant_id: Option<String>) -> Self {
        self.config.tenant_config.override_tenant_id = tenant_id;
        self
    }

    pub fn propagate_on_behalf_of(mut self, enabled: bool) -> Self {
        self.config.tenant_config.propagate_on_behalf_of = enabled;
        self
    }

    pub fn fallback_tenant_id(mut self, tenant_id: Option<String>) -> Self {
        self.config.tenant_config.fallback_tenant_id = tenant_id;
        self
    }

    pub fn build(self) -> ClientConfig {
        self.config
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
