// ABOUTME: Multi-source tenant extraction from JWT, headers, payload, and query parameters
// ABOUTME: Implements priority-based resolution with JWT taking precedence over other sources

//! Tenant extraction from multiple sources with priority resolution.
//!
//! This module implements the core tenant extraction logic that can pull tenant
//! and onBehalfOf information from JWT tokens, HTTP headers, request payloads,
//! and query parameters. JWT tokens have the highest priority, followed by headers,
//! payload, and query parameters.

use super::{ExtractionPriority, JwtParseError, JwtParser};
use crate::envelope::meta::OnBehalfOfMeta;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during tenant extraction
#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("JWT parsing failed: {0}")]
    JwtError(#[from] JwtParseError),

    #[error("tenant extraction disabled in configuration")]
    ExtractionDisabled,

    #[error("no tenant found in any source")]
    NoTenantFound,

    #[error("invalid JSON in payload: {0}")]
    InvalidJson(String),

    #[error("missing authorization header")]
    MissingAuthHeader,

    #[error("invalid authorization header format")]
    InvalidAuthHeaderFormat,

    #[error("configuration error: {0}")]
    ConfigError(String),
}

/// Source of extracted tenant information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionSource {
    /// Extracted from JWT token
    Jwt,
    /// Extracted from HTTP header
    Header(String),
    /// Extracted from request payload/body
    Payload(String),
    /// Extracted from query parameter
    QueryParam(String),
}

impl ExtractionSource {
    /// Get the priority of this extraction source
    pub fn priority(&self) -> ExtractionPriority {
        match self {
            Self::Jwt => ExtractionPriority::Jwt,
            Self::Header(_) => ExtractionPriority::Header,
            Self::Payload(_) => ExtractionPriority::Payload,
            Self::QueryParam(_) => ExtractionPriority::QueryParam,
        }
    }

    /// Get a human-readable description of the source
    pub fn description(&self) -> String {
        match self {
            Self::Jwt => "JWT token".to_string(),
            Self::Header(name) => format!("header '{}'", name),
            Self::Payload(path) => format!("payload field '{}'", path),
            Self::QueryParam(name) => format!("query parameter '{}'", name),
        }
    }
}

/// Extracted tenant information with source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Tenant ID
    pub tenant_id: Option<String>,
    /// OnBehalfOf metadata
    pub on_behalf_of: Option<OnBehalfOfMeta>,
    /// Source where the information was found
    pub source: ExtractionSource,
    /// Additional context from extraction
    pub context: HashMap<String, serde_json::Value>,
}

impl TenantInfo {
    /// Create new tenant info
    pub fn new(source: ExtractionSource) -> Self {
        Self {
            tenant_id: None,
            on_behalf_of: None,
            source,
            context: HashMap::new(),
        }
    }

    /// Set tenant ID
    pub fn with_tenant_id(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set onBehalfOf metadata
    pub fn with_on_behalf_of(mut self, on_behalf_of: OnBehalfOfMeta) -> Self {
        self.on_behalf_of = Some(on_behalf_of);
        self
    }

    /// Add context information
    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }

    /// Check if this info has any tenant data
    pub fn has_tenant_data(&self) -> bool {
        self.tenant_id.is_some() || self.on_behalf_of.is_some()
    }
}

/// Configuration for tenant extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Whether tenant extraction is enabled
    pub enabled: bool,
    /// JWT parser configuration
    pub jwt_debug_logging: bool,
    /// Header names to check for tenant information
    pub tenant_header_names: Vec<String>,
    /// JSON paths to check in payload for tenant information
    pub tenant_payload_paths: Vec<String>,
    /// Query parameter names to check for tenant information
    pub tenant_query_params: Vec<String>,
    /// Authorization header patterns (e.g., "Bearer", "JWT")
    pub auth_header_patterns: Vec<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_debug_logging: false,
            tenant_header_names: vec![
                "X-Tenant-ID".to_string(),
                "X-Organization-ID".to_string(),
                "Tenant-ID".to_string(),
                "Organization".to_string(),
            ],
            tenant_payload_paths: vec![
                "tenant".to_string(),
                "tenant_id".to_string(),
                "organization".to_string(),
                "org_id".to_string(),
            ],
            tenant_query_params: vec![
                "tenant".to_string(),
                "tenant_id".to_string(),
                "organization".to_string(),
                "org_id".to_string(),
            ],
            auth_header_patterns: vec!["Bearer".to_string(), "JWT".to_string()],
        }
    }
}

/// Multi-source tenant extractor
#[derive(Debug, Clone)]
pub struct TenantExtractor {
    config: ExtractionConfig,
    jwt_parser: JwtParser,
}

