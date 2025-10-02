// ABOUTME: Server-side middleware utilities for context propagation in REST and gRPC servers
// ABOUTME: Provides integration hooks for extracting envelope context from incoming requests

//! Server-side middleware utilities for context propagation.
//!
//! This module provides utilities for integrating envelope context propagation
//! into REST and gRPC servers, ensuring that metadata flows correctly through
//! service request processing pipelines.

use crate::{
    envelope::{propagation, Context, ContextMiddleware, EnvelopeMiddleware, HeaderLike},
    error::{QollectiveError, Result},
};

/// REST server middleware for HTTP requests
#[cfg(feature = "rest-server")]
#[derive(Clone)]
pub struct RestServerMiddleware {
    pub envelope_middleware: EnvelopeMiddleware,
}

#[cfg(feature = "rest-server")]
impl RestServerMiddleware {
    /// Create new REST server middleware
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

    /// Set tenant extraction enabled
    pub fn set_tenant_extraction_enabled(&mut self, enabled: bool) {
        self.envelope_middleware.config.tenant_extraction_enabled = enabled;
    }

    /// Extract context from axum request headers
    pub fn extract_from_axum_headers(&self, headers: &axum::http::HeaderMap) -> Result<Context> {
        let header_adapter = AxumHeaderAdapter::from_headers(headers);
        self.envelope_middleware.extract_context(&header_adapter)
    }

    /// Record HTTP request metrics
    pub fn record_http_metrics(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        crate::monitoring::record_http_request(method, path, status, duration);
    }

    /// Record envelope operation metrics
    pub fn record_envelope_metrics(
        &self,
        operation: &str,
        duration: std::time::Duration,
        success: bool,
    ) {
        crate::monitoring::record_envelope_operation(operation, duration, success);
    }

    /// Inject context into axum response headers
    pub fn inject_into_axum_headers(
        &self,
        context: &Context,
        headers: &mut axum::http::HeaderMap,
    ) -> Result<()> {
        let mut header_adapter = AxumHeaderAdapterMut::new(headers);
        self.envelope_middleware
            .inject_context(context, &mut header_adapter)
    }

    /// Create an axum layer for automatic context propagation
    pub fn axum_layer(&self) -> ContextPropagationLayer {
        ContextPropagationLayer::new(self.envelope_middleware.clone())
    }
}

#[cfg(feature = "rest-server")]
impl Default for RestServerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Axum layer for context propagation
#[cfg(feature = "rest-server")]
#[derive(Clone)]
pub struct ContextPropagationLayer {
    middleware: EnvelopeMiddleware,
}

#[cfg(feature = "rest-server")]
impl ContextPropagationLayer {
    /// Create new context propagation layer
    pub fn new(middleware: EnvelopeMiddleware) -> Self {
        Self { middleware }
    }
}

#[cfg(feature = "rest-server")]
impl<S> tower::Layer<S> for ContextPropagationLayer {
    type Service = ContextPropagationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ContextPropagationService {
            inner,
            middleware: self.middleware.clone(),
        }
    }
}

/// Axum service for context propagation
#[cfg(feature = "rest-server")]
#[derive(Clone)]
pub struct ContextPropagationService<S> {
    inner: S,
    middleware: EnvelopeMiddleware,
}

#[cfg(feature = "rest-server")]
impl<S> tower::Service<axum::http::Request<axum::body::Body>> for ContextPropagationService<S>
where
    S: tower::Service<
            axum::http::Request<axum::body::Body>,
            Response = axum::http::Response<axum::body::Body>,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = axum::http::Response<axum::body::Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<Self::Response, Self::Error>>
                + Send,
        >,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: axum::http::Request<axum::body::Body>) -> Self::Future {
        let middleware = self.middleware.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let start_time = std::time::Instant::now();
            let method = request.method().to_string();
            let path = request.uri().path().to_string();

            // Extract context from request headers
            let header_adapter = AxumHeaderAdapter::from_headers(request.headers());
            let context = middleware
                .extract_context(&header_adapter)
                .unwrap_or_else(|_| Context::empty());

            // Process incoming context
            let processed_context = middleware
                .process_incoming_context(&context)
                .unwrap_or(context);

            // Run the request with the context
            let response = processed_context
                .run_with(async { inner.call(request).await.map_err(Into::into) })
                .await?;

            // Record metrics
            let duration = start_time.elapsed();
            let status = response.status().as_u16();
            crate::monitoring::record_http_request(&method, &path, status, duration);

            Ok(response)
        })
    }
}

/// Read-only adapter for axum HeaderMap
#[cfg(feature = "rest-server")]
struct AxumHeaderAdapter<'a> {
    headers: &'a axum::http::HeaderMap,
}

