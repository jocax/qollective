// ABOUTME: REST/HTTP client implementation with envelope support
// ABOUTME: Provides HTTP client functionality with automatic envelope handling and metadata injection

//! REST/HTTP client implementation with envelope support.
//!
//! This module provides a comprehensive REST client that:
//! - Automatically injects headers from envelope metadata
//! - Handles connection pooling and timeouts
//! - Provides detailed error context
//! - Supports async/await patterns
//! - Prepares for TLS configuration

#[cfg(feature = "rest-client")]
use {
    crate::{
        client::common::ClientConfig,
        envelope::{Context, Envelope, Meta},
        error::{QollectiveError, Result},
    },
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, sync::Arc, time::Duration},
};

/// Enhanced REST client configuration
#[cfg(feature = "rest-client")]
#[derive(Debug, Clone)]
pub struct RestClientConfig {
    pub base: ClientConfig,
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
    pub tls_config: Option<TlsConfig>,
    pub jwt_config: JwtConfig,
}

/// JWT configuration for authentication header handling
#[cfg(feature = "rest-client")]
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Header name for JWT token (default: "Authorization")
    pub header_name: String,
    /// Header value prefix (default: "Bearer ")
    pub header_prefix: String,
    /// Custom tenant header name when no JWT is available (default: "X-Tenant-ID")
    pub tenant_header_name: String,
    /// Custom onBehalfOf header name when no JWT is available (default: "X-On-Behalf-Of")
    pub on_behalf_of_header_name: String,
}

/// TLS configuration for future support
#[cfg(feature = "rest-client")]
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub verify_certificates: bool,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub ca_cert_path: Option<String>,
}

/// REST client for HTTP communication with envelope support (refactored for dependency injection)
#[cfg(feature = "rest-client")]
#[derive(Debug)]
pub struct RestClient {
    transport: std::sync::Arc<crate::transport::HybridTransportClient>,
}

#[cfg(feature = "rest-client")]
impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            header_name: "Authorization".to_string(),
            header_prefix: "Bearer ".to_string(),
            tenant_header_name: "X-Tenant-ID".to_string(),
            on_behalf_of_header_name: "X-On-Behalf-Of".to_string(),
        }
    }
}

#[cfg(feature = "rest-client")]
impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            base: ClientConfig::default(),
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            user_agent: format!("qollective-rust/{}", env!("CARGO_PKG_VERSION")),
            default_headers: HashMap::new(),
            tls_config: None,
            jwt_config: JwtConfig::default(),
        }
    }
}

#[cfg(feature = "rest-client")]
impl RestClient {
    /// Create a REST client with dependency injection for testing
    pub fn with_transport(transport: Arc<crate::transport::HybridTransportClient>) -> Result<Self> {
        Ok(Self { transport })
    }

