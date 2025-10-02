// ABOUTME: JWT tenant extraction framework with pluggable token location and parsing strategies
// ABOUTME: Provides builder pattern for easy configuration and customization of JWT processing

//! # JWT Tenant Extraction
//!
//! This module provides a flexible, pluggable framework for extracting tenant information
//! from JWT tokens in HTTP requests. It uses the builder pattern to make it easy to use
//! with sensible defaults while allowing complete customization for advanced use cases.
//!
//! ## Basic Usage
//!
//! ```rust
//! use qollective::envelope::{JwtProcessor, HttpRequest};
//!
//! // Use default implementation (Authorization Bearer header, parse-only JWT)
//! let processor = JwtProcessor::builder().build();
//!
//! let request = HttpRequest::new()
//!     .with_header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...");
//!
//! match processor.extract_from_request(&request) {
//!     Ok(Some(tenant_info)) => {
//!         println!("Tenant: {:?}", tenant_info.tenant_key);
//!         println!("On behalf of: {:?}", tenant_info.on_behalf_of);
//!     }
//!     Ok(None) => println!("No JWT token found"),
//!     Err(e) => println!("Error processing JWT: {}", e),
//! }
//! ```
//!
//! ## Custom Token Location
//!
//! ```rust
//! use qollective::envelope::{JwtProcessor, JwtTokenLocator, HttpRequest, TokenLocationError};
//!
//! // Custom locator that looks in a specific header
//! struct CustomHeaderLocator {
//!     header_name: String,
//! }
//!
//! impl JwtTokenLocator for CustomHeaderLocator {
//!     fn locate_token(&self, request: &HttpRequest) -> Result<Option<String>, TokenLocationError> {
//!         Ok(request.headers.get(&self.header_name).cloned())
//!     }
//! }
//!
//! let processor = JwtProcessor::builder()
//!     .token_locator(CustomHeaderLocator {
//!         header_name: "X-Auth-Token".to_string()
//!     })
//!     .build();
//! ```
//!
//! ## Custom JWT Processing
//!
//! ```rust
//! use qollective::envelope::{
//!     JwtProcessor, JwtTenantExtractor, JwtTenantInfo, OnBehalfOfInfo, JwtExtractionError
//! };
//!
//! // Custom extractor that validates signatures
//! struct ValidatingExtractor {
//!     secret: String,
//! }
//!
//! impl JwtTenantExtractor for ValidatingExtractor {
//!     fn extract_tenant_info(&self, token: &str) -> Result<JwtTenantInfo, JwtExtractionError> {
//!         // Your custom validation logic here
//!         // This is where you'd add signature validation, decryption, etc.
//!         
//!         // For example, using the jsonwebtoken crate:
//!         // let token_data = decode::<Claims>(token, &key, &validation)?;
//!         
//!         Ok(JwtTenantInfo {
//!             tenant_key: Some("validated-tenant".to_string()),
//!             on_behalf_of: None,
//!         })
//!     }
//! }
//!
//! let processor = JwtProcessor::builder()
//!     .tenant_extractor(ValidatingExtractor {
//!         secret: "your-secret-key".to_string()
//!     })
//!     .build();
//! ```
//!
//! ## Complete Customization
//!
//! ```rust,no_run
//! use qollective::envelope::{JwtProcessor, JwtTokenLocator, JwtTenantExtractor};
//! use qollective::envelope::{HttpRequest, TokenLocationError, JwtTenantInfo, JwtExtractionError};
//!
//! // Example custom implementations
//! struct CustomLocator;
//! impl JwtTokenLocator for CustomLocator {
//!     fn locate_token(&self, _request: &HttpRequest) -> Result<Option<String>, TokenLocationError> {
//!         // Your custom token location logic here
//!         Ok(None)
//!     }
//! }
//!
//! struct CustomExtractor;
//! impl JwtTenantExtractor for CustomExtractor {
//!     fn extract_tenant_info(&self, _token: &str) -> Result<JwtTenantInfo, JwtExtractionError> {
//!         // Your custom tenant extraction logic here
//!         Err(JwtExtractionError::InvalidStructure("not implemented".to_string()))
//!     }
//! }
//!
//! let processor = JwtProcessor::builder()
//!     .token_locator(CustomLocator)    // Your token location logic
//!     .tenant_extractor(CustomExtractor) // Your JWT processing logic
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "tenant-extraction")]
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

