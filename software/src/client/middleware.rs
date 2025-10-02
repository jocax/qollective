// ABOUTME: Client-side middleware utilities for context propagation in REST and gRPC clients
// ABOUTME: Provides integration hooks for injecting envelope context into outgoing requests

//! Client-side middleware utilities for context propagation.
//!
//! This module provides utilities for integrating envelope context propagation
//! into REST and gRPC clients, ensuring that metadata flows correctly through
//! service call chains.

use crate::{
    client::common::TenantClientConfig,
    envelope::{Context, ContextMiddleware, EnvelopeMiddleware, HeaderLike},
    error::{QollectiveError, Result},
};
use std::str::FromStr;

/// Apply tenant configuration to context before propagation
pub fn apply_tenant_config(
    context: &mut Context,
    tenant_config: &TenantClientConfig,
) -> Result<()> {
    let meta = context.meta_mut();

    // Handle tenant ID override or auto-propagation
    if let Some(override_tenant) = &tenant_config.override_tenant_id {
        meta.tenant = Some(override_tenant.clone());
    } else if tenant_config.auto_propagate_tenant {
        // If no tenant in current context and we have a fallback, use it
        if meta.tenant.is_none() {
            if let Some(fallback_tenant) = &tenant_config.fallback_tenant_id {
                meta.tenant = Some(fallback_tenant.clone());
            }
        }
        // Otherwise, keep the existing tenant from context (if any)
    } else {
        // Auto-propagation is disabled, clear tenant
        meta.tenant = None;
    }

    // Handle onBehalfOf propagation
    if !tenant_config.propagate_on_behalf_of {
        meta.on_behalf_of = None;
    }

    Ok(())
}

/// REST client middleware for HTTP headers
#[cfg(feature = "rest-client")]
pub struct RestClientMiddleware {
    envelope_middleware: EnvelopeMiddleware,
}

#[cfg(feature = "rest-client")]
impl RestClientMiddleware {
    /// Create new REST client middleware
    pub fn new() -> Self {
        Self {
            envelope_middleware: EnvelopeMiddleware::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_envelope_middleware(middleware: EnvelopeMiddleware) -> Self {
        Self {
            envelope_middleware: middleware,
        }
    }

    /// Inject context into reqwest headers
    pub fn inject_into_reqwest_headers(
        &self,
        context: &Context,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<()> {
        let mut header_adapter = ReqwestHeaderAdapter::new(headers);
        self.envelope_middleware
            .inject_context(context, &mut header_adapter)
    }

    /// Extract context from reqwest headers
    pub fn extract_from_reqwest_headers(
        &self,
        headers: &reqwest::header::HeaderMap,
    ) -> Result<Context> {
        let header_adapter = ReqwestHeaderAdapter::from_headers(headers);
        self.envelope_middleware.extract_context(&header_adapter)
    }

    /// Inject context into reqwest headers with tenant configuration
    pub fn inject_into_reqwest_headers_with_tenant_config(
        &self,
        context: &Context,
        headers: &mut reqwest::header::HeaderMap,
        tenant_config: &TenantClientConfig,
    ) -> Result<()> {
        let mut processed_context = context.clone();
        apply_tenant_config(&mut processed_context, tenant_config)?;

        let mut header_adapter = ReqwestHeaderAdapter::new(headers);
        self.envelope_middleware
            .inject_context(&processed_context, &mut header_adapter)
    }
}

#[cfg(feature = "rest-client")]
impl Default for RestClientMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter for reqwest HeaderMap to implement HeaderLike
#[cfg(feature = "rest-client")]
struct ReqwestHeaderAdapter<'a> {
    headers: &'a mut reqwest::header::HeaderMap,
}

#[cfg(feature = "rest-client")]
impl<'a> ReqwestHeaderAdapter<'a> {
    fn new(headers: &'a mut reqwest::header::HeaderMap) -> Self {
        Self { headers }
    }

    fn from_headers(headers: &'a reqwest::header::HeaderMap) -> ReqwestHeaderAdapterRead<'a> {
        ReqwestHeaderAdapterRead { headers }
    }
}

#[cfg(feature = "rest-client")]
impl<'a> HeaderLike for ReqwestHeaderAdapter<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        use reqwest::header::HeaderName;

        if let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) {
            if let Some(header_value) = self.headers.get(&header_name) {
                return header_value.to_str().ok();
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        use reqwest::header::{HeaderName, HeaderValue};

        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|e| {
            QollectiveError::transport(format!("Invalid header name '{}': {}", name, e))
        })?;
        let header_value = HeaderValue::from_str(value).map_err(|e| {
            QollectiveError::transport(format!("Invalid header value for '{}': {}", name, e))
        })?;

        self.headers.insert(header_name, header_value);
        Ok(())
    }

    fn keys(&self) -> Vec<String> {
        self.headers
            .keys()
            .map(|k| k.as_str().to_string())
            .collect()
    }
}

/// Read-only adapter for reqwest HeaderMap
#[cfg(feature = "rest-client")]
struct ReqwestHeaderAdapterRead<'a> {
    headers: &'a reqwest::header::HeaderMap,
}

