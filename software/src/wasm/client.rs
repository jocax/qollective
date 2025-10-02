// ABOUTME: Unified WASM client for all protocol communication
// ABOUTME: Provides single API for REST, WebSocket, NATS, and MCP envelope communication in browsers

//! Unified WASM client for browser-based envelope communication.
//!
//! This module provides a single client interface that can communicate
//! over multiple protocols while maintaining the envelope pattern.

use crate::config::wasm::WasmClientConfig;
use crate::error::{QollectiveError, Result};
use crate::wasm::error_translator::ErrorTranslator;
use crate::wasm::js_types::{WasmEnvelope, WasmEnvelopeError, WasmMeta};
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

#[cfg(feature = "rest-client")]
use crate::wasm::rest::WasmRestClient;

#[cfg(feature = "websocket-client")]
use crate::wasm::websocket::WasmWebSocketClient;

use crate::wasm::crypto::WasmCertificateManager;
use crate::wasm::mcp_adapter::McpAdapter;

/// Unified WASM client for all protocol communication
#[wasm_bindgen]
pub struct WasmClient {
    config: WasmClientConfig,

    #[cfg(feature = "rest-client")]
    rest_client: Option<WasmRestClient>,

    #[cfg(feature = "websocket-client")]
    websocket_client: Option<WasmWebSocketClient>,

    mcp_adapter: Option<McpAdapter>,
    cert_manager: WasmCertificateManager,
}

#[wasm_bindgen]
impl WasmClient {
    /// Create a new WASM client with configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmClient, JsValue> {
        // Enable better panic messages in debug mode
        #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

        let config: WasmClientConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid configuration: {}", e)))?;

        let cert_manager =
            WasmCertificateManager::new(&config.certificate_config).map_err(|e| {
                JsValue::from_str(&format!("Failed to initialize certificate manager: {}", e))
            })?;