/// Information extracted from JWT tokens for tenant context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwtTenantInfo {
    /// Primary tenant key from the JWT payload
    pub tenant_key: Option<String>,
    /// On-behalf-of information for delegated operations
    pub on_behalf_of: Option<OnBehalfOfInfo>,
}

/// On-behalf-of information for delegated tenant operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnBehalfOfInfo {
    /// Tenant identifier for delegated operations
    pub tenant: Option<String>,
    /// User identifier for delegated operations
    pub user: Option<String>,
}

/// Generic HTTP request abstraction for token location
#[derive(Debug)]
pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            query_params: HashMap::new(),
            cookies: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    pub fn with_cookie(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.cookies.insert(key.into(), value.into());
        self
    }
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during token location
#[derive(Debug, Error)]
pub enum TokenLocationError {
    #[error("Token not found in expected location")]
    TokenNotFound,
    #[error("Invalid token format: {0}")]
    InvalidFormat(String),
    #[error("Multiple conflicting tokens found")]
    ConflictingTokens,
    #[error("Request processing error: {0}")]
    RequestError(String),
}

/// Errors that can occur during JWT tenant extraction
#[derive(Debug, Error)]
pub enum JwtExtractionError {
    #[error("Invalid JWT structure: {0}")]
    InvalidStructure(String),
    #[error("Invalid base64 encoding: {0}")]
    InvalidEncoding(String),
    #[error("Invalid JSON in payload: {0}")]
    InvalidJson(String),
    #[error("Missing required claim: {0}")]
    MissingClaim(String),
    #[error("Invalid claim type for {0}: expected {1}")]
    InvalidClaimType(String, String),
    #[error("Token processing error: {0}")]
    ProcessingError(String),
}

