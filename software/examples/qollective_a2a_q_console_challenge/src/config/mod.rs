// ABOUTME: Central configuration for Qollective A2A Enterprise Star Trek example
// ABOUTME: Provides typed configuration mapping TOML to framework config structs

//! Configuration management for the Qollective A2A Enterprise example.
//!
//! This module provides configuration structures that map from TOML configuration
//! to the framework's native configuration types, following the pattern established
//! by the holodeck example.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[cfg(test)]
mod tests;

/// Central configuration for the entire Qollective A2A Enterprise system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    pub enterprise: EnterpriseServerConfig,
    pub nats: NatsExampleConfig,
    pub tls: TlsExampleConfig,
    pub a2a_server: A2AServerExampleConfig,
    pub a2a_client: A2AClientExampleConfig,
    pub agents: HashMap<String, AgentExampleConfig>,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
}

/// Enterprise-specific server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseServerConfig {
    pub server_id: String,
    pub server_name: String,
    pub ship_registry: String,
    pub ship_class: String,
    pub timeouts: EnterpriseTimeouts,
    pub limits: EnterpriseLimits,
}

/// Enterprise timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseTimeouts {
    pub crew_ttl_secs: u64,
    pub cleanup_interval_secs: u64,
    pub health_check_interval_secs: u64,
}

/// Enterprise limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseLimits {
    pub max_crew_size: usize,
    pub max_capabilities_per_crew: usize,
    pub max_reconnect_attempts: u32,
}

/// Example NATS configuration that converts to framework NatsConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsExampleConfig {
    pub connection_timeout_ms: u64,
    pub reconnect_timeout_ms: u64,
    pub max_reconnect_attempts: u32,
    pub connection: NatsConnectionExampleConfig,
    pub client: NatsClientExampleConfig,
    pub discovery: NatsDiscoveryExampleConfig,
}

/// NATS connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConnectionExampleConfig {
    pub urls: Vec<String>,
    pub ping_interval_ms: u64,
    pub max_outstanding_pings: u16,
}

/// NATS client behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsClientExampleConfig {
    pub client_name: String,
    pub max_pending_bytes: usize,
    pub max_pending_messages: usize,
    pub enable_reconnect: bool,
    pub no_echo: bool,
}

/// NATS discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsDiscoveryExampleConfig {
    pub enabled: bool,
    pub ttl_ms: u64,
    pub heartbeat_interval_ms: u64,
}

/// Example TLS configuration that converts to framework TlsConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsExampleConfig {
    /// Enable TLS for connections
    pub enabled: bool,
    
    /// Allow insecure TLS connections (skip certificate verification)
    #[serde(default)]
    pub insecure: bool,
    
    /// Path to CA certificate file
    pub ca_cert_path: String,
    
    /// Path to client certificate file
    pub cert_path: String,
    
    /// Path to private key file
    pub key_path: String,
    
    /// TLS verification mode: "mutual_tls", "skip"
    pub verification_mode: String,
    
    /// Server name for SNI (Server Name Indication)
    #[serde(default)]
    pub server_name: Option<String>,
    
    /// Supported TLS protocol versions
    #[serde(default)]
    pub protocol_versions: Vec<String>,
    
    /// Supported cipher suites (if not specified, use system defaults)
    #[serde(default)]
    pub cipher_suites: Vec<String>,
    
    /// ALPN (Application-Layer Protocol Negotiation) protocols
    #[serde(default)]
    pub alpn_protocols: Vec<String>,
    
    /// Certificate chain validation settings
    #[serde(default)]
    pub certificate_validation: CertificateValidationConfig,
    
    /// Connection timeout for TLS handshake in milliseconds
    #[serde(default = "default_tls_timeout")]
    pub handshake_timeout_ms: u64,
}

/// Certificate validation configuration for enhanced TLS security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidationConfig {
    /// Verify certificate chain up to root CA
    #[serde(default = "default_verify_chain")]
    pub verify_chain: bool,
    
    /// Verify certificate expiration dates
    #[serde(default = "default_verify_expiration")]
    pub verify_expiration: bool,
    
    /// Verify certificate hostname matches connection target
    #[serde(default = "default_verify_hostname")]
    pub verify_hostname: bool,
    
    /// Allow self-signed certificates (development only)
    #[serde(default)]
    pub allow_self_signed: bool,
    
    /// Maximum certificate chain depth to validate
    #[serde(default = "default_max_chain_depth")]
    pub max_chain_depth: u32,
    
    /// Certificate revocation check method: "none", "crl", "ocsp", "ocsp_stapling"
    #[serde(default = "default_revocation_check")]
    pub revocation_check: String,
}