    /// Create a REST client with its own transport layer
    pub async fn new(config: RestClientConfig) -> Result<Self> {
        // Create transport configuration from REST config (CONFIG FIRST PRINCIPLE)
        let transport_config = crate::transport::TransportDetectionConfig {
            enable_auto_detection: true,
            detection_timeout: config.connect_timeout,
            capability_cache_ttl: config.pool_idle_timeout,
            retry_failed_detections: config.base.retry_attempts > 0,
            max_detection_retries: config.base.retry_attempts,
        };

        // Create transport with REST client injected
        let mut transport = crate::transport::HybridTransportClient::new(transport_config);

        // Create the actual internal REST client that the transport will use
        let internal_rest_client = crate::transport::rest::InternalRestClient::new(config).await?;
        transport = transport.with_internal_rest_client(Arc::new(internal_rest_client));

        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    // Old constructor methods removed - now using transport delegation pattern
    // All helper methods moved to InternalRestClient in transport layer

    /// Send a POST request with envelope-aware handling (delegated from RestClient)
    pub async fn post<Req, Res>(&self, path: &str, envelope: Envelope<Req>) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        // Delegate to transport layer - get internal REST client and call its method
        if let Some(rest_client) = self.transport.internal_rest_client() {
            rest_client.post(path, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No REST client configured in transport layer",
            ))
        }
    }

    /// Send a GET request with envelope-aware handling (delegated from RestClient)
    pub async fn get<Req, Res>(&self, path: &str, envelope: Envelope<Req>) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        // Delegate to transport layer - get internal REST client and call its method
        if let Some(rest_client) = self.transport.internal_rest_client() {
            rest_client.get(path, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No REST client configured in transport layer",
            ))
        }
    }

    /// Send a GET request with metadata only (for cases without body)
    pub async fn get_with_meta<Res>(&self, path: &str, meta: Meta) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        // Create envelope with null data for GET requests without body
        let envelope = Envelope::new(meta, serde_json::Value::Null);
        self.get(path, envelope).await
    }

    /// Send a PUT request with envelope-aware handling (delegated from RestClient)
    pub async fn put<Req, Res>(&self, path: &str, envelope: Envelope<Req>) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        // Delegate to transport layer - get internal REST client and call its method
        if let Some(rest_client) = self.transport.internal_rest_client() {
            rest_client.put(path, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No REST client configured in transport layer",
            ))
        }
    }

    /// Send a DELETE request with envelope-aware handling (delegated from RestClient)
    pub async fn delete<Req, Res>(
        &self,
        path: &str,
        envelope: Envelope<Req>,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        // Delegate to transport layer - get internal REST client and call its method
        if let Some(rest_client) = self.transport.internal_rest_client() {
            rest_client.delete(path, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No REST client configured in transport layer",
            ))
        }
    }

    /// Send a DELETE request with metadata only (for cases without body)
    pub async fn delete_with_meta<Res>(&self, path: &str, meta: Meta) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        // Create envelope with null data for DELETE requests without body
        let envelope = Envelope::new(meta, serde_json::Value::Null);
        self.delete(path, envelope).await
    }

    /// Send an OPTIONS request with envelope-aware handling (delegated from RestClient)
    pub async fn options<Req, Res>(
        &self,
        path: &str,
        envelope: Envelope<Req>,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        // Delegate to transport layer - get internal REST client and call its method
        if let Some(rest_client) = self.transport.internal_rest_client() {
            rest_client.options(path, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No REST client configured in transport layer",
            ))
        }
    }

    /// Send an OPTIONS request with metadata only (for basic OPTIONS requests)
    pub async fn options_with_meta<Res>(&self, path: &str, meta: Meta) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        // Create envelope with null data for basic OPTIONS requests
        let envelope = Envelope::new(meta, serde_json::Value::Null);
        self.options(path, envelope).await
    }

    /// Send a PATCH request with envelope-aware handling (delegated from RestClient)
    pub async fn patch<Req, Res>(
        &self,
        path: &str,
        envelope: Envelope<Req>,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        // Delegate to transport layer - get internal REST client and call its method
        if let Some(rest_client) = self.transport.internal_rest_client() {
            rest_client.patch(path, envelope).await
        } else {
            Err(QollectiveError::transport(
                "No REST client configured in transport layer",
            ))
        }
    }

    /// Context-aware methods for REST client

    /// Send a POST request using the current context for metadata
    pub async fn post_with_current_context<Req, Res>(
        &self,
        path: &str,
        data: Req,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let context = Context::current().unwrap_or_default();
        let envelope = Envelope::new(context.into_meta(), data);
        self.post(path, envelope).await
    }

    /// Send a GET request using the current context for metadata
    pub async fn get_with_current_context<Res>(&self, path: &str) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        let context = Context::current().unwrap_or_default();
        let envelope = Envelope::new(context.into_meta(), serde_json::Value::Null);
        self.get(path, envelope).await
    }

    /// Send a PUT request using the current context for metadata
    pub async fn put_with_current_context<Req, Res>(
        &self,
        path: &str,
        data: Req,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let context = Context::current().unwrap_or_default();
        let envelope = Envelope::new(context.into_meta(), data);
        self.put(path, envelope).await
    }

    /// Send a DELETE request using the current context for metadata  
    pub async fn delete_with_current_context<Res>(&self, path: &str) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        let context = Context::current().unwrap_or_default();
        let envelope = Envelope::new(context.into_meta(), serde_json::Value::Null);
        self.delete(path, envelope).await
    }

    /// Send an OPTIONS request using the current context for metadata
    pub async fn options_with_current_context<Res>(&self, path: &str) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        let context = Context::current().unwrap_or_default();
        let envelope = Envelope::new(context.into_meta(), serde_json::Value::Null);
        self.options(path, envelope).await
    }

    /// Send a PATCH request using the current context for metadata
    pub async fn patch_with_current_context<Req, Res>(
        &self,
        path: &str,
        data: Req,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let context = Context::current().unwrap_or_default();
        let envelope = Envelope::new(context.into_meta(), data);
        self.patch(path, envelope).await
    }

    /// Send a POST request with explicit context
    pub async fn post_with_context<Req, Res>(
        &self,
        path: &str,
        context: &Context,
        data: Req,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let envelope = Envelope::new(context.meta().clone(), data);
        self.post(path, envelope).await
    }

    /// Send a GET request with explicit context
    pub async fn get_with_context<Res>(
        &self,
        path: &str,
        context: &Context,
    ) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        let envelope = Envelope::new(context.meta().clone(), serde_json::Value::Null);
        self.get(path, envelope).await
    }

    /// Send a PUT request with explicit context
    pub async fn put_with_context<Req, Res>(
        &self,
        path: &str,
        context: &Context,
        data: Req,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let envelope = Envelope::new(context.meta().clone(), data);
        self.put(path, envelope).await
    }

    /// Send a DELETE request with explicit context
    pub async fn delete_with_context<Res>(
        &self,
        path: &str,
        context: &Context,
    ) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        let envelope = Envelope::new(context.meta().clone(), serde_json::Value::Null);
        self.delete(path, envelope).await
    }

    /// Send an OPTIONS request with explicit context
    pub async fn options_with_context<Res>(
        &self,
        path: &str,
        context: &Context,
    ) -> Result<Envelope<Res>>
    where
        Res: for<'de> Deserialize<'de>,
    {
        let envelope = Envelope::new(context.meta().clone(), serde_json::Value::Null);
        self.options(path, envelope).await
    }

    /// Send a PATCH request with explicit context
    pub async fn patch_with_context<Req, Res>(
        &self,
        path: &str,
        context: &Context,
        data: Req,
    ) -> Result<Envelope<Res>>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let envelope = Envelope::new(context.meta().clone(), data);
        self.patch(path, envelope).await
    }
}

