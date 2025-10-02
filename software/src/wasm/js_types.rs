// ABOUTME: JavaScript type definitions and conversions for WASM interop
// ABOUTME: Provides wasm-bindgen compatible envelope structures for browser communication

//! JavaScript type definitions for WASM envelope communication.
//!
//! This module provides wasm-bindgen compatible structures that mirror
//! the Rust envelope types but are optimized for JavaScript interop.

use crate::envelope::{Context, Envelope, Meta};
use crate::error::QollectiveError;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// JavaScript-compatible envelope wrapper
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEnvelope {
    meta: WasmMeta,
    #[wasm_bindgen(skip)]
    pub data: serde_json::Value,
    error: Option<WasmEnvelopeError>,
}

/// JavaScript-compatible metadata structure
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMeta {
    timestamp: Option<String>,
    request_id: Option<String>,
    version: Option<String>,
    duration: Option<u64>,
    tenant: Option<String>,
    on_behalf_of: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
    user_id: Option<String>,
    session_id: Option<String>,
    correlation_id: Option<String>,
}

/// JavaScript-compatible context structure
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmContext {
    tenant: Option<String>,
    user_id: Option<String>,
    session_id: Option<String>,
    trace_id: Option<String>,
    correlation_id: Option<String>,
}

/// JavaScript-compatible error structure
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEnvelopeError {
    code: String,
    message: String,
    details: Option<String>,
    trace: Option<String>,
    user_friendly: bool,
    retry_policy: String,
}

#[wasm_bindgen]
impl WasmEnvelope {
    /// Create a new WASM envelope
    #[wasm_bindgen(constructor)]
    pub fn new(meta: WasmMeta, data: JsValue) -> Result<WasmEnvelope, JsValue> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize data: {}", e)))?;

        Ok(WasmEnvelope {
            meta,
            data,
            error: None,
        })
    }

    /// Get envelope metadata
    #[wasm_bindgen(getter)]
    pub fn meta(&self) -> WasmMeta {
        self.meta.clone()
    }

    /// Set envelope metadata
    #[wasm_bindgen(setter)]
    pub fn set_meta(&mut self, meta: WasmMeta) {
        self.meta = meta;
    }

    /// Get envelope data as JsValue
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize data: {}", e)))
    }

    /// Set envelope data from JsValue
    #[wasm_bindgen(setter)]
    pub fn set_data(&mut self, data: JsValue) -> Result<(), JsValue> {
        self.data = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize data: {}", e)))?;
        Ok(())
    }

    /// Get envelope error
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<WasmEnvelopeError> {
        self.error.clone()
    }

    /// Set envelope error
    #[wasm_bindgen(setter)]
    pub fn set_error(&mut self, error: Option<WasmEnvelopeError>) {
        self.error = error;
    }

    /// Check if envelope has error
    #[wasm_bindgen]
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Convert to JSON string
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))
    }

    /// Create from JSON string
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<WasmEnvelope, JsValue> {
        serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize envelope: {}", e)))
    }
}

#[wasm_bindgen]
impl WasmMeta {
    /// Create new metadata with auto-populated fields
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMeta {
        use crate::constants::metadata;

        WasmMeta {
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            request_id: Some(uuid::Uuid::now_v7().to_string()),
            version: Some(metadata::QOLLECTIVE_VERSION.to_string()),
            duration: None,
            tenant: None,
            on_behalf_of: None,
            trace_id: Some(uuid::Uuid::now_v7().to_string()),
            span_id: Some(uuid::Uuid::now_v7().to_string()),
            user_id: None,
            session_id: None,
            correlation_id: Some(uuid::Uuid::now_v7().to_string()),
        }
    }

    /// Create metadata with auto fields populated
    #[wasm_bindgen]
    pub fn with_auto_fields() -> WasmMeta {
        WasmMeta::new()
    }

    /// Set tenant information
    #[wasm_bindgen]
    pub fn with_tenant(mut self, tenant: &str) -> WasmMeta {
        self.tenant = Some(tenant.to_string());
        self
    }