/// Combined error type for JWT processing operations
#[derive(Debug, Error)]
pub enum JwtProcessingError {
    #[error("Token location error: {0}")]
    Location(#[from] TokenLocationError),
    #[error("Token extraction error: {0}")]
    Extraction(#[from] JwtExtractionError),
}

/// Trait for locating JWT tokens in HTTP requests
///
/// This trait allows users to customize where and how JWT tokens are extracted
/// from incoming requests. Common implementations might look in:
/// - Authorization headers (Bearer, JWT schemes)
/// - Custom headers
/// - Query parameters
/// - Cookies
/// - Multiple locations with fallback logic
pub trait JwtTokenLocator: Send + Sync {
    /// Locate and extract a JWT token from the HTTP request
    ///
    /// # Arguments
    /// * `request` - The HTTP request to search for tokens
    ///
    /// # Returns
    /// * `Ok(Some(token))` - Token found and extracted
    /// * `Ok(None)` - No token found (not an error for optional auth)
    /// * `Err(error)` - Error occurred during token location
    fn locate_token(&self, request: &HttpRequest) -> Result<Option<String>, TokenLocationError>;
}

/// Trait for extracting tenant information from JWT tokens
///
/// This trait allows users to customize how JWT tokens are parsed and validated.
/// The framework provides a default parse-only implementation, but users can
/// implement their own for:
/// - JWT signature validation
/// - Encrypted JWT (JWE) decryption
/// - Custom claim structures
/// - Token introspection against auth servers
/// - Multi-step validation workflows
pub trait JwtTenantExtractor: Send + Sync {
    /// Extract tenant information from a JWT token
    ///
    /// # Arguments
    /// * `token` - The JWT token string to process
    ///
    /// # Returns
    /// * `Ok(info)` - Successfully extracted tenant information
    /// * `Err(error)` - Error occurred during token processing
    fn extract_tenant_info(&self, token: &str) -> Result<JwtTenantInfo, JwtExtractionError>;
}

/// Main JWT processor that combines token location and tenant extraction
///
/// This struct uses the builder pattern to allow easy configuration of both
/// token location and extraction strategies. Users can customize either or both
/// components while keeping sensible defaults.
#[derive(Clone)]
pub struct JwtProcessor {
    token_locator: Arc<dyn JwtTokenLocator>,
    tenant_extractor: Arc<dyn JwtTenantExtractor>,
}

impl JwtProcessor {
    /// Create a new builder for configuring JWT processing
    pub fn builder() -> JwtProcessorBuilder {
        JwtProcessorBuilder::new()
    }

    /// Extract tenant information from an HTTP request
    ///
    /// This method combines token location and tenant extraction in a single call,
    /// making it easy to process requests end-to-end.
    ///
    /// # Arguments
    /// * `request` - The HTTP request to process
    ///
    /// # Returns
    /// * `Ok(Some(info))` - Token found and tenant info extracted
    /// * `Ok(None)` - No token found (not an error for optional auth)
    /// * `Err(error)` - Error occurred during processing
    pub fn extract_from_request(
        &self,
        request: &HttpRequest,
    ) -> Result<Option<JwtTenantInfo>, JwtProcessingError> {
        match self.token_locator.locate_token(request)? {
            Some(token) => {
                let tenant_info = self.tenant_extractor.extract_tenant_info(&token)?;
                Ok(Some(tenant_info))
            }
            None => Ok(None),
        }
    }

    /// Extract tenant information directly from a JWT token string
    ///
    /// This method bypasses token location and processes the token directly.
    ///
    /// # Arguments
    /// * `token` - The JWT token string to process
    ///
    /// # Returns
    /// * `Ok(info)` - Successfully extracted tenant information
    /// * `Err(error)` - Error occurred during token processing
    pub fn extract_from_token(&self, token: &str) -> Result<JwtTenantInfo, JwtExtractionError> {
        self.tenant_extractor.extract_tenant_info(token)
    }
}

/// Builder for configuring JWT processor instances
pub struct JwtProcessorBuilder {
    token_locator: Option<Arc<dyn JwtTokenLocator>>,
    tenant_extractor: Option<Arc<dyn JwtTenantExtractor>>,
}

impl JwtProcessorBuilder {
    /// Create a new builder with no custom configurations
    pub fn new() -> Self {
        Self {
            token_locator: None,
            tenant_extractor: None,
        }
    }

    /// Set a custom token locator
    ///
    /// # Arguments
    /// * `locator` - Custom implementation of token location logic
    pub fn token_locator<T: JwtTokenLocator + 'static>(mut self, locator: T) -> Self {
        self.token_locator = Some(Arc::new(locator));
        self
    }

    /// Set a custom tenant extractor
    ///
    /// # Arguments
    /// * `extractor` - Custom implementation of tenant extraction logic
    pub fn tenant_extractor<T: JwtTenantExtractor + 'static>(mut self, extractor: T) -> Self {
        self.tenant_extractor = Some(Arc::new(extractor));
        self
    }

    /// Build the JWT processor with the configured components
    ///
    /// Any components not explicitly set will use the default implementations.
    pub fn build(self) -> JwtProcessor {
        let token_locator = self
            .token_locator
            .unwrap_or_else(|| Arc::new(DefaultJwtTokenLocator));

        let tenant_extractor = self
            .tenant_extractor
            .unwrap_or_else(|| Arc::new(DefaultJwtTenantExtractor));

        JwtProcessor {
            token_locator,
            tenant_extractor,
        }
    }
}

impl Default for JwtProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Default implementation that looks for JWT tokens in Authorization header
/// with Bearer scheme (e.g., "Authorization: Bearer &lt;token&gt;")
#[derive(Debug, Clone)]
pub struct DefaultJwtTokenLocator;

impl JwtTokenLocator for DefaultJwtTokenLocator {
    fn locate_token(&self, request: &HttpRequest) -> Result<Option<String>, TokenLocationError> {
        // Try case-insensitive lookup for Authorization header
        let auth_header = request
            .headers
            .get("authorization")
            .or_else(|| request.headers.get("Authorization"));

        if let Some(auth_value) = auth_header {
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                if token.trim().is_empty() {
                    return Err(TokenLocationError::InvalidFormat(
                        "Bearer token is empty".to_string(),
                    ));
                }
                return Ok(Some(token.to_string()));
            }

            if let Some(token) = auth_value.strip_prefix("JWT ") {
                if token.trim().is_empty() {
                    return Err(TokenLocationError::InvalidFormat(
                        "JWT token is empty".to_string(),
                    ));
                }
                return Ok(Some(token.to_string()));
            }
        }

