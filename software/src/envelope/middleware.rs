// ABOUTME: Middleware integration utilities for context propagation across HTTP and gRPC
// ABOUTME: Provides unified middleware interfaces for both REST and gRPC to manage envelope contexts

//! Middleware integration utilities for context propagation.
//!
//! This module provides unified middleware interfaces for both REST and gRPC services to
//! seamlessly manage envelope contexts and metadata propagation across service boundaries.
//!
//! Key features:
//! - Context extraction from HTTP headers and gRPC metadata
//! - Automatic context propagation to thread-local storage
//! - Integration with axum middleware and tonic interceptors
//! - Performance metrics collection and tracing integration

use super::{Context, Meta};
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Trait for middleware that can extract and inject envelope context
pub trait ContextMiddleware {
    /// Extract context from the incoming request/metadata
    fn extract_context(&self, headers: &dyn HeaderLike) -> Result<Context>;

    /// Inject context into outgoing request/metadata
    fn inject_context(&self, context: &Context, headers: &mut dyn HeaderLike) -> Result<()>;

    /// Process context before handling the request
    fn process_incoming_context(&self, context: &Context) -> Result<Context> {
        Ok(context.clone())
    }

    /// Process context before sending the response
    fn process_outgoing_context(&self, context: &Context) -> Result<Context> {
        Ok(context.clone())
    }
}

/// Trait abstraction for different header/metadata types
pub trait HeaderLike {
    /// Get a header value by name
    fn get(&self, name: &str) -> Option<&str>;

    /// Set a header value
    fn set(&mut self, name: &str, value: &str) -> Result<()>;

    /// Get all header names
    fn keys(&self) -> Vec<String>;

    /// Check if a header exists
    fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

/// Standard envelope middleware implementation
#[derive(Debug, Clone)]
pub struct EnvelopeMiddleware {
    /// Configuration for header processing
    pub config: MiddlewareConfig,
}

/// Configuration for middleware behavior
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    /// Prefix for extension headers
    pub extension_header_prefix: String,
    /// Headers to extract as metadata
    pub metadata_headers: Vec<String>,
    /// Whether to collect performance metrics
    pub collect_metrics: bool,
    /// Whether to enable tracing
    pub enable_tracing: bool,
    /// Whether tenant extraction is enabled
    pub tenant_extraction_enabled: bool,
    /// Custom header mapping
    pub header_mapping: HashMap<String, String>,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            extension_header_prefix: "x-ext-".to_string(),
            metadata_headers: vec![
                "x-request-id".to_string(),
                "x-correlation-id".to_string(),
                "x-trace-id".to_string(),
                "x-span-id".to_string(),
                "x-user-id".to_string(),
                "x-session-id".to_string(),
                "x-version".to_string(),
                "x-timestamp".to_string(),
                "x-tenant".to_string(),
                "x-on-behalf-of".to_string(),
            ],
            collect_metrics: true,
            enable_tracing: true,
            tenant_extraction_enabled: std::env::var("QOLLECTIVE_TENANT_EXTRACTION")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
            header_mapping: HashMap::new(),
        }
    }
}

impl EnvelopeMiddleware {
    /// Create new middleware with default configuration
    pub fn new() -> Self {
        Self {
            config: MiddlewareConfig::default(),
        }
    }

    /// Create new middleware with custom configuration
    pub fn with_config(config: MiddlewareConfig) -> Self {
        Self { config }
    }

    /// Create a builder for fluent configuration
    pub fn builder() -> MiddlewareBuilder {
        MiddlewareBuilder::new()
    }
}

impl Default for EnvelopeMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMiddleware for EnvelopeMiddleware {
    fn extract_context(&self, headers: &dyn HeaderLike) -> Result<Context> {
        let mut meta = Meta::default();

        // Extract request ID
        if let Some(request_id_str) = headers.get("x-request-id") {
            if let Ok(request_id) = Uuid::parse_str(request_id_str) {
                meta.request_id = Some(request_id);
            }
        }

        // Extract timestamp
        if let Some(timestamp_str) = headers.get("x-timestamp") {
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                meta.timestamp = Some(timestamp.with_timezone(&Utc));
            }
        }

        // Extract version
        if let Some(version) = headers.get("x-version") {
            meta.version = Some(version.to_string());
        }