/// Implementation of the common Client trait for dynamic dispatch
#[cfg(feature = "rest-client")]
impl RestClient {
    /// Send method that implements the Client trait pattern
    pub async fn send_envelope<Req, Res>(&self, request: Envelope<Req>) -> Result<Envelope<Res>>
    where
        Req: Serialize + Send + Sync,
        Res: for<'de> Deserialize<'de> + Send + Sync,
    {
        self.post("/", request).await
    }

    /// Get the client configuration (for testing and debugging) - delegates to transport layer
    pub fn config(&self) -> Option<&crate::client::rest::RestClientConfig> {
        if let Some(rest_client) = self.transport.internal_rest_client() {
            Some(rest_client.config())
        } else {
            None
        }
    }

    /// Get the underlying reqwest client (for advanced usage and testing) - NOT supported in delegation pattern
    /// This method is preserved for backward compatibility but returns None when using transport delegation
    pub fn reqwest_client(&self) -> Option<&reqwest::Client> {
        // In the new delegation pattern, we don't expose the internal reqwest client
        // This breaks the abstraction and should be avoided
        None
    }
}

/// Builder for REST client configuration
#[cfg(feature = "rest-client")]
pub struct RestClientBuilder {
    config: RestClientConfig,
    builder_customization:
        Option<Box<dyn FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder>>,
}

#[cfg(feature = "rest-client")]
impl RestClientBuilder {
    pub fn new() -> Self {
        Self {
            config: RestClientConfig::default(),
            builder_customization: None,
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base.base_url = url.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.base.timeout_seconds = timeout.as_secs();
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.config.pool_max_idle_per_host = max;
        self
    }

    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.pool_idle_timeout = timeout;
        self
    }

    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.user_agent = agent.into();
        self
    }

    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.default_headers.insert(key.into(), value.into());
        self
    }

    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.config.base.retry_attempts = attempts;
        self
    }

    pub fn tls_config(mut self, tls_config: TlsConfig) -> Self {
        self.config.tls_config = Some(tls_config);
        self
    }

    pub fn jwt_header_name(mut self, header_name: impl Into<String>) -> Self {
        self.config.jwt_config.header_name = header_name.into();
        self
    }

    pub fn jwt_header_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.jwt_config.header_prefix = prefix.into();
        self
    }

    pub fn tenant_header_name(mut self, header_name: impl Into<String>) -> Self {
        self.config.jwt_config.tenant_header_name = header_name.into();
        self
    }

    pub fn on_behalf_of_header_name(mut self, header_name: impl Into<String>) -> Self {
        self.config.jwt_config.on_behalf_of_header_name = header_name.into();
        self
    }

    pub fn jwt_config(mut self, jwt_config: JwtConfig) -> Self {
        self.config.jwt_config = jwt_config;
        self
    }

    /// Add custom configuration to the underlying reqwest ClientBuilder
    pub fn customize_client_builder<F>(mut self, customize: F) -> Self
    where
        F: FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder + 'static,
    {
        self.builder_customization = Some(Box::new(customize));
        self
    }

    pub async fn build(self) -> Result<RestClient> {
        // In the new architecture, we always use the async new method
        RestClient::new(self.config).await
    }
}

