// ABOUTME: Unified tenant extraction that works across REST and gRPC servers with proper feature gates
// ABOUTME: Provides protocol-agnostic tenant extraction integration for envelope middleware

//! Unified tenant extraction for REST and gRPC servers.
//!
//! This module provides a unified interface for tenant extraction that works across
//! both REST and gRPC protocols with proper feature gates. It integrates seamlessly
//! with the existing envelope middleware system.

use crate::{
    envelope::{Context, HeaderLike},
    error::{QollectiveError, Result},
    tenant::extraction::ExtractionConfig,
    tenant::{TenantExtractionErrorHandler, TenantExtractor, TenantInfo},
};
use std::collections::HashMap;

use serde_json::Value;

/// Unified tenant extraction processor for both REST and gRPC
#[derive(Debug, Clone)]
pub struct UnifiedTenantExtractor {
    /// Core tenant extractor
    pub extractor: TenantExtractor,
    /// Error handler for secure error processing
    pub error_handler: TenantExtractionErrorHandler,
}

impl UnifiedTenantExtractor {
    /// Create new unified extractor with default configuration
    pub fn new() -> Self {
        let extraction_enabled = std::env::var("QOLLECTIVE_TENANT_EXTRACTION")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);

        Self {
            extractor: TenantExtractor::new(),
            error_handler: TenantExtractionErrorHandler::new(extraction_enabled),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ExtractionConfig) -> Self {
        let enabled = config.enabled;
        Self {
            extractor: TenantExtractor::with_config(config),
            error_handler: TenantExtractionErrorHandler::new(enabled),
        }
    }

    /// Enable or disable extraction
    pub fn set_enabled(&mut self, enabled: bool) {
        self.error_handler.set_extraction_enabled(enabled);
        self.extractor.set_enabled(enabled);
    }