        // Extract tenant information if enabled
        if self.config.tenant_extraction_enabled {
            if let Some(tenant) = headers.get("x-tenant") {
                meta.tenant = Some(tenant.to_string());
            }

            // Extract onBehalfOf information
            if let Some(on_behalf_of_str) = headers.get("x-on-behalf-of") {
                // Try to parse JSON format or simple delegation
                if let Ok(on_behalf_of_json) =
                    serde_json::from_str::<serde_json::Value>(on_behalf_of_str)
                {
                    // All fields are required in the new structure
                    if let (Some(original_user), Some(delegating_user), Some(delegating_tenant)) = (
                        on_behalf_of_json
                            .get("originalUser")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        on_behalf_of_json
                            .get("delegatingUser")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        on_behalf_of_json
                            .get("delegatingTenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    ) {
                        let on_behalf_of = crate::envelope::meta::OnBehalfOfMeta {
                            original_user,
                            delegating_user,
                            delegating_tenant,
                        };
                        meta.on_behalf_of = Some(on_behalf_of);
                    }
                }
            }
        }

        // Extract security metadata
        let user_id = headers.get("x-user-id").map(|s| s.to_string());
        let session_id = headers.get("x-session-id").map(|s| s.to_string());

        if user_id.is_some() || session_id.is_some() {
            meta.security = Some(crate::envelope::meta::SecurityMeta {
                user_id,
                session_id,
                auth_method: None,
                permissions: Vec::new(),
                ip_address: headers.get("x-forwarded-for").map(|s| s.to_string()),
                user_agent: headers.get("user-agent").map(|s| s.to_string()),
                roles: Vec::new(),
                token_expires_at: None,
            });
        }

        // Extract tracing metadata
        let trace_id = headers.get("x-trace-id").map(|s| s.to_string());
        let span_id = headers.get("x-span-id").map(|s| s.to_string());

        if trace_id.is_some() || span_id.is_some() {
            meta.tracing = Some(crate::envelope::meta::TracingMeta {
                trace_id,
                span_id,
                parent_span_id: headers.get("x-parent-span-id").map(|s| s.to_string()),
                baggage: std::collections::HashMap::new(),
                sampling_rate: None,
                sampled: None,
                trace_state: headers.get("tracestate").map(|s| s.to_string()),
                operation_name: None,
                span_kind: None,
                span_status: None,
                tags: std::collections::HashMap::new(),
            });
        }

        // Extract extension headers
        let mut extensions_map = HashMap::<String, serde_json::Value>::new();
        for key in headers.keys() {
            if key.starts_with(&self.config.extension_header_prefix) {
                if let Some(value) = headers.get(&key) {
                    let ext_key = key
                        .strip_prefix(&self.config.extension_header_prefix)
                        .unwrap_or(&key);

                    // Try to parse as JSON, fallback to string
                    let ext_value = serde_json::from_str::<Value>(value)
                        .unwrap_or_else(|_| Value::String(value.to_string()));
                    extensions_map.insert(ext_key.to_string(), ext_value);
                }
            }
        }

        if !extensions_map.is_empty() {
            meta.extensions = Some(crate::envelope::meta::ExtensionsMeta {
                sections: extensions_map,
            });
        }

        Ok(Context::new(meta))
    }