#[cfg(feature = "rest-server")]
impl<'a> AxumHeaderAdapter<'a> {
    fn from_headers(headers: &'a axum::http::HeaderMap) -> Self {
        Self { headers }
    }
}

#[cfg(feature = "rest-server")]
impl<'a> HeaderLike for AxumHeaderAdapter<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        use axum::http::HeaderName;

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

/// Mutable adapter for axum HeaderMap
#[cfg(feature = "rest-server")]
struct AxumHeaderAdapterMut<'a> {
    headers: &'a mut axum::http::HeaderMap,
}

#[cfg(feature = "rest-server")]
impl<'a> AxumHeaderAdapterMut<'a> {
    fn new(headers: &'a mut axum::http::HeaderMap) -> Self {
        Self { headers }
    }
}

#[cfg(feature = "rest-server")]
impl<'a> HeaderLike for AxumHeaderAdapterMut<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        use axum::http::HeaderName;

        if let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) {
            if let Some(header_value) = self.headers.get(&header_name) {
                return header_value.to_str().ok();
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        use axum::http::{HeaderName, HeaderValue};

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

/// gRPC server middleware for handling requests
#[cfg(feature = "grpc-server")]
#[derive(Clone)]
pub struct GrpcServerMiddleware {
    pub envelope_middleware: EnvelopeMiddleware,
}

#[cfg(feature = "grpc-server")]
impl GrpcServerMiddleware {
    /// Create new gRPC server middleware
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

    /// Set tenant extraction enabled
    pub fn set_tenant_extraction_enabled(&mut self, enabled: bool) {
        self.envelope_middleware.config.tenant_extraction_enabled = enabled;
    }

    /// Record gRPC request metrics
    pub fn record_grpc_metrics(&self, method: &str, status: &str, duration: std::time::Duration) {
        // Use our helper function from monitoring module
        crate::monitoring::middleware_integration::record_grpc_request(method, status, duration);
    }

    /// Extract context from tonic request metadata
    pub fn extract_from_tonic_metadata(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<Context> {
        let metadata_adapter = TonicMetadataAdapter::from_metadata(metadata);
        self.envelope_middleware.extract_context(&metadata_adapter)
    }

    /// Create a tonic interceptor for automatic context propagation
    pub fn tonic_interceptor(&self) -> ContextExtractionInterceptor {
        ContextExtractionInterceptor::new(self.envelope_middleware.clone())
    }
}

#[cfg(feature = "grpc-server")]
impl Default for GrpcServerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Interceptor for automatic context extraction in gRPC servers
#[cfg(feature = "grpc-server")]
#[derive(Clone)]
pub struct ContextExtractionInterceptor {
    middleware: EnvelopeMiddleware,
}

#[cfg(feature = "grpc-server")]
impl ContextExtractionInterceptor {
    /// Create new context extraction interceptor
    pub fn new(middleware: EnvelopeMiddleware) -> Self {
        Self { middleware }
    }
}

#[cfg(feature = "grpc-server")]
impl tonic::service::Interceptor for ContextExtractionInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        // Extract context from request metadata
        let metadata_adapter = TonicMetadataAdapter::from_metadata(request.metadata());
        let context = self
            .middleware
            .extract_context(&metadata_adapter)
            .unwrap_or_else(|_| Context::empty());

        // Process incoming context
        let processed_context = self
            .middleware
            .process_incoming_context(&context)
            .unwrap_or(context);

        // Store context in request extensions
        request.extensions_mut().insert(processed_context);

        // Store start time for metrics
        request.extensions_mut().insert(std::time::Instant::now());

        Ok(request)
    }
}

/// Read-only adapter for tonic MetadataMap
#[cfg(feature = "grpc-server")]
struct TonicMetadataAdapter<'a> {
    metadata: &'a tonic::metadata::MetadataMap,
}

#[cfg(feature = "grpc-server")]
impl<'a> TonicMetadataAdapter<'a> {
    fn from_metadata(metadata: &'a tonic::metadata::MetadataMap) -> Self {
        Self { metadata }
    }
}

#[cfg(feature = "grpc-server")]
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

    fn set(&mut self, _name: &str, _value: &str) -> Result<()> {
        Err(QollectiveError::internal(
            "Cannot modify read-only metadata",
        ))
    }

    fn keys(&self) -> Vec<String> {
        self.metadata.keys().map(|k| format!("{:?}", k)).collect()
    }
}

/// Utility functions for server middleware integration
pub mod utils {
    use super::*;

    /// Extract context from current request (works in axum handlers)
    #[cfg(feature = "rest-server")]
    pub fn extract_context_from_axum_request(headers: &axum::http::HeaderMap) -> Result<Context> {
        let middleware = RestServerMiddleware::new();
        middleware.extract_from_axum_headers(headers)
    }

