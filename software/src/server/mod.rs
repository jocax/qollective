// ABOUTME: Server implementations for different transport protocols
// ABOUTME: Provides HTTP and gRPC server functionality with envelope support

//! Server implementations for different transport protocols.
//!
//! This module provides server utilities for various protocols that automatically
//! handle envelope parsing, metadata extraction, and response building.

#[cfg(feature = "rest-server")]
pub mod rest;

#[cfg(feature = "grpc-server")]
pub mod grpc;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub mod nats;

#[cfg(feature = "a2a-server")]
pub mod a2a;

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub mod mcp;

#[cfg(feature = "mcp-jsonrpc")]
pub mod mcp_jsonrpc;

#[cfg(feature = "websocket-server")]
pub mod websocket;

// Common server traits and utilities
pub mod common;

// Middleware utilities for context propagation
pub mod middleware;

pub use common::{ServerBuilder, ServerConfig};

#[cfg(feature = "rest-server")]
pub use rest::{
    CorsConfig, MetadataEncoding, MetadataHandlingConfig, RestServer, RestServerConfig,
};

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use nats::NatsServer;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use crate::traits::handlers::EnvelopeHandler;

#[cfg(feature = "a2a-server")]
pub use a2a::A2AServer;

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
pub use mcp::{McpPrompt, McpPromptArgument, McpResource, McpServer, McpServerConfig};

#[cfg(feature = "mcp-jsonrpc")]
pub use mcp_jsonrpc::{McpJsonRpcServer, McpJsonRpcServerConfig};

#[cfg(feature = "websocket-server")]
pub use websocket::{WebSocketServer, WebSocketServerConfig};