        Ok(WasmClient {
            config: config.clone(),

            #[cfg(feature = "rest-client")]
            rest_client: if config.rest_enabled {
                Some(
                    WasmRestClient::new(config.rest_config.clone(), cert_manager.clone()).map_err(
                        |e| JsValue::from_str(&format!("Failed to initialize REST client: {}", e)),
                    )?,
                )
            } else {
                None
            },

            #[cfg(feature = "websocket-client")]
            websocket_client: if config.websocket_enabled {
                Some(
                    WasmWebSocketClient::new(config.websocket_config.clone(), cert_manager.clone())
                        .map_err(|e| {
                            JsValue::from_str(&format!(
                                "Failed to initialize WebSocket client: {}",
                                e
                            ))
                        })?,
                )
            } else {
                None
            },

            mcp_adapter: if config.mcp_enabled {
                Some(McpAdapter::new(config.mcp_config.clone()).map_err(|e| {
                    JsValue::from_str(&format!("Failed to initialize MCP adapter: {}", e))
                })?)
            } else {
                None
            },

            cert_manager,
        })
    }

    /// Send envelope via REST HTTP/HTTPS
    #[cfg(feature = "rest-client")]
    #[wasm_bindgen]
    pub async fn send_rest_envelope(
        &self,
        url: &str,
        envelope: JsValue,
    ) -> Result<JsValue, JsValue> {
        let rest_client = self
            .rest_client
            .as_ref()
            .ok_or_else(|| JsValue::from_str("REST client not enabled"))?;

        let envelope: WasmEnvelope = serde_wasm_bindgen::from_value(envelope)
            .map_err(|e| JsValue::from_str(&format!("Invalid envelope: {}", e)))?;

        // Auto-populate metadata if missing
        let mut envelope = envelope;
        if envelope.meta().request_id().is_none() {
            envelope.set_meta(WasmMeta::with_auto_fields());
        }

        let response = rest_client
            .send_envelope(url, envelope)
            .await
            .map_err(|e| JsValue::from_str(&ErrorTranslator::translate_for_user(&e)))?;

        serde_wasm_bindgen::to_value(&response)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
    }

    /// Send envelope via WebSocket
    #[cfg(feature = "websocket-client")]
    #[wasm_bindgen]
    pub async fn send_websocket_envelope(
        &self,
        url: &str,
        envelope: JsValue,
    ) -> Result<JsValue, JsValue> {
        let ws_client = self
            .websocket_client
            .as_ref()
            .ok_or_else(|| JsValue::from_str("WebSocket client not enabled"))?;

        let envelope: WasmEnvelope = serde_wasm_bindgen::from_value(envelope)
            .map_err(|e| JsValue::from_str(&format!("Invalid envelope: {}", e)))?;

        // Auto-populate metadata if missing
        let mut envelope = envelope;
        if envelope.meta().request_id().is_none() {
            envelope.set_meta(WasmMeta::with_auto_fields());
        }

        let response = ws_client
            .send_envelope(url, envelope)
            .await
            .map_err(|e| JsValue::from_str(&ErrorTranslator::translate_for_user(&e)))?;

        serde_wasm_bindgen::to_value(&response)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
    }

    /// Call MCP tool with envelope context injection
    #[wasm_bindgen]
    pub async fn call_mcp_tool(
        &self,
        server_url: &str,
        tool_name: &str,
        arguments: JsValue,
        context: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        let mcp_adapter = self
            .mcp_adapter
            .as_ref()
            .ok_or_else(|| JsValue::from_str("MCP adapter not enabled"))?;

        // Convert arguments to JSON value
        let arguments: serde_json::Value = serde_wasm_bindgen::from_value(arguments)
            .map_err(|e| JsValue::from_str(&format!("Invalid arguments: {}", e)))?;

        // Convert context if provided
        let context: Option<serde_json::Value> = if let Some(ctx) = context {
            Some(
                serde_wasm_bindgen::from_value(ctx)
                    .map_err(|e| JsValue::from_str(&format!("Invalid context: {}", e)))?,
            )
        } else {
            None
        };

        // Create envelope for MCP tool call
        let meta = WasmMeta::with_auto_fields();
        let tool_call_data = serde_json::json!({
            "tool": tool_name,
            "arguments": arguments,
            "context": context
        });

        let envelope = WasmEnvelope::new(meta, serde_wasm_bindgen::to_value(&tool_call_data)?)
            .map_err(|e| JsValue::from_str(&format!("Failed to create envelope: {}", e)))?;

        let response = mcp_adapter
            .call_tool(server_url, envelope)
            .await
            .map_err(|e| JsValue::from_str(&ErrorTranslator::translate_for_user(&e)))?;

        serde_wasm_bindgen::to_value(&response)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
    }

    /// Create envelope with auto-populated metadata
    #[wasm_bindgen]
    pub fn create_envelope(&self, data: JsValue) -> Result<JsValue, JsValue> {
        let meta = WasmMeta::with_auto_fields();
        let envelope = WasmEnvelope::new(meta, data)?;

        serde_wasm_bindgen::to_value(&envelope)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))
    }

    /// Create envelope with custom metadata
    #[wasm_bindgen]
    pub fn create_envelope_with_meta(
        &self,
        meta: JsValue,
        data: JsValue,
    ) -> Result<JsValue, JsValue> {
        let meta: WasmMeta = serde_wasm_bindgen::from_value(meta)
            .map_err(|e| JsValue::from_str(&format!("Invalid metadata: {}", e)))?;

        let envelope = WasmEnvelope::new(meta, data)?;

        serde_wasm_bindgen::to_value(&envelope)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))
    }

    /// Get client configuration
    #[wasm_bindgen]
    pub fn get_config(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.config)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize config: {}", e)))
    }

    /// Check if REST is enabled
    #[cfg(feature = "rest-client")]
    #[wasm_bindgen]
    pub fn is_rest_enabled(&self) -> bool {
        self.rest_client.is_some()
    }

    /// Check if WebSocket is enabled
    #[cfg(feature = "websocket-client")]
    #[wasm_bindgen]
    pub fn is_websocket_enabled(&self) -> bool {
        self.websocket_client.is_some()
    }

    /// Check if MCP is enabled
    #[wasm_bindgen]
    pub fn is_mcp_enabled(&self) -> bool {
        self.mcp_adapter.is_some()
    }

    /// Get client capabilities
    #[wasm_bindgen]
    pub fn get_capabilities(&self) -> JsValue {
        let capabilities = serde_json::json!({
            "rest": {
                "enabled": self.config.rest_enabled,
                "https_supported": true,
                "mtls_supported": true
            },
            "websocket": {
                "enabled": self.config.websocket_enabled,
                "wss_supported": true,
                "auto_reconnect": true
            },
            "mcp": {
                "enabled": self.config.mcp_enabled,
                "context_injection": true,
                "error_translation": true
            },
            "envelope": {
                "auto_metadata": true,
                "context_propagation": true,
                "error_handling": true
            }
        });

        serde_wasm_bindgen::to_value(&capabilities).unwrap_or(JsValue::NULL)
    }

    /// Test connectivity to a given endpoint
    #[wasm_bindgen]
    pub async fn test_connectivity(&self, url: &str, protocol: &str) -> Result<JsValue, JsValue> {
        let result = match protocol.to_lowercase().as_str() {
            #[cfg(feature = "rest-client")]
            "rest" | "http" | "https" => {
                if let Some(rest_client) = &self.rest_client {
                    rest_client.test_connectivity(url).await
                } else {
                    Err(QollectiveError::feature_not_enabled(
                        "REST client not enabled",
                    ))
                }
            }
            #[cfg(feature = "websocket-client")]
            "websocket" | "ws" | "wss" => {
                if let Some(ws_client) = &self.websocket_client {
                    ws_client.test_connectivity(url).await
                } else {
                    Err(QollectiveError::feature_not_enabled(
                        "WebSocket client not enabled",
                    ))
                }
            }
            "mcp" => {
                if let Some(mcp_adapter) = &self.mcp_adapter {
                    mcp_adapter.test_connectivity(url).await
                } else {
                    Err(QollectiveError::feature_not_enabled(
                        "MCP adapter not enabled",
                    ))
                }
            }
            _ => Err(QollectiveError::validation("Unsupported protocol")),
        };

        match result {
            Ok(response) => {
                let success_response = serde_json::json!({
                    "success": true,
                    "protocol": protocol,
                    "url": url,
                    "response_time_ms": response.response_time_ms,
                    "details": response
                });
                serde_wasm_bindgen::to_value(&success_response)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize response: {}", e)))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "success": false,
                    "protocol": protocol,
                    "url": url,
                    "error": ErrorTranslator::translate_for_user(&e)
                });
                serde_wasm_bindgen::to_value(&error_response).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize error response: {}", e))
                })
            }
        }
    }

    /// Validate envelope structure
    #[wasm_bindgen]
    pub fn validate_envelope(&self, envelope: JsValue) -> Result<JsValue, JsValue> {
        let envelope: WasmEnvelope = serde_wasm_bindgen::from_value(envelope)
            .map_err(|e| JsValue::from_str(&format!("Invalid envelope structure: {}", e)))?;

        let mut validation_result = serde_json::json!({
            "valid": true,
            "errors": [],
            "warnings": []
        });

        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // Validate metadata
        let meta = envelope.meta();
        if meta.request_id().is_none() {
            warnings.push("Missing request_id - will be auto-generated".to_string());
        }
        if meta.timestamp().is_none() {
            warnings.push("Missing timestamp - will be auto-generated".to_string());
        }
        if meta.tenant().is_none() {
            warnings.push("Missing tenant information".to_string());
        }

        // Validate data
        if envelope.data().is_err() {
            errors.push("Invalid data section".to_string());
        }

        // Check for errors
        if envelope.has_error() && envelope.data().is_ok() {
            warnings.push("Envelope contains both data and error".to_string());
        }

        validation_result["valid"] = serde_json::Value::Bool(errors.is_empty());
        validation_result["errors"] =
            serde_json::Value::Array(errors.into_iter().map(serde_json::Value::String).collect());
        validation_result["warnings"] = serde_json::Value::Array(
            warnings
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        );

        serde_wasm_bindgen::to_value(&validation_result).map_err(|e| {
            JsValue::from_str(&format!("Failed to serialize validation result: {}", e))
        })
    }
}

// Free functions for JavaScript convenience
#[wasm_bindgen]
pub fn create_meta() -> WasmMeta {
    WasmMeta::with_auto_fields()
}

#[wasm_bindgen]
pub fn create_meta_with_tenant(tenant: &str) -> WasmMeta {
    WasmMeta::with_auto_fields().with_tenant(tenant)
}

#[wasm_bindgen]
pub fn get_qollective_version() -> String {
    use crate::constants::metadata;
    metadata::QOLLECTIVE_VERSION.to_string()
}