impl Default for CertificateValidationConfig {
    fn default() -> Self {
        Self {
            verify_chain: default_verify_chain(),
            verify_expiration: default_verify_expiration(),
            verify_hostname: default_verify_hostname(),
            allow_self_signed: false,
            max_chain_depth: default_max_chain_depth(),
            revocation_check: default_revocation_check(),
        }
    }
}

/// Default TLS handshake timeout in milliseconds
fn default_tls_timeout() -> u64 {
    10_000 // 10 seconds
}

/// Default certificate chain verification setting
fn default_verify_chain() -> bool {
    true
}

/// Default certificate expiration verification setting
fn default_verify_expiration() -> bool {
    true
}

/// Default hostname verification setting
fn default_verify_hostname() -> bool {
    true
}

/// Default maximum certificate chain depth
fn default_max_chain_depth() -> u32 {
    10
}

/// Default certificate revocation check method
fn default_revocation_check() -> String {
    "none".to_string()
}

/// Example A2A Server configuration that converts to framework A2AServerConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AServerExampleConfig {
    pub registry: RegistryExampleConfig,
    pub subjects: SubjectsExampleConfig,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryExampleConfig {
    pub agent_ttl_secs: u64,
    pub cleanup_interval_secs: u64,
    pub max_agents: usize,
    pub enable_health_monitoring: bool,
    pub enable_agent_logging: bool,
    pub enable_capability_indexing: bool,
    pub max_capabilities_per_agent: usize,
    pub logging_agent_capability: String,
}

/// NATS subjects configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectsExampleConfig {
    pub prefix: String,
    pub agent_registration: String,
    pub agent_deregistration: String,
    pub agent_discovery: String,
    pub agent_health: String,
    pub agent_capabilities: String,
    pub agent_registry_events: String,
    pub agent_registry_announce: String,
    pub enterprise_bridge_challenge: String,
}

/// Example A2A Client configuration that converts to framework A2AClientConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AClientExampleConfig {
    pub heartbeat_interval_secs: u64,
    pub discovery_cache_ttl_secs: u64,
    pub enable_metrics: bool,
    pub retry: RetryExampleConfig,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryExampleConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExampleConfig {
    pub capabilities: Vec<String>,
    pub location: String,
    pub function: String,
    pub service_type: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub show_timestamps: bool,
    pub enable_detailed_logging: bool,
    pub enable_performance_metrics: bool,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_health_checks: bool,
    pub enable_registry_monitoring: bool,
    pub enable_connection_monitoring: bool,
    pub enable_performance_metrics: bool,
    pub metrics_interval_secs: u64,
}

impl EnterpriseConfig {
    /// Load configuration from TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: EnterpriseConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from default location
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_file("config.toml")
    }
}

impl NatsExampleConfig {
    /// Convert to framework NatsConfig
    pub fn to_framework_config(&self, tls_config: &TlsExampleConfig) -> qollective::config::nats::NatsConfig {
        qollective::config::nats::NatsConfig {
            connection: qollective::config::nats::NatsConnectionConfig {
                urls: self.connection.urls.clone(),
                connection_timeout_ms: self.connection_timeout_ms,
                reconnect_timeout_ms: self.reconnect_timeout_ms,
                max_reconnect_attempts: Some(self.max_reconnect_attempts),
                username: None,
                password: None,
                token: None,
                tls: tls_config.to_framework_tls_config(),
                crypto_provider_strategy: None,
                custom_headers: HashMap::new(),
                client_name: Some(self.client.client_name.clone()),
            },
            client: qollective::config::nats::NatsClientBehaviorConfig {
                request_timeout_ms: 30000,
                max_pending_messages: self.client.max_pending_messages,
                retry_attempts: self.max_reconnect_attempts,
                retry_delay_ms: 1000,
                connection_pool_size: 10,
            },
            server: qollective::config::nats::NatsServerConfig {
                enabled: false, // We connect to external NATS
                ..Default::default()
            },
            discovery: qollective::config::nats::NatsDiscoveryConfig {
                enabled: self.discovery.enabled,
                agent_registry_subject: "qollective.agents.registry".to_string(),
                capability_subject: "qollective.agents.capabilities".to_string(),
                announcement_interval_ms: self.discovery.heartbeat_interval_ms,
                ttl_ms: self.discovery.ttl_ms,
                auto_register: true,
            },
        }
    }

