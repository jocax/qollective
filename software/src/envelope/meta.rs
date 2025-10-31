// ABOUTME: Metadata structures and builders for envelope sections
// ABOUTME: Defines the core metadata types for security, debug, performance, etc.

//! Metadata structures and builders for envelope sections.

#[allow(unused_imports)]
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Core metadata container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Enhanced Meta",
    description = "Comprehensive metadata structure including all context types and extensible custom fields for enterprise applications"
))]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<OnBehalfOfMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<MonitoringMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<TracingMeta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionsMeta>,
}

/// OnBehalfOf metadata for delegation context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Delegation Context",
    description = "Context for on-behalf-of scenarios and delegation chains in enterprise systems",
))]
pub struct OnBehalfOfMeta {
    /// The original user or entity being acted upon
    #[serde(rename = "originalUser")]
    #[cfg_attr(feature = "openapi", schema(example = "picard@starfleet.local"))]
    pub original_user: String,

    /// The user or service performing the action on behalf of the original user
    #[serde(rename = "delegatingUser")]
    #[cfg_attr(feature = "openapi", schema(example = "riker@starfleet.local"))]
    pub delegating_user: String,

    /// The tenant context of the delegating user
    #[serde(rename = "delegatingTenant")]
    #[cfg_attr(feature = "openapi", schema(example = "enterprise_starfleet"))]
    pub delegating_tenant: String,
}

/// Authentication method enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Authentication Method",
    description = "Supported authentication methods for enterprise security"
))]
pub enum AuthMethod {
    Unspecified,
    OAuth2,
    Jwt,
    ApiKey,
    Basic,
    Saml,
    Oidc,
    None,
}

/// Security metadata section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Security Context",
    description = "Comprehensive security context including authentication, authorization, and session management for enterprise applications",
))]
pub struct SecurityMeta {
    /// User identifier for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "picard@starfleet.local"))]
    pub user_id: Option<String>,

    /// Session identifier for tracking user sessions
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "bridge_session_001"))]
    pub session_id: Option<String>,

    /// Authentication method used for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<AuthMethod>,

    /// List of permissions granted to the user
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "openapi", schema(example = "COMMAND_SHIP,CREW_MANAGEMENT"))]
    pub permissions: Vec<String>,

    /// Client IP address for security tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "192.168.1.100"))]
    pub ip_address: Option<String>,

    /// User agent string from the client
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "Enterprise Bridge Console v2.0"))]
    pub user_agent: Option<String>,

    /// User roles within the system
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "openapi", schema(example = "Captain,Bridge Officer"))]
    pub roles: Vec<String>,

    /// Token expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "2025-08-23T11:30:45.123Z"))]
    pub token_expires_at: Option<DateTime<Utc>>,
}

/// Log level enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum LogLevel {
    Unspecified,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Database query execution information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct DbQuery {
    /// The database query that was executed
    pub query: String,
    /// Query execution duration in milliseconds
    pub duration: f64,
    /// Number of rows affected by the query (for INSERT/UPDATE/DELETE operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows_affected: Option<i32>,
    /// Database name or identifier where the query was executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
}

/// Memory usage metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MemoryUsage {
    /// Amount of heap memory currently in use (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heap_used: Option<i64>,
    /// Total heap memory available (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heap_total: Option<i64>,
    /// External memory usage such as buffers or off-heap storage (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<i64>,
}

/// Performance profiling data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProfilingData {
    /// CPU time consumed during request processing (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_time: Option<f64>,
    /// Wall clock time for request processing (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_time: Option<f64>,
    /// Number of memory allocations performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocations: Option<i64>,
}

/// Debug metadata section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Debug Context",
    description = "Development and debugging information including debug mode, profiling, and development tools support",
))]
pub struct DebugMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub db_queries: Vec<DbQuery>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage: Option<MemoryUsage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub environment_vars: HashMap<String, String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub request_headers: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiling_data: Option<ProfilingData>,
}

/// Call status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CallStatus {
    /// Status not specified or unknown
    Unspecified,
    /// Call completed successfully
    Success,
    /// Call failed with an error
    Error,
    /// Call timed out before completion
    Timeout,
}

/// Cache operation statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CacheOperations {
    /// Number of cache hits during request processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hits: Option<i32>,
    /// Number of cache misses during request processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misses: Option<i32>,
    /// Number of cache set operations performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sets: Option<i32>,
}

