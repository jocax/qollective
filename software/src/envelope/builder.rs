// ABOUTME: Envelope builder and container structures
// ABOUTME: Provides fluent API for building envelopes with metadata and payload

//! Envelope builder and container structures.

use super::meta::Meta;
use crate::error::Result;

use serde::{Deserialize, Serialize};
use serde_json;
#[cfg(feature = "openapi")]
#[allow(unused_imports)] // Used in utoipa schema examples, but not detected by dead code analysis
use serde_json::json;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Core envelope structure containing metadata and payload for enterprise applications
///
/// The Envelope<T> provides a unified wrapper for all inter-service communication,
/// ensuring consistent metadata propagation and error handling across protocols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Envelope",
    description = "Universal envelope structure wrapping all service communication with comprehensive metadata and optional error information",
    example = json!({
        "meta": {
            "timestamp": "2025-08-23T10:30:45.123Z",
            "request_id": "01912345-1234-5678-9abc-123456789def",
            "version": "1.0",
            "tenant": "enterprise_starfleet"
        },
        "payload": {
            "message": "Welcome to the Enterprise Bridge",
            "status": "ready"
        }
    })
))]
pub struct Envelope<T> {
    /// Comprehensive metadata including security, tracing, performance, and debugging information
    pub meta: Meta,

    /// The actual payload being transmitted between services
    pub payload: T,

    /// Optional error information for failed operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EnvelopeError>,
}

/// Error information in envelope responses with comprehensive OpenAPI documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Envelope Error",
    description = "Detailed error information for failed envelope operations",
    example = json!({
        "code": "VALIDATION_FAILED",
        "message": "Required tenant_id field is missing",
        "details": {
            "field": "tenant_id",
            "expected": "string",
            "got": "null"
        },
        "trace": "at validate_envelope (envelope.rs:123)",
        "http_status_code": 400
    })
))]
pub struct EnvelopeError {
    /// Error code for programmatic error handling
    #[cfg_attr(feature = "openapi", schema(example = "VALIDATION_FAILED"))]
    pub code: String,

    /// Human-readable error message
    #[cfg_attr(feature = "openapi", schema(example = "Required tenant_id field is missing"))]
    pub message: String,

    /// Additional structured error details
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(
        example = json!({"field": "tenant_id", "expected": "string", "got": "null"})
    ))]
    pub details: Option<serde_json::Value>,

    /// Optional stack trace for debugging
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "at validate_envelope (envelope.rs:123)"))]
    pub trace: Option<String>,

    /// HTTP status code to use for this error (HTTP protocols only)
    /// Only available when HTTP-related features are enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    #[cfg_attr(feature = "openapi", schema(example = 400))]
    pub http_status_code: Option<u16>,
}

/// Builder for creating envelopes with fluent API and comprehensive OpenAPI support
///
/// EnvelopeBuilder provides a type-safe way to construct Envelope instances with
/// proper validation and error handling for enterprise applications.
#[derive(Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Envelope Builder",
    description = "Fluent builder for constructing Envelope instances with validation and error handling",
))]
pub struct EnvelopeBuilder<T> {
    /// Metadata to be included in the envelope
    meta: Meta,

    /// Optional data payload
    payload: Option<T>,

    /// Optional error information
    error: Option<EnvelopeError>,
}

impl<T> EnvelopeBuilder<T> {
    pub fn new() -> Self {
        Self {
            meta: Meta {
                timestamp: None,
                request_id: None,
                version: None,
                duration: None,
                tenant: None,
                on_behalf_of: None,
                security: None,
                debug: None,
                performance: None,
                monitoring: None,
                tracing: None,
                extensions: None,
            },
            payload: None,
            error: None,
        }
    }

    pub fn with_payload(mut self, payload: T) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_meta(mut self, meta: Meta) -> Self {
        self.meta = meta;
        self
    }

    pub fn with_error(mut self, error: EnvelopeError) -> Self {
        self.error = Some(error);
        self
    }

    /// Build the envelope with validation
    pub fn build(self) -> Result<Envelope<T>> {
        let payload = self
            .payload
            .ok_or_else(|| crate::error::QollectiveError::envelope("payload is required"))?;

        Ok(Envelope {
            meta: self.meta,
            payload,
            error: self.error,
        })
    }

    /// Build a successful envelope (data required, no error)
    pub fn build_success(self) -> Result<Envelope<T>> {
        if self.error.is_some() {
            return Err(crate::error::QollectiveError::envelope("cannot build success envelope with error"));
        }
        self.build()
    }

    /// Build an error envelope (error required, data still needed)
    pub fn build_error(self) -> Result<Envelope<T>> {
        if self.error.is_none() {
            return Err(crate::error::QollectiveError::envelope("cannot build error envelope without error details"));
        }
        self.build()
    }

    /// Add tenant information to metadata
    pub fn with_tenant(mut self, tenant: String) -> Self {
        self.meta.tenant = Some(tenant);
        self
    }

    /// Add request ID to metadata
    pub fn with_request_id(mut self, request_id: uuid::Uuid) -> Self {
        self.meta.request_id = Some(request_id);
        self
    }

    /// Add timestamp to metadata (sets to current time)
    pub fn with_timestamp(mut self) -> Self {
        self.meta.timestamp = Some(chrono::Utc::now());
        self
    }

    /// Add version to metadata
    pub fn with_version(mut self, version: String) -> Self {
        self.meta.version = Some(version);
        self
    }
}

impl<T> Default for EnvelopeBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Envelope<T> {
    /// Create a new envelope with metadata and data
    pub fn new(meta: Meta, payload: T) -> Self {
        Self {
            meta,
            payload,
            error: None,
        }
    }

    /// Create a new envelope with minimal metadata
    pub fn new_minimal(payload: T) -> Self {
        let meta = Meta::for_new_request();
        Self::new(meta, payload)
    }

    /// Create an error envelope
    pub fn error(meta: Meta, payload: T, error: EnvelopeError) -> Self {
        Self {
            meta,
            payload,
            error: Some(error),
        }
    }

    /// Get the fluent builder
    pub fn builder() -> EnvelopeBuilder<T> {
        EnvelopeBuilder::new()
    }

    /// Extract the meta and data components
    pub fn extract(self) -> (Meta, T) {
        (self.meta, self.payload)
    }

    /// Extract all components including error
    pub fn extract_all(self) -> (Meta, T, Option<EnvelopeError>) {
        (self.meta, self.payload, self.error)
    }

    /// Check if this envelope contains an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Check if this envelope is successful (no error)
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}
