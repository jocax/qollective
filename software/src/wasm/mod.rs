// ABOUTME: WASM client module exports and feature gates
// ABOUTME: Provides WebAssembly support for envelope communication across protocols

//! WebAssembly client module for browser-based envelope communication.
//!
//! This module provides WASM-compatible clients for REST, WebSocket, NATS, and MCP protocols
//! while maintaining the envelope pattern and providing JavaScript interop.

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod client;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod js_types;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod error_translator;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-client",
    feature = "rest-client"
))]
pub mod rest;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-client",
    feature = "websocket-client"
))]
pub mod websocket;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod mcp_adapter;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod crypto;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-client",
    feature = "jsonrpc-client"
))]
pub mod jsonrpc;

// Re-exports for JavaScript interop
#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub use client::WasmClient;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub use js_types::{WasmContext, WasmEnvelope, WasmMeta};

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub use error_translator::ErrorTranslator;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-client",
    feature = "rest-client"
))]
pub use rest::WasmRestClient;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-client",
    feature = "websocket-client"
))]
pub use websocket::WasmWebSocketClient;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub use mcp_adapter::McpAdapter;

#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub use crypto::WasmCertificateManager;

#[cfg(all(
    target_arch = "wasm32",
    feature = "wasm-client",
    feature = "jsonrpc-client"
))]
pub use jsonrpc::{WasmJsonRpcClient, WasmJsonRpcClientJs, WasmJsonRpcConfig};

// Feature not enabled error
#[cfg(not(all(target_arch = "wasm32", feature = "wasm-client")))]
compile_error!("WASM features are only available when compiling for wasm32 target with wasm-client feature enabled");
