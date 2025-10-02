// ABOUTME: Security configuration builder with environment variable support and presets
// ABOUTME: Provides configurable security policies, storage backends, and validation strategies

use crate::constants::{network, timeouts};
use std::collections::HashMap;
use std::env;

/// Security Configuration for token propagation and validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityConfig {
    pub jwt_validation: JwtValidationConfig,
    pub storage: StorageConfig,
    pub scope_validation: ScopeValidationConfig,
    pub transmission: TransmissionConfig,
    pub expiration: ExpirationConfig,
    pub audit: AuditConfig,
}

impl SecurityConfig {
    pub fn builder() -> SecurityConfigBuilder {
        SecurityConfigBuilder::new()
    }

    /// Development preset - in-memory, permissive
    pub fn development() -> Self {
        Self {
            jwt_validation: JwtValidationConfig {
                provider: "default".to_string(),
                verify_signature: false,
                verify_expiry: false,
                issuer: None,
                audience: None,
                algorithms: vec!["HS256".to_string()],
            },
            storage: StorageConfig {
                backend: "memory".to_string(),
                connection_string: None,
                encryption_key: None,
                ttl_seconds: None,
            },
            scope_validation: ScopeValidationConfig {
                strategy: "default".to_string(),
                enforce_scopes: false,
                default_scopes: vec!["read".to_string()],
                role_hierarchy: HashMap::new(),
            },
            transmission: TransmissionConfig {
                require_https: false,
                add_security_headers: false,
                token_header: "Authorization".to_string(),
                token_prefix: "Bearer ".to_string(),
            },
            expiration: ExpirationConfig {
                check_expiry: false,
                refresh_threshold_seconds: timeouts::DEFAULT_JWT_REFRESH_THRESHOLD_SECS,
                auto_refresh: false,
            },
            audit: AuditConfig {
                enabled: true,
                backend: "memory".to_string(),
                log_file_path: None,
                log_jwt_validation: true,
                log_authentication: true,
                log_authorization: true,
                log_level: "info".to_string(),
                include_details: true,
                max_events_memory: Some(1000),
            },
        }
    }

    /// Production preset - secure defaults
    pub fn production() -> Self {
        Self {
            jwt_validation: JwtValidationConfig {
                provider: "default".to_string(),
                verify_signature: true,
                verify_expiry: true,
                issuer: None,
                audience: None,
                algorithms: vec!["RS256".to_string(), "ES256".to_string()],
            },
            storage: StorageConfig {
                backend: "redis".to_string(),
                connection_string: Some(network::DEFAULT_REDIS_CONNECTION_STRING.to_string()),
                encryption_key: None,
                ttl_seconds: Some(timeouts::DEFAULT_SECURITY_TTL_SECS),
            },
            scope_validation: ScopeValidationConfig {
                strategy: "rbac".to_string(),
                enforce_scopes: true,
                default_scopes: vec![],
                role_hierarchy: Self::default_role_hierarchy(),
            },
            transmission: TransmissionConfig {
                require_https: true,
                add_security_headers: true,
                token_header: "Authorization".to_string(),
                token_prefix: "Bearer ".to_string(),
            },
            expiration: ExpirationConfig {
                check_expiry: true,
                refresh_threshold_seconds: timeouts::DEFAULT_JWT_REFRESH_THRESHOLD_SECS,
                auto_refresh: true,
            },
            audit: AuditConfig {
                enabled: true,
                backend: "file".to_string(),
                log_file_path: Some("/var/log/qollective/security-audit.log".to_string()),
                log_jwt_validation: true,
                log_authentication: true,
                log_authorization: true,
                log_level: "warning".to_string(),
                include_details: false,
                max_events_memory: None,
            },
        }
    }

    fn default_role_hierarchy() -> HashMap<String, Vec<String>> {
        let mut hierarchy = HashMap::new();
        hierarchy.insert(
            "admin".to_string(),
            vec![
                "admin:all".to_string(),
                "manager:all".to_string(),
                "user:all".to_string(),
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
        );
        hierarchy.insert(
            "manager".to_string(),
            vec![
                "manager:all".to_string(),
                "user:all".to_string(),
                "read".to_string(),
                "write".to_string(),
            ],
        );
        hierarchy.insert(
            "user".to_string(),
            vec!["user:all".to_string(), "read".to_string()],
        );
        hierarchy
    }
}

/// JWT Validation Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JwtValidationConfig {
    pub provider: String, // "default", "auth0", "okta", "azure-ad", "custom"
    pub verify_signature: bool,
    pub verify_expiry: bool,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub algorithms: Vec<String>,
}

/// Storage Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageConfig {
    pub backend: String, // "memory", "redis", "database", "vault"
    pub connection_string: Option<String>,
    pub encryption_key: Option<String>,
    pub ttl_seconds: Option<u64>,
}