/// External service call performance data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ExternalCall {
    /// Name or identifier of the external service called
    pub service: String,
    /// Duration of the external service call (milliseconds)
    pub duration: f64,
    /// Status of the external service call result
    pub status: CallStatus,
    /// Specific endpoint or operation called on the external service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Performance metadata section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Performance Context",
    description = "Performance metrics and monitoring data for request timing and resource usage analysis",
))]
pub struct PerformanceMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_query_time: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_query_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit_ratio: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_operations: Option<CacheOperations>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_allocated: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_peak: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_latency: Option<f64>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external_calls: Vec<ExternalCall>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_collections: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_time: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_count: Option<i32>,

    // Keep the original field for backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
}

/// Environment enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Environment {
    Unspecified,
    Development,
    Staging,
    Testing,
    Production,
    Canary,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum HealthStatus {
    /// Health status not specified or unknown
    Unspecified,
    /// Service is operating normally
    Healthy,
    /// Service is functional but with reduced performance
    Degraded,
    /// Service is not functioning properly
    Unhealthy,
}

/// Monitoring metadata section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Infrastructure Context",
    description = "Infrastructure and deployment information including service, environment, and networking details",
))]
pub struct MonitoringMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub datacenter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Environment>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status: Option<HealthStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<f64>,
}

/// OpenTelemetry span kind enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SpanKind {
    Unspecified,
    Server,
    Client,
    Producer,
    Consumer,
    Internal,
}

/// Span status code enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SpanStatusCode {
    /// Status code not specified
    Unspecified,
    /// Span completed successfully
    Ok,
    /// Span completed with an error
    Error,
    /// Span timed out during execution
    Timeout,
}

/// Span status information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SpanStatus {
    /// Status code indicating the span completion state
    pub code: SpanStatusCode,
    /// Optional human-readable status message with additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Trace value for tags/attributes (supports string, number, boolean)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum TraceValue {
    /// String trace value
    String(String),
    /// Numeric trace value
    Number(f64),
    /// Boolean trace value
    Boolean(bool),
}

/// Tracing metadata section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Tracing Context",
    description = "OpenTelemetry-compatible distributed tracing context with trace and span information",
))]
pub struct TracingMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub baggage: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_kind: Option<SpanKind>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_status: Option<SpanStatus>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub tags: HashMap<String, TraceValue>,
}

/// Extensible metadata container for service-specific data
///
/// ExtensionsMeta provides the proper mechanism for extending Qollective
/// metadata with custom fields while maintaining type safety and schema validation.
/// The flattened HashMap allows arbitrary extension data to be included
/// at the same level as core metadata fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Extensions Meta",
    description = "Extensible metadata container for service-specific and application-specific data",
    example = json!({
        "service_name": "holodeck-coordinator",
        "environment": "production",
        "custom_fields": {
            "starship_class": "Galaxy",
            "registry": "NCC-1701-D"
        }
    })
))]
pub struct ExtensionsMeta {
    /// Custom extension sections flattened into the parent structure
    ///
    /// This HashMap is flattened using serde, meaning its contents
    /// appear at the same level as other metadata fields in JSON.
    /// This is the correct way to extend Qollective metadata.
    #[serde(flatten)]
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "service_name": "holodeck-coordinator",
            "environment": "production"
        }),
        additional_properties = true
    ))]
    pub sections: HashMap<String, serde_json::Value>,
}

/// Builder for metadata sections
pub struct MetaBuilder {
    meta: Meta,
}

impl MetaBuilder {
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
        }
    }

    pub fn build(self) -> Meta {
        self.meta
    }
}

impl Default for MetaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Meta {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Meta {
    /// Create a response metadata preserving key fields from the original request
    /// This follows the same pattern used by the gRPC server for consistency
    pub fn preserve_for_response(original_meta: Option<&Meta>) -> Self {
        if let Some(orig) = original_meta {
            Self {
                // Preserve these critical fields from the original request
                request_id: orig.request_id,
                tenant: orig.tenant.clone(),
                version: orig.version.clone(),
                on_behalf_of: orig.on_behalf_of.clone(),
                security: orig.security.clone(),

                // Update timestamp to response time
                timestamp: Some(chrono::Utc::now()),

                // Reset response-specific fields
                duration: None,
                debug: None,
                performance: None,
                monitoring: None,
                tracing: orig.tracing.clone(),  // Preserve tracing metadata for distributed tracing
                extensions: None,
            }
        } else {
            // Fallback with minimal default metadata
            Self {
                timestamp: Some(chrono::Utc::now()),
                version: Some("1.0".to_string()),
                ..Default::default()
            }
        }
    }

    /// Create metadata for new requests (when no original metadata exists)
    pub fn for_new_request() -> Self {
        Self {
            timestamp: Some(chrono::Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("1.0".to_string()),
            ..Default::default()
        }
    }
}

/// Trait for metadata sections
pub trait MetaSection {
    fn is_enabled(&self) -> bool;
    fn as_json(&self) -> Result<serde_json::Value>;
}
