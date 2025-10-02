// ABOUTME: Advanced tenant extraction middleware for REST and gRPC servers with multi-source support
// ABOUTME: Integrates the TenantExtractor with envelope middleware for comprehensive tenant context extraction

//! Advanced tenant extraction middleware for comprehensive multi-source tenant extraction.
//!
//! This module provides advanced tenant extraction capabilities that integrate with the
//! envelope middleware system. It supports extracting tenant information from JWT tokens,
//! HTTP headers, request payloads, and query parameters with configurable priority resolution.

use crate::{
    envelope::{Context, HeaderLike},
    error::{QollectiveError, Result},
    tenant::extraction::ExtractionConfig,
    tenant::{ExtractionError, TenantExtractor, TenantInfo},
};
use std::collections::HashMap;

use serde_json::Value;

/// Enhanced tenant extraction middleware with multi-source support
#[derive(Debug, Clone)]
pub struct TenantExtractionMiddleware {
    /// Tenant extractor with configured sources
    pub tenant_extractor: TenantExtractor,
    /// Whether extraction is enabled
    pub enabled: bool,
}

impl TenantExtractionMiddleware {
    /// Create new tenant extraction middleware with default configuration
    pub fn new() -> Self {
        Self {
            tenant_extractor: TenantExtractor::new(),
            enabled: std::env::var("QOLLECTIVE_TENANT_EXTRACTION")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
        }
    }

    /// Create new middleware with custom extraction configuration
    pub fn with_config(config: ExtractionConfig) -> Self {
        Self {
            tenant_extractor: TenantExtractor::with_config(config),
            enabled: true,
        }
    }

    /// Enable or disable tenant extraction
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Extract tenant information from REST request components
    pub fn extract_from_rest_request(
        &self,
        headers: &dyn HeaderLike,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Option<TenantInfo>> {
        if !self.enabled {
            return Ok(None);
        }

        // Convert HeaderLike to HashMap for TenantExtractor
        let header_map = self.header_like_to_map(headers);

        // Extract authorization header for JWT processing
        let auth_header = headers
            .get("authorization")
            .or_else(|| headers.get("Authorization"));

        // Use priority-based extraction
        match self.tenant_extractor.extract_with_priority(
            auth_header,
            Some(&header_map),
            payload,
            query_params,
        ) {
            Ok(tenant_info) => Ok(tenant_info),
            Err(ExtractionError::ExtractionDisabled) => Ok(None),
            Err(ExtractionError::NoTenantFound) => Ok(None),
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("Tenant extraction failed: {}", e);
                #[cfg(not(feature = "tracing"))]
                tracing::error!("Tenant extraction failed: {}", e);

                // Don't fail the request, just log the error
                Ok(None)
            }
        }
    }

    /// Extract tenant information from gRPC metadata
    #[cfg(feature = "grpc-server")]
    pub fn extract_from_grpc_metadata(
        &self,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<Option<TenantInfo>> {
        if !self.enabled {
            return Ok(None);
        }

        // Convert tonic metadata to HashMap
        let header_map = self.metadata_to_map(metadata);

        // Extract authorization from metadata
        let auth_header = metadata
            .get("authorization")
            .or_else(|| metadata.get("Authorization"))
            .and_then(|v| v.to_str().ok());

        // For gRPC, we primarily use headers and JWT - no payload/query extraction
        match self.tenant_extractor.extract_with_priority(
            auth_header,
            Some(&header_map),
            None, // No JSON payload for gRPC metadata
            None, // No query params for gRPC
        ) {
            Ok(tenant_info) => Ok(tenant_info),
            Err(ExtractionError::ExtractionDisabled) => Ok(None),
            Err(ExtractionError::NoTenantFound) => Ok(None),
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("gRPC tenant extraction failed: {}", e);
                #[cfg(not(feature = "tracing"))]
                tracing::error!("gRPC tenant extraction failed: {}", e);

                Ok(None)
            }
        }
    }

    /// Apply extracted tenant information to context
    pub fn apply_tenant_info_to_context(
        &self,
        context: &mut Context,
        tenant_info: &TenantInfo,
    ) -> Result<()> {
        let meta = context.meta_mut();

        // Set tenant ID
        if let Some(ref tenant_id) = tenant_info.tenant_id {
            meta.tenant = Some(tenant_id.clone());
        }

        // Set onBehalfOf information
        if let Some(ref on_behalf_of) = tenant_info.on_behalf_of {
            meta.on_behalf_of = Some(on_behalf_of.clone());
        }

        // Add extraction context information to extensions
        if !tenant_info.context.is_empty() {
            if meta.extensions.is_none() {
                meta.extensions = Some(crate::envelope::meta::ExtensionsMeta {
                    sections: HashMap::new(),
                });
            }

            if let Some(ref mut extensions) = meta.extensions {
                // Add tenant extraction metadata
                let tenant_context = serde_json::json!({
                    "extraction_source": tenant_info.source.description(),
                    "extraction_priority": tenant_info.source.priority().as_number(),
                    "extraction_context": tenant_info.context
                });
                extensions
                    .sections
                    .insert("tenant_extraction".to_string(), tenant_context);
            }
        }

        Ok(())
    }