/// Scope Validation Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScopeValidationConfig {
    pub strategy: String, // "default", "rbac", "abac", "timebound"
    pub enforce_scopes: bool,
    pub default_scopes: Vec<String>,
    pub role_hierarchy: HashMap<String, Vec<String>>,
}

/// Transmission Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransmissionConfig {
    pub require_https: bool,
    pub add_security_headers: bool,
    pub token_header: String,
    pub token_prefix: String,
}

/// Expiration Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExpirationConfig {
    pub check_expiry: bool,
    pub refresh_threshold_seconds: u64,
    pub auto_refresh: bool,
}

/// Audit Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub backend: String, // "memory", "file", "syslog", "database"
    pub log_file_path: Option<String>,
    pub log_jwt_validation: bool,
    pub log_authentication: bool,
    pub log_authorization: bool,
    pub log_level: String, // "info", "warning", "error", "critical"
    pub include_details: bool,
    pub max_events_memory: Option<u32>,
}

/// Security Configuration Builder with environment variable support
pub struct SecurityConfigBuilder {
    jwt_validation: Option<JwtValidationConfig>,
    storage: Option<StorageConfig>,
    scope_validation: Option<ScopeValidationConfig>,
    transmission: Option<TransmissionConfig>,
    expiration: Option<ExpirationConfig>,
    audit: Option<AuditConfig>,
}

impl SecurityConfigBuilder {
    pub fn new() -> Self {
        Self {
            jwt_validation: None,
            storage: None,
            scope_validation: None,
            transmission: None,
            expiration: None,
            audit: None,
        }
    }

    /// Load from preset configuration
    pub fn from_preset(preset: &str) -> Self {
        match preset {
            "development" => Self::from_config(SecurityConfig::development()),
            "production" => Self::from_config(SecurityConfig::production()),
            _ => Self::new(),
        }
    }

    /// Load from existing configuration
    pub fn from_config(config: SecurityConfig) -> Self {
        Self {
            jwt_validation: Some(config.jwt_validation),
            storage: Some(config.storage),
            scope_validation: Some(config.scope_validation),
            transmission: Some(config.transmission),
            expiration: Some(config.expiration),
            audit: Some(config.audit),
        }
    }

    /// Configure JWT validation
    pub fn with_jwt_validation(mut self, config: JwtValidationConfig) -> Self {
        self.jwt_validation = Some(config);
        self
    }

    /// Configure storage backend
    pub fn with_storage(mut self, config: StorageConfig) -> Self {
        self.storage = Some(config);
        self
    }

    /// Configure scope validation
    pub fn with_scope_validation(mut self, config: ScopeValidationConfig) -> Self {
        self.scope_validation = Some(config);
        self
    }

    /// Configure token transmission
    pub fn with_transmission(mut self, config: TransmissionConfig) -> Self {
        self.transmission = Some(config);
        self
    }

    /// Configure token expiration handling
    pub fn with_expiration(mut self, config: ExpirationConfig) -> Self {
        self.expiration = Some(config);
        self
    }

    /// Configure audit logging
    pub fn with_audit(mut self, config: AuditConfig) -> Self {
        self.audit = Some(config);
        self
    }