    /// Set user information
    #[wasm_bindgen]
    pub fn with_user_id(mut self, user_id: &str) -> WasmMeta {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Set session information
    #[wasm_bindgen]
    pub fn with_session_id(mut self, session_id: &str) -> WasmMeta {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Get request ID
    #[wasm_bindgen(getter)]
    pub fn request_id(&self) -> Option<String> {
        self.request_id.clone()
    }

    /// Get timestamp
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Option<String> {
        self.timestamp.clone()
    }

    /// Get tenant
    #[wasm_bindgen(getter)]
    pub fn tenant(&self) -> Option<String> {
        self.tenant.clone()
    }

    /// Get user ID
    #[wasm_bindgen(getter)]
    pub fn user_id(&self) -> Option<String> {
        self.user_id.clone()
    }

    /// Get trace ID
    #[wasm_bindgen(getter)]
    pub fn trace_id(&self) -> Option<String> {
        self.trace_id.clone()
    }
}

#[wasm_bindgen]
impl WasmContext {
    /// Create new context
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmContext {
        WasmContext {
            tenant: None,
            user_id: None,
            session_id: None,
            trace_id: None,
            correlation_id: None,
        }
    }

    /// Set tenant
    #[wasm_bindgen]
    pub fn with_tenant(mut self, tenant: &str) -> WasmContext {
        self.tenant = Some(tenant.to_string());
        self
    }

    /// Get tenant
    #[wasm_bindgen(getter)]
    pub fn tenant(&self) -> Option<String> {
        self.tenant.clone()
    }

    /// Get user ID
    #[wasm_bindgen(getter)]
    pub fn user_id(&self) -> Option<String> {
        self.user_id.clone()
    }
}

#[wasm_bindgen]
impl WasmEnvelopeError {
    /// Create new error
    #[wasm_bindgen(constructor)]
    pub fn new(code: &str, message: &str) -> WasmEnvelopeError {
        WasmEnvelopeError {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            trace: None,
            user_friendly: false,
            retry_policy: "none".to_string(),
        }
    }

    /// Set error details
    #[wasm_bindgen]
    pub fn with_details(mut self, details: &str) -> WasmEnvelopeError {
        self.details = Some(details.to_string());
        self
    }

    /// Mark as user-friendly error
    #[wasm_bindgen]
    pub fn with_user_friendly(mut self, user_friendly: bool) -> WasmEnvelopeError {
        self.user_friendly = user_friendly;
        self
    }

    /// Set retry policy
    #[wasm_bindgen]
    pub fn with_retry_policy(mut self, policy: &str) -> WasmEnvelopeError {
        self.retry_policy = policy.to_string();
        self
    }

    /// Get error code
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }

    /// Get error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Check if user-friendly
    #[wasm_bindgen(getter)]
    pub fn user_friendly(&self) -> bool {
        self.user_friendly
    }

    /// Get retry policy
    #[wasm_bindgen(getter)]
    pub fn retry_policy(&self) -> String {
        self.retry_policy.clone()
    }
}

// Conversion traits between Rust and WASM types
impl From<Meta> for WasmMeta {
    fn from(meta: Meta) -> Self {
        WasmMeta {
            timestamp: meta.timestamp.map(|t| t.to_rfc3339()),
            request_id: meta.request_id,
            version: meta.version,
            duration: meta.duration.map(|d| d.as_millis() as u64),
            tenant: meta.tenant,
            on_behalf_of: meta.on_behalf_of,
            trace_id: meta.tracing.as_ref().and_then(|t| t.trace_id.clone()),
            span_id: meta.tracing.as_ref().and_then(|t| t.span_id.clone()),
            user_id: meta.security.as_ref().and_then(|s| s.user_id.clone()),
            session_id: meta.security.as_ref().and_then(|s| s.session_id.clone()),
            correlation_id: meta
                .security
                .as_ref()
                .and_then(|s| s.correlation_id.clone()),
        }
    }
}

impl From<WasmMeta> for Meta {
    fn from(wasm_meta: WasmMeta) -> Self {
        use crate::envelope::meta::*;
        use std::time::Duration;

        let mut meta = Meta {
            timestamp: wasm_meta
                .timestamp
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|t| t.with_timezone(&chrono::Utc)),
            request_id: wasm_meta.request_id,
            version: wasm_meta.version,
            duration: wasm_meta.duration.map(Duration::from_millis),
            tenant: wasm_meta.tenant,
            on_behalf_of: wasm_meta.on_behalf_of,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        };