#[cfg(feature = "rest-client")]
impl<'a> HeaderLike for ReqwestHeaderAdapterRead<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        use reqwest::header::HeaderName;

        if let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) {
            if let Some(header_value) = self.headers.get(&header_name) {
                return header_value.to_str().ok();
            }
        }
        None
    }

    fn set(&mut self, _name: &str, _value: &str) -> Result<()> {
        Err(QollectiveError::internal("Cannot modify read-only headers"))
    }

    fn keys(&self) -> Vec<String> {
        self.headers
            .keys()
            .map(|k| k.as_str().to_string())
            .collect()
    }
}

/// gRPC client middleware for metadata
#[cfg(feature = "grpc-client")]
#[derive(Clone)]
pub struct GrpcClientMiddleware {
    envelope_middleware: EnvelopeMiddleware,
}

#[cfg(feature = "grpc-client")]
impl GrpcClientMiddleware {
    /// Create new gRPC client middleware
    pub fn new() -> Self {
        Self {
            envelope_middleware: EnvelopeMiddleware::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_envelope_middleware(middleware: EnvelopeMiddleware) -> Self {
        Self {
            envelope_middleware: middleware,
        }
    }

    /// Inject context into tonic metadata
    pub fn inject_into_tonic_metadata(
        &self,
        context: &Context,
        metadata: &mut tonic::metadata::MetadataMap,
    ) -> Result<()> {
        let mut metadata_adapter = TonicMetadataAdapter::new(metadata);
        self.envelope_middleware
            .inject_context(context, &mut metadata_adapter)
    }

    /// Extract context from tonic metadata
    pub fn extract_from_tonic_metadata(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<Context> {
        let metadata_adapter = TonicMetadataAdapterRead::from_metadata(metadata);
        self.envelope_middleware.extract_context(&metadata_adapter)
    }

    /// Inject context into tonic metadata with tenant configuration
    pub fn inject_into_tonic_metadata_with_tenant_config(
        &self,
        context: &Context,
        metadata: &mut tonic::metadata::MetadataMap,
        tenant_config: &TenantClientConfig,
    ) -> Result<()> {
        let mut processed_context = context.clone();
        apply_tenant_config(&mut processed_context, tenant_config)?;

        let mut metadata_adapter = TonicMetadataAdapter::new(metadata);
        self.envelope_middleware
            .inject_context(&processed_context, &mut metadata_adapter)
    }
}

#[cfg(feature = "grpc-client")]
impl Default for GrpcClientMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter for tonic MetadataMap to implement HeaderLike
#[cfg(feature = "grpc-client")]
struct TonicMetadataAdapter<'a> {
    metadata: &'a mut tonic::metadata::MetadataMap,
}

#[cfg(feature = "grpc-client")]
impl<'a> TonicMetadataAdapter<'a> {
    fn new(metadata: &'a mut tonic::metadata::MetadataMap) -> Self {
        Self { metadata }
    }
}

#[cfg(feature = "grpc-client")]
impl<'a> HeaderLike for TonicMetadataAdapter<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        use tonic::metadata::MetadataKey;

        if let Ok(key) = MetadataKey::from_bytes(name.as_bytes()) {
            if let Some(value) = self.metadata.get(&key) {
                return value.to_str().ok();
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        use tonic::metadata::{MetadataKey, MetadataValue};

        let key = MetadataKey::from_bytes(name.as_bytes()).map_err(|e| {
            QollectiveError::transport(format!("Invalid metadata key '{}': {}", name, e))
        })?;
        let value = MetadataValue::from_str(value).map_err(|e| {
            QollectiveError::transport(format!("Invalid metadata value for '{}': {}", name, e))
        })?;

        self.metadata.insert(key, value);
        Ok(())
    }

    fn keys(&self) -> Vec<String> {
        self.metadata.keys().map(|k| format!("{:?}", k)).collect()
    }
}

/// Read-only adapter for tonic MetadataMap
#[cfg(feature = "grpc-client")]
struct TonicMetadataAdapterRead<'a> {
    metadata: &'a tonic::metadata::MetadataMap,
}

#[cfg(feature = "grpc-client")]
impl<'a> TonicMetadataAdapterRead<'a> {
    fn from_metadata(metadata: &'a tonic::metadata::MetadataMap) -> Self {
        Self { metadata }
    }
}

#[cfg(feature = "grpc-client")]
impl<'a> HeaderLike for TonicMetadataAdapterRead<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        use tonic::metadata::MetadataKey;

