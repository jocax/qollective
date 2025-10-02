// ABOUTME: MCP protocol adapter for envelope-to-JSON-RPC translation
// ABOUTME: Bridges Qollective envelopes with MCP JSON-RPC protocol in WASM context

//! MCP protocol adapter for WASM applications.
//!
//! This module provides translation between Qollective envelopes and
//! MCP JSON-RPC protocol while injecting context and handling errors.

use crate::config::wasm::{McpAdapterConfig, McpErrorPolicy};
use crate::error::{QollectiveError, Result};
use crate::wasm::js_types::{WasmEnvelope, WasmMeta};
use crate::wasm::rest::ConnectivityResult;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

#[cfg(feature = "jsonrpc-client")]
use crate::wasm::jsonrpc::{WasmJsonRpcClient, WasmJsonRpcConfig};

/// MCP protocol adapter for WASM
#[derive(Debug, Clone)]
pub struct McpAdapter {
    config: McpAdapterConfig,
    request_id_counter: u64,
    #[cfg(feature = "jsonrpc-client")]
    jsonrpc_client: Option<WasmJsonRpcClient>,
}

/// MCP JSON-RPC request structure
#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

/// MCP JSON-RPC error structure
#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Tool call data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolCall {
    pub tool: String,
    pub arguments: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
}

/// Tool response data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolResponse {
    pub tool: String,
    pub result: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Enhanced context for MCP tool calls
#[derive(Debug, Serialize, Deserialize)]
struct EnhancedContext {
    /// Original context from envelope
    #[serde(skip_serializing_if = "Option::is_none")]
    original: Option<Value>,

    /// Tenant information
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant: Option<String>,

    /// User information
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,

    /// Session information
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<String>,

    /// Trace information
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<String>,

    /// Security context
    #[serde(skip_serializing_if = "Option::is_none")]
    security: Option<Value>,

    /// Custom fields
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    custom: HashMap<String, String>,

    /// Envelope metadata for tracing
    envelope_meta: Value,
}

impl McpAdapter {
    /// Create new MCP adapter
    pub fn new(config: McpAdapterConfig) -> Result<Self> {
        Ok(Self {
            config,
            request_id_counter: 0,
            #[cfg(feature = "jsonrpc-client")]
            jsonrpc_client: None,
        })
    }

    /// Create new MCP adapter with jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn new_with_jsonrpc(config: McpAdapterConfig, server_url: &str) -> Result<Self> {
        let jsonrpc_config = WasmJsonRpcConfig {
            server_url: server_url.to_string(),
            timeout_ms: config.tool_timeout_ms,
            max_concurrent_requests: config.max_concurrent_calls,
            ..Default::default()
        };

        let wasm_config = crate::config::wasm::WasmClientConfig::default();
        let jsonrpc_client = WasmJsonRpcClient::new(wasm_config, jsonrpc_config).await?;