    /// Extract context from tonic request extensions
    #[cfg(feature = "grpc-server")]
    pub fn extract_context_from_tonic_request<T>(request: &tonic::Request<T>) -> Option<Context> {
        // First try to get context from extensions (if already processed)
        if let Some(context) = request.extensions().get::<Context>() {
            return Some(context.clone());
        }

        // If not in extensions, extract from metadata
        let middleware = GrpcServerMiddleware::new();
        middleware
            .extract_from_tonic_metadata(request.metadata())
            .ok()
    }

    /// Create a child context for outgoing requests
    pub fn create_child_context_for_outgoing(parent: &Context) -> Context {
        propagation::create_child_context(parent)
    }

    /// Merge request context with current context
    pub fn merge_with_current_context(request_context: &Context) -> Context {
        if let Some(current) = Context::current() {
            propagation::merge_contexts(&current, request_context)
        } else {
            request_context.clone()
        }
    }

    /// Cross-protocol context propagation utilities
    pub mod cross_protocol {
        use super::*;

        /// Propagate context from REST to gRPC call
        #[cfg(all(feature = "rest-server", feature = "grpc-client"))]
        pub async fn propagate_rest_to_grpc<T>(
            rest_context: &Context,
            grpc_request: &mut tonic::Request<T>,
        ) -> Result<()> {
            let grpc_middleware = crate::client::middleware::GrpcClientMiddleware::new();
            grpc_middleware.inject_into_tonic_metadata(rest_context, grpc_request.metadata_mut())
        }

        /// Propagate context from gRPC to REST call
        #[cfg(all(feature = "grpc-server", feature = "rest-client"))]
        pub async fn propagate_grpc_to_rest(
            grpc_context: &Context,
            rest_headers: &mut reqwest::header::HeaderMap,
        ) -> Result<()> {
            let rest_middleware = crate::client::middleware::RestClientMiddleware::new();
            rest_middleware.inject_into_reqwest_headers(grpc_context, rest_headers)
        }

        /// Extract unified context from either REST or gRPC request
        pub fn extract_unified_context_from_request(
            rest_headers: Option<&axum::http::HeaderMap>,
            #[cfg(feature = "grpc-server")] grpc_request: Option<&tonic::metadata::MetadataMap>,
            #[cfg(not(feature = "grpc-server"))] grpc_request: Option<&()>,
        ) -> Context {
            if let Some(headers) = rest_headers {
                #[cfg(feature = "rest-server")]
                {
                    extract_context_from_axum_request(headers).unwrap_or_else(|_| Context::empty())
                }
                #[cfg(not(feature = "rest-server"))]
                {
                    Context::empty()
                }
            } else if grpc_request.is_some() {
                #[cfg(feature = "grpc-server")]
                {
                    let metadata = grpc_request.unwrap();
                    let middleware = GrpcServerMiddleware::new();
                    middleware
                        .extract_from_tonic_metadata(metadata)
                        .unwrap_or_else(|_| Context::empty())
                }
                #[cfg(not(feature = "grpc-server"))]
                {
                    Context::empty()
                }
            } else {
                Context::empty()
            }
        }

