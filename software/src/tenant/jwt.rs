// ABOUTME: JWT token parsing for tenant and onBehalfOf extraction (parse-only, no signature validation)
// ABOUTME: Provides safe JWT claims extraction without cryptographic verification for upstream validation

//! JWT parsing functionality for tenant extraction.
//!
//! This module provides parse-only JWT processing to extract tenant and onBehalfOf
//! information from JWT tokens. No signature validation is performed - tokens are
//! parsed as-is to extract claims. This design assumes that signature validation
//! occurs elsewhere in the system (upstream gateways, downstream services, etc.).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[cfg(feature = "tenant-extraction")]
use jsonwebtoken::{decode_header, Algorithm, DecodingKey, Validation};

use serde_json::Value;

/// Errors that can occur during JWT parsing
#[derive(Debug, Error)]
pub enum JwtParseError {
    #[error("invalid JWT format: {0}")]
    InvalidFormat(String),

    #[error("failed to decode JWT header: {0}")]
    HeaderDecodeError(String),

    #[error("failed to decode JWT payload: {0}")]
    PayloadDecodeError(String),

    #[error("missing required claim: {0}")]
    MissingClaim(String),

    #[error("invalid claim type for field '{field}': expected {expected}, got {actual}")]
    InvalidClaimType {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("JWT parsing feature not enabled")]
    FeatureNotEnabled,
}

/// JWT claims structure for tenant extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Standard JWT claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// Tenant-specific claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// OnBehalfOf delegation claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate_for: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub acting_as: Option<String>,

    /// Additional arbitrary claims
    #[serde(flatten)]
    pub additional_claims: HashMap<String, Value>,
}

impl JwtClaims {
    /// Extract tenant ID from claims using multiple possible field names
    pub fn extract_tenant_id(&self) -> Option<String> {
        // Priority order: tenant > tenant_id > organization > org_id
        self.tenant
            .clone()
            .or_else(|| self.tenant_id.clone())
            .or_else(|| self.organization.clone())
            .or_else(|| self.org_id.clone())
            .or_else(|| {
                // Check additional claims for common tenant field names
                let tenant_fields = ["tenantId", "organizationId", "companyId", "clientId"];
                for field in &tenant_fields {
                    if let Some(value) = self.additional_claims.get(*field) {
                        if let Some(s) = value.as_str() {
                            return Some(s.to_string());
                        }
                    }
                }
                None
            })
    }

    /// Extract onBehalfOf information from claims
    pub fn extract_on_behalf_of(&self) -> Option<String> {
        // Priority order: on_behalf_of > delegate_for > acting_as
        self.on_behalf_of
            .clone()
            .or_else(|| self.delegate_for.clone())
            .or_else(|| self.acting_as.clone())
            .or_else(|| {
                // Check additional claims for common delegation field names
                let delegation_fields = ["onBehalfOf", "delegateFor", "actingAs", "impersonating"];
                for field in &delegation_fields {
                    if let Some(value) = self.additional_claims.get(*field) {
                        if let Some(s) = value.as_str() {
                            return Some(s.to_string());
                        }
                    }
                }
                None
            })
    }
}

/// JWT parser for tenant extraction
#[derive(Debug, Clone)]
pub struct JwtParser {
    /// Whether to log parsing attempts for debugging
    pub debug_logging: bool,
}

impl JwtParser {
    /// Create a new JWT parser
    pub fn new() -> Self {
        Self {
            debug_logging: false,
        }
    }

    /// Create a new JWT parser with debug logging enabled
    pub fn with_debug_logging() -> Self {
        Self {
            debug_logging: true,
        }
    }