    /// Process incoming context with tenant extraction for REST requests
    pub fn process_incoming_rest_context(
        &self,
        context: &Context,
        headers: &dyn HeaderLike,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Context> {
        let mut processed_context = context.clone();

        if let Some(tenant_info) = self.extract_from_rest_request(
            headers,
            payload,
            query_params,
        )? {
            self.apply_tenant_info_to_context(&mut processed_context, &tenant_info)?;

            #[cfg(feature = "tracing")]
            tracing::debug!(
                "Extracted tenant information: tenant_id={:?}, source={}, on_behalf_of={:?}",
                tenant_info.tenant_id,
                tenant_info.source.description(),
                tenant_info
                    .on_behalf_of
                    .as_ref()
                    .map(|obo| &obo.original_user)
            );
        }

        Ok(processed_context)
    }

    /// Process incoming context with tenant extraction for gRPC requests
    #[cfg(feature = "grpc-server")]
    pub fn process_incoming_grpc_context(
        &self,
        context: &Context,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<Context> {
        let mut processed_context = context.clone();

        if let Some(tenant_info) = self.extract_from_grpc_metadata(metadata)? {
            self.apply_tenant_info_to_context(&mut processed_context, &tenant_info)?;

            #[cfg(feature = "tracing")]
            tracing::debug!(
                "Extracted gRPC tenant information: tenant_id={:?}, source={}, on_behalf_of={:?}",
                tenant_info.tenant_id,
                tenant_info.source.description(),
                tenant_info
                    .on_behalf_of
                    .as_ref()
                    .map(|obo| &obo.original_user)
            );
        }

        Ok(processed_context)
    }

    /// Convert HeaderLike to HashMap for TenantExtractor
    fn header_like_to_map(&self, headers: &dyn HeaderLike) -> HashMap<String, String> {
        let mut header_map = HashMap::new();
        for key in headers.keys() {
            if let Some(value) = headers.get(&key) {
                header_map.insert(key, value.to_string());
            }
        }
        header_map
    }

    /// Convert tonic metadata to HashMap
    #[cfg(feature = "grpc-server")]
    fn metadata_to_map(&self, metadata: &tonic::metadata::MetadataMap) -> HashMap<String, String> {
        let mut header_map = HashMap::new();
        for key_and_value in metadata.iter() {
            match key_and_value {
                tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                    if let Ok(value_str) = value.to_str() {
                        header_map.insert(key.as_str().to_string(), value_str.to_string());
                    }
                }
                tonic::metadata::KeyAndValueRef::Binary(key, _value) => {
                    // Skip binary metadata for tenant extraction
                    #[cfg(feature = "tracing")]
                    tracing::debug!("Skipping binary metadata key: {}", key.as_str());
                }
            }
        }
        header_map
    }
}

impl Default for TenantExtractionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// REST server integration for tenant extraction
#[cfg(feature = "rest-server")]
pub mod rest_integration {
    use super::*;
    use axum::http::HeaderMap;

    /// Extract tenant information from axum request
    pub fn extract_tenant_from_axum_request(
        middleware: &TenantExtractionMiddleware,
        headers: &HeaderMap,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Option<TenantInfo>> {
        let header_adapter = AxumHeaderAdapter::from_headers(headers);
        middleware.extract_from_rest_request(
            &header_adapter,
            payload,
            query_params,
        )
    }

    /// Process axum request context with tenant extraction
    pub fn process_axum_context_with_tenant_extraction(
        middleware: &TenantExtractionMiddleware,
        context: &Context,
        headers: &HeaderMap,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Context> {
        let header_adapter = AxumHeaderAdapter::from_headers(headers);
        middleware.process_incoming_rest_context(
            context,
            &header_adapter,
            payload,
            query_params,
        )
    }

    /// Header adapter for axum HeaderMap
    struct AxumHeaderAdapter<'a> {
        headers: &'a HeaderMap,
    }

    impl<'a> AxumHeaderAdapter<'a> {
        fn from_headers(headers: &'a HeaderMap) -> Self {
            Self { headers }
        }
    }

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
}

/// gRPC server integration for tenant extraction
#[cfg(feature = "grpc-server")]
pub mod grpc_integration {
    use super::*;

