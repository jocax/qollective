//! Error types for LLM client operations
//!
//! This module defines all error types that can occur during LLM client
//! configuration, initialization, and execution.

use crate::constants::*;
use thiserror::Error;

/// Main error type for LLM operations
#[derive(Error, Debug)]
pub enum LlmError {
    /// Configuration error (invalid or missing required configuration)
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Missing credentials for provider that requires authentication
    #[error("Missing credentials: {0}")]
    MissingCredentials(String),

    /// Requested model is not available
    #[error("Model not available: {model_name}")]
    ModelNotAvailable {
        /// The model name that was requested
        model_name: String,
        /// Optional suggestion for fallback model
        fallback_suggestion: Option<String>,
    },

    /// Provider is unreachable (network error, service down, etc.)
    #[error("Provider unreachable: {provider} at {url} - {reason}")]
    ProviderUnreachable {
        /// Provider type (e.g., "shimmy", "openai")
        provider: String,
        /// Provider URL
        url: String,
        /// Reason for unreachability
        reason: String,
    },

    /// Invalid tenant configuration
    #[error("Invalid tenant configuration for tenant_id={tenant_id}: {reason}")]
    InvalidTenantConfig {
        /// Tenant identifier
        tenant_id: String,
        /// Reason why configuration is invalid
        reason: String,
    },

    /// LLM request failed during execution
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// Error from underlying rig-core library
    #[error("Rig error: {0}")]
    RigError(String),

    /// IO error during configuration loading
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Configuration parsing error
    #[error("Config parse error: {0}")]
    ConfigParseError(String),
}

impl LlmError {
    /// Create a ConfigError with a custom message
    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a MissingCredentials error with a custom message
    pub fn missing_credentials(msg: impl Into<String>) -> Self {
        Self::MissingCredentials(msg.into())
    }

    /// Create a ModelNotAvailable error for a specific model
    pub fn model_not_available(model_name: impl Into<String>) -> Self {
        Self::ModelNotAvailable {
            model_name: model_name.into(),
            fallback_suggestion: None,
        }
    }

    /// Create a ModelNotAvailable error with a fallback suggestion
    pub fn model_not_available_with_fallback(
        model_name: impl Into<String>,
        fallback: impl Into<String>,
    ) -> Self {
        Self::ModelNotAvailable {
            model_name: model_name.into(),
            fallback_suggestion: Some(fallback.into()),
        }
    }

    /// Create a ProviderUnreachable error
    pub fn provider_unreachable(
        provider: impl Into<String>,
        url: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ProviderUnreachable {
            provider: provider.into(),
            url: url.into(),
            reason: reason.into(),
        }
    }

    /// Create an InvalidTenantConfig error
    pub fn invalid_tenant_config(
        tenant_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidTenantConfig {
            tenant_id: tenant_id.into(),
            reason: reason.into(),
        }
    }

    /// Create a RequestFailed error
    pub fn request_failed(msg: impl Into<String>) -> Self {
        Self::RequestFailed(msg.into())
    }

    /// Create a RigError from rig-core error
    pub fn rig_error(msg: impl Into<String>) -> Self {
        Self::RigError(msg.into())
    }

    /// Create a ConfigParseError
    pub fn config_parse_error(msg: impl Into<String>) -> Self {
        Self::ConfigParseError(msg.into())
    }
}

/// Validate that a string value is not empty
#[allow(dead_code)]
pub(crate) fn validate_not_empty(value: &str, field_name: &str) -> Result<(), LlmError> {
    if value.trim().is_empty() {
        return Err(LlmError::config_error(format!(
            "{}: {}",
            field_name, ERROR_EMPTY_CONFIG_VALUE
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let err = LlmError::config_error("test error");
        assert!(matches!(err, LlmError::ConfigError(_)));
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_missing_credentials() {
        let err = LlmError::missing_credentials("API key required");
        assert!(matches!(err, LlmError::MissingCredentials(_)));
        assert_eq!(err.to_string(), "Missing credentials: API key required");
    }

    #[test]
    fn test_model_not_available() {
        let err = LlmError::model_not_available("gpt-4");
        match err {
            LlmError::ModelNotAvailable {
                model_name,
                fallback_suggestion,
            } => {
                assert_eq!(model_name, "gpt-4");
                assert!(fallback_suggestion.is_none());
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_model_not_available_with_fallback() {
        let err = LlmError::model_not_available_with_fallback("gpt-4", "gpt-3.5-turbo");
        match err {
            LlmError::ModelNotAvailable {
                model_name,
                fallback_suggestion,
            } => {
                assert_eq!(model_name, "gpt-4");
                assert_eq!(fallback_suggestion, Some("gpt-3.5-turbo".to_string()));
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_provider_unreachable() {
        let err = LlmError::provider_unreachable("openai", "https://api.openai.com", "timeout");
        match err {
            LlmError::ProviderUnreachable {
                provider,
                url,
                reason,
            } => {
                assert_eq!(provider, "openai");
                assert_eq!(url, "https://api.openai.com");
                assert_eq!(reason, "timeout");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_invalid_tenant_config() {
        let err = LlmError::invalid_tenant_config("tenant-123", "missing API key");
        match err {
            LlmError::InvalidTenantConfig { tenant_id, reason } => {
                assert_eq!(tenant_id, "tenant-123");
                assert_eq!(reason, "missing API key");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("valid", "test_field").is_ok());
        assert!(validate_not_empty("", "test_field").is_err());
        assert!(validate_not_empty("   ", "test_field").is_err());
    }
}
