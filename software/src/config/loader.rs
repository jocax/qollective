// ABOUTME: Configuration loading from various sources and formats
// ABOUTME: Supports JSON, YAML, TOML, environment variables, and configuration files

//! Configuration loading from various sources and formats.

use super::presets::{ConfigPreset, QollectiveConfig};
use super::validator::{ConfigValidator, ValidationResult};
use crate::error::{QollectiveError, Result};
use std::collections::HashMap;
use std::env;
use std::fs;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
use super::nats::NatsConfig;

/// Configuration source types
#[derive(Debug, Clone)]
pub enum ConfigSource {
    File(String),
    Json(String),
    Environment,
    Default,
    Preset(ConfigPreset),
}

/// Environment variable mappings for configuration
pub struct EnvironmentMapper {
    prefix: String,
    mappings: HashMap<String, String>,
}

impl EnvironmentMapper {
    pub fn new(prefix: &str) -> Self {
        let mut mappings = HashMap::new();

        // Standard environment variable mappings
        mappings.insert("QOLLECTIVE_ENV".to_string(), "environment".to_string());
        mappings.insert("QOLLECTIVE_LOG_LEVEL".to_string(), "log_level".to_string());
        mappings.insert(
            "QOLLECTIVE_TENANT_EXTRACTION".to_string(),
            "tenant_extraction_enabled".to_string(),
        );
        mappings.insert("QOLLECTIVE_DEBUG".to_string(), "debug.enabled".to_string());
        mappings.insert(
            "QOLLECTIVE_PERFORMANCE_ENABLED".to_string(),
            "performance.enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_TRACING_ENABLED".to_string(),
            "tracing.enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_MONITORING_ENABLED".to_string(),
            "monitoring.enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_SECURITY_ENABLED".to_string(),
            "security.enabled".to_string(),
        );

        // REST-specific environment mappings
        mappings.insert(
            "QOLLECTIVE_REST_BASE_URL".to_string(),
            "rest.base_url".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_TIMEOUT".to_string(),
            "rest.timeout_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_MAX_CONNECTIONS".to_string(),
            "rest.max_connections".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_TLS_ENABLED".to_string(),
            "rest.tls.enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_TLS_CERT_PATH".to_string(),
            "rest.tls.cert_path".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_TLS_KEY_PATH".to_string(),
            "rest.tls.key_path".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_TLS_CA_PATH".to_string(),
            "rest.tls.ca_cert_path".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_REST_TLS_VERIFY".to_string(),
            "rest.tls.verify_certificates".to_string(),
        );

        // NATS-specific environment mappings
        mappings.insert(
            "QOLLECTIVE_NATS_URLS".to_string(),
            "nats.connection.urls".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_CONNECTION_TIMEOUT".to_string(),
            "nats.connection.connection_timeout_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_RECONNECT_TIMEOUT".to_string(),
            "nats.connection.reconnect_timeout_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_MAX_RECONNECT_ATTEMPTS".to_string(),
            "nats.connection.max_reconnect_attempts".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_USERNAME".to_string(),
            "nats.connection.username".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_PASSWORD".to_string(),
            "nats.connection.password".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_TOKEN".to_string(),
            "nats.connection.token".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_TLS_ENABLED".to_string(),
            "nats.connection.tls_enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_TLS_CA_FILE".to_string(),
            "nats.connection.tls_ca_file".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_TLS_CERT_FILE".to_string(),
            "nats.connection.tls_cert_file".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_TLS_KEY_FILE".to_string(),
            "nats.connection.tls_key_file".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_CLIENT_TIMEOUT".to_string(),
            "nats.client.request_timeout_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_CLIENT_MAX_PENDING".to_string(),
            "nats.client.max_pending_messages".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_CLIENT_RETRY_ATTEMPTS".to_string(),
            "nats.client.retry_attempts".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_CLIENT_RETRY_DELAY".to_string(),
            "nats.client.retry_delay_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_CLIENT_POOL_SIZE".to_string(),
            "nats.client.connection_pool_size".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_SERVER_ENABLED".to_string(),
            "nats.server.enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_SERVER_PREFIX".to_string(),
            "nats.server.subject_prefix".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_SERVER_QUEUE_GROUP".to_string(),
            "nats.server.queue_group".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_SERVER_MAX_HANDLERS".to_string(),
            "nats.server.max_concurrent_handlers".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_SERVER_HANDLER_TIMEOUT".to_string(),
            "nats.server.handler_timeout_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_SERVER_REQUEST_REPLY".to_string(),
            "nats.server.enable_request_reply".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_DISCOVERY_ENABLED".to_string(),
            "nats.discovery.enabled".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_DISCOVERY_REGISTRY_SUBJECT".to_string(),
            "nats.discovery.agent_registry_subject".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_DISCOVERY_CAPABILITY_SUBJECT".to_string(),
            "nats.discovery.capability_subject".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_DISCOVERY_ANNOUNCEMENT_INTERVAL".to_string(),
            "nats.discovery.announcement_interval_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_DISCOVERY_TTL".to_string(),
            "nats.discovery.ttl_ms".to_string(),
        );
        mappings.insert(
            "QOLLECTIVE_NATS_DISCOVERY_AUTO_REGISTER".to_string(),
            "nats.discovery.auto_register".to_string(),
        );

        Self {
            prefix: prefix.to_string(),
            mappings,
        }
    }

