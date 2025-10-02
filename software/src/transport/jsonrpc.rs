// ABOUTME: Common jsonrpsee utilities for envelope integration and JSON-RPC 2.0 support
// ABOUTME: Provides wrapper types and middleware for seamless envelope pattern integration

//! Common jsonrpsee utilities for envelope integration.
//!
//! This module provides wrapper types and middleware for integrating jsonrpsee
//! JSON-RPC 2.0 clients and servers with the Qollective envelope pattern.
//! It preserves envelope metadata propagation while leveraging
//! jsonrpsee's production-ready JSON-RPC implementation.

use crate::envelope::{Envelope, EnvelopeBuilder, Meta};
use crate::error::{QollectiveError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// JSON-RPC envelope wrapper for transparent envelope integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcEnvelope<T> {
    /// Envelope metadata
    pub meta: Meta,
    /// Actual JSON-RPC payload
    pub payload: T,
}

impl<T> JsonRpcEnvelope<T> {
    /// Create a new JsonRpcEnvelope from an existing envelope
    pub fn from_envelope(envelope: Envelope<T>) -> Self {
        Self {
            meta: envelope.meta,
            payload: envelope.payload,
        }
    }

    /// Convert back to a standard envelope
    pub fn into_envelope(self) -> Result<Envelope<T>> {
        EnvelopeBuilder::new()
            .with_payload(self.payload)
            .with_meta(self.meta)
            .build()
    }
}

/// JSON-RPC error wrapper that preserves envelope context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcEnvelopeError {
    /// JSON-RPC error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional error data
    pub data: Option<Value>,
    /// Preserved envelope metadata
    pub meta: Option<Meta>,
}