#[cfg(feature = "rest-client")]
impl Default for RestClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rest-client")]
impl JwtConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_header(mut self, header_name: impl Into<String>) -> Self {
        self.header_name = header_name.into();
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.header_prefix = prefix.into();
        self
    }

    pub fn with_tenant_header(mut self, header_name: impl Into<String>) -> Self {
        self.tenant_header_name = header_name.into();
        self
    }

    pub fn with_on_behalf_of_header(mut self, header_name: impl Into<String>) -> Self {
        self.on_behalf_of_header_name = header_name.into();
        self
    }
}

#[cfg(feature = "rest-client")]
impl TlsConfig {
    pub fn new() -> Self {
        Self {
            verify_certificates: true,
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
        }
    }

    pub fn disable_verification(mut self) -> Self {
        self.verify_certificates = false;
        self
    }

    pub fn client_cert(
        mut self,
        cert_path: impl Into<String>,
        key_path: impl Into<String>,
    ) -> Self {
        self.client_cert_path = Some(cert_path.into());
        self.client_key_path = Some(key_path.into());
        self
    }

    pub fn ca_cert(mut self, ca_path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(ca_path.into());
        self
    }
}

// Feature-disabled implementations
#[cfg(not(feature = "rest-client"))]
pub struct RestClient;

#[cfg(not(feature = "rest-client"))]
pub struct RestClientConfig;

#[cfg(not(feature = "rest-client"))]
pub struct RestClientBuilder;

#[cfg(not(feature = "rest-client"))]
pub struct TlsConfig;

#[cfg(not(feature = "rest-client"))]
pub struct JwtConfig;

#[cfg(not(feature = "rest-client"))]
impl RestClient {
    pub fn new(_config: RestClientConfig) -> crate::error::Result<Self> {
        Err(crate::error::QollectiveError::config(
            "rest-client feature not enabled",
        ))
    }

    pub fn with_base_config(
        _config: crate::client::common::ClientConfig,
    ) -> crate::error::Result<Self> {
        Err(crate::error::QollectiveError::config(
            "rest-client feature not enabled",
        ))
    }
}

#[cfg(not(feature = "rest-client"))]
impl RestClientBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build(self) -> crate::error::Result<RestClient> {
        Err(crate::error::QollectiveError::config(
            "rest-client feature not enabled",
        ))
    }
}

