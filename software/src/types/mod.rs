// ABOUTME: Shared type definitions used across client and server modules
// ABOUTME: Eliminates circular dependencies and ensures consistent data structures

//! Shared types for the Qollective framework
//!
//! This module contains type definitions that are shared between client and server
//! components to ensure consistency and eliminate circular dependencies.

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub mod mcp;

#[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
pub mod a2a;

// Re-export commonly used types
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub use mcp::{AsyncConfig, McpData, McpDiscoveryData, McpServerInfo, ServerMetadata, SslConfig};

#[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
pub use a2a::{
    AgentId, AgentInfo, CapabilityQuery, DeregistrationRequest, Heartbeat, RegistryEvent,
};