    /// Process context with tenant extraction (protocol-agnostic)
    pub fn process_context_with_tenant_extraction(
        &self,
        context: &Context,
        headers: &dyn HeaderLike,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Context> {
        let mut processed_context = context.clone();

        // Extract tenant information using unified error handling
        if let Some(tenant_info) = self.extract_tenant_info(
            headers,
            payload,
            query_params,
        )? {
            self.apply_tenant_info_to_context(&mut processed_context, &tenant_info)?;
        }

        Ok(processed_context)
    }

    /// Extract tenant information using unified error handling
    fn extract_tenant_info(
        &self,
        headers: &dyn HeaderLike,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Option<TenantInfo>> {
        // Convert HeaderLike to HashMap for TenantExtractor
        let header_map = self.header_like_to_map(headers);

        // Extract authorization header for JWT processing
        let auth_header = headers
            .get("authorization")
            .or_else(|| headers.get("Authorization"));

        // Try JWT extraction first (highest priority)
        if let Some(auth_header) = auth_header {
            match self.extractor.extract_from_jwt(auth_header) {
                Ok(Some(tenant_info)) => {
                    return self
                        .error_handler
                        .handle_extraction_success(Some(tenant_info), "jwt");
                }
                Ok(None) => {
                    // No tenant in JWT, continue to other sources
                }
                Err(e) => {
                    // Handle JWT extraction error
                    let result = self.error_handler.handle_extraction_error(e, "jwt")?;
                    if result.is_some() {
                        return Ok(result);
                    }
                    // If error handler returns None, continue to other sources
                }
            }
        }

        // Try header extraction
        match self.extractor.extract_from_headers(&header_map) {
            Ok(Some(tenant_info)) => {
                return self
                    .error_handler
                    .handle_extraction_success(Some(tenant_info), "headers");
            }
            Ok(None) => {
                // No tenant in headers, continue
            }
            Err(e) => {
                let result = self.error_handler.handle_extraction_error(e, "headers")?;
                if result.is_some() {
                    return Ok(result);
                }
            }
        }

        // Try payload extraction
        if let Some(payload) = payload {
            match self.extractor.extract_from_payload(payload) {
                Ok(Some(tenant_info)) => {
                    return self
                        .error_handler
                        .handle_extraction_success(Some(tenant_info), "payload");
                }
                Ok(None) => {
                    // No tenant in payload, continue
                }
                Err(e) => {
                    let result = self.error_handler.handle_extraction_error(e, "payload")?;
                    if result.is_some() {
                        return Ok(result);
                    }
                }
            }
        }

        // Try query parameter extraction
        if let Some(query_params) = query_params {
            match self.extractor.extract_from_query_params(query_params) {
                Ok(Some(tenant_info)) => {
                    return self
                        .error_handler
                        .handle_extraction_success(Some(tenant_info), "query_params");
                }
                Ok(None) => {
                    // No tenant in query params
                }
                Err(e) => {
                    let result = self
                        .error_handler
                        .handle_extraction_error(e, "query_params")?;
                    if result.is_some() {
                        return Ok(result);
                    }
                }
            }
        }

        // No tenant found in any source
        Ok(None)
    }

    /// Apply extracted tenant information to context
    fn apply_tenant_info_to_context(
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
}

impl Default for UnifiedTenantExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// REST-specific integration with proper feature gates
#[cfg(feature = "rest-server")]
pub mod rest {
    use super::*;
    use axum::http::HeaderMap;

    /// Process REST context with tenant extraction
    pub fn process_rest_context_with_tenant_extraction(
        extractor: &UnifiedTenantExtractor,
        context: &Context,
        headers: &HeaderMap,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Context> {
        let header_adapter = RestHeaderAdapter::from_headers(headers);
        extractor.process_context_with_tenant_extraction(
            context,
            &header_adapter,
            payload,
            query_params,
        )
    }

    /// Extract query parameters from URI query string
    pub fn extract_query_params(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let decoded_key = urlencoding::decode(key).unwrap_or_else(|_| key.into());
                let decoded_value = urlencoding::decode(value).unwrap_or_else(|_| value.into());
                params.insert(decoded_key.to_string(), decoded_value.to_string());
            }
        }
        params
    }

    /// Header adapter for axum HeaderMap
    struct RestHeaderAdapter<'a> {
        headers: &'a HeaderMap,
    }

    impl<'a> RestHeaderAdapter<'a> {
        fn from_headers(headers: &'a HeaderMap) -> Self {
            Self { headers }
        }
    }

    impl<'a> HeaderLike for RestHeaderAdapter<'a> {
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

/// gRPC-specific integration with proper feature gates
#[cfg(feature = "grpc-server")]
pub mod grpc {
    use super::*;
    use tonic::metadata::MetadataMap;

    /// Process gRPC context with tenant extraction
    pub fn process_grpc_context_with_tenant_extraction(
        extractor: &UnifiedTenantExtractor,
        context: &Context,
        metadata: &MetadataMap,
    ) -> Result<Context> {
        let metadata_adapter = GrpcMetadataAdapter::from_metadata(metadata);
        extractor.process_context_with_tenant_extraction(
            context,
            &metadata_adapter,
            None, // gRPC doesn't have JSON payload extraction
            None, // gRPC doesn't have query parameters
        )
    }

    /// Metadata adapter for tonic MetadataMap
    struct GrpcMetadataAdapter<'a> {
        metadata: &'a MetadataMap,
    }

    impl<'a> GrpcMetadataAdapter<'a> {
        fn from_metadata(metadata: &'a MetadataMap) -> Self {
            Self { metadata }
        }
    }

    impl<'a> HeaderLike for GrpcMetadataAdapter<'a> {
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
    fn test_unified_extractor_creation() {
        let extractor = UnifiedTenantExtractor::new();

        // Default should respect environment variable
        // We can't directly access enabled anymore, but can test through error handler
        assert!(!extractor.error_handler.should_fail_on_error()); // Default strategy
    }

    #[test]
    fn test_unified_tenant_extraction() {
        let mut config = ExtractionConfig::default();
        config.enabled = true;
        let extractor = UnifiedTenantExtractor::with_config(config);

        let context = Context::empty();
        let mut headers = TestHeaders::new();
        headers.insert("X-Tenant-ID", "unified-tenant");

        let processed_context = extractor
            .process_context_with_tenant_extraction(
                &context,
                &headers,
                None,
                None,
            )
            .unwrap();

        assert_eq!(
            processed_context.meta().tenant,
            Some("unified-tenant".to_string())
        );
    }

    #[test]
    fn test_extraction_disabled() {
        let mut extractor = UnifiedTenantExtractor::new();
        extractor.set_enabled(false);

        let context = Context::empty();
        let mut headers = TestHeaders::new();
        headers.insert("X-Tenant-ID", "should-be-ignored");

        let processed_context = extractor
            .process_context_with_tenant_extraction(
                &context,
                &headers,
                None,
                None,
            )
            .unwrap();

        // When disabled, tenant should not be extracted
        assert!(processed_context.meta().tenant.is_none());
    }

    #[cfg(feature = "rest-server")]
    #[test]
    fn test_query_param_extraction() {
        let query = "tenant=test-tenant&user=user123&encoded%20key=encoded%20value";
        let params = rest::extract_query_params(query);

        assert_eq!(params.get("tenant"), Some(&"test-tenant".to_string()));
        assert_eq!(params.get("user"), Some(&"user123".to_string()));
        assert_eq!(
            params.get("encoded key"),
            Some(&"encoded value".to_string())
        );
    }

    #[test]
    fn test_header_like_to_map_conversion() {
        let extractor = UnifiedTenantExtractor::new();
        let mut headers = TestHeaders::new();
        headers.insert("key1", "value1");
        headers.insert("key2", "value2");

        let header_map = extractor.header_like_to_map(&headers);

        assert_eq!(header_map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(header_map.get("key2"), Some(&"value2".to_string()));
    }
}