    fn inject_context(&self, context: &Context, headers: &mut dyn HeaderLike) -> Result<()> {
        let meta = context.meta();

        // Inject request ID
        if let Some(request_id) = meta.request_id {
            headers.set("x-request-id", &request_id.to_string())?;
        }

        // Inject timestamp
        if let Some(timestamp) = meta.timestamp {
            headers.set("x-timestamp", &timestamp.to_rfc3339())?;
        }

        // Inject version
        if let Some(ref version) = meta.version {
            headers.set("x-version", version)?;
        }

        // Inject tenant information if enabled
        if self.config.tenant_extraction_enabled {
            if let Some(ref tenant) = meta.tenant {
                headers.set("x-tenant", tenant)?;
            }

            if let Some(ref on_behalf_of) = meta.on_behalf_of {
                let mut on_behalf_of_json = serde_json::Map::new();

                // All fields are required in the new structure
                on_behalf_of_json.insert(
                    "originalUser".to_string(),
                    serde_json::Value::String(on_behalf_of.original_user.clone()),
                );
                on_behalf_of_json.insert(
                    "delegatingUser".to_string(),
                    serde_json::Value::String(on_behalf_of.delegating_user.clone()),
                );
                on_behalf_of_json.insert(
                    "delegatingTenant".to_string(),
                    serde_json::Value::String(on_behalf_of.delegating_tenant.clone()),
                );

                let on_behalf_of_str = serde_json::to_string(&on_behalf_of_json)
                    .unwrap_or_else(|_| "{}".to_string());
                headers.set("x-on-behalf-of", &on_behalf_of_str)?;
            }
        }

        // Inject security metadata
        if let Some(ref security) = meta.security {
            if let Some(ref user_id) = security.user_id {
                headers.set("x-user-id", user_id)?;
            }
            if let Some(ref session_id) = security.session_id {
                headers.set("x-session-id", session_id)?;
            }
            if let Some(ref ip_address) = security.ip_address {
                headers.set("x-forwarded-for", ip_address)?;
            }
        }

        // Inject tracing metadata
        if let Some(ref tracing) = meta.tracing {
            if let Some(ref trace_id) = tracing.trace_id {
                headers.set("x-trace-id", trace_id)?;
            }
            if let Some(ref span_id) = tracing.span_id {
                headers.set("x-span-id", span_id)?;
            }
        }

        // Inject extension headers
        if let Some(ref extensions) = meta.extensions {
            for (key, value) in &extensions.sections {
                let header_name = format!("{}{}", self.config.extension_header_prefix, key);
                let header_value = value.to_string();
                headers.set(&header_name, &header_value)?;
            }
        }

        Ok(())
    }

    fn process_incoming_context(&self, context: &Context) -> Result<Context> {
        let mut processed_context = context.clone();

        // Add processing timestamp if not present
        if processed_context.meta().timestamp.is_none() {
            processed_context.meta_mut().timestamp = Some(Utc::now());
        }

        // Generate request ID if not present
        if processed_context.meta().request_id.is_none() {
            processed_context.meta_mut().request_id = Some(Uuid::now_v7());
        }

        // Add performance metadata if collection is enabled
        if self.config.collect_metrics {
            processed_context.meta_mut().performance =
                Some(crate::envelope::meta::PerformanceMeta {
                    db_query_time: None,
                    db_query_count: None,
                    cache_hit_ratio: None,
                    cache_operations: None,
                    memory_allocated: None,
                    memory_peak: None,
                    cpu_usage: None,
                    network_latency: None,
                    external_calls: Vec::new(),
                    gc_collections: None,
                    gc_time: None,
                    thread_count: None,
                    processing_time_ms: None,
                });
        }

        Ok(processed_context)
    }

    fn process_outgoing_context(&self, context: &Context) -> Result<Context> {
        let mut processed_context = context.clone();

        // Update performance metadata if collection is enabled
        if self.config.collect_metrics {
            if let Some(ref mut perf) = processed_context.meta_mut().performance {
                // For now, just mark that we processed it
                // In a real implementation, we would calculate actual metrics
                if perf.processing_time_ms.is_none() {
                    perf.processing_time_ms = Some(1); // Placeholder
                }
            }
        }

        Ok(processed_context)
    }
}

/// Builder for middleware configuration
#[derive(Debug)]
pub struct MiddlewareBuilder {
    config: MiddlewareConfig,
}

impl MiddlewareBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: MiddlewareConfig::default(),
        }
    }

    /// Set the extension header prefix
    pub fn extension_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.extension_header_prefix = prefix.into();
        self
    }

    /// Add a metadata header to extract
    pub fn add_metadata_header(mut self, header: impl Into<String>) -> Self {
        self.config.metadata_headers.push(header.into());
        self
    }

    /// Set whether to collect performance metrics
    pub fn collect_metrics(mut self, enabled: bool) -> Self {
        self.config.collect_metrics = enabled;
        self
    }

    /// Set whether to enable tracing
    pub fn enable_tracing(mut self, enabled: bool) -> Self {
        self.config.enable_tracing = enabled;
        self
    }

    /// Add a custom header mapping
    pub fn map_header(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.config.header_mapping.insert(from.into(), to.into());
        self
    }

    /// Build the middleware
    pub fn build(self) -> EnvelopeMiddleware {
        EnvelopeMiddleware::with_config(self.config)
    }
}

