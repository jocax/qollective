// ABOUTME: Unified error handling for tenant extraction with security-first approach
// ABOUTME: Sanitizes error details to prevent information leakage and maps to envelope schema

//! Unified error handling for tenant extraction with security considerations.
//!
//! This module provides secure error handling for tenant extraction that prevents
//! information leakage while ensuring all errors map properly to envelope responses.
//! Error details are sanitized to avoid exposing JWT tokens, headers, or system internals.

use super::{ExtractionError, TenantInfo};
use crate::error::{QollectiveError, Result};

/// Error handling strategy for tenant extraction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorStrategy {
    /// Fail fast - return error immediately on any extraction failure
    FailFast,
    /// Log and continue - log errors but don't fail the request  
    LogAndContinue,
    /// Silent ignore - ignore all extraction errors silently
    SilentIgnore,
}

impl Default for ErrorStrategy {
    fn default() -> Self {
        Self::LogAndContinue
    }
}

/// Configuration for tenant extraction error handling
#[derive(Debug, Clone)]
pub struct ErrorHandlerConfig {
    /// Error handling strategy
    pub strategy: ErrorStrategy,
    /// Whether to include minimal error codes in logs (never sensitive details)
    pub include_error_codes: bool,
}

impl Default for ErrorHandlerConfig {
    fn default() -> Self {
        Self {
            strategy: ErrorStrategy::LogAndContinue,
            include_error_codes: true,
        }
    }
}

/// Unified error handler for tenant extraction
#[derive(Debug, Clone)]
pub struct TenantExtractionErrorHandler {
    config: ErrorHandlerConfig,
    extraction_enabled: bool,
}

impl TenantExtractionErrorHandler {
    /// Create a new error handler with default configuration
    pub fn new(extraction_enabled: bool) -> Self {
        Self {
            config: ErrorHandlerConfig::default(),
            extraction_enabled,
        }
    }

    /// Create a new error handler with custom configuration
    pub fn with_config(config: ErrorHandlerConfig, extraction_enabled: bool) -> Self {
        Self {
            config,
            extraction_enabled,
        }
    }

    /// Handle an extraction error and return sanitized envelope-compatible error
    pub fn handle_extraction_error(
        &self,
        error: ExtractionError,
        extraction_source: &str,
    ) -> Result<Option<TenantInfo>> {
        // If extraction is disabled, return None (not an error)
        if !self.extraction_enabled {
            return Ok(None);
        }

        // Generate sanitized error code and message
        let (error_code, safe_message) = self.sanitize_error(&error);

        match &self.config.strategy {
            ErrorStrategy::FailFast => {
                self.log_error_safely(error_code, extraction_source);
                Err(QollectiveError::tenant_extraction(safe_message))
            }
            ErrorStrategy::LogAndContinue => {
                self.log_error_safely(error_code, extraction_source);
                Ok(None)
            }
            ErrorStrategy::SilentIgnore => {
                // No logging for silent ignore
                Ok(None)
            }
        }
    }

    /// Handle successful extraction with minimal logging
    pub fn handle_extraction_success(
        &self,
        result: Option<TenantInfo>,
        extraction_source: &str,
    ) -> Result<Option<TenantInfo>> {
        if result.is_some() {
            #[cfg(feature = "tracing")]
            tracing::debug!("Tenant extracted from {}", extraction_source);
        }

        Ok(result)
    }

    /// Sanitize error details to prevent information leakage
    fn sanitize_error(&self, error: &ExtractionError) -> (&'static str, String) {
        match error {
            ExtractionError::JwtError(_) => (
                "JWT_PARSE_ERROR",
                "Authentication token format invalid".to_string(),
            ),
            ExtractionError::ExtractionDisabled => (
                "EXTRACTION_DISABLED",
                "Tenant extraction not enabled".to_string(),
            ),
            ExtractionError::NoTenantFound => ("NO_TENANT", "No tenant context found".to_string()),
            ExtractionError::InvalidJson(_) => (
                "PAYLOAD_FORMAT_ERROR",
                "Request payload format invalid".to_string(),
            ),
            ExtractionError::MissingAuthHeader => {
                ("MISSING_AUTH", "Authentication header required".to_string())
            }
            ExtractionError::InvalidAuthHeaderFormat => (
                "AUTH_FORMAT_ERROR",
                "Authentication header format invalid".to_string(),
            ),
            ExtractionError::ConfigError(_) => (
                "CONFIG_ERROR",
                "Tenant extraction configuration error".to_string(),
            ),
        }
    }