        if let Ok(key) = MetadataKey::from_bytes(name.as_bytes()) {
            if let Some(value) = self.metadata.get(&key) {
                return value.to_str().ok();
            }
        }
        None
    }

    fn set(&mut self, _name: &str, _value: &str) -> Result<()> {
        Err(QollectiveError::internal(
            "Cannot modify read-only metadata",
        ))
    }

    fn keys(&self) -> Vec<String> {
        self.metadata.keys().map(|k| format!("{:?}", k)).collect()
    }
}

/// Interceptor for automatic context injection in gRPC clients
#[cfg(feature = "grpc-client")]
#[derive(Clone)]
pub struct ContextInjectionInterceptor {
    middleware: GrpcClientMiddleware,
    tenant_config: Option<TenantClientConfig>,
}

#[cfg(feature = "grpc-client")]
impl ContextInjectionInterceptor {
    /// Create new context injection interceptor
    pub fn new() -> Self {
        Self {
            middleware: GrpcClientMiddleware::new(),
            tenant_config: None,
        }
    }

    /// Create with custom middleware
    pub fn with_middleware(middleware: GrpcClientMiddleware) -> Self {
        Self {
            middleware,
            tenant_config: None,
        }
    }

    /// Create with tenant configuration
    pub fn with_tenant_config(tenant_config: TenantClientConfig) -> Self {
        Self {
            middleware: GrpcClientMiddleware::new(),
            tenant_config: Some(tenant_config),
        }
    }

    /// Create with both custom middleware and tenant configuration
    pub fn with_middleware_and_tenant_config(
        middleware: GrpcClientMiddleware,
        tenant_config: TenantClientConfig,
    ) -> Self {
        Self {
            middleware,
            tenant_config: Some(tenant_config),
        }
    }
}

