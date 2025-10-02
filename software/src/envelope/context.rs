// ABOUTME: Context propagation utilities for metadata flow
// ABOUTME: Handles seamless context propagation through service layers

//! Context propagation utilities for metadata flow.
//!
//! This module provides comprehensive context management including:
//! - Thread-local context storage for async environments
//! - Extension metadata access and manipulation
//! - Context propagation across service boundaries
//! - Builder pattern for fluent context construction

use super::meta::{ExtensionsMeta, Meta};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::task_local;
use uuid::Uuid;

use serde_json::Value;

// Thread-local context storage for async environments
task_local! {
    static CURRENT_CONTEXT: Option<Context>;
}

/// Context container for propagating metadata through service layers
#[derive(Debug, Clone)]
pub struct Context {
    meta: Meta,
}

impl Context {
    /// Create a new context with the given metadata
    pub fn new(meta: Meta) -> Self {
        Self { meta }
    }

    /// Create an empty context
    pub fn empty() -> Self {
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

    /// Get a reference to the underlying metadata
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Get a mutable reference to the underlying metadata
    pub fn meta_mut(&mut self) -> &mut Meta {
        &mut self.meta
    }

    /// Convert the context into its underlying metadata
    pub fn into_meta(self) -> Meta {
        self.meta
    }

    /// Get the current context from thread-local storage
    pub fn current() -> Option<Self> {
        CURRENT_CONTEXT.try_with(|ctx| ctx.clone()).unwrap_or(None)
    }

    /// Run a closure with this context set as the current context
    pub async fn run_with<F, R>(&self, f: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        CURRENT_CONTEXT.scope(Some(self.clone()), f).await
    }

    /// Access extension metadata, creating it if it doesn't exist
    pub fn extensions(&mut self) -> &mut ExtensionsMeta {
        if self.meta.extensions.is_none() {
            self.meta.extensions = Some(ExtensionsMeta {
                sections: HashMap::new(),
            });
        }
        self.meta.extensions.as_mut().unwrap()
    }

    /// Get extension metadata if it exists
    pub fn extensions_ref(&self) -> Option<&ExtensionsMeta> {
        self.meta.extensions.as_ref()
    }

    /// Set a value in the extensions metadata
    pub fn set_extension(&mut self, key: impl Into<String>, value: Value) {
        self.extensions().sections.insert(key.into(), value);
    }

    /// Get a value from the extensions metadata
    pub fn get_extension(&self, key: &str) -> Option<&Value> {
        self.extensions_ref()?.sections.get(key)
    }

    /// Remove a value from the extensions metadata
    pub fn remove_extension(&mut self, key: &str) -> Option<Value> {
        self.extensions().sections.remove(key)
    }

    /// Create a context builder for fluent construction
    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }
}

impl From<Meta> for Context {
    fn from(meta: Meta) -> Self {
        Self::new(meta)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::empty()
    }
}

/// Builder for fluent context construction
#[derive(Debug)]
pub struct ContextBuilder {
    meta: Meta,
}

impl ContextBuilder {
    /// Create a new context builder
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