    /// Convert to framework NatsClientConfig for A2A clients
    pub fn to_framework_client_config(&self, tls_config: &TlsExampleConfig) -> qollective::config::nats::NatsClientConfig {
        qollective::config::nats::NatsClientConfig {
            connection: qollective::config::nats::NatsConnectionConfig {
                urls: self.connection.urls.clone(),
                connection_timeout_ms: self.connection_timeout_ms,
                reconnect_timeout_ms: self.reconnect_timeout_ms,
                max_reconnect_attempts: Some(self.max_reconnect_attempts),
                username: None,
                password: None,
                token: None,
                tls: tls_config.to_framework_tls_config(),
                crypto_provider_strategy: None,
                custom_headers: HashMap::new(),
                client_name: Some(self.client.client_name.clone()),
            },
            client_behavior: qollective::config::nats::NatsClientBehaviorConfig {
                request_timeout_ms: 30000,
                max_pending_messages: self.client.max_pending_messages,
                retry_attempts: self.max_reconnect_attempts,
                retry_delay_ms: 1000,
                connection_pool_size: 10,
            },
            discovery_cache_ttl_ms: self.discovery.ttl_ms,
        }
    }
}

impl TlsExampleConfig {
    /// Convert to framework TlsConfig with enhanced path resolution
    pub fn to_framework_tls_config(&self) -> qollective::config::tls::TlsConfig {
        if !self.enabled {
            return qollective::config::tls::TlsConfig {
                enabled: false,
                ca_cert_path: None,
                cert_path: None,
                key_path: None,
                verification_mode: qollective::config::tls::VerificationMode::MutualTls,
            };
        }

        qollective::config::tls::TlsConfig {
            enabled: self.enabled,
            ca_cert_path: Some(self.resolve_certificate_path(&self.ca_cert_path, crate::constants::tls_env::TLS_CA_CERT_PATH)),
            cert_path: Some(self.resolve_certificate_path(&self.cert_path, crate::constants::tls_env::TLS_CERT_PATH)),
            key_path: Some(self.resolve_certificate_path(&self.key_path, crate::constants::tls_env::TLS_KEY_PATH)),
            verification_mode: self.parse_verification_mode(),
        }
    }

    /// Smart certificate path resolution with fallback mechanisms
    /// 
    /// Resolution order:
    /// 1. Environment variable override (if set)
    /// 2. Absolute path (if path starts with '/')
    /// 3. Relative path resolved from TLS base path constants
    /// 4. Fallback to framework default paths
    fn resolve_certificate_path(&self, config_path: &str, env_var: &str) -> std::path::PathBuf {
        use qollective::constants::network::tls_paths;
        use crate::constants::tls_env;
        use std::path::PathBuf;
        use std::env;

        // 1. Check for environment variable override first
        if let Ok(env_path) = env::var(env_var) {
            return PathBuf::from(env_path);
        }

        // 2. If absolute path, use as-is
        if config_path.starts_with('/') {
            return PathBuf::from(config_path);
        }

        // 3. For relative paths, resolve through framework constants with fallback
        let base_path = self.resolve_tls_base_path();
        
        // Determine which certificate file to resolve using constants
        match env_var {
            env_var if env_var == tls_env::TLS_CA_CERT_PATH => {
                PathBuf::from(tls_paths::ca_file_path(&base_path))
            },
            env_var if env_var == tls_env::TLS_CERT_PATH => {
                PathBuf::from(tls_paths::cert_file_path(&base_path))
            },
            env_var if env_var == tls_env::TLS_KEY_PATH => {
                PathBuf::from(tls_paths::key_file_path(&base_path))
            },
            _ => {
                // Fallback: construct path by joining base path with filename
                PathBuf::from(&base_path).join(config_path)
            }
        }
    }

    /// Resolve TLS base path with environment variable override support
    fn resolve_tls_base_path(&self) -> String {
        use qollective::constants::network::tls_paths;
        use crate::constants::tls_env;
        use std::env;

        // Check for base path environment variable override
        if let Ok(base_path) = env::var(tls_env::TLS_CERT_BASE_PATH) {
            return base_path;
        }

        // Use framework constant resolution
        tls_paths::resolve_tls_cert_base_path()
    }