        // Build tracing section if any tracing fields are present
        if wasm_meta.trace_id.is_some() || wasm_meta.span_id.is_some() {
            meta.tracing = Some(TracingMeta {
                trace_id: wasm_meta.trace_id,
                span_id: wasm_meta.span_id,
                parent_span_id: None,
                baggage: None,
                context: None,
            });
        }

        // Build security section if any security fields are present
        if wasm_meta.user_id.is_some()
            || wasm_meta.session_id.is_some()
            || wasm_meta.correlation_id.is_some()
        {
            meta.security = Some(SecurityMeta {
                user_id: wasm_meta.user_id,
                session_id: wasm_meta.session_id,
                correlation_id: wasm_meta.correlation_id,
                permissions: None,
                scopes: None,
                auth_method: None,
                token_type: None,
            });
        }

        meta
    }
}

impl<T> From<Envelope<T>> for WasmEnvelope
where
    T: Serialize,
{
    fn from(envelope: Envelope<T>) -> Self {
        let data = serde_json::to_value(&envelope.payload).unwrap_or(serde_json::Value::Null);
        let error = envelope.error.map(|e| WasmEnvelopeError {
            code: e.code,
            message: e.message,
            details: e.details.map(|d| d.to_string()),
            trace: e.trace,
            user_friendly: false,
            retry_policy: "none".to_string(),
        });

        WasmEnvelope {
            meta: WasmMeta::from(envelope.meta),
            data,
            error,
        }
    }
}

impl From<QollectiveError> for WasmEnvelopeError {
    fn from(error: QollectiveError) -> Self {
        use crate::error::QollectiveError;

        let (code, user_friendly, retry_policy) = match &error {
            QollectiveError::Validation(_) => ("validation_error", true, "none"),
            QollectiveError::Transport(_) => ("transport_error", false, "exponential_backoff"),
            QollectiveError::Connection(_) => ("connection_error", false, "immediate_retry"),
            QollectiveError::Security(_) => ("security_error", true, "none"),
            QollectiveError::External(_) => ("external_error", false, "linear_backoff"),
            QollectiveError::Internal(_) => ("internal_error", false, "none"),
            _ => ("unknown_error", false, "none"),
        };

        WasmEnvelopeError {
            code: code.to_string(),
            message: error.to_string(),
            details: None,
            trace: None,
            user_friendly,
            retry_policy: retry_policy.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_meta_auto_fields() {
        let meta = WasmMeta::new();
        assert!(meta.timestamp.is_some());
        assert!(meta.request_id.is_some());
        assert!(meta.version.is_some());
        assert!(meta.trace_id.is_some());
        assert!(meta.span_id.is_some());
        assert!(meta.correlation_id.is_some());
    }

    #[test]
    fn test_meta_conversion() {
        let wasm_meta = WasmMeta::new().with_tenant("test-tenant");
        let rust_meta: Meta = wasm_meta.clone().into();
        let converted_back: WasmMeta = rust_meta.into();

        assert_eq!(wasm_meta.tenant, converted_back.tenant);
        assert_eq!(wasm_meta.request_id, converted_back.request_id);
    }

    #[test]
    fn test_error_conversion() {
        let qollective_error = QollectiveError::validation("Invalid input");
        let wasm_error: WasmEnvelopeError = qollective_error.into();

        assert_eq!(wasm_error.code, "validation_error");
        assert!(wasm_error.user_friendly);
        assert_eq!(wasm_error.retry_policy, "none");
    }
}
