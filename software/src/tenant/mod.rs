// ABOUTME: Tenant extraction and JWT parsing functionality for multi-tenant support
// ABOUTME: Provides parse-only JWT processing and tenant context extraction from multiple sources

//! Tenant extraction module for multi-tenant support.
//!
//! This module provides functionality for extracting tenant information from various sources
//! including JWT tokens, HTTP headers, request payloads, and query parameters. The module
//! implements parse-only JWT processing without signature validation, leaving authentication
//! and authorization concerns to upstream or downstream services.

pub mod error_handler;
pub mod extraction;
pub mod jwt;

pub use error_handler::{
    create_error_handler_from_env, ErrorStrategy, TenantExtractionErrorHandler,
};
pub use extraction::{ExtractionError, ExtractionSource, TenantExtractor, TenantInfo};
pub use jwt::{JwtClaims, JwtParseError, JwtParser};

/// Priority order for tenant extraction sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExtractionPriority {
    /// JWT token claims (highest priority)
    Jwt = 1,
    /// HTTP headers
    Header = 2,
    /// Request payload/body
    Payload = 3,
    /// Query parameters (lowest priority)
    QueryParam = 4,
}

impl ExtractionPriority {
    /// Get all extraction priorities in order
    pub fn all() -> Vec<Self> {
        vec![Self::Jwt, Self::Header, Self::Payload, Self::QueryParam]
    }

    /// Get priority as a number for comparison
    pub fn as_number(&self) -> u8 {
        *self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_priority_ordering() {
        // ARRANGE: Get all priorities
        let priorities = ExtractionPriority::all();

        // ASSERT: Verify correct order
        assert_eq!(priorities[0], ExtractionPriority::Jwt);
        assert_eq!(priorities[1], ExtractionPriority::Header);
        assert_eq!(priorities[2], ExtractionPriority::Payload);
        assert_eq!(priorities[3], ExtractionPriority::QueryParam);

        // Verify numerical ordering
        assert!(ExtractionPriority::Jwt < ExtractionPriority::Header);
        assert!(ExtractionPriority::Header < ExtractionPriority::Payload);
        assert!(ExtractionPriority::Payload < ExtractionPriority::QueryParam);
    }

    #[test]
    fn test_priority_as_number() {
        assert_eq!(ExtractionPriority::Jwt.as_number(), 1);
        assert_eq!(ExtractionPriority::Header.as_number(), 2);
        assert_eq!(ExtractionPriority::Payload.as_number(), 3);
        assert_eq!(ExtractionPriority::QueryParam.as_number(), 4);
    }
}