    /// Parse a JWT token without signature validation
    ///
    /// This method extracts claims from a JWT token by decoding the payload
    /// without performing signature verification. This is intentional as the
    /// system assumes signature validation occurs elsewhere.
    pub fn parse_claims(&self, token: &str) -> Result<JwtClaims, JwtParseError> {
        #[cfg(not(feature = "tenant-extraction"))]
        {
            return Err(JwtParseError::FeatureNotEnabled);
        }

        #[cfg(feature = "tenant-extraction")]
        {
            // Validate basic JWT format (three parts separated by dots)
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                return Err(JwtParseError::InvalidFormat(format!(
                    "expected 3 parts, got {}",
                    parts.len()
                )));
            }

            if self.debug_logging {
                tracing::debug!("Parsing JWT token with {} parts", parts.len());
            }

            // Decode without signature verification
            // We use a dummy key and disable signature validation
            let mut validation = Validation::new(Algorithm::HS256);
            validation.insecure_disable_signature_validation();
            validation.validate_exp = false; // Don't validate expiration
            validation.validate_nbf = false; // Don't validate not-before
            validation.validate_aud = false; // Don't validate audience

            // Use a dummy key since we're not validating signatures
            let dummy_key = DecodingKey::from_secret(&[]);

            match jsonwebtoken::decode::<JwtClaims>(token, &dummy_key, &validation) {
                Ok(token_data) => {
                    if self.debug_logging {
                        let tenant_id = token_data.claims.extract_tenant_id();
                        let on_behalf_of = token_data.claims.extract_on_behalf_of();
                        tracing::debug!(
                            "Successfully parsed JWT claims: tenant_id={:?}, on_behalf_of={:?}",
                            tenant_id,
                            on_behalf_of
                        );
                    }
                    Ok(token_data.claims)
                }
                Err(e) => {
                    if self.debug_logging {
                        tracing::warn!("Failed to parse JWT claims: {}", e);
                    }
                    Err(JwtParseError::PayloadDecodeError(e.to_string()))
                }
            }
        }
    }

    /// Extract only the header from a JWT token (useful for debugging)
    pub fn parse_header(&self, token: &str) -> Result<HashMap<String, Value>, JwtParseError> {
        #[cfg(not(feature = "tenant-extraction"))]
        {
            return Err(JwtParseError::FeatureNotEnabled);
        }

        #[cfg(feature = "tenant-extraction")]
        {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                return Err(JwtParseError::InvalidFormat(format!(
                    "expected 3 parts, got {}",
                    parts.len()
                )));
            }

            match decode_header(token) {
                Ok(header) => {
                    let mut header_map = HashMap::new();

                    // Convert header fields to Value
                    header_map.insert(
                        "alg".to_string(),
                        Value::String(format!("{:?}", header.alg)),
                    );
                    if let Some(typ) = header.typ {
                        header_map.insert("typ".to_string(), Value::String(typ));
                    }
                    if let Some(kid) = header.kid {
                        header_map.insert("kid".to_string(), Value::String(kid));
                    }

                    Ok(header_map)
                }
                Err(e) => Err(JwtParseError::HeaderDecodeError(e.to_string())),
            }
        }
    }

    /// Validate that a token looks like a JWT without full parsing
    pub fn is_valid_jwt_format(&self, token: &str) -> bool {
        // Basic format check: three parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() == 3 && !parts.iter().any(|part| part.is_empty())
    }
}

impl Default for JwtParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_claims_extract_tenant_id() {
        // Test with tenant field
        let claims = JwtClaims {
            tenant: Some("tenant123".to_string()),
            tenant_id: Some("tenant456".to_string()),
            organization: Some("org789".to_string()),
            org_id: Some("org000".to_string()),
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            on_behalf_of: None,
            delegate_for: None,
            acting_as: None,
            additional_claims: HashMap::new(),
        };

        // Should prioritize 'tenant' field
        assert_eq!(claims.extract_tenant_id(), Some("tenant123".to_string()));

        // Test with only tenant_id
        let claims = JwtClaims {
            tenant: None,
            tenant_id: Some("tenant456".to_string()),
            organization: Some("org789".to_string()),
            org_id: Some("org000".to_string()),
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            on_behalf_of: None,
            delegate_for: None,
            acting_as: None,
            additional_claims: HashMap::new(),
        };