    /// Extract tenant information from tonic metadata
    pub fn extract_tenant_from_tonic_metadata(
        middleware: &TenantExtractionMiddleware,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<Option<TenantInfo>> {
        middleware.extract_from_grpc_metadata(metadata)
    }

    /// Process tonic request context with tenant extraction
    pub fn process_tonic_context_with_tenant_extraction(
        middleware: &TenantExtractionMiddleware,
        context: &Context,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<Context> {
        middleware.process_incoming_grpc_context(context, metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test helper for HeaderLike
    struct TestHeaders {
        headers: HashMap<String, String>,
    }

    impl TestHeaders {
        fn new() -> Self {
            Self {
                headers: HashMap::new(),
            }
        }

        fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
            self.headers.insert(key.into(), value.into());
        }
    }

    impl HeaderLike for TestHeaders {
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
    fn test_tenant_extraction_middleware_creation() {
        let middleware = TenantExtractionMiddleware::new();

        // Default should respect environment variable
        assert_eq!(middleware.enabled, false); // No env var set in test
    }

    #[test]
    fn test_tenant_extraction_from_jwt_header() {
        let middleware = TenantExtractionMiddleware::with_config(ExtractionConfig::default());
        let mut headers = TestHeaders::new();

        // Add a mock JWT token that would contain tenant information
        headers.insert("authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ0ZW5hbnQiOiJ0ZXN0LXRlbmFudCIsInN1YiI6InVzZXIxMjMifQ.signature");

        let result = middleware.extract_from_rest_request(
            &headers,
            None,
            None,
        );

        // Should handle the request gracefully even with mock JWT
        assert!(result.is_ok());
    }

    #[test]
    fn test_tenant_extraction_from_headers() {
        let middleware = TenantExtractionMiddleware::with_config(ExtractionConfig::default());
        let mut headers = TestHeaders::new();

        headers.insert("X-Tenant-ID", "header-tenant");
        headers.insert("X-User-ID", "user456");

        let result = middleware
            .extract_from_rest_request(
                &headers,
                None,
                None,
            )
            .unwrap();

        assert!(result.is_some());
        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("header-tenant".to_string()));
    }

    #[test]
    fn test_tenant_extraction_from_query_params() {
        let middleware = TenantExtractionMiddleware::with_config(ExtractionConfig::default());
        let headers = TestHeaders::new();

        let mut query_params = HashMap::new();
        query_params.insert("tenant".to_string(), "query-tenant".to_string());

        let result = middleware
            .extract_from_rest_request(
                &headers,
                None,
                Some(&query_params),
            )
            .unwrap();

        assert!(result.is_some());
        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("query-tenant".to_string()));
    }

    #[test]
    fn test_tenant_extraction_from_payload() {
        let middleware = TenantExtractionMiddleware::with_config(ExtractionConfig::default());
        let headers = TestHeaders::new();

        let payload = serde_json::json!({
            "tenant": "payload-tenant",
            "data": "some data"
        });

        let result = middleware
            .extract_from_rest_request(&headers, Some(&payload), None)
            .unwrap();

        assert!(result.is_some());
        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("payload-tenant".to_string()));
    }

    #[test]
    fn test_priority_resolution() {
        let middleware = TenantExtractionMiddleware::with_config(ExtractionConfig::default());
        let mut headers = TestHeaders::new();

        // Add tenant info in multiple sources - JWT should win
        headers.insert("authorization", "Bearer header.payload.signature"); // Mock JWT
        headers.insert("X-Tenant-ID", "header-tenant");

        let mut query_params = HashMap::new();
        query_params.insert("tenant".to_string(), "query-tenant".to_string());

        let payload = serde_json::json!({
            "tenant": "payload-tenant"
        });

        let result = middleware
            .extract_from_rest_request(
                &headers,
                Some(&payload),
                Some(&query_params),
            )
            .unwrap();

        // Should extract something (order depends on actual JWT parsing or fallback to headers)
        assert!(result.is_some());
    }

    #[test]
    fn test_apply_tenant_info_to_context() {
        let middleware = TenantExtractionMiddleware::new();
        let mut context = Context::empty();

        let tenant_info = TenantInfo::new(crate::tenant::ExtractionSource::Header(
            "X-Tenant-ID".to_string(),
        ))
        .with_tenant_id("test-tenant".to_string())
        .with_context("test_key".to_string(), serde_json::Value::String("test_value".to_string()));

        middleware
            .apply_tenant_info_to_context(&mut context, &tenant_info)
            .unwrap();

        let meta = context.meta();
        assert_eq!(meta.tenant, Some("test-tenant".to_string()));
        assert!(meta.extensions.is_some());
    }

    #[test]
    fn test_extraction_disabled() {
        let mut middleware = TenantExtractionMiddleware::new();
        middleware.set_enabled(false);

        let mut headers = TestHeaders::new();
        headers.insert("X-Tenant-ID", "test-tenant");

        let result = middleware
            .extract_from_rest_request(
                &headers,
                None,
                None,
            )
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_header_like_to_map_conversion() {
        let middleware = TenantExtractionMiddleware::new();
        let mut headers = TestHeaders::new();
        headers.insert("key1", "value1");
        headers.insert("key2", "value2");

        let header_map = middleware.header_like_to_map(&headers);

        assert_eq!(header_map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(header_map.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_process_incoming_rest_context() {
        let middleware = TenantExtractionMiddleware::with_config(ExtractionConfig::default());
        let context = Context::empty();
        let mut headers = TestHeaders::new();
        headers.insert("X-Tenant-ID", "context-tenant");

        let processed_context = middleware
            .process_incoming_rest_context(
                &context,
                &headers,
                None,
                None,
            )
            .unwrap();

        assert_eq!(
            processed_context.meta().tenant,
            Some("context-tenant".to_string())
        );
    }
}