        Ok(None)
    }
}

/// Default implementation that parses JWT tokens without signature validation
///
/// This implementation:
/// - Extracts `tenantkey` or `tenant_key` from the payload
/// - Extracts `onBehalfOf` object with optional `tenant` and `user` fields
/// - Does NOT validate JWT signatures (parse-only)
/// - Handles malformed JWTs gracefully with clear error messages
#[derive(Debug, Clone)]
pub struct DefaultJwtTenantExtractor;

#[cfg(feature = "tenant-extraction")]
impl JwtTenantExtractor for DefaultJwtTenantExtractor {
    fn extract_tenant_info(&self, token: &str) -> Result<JwtTenantInfo, JwtExtractionError> {
        // Split JWT into parts (header.payload.signature)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JwtExtractionError::InvalidStructure(format!(
                "JWT must have 3 parts separated by dots, found {}",
                parts.len()
            )));
        }

        // Decode the payload (second part)
        let payload_b64 = parts[1];

        // JWT uses base64url encoding which may not have padding
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|e| JwtExtractionError::InvalidEncoding(e.to_string()))?;

        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|e| JwtExtractionError::InvalidEncoding(format!("UTF-8 error: {}", e)))?;

        // Parse JSON payload
        let payload: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| JwtExtractionError::InvalidJson(e.to_string()))?;

        // Extract tenant key (try both "tenantkey" and "tenant_key")
        let tenant_key = payload
            .get("tenantkey")
            .or_else(|| payload.get("tenant_key"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract onBehalfOf information
        let on_behalf_of = if let Some(on_behalf_obj) = payload.get("onBehalfOf") {
            let tenant = on_behalf_obj
                .get("tenant")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let user = on_behalf_obj
                .get("user")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(OnBehalfOfInfo { tenant, user })
        } else {
            None
        };

        Ok(JwtTenantInfo {
            tenant_key,
            on_behalf_of,
        })
    }
}

