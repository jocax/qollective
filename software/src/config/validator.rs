// ABOUTME: Configuration validation for tenant extraction and framework settings
// ABOUTME: Provides comprehensive validation with detailed error reporting for all configuration sections

//! Configuration validation for tenant extraction and framework settings.
//!
//! This module provides comprehensive validation for all configuration sections,
//! with detailed error reporting and suggestions for fixing invalid configurations.

use super::presets::{QollectiveConfig, RestConfig, TenantClientConfig};
use crate::config::tls::TlsConfig;
use crate::error::{QollectiveError, Result};
use std::path::Path;

/// Configuration validation result with detailed error information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Detailed validation error with context and suggestions
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field_path: String,
    pub error_type: ValidationErrorType,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    Required,
    InvalidValue,
    InvalidFormat,
    InvalidRange,
    FileNotFound,
    PermissionDenied,
    Conflict,
    Security,
}

/// Configuration warnings for potentially problematic settings
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field_path: String,
    pub message: String,
    pub recommendation: String,
}

/// Comprehensive configuration validator
pub struct ConfigValidator {
    strict_mode: bool,
    environment: String,
}

impl ConfigValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            environment: "development".to_string(),
        }
    }

    /// Create a validator with strict mode enabled (production environments)
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            environment: "production".to_string(),
        }
    }

    /// Set the target environment for context-aware validation
    pub fn with_environment(mut self, env: &str) -> Self {
        self.environment = env.to_string();
        self.strict_mode = matches!(env, "production" | "staging");
        self
    }

    /// Validate a complete configuration
    pub fn validate(&self, config: &QollectiveConfig) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate tenant extraction configuration
        self.validate_tenant_extraction(config, &mut errors, &mut warnings);

        // Validate REST configuration if present
        if let Some(ref rest_config) = config.rest {
            self.validate_rest_config(rest_config, &mut errors, &mut warnings);
        }

        // Validate gRPC client configuration if present
        #[cfg(feature = "grpc-client")]
        if let Some(ref grpc_client_config) = config.grpc_client {
            self.validate_grpc_client_config(grpc_client_config, &mut errors, &mut warnings);
        }

        // Validate gRPC server configuration if present
        #[cfg(feature = "grpc-server")]
        if let Some(ref grpc_server_config) = config.grpc_server {
            self.validate_grpc_server_config(grpc_server_config, &mut errors, &mut warnings);
        }

        // Validate meta configuration
        self.validate_meta_config(&config.meta, &mut errors, &mut warnings);

        // Validate cross-section consistency
        self.validate_cross_section_consistency(config, &mut errors, &mut warnings);

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate tenant extraction configuration
    fn validate_tenant_extraction(
        &self,
        config: &QollectiveConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // In strict mode, tenant extraction should be explicitly configured
        if self.strict_mode && self.environment == "production" {
            if !config.tenant_extraction_enabled {
                warnings.push(ValidationWarning {
                    field_path: "tenant_extraction_enabled".to_string(),
                    message: "Tenant extraction is disabled in production environment".to_string(),
                    recommendation:
                        "Consider enabling tenant extraction for multi-tenant applications"
                            .to_string(),
                });
            }
        }

        // If tenant extraction is enabled, validate that at least one transport is configured
        if config.tenant_extraction_enabled {
            let has_rest = config.rest.is_some();
            #[cfg(feature = "grpc-client")]
            let has_grpc_client = config.grpc_client.is_some();
            #[cfg(not(feature = "grpc-client"))]
            let has_grpc_client = false;

            #[cfg(feature = "grpc-server")]
            let has_grpc_server = config.grpc_server.is_some();
            #[cfg(not(feature = "grpc-server"))]
            let has_grpc_server = false;

            let has_grpc = has_grpc_client || has_grpc_server;
            #[cfg(not(any(feature = "grpc-client", feature = "grpc-server")))]
            let has_grpc = false;

            if !has_rest && !has_grpc {
                errors.push(ValidationError {
                    field_path: "tenant_extraction_enabled".to_string(),
                    error_type: ValidationErrorType::Conflict,
                    message:
                        "Tenant extraction is enabled but no transport protocols are configured"
                            .to_string(),
                    suggestion: Some(
                        "Configure at least one transport (REST or gRPC) to use tenant extraction"
                            .to_string(),
                    ),
                });
            }
        }

        // Validate JWT extraction configuration if tenant extraction is enabled
        #[cfg(feature = "tenant-extraction")]
        if config.tenant_extraction_enabled {
            if let Some(ref jwt_config) = config.jwt_extraction {
                self.validate_jwt_extraction_config(jwt_config, errors, warnings);
            } else {
                warnings.push(ValidationWarning {
                    field_path: "jwt_extraction".to_string(),
                    message:
                        "Tenant extraction is enabled but no JWT extraction configuration provided"
                            .to_string(),
                    recommendation: "Configure JWT extraction settings for tenant identification"
                        .to_string(),
                });
            }
        }

        // Validate tenant client configurations if REST is configured
        if let Some(ref rest_config) = config.rest {
            if let Some(ref client_config) = rest_config.client {
                self.validate_tenant_client_config(&client_config.tenant_config, errors, warnings);
            }
        }
    }

    /// Validate JWT extraction configuration
    #[cfg(feature = "tenant-extraction")]
    fn validate_jwt_extraction_config(
        &self,
        jwt_config: &crate::tenant::extraction::ExtractionConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if !jwt_config.enabled {
            warnings.push(ValidationWarning {
                field_path: "jwt_extraction.enabled".to_string(),
                message: "JWT extraction is disabled while tenant extraction is enabled"
                    .to_string(),
                recommendation: "Enable JWT extraction or disable tenant extraction".to_string(),
            });
        }

        if jwt_config.tenant_header_names.is_empty() {
            warnings.push(ValidationWarning {
                field_path: "jwt_extraction.tenant_header_names".to_string(),
                message: "No tenant header names configured for extraction".to_string(),
                recommendation: "Configure at least one header name for tenant extraction"
                    .to_string(),
            });
        }

        if jwt_config.auth_header_patterns.is_empty() {
            errors.push(ValidationError {
                field_path: "jwt_extraction.auth_header_patterns".to_string(),
                error_type: ValidationErrorType::Required,
                message: "No authentication header patterns configured".to_string(),
                suggestion: Some(
                    "Configure at least one auth header pattern (e.g., 'Bearer', 'JWT')"
                        .to_string(),
                ),
            });
        }

        // Validate auth header patterns
        for pattern in &jwt_config.auth_header_patterns {
            if pattern.trim().is_empty() {
                errors.push(ValidationError {
                    field_path: "jwt_extraction.auth_header_patterns".to_string(),
                    error_type: ValidationErrorType::InvalidValue,
                    message: format!("Empty auth header pattern found: '{}'", pattern),
                    suggestion: Some(
                        "Remove empty patterns or provide valid authentication schemes".to_string(),
                    ),
                });
            }
        }

        if self.strict_mode && jwt_config.jwt_debug_logging && self.environment == "production" {
            warnings.push(ValidationWarning {
                field_path: "jwt_extraction.jwt_debug_logging".to_string(),
                message: "JWT debug logging is enabled in production environment".to_string(),
                recommendation: "Disable debug logging in production for security and performance"
                    .to_string(),
            });
        }
    }

    /// Validate tenant client configuration
    fn validate_tenant_client_config(
        &self,
        tenant_config: &TenantClientConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Validate tenant ID conflicts
        if tenant_config.override_tenant_id.is_some() && tenant_config.auto_propagate_tenant {
            warnings.push(ValidationWarning {
                field_path: "rest.client.tenant_config".to_string(),
                message: "Both override_tenant_id and auto_propagate_tenant are enabled".to_string(),
                recommendation: "override_tenant_id takes precedence over auto_propagate_tenant. Consider disabling auto_propagate_tenant if you always want to use the override".to_string(),
            });
        }

        // Validate tenant ID format if provided
        if let Some(ref tenant_id) = tenant_config.override_tenant_id {
            if tenant_id.is_empty() {
                errors.push(ValidationError {
                    field_path: "rest.client.tenant_config.override_tenant_id".to_string(),
                    error_type: ValidationErrorType::InvalidValue,
                    message: "override_tenant_id cannot be empty".to_string(),
                    suggestion: Some("Provide a valid tenant ID or set to None".to_string()),
                });
            } else if tenant_id.len() > 255 {
                errors.push(ValidationError {
                    field_path: "rest.client.tenant_config.override_tenant_id".to_string(),
                    error_type: ValidationErrorType::InvalidRange,
                    message: "override_tenant_id exceeds maximum length of 255 characters"
                        .to_string(),
                    suggestion: Some("Use a shorter tenant ID".to_string()),
                });
            }
        }

        // Same validation for fallback tenant ID
        if let Some(ref fallback_id) = tenant_config.fallback_tenant_id {
            if fallback_id.is_empty() {
                errors.push(ValidationError {
                    field_path: "rest.client.tenant_config.fallback_tenant_id".to_string(),
                    error_type: ValidationErrorType::InvalidValue,
                    message: "fallback_tenant_id cannot be empty".to_string(),
                    suggestion: Some("Provide a valid tenant ID or set to None".to_string()),
                });
            } else if fallback_id.len() > 255 {
                errors.push(ValidationError {
                    field_path: "rest.client.tenant_config.fallback_tenant_id".to_string(),
                    error_type: ValidationErrorType::InvalidRange,
                    message: "fallback_tenant_id exceeds maximum length of 255 characters"
                        .to_string(),
                    suggestion: Some("Use a shorter tenant ID".to_string()),
                });
            }
        }
    }

    /// Validate REST configuration
    fn validate_rest_config(
        &self,
        rest_config: &RestConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Validate client configuration
        if let Some(ref client_config) = rest_config.client {
            self.validate_rest_client_config(client_config, errors, warnings);
        }

        // Validate server configuration
        if let Some(ref server_config) = rest_config.server {
            self.validate_rest_server_config(server_config, errors, warnings);
        }

        // Warn if neither client nor server is configured
        if rest_config.client.is_none() && rest_config.server.is_none() {
            warnings.push(ValidationWarning {
                field_path: "rest".to_string(),
                message:
                    "REST configuration is present but neither client nor server is configured"
                        .to_string(),
                recommendation: "Configure at least one of rest.client or rest.server".to_string(),
            });
        }
    }

    /// Validate REST client configuration
    fn validate_rest_client_config(
        &self,
        client_config: &super::presets::RestClientConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Validate timeout values
        if client_config.timeout_ms == 0 {
            errors.push(ValidationError {
                field_path: "rest.client.timeout_ms".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "Timeout cannot be zero".to_string(),
                suggestion: Some(
                    "Set a reasonable timeout value (e.g., 30000 for 30 seconds)".to_string(),
                ),
            });
        } else if client_config.timeout_ms > 300000 {
            // 5 minutes
            warnings.push(ValidationWarning {
                field_path: "rest.client.timeout_ms".to_string(),
                message: "Very long timeout configured (> 5 minutes)".to_string(),
                recommendation: "Consider using a shorter timeout for better user experience"
                    .to_string(),
            });
        }

        // Validate connection limits
        if client_config.max_connections == 0 {
            errors.push(ValidationError {
                field_path: "rest.client.max_connections".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "max_connections cannot be zero".to_string(),
                suggestion: Some("Set a reasonable connection limit (e.g., 100)".to_string()),
            });
        } else if client_config.max_connections > 10000 {
            warnings.push(ValidationWarning {
                field_path: "rest.client.max_connections".to_string(),
                message: "Very high connection limit configured (> 10000)".to_string(),
                recommendation: "High connection limits may impact performance and resource usage"
                    .to_string(),
            });
        }

        // Validate base URL format if provided
        if let Some(ref base_url) = client_config.base_url {
            if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                errors.push(ValidationError {
                    field_path: "rest.client.base_url".to_string(),
                    error_type: ValidationErrorType::InvalidFormat,
                    message: "base_url must start with http:// or https://".to_string(),
                    suggestion: Some(
                        "Use a valid URL format (e.g., https://api.example.com)".to_string(),
                    ),
                });
            }

            // Security warning for HTTP in production
            if self.strict_mode && base_url.starts_with("http://") {
                warnings.push(ValidationWarning {
                    field_path: "rest.client.base_url".to_string(),
                    message: "Using HTTP (not HTTPS) in production environment".to_string(),
                    recommendation: "Use HTTPS for secure communication in production".to_string(),
                });
            }
        }

        // Validate TLS configuration
        self.validate_tls_config(&client_config.tls, "rest.client.tls", errors, warnings);

        // Validate retry configuration
        if client_config.retry_attempts > 10 {
            warnings.push(ValidationWarning {
                field_path: "rest.client.retry_attempts".to_string(),
                message: "Very high retry count configured (> 10)".to_string(),
                recommendation: "High retry counts may cause excessive delays and resource usage"
                    .to_string(),
            });
        }
    }

    /// Validate REST server configuration
    fn validate_rest_server_config(
        &self,
        server_config: &super::presets::RestServerConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Validate port number
        if server_config.port == 0 {
            errors.push(ValidationError {
                field_path: "rest.server.port".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "Port cannot be zero".to_string(),
                suggestion: Some("Use a valid port number (e.g., 8080)".to_string()),
            });
        } else if server_config.port < 1024 && !self.strict_mode {
            warnings.push(ValidationWarning {
                field_path: "rest.server.port".to_string(),
                message: "Using privileged port (< 1024)".to_string(),
                recommendation: "Privileged ports require root access. Consider using ports > 1024"
                    .to_string(),
            });
        }

        // Validate bind address
        if server_config.bind_address.is_empty() {
            errors.push(ValidationError {
                field_path: "rest.server.bind_address".to_string(),
                error_type: ValidationErrorType::Required,
                message: "bind_address cannot be empty".to_string(),
                suggestion: Some(
                    "Use a valid IP address (e.g., '0.0.0.0' or '127.0.0.1')".to_string(),
                ),
            });
        }

        // Security warning for binding to all interfaces in production
        if self.strict_mode && server_config.bind_address == "0.0.0.0" {
            warnings.push(ValidationWarning {
                field_path: "rest.server.bind_address".to_string(),
                message: "Binding to all interfaces (0.0.0.0) in production".to_string(),
                recommendation: "Consider binding to specific interfaces for better security"
                    .to_string(),
            });
        }

        // Validate TLS configuration
        self.validate_tls_config(&server_config.tls, "rest.server.tls", errors, warnings);

        // Validate timeout
        if server_config.request_timeout_ms == 0 {
            errors.push(ValidationError {
                field_path: "rest.server.request_timeout_ms".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "Request timeout cannot be zero".to_string(),
                suggestion: Some(
                    "Set a reasonable timeout value (e.g., 30000 for 30 seconds)".to_string(),
                ),
            });
        }

        // Validate connection limits
        if server_config.max_connections == 0 {
            errors.push(ValidationError {
                field_path: "rest.server.max_connections".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "max_connections cannot be zero".to_string(),
                suggestion: Some("Set a reasonable connection limit (e.g., 1000)".to_string()),
            });
        }
    }

    /// Validate TLS configuration
    fn validate_tls_config(
        &self,
        tls_config: &TlsConfig,
        field_prefix: &str,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if !tls_config.enabled {
            if self.strict_mode {
                warnings.push(ValidationWarning {
                    field_path: field_prefix.to_string(),
                    message: "TLS is disabled in production environment".to_string(),
                    recommendation: "Enable TLS for secure communication in production".to_string(),
                });
            }
            return;
        }

        // Validate certificate files if specified
        if let Some(ref cert_path) = tls_config.cert_path {
            if !Path::new(cert_path).exists() {
                errors.push(ValidationError {
                    field_path: format!("{}.cert_path", field_prefix),
                    error_type: ValidationErrorType::FileNotFound,
                    message: format!("Certificate file not found: {}", cert_path.display()),
                    suggestion: Some(
                        "Ensure the certificate file exists and is readable".to_string(),
                    ),
                });
            }
        }

        if let Some(ref key_path) = tls_config.key_path {
            if !Path::new(key_path).exists() {
                errors.push(ValidationError {
                    field_path: format!("{}.key_path", field_prefix),
                    error_type: ValidationErrorType::FileNotFound,
                    message: format!("Private key file not found: {}", key_path.display()),
                    suggestion: Some(
                        "Ensure the private key file exists and is readable".to_string(),
                    ),
                });
            }
        }

        if let Some(ref ca_path) = tls_config.ca_cert_path {
            if !Path::new(ca_path).exists() {
                errors.push(ValidationError {
                    field_path: format!("{}.ca_cert_path", field_prefix),
                    error_type: ValidationErrorType::FileNotFound,
                    message: format!("CA certificate file not found: {}", ca_path.display()),
                    suggestion: Some(
                        "Ensure the CA certificate file exists and is readable".to_string(),
                    ),
                });
            }
        }

        // Validate certificate verification mode
        match tls_config.verification_mode {
            crate::config::tls::VerificationMode::Skip => {
                if self.strict_mode {
                    warnings.push(ValidationWarning {
                        field_path: format!("{}.verification_mode", field_prefix),
                        message: "Certificate verification is disabled in production".to_string(),
                        recommendation:
                            "Use SystemCa or CustomCa verification mode for secure communication"
                                .to_string(),
                    });
                }
            }
            crate::config::tls::VerificationMode::CustomCa => {
                if tls_config.ca_cert_path.is_none() {
                    errors.push(ValidationError {
                        field_path: format!("{}.ca_cert_path", field_prefix),
                        error_type: ValidationErrorType::Required,
                        message: "CA certificate path is required when using CustomCa verification"
                            .to_string(),
                        suggestion: Some("Provide a valid CA certificate file path".to_string()),
                    });
                }
            }
            crate::config::tls::VerificationMode::MutualTls => {
                if tls_config.cert_path.is_none() || tls_config.key_path.is_none() {
                    errors.push(ValidationError {
                        field_path: format!(
                            "{}.cert_path or {}.key_path",
                            field_prefix, field_prefix
                        ),
                        error_type: ValidationErrorType::Required,
                        message: "Client certificate and key paths are required for mutual TLS"
                            .to_string(),
                        suggestion: Some(
                            "Provide both cert_path and key_path for mutual TLS authentication"
                                .to_string(),
                        ),
                    });
                }
            }
            crate::config::tls::VerificationMode::SystemCa => {
                // SystemCa is the most secure default - no warnings needed
            }
        }
    }

    /// Validate gRPC client configuration
    #[cfg(feature = "grpc-client")]
    fn validate_grpc_client_config(
        &self,
        client_config: &super::grpc::GrpcClientConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Similar validation logic as REST client
        if client_config.timeout_ms == 0 {
            errors.push(ValidationError {
                field_path: "grpc.client.timeout_ms".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "Timeout cannot be zero".to_string(),
                suggestion: Some(
                    "Set a reasonable timeout value (e.g., 30000 for 30 seconds)".to_string(),
                ),
            });
        }

        if client_config.max_connections == 0 {
            errors.push(ValidationError {
                field_path: "grpc.client.max_connections".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "max_connections cannot be zero".to_string(),
                suggestion: Some("Set a reasonable connection limit (e.g., 100)".to_string()),
            });
        }

        // Validate TLS configuration
        self.validate_tls_config(&client_config.tls, "grpc.client.tls", errors, warnings);
    }

    /// Validate gRPC server configuration
    #[cfg(feature = "grpc-server")]
    fn validate_grpc_server_config(
        &self,
        server_config: &super::grpc::GrpcServerConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Similar validation logic as REST server
        if server_config.port == 0 {
            errors.push(ValidationError {
                field_path: "grpc.server.port".to_string(),
                error_type: ValidationErrorType::InvalidValue,
                message: "Port cannot be zero".to_string(),
                suggestion: Some("Use a valid port number (e.g., 50051)".to_string()),
            });
        }

        if server_config.bind_address.is_empty() {
            errors.push(ValidationError {
                field_path: "grpc.server.bind_address".to_string(),
                error_type: ValidationErrorType::Required,
                message: "bind_address cannot be empty".to_string(),
                suggestion: Some(
                    "Use a valid IP address (e.g., '0.0.0.0' or '127.0.0.1')".to_string(),
                ),
            });
        }

        // Validate TLS configuration
        self.validate_tls_config(&server_config.tls, "grpc.server.tls", errors, warnings);
    }

    /// Validate meta configuration
    fn validate_meta_config(
        &self,
        meta_config: &super::meta::MetaConfig,
        _errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Check for conflicting configurations
        if let (Some(ref debug), Some(ref performance)) =
            (&meta_config.debug, &meta_config.performance)
        {
            if debug.enabled && performance.enabled {
                let debug_all = matches!(debug.properties, super::meta::PropertyConfig::All);
                let perf_all = matches!(performance.properties, super::meta::PropertyConfig::All);

                if debug_all && perf_all && self.strict_mode {
                    warnings.push(ValidationWarning {
                        field_path: "meta".to_string(),
                        message: "Both debug and performance metadata are fully enabled"
                            .to_string(),
                        recommendation:
                            "Consider disabling debug metadata in production for better performance"
                                .to_string(),
                    });
                }
            }
        }

        // Warn if no metadata sections are enabled
        let any_enabled = [
            meta_config
                .debug
                .as_ref()
                .map(|d| d.enabled)
                .unwrap_or(false),
            meta_config
                .performance
                .as_ref()
                .map(|p| p.enabled)
                .unwrap_or(false),
            meta_config
                .security
                .as_ref()
                .map(|s| s.enabled)
                .unwrap_or(false),
            meta_config
                .monitoring
                .as_ref()
                .map(|m| m.enabled)
                .unwrap_or(false),
            meta_config
                .tracing
                .as_ref()
                .map(|t| t.enabled)
                .unwrap_or(false),
        ]
        .iter()
        .any(|&enabled| enabled);

        if !any_enabled {
            warnings.push(ValidationWarning {
                field_path: "meta".to_string(),
                message: "No metadata sections are enabled".to_string(),
                recommendation:
                    "Enable at least one metadata section to benefit from the Qollective framework"
                        .to_string(),
            });
        }
    }

    /// Validate cross-section consistency
    fn validate_cross_section_consistency(
        &self,
        config: &QollectiveConfig,
        _errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Check for tenant extraction enabled but no transport configured to use it
        if config.tenant_extraction_enabled {
            let mut has_tenant_aware_transport = false;

            if let Some(ref rest_config) = config.rest {
                if rest_config.client.is_some() || rest_config.server.is_some() {
                    has_tenant_aware_transport = true;
                }
            }

            #[cfg(feature = "grpc-client")]
            if config.grpc_client.is_some() {
                has_tenant_aware_transport = true;
            }

            #[cfg(feature = "grpc-server")]
            if config.grpc_server.is_some() {
                has_tenant_aware_transport = true;
            }

            if !has_tenant_aware_transport {
                warnings.push(ValidationWarning {
                    field_path: "tenant_extraction_enabled".to_string(),
                    message:
                        "Tenant extraction is enabled but no configured transports will use it"
                            .to_string(),
                    recommendation:
                        "Configure REST or gRPC client/server to make use of tenant extraction"
                            .to_string(),
                });
            }
        }

        // Check for security metadata enabled but no security-focused transport configuration
        if let Some(ref security_config) = config.meta.security {
            if security_config.enabled {
                let mut has_secure_transport = false;

                if let Some(ref rest_config) = config.rest {
                    if let Some(ref client) = rest_config.client {
                        if client.tls.enabled {
                            has_secure_transport = true;
                        }
                    }
                    if let Some(ref server) = rest_config.server {
                        if server.tls.enabled {
                            has_secure_transport = true;
                        }
                    }
                }

                #[cfg(feature = "grpc-client")]
                if let Some(ref grpc_client_config) = config.grpc_client {
                    if grpc_client_config.tls.enabled {
                        has_secure_transport = true;
                    }
                }

                #[cfg(feature = "grpc-server")]
                if let Some(ref grpc_server_config) = config.grpc_server {
                    if grpc_server_config.tls.enabled {
                        has_secure_transport = true;
                    }
                }

                if !has_secure_transport && self.strict_mode {
                    warnings.push(ValidationWarning {
                        field_path: "meta.security".to_string(),
                        message:
                            "Security metadata is enabled but no secure transports are configured"
                                .to_string(),
                        recommendation:
                            "Enable TLS on at least one transport when using security metadata"
                                .to_string(),
                    });
                }
            }
        }
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    /// Convert validation result to a Qollective error if validation failed
    pub fn to_result(&self) -> Result<()> {
        if self.is_valid {
            Ok(())
        } else {
            let error_messages: Vec<String> = self
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.field_path, e.message))
                .collect();
            Err(QollectiveError::config(format!(
                "Configuration validation failed: {}",
                error_messages.join("; ")
            )))
        }
    }

    /// Get a summary of validation issues
    pub fn summary(&self) -> String {
        format!(
            "Validation: {} error(s), {} warning(s)",
            self.errors.len(),
            self.warnings.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::presets::ConfigPreset;

    #[test]
    fn test_valid_development_config() {
        let config = ConfigPreset::Development.to_config();
        let validator = ConfigValidator::new();
        let result = validator.validate(&config);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_production_config_validation() {
        let mut config = ConfigPreset::Production.to_config();

        // Remove TLS file paths for testing since they don't exist on test system
        if let Some(ref mut rest_config) = config.rest {
            if let Some(ref mut server_config) = rest_config.server {
                server_config.tls.cert_path = None;
                server_config.tls.key_path = None;
            }
            if let Some(ref mut client_config) = rest_config.client {
                client_config.tls.cert_path = None;
                client_config.tls.key_path = None;
            }
        }

        #[cfg(feature = "grpc-server")]
        if let Some(ref mut grpc_server_config) = config.grpc_server {
            grpc_server_config.tls.cert_path = None;
            grpc_server_config.tls.key_path = None;
        }

        #[cfg(feature = "grpc-client")]
        if let Some(ref mut grpc_client_config) = config.grpc_client {
            grpc_client_config.tls.cert_path = None;
            grpc_client_config.tls.key_path = None;
        }

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        if let Some(ref mut nats_config) = config.nats {
            nats_config.connection.tls.cert_path = None;
            nats_config.connection.tls.key_path = None;
        }

        let validator = ConfigValidator::strict().with_environment("production");
        let result = validator.validate(&config);

        // Should be valid but may have warnings
        assert!(
            result.is_valid,
            "Production config should be valid after removing TLS file paths"
        );

        // Should have warnings about binding to all interfaces
        assert!(
            !result.warnings.is_empty(),
            "Production config should have warnings"
        );
    }

    #[test]
    fn test_invalid_timeout_configuration() {
        let mut config = ConfigPreset::Development.to_config();
        if let Some(ref mut rest_config) = config.rest {
            if let Some(ref mut client_config) = rest_config.client {
                client_config.timeout_ms = 0; // Invalid timeout
            }
        }

        let validator = ConfigValidator::new();
        let result = validator.validate(&config);

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_tenant_extraction_without_transport() {
        let mut config = ConfigPreset::Development.to_config();
        config.tenant_extraction_enabled = true;
        config.rest = None;

        #[cfg(feature = "grpc-client")]
        {
            config.grpc_client = None;
        }

        #[cfg(feature = "grpc-server")]
        {
            config.grpc_server = None;
        }

        let validator = ConfigValidator::new();
        let result = validator.validate(&config);

        // Should have warnings about tenant extraction without transport
        assert!(!result.warnings.is_empty());
    }
}
