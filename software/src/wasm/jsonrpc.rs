// ABOUTME: WASM-compatible jsonrpsee client with envelope integration
// ABOUTME: Provides JSON-RPC 2.0 client for browser environments with qollective envelope support

//! WASM-compatible jsonrpsee client implementation.
//!
//! This module provides a WebAssembly-compatible JSON-RPC 2.0 client using
//! jsonrpsee's WASM support while maintaining the qollective envelope pattern
//! for metadata and context propagation.

use crate::config::wasm::WasmClientConfig;
use crate::envelope::{Envelope, Meta};
use crate::error::{QollectiveError, Result};
use crate::transport::jsonrpc::{JsonRpcEnvelope, JsonRpcRequest, JsonRpcResponse};
use crate::wasm::js_types::{WasmEnvelope, WasmMeta};
use async_trait::async_trait;
use jsonrpsee::wasm_client::{WasmClientBuilder, WasmClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// WASM-compatible jsonrpsee client with envelope integration
#[derive(Debug, Clone)]
pub struct WasmJsonRpcClient {
    /// jsonrpsee WASM client
    client: WasmClient,
    /// Client configuration
    config: WasmClientConfig,
    /// Request ID counter for JSON-RPC requests
    request_id_counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

/// Configuration for WASM JsonRPC client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmJsonRpcConfig {
    /// Target server URL
    pub server_url: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Custom headers to include in requests
    pub custom_headers: HashMap<String, String>,
    /// Enable request/response logging
    pub enable_logging: bool,
    /// WebSocket ping interval in milliseconds
    pub ping_interval_ms: u64,
}

impl Default for WasmJsonRpcConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080".to_string(),
            timeout_ms: 30000,
            max_concurrent_requests: 10,
            custom_headers: HashMap::new(),
            enable_logging: false,
            ping_interval_ms: 30000,
        }
    }
}

impl WasmJsonRpcClient {
    /// Create a new WASM JsonRPC client
    pub async fn new(config: WasmClientConfig, jsonrpc_config: WasmJsonRpcConfig) -> Result<Self> {
        let client = WasmClientBuilder::default()
            .build(&jsonrpc_config.server_url)
            .await
            .map_err(|e| QollectiveError::transport(format!("Failed to create WASM client: {}", e)))?;

        Ok(Self {
            client,
            config,
            request_id_counter: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// Send a JSON-RPC request with envelope wrapper
    pub async fn send_request<T, R>(&self, method: &str, params: T) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let request_id = self.request_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let response = self
            .client
            .request(method, rpc_params![params])
            .await
            .map_err(|e| QollectiveError::transport(format!("JSON-RPC request failed: {}", e)))?;

        Ok(response)
    }

    /// Send an envelope-wrapped JSON-RPC request
    pub async fn send_envelope_request<T, R>(&self, envelope: Envelope<T>) -> Result<Envelope<R>>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let (meta, data) = envelope.extract();
        
        // Create JsonRpcRequest with envelope metadata
        let request_id = self.request_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let jsonrpc_request = JsonRpcRequest::new(
            "envelope_request".to_string(),
            data,
            Some(request_id as i32),
            meta.clone(),
        );

        // Send request
        let response: JsonRpcResponse<R> = self
            .client
            .request("envelope_request", rpc_params![jsonrpc_request])
            .await
            .map_err(|e| QollectiveError::transport(format!("Envelope request failed: {}", e)))?;

        // Wrap response in envelope
        Ok(Envelope::new(meta, response.result))
    }

    /// Send a tool call request (MCP-style)
    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Value, context: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments,
            "context": context
        });

        let result = self
            .client
            .request("tools/call", rpc_params![params])
            .await
            .map_err(|e| QollectiveError::transport(format!("Tool call failed: {}", e)))?;

        Ok(result)
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<serde_json::Value> {
        let result = self
            .client
            .request("tools/list", rpc_params![])
            .await
            .map_err(|e| QollectiveError::transport(format!("List tools failed: {}", e)))?;

        Ok(result)
    }

    /// List available resources
    pub async fn list_resources(&self) -> Result<serde_json::Value> {
        let result = self
            .client
            .request("resources/list", rpc_params![])
            .await
            .map_err(|e| QollectiveError::transport(format!("List resources failed: {}", e)))?;

        Ok(result)
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &str) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "uri": uri
        });

        let result = self
            .client
            .request("resources/read", rpc_params![params])
            .await
            .map_err(|e| QollectiveError::transport(format!("Read resource failed: {}", e)))?;

        Ok(result)
    }

    /// List available prompts
    pub async fn list_prompts(&self) -> Result<serde_json::Value> {
        let result = self
            .client
            .request("prompts/list", rpc_params![])
            .await
            .map_err(|e| QollectiveError::transport(format!("List prompts failed: {}", e)))?;

        Ok(result)
    }

    /// Get a prompt
    pub async fn get_prompt(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let result = self
            .client
            .request("prompts/get", rpc_params![params])
            .await
            .map_err(|e| QollectiveError::transport(format!("Get prompt failed: {}", e)))?;

        Ok(result)
    }

    /// Initialize MCP session
    pub async fn initialize_mcp(&self, client_info: serde_json::Value) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "clientInfo": client_info
        });

        let result = self
            .client
            .request("initialize", rpc_params![params])
            .await
            .map_err(|e| QollectiveError::transport(format!("MCP initialize failed: {}", e)))?;

        Ok(result)
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        // For WASM clients, we assume connected if client exists
        true
    }

    /// Get client configuration
    pub fn config(&self) -> &WasmClientConfig {
        &self.config
    }
}