impl Default for MiddlewareBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Context propagation utilities
pub mod propagation {
    use super::*;

    /// Run a closure with the given context set as current
    pub async fn with_context<F, R>(context: &Context, f: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        context.run_with(f).await
    }

    /// Extract context from current thread-local storage or create empty
    pub fn current_context_or_empty() -> Context {
        Context::current().unwrap_or_default()
    }

    /// Merge two contexts, with the second taking precedence
    pub fn merge_contexts(base: &Context, overlay: &Context) -> Context {
        let mut merged = base.clone();
        let overlay_meta = overlay.meta();
        let merged_meta = merged.meta_mut();

        // Merge non-None fields from overlay
        if overlay_meta.timestamp.is_some() {
            merged_meta.timestamp = overlay_meta.timestamp;
        }
        if overlay_meta.request_id.is_some() {
            merged_meta.request_id = overlay_meta.request_id;
        }
        if overlay_meta.version.is_some() {
            merged_meta.version = overlay_meta.version.clone();
        }
        if overlay_meta.duration.is_some() {
            merged_meta.duration = overlay_meta.duration;
        }
        if overlay_meta.security.is_some() {
            merged_meta.security = overlay_meta.security.clone();
        }
        if overlay_meta.debug.is_some() {
            merged_meta.debug = overlay_meta.debug.clone();
        }
        if overlay_meta.performance.is_some() {
            merged_meta.performance = overlay_meta.performance.clone();
        }
        if overlay_meta.monitoring.is_some() {
            merged_meta.monitoring = overlay_meta.monitoring.clone();
        }
        if overlay_meta.tracing.is_some() {
            merged_meta.tracing = overlay_meta.tracing.clone();
        }

        // Merge extensions
        if let Some(ref overlay_ext) = overlay_meta.extensions {
            if merged_meta.extensions.is_none() {
                merged_meta.extensions = Some(crate::envelope::meta::ExtensionsMeta {
                    sections: HashMap::new(),
                });
            }

            if let Some(ref mut merged_ext) = merged_meta.extensions {
                for (key, value) in &overlay_ext.sections {
                    merged_ext.sections.insert(key.clone(), value.clone());
                }
            }
        }

        merged
    }