    /// Parse verification mode with fallback to mutual TLS
    fn parse_verification_mode(&self) -> qollective::config::tls::VerificationMode {
        match self.verification_mode.as_str() {
            "mutual_tls" => qollective::config::tls::VerificationMode::MutualTls,
            "skip" => qollective::config::tls::VerificationMode::Skip,
            _ => {
                // Log warning for unknown verification mode
                eprintln!("Warning: Unknown TLS verification mode '{}', defaulting to 'mutual_tls'", 
                         self.verification_mode);
                qollective::config::tls::VerificationMode::MutualTls
            }
        }
    }

    /// Validate certificate paths exist (when enabled)
    pub fn validate_certificate_paths(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        let framework_config = self.to_framework_tls_config();
        
        // Check CA certificate
        if let Some(ca_path) = &framework_config.ca_cert_path {
            if !ca_path.exists() {
                return Err(format!("CA certificate not found: {}", ca_path.display()).into());
            }
            if !ca_path.is_file() {
                return Err(format!("CA certificate path is not a file: {}", ca_path.display()).into());
            }
        }

        // Check client certificate
        if let Some(cert_path) = &framework_config.cert_path {
            if !cert_path.exists() {
                return Err(format!("Client certificate not found: {}", cert_path.display()).into());
            }
            if !cert_path.is_file() {
                return Err(format!("Client certificate path is not a file: {}", cert_path.display()).into());
            }
        }

        // Check private key
        if let Some(key_path) = &framework_config.key_path {
            if !key_path.exists() {
                return Err(format!("Private key not found: {}", key_path.display()).into());
            }
            if !key_path.is_file() {
                return Err(format!("Private key path is not a file: {}", key_path.display()).into());
            }
        }

        Ok(())
    }

    /// Get comprehensive TLS configuration summary for debugging
    pub fn get_certificate_paths_summary(&self) -> String {
        if !self.enabled {
            return "TLS disabled".to_string();
        }

        let framework_config = self.to_framework_tls_config();
        
        let mut summary = format!(
            "TLS Configuration Summary:\n\
             - Enabled: {}\n\
             - Insecure Mode: {}\n\
             - Verification Mode: {:?}\n\
             - Handshake Timeout: {}ms\n\
             - CA Certificate: {}\n\
             - Client Certificate: {}\n\
             - Private Key: {}",
            self.enabled,
            self.insecure,
            framework_config.verification_mode,
            self.handshake_timeout_ms,
            framework_config.ca_cert_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Not configured".to_string()),
            framework_config.cert_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Not configured".to_string()),
            framework_config.key_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Not configured".to_string())
        );

        // Add optional configuration details
        if let Some(server_name) = &self.server_name {
            summary.push_str(&format!("\n - Server Name (SNI): {}", server_name));
        }

        if !self.protocol_versions.is_empty() {
            summary.push_str(&format!("\n - Protocol Versions: {:?}", self.protocol_versions));
        }

        if !self.cipher_suites.is_empty() {
            summary.push_str(&format!("\n - Cipher Suites: {:?}", self.cipher_suites));
        }

        if !self.alpn_protocols.is_empty() {
            summary.push_str(&format!("\n - ALPN Protocols: {:?}", self.alpn_protocols));
        }

        // Add certificate validation details
        summary.push_str(&format!(
            "\n - Certificate Validation:\n\
               - Verify Chain: {}\n\
               - Verify Expiration: {}\n\
               - Verify Hostname: {}\n\
               - Allow Self-Signed: {}\n\
               - Max Chain Depth: {}\n\
               - Revocation Check: {}",
            self.certificate_validation.verify_chain,
            self.certificate_validation.verify_expiration,
            self.certificate_validation.verify_hostname,
            self.certificate_validation.allow_self_signed,
            self.certificate_validation.max_chain_depth,
            self.certificate_validation.revocation_check
        ));

        summary
    }

    /// Get environment variable override summary
    pub fn get_env_override_summary(&self) -> String {
        use crate::constants::tls_env;
        use std::env;

        let mut overrides = Vec::new();

        if env::var(tls_env::TLS_CERT_BASE_PATH).is_ok() {
            overrides.push(format!("{}: {}", tls_env::TLS_CERT_BASE_PATH, 
                                 env::var(tls_env::TLS_CERT_BASE_PATH).unwrap()));
        }

        if env::var(tls_env::TLS_CA_CERT_PATH).is_ok() {
            overrides.push(format!("{}: {}", tls_env::TLS_CA_CERT_PATH, 
                                 env::var(tls_env::TLS_CA_CERT_PATH).unwrap()));
        }

        if env::var(tls_env::TLS_CERT_PATH).is_ok() {
            overrides.push(format!("{}: {}", tls_env::TLS_CERT_PATH, 
                                 env::var(tls_env::TLS_CERT_PATH).unwrap()));
        }

        if env::var(tls_env::TLS_KEY_PATH).is_ok() {
            overrides.push(format!("{}: {}", tls_env::TLS_KEY_PATH, 
                                 env::var(tls_env::TLS_KEY_PATH).unwrap()));
        }

        if overrides.is_empty() {
            "No environment variable overrides active".to_string()
        } else {
            format!("Active Environment Variable Overrides:\n{}", 
                   overrides.join("\n"))
        }
    }
}