#[cfg(feature = "grpc-client")]
impl Default for ContextInjectionInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "grpc-client")]
impl tonic::service::Interceptor for ContextInjectionInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        // Get current context or create empty one
        let context = Context::current().unwrap_or_default();

        // Inject context into request metadata with optional tenant configuration
        let result = if let Some(ref tenant_config) = self.tenant_config {
            self.middleware
                .inject_into_tonic_metadata_with_tenant_config(
                    &context,
                    request.metadata_mut(),
                    tenant_config,
                )
        } else {
            self.middleware
                .inject_into_tonic_metadata(&context, request.metadata_mut())
        };

        if let Err(e) = result {
            return Err(tonic::Status::internal(format!(
                "Failed to inject context: {}",
                e
            )));
        }

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Context;
    use uuid::Uuid;

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_rest_client_middleware_creation() {
        let middleware = RestClientMiddleware::new();
        assert_eq!(
            middleware
                .envelope_middleware
                .config
                .extension_header_prefix,
            "x-ext-"
        );
    }

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_reqwest_header_adapter() {
        use reqwest::header::HeaderMap;

        let mut headers = HeaderMap::new();
        let mut adapter = ReqwestHeaderAdapter::new(&mut headers);

        // Test setting and getting headers
        adapter.set("x-test-header", "test-value").unwrap();
        assert_eq!(adapter.get("x-test-header").unwrap(), "test-value");

        // Test keys
        let keys = adapter.keys();
        assert!(keys.contains(&"x-test-header".to_string()));
    }

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_rest_context_injection() {
        use reqwest::header::HeaderMap;

        let middleware = RestClientMiddleware::new();
        let mut headers = HeaderMap::new();

        let context = Context::builder()
            .request_id(Uuid::now_v7())
            .version("1.0.0")
            .build();

        middleware
            .inject_into_reqwest_headers(&context, &mut headers)
            .unwrap();

        assert!(headers.contains_key("x-request-id"));
        assert!(headers.contains_key("x-version"));
        assert_eq!(headers.get("x-version").unwrap().to_str().unwrap(), "1.0.0");
    }

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_rest_context_extraction() {
        use reqwest::header::{HeaderMap, HeaderValue};

        let middleware = RestClientMiddleware::new();
        let mut headers = HeaderMap::new();

        let request_id = Uuid::now_v7();
        headers.insert(
            "x-request-id",
            HeaderValue::from_str(&request_id.to_string()).unwrap(),
        );
        headers.insert("x-version", HeaderValue::from_str("2.0.0").unwrap());
        headers.insert("x-user-id", HeaderValue::from_str("user123").unwrap());

        let context = middleware.extract_from_reqwest_headers(&headers).unwrap();
        let meta = context.meta();

        assert_eq!(meta.request_id, Some(request_id));
        assert_eq!(meta.version.as_ref().unwrap(), "2.0.0");
        assert_eq!(
            meta.security.as_ref().unwrap().user_id.as_ref().unwrap(),
            "user123"
        );
    }

    #[cfg(feature = "grpc-client")]
    #[test]
    fn test_grpc_client_middleware_creation() {
        let middleware = GrpcClientMiddleware::new();
        assert_eq!(
            middleware
                .envelope_middleware
                .config
                .extension_header_prefix,
            "x-ext-"
        );
    }

    #[cfg(feature = "grpc-client")]
    #[test]
    fn test_tonic_metadata_adapter() {
        use tonic::metadata::MetadataMap;

        let mut metadata = MetadataMap::new();
        let mut adapter = TonicMetadataAdapter::new(&mut metadata);

        // Test setting and getting metadata
        adapter.set("x-test-key", "test-value").unwrap();
        assert_eq!(adapter.get("x-test-key").unwrap(), "test-value");

        // Test keys
        let keys = adapter.keys();
        assert!(keys.iter().any(|k| k.contains("x-test-key")));
    }

    #[cfg(feature = "grpc-client")]
    #[test]
    fn test_grpc_context_injection() {
        use tonic::metadata::MetadataMap;

        let middleware = GrpcClientMiddleware::new();
        let mut metadata = MetadataMap::new();

        let context = Context::builder()
            .request_id(Uuid::now_v7())
            .version("1.0.0")
            .build();

        middleware
            .inject_into_tonic_metadata(&context, &mut metadata)
            .unwrap();

        assert!(metadata.contains_key("x-request-id"));
        assert!(metadata.contains_key("x-version"));
        assert_eq!(
            metadata.get("x-version").unwrap().to_str().unwrap(),
            "1.0.0"
        );
    }

    #[cfg(feature = "grpc-client")]
    #[test]
    fn test_grpc_context_extraction() {
        use tonic::metadata::{MetadataMap, MetadataValue};

        let middleware = GrpcClientMiddleware::new();
        let mut metadata = MetadataMap::new();

        let request_id = Uuid::now_v7();
        use std::str::FromStr;
        metadata.insert(
            "x-request-id",
            MetadataValue::from_str(&request_id.to_string()).unwrap(),
        );
        metadata.insert("x-version", MetadataValue::from_str("2.0.0").unwrap());
        metadata.insert("x-trace-id", MetadataValue::from_str("trace123").unwrap());

        let context = middleware.extract_from_tonic_metadata(&metadata).unwrap();
        let meta = context.meta();

        assert_eq!(meta.request_id, Some(request_id));
        assert_eq!(meta.version.as_ref().unwrap(), "2.0.0");
        assert_eq!(
            meta.tracing.as_ref().unwrap().trace_id.as_ref().unwrap(),
            "trace123"
        );
    }

    #[cfg(feature = "grpc-client")]
    #[tokio::test]
    async fn test_context_injection_interceptor() {
        use tonic::Request;

        let mut interceptor = ContextInjectionInterceptor::new();
        let request = Request::new(());

        // Create a context with some data and run the test within it
        let test_context = Context::builder()
            .version("test-version")
            .request_id(uuid::Uuid::now_v7())
            .build();

        let result = test_context
            .run_with(async {
                use tonic::service::Interceptor;
                interceptor.call(request)
            })
            .await;

        assert!(result.is_ok());

        let processed_request = result.unwrap();
        // Should have metadata injected from the current context
        assert!(processed_request.metadata().contains_key("x-version"));
        assert!(processed_request.metadata().contains_key("x-request-id"));
    }

    #[test]
    fn test_apply_tenant_config_with_override() {
        use super::super::common::TenantClientConfig;
        use crate::envelope::meta::Meta;

        let mut context = Context::new(Meta {
            timestamp: None,
            request_id: None,
            version: None,
            duration: None,
            tenant: Some("original-tenant".to_string()),
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        });

        let tenant_config = TenantClientConfig {
            auto_propagate_tenant: true,
            override_tenant_id: Some("override-tenant".to_string()),
            propagate_on_behalf_of: true,
            fallback_tenant_id: None,
        };

        apply_tenant_config(&mut context, &tenant_config).unwrap();

        assert_eq!(context.meta().tenant.as_ref().unwrap(), "override-tenant");
    }

    #[test]
    fn test_apply_tenant_config_with_fallback() {
        use super::super::common::TenantClientConfig;
        use crate::envelope::meta::Meta;

        let mut context = Context::new(Meta {
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
        });

        let tenant_config = TenantClientConfig {
            auto_propagate_tenant: true,
            override_tenant_id: None,
            propagate_on_behalf_of: true,
            fallback_tenant_id: Some("fallback-tenant".to_string()),
        };

        apply_tenant_config(&mut context, &tenant_config).unwrap();

        assert_eq!(context.meta().tenant.as_ref().unwrap(), "fallback-tenant");
    }

    #[test]
    fn test_apply_tenant_config_disabled_propagation() {
        use super::super::common::TenantClientConfig;
        use crate::envelope::meta::{Meta, OnBehalfOfMeta};

        let mut context = Context::new(Meta {
            timestamp: None,
            request_id: None,
            version: None,
            duration: None,
            tenant: Some("existing-tenant".to_string()),
            on_behalf_of: Some(OnBehalfOfMeta {
                original_user: "user123".to_string(),
                delegating_user: "delegator789".to_string(),
                delegating_tenant: "delegator-tenant".to_string(),
            }),
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        });

        let tenant_config = TenantClientConfig {
            auto_propagate_tenant: false,
            override_tenant_id: None,
            propagate_on_behalf_of: false,
            fallback_tenant_id: None,
        };

        apply_tenant_config(&mut context, &tenant_config).unwrap();

        assert!(context.meta().tenant.is_none());
        assert!(context.meta().on_behalf_of.is_none());
    }

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_rest_client_middleware_with_tenant_config() {
        use super::super::common::TenantClientConfig;
        use crate::envelope::middleware::{EnvelopeMiddleware, MiddlewareConfig};
        use reqwest::header::HeaderMap;

        // Create middleware with tenant extraction enabled
        let mut config = MiddlewareConfig::default();
        config.tenant_extraction_enabled = true;
        let envelope_middleware = EnvelopeMiddleware::with_config(config);
        let middleware = RestClientMiddleware::with_envelope_middleware(envelope_middleware);
        let mut headers = HeaderMap::new();

        let context = Context::builder()
            .request_id(Uuid::now_v7())
            .version("1.0.0")
            .tenant("original-tenant")
            .build();

        let tenant_config = TenantClientConfig {
            auto_propagate_tenant: true,
            override_tenant_id: Some("override-tenant".to_string()),
            propagate_on_behalf_of: true,
            fallback_tenant_id: None,
        };

        middleware
            .inject_into_reqwest_headers_with_tenant_config(&context, &mut headers, &tenant_config)
            .unwrap();

        assert!(headers.contains_key("x-request-id"));
        assert!(headers.contains_key("x-version"));
        assert!(headers.contains_key("x-tenant"));
        assert_eq!(
            headers.get("x-tenant").unwrap().to_str().unwrap(),
            "override-tenant"
        );
    }
}