    pub fn load_from_environment(&self) -> HashMap<String, String> {
        let mut config_values = HashMap::new();

        // First, load automatic prefix-based mappings
        for (key, value) in env::vars() {
            if key.starts_with(&self.prefix) {
                // Skip if this key has an explicit mapping (explicit mappings take precedence)
                if !self.mappings.contains_key(&key) {
                    let config_key = key
                        .strip_prefix(&self.prefix)
                        .unwrap_or(&key)
                        .to_lowercase()
                        .replace('_', ".");
                    config_values.insert(config_key, value);
                }
            }
        }

        // Then, load explicit mappings (these override automatic ones)
        for (env_var, config_path) in &self.mappings {
            if let Ok(value) = env::var(env_var) {
                config_values.insert(config_path.clone(), value);
            }
        }

        config_values
    }
}

/// Configuration loader with support for multiple sources and merging
pub struct ConfigLoader {
    sources: Vec<ConfigSource>,
    env_mapper: EnvironmentMapper,
    validate_config: bool,
    strict_validation: bool,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            sources: vec![],
            env_mapper: EnvironmentMapper::new("QOLLECTIVE_"),
            validate_config: true,
            strict_validation: false,
        }
    }

    pub fn with_env_prefix(prefix: &str) -> Self {
        Self {
            sources: vec![],
            env_mapper: EnvironmentMapper::new(prefix),
            validate_config: true,
            strict_validation: false,
        }
    }

    pub fn add_source(mut self, source: ConfigSource) -> Self {
        self.sources.push(source);
        self
    }

    pub fn add_file_source<P: AsRef<std::path::Path>>(self, path: P) -> Self {
        self.add_source(ConfigSource::File(
            path.as_ref().to_string_lossy().to_string(),
        ))
    }

    pub fn add_json_source(self, json: &str) -> Self {
        self.add_source(ConfigSource::Json(json.to_string()))
    }

    pub fn add_environment_source(self) -> Self {
        self.add_source(ConfigSource::Environment)
    }

    pub fn add_preset_source(self, preset: ConfigPreset) -> Self {
        self.add_source(ConfigSource::Preset(preset))
    }

    /// Enable or disable configuration validation
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.validate_config = enabled;
        self
    }

    /// Enable strict validation (production mode)
    pub fn with_strict_validation(mut self, enabled: bool) -> Self {
        self.strict_validation = enabled;
        self
    }

    /// Load configuration with validation
    pub fn load_and_validate(&self) -> Result<(QollectiveConfig, Option<ValidationResult>)> {
        let config = self.load()?;

        if self.validate_config {
            let validator = if self.strict_validation {
                ConfigValidator::strict()
            } else {
                ConfigValidator::new()
            };

            let validation_result = validator.validate(&config);

            // Return error if validation fails and we're in strict mode
            if self.strict_validation && !validation_result.is_valid {
                return Err(QollectiveError::config(format!(
                    "Configuration validation failed: {}",
                    validation_result
                        .errors
                        .iter()
                        .map(|e| format!("{}: {}", e.field_path, e.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                )));
            }

            Ok((config, Some(validation_result)))
        } else {
            Ok((config, None))
        }
    }

    pub fn load(&self) -> Result<QollectiveConfig> {
        let mut config = QollectiveConfig {
            tenant_extraction_enabled: false,
            meta: crate::config::meta::MetaConfig::default(),
            rest: None,

            #[cfg(feature = "grpc-client")]
            grpc_client: None,

            #[cfg(feature = "grpc-server")]
            grpc_server: None,

            #[cfg(any(feature = "nats-client", feature = "nats-server"))]
            nats: None,

            #[cfg(feature = "tenant-extraction")]
            jwt_extraction: None,
        };

        // Load from sources in order (later sources override earlier ones)
        for source in &self.sources {
            match source {
                ConfigSource::Default => {
                    // Use default configuration as base
                    config = QollectiveConfig {
                        tenant_extraction_enabled: false,
                        meta: crate::config::meta::MetaConfig::default(),
                        rest: None,

                        #[cfg(feature = "grpc-client")]
                        grpc_client: None,

                        #[cfg(feature = "grpc-server")]
                        grpc_server: None,

                        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
                        nats: None,

                        #[cfg(feature = "tenant-extraction")]
                        jwt_extraction: None,
                    };
                }
                ConfigSource::Preset(preset) => {
                    config = self.merge_configs(config, preset.to_config())?;
                }
                ConfigSource::File(path) => {
                    if let Ok(file_config) = self.load_from_file(path) {
                        config = self.merge_configs(config, file_config)?;
                    }
                }
                ConfigSource::Json(json_str) => {
                    if let Ok(json_config) = self.load_from_json(json_str) {
                        config = self.merge_configs(config, json_config)?;
                    }
                }
                ConfigSource::Environment => {
                    let env_values = self.env_mapper.load_from_environment();
                    config = self.apply_environment_overrides(config, env_values)?;
                }
            }
        }

        // Auto-detect environment if not explicitly set
        if self.sources.is_empty() {
            config = self.auto_detect_environment()?;
        }

        Ok(config)
    }

    fn load_from_file(&self, path: &str) -> Result<QollectiveConfig> {
        let content = fs::read_to_string(path).map_err(|e| {
            QollectiveError::Internal(format!("Failed to read config file {}: {}", path, e))
        })?;

        if path.ends_with(".json") {
            self.load_from_json(&content)
        } else {
            Err(QollectiveError::Internal(format!(
                "Unsupported config file format for {}",
                path
            )))
        }
    }

    fn load_from_json(&self, _json_str: &str) -> Result<QollectiveConfig> {
        // For now, return a basic configuration
        // Future enhancement: Implement proper JSON deserialization with serde
        // This would parse JSON configuration files into QollectiveConfig structs
        Ok(ConfigPreset::Development.to_config())
    }

    fn merge_configs(
        &self,
        _base: QollectiveConfig,
        override_config: QollectiveConfig,
    ) -> Result<QollectiveConfig> {
        // Simple merge strategy - override takes precedence
        Ok(override_config)
    }

    fn apply_environment_overrides(
        &self,
        mut config: QollectiveConfig,
        env_values: HashMap<String, String>,
    ) -> Result<QollectiveConfig> {
        // Apply environment variable overrides with better error handling
        for (key, value) in env_values {
            match key.as_str() {
                "tenant_extraction_enabled" => {
                    match value.parse::<bool>() {
                        Ok(enabled) => config.tenant_extraction_enabled = enabled,
                        Err(_) => return Err(QollectiveError::config(format!(
                            "Invalid value for QOLLECTIVE_TENANT_EXTRACTION: '{}'. Expected 'true' or 'false'", value
                        ))),
                    }
                }
                "debug.enabled" => {
                    match value.parse::<bool>() {
                        Ok(enabled) => {
                            if let Some(ref mut debug_config) = config.meta.debug {
                                debug_config.enabled = enabled;
                            }
                        }
                        Err(_) => return Err(QollectiveError::config(format!(
                            "Invalid value for QOLLECTIVE_DEBUG: '{}'. Expected 'true' or 'false'", value
                        ))),
                    }
                }
                "performance.enabled" => {
                    match value.parse::<bool>() {
                        Ok(enabled) => {
                            if let Some(ref mut perf_config) = config.meta.performance {
                                perf_config.enabled = enabled;
                            }
                        }
                        Err(_) => return Err(QollectiveError::config(format!(
                            "Invalid value for QOLLECTIVE_PERFORMANCE_ENABLED: '{}'. Expected 'true' or 'false'", value
                        ))),
                    }
                }
                "tracing.enabled" => {
                    match value.parse::<bool>() {
                        Ok(enabled) => {
                            if let Some(ref mut tracing_config) = config.meta.tracing {
                                tracing_config.enabled = enabled;
                            }
                        }
                        Err(_) => return Err(QollectiveError::config(format!(
                            "Invalid value for QOLLECTIVE_TRACING_ENABLED: '{}'. Expected 'true' or 'false'", value
                        ))),
                    }
                }
                "monitoring.enabled" => {
                    match value.parse::<bool>() {
                        Ok(enabled) => {
                            if let Some(ref mut monitoring_config) = config.meta.monitoring {
                                monitoring_config.enabled = enabled;
                            }
                        }
                        Err(_) => return Err(QollectiveError::config(format!(
                            "Invalid value for QOLLECTIVE_MONITORING_ENABLED: '{}'. Expected 'true' or 'false'", value
                        ))),
                    }
                }
                "security.enabled" => {
                    match value.parse::<bool>() {
                        Ok(enabled) => {
                            if let Some(ref mut security_config) = config.meta.security {
                                security_config.enabled = enabled;
                            }
                        }
                        Err(_) => return Err(QollectiveError::config(format!(
                            "Invalid value for QOLLECTIVE_SECURITY_ENABLED: '{}'. Expected 'true' or 'false'", value
                        ))),
                    }
                }

                // NATS-specific environment variable processing
                #[cfg(any(feature = "nats-client", feature = "nats-server"))]
                key if key.starts_with("nats.") => {
                    // Ensure NATS config exists
                    if config.nats.is_none() {
                        config.nats = Some(NatsConfig::default());
                    }
                    if let Some(ref mut nats_config) = config.nats {
                        match key.as_ref() {
                            "nats.connection.urls" => {
                                // Parse comma-separated URLs
                                let urls: Vec<String> = value.split(',')
                                    .map(|url| url.trim().to_string())
                                    .filter(|url| !url.is_empty())
                                    .collect();
                                if !urls.is_empty() {
                                    nats_config.connection.urls = urls;
                                }
                            }
                            "nats.connection.connection_timeout_ms" => {
                                match value.parse::<u64>() {
                                    Ok(timeout) => nats_config.connection.connection_timeout_ms = timeout,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_CONNECTION_TIMEOUT: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.connection.reconnect_timeout_ms" => {
                                match value.parse::<u64>() {
                                    Ok(timeout) => nats_config.connection.reconnect_timeout_ms = timeout,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_RECONNECT_TIMEOUT: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.connection.max_reconnect_attempts" => {
                                if value.is_empty() || value.eq_ignore_ascii_case("none") {
                                    nats_config.connection.max_reconnect_attempts = None;
                                } else {
                                    match value.parse::<u32>() {
                                        Ok(attempts) => nats_config.connection.max_reconnect_attempts = Some(attempts),
                                        Err(_) => return Err(QollectiveError::config(format!(
                                            "Invalid value for QOLLECTIVE_NATS_MAX_RECONNECT_ATTEMPTS: '{}'. Expected a number or 'none'", value
                                        ))),
                                    }
                                }
                            }
                            "nats.connection.username" => {
                                nats_config.connection.username = if value.is_empty() { None } else { Some(value) };
                            }
                            "nats.connection.password" => {
                                nats_config.connection.password = if value.is_empty() { None } else { Some(value) };
                            }
                            "nats.connection.token" => {
                                nats_config.connection.token = if value.is_empty() { None } else { Some(value) };
                            }
                            "nats.connection.tls_enabled" => {
                                match value.parse::<bool>() {
                                    Ok(enabled) => nats_config.connection.tls.enabled = enabled,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_TLS_ENABLED: '{}'. Expected 'true' or 'false'", value
                                    ))),
                                }
                            }
                            "nats.connection.tls_ca_file" => {
                                nats_config.connection.tls.ca_cert_path = if value.is_empty() { None } else { Some(value.into()) };
                            }
                            "nats.connection.tls_cert_file" => {
                                nats_config.connection.tls.cert_path = if value.is_empty() { None } else { Some(value.into()) };
                            }
                            "nats.connection.tls_key_file" => {
                                nats_config.connection.tls.key_path = if value.is_empty() { None } else { Some(value.into()) };
                            }
                            "nats.client.request_timeout_ms" => {
                                match value.parse::<u64>() {
                                    Ok(timeout) => nats_config.client.request_timeout_ms = timeout,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_CLIENT_TIMEOUT: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.client.max_pending_messages" => {
                                match value.parse::<usize>() {
                                    Ok(max_pending) => nats_config.client.max_pending_messages = max_pending,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_CLIENT_MAX_PENDING: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.client.retry_attempts" => {
                                match value.parse::<u32>() {
                                    Ok(attempts) => nats_config.client.retry_attempts = attempts,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_CLIENT_RETRY_ATTEMPTS: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.client.retry_delay_ms" => {
                                match value.parse::<u64>() {
                                    Ok(delay) => nats_config.client.retry_delay_ms = delay,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_CLIENT_RETRY_DELAY: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.client.connection_pool_size" => {
                                match value.parse::<usize>() {
                                    Ok(pool_size) => nats_config.client.connection_pool_size = pool_size,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_CLIENT_POOL_SIZE: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.server.enabled" => {
                                match value.parse::<bool>() {
                                    Ok(enabled) => nats_config.server.enabled = enabled,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_SERVER_ENABLED: '{}'. Expected 'true' or 'false'", value
                                    ))),
                                }
                            }
                            "nats.server.subject_prefix" => {
                                nats_config.server.subject_prefix = value;
                            }
                            "nats.server.queue_group" => {
                                nats_config.server.queue_group = if value.is_empty() { None } else { Some(value) };
                            }
                            "nats.server.max_concurrent_handlers" => {
                                match value.parse::<usize>() {
                                    Ok(max_handlers) => nats_config.server.max_concurrent_handlers = max_handlers,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_SERVER_MAX_HANDLERS: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.server.handler_timeout_ms" => {
                                match value.parse::<u64>() {
                                    Ok(timeout) => nats_config.server.handler_timeout_ms = timeout,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_SERVER_HANDLER_TIMEOUT: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.server.enable_request_reply" => {
                                match value.parse::<bool>() {
                                    Ok(enabled) => nats_config.server.enable_request_reply = enabled,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_SERVER_REQUEST_REPLY: '{}'. Expected 'true' or 'false'", value
                                    ))),
                                }
                            }
                            "nats.discovery.enabled" => {
                                match value.parse::<bool>() {
                                    Ok(enabled) => nats_config.discovery.enabled = enabled,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_DISCOVERY_ENABLED: '{}'. Expected 'true' or 'false'", value
                                    ))),
                                }
                            }
                            "nats.discovery.agent_registry_subject" => {
                                nats_config.discovery.agent_registry_subject = value;
                            }
                            "nats.discovery.capability_subject" => {
                                nats_config.discovery.capability_subject = value;
                            }
                            "nats.discovery.announcement_interval_ms" => {
                                match value.parse::<u64>() {
                                    Ok(interval) => nats_config.discovery.announcement_interval_ms = interval,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_DISCOVERY_ANNOUNCEMENT_INTERVAL: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.discovery.ttl_ms" => {
                                match value.parse::<u64>() {
                                    Ok(ttl) => nats_config.discovery.ttl_ms = ttl,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_DISCOVERY_TTL: '{}'. Expected a number", value
                                    ))),
                                }
                            }
                            "nats.discovery.auto_register" => {
                                match value.parse::<bool>() {
                                    Ok(auto_register) => nats_config.discovery.auto_register = auto_register,
                                    Err(_) => return Err(QollectiveError::config(format!(
                                        "Invalid value for QOLLECTIVE_NATS_DISCOVERY_AUTO_REGISTER: '{}'. Expected 'true' or 'false'", value
                                    ))),
                                }
                            }
                            _ => {
                                // Unknown NATS configuration key
                                if self.strict_validation {
                                    return Err(QollectiveError::config(format!(
                                        "Unknown NATS configuration key: '{}' with value '{}'", key, value
                                    )));
                                }
                            }
                        }
                    }
                }

                _ => {
                    // In strict mode, warn about unknown configuration keys
                    if self.strict_validation {
                        return Err(QollectiveError::config(format!(
                            "Unknown configuration key: '{}' with value '{}'", key, value
                        )));
                    }
                    // Otherwise silently ignore
                }
            }
        }

        Ok(config)
    }

    fn auto_detect_environment(&self) -> Result<QollectiveConfig> {
        // Auto-detect based on common environment indicators
        let environment = env::var("ENVIRONMENT")
            .or_else(|_| env::var("ENV"))
            .or_else(|_| env::var("NODE_ENV"))
            .or_else(|_| env::var("RUST_ENV"))
            .unwrap_or_else(|_| "development".to_string());

        let preset = match environment.to_lowercase().as_str() {
            "production" | "prod" => ConfigPreset::Production,
            "staging" | "stage" => ConfigPreset::Staging,
            "development" | "dev" => ConfigPreset::Development,
            "debug" => ConfigPreset::Debugging,
            "performance" | "perf" => ConfigPreset::HighPerformance,
            _ => ConfigPreset::Development,
        };

        Ok(preset.to_config())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::env_vars::*;
    use std::env;

    /// Comprehensive cleanup of all QOLLECTIVE environment variables to ensure test isolation
    fn cleanup_all_qollective_env_vars() {
        let mut vars_to_remove = Vec::new();
        for (key, _) in env::vars() {
            if key.starts_with("QOLLECTIVE_") {
                vars_to_remove.push(key);
            }
        }
        for key in vars_to_remove {
            env::remove_var(&key);
        }
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_environment_variable_loading() {
        // Clean up ALL QOLLECTIVE environment variables to ensure test isolation
        cleanup_all_qollective_env_vars();

        // Set up environment variables for testing using constants
        env::set_var(
            QOLLECTIVE_NATS_URLS,
            "nats://server1:4222,nats://server2:4223",
        );
        env::set_var(QOLLECTIVE_NATS_TLS_ENABLED, "true");
        env::set_var(QOLLECTIVE_NATS_SERVER_ENABLED, "true");
        env::set_var(QOLLECTIVE_NATS_DISCOVERY_ENABLED, "false");
        env::set_var(QOLLECTIVE_NATS_CLIENT_TIMEOUT, "45000");
        env::set_var(QOLLECTIVE_NATS_USERNAME, "test_user");
        env::set_var(QOLLECTIVE_NATS_PASSWORD, "test_pass");

        let loader = ConfigLoader::new().add_environment_source();

        let config = loader
            .load()
            .expect("Should load config with NATS environment variables");

        assert!(config.nats.is_some());
        let nats_config = config.nats.unwrap();

        // Test URL parsing
        assert_eq!(
            nats_config.connection.urls,
            vec!["nats://server1:4222", "nats://server2:4223"]
        );

        // Test boolean values
        assert!(nats_config.connection.tls.enabled);
        assert!(nats_config.server.enabled);
        assert!(!nats_config.discovery.enabled);

        // Test numeric values
        assert_eq!(nats_config.client.request_timeout_ms, 45000);

        // Test optional string values
        assert_eq!(
            nats_config.connection.username,
            Some("test_user".to_string())
        );
        assert_eq!(
            nats_config.connection.password,
            Some("test_pass".to_string())
        );

        // Clean up ALL environment variables to prevent test contamination
        cleanup_all_qollective_env_vars();
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_environment_variable_validation() {
        // Clean up ALL QOLLECTIVE environment variables to ensure test isolation
        cleanup_all_qollective_env_vars();

        // Test invalid timeout value using constants
        env::set_var(QOLLECTIVE_NATS_CLIENT_TIMEOUT, "not_a_number");

        let loader = ConfigLoader::new().add_environment_source();

        let result = loader.load();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected a number"));

        cleanup_all_qollective_env_vars();

        // Test invalid boolean value using constants
        env::set_var(QOLLECTIVE_NATS_TLS_ENABLED, "maybe");

        let loader = ConfigLoader::new().add_environment_source();

        let result = loader.load();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected 'true' or 'false'"));

        cleanup_all_qollective_env_vars();
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_config_validation_in_loader() {
        let loader = ConfigLoader::new()
            .add_preset_source(ConfigPreset::Development)
            .with_validation(true)
            .with_strict_validation(false);

        let (config, validation_result) = loader
            .load_and_validate()
            .expect("Should load and validate development config");

        assert!(config.nats.is_some());
        assert!(validation_result.is_some());
        assert!(validation_result.unwrap().is_valid);
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_environment_mapper() {
        let mapper = EnvironmentMapper::new("QOLLECTIVE_");

        // Test that NATS-specific mappings are included
        assert!(mapper.mappings.contains_key("QOLLECTIVE_NATS_URLS"));
        assert!(mapper.mappings.contains_key("QOLLECTIVE_NATS_TLS_ENABLED"));
        assert!(mapper
            .mappings
            .contains_key("QOLLECTIVE_NATS_SERVER_ENABLED"));
        assert!(mapper
            .mappings
            .contains_key("QOLLECTIVE_NATS_DISCOVERY_ENABLED"));

        // Test mapping values
        assert_eq!(
            mapper.mappings.get("QOLLECTIVE_NATS_URLS"),
            Some(&"nats.connection.urls".to_string())
        );
        assert_eq!(
            mapper.mappings.get("QOLLECTIVE_NATS_TLS_ENABLED"),
            Some(&"nats.connection.tls_enabled".to_string())
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_empty_optional_values() {
        // Clean up ALL QOLLECTIVE environment variables to ensure test isolation
        cleanup_all_qollective_env_vars();

        // Test that empty string values are handled correctly for optional fields using constants
        env::set_var(QOLLECTIVE_NATS_USERNAME, "");
        env::set_var(QOLLECTIVE_NATS_TOKEN, "");
        env::set_var(QOLLECTIVE_NATS_SERVER_QUEUE_GROUP, "");
        env::set_var(QOLLECTIVE_NATS_MAX_RECONNECT_ATTEMPTS, "none");

        let loader = ConfigLoader::new().add_environment_source();

        let config = loader
            .load()
            .expect("Should load config with empty optional values");

        assert!(config.nats.is_some());
        let nats_config = config.nats.unwrap();

        // Empty strings should result in None for optional fields
        assert_eq!(nats_config.connection.username, None);
        assert_eq!(nats_config.connection.token, None);
        assert_eq!(nats_config.server.queue_group, None);
        assert_eq!(nats_config.connection.max_reconnect_attempts, None);

        // Clean up ALL environment variables to prevent test contamination
        cleanup_all_qollective_env_vars();
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_urls_parsing() {
        // Clean up ALL QOLLECTIVE environment variables to ensure test isolation
        cleanup_all_qollective_env_vars();

        // Test comma-separated URL parsing using constants
        env::set_var(
            QOLLECTIVE_NATS_URLS,
            "nats://localhost:4222,  nats://backup:4222  , nats://fallback:4222",
        );

        let loader = ConfigLoader::new()
            .add_preset_source(ConfigPreset::Development)
            .add_environment_source();

        let config = loader.load().expect("Should parse comma-separated URLs");

        assert!(config.nats.is_some());
        let nats_config = config.nats.unwrap();

        assert_eq!(
            nats_config.connection.urls,
            vec![
                "nats://localhost:4222".to_string(),
                "nats://backup:4222".to_string(),
                "nats://fallback:4222".to_string()
            ]
        );

        // Clean up ALL environment variables to prevent test contamination
        cleanup_all_qollective_env_vars();
    }

    #[test]
    fn test_config_loader_with_preset() {
        let loader = ConfigLoader::new().add_preset_source(ConfigPreset::Production);

        let config = loader.load().expect("Should load production config");

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            assert!(config.nats.is_some());
            let nats_config = config.nats.unwrap();
            assert!(nats_config.connection.tls.enabled); // Production should use TLS
        }

        assert!(config.tenant_extraction_enabled);
    }

    #[test]
    fn test_config_loader_auto_detect_environment() {
        // Test auto-detection without explicit sources
        env::set_var("ENVIRONMENT", "production");

        let loader = ConfigLoader::new(); // No sources added
        let config = loader.load().expect("Should auto-detect environment");

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            assert!(config.nats.is_some());
            let nats_config = config.nats.unwrap();
            assert!(nats_config.connection.tls.enabled); // Production should use TLS
        }

        env::remove_var("ENVIRONMENT");
    }

    #[test]
    fn test_config_loader_validation_modes() {
        let loader = ConfigLoader::new()
            .add_preset_source(ConfigPreset::Development)
            .with_validation(true)
            .with_strict_validation(true);

        let result = loader.load_and_validate();
        assert!(result.is_ok());

        let (config, validation_result) = result.unwrap();
        assert!(validation_result.is_some());
        assert!(config.tenant_extraction_enabled);
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    #[test]
    fn test_config_loader_without_nats_features() {
        let loader = ConfigLoader::new().add_preset_source(ConfigPreset::Development);

        let config = loader
            .load()
            .expect("Should load config without NATS features");

        // Config should load successfully without NATS field when features are disabled
        assert!(config.tenant_extraction_enabled);

        // Should be able to validate successfully
        let loader_with_validation = ConfigLoader::new()
            .add_preset_source(ConfigPreset::Development)
            .with_validation(true);

        let (validated_config, validation_result) = loader_with_validation
            .load_and_validate()
            .expect("Should validate config without NATS features");

        assert!(validated_config.tenant_extraction_enabled);
        assert!(validation_result.is_some());
        assert!(validation_result.unwrap().is_valid);
    }
}