    /// Apply environment variable overrides
    pub fn apply_environment_overrides(mut self) -> Self {
        // Ensure we have configurations to override (use defaults if not set)
        // This handles the case where someone calls apply_environment_overrides()
        // without setting up the builder first
        if self.jwt_validation.is_none() {
            self.jwt_validation = Some(SecurityConfig::development().jwt_validation);
        }
        if self.storage.is_none() {
            self.storage = Some(SecurityConfig::development().storage);
        }
        if self.scope_validation.is_none() {
            self.scope_validation = Some(SecurityConfig::development().scope_validation);
        }
        if self.transmission.is_none() {
            self.transmission = Some(SecurityConfig::development().transmission);
        }
        if self.expiration.is_none() {
            self.expiration = Some(SecurityConfig::development().expiration);
        }
        if self.audit.is_none() {
            self.audit = Some(SecurityConfig::development().audit);
        }

        // JWT Validation overrides
        if let Some(ref mut jwt_config) = self.jwt_validation {
            if let Ok(provider) = env::var("QOLLECTIVE_JWT_PROVIDER") {
                jwt_config.provider = provider;
            }
            if let Ok(verify_sig) = env::var("QOLLECTIVE_JWT_VERIFY_SIGNATURE") {
                jwt_config.verify_signature =
                    verify_sig.parse().unwrap_or(jwt_config.verify_signature);
            }
            if let Ok(issuer) = env::var("QOLLECTIVE_JWT_ISSUER") {
                jwt_config.issuer = Some(issuer);
            }
            if let Ok(audience) = env::var("QOLLECTIVE_JWT_AUDIENCE") {
                jwt_config.audience = Some(audience);
            }
        }

        // Storage overrides
        if let Some(ref mut storage_config) = self.storage {
            if let Ok(backend) = env::var("QOLLECTIVE_STORAGE_BACKEND") {
                storage_config.backend = backend;
            }
            if let Ok(connection) = env::var("QOLLECTIVE_STORAGE_CONNECTION") {
                storage_config.connection_string = Some(connection);
            }
            if let Ok(key) = env::var("QOLLECTIVE_STORAGE_ENCRYPTION_KEY") {
                storage_config.encryption_key = Some(key);
            }
        }

        // Scope validation overrides
        if let Some(ref mut scope_config) = self.scope_validation {
            if let Ok(strategy) = env::var("QOLLECTIVE_SCOPE_STRATEGY") {
                scope_config.strategy = strategy;
            }
            if let Ok(enforce) = env::var("QOLLECTIVE_SCOPE_ENFORCE") {
                scope_config.enforce_scopes =
                    enforce.parse().unwrap_or(scope_config.enforce_scopes);
            }
        }

        // Transmission overrides
        if let Some(ref mut trans_config) = self.transmission {
            if let Ok(require_https) = env::var("QOLLECTIVE_REQUIRE_HTTPS") {
                trans_config.require_https =
                    require_https.parse().unwrap_or(trans_config.require_https);
            }
            if let Ok(header) = env::var("QOLLECTIVE_TOKEN_HEADER") {
                trans_config.token_header = header;
            }
        }

        // Expiration overrides
        if let Some(ref mut exp_config) = self.expiration {
            if let Ok(check_expiry) = env::var("QOLLECTIVE_CHECK_EXPIRY") {
                exp_config.check_expiry = check_expiry.parse().unwrap_or(exp_config.check_expiry);
            }
            if let Ok(auto_refresh) = env::var("QOLLECTIVE_AUTO_REFRESH") {
                exp_config.auto_refresh = auto_refresh.parse().unwrap_or(exp_config.auto_refresh);
            }
        }

        self
    }

    /// Build the final security configuration
    pub fn build(self) -> SecurityConfig {
        SecurityConfig {
            jwt_validation: self
                .jwt_validation
                .unwrap_or_else(|| SecurityConfig::development().jwt_validation),
            storage: self
                .storage
                .unwrap_or_else(|| SecurityConfig::development().storage),
            scope_validation: self
                .scope_validation
                .unwrap_or_else(|| SecurityConfig::development().scope_validation),
            transmission: self
                .transmission
                .unwrap_or_else(|| SecurityConfig::development().transmission),
            expiration: self
                .expiration
                .unwrap_or_else(|| SecurityConfig::development().expiration),
            audit: self
                .audit
                .unwrap_or_else(|| SecurityConfig::development().audit),
        }
    }
}

impl Default for SecurityConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_presets() {
        let dev_config = SecurityConfig::development();
        assert_eq!(dev_config.jwt_validation.provider, "default");
        assert!(!dev_config.jwt_validation.verify_signature);
        assert_eq!(dev_config.storage.backend, "memory");

        let prod_config = SecurityConfig::production();
        assert_eq!(prod_config.jwt_validation.provider, "default");
        assert!(prod_config.jwt_validation.verify_signature);
        assert_eq!(prod_config.storage.backend, "redis");
    }

    #[test]
    fn test_security_config_builder() {
        let config = SecurityConfigBuilder::from_preset("development")
            .with_jwt_validation(JwtValidationConfig {
                provider: "auth0".to_string(),
                verify_signature: true,
                verify_expiry: true,
                issuer: Some("https://dev.auth0.com".to_string()),
                audience: Some("api".to_string()),
                algorithms: vec!["RS256".to_string()],
            })
            .build();

        assert_eq!(config.jwt_validation.provider, "auth0");
        assert!(config.jwt_validation.verify_signature);
        assert_eq!(config.storage.backend, "memory"); // From development preset
    }

    #[test]
    fn test_environment_variable_overrides() {
        std::env::set_var("QOLLECTIVE_JWT_PROVIDER", "okta");
        std::env::set_var("QOLLECTIVE_STORAGE_BACKEND", "vault");

        let config = SecurityConfigBuilder::from_preset("development")
            .apply_environment_overrides()
            .build();

        assert_eq!(config.jwt_validation.provider, "okta");
        assert_eq!(config.storage.backend, "vault");

        // Cleanup
        std::env::remove_var("QOLLECTIVE_JWT_PROVIDER");
        std::env::remove_var("QOLLECTIVE_STORAGE_BACKEND");
    }
}