        Ok(Self {
            config,
            request_id_counter: 0,
            jsonrpc_client: Some(jsonrpc_client),
        })
    }

    /// Call MCP tool with envelope context injection
    pub async fn call_tool(
        &self,
        server_url: &str,
        envelope: WasmEnvelope,
    ) -> Result<WasmEnvelope> {
        // Validate server URL
        if server_url.is_empty() {
            return Err(QollectiveError::validation("MCP server URL cannot be empty"));
        }

        // Extract tool call data from envelope
        let tool_call: McpToolCall = envelope.data_as::<McpToolCall>()
            .map_err(|e| QollectiveError::deserialization(format!("Invalid tool call data: {}", e)))?;

        // Extract and enhance context from envelope metadata
        let enhanced_context = self.create_enhanced_context(&envelope, tool_call.context)?;

        // Inject context into tool arguments
        let enhanced_arguments = self.inject_context_into_arguments(
            &tool_call.arguments,
            &enhanced_context,
        )?;

        // Create MCP JSON-RPC request
        let request_id = self.generate_request_id(&envelope);
        let mcp_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(request_id),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": tool_call.tool,
                "arguments": enhanced_arguments
            })),
        };

        // Send request to MCP server with retry logic
        let mcp_response = self.send_mcp_request(server_url, &mcp_request).await?;

        // Convert MCP response back to envelope
        self.convert_mcp_response_to_envelope(envelope, &tool_call, mcp_response)
    }

    /// Call MCP tool using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn call_tool_jsonrpc(
        &self,
        envelope: WasmEnvelope,
    ) -> Result<WasmEnvelope> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        // Extract tool call data from envelope
        let tool_call: McpToolCall = envelope.data_as::<McpToolCall>()
            .map_err(|e| QollectiveError::deserialization(format!("Invalid tool call data: {}", e)))?;

        // Extract and enhance context from envelope metadata
        let enhanced_context = self.create_enhanced_context(&envelope, tool_call.context)?;

        // Inject context into tool arguments
        let enhanced_arguments = self.inject_context_into_arguments(
            &tool_call.arguments,
            &enhanced_context,
        )?;

        // Call tool using jsonrpsee client
        let result = jsonrpc_client.call_tool(
            &tool_call.tool,
            enhanced_arguments,
            Some(json!(enhanced_context))
        ).await?;

        // Create tool response
        let tool_response = McpToolResponse {
            tool: tool_call.tool.clone(),
            result,
            metadata: Some(json!(enhanced_context)),
            error: None,
        };

        // Convert to envelope
        let mut response_envelope = envelope.clone();
        response_envelope.set_data(tool_response)?;

        Ok(response_envelope)
    }

    /// List available tools using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn list_tools_jsonrpc(&self) -> Result<Value> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        jsonrpc_client.list_tools().await
    }

    /// List available resources using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn list_resources_jsonrpc(&self) -> Result<Value> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        jsonrpc_client.list_resources().await
    }

    /// Read a resource using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn read_resource_jsonrpc(&self, uri: &str) -> Result<Value> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        jsonrpc_client.read_resource(uri).await
    }

    /// List available prompts using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn list_prompts_jsonrpc(&self) -> Result<Value> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        jsonrpc_client.list_prompts().await
    }

    /// Get a prompt using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn get_prompt_jsonrpc(&self, name: &str, arguments: Option<Value>) -> Result<Value> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        jsonrpc_client.get_prompt(name, arguments).await
    }

    /// Initialize MCP session using jsonrpsee client
    #[cfg(feature = "jsonrpc-client")]
    pub async fn initialize_mcp_jsonrpc(&self, client_info: Value) -> Result<Value> {
        let jsonrpc_client = self.jsonrpc_client.as_ref()
            .ok_or_else(|| QollectiveError::config("JsonRPC client not configured".to_string()))?;

        jsonrpc_client.initialize_mcp(client_info).await
    }

    /// Create enhanced context from envelope metadata
    fn create_enhanced_context(
        &self,
        envelope: &WasmEnvelope,
        original_context: Option<Value>,
    ) -> Result<EnhancedContext> {
        let meta = envelope.meta();
        let config = &self.config.context_injection;

        let mut enhanced = EnhancedContext {
            original: original_context,
            tenant: None,
            user: None,
            session: None,
            trace: None,
            security: None,
            custom: HashMap::new(),
            envelope_meta: serde_json::to_value(meta)
                .map_err(|e| QollectiveError::serialization(format!("Failed to serialize meta: {}", e)))?,
        };

        // Inject tenant information
        if config.inject_tenant {
            enhanced.tenant = meta.tenant().map(|t| t.to_string());
        }

        // Inject user information (from context if available)
        if config.inject_user {
            if let Some(context) = meta.context() {
                if let Some(user) = context.get("user") {
                    enhanced.user = user.as_str().map(|s| s.to_string());
                }
            }
        }

        // Inject session information
        if config.inject_session {
            if let Some(context) = meta.context() {
                if let Some(session) = context.get("session") {
                    enhanced.session = session.as_str().map(|s| s.to_string());
                }
            }
        }

        // Inject trace information
        if config.inject_trace {
            enhanced.trace = meta.request_id().map(|id| id.to_string());
        }

        // Inject security context
        let security_config = &config.security_context;
        if security_config.include_permissions || security_config.include_scopes || security_config.include_roles {
            let mut security_context = json!({});

            if let Some(context) = meta.context() {
                if security_config.include_permissions {
                    if let Some(permissions) = context.get("permissions") {
                        security_context["permissions"] = permissions.clone();
                    }
                }

                if security_config.include_scopes {
                    if let Some(scopes) = context.get("scopes") {
                        security_context["scopes"] = scopes.clone();
                    }
                }

                if security_config.include_roles {
                    if let Some(roles) = context.get("roles") {
                        security_context["roles"] = roles.clone();
                    }
                }
            }

            if !security_context.as_object().unwrap().is_empty() {
                enhanced.security = Some(security_context);
            }
        }

        // Add custom fields
        for (key, value) in &config.custom_fields {
            enhanced.custom.insert(key.clone(), value.clone());
        }

        Ok(enhanced)
    }

    /// Inject context into tool arguments
    fn inject_context_into_arguments(
        &self,
        arguments: &Value,
        context: &EnhancedContext,
    ) -> Result<Value> {
        let mut enhanced_args = match arguments {
            Value::Object(map) => map.clone(),
            _ => {
                // If arguments is not an object, wrap it in one
                let mut map = serde_json::Map::new();
                map.insert("data".to_string(), arguments.clone());
                map
            }
        };

        // Add context to arguments under "_qollective_context" key
        enhanced_args.insert("_qollective_context".to_string(), json!(context));

        Ok(Value::Object(enhanced_args))
    }

    /// Generate unique request ID
    fn generate_request_id(&self, envelope: &WasmEnvelope) -> String {
        if let Some(request_id) = envelope.meta().request_id() {
            format!("qol_{}", request_id)
        } else {
            format!("qol_{}", js_sys::Date::now() as u64)
        }
    }

    /// Send MCP request to server with retry logic
    async fn send_mcp_request(
        &self,
        server_url: &str,
        request: &McpRequest,
    ) -> Result<McpResponse> {
        let request_json = serde_json::to_string(request)
            .map_err(|e| QollectiveError::serialization(format!("Failed to serialize MCP request: {}", e)))?;

        let mut last_error = QollectiveError::transport("No attempts made".to_string());

        // Apply retry policy
        let (max_retries, delays) = self.calculate_retry_delays()?;

        for attempt in 0..=max_retries {
            match self.send_http_request(server_url, &request_json).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = e;

                    if attempt < max_retries {
                        let delay = delays.get(attempt).unwrap_or(&1000);
                        self.sleep_ms(*delay).await;

                        web_sys::console::warn_1(&format!(
                            "MCP request attempt {} failed, retrying in {}ms",
                            attempt + 1, delay
                        ).into());
                    }
                }
            }
        }

        Err(last_error)
    }

    /// Calculate retry delays based on error policy
    fn calculate_retry_delays(&self) -> Result<(u32, Vec<u64>)> {
        match &self.config.error_policy {
            McpErrorPolicy::FailFast => Ok((0, vec![])),
            McpErrorPolicy::BestEffort => Ok((0, vec![])),
            McpErrorPolicy::RetryExponential { max_retries, base_delay_ms, max_delay_ms } => {
                let mut delays = Vec::new();
                let mut current_delay = *base_delay_ms;

                for _ in 0..*max_retries {
                    delays.push(current_delay.min(*max_delay_ms));
                    current_delay = (current_delay * 2).min(*max_delay_ms);
                }

                Ok((*max_retries, delays))
            }
            McpErrorPolicy::RetryLinear { max_retries, delay_ms } => {
                let delays = vec![*delay_ms; *max_retries as usize];
                Ok((*max_retries, delays))
            }
        }
    }

    /// Send HTTP request to MCP server
    async fn send_http_request(
        &self,
        server_url: &str,
        request_json: &str,
    ) -> Result<McpResponse> {
        let window = web_sys::window()
            .ok_or_else(|| QollectiveError::environment("No window object available"))?;

        // Prepare request headers
        let headers = Headers::new()
            .map_err(|_| QollectiveError::transport("Failed to create headers".to_string()))?;

        headers.set("Content-Type", "application/json")
            .map_err(|_| QollectiveError::transport("Failed to set content type".to_string()))?;

        // Create request init
        let mut request_init = RequestInit::new();
        request_init.method("POST");
        request_init.headers(&headers);
        request_init.body(Some(&JsValue::from_str(request_json)));
        request_init.mode(RequestMode::Cors);

        // Create and send request
        let request = Request::new_with_str_and_init(server_url, &request_init)
            .map_err(|_| QollectiveError::transport("Failed to create request".to_string()))?;

        let response_promise = window.fetch_with_request(&request);
        let response = JsFuture::from(response_promise).await
            .map_err(|_| QollectiveError::transport("Request failed".to_string()))?;

        let response: Response = response.dyn_into()
            .map_err(|_| QollectiveError::transport("Invalid response type".to_string()))?;

        // Check response status
        if !response.ok() {
            return Err(QollectiveError::transport(format!(
                "HTTP error: {} {}",
                response.status(),
                response.status_text()
            )));
        }

        // Parse response body
        let json_promise = response.json()
            .map_err(|_| QollectiveError::transport("Failed to read response".to_string()))?;

        let json_value = JsFuture::from(json_promise).await
            .map_err(|_| QollectiveError::transport("Failed to parse JSON".to_string()))?;

        let response_text = js_sys::JSON::stringify(&json_value)
            .map_err(|_| QollectiveError::deserialization("Failed to stringify response".to_string()))?
            .as_string()
            .ok_or_else(|| QollectiveError::deserialization("Response is not a string".to_string()))?;

        serde_json::from_str::<McpResponse>(&response_text)
            .map_err(|e| QollectiveError::deserialization(format!("Failed to parse MCP response: {}", e)))
    }

    /// Sleep for specified milliseconds
    async fn sleep_ms(&self, ms: u64) {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });

            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    ms as i32,
                )
                .unwrap();
        });

        JsFuture::from(promise).await.unwrap();
    }

    /// Convert MCP response back to envelope
    fn convert_mcp_response_to_envelope(
        &self,
        original_envelope: WasmEnvelope,
        tool_call: &McpToolCall,
        mcp_response: McpResponse,
    ) -> Result<WasmEnvelope> {
        let mut response_meta = original_envelope.meta().clone();

        // Update metadata for response
        response_meta.set_timestamp(Some(chrono::Utc::now()));

        let response_data = if let Some(error) = mcp_response.error {
            // Handle MCP error
            McpToolResponse {
                tool: tool_call.tool.clone(),
                result: json!(null),
                metadata: Some(json!({
                    "mcp_error_code": error.code,
                    "mcp_error_data": error.data,
                    "request_id": mcp_response.id
                })),
                error: Some(error.message),
            }
        } else if let Some(result) = mcp_response.result {
            // Handle successful response
            McpToolResponse {
                tool: tool_call.tool.clone(),
                result,
                metadata: Some(json!({
                    "mcp_version": "2024-11-05",
                    "request_id": mcp_response.id,
                    "context_injected": true
                })),
                error: None,
            }
        } else {
            return Err(QollectiveError::deserialization(
                "MCP response has neither result nor error".to_string()
            ));
        };

        WasmEnvelope::new(response_meta, serde_wasm_bindgen::to_value(&response_data)?)
            .map_err(|e| QollectiveError::serialization(format!("Failed to create response envelope: {}", e)))
    }

    /// Test connectivity to MCP server
    pub async fn test_connectivity(&self, url: &str) -> Result<ConnectivityResult> {
        let start_time = js_sys::Date::now() as u64;

        // Create a simple ping request
        let ping_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("ping"),
            method: "ping".to_string(),
            params: None,
        };

        match self.send_http_request(url, &serde_json::to_string(&ping_request)?).await {
            Ok(_) => {
                let end_time = js_sys::Date::now() as u64;
                Ok(ConnectivityResult {
                    response_time_ms: end_time - start_time,
                    status_code: 200,
                    success: true,
                })
            }
            Err(e) => {
                let end_time = js_sys::Date::now() as u64;

                // For connectivity test, we don't fail completely - we return info about the failure
                Ok(ConnectivityResult {
                    response_time_ms: end_time - start_time,
                    status_code: 0, // Unknown status
                    success: false,
                })
            }
        }
    }

    /// List available tools from MCP server (if supported)
    pub async fn list_tools(&self, server_url: &str) -> Result<Vec<String>> {
        let list_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("list_tools"),
            method: "tools/list".to_string(),
            params: None,
        };

        match self.send_http_request(server_url, &serde_json::to_string(&list_request)?).await {
            Ok(response) => {
                if let Some(result) = response.result {
                    if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                        let tool_names: Vec<String> = tools
                            .iter()
                            .filter_map(|tool| tool.get("name").and_then(|n| n.as_str()))
                            .map(|s| s.to_string())
                            .collect();

                        return Ok(tool_names);
                    }
                }
                Ok(vec![])
            }
            Err(_) => {
                // If listing tools fails, return empty list (server might not support it)
                Ok(vec![])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::wasm::{McpAdapterConfig, ContextInjectionConfig, SecurityContextConfig};

    fn create_test_config() -> McpAdapterConfig {
        McpAdapterConfig {
            default_servers: vec!["http://localhost:3000".to_string()],
            context_injection: ContextInjectionConfig {
                inject_tenant: true,
                inject_user: true,
                inject_session: true,
                inject_trace: true,
                custom_fields: [("env".to_string(), "test".to_string())].iter().cloned().collect(),
                security_context: SecurityContextConfig {
                    include_permissions: true,
                    include_scopes: true,
                    include_roles: true,
                    sanitize_sensitive: true,
                },
            },
            tool_timeout_ms: 30000,
            max_concurrent_calls: 5,
            error_policy: McpErrorPolicy::RetryExponential {
                max_retries: 2,
                base_delay_ms: 1000,
                max_delay_ms: 5000,
            },
            cache_tools: true,
            cache_ttl_secs: 300,
        }
    }

    #[test]
    fn test_mcp_adapter_creation() {
        let config = create_test_config();
        let adapter = McpAdapter::new(config);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_context_injection() {
        let config = create_test_config();
        let adapter = McpAdapter::new(config).unwrap();

        let mut meta = WasmMeta::with_auto_fields();
        meta.set_tenant(Some("test_tenant".to_string()));

        let envelope = WasmEnvelope::new(meta, serde_wasm_bindgen::to_value(&json!({
            "test": "data"
        })).unwrap()).unwrap();

        let context = adapter.create_enhanced_context(&envelope, None);
        assert!(context.is_ok());

        let enhanced_context = context.unwrap();
        assert_eq!(enhanced_context.tenant, Some("test_tenant".to_string()));
        assert!(enhanced_context.custom.contains_key("env"));
        assert_eq!(enhanced_context.custom.get("env"), Some(&"test".to_string()));
    }

    #[test]
    fn test_request_id_generation() {
        let config = create_test_config();
        let adapter = McpAdapter::new(config).unwrap();

        let mut meta = WasmMeta::with_auto_fields();
        meta.set_request_id(Some(uuid::Uuid::now_v7()));

        let envelope = WasmEnvelope::new(meta.clone(), serde_wasm_bindgen::to_value(&json!({})).unwrap()).unwrap();

        let request_id = adapter.generate_request_id(&envelope);
        assert!(request_id.starts_with("qol_"));
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = create_test_config();
        let adapter = McpAdapter::new(config).unwrap();

        let (max_retries, delays) = adapter.calculate_retry_delays().unwrap();
        assert_eq!(max_retries, 2);
        assert_eq!(delays.len(), 2);
        assert_eq!(delays[0], 1000); // Base delay
        assert_eq!(delays[1], 2000); // Exponential backoff
    }

    #[test]
    fn test_argument_enhancement() {
        let config = create_test_config();
        let adapter = McpAdapter::new(config).unwrap();

        let arguments = json!({"param1": "value1"});
        let context = EnhancedContext {
            original: None,
            tenant: Some("test_tenant".to_string()),
            user: None,
            session: None,
            trace: None,
            security: None,
            custom: HashMap::new(),
            envelope_meta: json!({}),
        };

        let enhanced = adapter.inject_context_into_arguments(&arguments, &context).unwrap();

        assert!(enhanced.get("param1").is_some());
        assert!(enhanced.get("_qollective_context").is_some());

        let injected_context = enhanced.get("_qollective_context").unwrap();
        assert_eq!(injected_context.get("tenant").unwrap(), &json!("test_tenant"));
    }
}