        /// Sync context between REST and gRPC servers running on same process
        pub async fn sync_server_contexts(
            rest_context: Option<&Context>,
            grpc_context: Option<&Context>,
        ) -> Context {
            match (rest_context, grpc_context) {
                (Some(rest), Some(grpc)) => {
                    // Merge contexts with gRPC taking precedence for request metadata
                    propagation::merge_contexts(rest, grpc)
                }
                (Some(rest), None) => rest.clone(),
                (None, Some(grpc)) => grpc.clone(),
                (None, None) => Context::empty(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Context;
    use chrono::Utc;
    use uuid::Uuid;

    #[cfg(feature = "rest-server")]
    #[test]
    fn test_rest_server_middleware_creation() {
        let middleware = RestServerMiddleware::new();
        assert_eq!(
            middleware
                .envelope_middleware
                .config
                .extension_header_prefix,
            "x-ext-"
        );
    }

    #[cfg(feature = "rest-server")]
    #[test]
    fn test_axum_header_adapter() {
        use axum::http::{HeaderMap, HeaderName, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-test-header"),
            HeaderValue::from_static("test-value"),
        );

        let adapter = AxumHeaderAdapter::from_headers(&headers);
        assert_eq!(adapter.get("x-test-header").unwrap(), "test-value");

        let keys = adapter.keys();
        assert!(keys.contains(&"x-test-header".to_string()));
    }

    #[cfg(feature = "rest-server")]
    #[test]
    fn test_rest_context_extraction() {
        use axum::http::{HeaderMap, HeaderName, HeaderValue};

        let middleware = RestServerMiddleware::new();
        let mut headers = HeaderMap::new();

        let request_id = Uuid::now_v7();
        headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&request_id.to_string()).unwrap(),
        );
        headers.insert(
            HeaderName::from_static("x-version"),
            HeaderValue::from_static("2.0.0"),
        );
        headers.insert(
            HeaderName::from_static("x-user-id"),
            HeaderValue::from_static("user456"),
        );

        let context = middleware.extract_from_axum_headers(&headers).unwrap();
        let meta = context.meta();

        assert_eq!(meta.request_id, Some(request_id));
        assert_eq!(meta.version.as_ref().unwrap(), "2.0.0");
        assert_eq!(
            meta.security.as_ref().unwrap().user_id.as_ref().unwrap(),
            "user456"
        );
    }

    #[cfg(feature = "rest-server")]
    #[test]
    fn test_rest_context_injection() {
        use axum::http::HeaderMap;

        let middleware = RestServerMiddleware::new();
        let mut headers = HeaderMap::new();

        let context = Context::builder()
            .request_id(Uuid::now_v7())
            .version("3.0.0")
            .build();

        middleware
            .inject_into_axum_headers(&context, &mut headers)
            .unwrap();

        assert!(headers.contains_key("x-request-id"));
        assert!(headers.contains_key("x-version"));
        assert_eq!(headers.get("x-version").unwrap().to_str().unwrap(), "3.0.0");
    }

    #[cfg(feature = "grpc-server")]
    #[test]
    fn test_grpc_server_middleware_creation() {
        let middleware = GrpcServerMiddleware::new();
        assert_eq!(
            middleware
                .envelope_middleware
                .config
                .extension_header_prefix,
            "x-ext-"
        );
    }

    #[cfg(feature = "grpc-server")]
    #[test]
    fn test_tonic_metadata_adapter() {
        use tonic::metadata::{MetadataMap, MetadataValue};

        let mut metadata = MetadataMap::new();
        metadata.insert("x-test-key", MetadataValue::from_static("test-value"));

        let adapter = TonicMetadataAdapter::from_metadata(&metadata);
        assert_eq!(adapter.get("x-test-key").unwrap(), "test-value");

        let keys = adapter.keys();
        assert!(keys.iter().any(|k| k.contains("x-test-key")));
    }

    #[cfg(feature = "grpc-server")]
    #[test]
    fn test_grpc_context_extraction() {
        use tonic::metadata::{MetadataMap, MetadataValue};

        let middleware = GrpcServerMiddleware::new();
        let mut metadata = MetadataMap::new();

        let request_id = Uuid::now_v7();
        use std::str::FromStr;
        metadata.insert(
            "x-request-id",
            MetadataValue::from_str(&request_id.to_string()).unwrap(),
        );
        metadata.insert("x-version", MetadataValue::from_str("2.0.0").unwrap());
        metadata.insert("x-trace-id", MetadataValue::from_str("trace789").unwrap());

        let context = middleware.extract_from_tonic_metadata(&metadata).unwrap();
        let meta = context.meta();

        assert_eq!(meta.request_id, Some(request_id));
        assert_eq!(meta.version.as_ref().unwrap(), "2.0.0");
        assert_eq!(
            meta.tracing.as_ref().unwrap().trace_id.as_ref().unwrap(),
            "trace789"
        );
    }

    #[cfg(feature = "grpc-server")]
    #[test]
    fn test_context_extraction_interceptor() {
        use tonic::{
            metadata::{MetadataMap, MetadataValue},
            Request,
        };

        let middleware = EnvelopeMiddleware::new();
        let mut interceptor = ContextExtractionInterceptor::new(middleware);

        let mut metadata = MetadataMap::new();
        use std::str::FromStr;
        metadata.insert("x-version", MetadataValue::from_str("1.0.0").unwrap());

        let mut request = Request::new(());
        *request.metadata_mut() = metadata;

        use tonic::service::Interceptor;
        let result = interceptor.call(request);
        assert!(result.is_ok());

        let processed_request = result.unwrap();
        // Should have context stored in extensions
        assert!(processed_request.extensions().get::<Context>().is_some());
    }

    #[test]
    fn test_utils_create_child_context() {
        let parent = Context::builder()
            .request_id(Uuid::now_v7())
            .version("parent-version")
            .build();

        let child = utils::create_child_context_for_outgoing(&parent);

        // Child should have different request ID but same version
        assert_ne!(child.meta().request_id, parent.meta().request_id);
        assert_eq!(child.meta().version, parent.meta().version);
    }

    #[test]
    fn test_utils_merge_with_current_context() {
        let request_context = Context::builder().version("request-version").build();

        // Without current context, should return request context
        let merged = utils::merge_with_current_context(&request_context);
        assert_eq!(merged.meta().version.as_ref().unwrap(), "request-version");
    }
}