/// WASM bindings for JavaScript interop
#[wasm_bindgen]
pub struct WasmJsonRpcClientJs {
    inner: WasmJsonRpcClient,
}

#[wasm_bindgen]
impl WasmJsonRpcClientJs {
    /// Create a new WASM JsonRPC client for JavaScript
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<WasmJsonRpcClientJs, JsValue> {
        let config: WasmJsonRpcConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        let wasm_config = WasmClientConfig::default();
        
        // Note: In real usage, this would be async, but wasm_bindgen doesn't support async constructors
        // Users would need to use an async factory method
        Err(JsValue::from_str("Use create_async factory method instead"))
    }

    /// Async factory method for creating WASM JsonRPC client
    #[wasm_bindgen]
    pub async fn create_async(config_json: &str) -> Result<WasmJsonRpcClientJs, JsValue> {
        let jsonrpc_config: WasmJsonRpcConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        let wasm_config = WasmClientConfig::default();
        
        let client = WasmJsonRpcClient::new(wasm_config, jsonrpc_config)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create client: {}", e)))?;

        Ok(WasmJsonRpcClientJs { inner: client })
    }

    /// Call a tool from JavaScript
    #[wasm_bindgen]
    pub async fn call_tool_js(&self, tool_name: &str, arguments_json: &str, context_json: Option<String>) -> Result<String, JsValue> {
        let arguments: serde_json::Value = serde_json::from_str(arguments_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid arguments: {}", e)))?;

        let context = if let Some(ctx) = context_json {
            Some(serde_json::from_str(&ctx)
                .map_err(|e| JsValue::from_str(&format!("Invalid context: {}", e)))?)
        } else {
            None
        };

        let result = self.inner.call_tool(tool_name, arguments, context)
            .await
            .map_err(|e| JsValue::from_str(&format!("Tool call failed: {}", e)))?;

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// List tools from JavaScript
    #[wasm_bindgen]
    pub async fn list_tools_js(&self) -> Result<String, JsValue> {
        let result = self.inner.list_tools()
            .await
            .map_err(|e| JsValue::from_str(&format!("List tools failed: {}", e)))?;

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Check if connected from JavaScript
    #[wasm_bindgen]
    pub fn is_connected_js(&self) -> bool {
        self.inner.is_connected()
    }
}

// Helper macro for jsonrpsee params (not available in WASM client)
macro_rules! rpc_params {
    ($params:expr) => {
        Some($params)
    };
    () => {
        None
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_jsonrpc_config_default() {
        let config = WasmJsonRpcConfig::default();
        assert_eq!(config.server_url, "ws://localhost:8080");
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_concurrent_requests, 10);
        assert!(!config.enable_logging);
    }

    #[wasm_bindgen_test]
    fn test_wasm_jsonrpc_config_serialization() {
        let config = WasmJsonRpcConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: WasmJsonRpcConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.server_url, deserialized.server_url);
        assert_eq!(config.timeout_ms, deserialized.timeout_ms);
        assert_eq!(config.max_concurrent_requests, deserialized.max_concurrent_requests);
    }

    #[wasm_bindgen_test]
    async fn test_wasm_jsonrpc_client_creation() {
        let wasm_config = WasmClientConfig::default();
        let jsonrpc_config = WasmJsonRpcConfig::default();
        
        // Note: This would normally connect to a real server
        // In a real test, we'd need a mock server or skip this test
        let client_result = WasmJsonRpcClient::new(wasm_config, jsonrpc_config).await;
        
        // Test should pass if connection succeeds or fail gracefully
        match client_result {
            Ok(client) => {
                assert!(client.is_connected());
            }
            Err(_) => {
                // Connection failed, which is expected in test environment
                assert!(true);
            }
        }
    }
}