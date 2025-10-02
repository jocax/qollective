// ABOUTME: Client implementations for different transport protocols
// ABOUTME: Provides HTTP and gRPC client functionality with envelope support

//! Client implementations for different transport protocols.
//!
//! This module provides clients for various protocols that automatically
//! handle envelope construction, metadata propagation, and response parsing.

#[cfg(feature = "rest-client")]
pub mod rest;

#[cfg(feature = "grpc-client")]
pub mod grpc;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub mod nats;

#[cfg(feature = "a2a-client")]
pub mod a2a;

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub mod mcp;

#[cfg(feature = "websocket-client")]
pub mod websocket;


// Common client traits and utilities
pub mod common;

// Middleware utilities for context propagation
pub mod middleware;

pub use common::{ClientBuilder, ClientConfig, TenantClientConfig};

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use nats::{ConnectionEvent, ConnectionMetrics, ConnectionState, NatsClient};

#[cfg(feature = "a2a-client")]
pub use a2a::{
    A2AClient, A2AMessageType, A2AMetadata, AgentMetadata, AgentProvider, AgentRegistry,
};

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub use mcp::{
    ChainResult, McpClient, McpMetadata, McpOperationType, ServerInfo, ToolCall, ToolChainRequest,
    ToolInfo, ToolListQuery, ToolResult,
};