        assert_eq!(claims.extract_tenant_id(), Some("tenant456".to_string()));

        // Test with additional claims
        let mut additional_claims = HashMap::new();
        additional_claims.insert(
            "tenantId".to_string(),
            Value::String("additional123".to_string()),
        );

        let claims = JwtClaims {
            tenant: None,
            tenant_id: None,
            organization: None,
            org_id: None,
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            on_behalf_of: None,
            delegate_for: None,
            acting_as: None,
            additional_claims,
        };

        assert_eq!(
            claims.extract_tenant_id(),
            Some("additional123".to_string())
        );
    }

    #[test]
    fn test_jwt_claims_extract_on_behalf_of() {
        // Test with on_behalf_of field
        let claims = JwtClaims {
            tenant: None,
            tenant_id: None,
            organization: None,
            org_id: None,
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            on_behalf_of: Some("user123".to_string()),
            delegate_for: Some("user456".to_string()),
            acting_as: Some("user789".to_string()),
            additional_claims: HashMap::new(),
        };

        // Should prioritize 'on_behalf_of' field
        assert_eq!(claims.extract_on_behalf_of(), Some("user123".to_string()));

        // Test with only delegate_for
        let claims = JwtClaims {
            tenant: None,
            tenant_id: None,
            organization: None,
            org_id: None,
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            on_behalf_of: None,
            delegate_for: Some("user456".to_string()),
            acting_as: Some("user789".to_string()),
            additional_claims: HashMap::new(),
        };

        assert_eq!(claims.extract_on_behalf_of(), Some("user456".to_string()));

        // Test with additional claims
        let mut additional_claims = HashMap::new();
        additional_claims.insert(
            "onBehalfOf".to_string(),
            Value::String("additional_user".to_string()),
        );

        let claims = JwtClaims {
            tenant: None,
            tenant_id: None,
            organization: None,
            org_id: None,
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            on_behalf_of: None,
            delegate_for: None,
            acting_as: None,
            additional_claims,
        };

        assert_eq!(
            claims.extract_on_behalf_of(),
            Some("additional_user".to_string())
        );
    }

    #[test]
    fn test_jwt_parser_creation() {
        let parser = JwtParser::new();
        assert!(!parser.debug_logging);

        let parser_with_debug = JwtParser::with_debug_logging();
        assert!(parser_with_debug.debug_logging);

        let default_parser = JwtParser::default();
        assert!(!default_parser.debug_logging);
    }

    #[test]
    fn test_jwt_format_validation() {
        let parser = JwtParser::new();

        // Valid format
        assert!(parser.is_valid_jwt_format("header.payload.signature"));

        // Invalid formats
        assert!(!parser.is_valid_jwt_format("header.payload"));
        assert!(!parser.is_valid_jwt_format("header"));
        assert!(!parser.is_valid_jwt_format(""));
        assert!(!parser.is_valid_jwt_format("header..signature"));
        assert!(!parser.is_valid_jwt_format(".payload.signature"));
    }

    #[cfg(feature = "tenant-extraction")]
    #[test]
    fn test_parse_claims_invalid_format() {
        let parser = JwtParser::new();

        // Test invalid formats
        let result = parser.parse_claims("invalid");
        assert!(matches!(result, Err(JwtParseError::InvalidFormat(_))));

        let result = parser.parse_claims("header.payload");
        assert!(matches!(result, Err(JwtParseError::InvalidFormat(_))));
    }

    #[cfg(not(feature = "tenant-extraction"))]
    #[test]
    fn test_parse_claims_feature_disabled() {
        let parser = JwtParser::new();

        let result = parser.parse_claims("header.payload.signature");
        assert!(matches!(result, Err(JwtParseError::FeatureNotEnabled)));

        let result = parser.parse_header("header.payload.signature");
        assert!(matches!(result, Err(JwtParseError::FeatureNotEnabled)));
    }
}