    /// Set the timestamp
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.meta.timestamp = Some(timestamp);
        self
    }

    /// Set the request ID
    pub fn request_id(mut self, request_id: Uuid) -> Self {
        self.meta.request_id = Some(request_id);
        self
    }

    /// Set the version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.meta.version = Some(version.into());
        self
    }

    /// Set the tenant ID
    pub fn tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.meta.tenant = Some(tenant_id.into());
        self
    }

    /// Set the onBehalfOf metadata
    pub fn on_behalf_of(mut self, on_behalf_of: super::meta::OnBehalfOfMeta) -> Self {
        self.meta.on_behalf_of = Some(on_behalf_of);
        self
    }

    /// Add an extension value
    pub fn extension(mut self, key: impl Into<String>, value: Value) -> Self {
        if self.meta.extensions.is_none() {
            self.meta.extensions = Some(ExtensionsMeta {
                sections: HashMap::new(),
            });
        }
        self.meta
            .extensions
            .as_mut()
            .unwrap()
            .sections
            .insert(key.into(), value);
        self
    }

    /// Build the context
    pub fn build(self) -> Context {
        Context::new(self.meta)
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for context propagation behavior
pub trait ContextPropagation {
    fn as_context(&self) -> Context;
    fn enrich_performance(&self) -> Meta;
    fn enrich_monitoring(&self) -> Meta;
    fn enrich_tracing(&self) -> Meta;
}

impl ContextPropagation for Meta {
    fn as_context(&self) -> Context {
        Context::new(self.clone())
    }

    fn enrich_performance(&self) -> Meta {
        // Extension point: Implement custom performance metadata enrichment
        // Example: Add current CPU usage, memory metrics, or request timing data
        // Default implementation returns metadata unchanged
        self.clone()
    }

    fn enrich_monitoring(&self) -> Meta {
        // Extension point: Implement custom monitoring metadata enrichment
        // Example: Add server_id, datacenter, build_version, or health status
        // Default implementation returns metadata unchanged
        self.clone()
    }

    fn enrich_tracing(&self) -> Meta {
        // Extension point: Implement custom tracing metadata enrichment
        // Example: Add trace_id, span_id, baggage, or OpenTelemetry context
        // Default implementation returns metadata unchanged
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    use serde_json::json;

    #[test]
    fn test_context_creation_and_basic_operations() {
        // ARRANGE: Create a context with metadata
        let request_id = Uuid::now_v7();
        let timestamp = Utc::now();

        let meta = Meta {
            timestamp: Some(timestamp),
            request_id: Some(request_id),
            version: Some("1.0.0".to_string()),
            duration: Some(123.45),
            tenant: None,
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        };

        // ACT: Create context
        let context = Context::new(meta.clone());

        // ASSERT: Verify context properties
        assert_eq!(context.meta().timestamp, Some(timestamp));
        assert_eq!(context.meta().request_id, Some(request_id));
        assert_eq!(context.meta().version.as_ref().unwrap(), "1.0.0");
        assert_eq!(context.meta().duration, Some(123.45));
    }

    #[test]
    fn test_context_empty_creation() {
        // ACT: Create empty context
        let context = Context::empty();

        // ASSERT: Verify all fields are None
        assert!(context.meta().timestamp.is_none());
        assert!(context.meta().request_id.is_none());
        assert!(context.meta().version.is_none());
        assert!(context.meta().duration.is_none());
        assert!(context.meta().extensions.is_none());
    }

    #[test]
    fn test_context_default() {
        // ACT: Create default context
        let context = Context::default();

        // ASSERT: Should be equivalent to empty
        assert!(context.meta().timestamp.is_none());
        assert!(context.meta().request_id.is_none());
        assert!(context.meta().version.is_none());
        assert!(context.meta().extensions.is_none());
    }

    #[test]
    fn test_context_from_meta() {
        // ARRANGE: Create meta
        let meta = Meta {
            timestamp: Some(Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("2.0.0".to_string()),
            duration: None,
            tenant: None,
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        };

        // ACT: Create context from meta
        let context: Context = meta.clone().into();

        // ASSERT: Verify conversion
        assert_eq!(context.meta().version.as_ref().unwrap(), "2.0.0");
        assert_eq!(context.meta().request_id, meta.request_id);
    }

    #[test]
    fn test_context_into_meta() {
        // ARRANGE: Create context
        let original_meta = Meta {
            timestamp: Some(Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("3.0.0".to_string()),
            duration: Some(456.78),
            tenant: None,
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        };
        let context = Context::new(original_meta.clone());

        // ACT: Convert back to meta
        let extracted_meta = context.into_meta();

        // ASSERT: Verify conversion
        assert_eq!(extracted_meta.version.as_ref().unwrap(), "3.0.0");
        assert_eq!(extracted_meta.duration, Some(456.78));
        assert_eq!(extracted_meta.request_id, original_meta.request_id);
    }

    #[test]
    fn test_extensions_with_serde_json() {
        // ARRANGE: Create empty context
        let mut context = Context::empty();

        // ACT: Add extension data
        context.set_extension("user_preference", json!({"theme": "dark", "lang": "en"}));
        context.set_extension("session_data", json!({"timeout": 3600}));

        // ASSERT: Verify extension data
        let user_pref = context.get_extension("user_preference").unwrap();
        assert_eq!(user_pref["theme"], "dark");
        assert_eq!(user_pref["lang"], "en");

        let session_data = context.get_extension("session_data").unwrap();
        assert_eq!(session_data["timeout"], 3600);

        // Test non-existent key
        assert!(context.get_extension("non_existent").is_none());
    }


    #[test]
    fn test_extension_removal() {
        // ARRANGE: Create context with extensions
        let mut context = Context::empty();
        context.set_extension("key1", json!("value1"));
        context.set_extension("key2", json!("value2"));

        // ACT: Remove one extension
        let removed = context.remove_extension("key1");

        // ASSERT: Verify removal
        assert_eq!(removed.unwrap(), json!("value1"));
        assert!(context.get_extension("key1").is_none());
        assert!(context.get_extension("key2").is_some());

        // Test removing non-existent key
        assert!(context.remove_extension("non_existent").is_none());
    }

    #[test]
    fn test_context_builder() {
        // ARRANGE: Prepare test data
        let request_id = Uuid::now_v7();
        let timestamp = Utc::now();

        // ACT: Build context using builder pattern
        let context = Context::builder()
            .request_id(request_id)
            .timestamp(timestamp)
            .version("1.2.3")
            .build();

        // ASSERT: Verify built context
        assert_eq!(context.meta().request_id, Some(request_id));
        assert_eq!(context.meta().timestamp, Some(timestamp));
        assert_eq!(context.meta().version.as_ref().unwrap(), "1.2.3");
    }

    #[test]
    fn test_context_builder_with_extensions() {
        // ACT: Build context with extensions
        let context = Context::builder()
            .version("2.0.0")
            .extension("config", json!({"debug": true}))
            .extension("metadata", json!({"source": "test"}))
            .build();

        // ASSERT: Verify extensions in built context
        assert_eq!(context.meta().version.as_ref().unwrap(), "2.0.0");

        let config = context.get_extension("config").unwrap();
        assert_eq!(config["debug"], true);

        let metadata = context.get_extension("metadata").unwrap();
        assert_eq!(metadata["source"], "test");
    }

    #[test]
    fn test_context_builder_default() {
        // ACT: Create default builder
        let builder = ContextBuilder::default();
        let context = builder.build();

        // ASSERT: Should create empty context
        assert!(context.meta().timestamp.is_none());
        assert!(context.meta().version.is_none());
        assert!(context.meta().extensions.is_none());
    }

    #[tokio::test]
    async fn test_thread_local_context_storage() {
        // ARRANGE: Create context
        let context = Context::builder().version("test-version").build();

        // Test that no context is set initially
        assert!(Context::current().is_none());

        // ACT & ASSERT: Run with context
        context
            .run_with(async {
                // Inside the context scope
                let current = Context::current().unwrap();
                assert_eq!(current.meta().version.as_ref().unwrap(), "test-version");

                // Test nested context operations
                let nested_result = async {
                    let current_nested = Context::current().unwrap();
                    assert_eq!(
                        current_nested.meta().version.as_ref().unwrap(),
                        "test-version"
                    );
                    "nested_success"
                }
                .await;

                assert_eq!(nested_result, "nested_success");
            })
            .await;

        // After context scope, should be None again
        assert!(Context::current().is_none());
    }

    #[tokio::test]
    async fn test_context_isolation_between_tasks() {
        // ARRANGE: Create two different contexts
        let context1 = Context::builder().version("v1").build();
        let context2 = Context::builder().version("v2").build();

        // ACT: Run both contexts concurrently
        let (result1, result2) = tokio::join!(
            context1.run_with(async {
                // Simulate some async work
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Context::current().unwrap().meta().version.clone().unwrap()
            }),
            context2.run_with(async {
                // Simulate some async work
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Context::current().unwrap().meta().version.clone().unwrap()
            })
        );

        // ASSERT: Each task should see its own context
        assert_eq!(result1, "v1");
        assert_eq!(result2, "v2");
    }

    #[test]
    fn test_context_propagation_trait() {
        // ARRANGE: Create meta with data
        let meta = Meta {
            timestamp: Some(Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("propagation-test".to_string()),
            duration: None,
            tenant: None,
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        };

        // ACT: Use ContextPropagation trait
        let context = meta.as_context();

        // ASSERT: Verify conversion
        assert_eq!(context.meta().version.as_ref().unwrap(), "propagation-test");

        // Test enrichment methods (currently just clones)
        let enriched_perf = meta.enrich_performance();
        let enriched_mon = meta.enrich_monitoring();
        let enriched_trace = meta.enrich_tracing();

        assert_eq!(enriched_perf.version, meta.version);
        assert_eq!(enriched_mon.version, meta.version);
        assert_eq!(enriched_trace.version, meta.version);
    }

    #[test]
    fn test_context_meta_mut() {
        // ARRANGE: Create context
        let mut context = Context::empty();

        // ACT: Modify metadata through mutable reference
        {
            let meta_mut = context.meta_mut();
            meta_mut.version = Some("modified".to_string());
            meta_mut.request_id = Some(Uuid::now_v7());
        }

        // ASSERT: Verify modifications
        assert_eq!(context.meta().version.as_ref().unwrap(), "modified");
        assert!(context.meta().request_id.is_some());
    }

    #[test]
    fn test_extensions_lazy_initialization() {
        // ARRANGE: Create empty context
        let mut context = Context::empty();

        // Verify extensions are None initially
        assert!(context.extensions_ref().is_none());

        // ACT: Access extensions (should create them)
        let extensions = context.extensions();

        // ASSERT: Extensions should now exist
        assert!(extensions.sections.is_empty());
        assert!(context.extensions_ref().is_some());
    }

    #[test]
    fn test_complex_extension_data() {
        // ARRANGE: Create context
        let mut context = Context::empty();

        // ACT: Add complex nested data
        let complex_data = json!({
            "user": {
                "id": 12345,
                "preferences": {
                    "theme": "dark",
                    "notifications": true,
                    "languages": ["en", "es", "fr"]
                }
            },
            "session": {
                "start_time": "2025-06-07T10:00:00Z",
                "actions": ["login", "navigate", "search"]
            }
        });

        context.set_extension("complex_data", complex_data.clone());

        // ASSERT: Verify complex data retrieval
        let retrieved = context.get_extension("complex_data").unwrap();
        assert_eq!(retrieved["user"]["id"], 12345);
        assert_eq!(retrieved["user"]["preferences"]["theme"], "dark");
        assert_eq!(retrieved["session"]["actions"][0], "login");

        // Verify array access
        let languages = &retrieved["user"]["preferences"]["languages"];
        assert!(languages.is_array());
        assert_eq!(languages.as_array().unwrap().len(), 3);
    }
}
