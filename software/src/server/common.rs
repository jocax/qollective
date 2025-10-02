// ABOUTME: Common server traits and utilities for protocol abstraction
// ABOUTME: Provides shared server functionality across different transport protocols

//! Common server traits and utilities for protocol abstraction.

use crate::constants::{limits, network};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: network::DEFAULT_BIND_ALL_INTERFACES.to_string(),
            port: network::DEFAULT_REST_SERVER_PORT,
            max_connections: limits::DEFAULT_GRPC_SERVER_MAX_CONNECTIONS,
        }
    }
}

/// Builder for server configuration
pub struct ServerBuilder {
    config: ServerConfig,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    pub fn bind_address(mut self, address: impl Into<String>) -> Self {
        self.config.bind_address = address.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    pub fn build(self) -> ServerConfig {
        self.config
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