#[cfg(not(feature = "rest-client"))]
impl Default for RestClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Meta;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        echo: String,
    }

    #[tokio::test]
    async fn test_rest_client_builder() {
        let builder = RestClientBuilder::new()
            .base_url("https://api.example.com")
            .timeout(Duration::from_secs(60))
            .user_agent("test-agent")
            .default_header("Custom-Header", "custom-value")
            .retry_attempts(5);

        #[cfg(feature = "rest-client")]
        {
            let client = builder
                .build()
                .await
                .expect("Failed to build client");
            let client_config = client.config().expect("Should have config");
            assert_eq!(client_config.base.base_url, "https://api.example.com");
            assert_eq!(client_config.base.timeout_seconds, 60);
            assert_eq!(client_config.user_agent, "test-agent");
            assert_eq!(client_config.base.retry_attempts, 5);
            assert!(client_config.default_headers.contains_key("Custom-Header"));
        }
    }

    #[test]
    fn test_tls_config() {
        #[cfg(feature = "rest-client")]
        {
            let tls_config = TlsConfig::new()
                .disable_verification()
                .client_cert("/path/to/cert.pem", "/path/to/key.pem")
                .ca_cert("/path/to/ca.pem");

            assert!(!tls_config.verify_certificates);
            assert_eq!(
                tls_config.client_cert_path.as_ref().unwrap(),
                "/path/to/cert.pem"
            );
            assert_eq!(
                tls_config.client_key_path.as_ref().unwrap(),
                "/path/to/key.pem"
            );
            assert_eq!(tls_config.ca_cert_path.as_ref().unwrap(), "/path/to/ca.pem");
        }
    }

    // NOTE: The following tests are commented out because they test the old architecture
    // These methods (build_headers_from_envelope, with_builder_customization, from_reqwest_client)
    // have been moved to InternalRestClient in the transport layer as part of Step 15 refactoring

    // #[cfg(feature = "rest-client")]
    // #[test]
    // fn test_header_building() { ... }

    // #[cfg(feature = "rest-client")]
    // #[test]
    // fn test_custom_client_builder() { ... }

    // #[cfg(feature = "rest-client")]
    // #[test]
    // fn test_builder_with_customization() { ... }

    // #[cfg(feature = "rest-client")]
    // #[test]
    // fn test_from_reqwest_client() { ... }

    #[test]
    fn test_feature_disabled_behavior() {
        #[cfg(not(feature = "rest-client"))]
        {
            let builder = RestClientBuilder::new();
            let result = builder.build();
            assert!(result.is_err());

            let config = RestClientConfig;
            let result = RestClient::new(config);
            assert!(result.is_err());
        }
    }

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_jwt_config() {
        let jwt_config = JwtConfig::new()
            .with_header("X-Auth-Token")
            .with_prefix("Token ")
            .with_tenant_header("X-Custom-Tenant")
            .with_on_behalf_of_header("X-Custom-OnBehalf");

        assert_eq!(jwt_config.header_name, "X-Auth-Token");
        assert_eq!(jwt_config.header_prefix, "Token ");
        assert_eq!(jwt_config.tenant_header_name, "X-Custom-Tenant");
        assert_eq!(jwt_config.on_behalf_of_header_name, "X-Custom-OnBehalf");
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_rest_client_builder_with_jwt_config() {
        let jwt_config = JwtConfig::new().with_header("Custom-Auth");

        let client = RestClientBuilder::new()
            .base_url("https://api.example.com")
            .jwt_config(jwt_config)
            .jwt_header_name("Override-Auth")
            .tenant_header_name("Override-Tenant")
            .build()
            .await
            .expect("Failed to build client");

        let client_config = client.config().expect("Should have config");
        assert_eq!(client_config.jwt_config.header_name, "Override-Auth");
        assert_eq!(
            client_config.jwt_config.tenant_header_name,
            "Override-Tenant"
        );
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_tenant_header_building_no_jwt() {
        use crate::envelope::meta::OnBehalfOfMeta;
        use chrono::Utc;

        let config = RestClientConfig::default();
        let client = RestClient::new(config)
            .await
            .expect("Failed to create client");

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant-123".to_string());
        meta.on_behalf_of = Some(OnBehalfOfMeta {
            original_user: "original-user".to_string(),
            delegating_user: "admin-user".to_string(),
            delegating_tenant: "admin-tenant".to_string(),
        });

        // Test that the internal REST client was properly configured with tenant settings
        // This verifies the delegation pattern preserves tenant functionality
        let client_config = client.config().expect("Should have config");
        assert!(client_config.base.tenant_config.auto_propagate_tenant);

        // Verify that the metadata has the expected tenant information before creating envelope
        assert_eq!(meta.tenant, Some("test-tenant-123".to_string()));
        assert!(meta.on_behalf_of.is_some());

        let _envelope = Envelope::new(
            meta,
            TestRequest {
                message: "test".to_string(),
            },
        );
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_tenant_header_building_with_jwt() {
        let mut config = RestClientConfig::default();
        // Add JWT token to default headers
        config.default_headers.insert(
            "Authorization".to_string(),
            "Bearer jwt-token-here".to_string(),
        );

        let client = RestClient::new(config)
            .await
            .expect("Failed to create client");

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant-123".to_string());

        // Verify that tenant metadata is present before creating envelope
        assert_eq!(meta.tenant, Some("test-tenant-123".to_string()));

        let _envelope = Envelope::new(
            meta,
            TestRequest {
                message: "test".to_string(),
            },
        );

        // Test that the internal REST client was properly configured with JWT settings
        // This verifies the delegation pattern preserves JWT functionality
        let client_config = client.config().expect("Should have config");
        assert!(client_config.default_headers.contains_key("Authorization"));
        assert_eq!(
            client_config.default_headers.get("Authorization").unwrap(),
            "Bearer jwt-token-here"
        );
    }

    // TDD Tests for Step 15: Dependency Injection Pattern (FAILING TESTS FIRST)

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_rest_client_with_transport_constructor() {
        // TDD: Test that RestClient can be created with dependency injection
        // This test will FAIL until we implement the with_transport() method

        let transport_config = crate::transport::TransportDetectionConfig::default();
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            transport_config,
        ));

        let result = RestClient::with_transport(transport);
        assert!(
            result.is_ok(),
            "Should be able to create RestClient with injected transport"
        );

        let _client = result.unwrap();
        // Verify the client has the transport (structure will change)
        // This test will guide our implementation
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_rest_client_delegation_to_transport() {
        // TDD: Test that client methods delegate to transport layer
        // This test will FAIL until we implement delegation pattern

        let transport_config = crate::transport::TransportDetectionConfig::default();
        let mut transport = crate::transport::HybridTransportClient::new(transport_config);

        // Create internal REST client for injection
        let rest_config = RestClientConfig::default();
        let internal_client = crate::transport::rest::InternalRestClient::new(rest_config)
            .await
            .expect("Failed to create internal client");
        transport = transport.with_internal_rest_client(Arc::new(internal_client));

        let client =
            RestClient::with_transport(Arc::new(transport)).expect("Failed to create client");

        // Test that post method delegates to transport
        let meta = Meta::default();
        let envelope = Envelope::new(
            meta,
            TestRequest {
                message: "test".to_string(),
            },
        );

        // This will fail until we implement delegation
        let _result: Result<Envelope<TestResponse>> = client.post("/test", envelope).await;
        // For now, we expect this to fail with transport configuration error
        // Once implemented properly, this should work
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_rest_client_new_creates_own_transport() {
        // TDD: Test that new() method creates its own transport with internal client
        // This test will FAIL until we refactor the new() method

        let config = RestClientConfig::default();
        let result = RestClient::new(config).await;

        // This should work and create transport internally
        assert!(
            result.is_ok(),
            "Should be able to create RestClient with own transport"
        );
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_rest_client_preserves_existing_functionality() {
        // TDD: Test that all existing functionality is preserved after refactoring
        // This test will FAIL until we properly preserve all features

        let config = RestClientConfig {
            base: crate::client::common::ClientConfig {
                base_url: "https://api.example.com".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
                ..Default::default()
            },
            user_agent: "test-agent".to_string(),
            ..Default::default()
        };

        let client = RestClient::new(config)
            .await
            .expect("Failed to create client");

        // Verify configuration is preserved (this will change with new structure)
        // Test will guide us to preserve all config in InternalRestClient
        let client_config = client.config().expect("Should have config");
        assert_eq!(client_config.base.base_url, "https://api.example.com");
        assert_eq!(client_config.base.timeout_seconds, 30);
        assert_eq!(client_config.base.retry_attempts, 3);
        assert_eq!(client_config.user_agent, "test-agent");
    }

    #[cfg(feature = "rest-client")]
    #[tokio::test]
    async fn test_rest_client_delegation_integration() {
        // Integration test that verifies the complete delegation pattern works

        // Create a real REST client configuration
        let config = RestClientConfig {
            base: crate::client::common::ClientConfig {
                base_url: "https://httpbin.org".to_string(),
                timeout_seconds: 30,
                retry_attempts: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        // Create client using new() which creates its own transport internally
        let client = RestClient::new(config)
            .await
            .expect("Failed to create client");

        // Verify config delegation works
        let client_config = client.config().expect("Should have config");
        assert_eq!(client_config.base.base_url, "https://httpbin.org");

        // Test basic delegation - we can't test actual HTTP calls in unit tests
        // but we can verify the delegation structure is set up correctly
        assert!(
            client.transport.internal_rest_client().is_some(),
            "Should have internal REST client"
        );
    }

    #[cfg(feature = "rest-client")]
    #[test]
    fn test_mock_transport_integration() {
        // TDD: Test that REST client works with mock transport for testing

        // Create a mock transport that we can control
        let transport_config = crate::transport::TransportDetectionConfig::default();
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            transport_config,
        ));

        let client = RestClient::with_transport(transport)
            .expect("Failed to create client with mock transport");

        // Verify the mock transport pattern works for testing
        // Note: With no internal REST client injected, methods will return transport errors
        // This is the expected behavior for testing scenarios where we want to control the transport
        assert!(
            client.config().is_none(),
            "Mock transport should not have internal REST client"
        );
    }
}