impl A2AServerExampleConfig {
    /// Convert to framework A2AServerConfig
    pub fn to_framework_config(
        &self,
        enterprise_config: &EnterpriseServerConfig,
        nats_client_config: qollective::config::nats::NatsClientConfig,
    ) -> qollective::config::a2a::A2AServerConfig {
        qollective::config::a2a::A2AServerConfig {
            server_id: enterprise_config.server_id.clone(),
            server_name: enterprise_config.server_name.clone(),
            registry: qollective::config::a2a::RegistryConfig {
                agent_ttl: Duration::from_secs(self.registry.agent_ttl_secs),
                cleanup_interval: Duration::from_secs(self.registry.cleanup_interval_secs),
                max_agents: self.registry.max_agents,
                enable_health_monitoring: self.registry.enable_health_monitoring,
                enable_agent_logging: self.registry.enable_agent_logging,
                agent_log_subject: self.subjects.agent_registry_events.clone(),
                enable_capability_indexing: self.registry.enable_capability_indexing,
                max_capabilities_per_agent: self.registry.max_capabilities_per_agent,
                logging_agent_capability: self.registry.logging_agent_capability.clone(),
            },
            routing: qollective::config::a2a::RoutingConfig::default(),
            health: qollective::config::a2a::HealthConfig::default(),
            transport: qollective::config::a2a::AgentTransportConfig::default(),
            nats_server: qollective::config::nats::NatsServerConfig {
                enabled: false, // We connect to external NATS
                ..Default::default()
            },
            nats_client: nats_client_config,
            max_concurrent_requests: 1000,
            request_timeout: Duration::from_secs(30),
            enable_request_queuing: true,
            max_queue_size: 10000,
            enable_rate_limiting: false,
            requests_per_second: 1000,
        }
    }
}

impl A2AClientExampleConfig {
    /// Convert to framework A2AClientConfig
    pub fn to_framework_config(
        &self,
        agent_config: &AgentExampleConfig,
        agent_id: &str,
        agent_name: &str,
        nats_client_config: qollective::config::nats::NatsClientConfig,
    ) -> qollective::config::a2a::A2AClientConfig {
        qollective::config::a2a::A2AClientConfig {
            client: qollective::config::a2a::AgentClientConfig {
                agent_id: agent_id.to_string(),
                agent_name: agent_name.to_string(),
                capabilities: agent_config.capabilities.clone(),
                nats_url: nats_client_config.connection.urls[0].clone(),
                endpoint: None,
                heartbeat_interval: Duration::from_secs(self.heartbeat_interval_secs),
                discovery_cache_ttl: Duration::from_secs(self.discovery_cache_ttl_secs),
                retry_config: qollective::config::a2a::RetryConfig {
                    max_retries: self.retry.max_retries,
                    initial_delay: Duration::from_millis(self.retry.initial_delay_ms),
                    max_delay: Duration::from_millis(self.retry.max_delay_ms),
                    backoff_multiplier: self.retry.backoff_multiplier,
                },
                subject_config: qollective::config::a2a::A2ASubjectConfig {
                    prefix: "qollective.agents".to_string(),
                    request_pattern: "qollective.agents.{agent_id}.request".to_string(),
                    notification_pattern: "qollective.agents.{agent_id}.notification".to_string(),
                    heartbeat_pattern: "qollective.agents.{agent_id}.heartbeat".to_string(),
                    discovery_pattern: "qollective.agents.discovery".to_string(),
                },
                enable_metrics: self.enable_metrics,
                metadata: HashMap::from([
                    ("location".to_string(), agent_config.location.clone()),
                    ("function".to_string(), agent_config.function.clone()),
                    ("service_type".to_string(), agent_config.service_type.clone()),
                ]),
            },
            transport: Default::default(),
            nats_client: nats_client_config,
            discovery_cache_ttl_ms: self.discovery_cache_ttl_secs * 1000,
        }
    }
}