impl TenantExtractor {
    /// Create a new tenant extractor with default configuration
    pub fn new() -> Self {
        let config = ExtractionConfig::default();
        let jwt_parser = if config.jwt_debug_logging {
            JwtParser::with_debug_logging()
        } else {
            JwtParser::new()
        };

        Self { config, jwt_parser }
    }

    /// Create a new tenant extractor with custom configuration
    pub fn with_config(config: ExtractionConfig) -> Self {
        let jwt_parser = if config.jwt_debug_logging {
            JwtParser::with_debug_logging()
        } else {
            JwtParser::new()
        };

        Self { config, jwt_parser }
    }

    /// Enable or disable tenant extraction
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Check if extraction is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Extract tenant information from authorization header (JWT token)
    pub fn extract_from_jwt(
        &self,
        auth_header: &str,
    ) -> Result<Option<TenantInfo>, ExtractionError> {
        if !self.config.enabled {
            return Err(ExtractionError::ExtractionDisabled);
        }

        // Try to extract JWT token from authorization header
        let token = self.extract_jwt_token(auth_header)?;

        match self.jwt_parser.parse_claims(&token) {
            Ok(claims) => {
                let mut tenant_info = TenantInfo::new(ExtractionSource::Jwt);

                // Extract tenant ID
                if let Some(tenant_id) = claims.extract_tenant_id() {
                    tenant_info = tenant_info.with_tenant_id(tenant_id);
                }

                // Extract onBehalfOf information from JWT
                // In JWT context:
                // - on_behalf_of claim contains the original user being acted for
                // - JWT subject (sub) is the delegating user performing the action
                // - JWT tenant is the delegating user's tenant
                if let Some(original_user) = claims.extract_on_behalf_of() {
                    // The JWT subject is the delegating user
                    if let Some(delegating_user) = claims.sub.clone() {
                        // The JWT tenant is the delegating tenant
                        // Use "unknown" as fallback if tenant cannot be determined
                        let delegating_tenant = claims.extract_tenant_id()
                            .unwrap_or_else(|| "unknown".to_string());

                        let on_behalf_of = OnBehalfOfMeta {
                            original_user,
                            delegating_user,
                            delegating_tenant,
                        };
                        tenant_info = tenant_info.with_on_behalf_of(on_behalf_of);
                    }
                }

                // Add JWT context information
                if let Some(iss) = claims.iss {
                    tenant_info = tenant_info.with_context("jwt_issuer".to_string(), serde_json::Value::String(iss));
                }
                if let Some(sub) = claims.sub {
                    tenant_info = tenant_info.with_context("jwt_subject".to_string(), serde_json::Value::String(sub));
                }

                Ok(if tenant_info.has_tenant_data() {
                    Some(tenant_info)
                } else {
                    None
                })
            }
            Err(e) => {
                // Log JWT parsing failures but don't fail the extraction
                #[cfg(feature = "tracing")]
                tracing::debug!("JWT parsing failed, will try other sources: {}", e);

                Ok(None)
            }
        }
    }

    /// Extract tenant information from HTTP headers
    pub fn extract_from_headers(
        &self,
        headers: &HashMap<String, String>,
    ) -> Result<Option<TenantInfo>, ExtractionError> {
        if !self.config.enabled {
            return Err(ExtractionError::ExtractionDisabled);
        }

        for header_name in &self.config.tenant_header_names {
            // Try case-insensitive header lookup
            let header_value = headers
                .iter()
                .find(|(k, _)| k.to_lowercase() == header_name.to_lowercase())
                .map(|(_, v)| v);

            if let Some(value) = header_value {
                if !value.trim().is_empty() {
                    let tenant_info =
                        TenantInfo::new(ExtractionSource::Header(header_name.clone()))
                            .with_tenant_id(value.trim().to_string())
                            .with_context("header_name".to_string(), serde_json::Value::String(header_name.clone()));

                    return Ok(Some(tenant_info));
                }
            }
        }

        Ok(None)
    }