impl JsonRpcEnvelopeError {
    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: String, meta: Option<Meta>) -> Self {
        Self {
            code,
            message,
            data: None,
            meta,
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str, meta: Option<Meta>) -> Self {
        Self::new(
            -32601,
            format!("Method not found: {}", method),
            meta,
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(message: &str, meta: Option<Meta>) -> Self {
        Self::new(
            -32602,
            format!("Invalid params: {}", message),
            meta,
        )
    }

    /// Create an internal error
    pub fn internal_error(message: &str, meta: Option<Meta>) -> Self {
        Self::new(
            -32603,
            format!("Internal error: {}", message),
            meta,
        )
    }

    /// Create a parse error
    pub fn parse_error(message: &str, meta: Option<Meta>) -> Self {
        Self::new(
            -32700,
            format!("Parse error: {}", message),
            meta,
        )
    }

    /// Create an invalid request error
    pub fn invalid_request(message: &str, meta: Option<Meta>) -> Self {
        Self::new(
            -32600,
            format!("Invalid request: {}", message),
            meta,
        )
    }
}

impl fmt::Display for JsonRpcEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error [{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcEnvelopeError {}

/// JSON-RPC request wrapper with envelope context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    /// JSON-RPC method name
    pub method: String,
    /// Request parameters
    pub params: T,
    /// Request ID
    pub id: Option<u64>,
    /// Envelope metadata
    pub meta: Meta,
}

impl<T> JsonRpcRequest<T> {
    /// Create from envelope and method
    pub fn from_envelope(envelope: Envelope<T>, method: String, id: Option<u64>) -> Self {
        Self {
            method,
            params: envelope.payload,
            id,
            meta: envelope.meta,
        }
    }

    /// Create a new JSON-RPC request
    pub fn new(method: String, params: T, id: Option<u64>, meta: Meta) -> Self {
        Self {
            method,
            params,
            id,
            meta,
        }
    }
}

/// JSON-RPC response wrapper with envelope context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    /// Response result
    pub result: Option<T>,
    /// Response error
    pub error: Option<JsonRpcEnvelopeError>,
    /// Request ID
    pub id: u64,
    /// Envelope metadata
    pub meta: Meta,
}

impl<T> JsonRpcResponse<T> {
    /// Create success response
    pub fn success(result: T, id: u64, meta: Meta) -> Self {
        Self {
            result: Some(result),
            error: None,
            id,
            meta,
        }
    }

    /// Create error response
    pub fn error(error: JsonRpcEnvelopeError, id: u64) -> Self {
        let meta = error.meta.clone().unwrap_or_default();
        
        Self {
            result: None,
            error: Some(error),
            id,
            meta,
        }
    }

    /// Convert to envelope
    pub fn into_envelope(self) -> Result<Envelope<std::result::Result<T, JsonRpcEnvelopeError>>> {
        let payload = match (self.result, self.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) => Err(error),
            (Some(_), Some(_)) => Err(JsonRpcEnvelopeError::internal_error(
                "Invalid response: both result and error present",
                None,
            )),
            (None, None) => Err(JsonRpcEnvelopeError::internal_error(
                "Invalid response: neither result nor error present",
                None,
            )),
        };

        EnvelopeBuilder::new()
            .with_payload(payload)
            .with_meta(self.meta)
            .build()
    }
}

/// Envelope-aware JSON-RPC method handler trait
#[async_trait::async_trait]
pub trait EnvelopeJsonRpcHandler<T, R>: Send + Sync {
    /// Handle JSON-RPC method call with envelope context
    async fn handle_method(
        &self,
        method: &str,
        request: JsonRpcRequest<T>,
    ) -> Result<JsonRpcResponse<R>>;
}

/// Utility functions for envelope integration
pub mod utils {
    use super::*;
    use crate::envelope::meta::{SecurityMeta, TracingMeta};

    /// Extract envelope metadata from JSON-RPC request headers
    pub fn extract_envelope_metadata(headers: &HashMap<String, String>) -> Meta {
        let mut meta = Meta::default();

        // Extract tracing context from headers
        if let Some(trace_id) = headers.get("x-trace-id") {
            let tracing_meta = TracingMeta {
                trace_id: Some(trace_id.clone()),
                span_id: headers.get("x-span-id").cloned(),
                parent_span_id: None,
                baggage: HashMap::new(),
                operation_name: None,
                sampling_rate: None,
                sampled: None,
                trace_state: None,
                span_kind: None,
                span_status: None,
                tags: HashMap::new(),
            };
            meta.tracing = Some(tracing_meta);
        }

        // Extract tenant information
        if let Some(tenant_id) = headers.get("x-tenant-id") {
            meta.tenant = Some(tenant_id.clone());
        }

        // Extract user information
        if let Some(user_id) = headers.get("x-user-id") {
            let security_meta = SecurityMeta {
                user_id: Some(user_id.clone()),
                session_id: headers.get("x-session-id").cloned(),
                auth_method: None,
                permissions: Vec::new(),
                ip_address: headers.get("x-ip-address").cloned(),
                user_agent: headers.get("x-user-agent").cloned(),
                roles: Vec::new(),
                token_expires_at: None,
            };
            meta.security = Some(security_meta);
        }

        meta
    }

    /// Inject envelope metadata into JSON-RPC request headers
    pub fn inject_envelope_metadata(meta: &Meta) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // Inject tracing context
        if let Some(ref tracing_meta) = meta.tracing {
            if let Some(ref trace_id) = tracing_meta.trace_id {
                headers.insert("x-trace-id".to_string(), trace_id.clone());
            }
            if let Some(ref span_id) = tracing_meta.span_id {
                headers.insert("x-span-id".to_string(), span_id.clone());
            }
        }

        // Inject tenant information
        if let Some(ref tenant_id) = meta.tenant {
            headers.insert("x-tenant-id".to_string(), tenant_id.clone());
        }

        // Inject user information
        if let Some(ref security_meta) = meta.security {
            if let Some(ref user_id) = security_meta.user_id {
                headers.insert("x-user-id".to_string(), user_id.clone());
            }
            if let Some(ref session_id) = security_meta.session_id {
                headers.insert("x-session-id".to_string(), session_id.clone());
            }
        }

        headers
    }

    /// Convert QollectiveError to JsonRpcEnvelopeError
    pub fn qollective_error_to_jsonrpc(
        error: QollectiveError,
        meta: Option<Meta>,
    ) -> JsonRpcEnvelopeError {
        let (code, message) = match error {
            QollectiveError::Validation(msg) => (-32602, format!("Validation error: {}", msg)),
            QollectiveError::Serialization(msg) => (-32603, format!("Serialization error: {}", msg)),
            QollectiveError::Deserialization(msg) => (-32603, format!("Deserialization error: {}", msg)),
            QollectiveError::Transport(msg) => (-32000, format!("Transport error: {}", msg)),
            QollectiveError::Connection(msg) => (-32000, format!("Connection error: {}", msg)),
            QollectiveError::Config(msg) => (-32000, format!("Configuration error: {}", msg)),
            QollectiveError::Security(msg) => (-32000, format!("Security error: {}", msg)),
            QollectiveError::Internal(msg) => (-32603, format!("Internal error: {}", msg)),
            QollectiveError::External(msg) => (-32000, format!("External service error: {}", msg)),
            QollectiveError::Remote(msg) => (-32000, format!("Remote service error: {}", msg)),
            QollectiveError::Envelope(msg) => (-32600, format!("Envelope error: {}", msg)),
            QollectiveError::AgentNotFound(msg) => (-32000, format!("Agent not found: {}", msg)),
            QollectiveError::ProtocolAdapter(msg) => (-32000, format!("Protocol adapter error: {}", msg)),
            QollectiveError::FeatureNotEnabled(msg) => (-32000, format!("Feature not enabled: {}", msg)),
            QollectiveError::McpProtocol(msg) => (-32000, format!("MCP protocol error: {}", msg)),
            QollectiveError::McpToolExecution(msg) => (-32000, format!("MCP tool execution error: {}", msg)),
            QollectiveError::McpServerRegistration(msg) => (-32000, format!("MCP server registration error: {}", msg)),
            QollectiveError::McpClientConnection(msg) => (-32000, format!("MCP client connection error: {}", msg)),
            QollectiveError::McpServerNotFound(msg) => (-32000, format!("MCP server not found: {}", msg)),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsConnection(msg) => (-32000, format!("NATS connection error: {}", msg)),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsMessage(msg) => (-32000, format!("NATS message error: {}", msg)),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsTimeout(msg) => (-32000, format!("NATS timeout error: {}", msg)),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsDiscovery(msg) => (-32000, format!("NATS discovery error: {}", msg)),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsSubject(msg) => (-32000, format!("NATS subject error: {}", msg)),
            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            QollectiveError::NatsAuth(msg) => (-32000, format!("NATS authentication error: {}", msg)),
            QollectiveError::Grpc(msg) => (-32000, format!("gRPC error: {}", msg)),
            QollectiveError::TenantExtraction(msg) => (-32000, format!("Tenant extraction error: {}", msg)),
            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            QollectiveError::McpError(err) => (-32000, format!("rmcp error: {}", err)),
        };

        JsonRpcEnvelopeError {
            code,
            message,
            data: None,
            meta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{EnvelopeBuilder, Meta};

    #[test]
    fn test_envelope_conversion() -> Result<()> {
        let envelope = EnvelopeBuilder::new()
            .with_payload("test_payload")
            .with_meta(Meta::default())
            .build()?;

        let jsonrpc_envelope = JsonRpcEnvelope::from_envelope(envelope.clone());
        let converted_back = jsonrpc_envelope.into_envelope()?;

        assert_eq!(envelope.payload, converted_back.payload);
        Ok(())
    }

    #[test]
    fn test_jsonrpc_request_creation() -> Result<()> {
        let envelope = EnvelopeBuilder::new()
            .with_payload("test_params")
            .with_meta(Meta::default())
            .build()?;

        let request = JsonRpcRequest::from_envelope(
            envelope,
            "test_method".to_string(),
            Some(1),
        );

        assert_eq!(request.method, "test_method");
        assert_eq!(request.params, "test_params");
        assert_eq!(request.id, Some(1));
        Ok(())
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse::success(
            "test_result",
            1,
            Meta::default(),
        );

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, 1);
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let error = JsonRpcEnvelopeError::new(
            -32000,
            "Test error".to_string(),
            Some(Meta::default()),
        );

        let response = JsonRpcResponse::<String>::error(error, 1);

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.id, 1);
    }

    #[test]
    fn test_error_constructors() {
        let meta = Some(Meta::default());
        
        let method_not_found = JsonRpcEnvelopeError::method_not_found("test_method", meta.clone());
        assert_eq!(method_not_found.code, -32601);
        
        let invalid_params = JsonRpcEnvelopeError::invalid_params("invalid data", meta.clone());
        assert_eq!(invalid_params.code, -32602);
        
        let internal_error = JsonRpcEnvelopeError::internal_error("something went wrong", meta.clone());
        assert_eq!(internal_error.code, -32603);
    }

    #[test]
    fn test_header_metadata_extraction() {
        let mut headers = HashMap::new();
        headers.insert("x-trace-id".to_string(), "test-trace-id".to_string());
        headers.insert("x-tenant-id".to_string(), "test-tenant".to_string());
        headers.insert("x-user-id".to_string(), "test-user".to_string());

        let meta = utils::extract_envelope_metadata(&headers);

        assert_eq!(meta.tenant, Some("test-tenant".to_string()));
        assert!(meta.tracing.is_some());
        assert!(meta.security.is_some());
        
        if let Some(tracing_meta) = meta.tracing {
            assert_eq!(tracing_meta.trace_id, Some("test-trace-id".to_string()));
        }
        
        if let Some(security_meta) = meta.security {
            assert_eq!(security_meta.user_id, Some("test-user".to_string()));
        }
    }

    #[test]
    fn test_metadata_injection() {
        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        
        let headers = utils::inject_envelope_metadata(&meta);
        
        assert_eq!(headers.get("x-tenant-id"), Some(&"test-tenant".to_string()));
    }
}