    /// Log error safely without exposing sensitive information
    fn log_error_safely(&self, error_code: &str, extraction_source: &str) {
        if self.config.include_error_codes {
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "Tenant extraction failed: source={}, code={}",
                extraction_source,
                error_code
            );

            #[cfg(not(feature = "tracing"))]
            tracing::warn!(
                "WARN: Tenant extraction failed: source={}, code={}",
                extraction_source,
                error_code
            );
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!("Tenant extraction failed from {}", extraction_source);

            #[cfg(not(feature = "tracing"))]
            tracing::warn!("WARN: Tenant extraction failed from {}", extraction_source);
        }
    }

    /// Check if extraction errors should cause request failure
    pub fn should_fail_on_error(&self) -> bool {
        matches!(self.config.strategy, ErrorStrategy::FailFast)
    }

    /// Get the current error strategy
    pub fn get_strategy(&self) -> &ErrorStrategy {
        &self.config.strategy
    }

    /// Set the error handling strategy
    pub fn set_strategy(&mut self, strategy: ErrorStrategy) {
        self.config.strategy = strategy;
    }

    /// Enable or disable extraction
    pub fn set_extraction_enabled(&mut self, enabled: bool) {
        self.extraction_enabled = enabled;
    }
}

impl Default for TenantExtractionErrorHandler {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Create error handler from environment with secure defaults
pub fn create_error_handler_from_env() -> TenantExtractionErrorHandler {
    let extraction_enabled = std::env::var("QOLLECTIVE_TENANT_EXTRACTION")
        .map(|v| v.parse().unwrap_or(false))
        .unwrap_or(false);

    let strategy = std::env::var("QOLLECTIVE_TENANT_ERROR_STRATEGY")
        .map(|s| match s.to_lowercase().as_str() {
            "fail-fast" | "failfast" => ErrorStrategy::FailFast,
            "silent" | "silent-ignore" => ErrorStrategy::SilentIgnore,
            _ => ErrorStrategy::LogAndContinue,
        })
        .unwrap_or_default();

    let config = ErrorHandlerConfig {
        strategy,
        include_error_codes: true, // Error codes are safe to include
    };

    TenantExtractionErrorHandler::with_config(config, extraction_enabled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::ExtractionError;

    #[test]
    fn test_error_sanitization() {
        let handler = TenantExtractionErrorHandler::new(true);

        // Test JWT error sanitization - should not expose token details
        let jwt_error = ExtractionError::JwtError(crate::tenant::JwtParseError::InvalidFormat(
            "detailed internal error with sensitive info".to_string(),
        ));
        let (code, message) = handler.sanitize_error(&jwt_error);
        assert_eq!(code, "JWT_PARSE_ERROR");
        assert_eq!(message, "Authentication token format invalid");
        assert!(!message.contains("sensitive info"));

        // Test config error sanitization
        let config_error =
            ExtractionError::ConfigError("Internal config path: /secret/path".to_string());
        let (code, message) = handler.sanitize_error(&config_error);
        assert_eq!(code, "CONFIG_ERROR");
        assert!(!message.contains("/secret/path"));
    }

    #[test]
    fn test_handle_extraction_error_fail_fast() {
        let mut handler = TenantExtractionErrorHandler::new(true);
        handler.set_strategy(ErrorStrategy::FailFast);

        let error = ExtractionError::NoTenantFound;
        let result = handler.handle_extraction_error(error, "test_source");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, QollectiveError::TenantExtraction(_)));

        // Verify error message is sanitized
        if let QollectiveError::TenantExtraction(msg) = err {
            assert_eq!(msg, "No tenant context found");
        }
    }

    #[test]
    fn test_handle_extraction_error_log_and_continue() {
        let mut handler = TenantExtractionErrorHandler::new(true);
        handler.set_strategy(ErrorStrategy::LogAndContinue);

        let error = ExtractionError::InvalidAuthHeaderFormat;
        let result = handler.handle_extraction_error(error, "test_source");

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extraction_disabled_not_an_error() {
        let handler = TenantExtractionErrorHandler::new(false);

        let error = ExtractionError::NoTenantFound;
        let result = handler.handle_extraction_error(error, "test_source");

        // When extraction is disabled, any error should return Ok(None)
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_error_strategy_security() {
        let mut handler = TenantExtractionErrorHandler::new(true);

        // Test that silent ignore truly logs nothing and returns Ok(None)
        handler.set_strategy(ErrorStrategy::SilentIgnore);
        let error = ExtractionError::JwtError(crate::tenant::JwtParseError::PayloadDecodeError(
            "super sensitive JWT payload details".to_string(),
        ));
        let result = handler.handle_extraction_error(error, "jwt");

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_create_error_handler_from_env() {
        // Test with no environment variables
        std::env::remove_var("QOLLECTIVE_TENANT_EXTRACTION");
        std::env::remove_var("QOLLECTIVE_TENANT_ERROR_STRATEGY");

        let handler = create_error_handler_from_env();
        assert!(!handler.extraction_enabled); // Default is false
        assert_eq!(handler.get_strategy(), &ErrorStrategy::LogAndContinue);
    }
}