    /// Extract tenant information from request payload
    pub fn extract_from_payload(
        &self,
        payload: &Value,
    ) -> Result<Option<TenantInfo>, ExtractionError> {
        if !self.config.enabled {
            return Err(ExtractionError::ExtractionDisabled);
        }

        for path in &self.config.tenant_payload_paths {
            if let Some(value) = self.get_json_value_by_path(payload, path) {
                if let Some(tenant_id) = value.as_str() {
                    if !tenant_id.trim().is_empty() {
                        let tenant_info = TenantInfo::new(ExtractionSource::Payload(path.clone()))
                            .with_tenant_id(tenant_id.trim().to_string())
                            .with_context("payload_path".to_string(), serde_json::Value::String(path.clone()));

                        return Ok(Some(tenant_info));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Extract tenant information from query parameters
    pub fn extract_from_query_params(
        &self,
        query_params: &HashMap<String, String>,
    ) -> Result<Option<TenantInfo>, ExtractionError> {
        if !self.config.enabled {
            return Err(ExtractionError::ExtractionDisabled);
        }

        for param_name in &self.config.tenant_query_params {
            if let Some(value) = query_params.get(param_name) {
                if !value.trim().is_empty() {
                    let tenant_info =
                        TenantInfo::new(ExtractionSource::QueryParam(param_name.clone()))
                            .with_tenant_id(value.trim().to_string())
                            .with_context("param_name".to_string(), serde_json::Value::String(param_name.clone()));

                    return Ok(Some(tenant_info));
                }
            }
        }

        Ok(None)
    }

    /// Extract JWT token from authorization header
    fn extract_jwt_token(&self, auth_header: &str) -> Result<String, ExtractionError> {
        let auth_header = auth_header.trim();

        for pattern in &self.config.auth_header_patterns {
            let prefix = format!("{} ", pattern);
            if auth_header.starts_with(&prefix) {
                let token = auth_header[prefix.len()..].trim();
                if self.jwt_parser.is_valid_jwt_format(token) {
                    return Ok(token.to_string());
                }
            }
        }

        // If no pattern matches but header looks like a JWT, try it directly
        if self.jwt_parser.is_valid_jwt_format(auth_header) {
            return Ok(auth_header.to_string());
        }

        Err(ExtractionError::InvalidAuthHeaderFormat)
    }

    /// Get JSON value by simple path (dot notation)
    fn get_json_value_by_path<'a>(&self, payload: &'a Value, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = payload;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Extract tenant information from all sources with priority resolution
    pub fn extract_with_priority(
        &self,
        auth_header: Option<&str>,
        headers: Option<&HashMap<String, String>>,
        payload: Option<&Value>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Option<TenantInfo>, ExtractionError> {
        if !self.config.enabled {
            return Err(ExtractionError::ExtractionDisabled);
        }

        let mut results = Vec::new();

        // Try JWT extraction first (highest priority)
        if let Some(auth_header) = auth_header {
            if let Ok(Some(tenant_info)) = self.extract_from_jwt(auth_header) {
                results.push(tenant_info);
            }
        }

        // Try header extraction
        if let Some(headers) = headers {
            if let Ok(Some(tenant_info)) = self.extract_from_headers(headers) {
                results.push(tenant_info);
            }
        }

        // Try payload extraction
        if let Some(payload) = payload {
            if let Ok(Some(tenant_info)) = self.extract_from_payload(payload) {
                results.push(tenant_info);
            }
        }

        // Try query parameter extraction
        if let Some(query_params) = query_params {
            if let Ok(Some(tenant_info)) = self.extract_from_query_params(query_params) {
                results.push(tenant_info);
            }
        }

        // Return the highest priority result
        results.sort_by_key(|info| info.source.priority());
        Ok(results.into_iter().next())
    }
}

impl Default for TenantExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_source_priority() {
        assert_eq!(ExtractionSource::Jwt.priority(), ExtractionPriority::Jwt);
        assert_eq!(
            ExtractionSource::Header("test".to_string()).priority(),
            ExtractionPriority::Header
        );
        assert_eq!(
            ExtractionSource::Payload("test".to_string()).priority(),
            ExtractionPriority::Payload
        );
        assert_eq!(
            ExtractionSource::QueryParam("test".to_string()).priority(),
            ExtractionPriority::QueryParam
        );
    }

    #[test]
    fn test_extraction_source_description() {
        assert_eq!(ExtractionSource::Jwt.description(), "JWT token");
        assert_eq!(
            ExtractionSource::Header("X-Tenant-ID".to_string()).description(),
            "header 'X-Tenant-ID'"
        );
        assert_eq!(
            ExtractionSource::Payload("tenant.id".to_string()).description(),
            "payload field 'tenant.id'"
        );
        assert_eq!(
            ExtractionSource::QueryParam("tenant_id".to_string()).description(),
            "query parameter 'tenant_id'"
        );
    }

    #[test]
    fn test_tenant_info_builder() {
        let info = TenantInfo::new(ExtractionSource::Jwt)
            .with_tenant_id("tenant123".to_string())
            .with_context("source".to_string(), serde_json::Value::String("test".to_string()));

        assert_eq!(info.tenant_id, Some("tenant123".to_string()));
        assert_eq!(info.source, ExtractionSource::Jwt);
        assert_eq!(info.context.get("source"), Some(&serde_json::Value::String("test".to_string())));
        assert!(info.has_tenant_data());
    }

    #[test]
    fn test_tenant_info_has_tenant_data() {
        let info_empty = TenantInfo::new(ExtractionSource::Jwt);
        assert!(!info_empty.has_tenant_data());

        let info_with_tenant =
            TenantInfo::new(ExtractionSource::Jwt).with_tenant_id("tenant123".to_string());
        assert!(info_with_tenant.has_tenant_data());

        let info_with_on_behalf_of =
            TenantInfo::new(ExtractionSource::Jwt).with_on_behalf_of(OnBehalfOfMeta {
                original_user: "user123".to_string(),
                delegating_user: "admin456".to_string(),
                delegating_tenant: "admin-tenant".to_string(),
            });
        assert!(info_with_on_behalf_of.has_tenant_data());
    }

    #[test]
    fn test_extraction_config_default() {
        let config = ExtractionConfig::default();
        assert!(config.enabled);
        assert!(!config.jwt_debug_logging);
        assert!(!config.tenant_header_names.is_empty());
        assert!(!config.tenant_payload_paths.is_empty());
        assert!(!config.tenant_query_params.is_empty());
        assert!(!config.auth_header_patterns.is_empty());
    }

    #[test]
    fn test_tenant_extractor_creation() {
        let extractor = TenantExtractor::new();
        assert!(extractor.config.enabled);

        let custom_config = ExtractionConfig {
            enabled: false,
            jwt_debug_logging: true,
            ..Default::default()
        };
        let extractor_custom = TenantExtractor::with_config(custom_config);
        assert!(!extractor_custom.config.enabled);
    }

    #[test]
    fn test_extract_from_headers() {
        let extractor = TenantExtractor::new();
        let mut headers = HashMap::new();
        headers.insert("X-Tenant-ID".to_string(), "tenant123".to_string());
        headers.insert("Other-Header".to_string(), "value".to_string());

        let result = extractor.extract_from_headers(&headers).unwrap();
        assert!(result.is_some());

        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("tenant123".to_string()));
        assert_eq!(
            tenant_info.source,
            ExtractionSource::Header("X-Tenant-ID".to_string())
        );
    }

    #[test]
    fn test_extract_from_headers_case_insensitive() {
        let extractor = TenantExtractor::new();
        let mut headers = HashMap::new();
        headers.insert("x-tenant-id".to_string(), "tenant123".to_string()); // lowercase

        let result = extractor.extract_from_headers(&headers).unwrap();
        assert!(result.is_some());

        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("tenant123".to_string()));
    }

    #[test]
    fn test_extract_from_query_params() {
        let extractor = TenantExtractor::new();
        let mut query_params = HashMap::new();
        query_params.insert("tenant".to_string(), "tenant456".to_string());
        query_params.insert("other_param".to_string(), "value".to_string());

        let result = extractor.extract_from_query_params(&query_params).unwrap();
        assert!(result.is_some());

        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("tenant456".to_string()));
        assert_eq!(
            tenant_info.source,
            ExtractionSource::QueryParam("tenant".to_string())
        );
    }

    #[test]
    fn test_extract_from_payload() {
        let extractor = TenantExtractor::new();
        let payload = serde_json::json!({
            "tenant": "tenant789",
            "other_data": "value"
        });

        let result = extractor.extract_from_payload(&payload).unwrap();
        assert!(result.is_some());

        let tenant_info = result.unwrap();
        assert_eq!(tenant_info.tenant_id, Some("tenant789".to_string()));
        assert_eq!(
            tenant_info.source,
            ExtractionSource::Payload("tenant".to_string())
        );
    }

    #[test]
    fn test_get_json_value_by_path() {
        let extractor = TenantExtractor::new();
        let payload = serde_json::json!({
            "user": {
                "tenant": "nested_tenant"
            },
            "tenant": "root_tenant"
        });

        let root_value = extractor.get_json_value_by_path(&payload, "tenant");
        assert_eq!(root_value, Some(&Value::String("root_tenant".to_string())));

        let nested_value = extractor.get_json_value_by_path(&payload, "user.tenant");
        assert_eq!(
            nested_value,
            Some(&Value::String("nested_tenant".to_string()))
        );

        let missing_value = extractor.get_json_value_by_path(&payload, "missing.path");
        assert_eq!(missing_value, None);
    }

    #[test]
    fn test_extract_jwt_token() {
        let extractor = TenantExtractor::new();

        // Test Bearer token format
        let result = extractor.extract_jwt_token("Bearer header.payload.signature");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "header.payload.signature");

        // Test JWT token format
        let result = extractor.extract_jwt_token("JWT header.payload.signature");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "header.payload.signature");

        // Test direct JWT format
        let result = extractor.extract_jwt_token("header.payload.signature");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "header.payload.signature");

        // Test invalid format
        let result = extractor.extract_jwt_token("Bearer invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_extraction_disabled() {
        let config = ExtractionConfig {
            enabled: false,
            ..Default::default()
        };
        let extractor = TenantExtractor::with_config(config);

        let result = extractor.extract_from_headers(&HashMap::new());
        assert!(matches!(result, Err(ExtractionError::ExtractionDisabled)));
    }
}