    /// Create a child context with new request/trace IDs
    pub fn create_child_context(parent: &Context) -> Context {
        let mut child = parent.clone();
        let child_meta = child.meta_mut();

        // Generate new request ID for child
        child_meta.request_id = Some(Uuid::now_v7());

        // Update tracing information for child
        if let Some(ref mut tracing) = child_meta.tracing {
            // Generate new span ID (keep same trace ID)
            tracing.span_id = Some(Uuid::now_v7().to_string());
        }

        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock implementation of HeaderLike for testing
    #[derive(Debug, Default)]
    struct MockHeaders {
        headers: HashMap<String, String>,
    }

    impl HeaderLike for MockHeaders {
        fn get(&self, name: &str) -> Option<&str> {
            self.headers.get(name).map(|s| s.as_str())
        }

        fn set(&mut self, name: &str, value: &str) -> Result<()> {
            self.headers.insert(name.to_string(), value.to_string());
            Ok(())
        }

        fn keys(&self) -> Vec<String> {
            self.headers.keys().cloned().collect()
        }
    }

    #[test]
    fn test_envelope_middleware_creation() {
        let middleware = EnvelopeMiddleware::new();
        assert_eq!(middleware.config.extension_header_prefix, "x-ext-");
        assert!(middleware.config.collect_metrics);
        assert!(middleware.config.enable_tracing);
    }

    #[test]
    fn test_middleware_builder() {
        let middleware = EnvelopeMiddleware::builder()
            .extension_prefix("custom-")
            .add_metadata_header("x-custom-header")
            .collect_metrics(false)
            .enable_tracing(false)
            .map_header("old-header", "new-header")
            .build();

        assert_eq!(middleware.config.extension_header_prefix, "custom-");
        assert!(!middleware.config.collect_metrics);
        assert!(!middleware.config.enable_tracing);
        assert!(middleware
            .config
            .metadata_headers
            .contains(&"x-custom-header".to_string()));
        assert_eq!(
            middleware.config.header_mapping.get("old-header"),
            Some(&"new-header".to_string())
        );
    }

    #[test]
    fn test_context_extraction_basic_headers() {
        let middleware = EnvelopeMiddleware::new();
        let mut headers = MockHeaders::default();

        let request_id = Uuid::now_v7();
        let timestamp = Utc::now();

        headers
            .set("x-request-id", &request_id.to_string())
            .unwrap();
        headers.set("x-timestamp", &timestamp.to_rfc3339()).unwrap();
        headers.set("x-version", "1.0.0").unwrap();
        headers.set("x-user-id", "user123").unwrap();
        headers.set("x-trace-id", "trace456").unwrap();

        let context = middleware.extract_context(&headers).unwrap();
        let meta = context.meta();

        assert_eq!(meta.request_id, Some(request_id));
        assert_eq!(meta.version.as_ref().unwrap(), "1.0.0");
        assert_eq!(
            meta.security.as_ref().unwrap().user_id.as_ref().unwrap(),
            "user123"
        );
        assert_eq!(
            meta.tracing.as_ref().unwrap().trace_id.as_ref().unwrap(),
            "trace456"
        );
    }

    #[test]
    fn test_context_extraction_extension_headers() {
        let middleware = EnvelopeMiddleware::new();
        let mut headers = MockHeaders::default();

        headers.set("x-ext-custom-field", "custom-value").unwrap();
        headers.set("x-ext-another-field", "another-value").unwrap();

        let context = middleware.extract_context(&headers).unwrap();
        let meta = context.meta();

        assert!(meta.extensions.is_some());
        let extensions = meta.extensions.as_ref().unwrap();

        assert_eq!(
            extensions.sections.get("custom-field").unwrap(),
            "custom-value"
        );
        assert_eq!(
            extensions.sections.get("another-field").unwrap(),
            "another-value"
        );
    }

    #[test]
    fn test_context_injection() {
        let middleware = EnvelopeMiddleware::new();
        let mut headers = MockHeaders::default();

        let request_id = Uuid::now_v7();
        let timestamp = Utc::now();

        let context = Context::builder()
            .request_id(request_id)
            .timestamp(timestamp)
            .version("2.0.0")
            .build();

        middleware.inject_context(&context, &mut headers).unwrap();

        assert_eq!(headers.get("x-request-id").unwrap(), request_id.to_string());
        assert_eq!(headers.get("x-timestamp").unwrap(), timestamp.to_rfc3339());
        assert_eq!(headers.get("x-version").unwrap(), "2.0.0");
    }

    #[test]
    fn test_context_processing_incoming() {
        let middleware = EnvelopeMiddleware::new();
        let context = Context::empty();

        let processed = middleware.process_incoming_context(&context).unwrap();

        // Should have added timestamp and request ID
        assert!(processed.meta().timestamp.is_some());
        assert!(processed.meta().request_id.is_some());
        assert!(processed.meta().performance.is_some());
    }

    #[test]
    fn test_context_processing_outgoing() {
        let middleware = EnvelopeMiddleware::new();
        let mut context = Context::empty();

        // Add performance metadata with start time
        context.meta_mut().performance = Some(crate::envelope::meta::PerformanceMeta {
            db_query_time: None,
            db_query_count: None,
            cache_hit_ratio: None,
            cache_operations: None,
            memory_allocated: None,
            memory_peak: None,
            cpu_usage: None,
            network_latency: None,
            external_calls: Vec::new(),
            gc_collections: None,
            gc_time: None,
            thread_count: None,
            processing_time_ms: None,
        });

        let processed = middleware.process_outgoing_context(&context).unwrap();

        // Should have updated processing time
        assert!(processed
            .meta()
            .performance
            .as_ref()
            .unwrap()
            .processing_time_ms
            .is_some());
    }

    #[test]
    fn test_propagation_merge_contexts() {
        let base = Context::builder().version("1.0.0").build();

        let overlay = Context::builder()
            .version("2.0.0")
            .request_id(Uuid::now_v7())
            .build();

        let merged = propagation::merge_contexts(&base, &overlay);

        // Version should come from overlay
        assert_eq!(merged.meta().version.as_ref().unwrap(), "2.0.0");
        // Request ID should come from overlay
        assert!(merged.meta().request_id.is_some());
    }

    #[test]
    fn test_propagation_create_child_context() {
        let parent_request_id = Uuid::now_v7();
        let parent_span_id = "parent-span-123";
        let parent_trace_id = "trace-456";

        let mut parent = Context::builder().request_id(parent_request_id).build();

        // Add tracing info
        parent.meta_mut().tracing = Some(crate::envelope::meta::TracingMeta {
            trace_id: Some(parent_trace_id.to_string()),
            span_id: Some(parent_span_id.to_string()),
            parent_span_id: None,
            baggage: std::collections::HashMap::new(),
            sampling_rate: None,
            sampled: None,
            trace_state: None,
            operation_name: None,
            span_kind: None,
            span_status: None,
            tags: std::collections::HashMap::new(),
        });

        let child = propagation::create_child_context(&parent);

        // Child should have different request ID
        assert_ne!(child.meta().request_id, Some(parent_request_id));

        // Child should have same trace ID but different span ID
        let child_tracing = child.meta().tracing.as_ref().unwrap();
        assert_eq!(child_tracing.trace_id.as_ref().unwrap(), parent_trace_id);
        assert_ne!(child_tracing.span_id.as_ref().unwrap(), parent_span_id);
    }

    #[test]
    fn test_tenant_extraction_when_enabled() {
        // Test that tenant information is extracted when tenant extraction is enabled
        let mut config = MiddlewareConfig::default();
        config.tenant_extraction_enabled = true;

        let middleware = EnvelopeMiddleware::with_config(config);
        let mut headers = std::collections::HashMap::new();
        headers.insert("x-tenant".to_string(), "test-tenant".to_string());
        headers.insert(
            "x-on-behalf-of".to_string(),
            r#"{"originalUser":"user123","originalTenant":"original-tenant"}"#.to_string(),
        );

        let adapter = TestHeaderAdapter::new(headers);
        let context = middleware.extract_context(&adapter).unwrap();
        let meta = context.meta();

        assert_eq!(meta.tenant.as_ref().unwrap(), "test-tenant");
        assert!(meta.on_behalf_of.is_some());
        let on_behalf_of = meta.on_behalf_of.as_ref().unwrap();
        assert_eq!(on_behalf_of.original_user.as_ref().unwrap(), "user123");
        assert_eq!(
            on_behalf_of.original_tenant.as_ref().unwrap(),
            "original-tenant"
        );
    }

    #[test]
    fn test_tenant_extraction_when_disabled() {
        // Test that tenant information is NOT extracted when tenant extraction is disabled
        let mut config = MiddlewareConfig::default();
        config.tenant_extraction_enabled = false;

        let middleware = EnvelopeMiddleware::with_config(config);
        let mut headers = std::collections::HashMap::new();
        headers.insert("x-tenant".to_string(), "test-tenant".to_string());
        headers.insert(
            "x-on-behalf-of".to_string(),
            r#"{"originalUser":"user123"}"#.to_string(),
        );

        let adapter = TestHeaderAdapter::new(headers);
        let context = middleware.extract_context(&adapter).unwrap();
        let meta = context.meta();

        assert!(meta.tenant.is_none());
        assert!(meta.on_behalf_of.is_none());
    }

    // Test header adapter for unit testing
    struct TestHeaderAdapter {
        headers: std::collections::HashMap<String, String>,
    }

    impl TestHeaderAdapter {
        fn new(headers: std::collections::HashMap<String, String>) -> Self {
            Self { headers }
        }
    }

    impl HeaderLike for TestHeaderAdapter {
        fn get(&self, name: &str) -> Option<&str> {
            self.headers.get(name).map(|s| s.as_str())
        }

        fn set(&mut self, name: &str, value: &str) -> Result<()> {
            self.headers.insert(name.to_string(), value.to_string());
            Ok(())
        }

        fn keys(&self) -> Vec<String> {
            self.headers.keys().cloned().collect()
        }
    }

    #[tokio::test]
    async fn test_propagation_with_context() {
        let context = Context::builder().version("test-version").build();

        let result = propagation::with_context(&context, async {
            let current = Context::current().unwrap();
            current.meta().version.clone().unwrap()
        })
        .await;

        assert_eq!(result, "test-version");
    }

    #[test]
    fn test_propagation_current_context_or_empty() {
        // Should return empty context when no context is set
        let context = propagation::current_context_or_empty();
        assert!(context.meta().version.is_none());
        assert!(context.meta().request_id.is_none());
    }
}