#[cfg(not(feature = "tenant-extraction"))]
impl JwtTenantExtractor for DefaultJwtTenantExtractor {
    fn extract_tenant_info(&self, _token: &str) -> Result<JwtTenantInfo, JwtExtractionError> {
        Err(JwtExtractionError::ProcessingError(
            "JWT tenant extraction feature not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_builder() {
        let request = HttpRequest::new()
            .with_header("authorization", "Bearer token123")
            .with_query_param("user", "test")
            .with_cookie("session", "abc");

        assert_eq!(
            request.headers.get("authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(request.query_params.get("user"), Some(&"test".to_string()));
        assert_eq!(request.cookies.get("session"), Some(&"abc".to_string()));
    }

    #[test]
    fn test_jwt_processor_builder_defaults() {
        let processor = JwtProcessor::builder().build();

        // Should not panic - verifies that defaults are properly set
        let request = HttpRequest::new().with_header("Authorization", "Bearer test.token.here");

        // This will fail with invalid JWT, but should not panic
        let result = processor.extract_from_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_token_locator_bearer() {
        let locator = DefaultJwtTokenLocator;
        let request = HttpRequest::new().with_header("Authorization", "Bearer mytoken123");

        let result = locator.locate_token(&request).unwrap();
        assert_eq!(result, Some("mytoken123".to_string()));
    }

    #[test]
    fn test_default_token_locator_jwt_scheme() {
        let locator = DefaultJwtTokenLocator;
        let request = HttpRequest::new().with_header("Authorization", "JWT mytoken123");

        let result = locator.locate_token(&request).unwrap();
        assert_eq!(result, Some("mytoken123".to_string()));
    }

    #[test]
    fn test_default_token_locator_no_token() {
        let locator = DefaultJwtTokenLocator;
        let request = HttpRequest::new().with_header("Content-Type", "application/json");

        let result = locator.locate_token(&request).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_default_token_locator_empty_bearer() {
        let locator = DefaultJwtTokenLocator;
        let request = HttpRequest::new().with_header("Authorization", "Bearer ");

        let result = locator.locate_token(&request);
        assert!(matches!(result, Err(TokenLocationError::InvalidFormat(_))));
    }

    #[test]
    fn test_default_token_locator_case_insensitive() {
        let locator = DefaultJwtTokenLocator;
        let request = HttpRequest::new().with_header("authorization", "Bearer lowercaseheader");

        let result = locator.locate_token(&request).unwrap();
        assert_eq!(result, Some("lowercaseheader".to_string()));
    }

    // Test custom token locator implementation
    #[derive(Debug)]
    struct CustomHeaderLocator {
        header_name: String,
    }

    impl CustomHeaderLocator {
        fn new(header_name: impl Into<String>) -> Self {
            Self {
                header_name: header_name.into(),
            }
        }
    }

    impl JwtTokenLocator for CustomHeaderLocator {
        fn locate_token(
            &self,
            request: &HttpRequest,
        ) -> Result<Option<String>, TokenLocationError> {
            Ok(request.headers.get(&self.header_name).cloned())
        }
    }

    #[test]
    fn test_custom_token_locator() {
        let locator = CustomHeaderLocator::new("X-Auth-Token");
        let request = HttpRequest::new().with_header("X-Auth-Token", "custom-token-123");

        let result = locator.locate_token(&request).unwrap();
        assert_eq!(result, Some("custom-token-123".to_string()));
    }

    #[test]
    fn test_jwt_processor_with_custom_locator() {
        let custom_locator = CustomHeaderLocator::new("X-Custom-JWT");
        let processor = JwtProcessor::builder()
            .token_locator(custom_locator)
            .build();

        let request = HttpRequest::new().with_header("X-Custom-JWT", "invalid.jwt.token");

        // Should find the token but fail on parsing
        let result = processor.extract_from_request(&request);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JwtProcessingError::Extraction(_)
        ));
    }

    // Test custom tenant extractor implementation
    #[derive(Debug)]
    struct MockTenantExtractor {
        tenant_key: String,
    }

    impl MockTenantExtractor {
        fn new(tenant_key: impl Into<String>) -> Self {
            Self {
                tenant_key: tenant_key.into(),
            }
        }
    }

    impl JwtTenantExtractor for MockTenantExtractor {
        fn extract_tenant_info(&self, _token: &str) -> Result<JwtTenantInfo, JwtExtractionError> {
            Ok(JwtTenantInfo {
                tenant_key: Some(self.tenant_key.clone()),
                on_behalf_of: None,
            })
        }
    }

    #[test]
    fn test_jwt_processor_with_custom_extractor() {
        let mock_extractor = MockTenantExtractor::new("test-tenant");
        let processor = JwtProcessor::builder()
            .tenant_extractor(mock_extractor)
            .build();

        let request = HttpRequest::new().with_header("Authorization", "Bearer any-token");

        let result = processor.extract_from_request(&request).unwrap();
        assert!(result.is_some());

        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_key, Some("test-tenant".to_string()));
        assert_eq!(tenant_info.on_behalf_of, None);
    }

    #[test]
    fn test_jwt_processor_with_both_custom_components() {
        let custom_locator = CustomHeaderLocator::new("X-JWT-Token");
        let mock_extractor = MockTenantExtractor::new("custom-tenant");

        let processor = JwtProcessor::builder()
            .token_locator(custom_locator)
            .tenant_extractor(mock_extractor)
            .build();

        let request = HttpRequest::new().with_header("X-JWT-Token", "some-jwt-token");

        let result = processor.extract_from_request(&request).unwrap();
        assert!(result.is_some());

        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_key, Some("custom-tenant".to_string()));
    }

    #[test]
    fn test_jwt_processor_no_token_found() {
        let processor = JwtProcessor::builder().build();
        let request = HttpRequest::new().with_header("Content-Type", "application/json");

        let result = processor.extract_from_request(&request).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_jwt_processor_extract_from_token_directly() {
        let mock_extractor = MockTenantExtractor::new("direct-tenant");
        let processor = JwtProcessor::builder()
            .tenant_extractor(mock_extractor)
            .build();

        let result = processor.extract_from_token("any-token").unwrap();
        assert_eq!(result.tenant_key, Some("direct-tenant".to_string()));
    }

    #[cfg(feature = "tenant-extraction")]
    mod jwt_extraction_tests {
        use super::*;

        fn create_test_jwt(payload: &str) -> String {
            // Create a test JWT with header.payload.signature format
            let header = r#"{"alg":"HS256","typ":"JWT"}"#;
            let header_b64 = URL_SAFE_NO_PAD.encode(header.as_bytes());
            let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_bytes());
            let signature = "fake_signature";

            format!("{}.{}.{}", header_b64, payload_b64, signature)
        }

        #[test]
        fn test_default_extractor_with_tenant_key() {
            let extractor = DefaultJwtTenantExtractor;
            let payload = r#"{"tenantkey":"test-tenant-123","other":"data"}"#;
            let jwt = create_test_jwt(payload);

            let result = extractor.extract_tenant_info(&jwt).unwrap();
            assert_eq!(result.tenant_key, Some("test-tenant-123".to_string()));
            assert_eq!(result.on_behalf_of, None);
        }

        #[test]
        fn test_default_extractor_with_tenant_key_underscore() {
            let extractor = DefaultJwtTenantExtractor;
            let payload = r#"{"tenant_key":"test-tenant-456","other":"data"}"#;
            let jwt = create_test_jwt(payload);

            let result = extractor.extract_tenant_info(&jwt).unwrap();
            assert_eq!(result.tenant_key, Some("test-tenant-456".to_string()));
            assert_eq!(result.on_behalf_of, None);
        }

        #[test]
        fn test_default_extractor_with_on_behalf_of() {
            let extractor = DefaultJwtTenantExtractor;
            let payload = r#"{"onBehalfOf":{"tenant":"delegation-tenant","user":"john.doe"}}"#;
            let jwt = create_test_jwt(payload);

            let result = extractor.extract_tenant_info(&jwt).unwrap();
            assert_eq!(result.tenant_key, None);
            assert!(result.on_behalf_of.is_some());

            let on_behalf = result.on_behalf_of.unwrap();
            assert_eq!(on_behalf.tenant, Some("delegation-tenant".to_string()));
            assert_eq!(on_behalf.user, Some("john.doe".to_string()));
        }

        #[test]
        fn test_default_extractor_with_both_claims() {
            let extractor = DefaultJwtTenantExtractor;
            let payload = r#"{"tenantkey":"primary-tenant","onBehalfOf":{"tenant":"delegate-tenant","user":"admin"}}"#;
            let jwt = create_test_jwt(payload);

            let result = extractor.extract_tenant_info(&jwt).unwrap();
            assert_eq!(result.tenant_key, Some("primary-tenant".to_string()));
            assert!(result.on_behalf_of.is_some());

            let on_behalf = result.on_behalf_of.unwrap();
            assert_eq!(on_behalf.tenant, Some("delegate-tenant".to_string()));
            assert_eq!(on_behalf.user, Some("admin".to_string()));
        }

        #[test]
        fn test_default_extractor_no_tenant_claims() {
            let extractor = DefaultJwtTenantExtractor;
            let payload = r#"{"sub":"user123","iss":"auth-service","exp":1234567890}"#;
            let jwt = create_test_jwt(payload);

            let result = extractor.extract_tenant_info(&jwt).unwrap();
            assert_eq!(result.tenant_key, None);
            assert_eq!(result.on_behalf_of, None);
        }

        #[test]
        fn test_default_extractor_invalid_jwt_structure() {
            let extractor = DefaultJwtTenantExtractor;

            let result = extractor.extract_tenant_info("invalid.jwt");
            assert!(matches!(
                result,
                Err(JwtExtractionError::InvalidStructure(_))
            ));
        }

        #[test]
        fn test_default_extractor_invalid_base64() {
            let extractor = DefaultJwtTenantExtractor;

            let result = extractor.extract_tenant_info("header.invalid-base64!.signature");
            assert!(matches!(
                result,
                Err(JwtExtractionError::InvalidEncoding(_))
            ));
        }

        #[test]
        fn test_default_extractor_invalid_json() {
            let extractor = DefaultJwtTenantExtractor;
            let invalid_payload = URL_SAFE_NO_PAD.encode(b"not-json");
            let jwt = format!("header.{}.signature", invalid_payload);

            let result = extractor.extract_tenant_info(&jwt);
            assert!(matches!(result, Err(JwtExtractionError::InvalidJson(_))));
        }
    }
}
