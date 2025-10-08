// ABOUTME: Preset configurations for different environments and use cases
// ABOUTME: Provides production, development, debugging, and high-performance configurations

//! Preset configurations for different environments and use cases.

use super::meta::{MetaConfig, MetaSectionConfig, PropertyConfig};
use crate::constants::network;
use serde::{Deserialize, Serialize};

#[cfg(feature = "tenant-extraction")]
use crate::tenant::extraction::ExtractionConfig;

/// REST-specific configuration for both client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestConfig {
    pub client: Option<RestClientConfig>,
    pub server: Option<RestServerConfig>,
}

/// REST client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestClientConfig {
    pub base_url: Option<String>,
    pub timeout_ms: u64,
    pub max_connections: usize,
    pub user_agent: String,
    pub default_headers: std::collections::HashMap<String, String>,
    pub retry_attempts: u32,
    pub tls: TlsConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
    pub tenant_config: TenantClientConfig,
}

/// Tenant-specific client configuration for REST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantClientConfig {
    /// Whether to automatically propagate tenant context from current context
    pub auto_propagate_tenant: bool,
    /// Override tenant ID for all outgoing requests (overrides context-based tenant)
    pub override_tenant_id: Option<String>,
    /// Whether to propagate onBehalfOf metadata
    pub propagate_on_behalf_of: bool,
    /// Fallback tenant ID when no context is available
    pub fallback_tenant_id: Option<String>,
}

impl Default for TenantClientConfig {
    fn default() -> Self {
        Self {
            auto_propagate_tenant: true,
            override_tenant_id: None,
            propagate_on_behalf_of: true,
            fallback_tenant_id: None,
        }
    }
}

/// REST server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
    pub cors: CorsConfig,
    pub tls: TlsConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
}

/// TLS configuration for REST (using unified TLS config)
pub use crate::config::tls::TlsConfig;

/// CORS configuration for REST server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u64,
}

/// Logging configuration for REST operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub log_requests: bool,
    pub log_responses: bool,
    pub log_headers: bool,
    pub log_body: bool,
    pub log_level: String,
    pub structured_logging: bool,
}

/// Performance configuration for REST operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enabled: bool,
    pub track_request_duration: bool,
    pub track_response_size: bool,
    pub track_connection_pool: bool,
    pub benchmarking_enabled: bool,
    pub metrics_collection: bool,
}

/// Main configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QollectiveConfig {
    pub tenant_extraction_enabled: bool,
    pub meta: MetaConfig,
    pub rest: Option<RestConfig>,

    #[cfg(feature = "grpc-client")]
    pub grpc_client: Option<super::grpc::GrpcClientConfig>,

    #[cfg(feature = "grpc-server")]
    pub grpc_server: Option<super::grpc::GrpcServerConfig>,

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub nats: Option<super::nats::NatsConfig>,

    #[cfg(feature = "tenant-extraction")]
    pub jwt_extraction: Option<ExtractionConfig>,
}

/// Predefined configuration presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPreset {
    Production,
    Development,
    Staging,
    HighPerformance,
    Debugging,
}

impl ConfigPreset {
    pub fn to_config(&self) -> QollectiveConfig {
        match self {
            ConfigPreset::Production => production_config(),
            ConfigPreset::Development => development_config(),
            ConfigPreset::Staging => staging_config(),
            ConfigPreset::HighPerformance => high_performance_config(),
            ConfigPreset::Debugging => debugging_config(),
        }
    }
}

impl QollectiveConfig {
    /// Create a new QollectiveConfig with sensible defaults
    pub fn new() -> Self {
        ConfigPreset::Development.to_config()
    }

    /// Validate the entire configuration including all sub-configurations
    pub fn validate(&self) -> Result<(), String> {
        // Validate NATS configuration if present
        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        if let Some(ref nats_config) = self.nats {
            nats_config.connection.validate()?;
            nats_config.client.validate()?;
            nats_config.server.validate()?;
            nats_config.discovery.validate()?;
        }

        // Add other validations here as needed
        Ok(())
    }

    /// Create a configured TenantExtractor based on the current configuration
    #[cfg(feature = "tenant-extraction")]
    pub fn create_tenant_extractor(&self) -> Option<crate::tenant::TenantExtractor> {
        if !self.tenant_extraction_enabled {
            return None;
        }

        if let Some(ref jwt_config) = self.jwt_extraction {
            Some(crate::tenant::TenantExtractor::with_config(
                jwt_config.clone(),
            ))
        } else {
            Some(crate::tenant::TenantExtractor::new())
        }
    }

    /// Get JWT extraction configuration if available
    #[cfg(feature = "tenant-extraction")]
    pub fn jwt_extraction_config(&self) -> Option<&ExtractionConfig> {
        self.jwt_extraction.as_ref()
    }

    /// Get NATS configuration if available
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn nats_config(&self) -> Option<&super::nats::NatsConfig> {
        self.nats.as_ref()
    }

    /// Create a builder for QollectiveConfig
    pub fn builder() -> QollectiveConfigBuilder {
        QollectiveConfigBuilder::new()
    }
}

/// Builder for QollectiveConfig with fluent API
pub struct QollectiveConfigBuilder {
    config: QollectiveConfig,
}

impl QollectiveConfigBuilder {
    /// Create a new builder with default development configuration
    pub fn new() -> Self {
        Self {
            config: ConfigPreset::Development.to_config(),
        }
    }

    /// Start with a specific preset
    pub fn from_preset(preset: ConfigPreset) -> Self {
        Self {
            config: preset.to_config(),
        }
    }

    /// Enable or disable tenant extraction
    pub fn with_tenant_extraction(mut self, enabled: bool) -> Self {
        self.config.tenant_extraction_enabled = enabled;
        self
    }

    /// Set NATS URLs
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_urls(mut self, urls: Vec<String>) -> Self {
        if let Some(ref mut nats_config) = self.config.nats {
            nats_config.connection.urls = urls;
        }
        self
    }

    /// Enable or disable NATS server
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_server_enabled(mut self, enabled: bool) -> Self {
        if let Some(ref mut nats_config) = self.config.nats {
            nats_config.server.enabled = enabled;
        }
        self
    }

    /// Enable or disable NATS discovery
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_discovery_enabled(mut self, enabled: bool) -> Self {
        if let Some(ref mut nats_config) = self.config.nats {
            nats_config.discovery.enabled = enabled;
        }
        self
    }

    /// Set NATS authentication credentials
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_credentials(mut self, username: String, password: String) -> Self {
        if let Some(ref mut nats_config) = self.config.nats {
            nats_config.connection.username = Some(username);
            nats_config.connection.password = Some(password);
        }
        self
    }

    /// Enable or disable NATS TLS
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub fn with_nats_tls(mut self, enabled: bool) -> Self {
        if let Some(ref mut nats_config) = self.config.nats {
            nats_config.connection.tls.enabled = enabled;
            // When enabling TLS, provide default certificate paths for testing
            if enabled {
                nats_config.connection.tls.cert_path = Some("/path/to/test-cert.pem".into());
                nats_config.connection.tls.key_path = Some("/path/to/test-key.pem".into());
                nats_config.connection.tls.verification_mode =
                    crate::config::tls::VerificationMode::Skip;
            }
        }
        self
    }

    /// Build and validate the configuration
    pub fn build(self) -> Result<QollectiveConfig, String> {
        self.config.validate()?;
        Ok(self.config)
    }
}

fn production_config() -> QollectiveConfig {
    QollectiveConfig {
        tenant_extraction_enabled: true,
        meta: MetaConfig {
            security: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::Specific(
                    [
                        ("user_id".to_string(), true),
                        ("session_id".to_string(), true),
                        ("ip_address".to_string(), false),
                    ]
                    .into_iter()
                    .collect(),
                ),
            }),
            debug: Some(MetaSectionConfig {
                enabled: false,
                properties: PropertyConfig::None,
            }),
            performance: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::Specific(
                    [("db_query_time".to_string(), true)].into_iter().collect(),
                ),
            }),
            monitoring: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            tracing: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            extensions: None,
        },
        rest: Some(RestConfig {
            client: Some(RestClientConfig {
                base_url: None,
                timeout_ms: 30000,
                max_connections: 100,
                user_agent: "qollective-client/1.0".to_string(),
                default_headers: [
                    ("Accept".to_string(), "application/json".to_string()),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ]
                .into_iter()
                .collect(),
                retry_attempts: 3,
                tls: TlsConfig {
                    enabled: true,
                    cert_path: Some("/etc/ssl/certs/rest-client.crt".into()),
                    key_path: Some("/etc/ssl/private/rest-client.key".into()),
                    ca_cert_path: None,
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                logging: LoggingConfig {
                    enabled: true,
                    log_requests: true,
                    log_responses: false,
                    log_headers: false,
                    log_body: false,
                    log_level: "info".to_string(),
                    structured_logging: true,
                },
                performance: PerformanceConfig {
                    enabled: true,
                    track_request_duration: true,
                    track_response_size: true,
                    track_connection_pool: true,
                    benchmarking_enabled: false,
                    metrics_collection: true,
                },
                tenant_config: TenantClientConfig::default(),
            }),
            server: Some(RestServerConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
                max_connections: 1000,
                request_timeout_ms: 30000,
                cors: CorsConfig {
                    enabled: false,
                    allowed_origins: vec!["https://yourdomain.com".to_string()],
                    allowed_methods: vec![
                        "GET".to_string(),
                        "POST".to_string(),
                        "PUT".to_string(),
                        "DELETE".to_string(),
                    ],
                    allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                    max_age_seconds: 3600,
                },
                tls: TlsConfig {
                    enabled: true,
                    cert_path: Some("/etc/ssl/certs/server.crt".into()),
                    key_path: Some("/etc/ssl/private/server.key".into()),
                    ca_cert_path: None,
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                logging: LoggingConfig {
                    enabled: true,
                    log_requests: true,
                    log_responses: false,
                    log_headers: false,
                    log_body: false,
                    log_level: "info".to_string(),
                    structured_logging: true,
                },
                performance: PerformanceConfig {
                    enabled: true,
                    track_request_duration: true,
                    track_response_size: true,
                    track_connection_pool: true,
                    benchmarking_enabled: false,
                    metrics_collection: true,
                },
            }),
        }),

        #[cfg(feature = "grpc-client")]
        grpc_client: Some(super::grpc::GrpcClientConfig {
            base_url: None,
            timeout_ms: 30000,
            max_connections: 100,
            user_agent: "qollective-grpc-client/1.0".to_string(),
            default_headers: std::collections::HashMap::new(),
            retry_attempts: 3,
            tls: TlsConfig {
                enabled: true,
                cert_path: Some("/etc/ssl/certs/grpc-client.crt".into()),
                key_path: Some("/etc/ssl/private/grpc-client.key".into()),
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::SystemCa,
            },
            logging: LoggingConfig {
                enabled: true,
                log_requests: true,
                log_responses: false,
                log_headers: false,
                log_body: false,
                log_level: "info".to_string(),
                structured_logging: true,
            },
            performance: PerformanceConfig {
                enabled: true,
                track_request_duration: true,
                track_response_size: true,
                track_connection_pool: true,
                benchmarking_enabled: false,
                metrics_collection: true,
            },
            connection_pool: super::grpc::ConnectionPoolConfig {
                enabled: true,
                max_idle_connections: 20,
                idle_timeout_ms: 300000,
                connection_timeout_ms: 10000,
                keep_alive_time_ms: 60000,
                keep_alive_timeout_ms: 5000,
                keep_alive_while_idle: false,
            },
            health_check: super::grpc::HealthCheckConfig {
                enabled: true,
                interval_ms: 30000,
                timeout_ms: 5000,
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                service_names: vec!["qollective.health".to_string()],
            },
            jwt_config: super::grpc::GrpcJwtConfig::default(),
            tenant_config: crate::client::common::TenantClientConfig::default(),
        }),

        #[cfg(feature = "grpc-server")]
        grpc_server: Some(super::grpc::GrpcServerConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 50051,
            max_connections: 1000,
            request_timeout_ms: 30000,
            tls: TlsConfig {
                enabled: true,
                cert_path: Some("/etc/ssl/certs/grpc-server.crt".into()),
                key_path: Some("/etc/ssl/private/grpc-server.key".into()),
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::SystemCa,
            },
            logging: LoggingConfig {
                enabled: true,
                log_requests: true,
                log_responses: false,
                log_headers: false,
                log_body: false,
                log_level: "info".to_string(),
                structured_logging: true,
            },
            performance: PerformanceConfig {
                enabled: true,
                track_request_duration: true,
                track_response_size: true,
                track_connection_pool: true,
                benchmarking_enabled: false,
                metrics_collection: true,
            },
            health_check: super::grpc::HealthCheckConfig {
                enabled: true,
                interval_ms: 30000,
                timeout_ms: 5000,
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                service_names: vec!["qollective.health".to_string()],
            },
            reflection: super::grpc::ReflectionConfig {
                enabled: false,
                include_services: vec![],
                exclude_services: vec![],
            },
            concurrency: super::grpc::ConcurrencyConfig {
                max_concurrent_streams: 1000,
                max_frame_size: 16384,
                initial_window_size: 65535,
                initial_connection_window_size: 65535,
                max_header_list_size: 16384,
            },
        }),

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        nats: Some(super::nats::NatsConfig {
            connection: super::nats::NatsConnectionConfig {
                urls: vec!["nats://localhost:4222".to_string()],
                connection_timeout_ms: 5000,
                reconnect_timeout_ms: 2000,
                max_reconnect_attempts: Some(5),
                username: None,
                password: None,
                token: None,
                nkey_file: None,
                nkey_seed: None,
                tls: crate::config::tls::TlsConfig {
                    enabled: true, // Production should use TLS
                    ca_cert_path: None,
                    cert_path: Some("/etc/ssl/certs/nats-client.crt".into()),
                    key_path: Some("/etc/ssl/private/nats-client.key".into()),
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                crypto_provider_strategy: None,
                custom_headers: std::collections::HashMap::new(),
                client_name: Some(network::client_names::A2A_SERVER.to_string()),
            },
            client: super::nats::NatsClientBehaviorConfig {
                request_timeout_ms: 30000,
                max_pending_messages: 512,
                retry_attempts: 3,
                retry_delay_ms: 1000,
                connection_pool_size: 5,
            },
            server: super::nats::NatsServerConfig {
                enabled: false, // Production starts with server disabled by default
                subject_prefix: "qollective".to_string(),
                queue_group: Some("qollective-workers".to_string()),
                max_concurrent_handlers: 50,
                handler_timeout_ms: 30000,
                enable_request_reply: true,
            },
            discovery: super::nats::NatsDiscoveryConfig {
                enabled: false, // Production starts with discovery disabled by default
                agent_registry_subject: "qollective.agents".to_string(),
                capability_subject: "qollective.capabilities".to_string(),
                announcement_interval_ms: 30000,
                ttl_ms: 90000,
                auto_register: true,
            },
        }),

        #[cfg(feature = "tenant-extraction")]
        jwt_extraction: Some(ExtractionConfig {
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
        }),
    }
}

fn development_config() -> QollectiveConfig {
    QollectiveConfig {
        tenant_extraction_enabled: true,
        meta: MetaConfig {
            security: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            debug: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            performance: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            monitoring: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            tracing: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::All,
            }),
            extensions: None,
        },
        rest: Some(RestConfig {
            client: Some(RestClientConfig {
                base_url: Some("http://localhost:8080".to_string()),
                timeout_ms: 10000,
                max_connections: 50,
                user_agent: "qollective-dev-client/1.0".to_string(),
                default_headers: [
                    ("Accept".to_string(), "application/json".to_string()),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ]
                .into_iter()
                .collect(),
                retry_attempts: 1,
                tls: TlsConfig {
                    enabled: false,
                    cert_path: None,
                    key_path: None,
                    ca_cert_path: None,
                    verification_mode: crate::config::tls::VerificationMode::Skip,
                },
                logging: LoggingConfig {
                    enabled: true,
                    log_requests: true,
                    log_responses: true,
                    log_headers: true,
                    log_body: true,
                    log_level: "debug".to_string(),
                    structured_logging: true,
                },
                performance: PerformanceConfig {
                    enabled: true,
                    track_request_duration: true,
                    track_response_size: true,
                    track_connection_pool: true,
                    benchmarking_enabled: true,
                    metrics_collection: true,
                },
                tenant_config: TenantClientConfig::default(),
            }),
            server: Some(RestServerConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 100,
                request_timeout_ms: 10000,
                cors: CorsConfig {
                    enabled: true,
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec![
                        "GET".to_string(),
                        "POST".to_string(),
                        "PUT".to_string(),
                        "DELETE".to_string(),
                        "OPTIONS".to_string(),
                    ],
                    allowed_headers: vec!["*".to_string()],
                    max_age_seconds: 86400,
                },
                tls: TlsConfig {
                    enabled: false,
                    cert_path: None,
                    key_path: None,
                    ca_cert_path: None,
                    verification_mode: crate::config::tls::VerificationMode::Skip,
                },
                logging: LoggingConfig {
                    enabled: true,
                    log_requests: true,
                    log_responses: true,
                    log_headers: true,
                    log_body: true,
                    log_level: "debug".to_string(),
                    structured_logging: true,
                },
                performance: PerformanceConfig {
                    enabled: true,
                    track_request_duration: true,
                    track_response_size: true,
                    track_connection_pool: true,
                    benchmarking_enabled: true,
                    metrics_collection: true,
                },
            }),
        }),

        #[cfg(feature = "grpc-client")]
        grpc_client: Some(super::grpc::GrpcClientConfig {
            base_url: Some("http://localhost:50051".to_string()),
            timeout_ms: 10000,
            max_connections: 50,
            user_agent: "qollective-grpc-dev-client/1.0".to_string(),
            default_headers: std::collections::HashMap::new(),
            retry_attempts: 1,
            tls: TlsConfig {
                enabled: false,
                cert_path: None,
                key_path: None,
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::Skip,
            },
            logging: LoggingConfig {
                enabled: true,
                log_requests: true,
                log_responses: true,
                log_headers: true,
                log_body: true,
                log_level: "debug".to_string(),
                structured_logging: true,
            },
            performance: PerformanceConfig {
                enabled: true,
                track_request_duration: true,
                track_response_size: true,
                track_connection_pool: true,
                benchmarking_enabled: true,
                metrics_collection: true,
            },
            connection_pool: super::grpc::ConnectionPoolConfig::default(),
            health_check: super::grpc::HealthCheckConfig {
                enabled: false,
                ..super::grpc::HealthCheckConfig::default()
            },
            jwt_config: super::grpc::GrpcJwtConfig::default(),
            tenant_config: crate::client::common::TenantClientConfig::default(),
        }),

        #[cfg(feature = "grpc-server")]
        grpc_server: Some(super::grpc::GrpcServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 50051,
            max_connections: 100,
            request_timeout_ms: 10000,
            tls: TlsConfig {
                enabled: false,
                cert_path: None,
                key_path: None,
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::Skip,
            },
            logging: LoggingConfig {
                enabled: true,
                log_requests: true,
                log_responses: true,
                log_headers: true,
                log_body: true,
                log_level: "debug".to_string(),
                structured_logging: true,
            },
            performance: PerformanceConfig {
                enabled: true,
                track_request_duration: true,
                track_response_size: true,
                track_connection_pool: true,
                benchmarking_enabled: true,
                metrics_collection: true,
            },
            health_check: super::grpc::HealthCheckConfig {
                enabled: false,
                ..super::grpc::HealthCheckConfig::default()
            },
            reflection: super::grpc::ReflectionConfig {
                enabled: true,
                include_services: vec!["qollective.health".to_string()],
                exclude_services: vec![],
            },
            concurrency: super::grpc::ConcurrencyConfig::default(),
        }),

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        nats: Some(super::nats::NatsConfig {
            connection: super::nats::NatsConnectionConfig {
                urls: vec!["nats://localhost:4222".to_string()],
                connection_timeout_ms: 5000,
                reconnect_timeout_ms: 2000,
                max_reconnect_attempts: Some(5),
                username: None,
                password: None,
                token: None,
                nkey_file: None,
                nkey_seed: None,
                tls: crate::config::tls::TlsConfig {
                    enabled: false, // Development uses plain connections
                    ca_cert_path: None,
                    cert_path: None,
                    key_path: None,
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                crypto_provider_strategy: None,
                custom_headers: std::collections::HashMap::new(),
                client_name: Some(network::client_names::A2A_SERVER.to_string()),
            },
            client: super::nats::NatsClientBehaviorConfig {
                request_timeout_ms: 30000,
                max_pending_messages: 512,
                retry_attempts: 3,
                retry_delay_ms: 1000,
                connection_pool_size: 5,
            },
            server: super::nats::NatsServerConfig {
                enabled: true, // Development enables server for testing
                subject_prefix: "qollective".to_string(),
                queue_group: Some("qollective-dev".to_string()),
                max_concurrent_handlers: 50,
                handler_timeout_ms: 30000,
                enable_request_reply: true,
            },
            discovery: super::nats::NatsDiscoveryConfig {
                enabled: true, // Development enables discovery for testing
                agent_registry_subject: "qollective.agents".to_string(),
                capability_subject: "qollective.capabilities".to_string(),
                announcement_interval_ms: 30000,
                ttl_ms: 90000,
                auto_register: true,
            },
        }),

        #[cfg(feature = "tenant-extraction")]
        jwt_extraction: Some(ExtractionConfig {
            enabled: true,
            jwt_debug_logging: true, // Enable debug logging for development
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
        }),
    }
}

fn staging_config() -> QollectiveConfig {
    development_config()
}

fn high_performance_config() -> QollectiveConfig {
    QollectiveConfig {
        tenant_extraction_enabled: true,
        meta: MetaConfig {
            security: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::Specific(
                    [("user_id".to_string(), true)].into_iter().collect(),
                ),
            }),
            debug: Some(MetaSectionConfig {
                enabled: false,
                properties: PropertyConfig::None,
            }),
            performance: Some(MetaSectionConfig {
                enabled: false,
                properties: PropertyConfig::None,
            }),
            monitoring: Some(MetaSectionConfig {
                enabled: false,
                properties: PropertyConfig::None,
            }),
            tracing: Some(MetaSectionConfig {
                enabled: true,
                properties: PropertyConfig::Specific(
                    [("trace_id".to_string(), true)].into_iter().collect(),
                ),
            }),
            extensions: None,
        },
        rest: Some(RestConfig {
            client: Some(RestClientConfig {
                base_url: None,
                timeout_ms: 5000,
                max_connections: 200,
                user_agent: "qollective-high-perf/1.0".to_string(),
                default_headers: [("Accept".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect(),
                retry_attempts: 0,
                tls: TlsConfig {
                    enabled: true,
                    cert_path: None,
                    key_path: None,
                    ca_cert_path: None,
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                logging: LoggingConfig {
                    enabled: false,
                    log_requests: false,
                    log_responses: false,
                    log_headers: false,
                    log_body: false,
                    log_level: "error".to_string(),
                    structured_logging: false,
                },
                performance: PerformanceConfig {
                    enabled: false,
                    track_request_duration: false,
                    track_response_size: false,
                    track_connection_pool: false,
                    benchmarking_enabled: false,
                    metrics_collection: false,
                },
                tenant_config: TenantClientConfig {
                    auto_propagate_tenant: false,
                    override_tenant_id: None,
                    propagate_on_behalf_of: false,
                    fallback_tenant_id: None,
                },
            }),
            server: Some(RestServerConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
                max_connections: 10000,
                request_timeout_ms: 5000,
                cors: CorsConfig {
                    enabled: false,
                    allowed_origins: vec![],
                    allowed_methods: vec![],
                    allowed_headers: vec![],
                    max_age_seconds: 0,
                },
                tls: TlsConfig {
                    enabled: true,
                    cert_path: Some("/etc/ssl/certs/server.crt".into()),
                    key_path: Some("/etc/ssl/private/server.key".into()),
                    ca_cert_path: None,
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                logging: LoggingConfig {
                    enabled: false,
                    log_requests: false,
                    log_responses: false,
                    log_headers: false,
                    log_body: false,
                    log_level: "error".to_string(),
                    structured_logging: false,
                },
                performance: PerformanceConfig {
                    enabled: false,
                    track_request_duration: false,
                    track_response_size: false,
                    track_connection_pool: false,
                    benchmarking_enabled: false,
                    metrics_collection: false,
                },
            }),
        }),

        #[cfg(feature = "grpc-client")]
        grpc_client: Some(super::grpc::GrpcClientConfig {
            base_url: None,
            timeout_ms: 5000,
            max_connections: 200,
            user_agent: "qollective-grpc-high-perf/1.0".to_string(),
            default_headers: std::collections::HashMap::new(),
            retry_attempts: 0,
            tls: TlsConfig {
                enabled: true,
                cert_path: None,
                key_path: None,
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::SystemCa,
            },
            logging: LoggingConfig {
                enabled: false,
                log_requests: false,
                log_responses: false,
                log_headers: false,
                log_body: false,
                log_level: "error".to_string(),
                structured_logging: false,
            },
            performance: PerformanceConfig {
                enabled: false,
                track_request_duration: false,
                track_response_size: false,
                track_connection_pool: false,
                benchmarking_enabled: false,
                metrics_collection: false,
            },
            connection_pool: super::grpc::ConnectionPoolConfig {
                enabled: true,
                max_idle_connections: 50,
                idle_timeout_ms: 30000,
                connection_timeout_ms: 2000,
                keep_alive_time_ms: 30000,
                keep_alive_timeout_ms: 2000,
                keep_alive_while_idle: false,
            },
            health_check: super::grpc::HealthCheckConfig {
                enabled: false,
                ..super::grpc::HealthCheckConfig::default()
            },
            jwt_config: super::grpc::GrpcJwtConfig::default(),
            tenant_config: crate::client::common::TenantClientConfig::default(),
        }),

        #[cfg(feature = "grpc-server")]
        grpc_server: Some(super::grpc::GrpcServerConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 50051,
            max_connections: 10000,
            request_timeout_ms: 5000,
            tls: TlsConfig {
                enabled: true,
                cert_path: Some("/etc/ssl/certs/grpc-server.crt".into()),
                key_path: Some("/etc/ssl/private/grpc-server.key".into()),
                ca_cert_path: None,
                verification_mode: crate::config::tls::VerificationMode::SystemCa,
            },
            logging: LoggingConfig {
                enabled: false,
                log_requests: false,
                log_responses: false,
                log_headers: false,
                log_body: false,
                log_level: "error".to_string(),
                structured_logging: false,
            },
            performance: PerformanceConfig {
                enabled: false,
                track_request_duration: false,
                track_response_size: false,
                track_connection_pool: false,
                benchmarking_enabled: false,
                metrics_collection: false,
            },
            health_check: super::grpc::HealthCheckConfig {
                enabled: false,
                ..super::grpc::HealthCheckConfig::default()
            },
            reflection: super::grpc::ReflectionConfig {
                enabled: false,
                include_services: vec![],
                exclude_services: vec![],
            },
            concurrency: super::grpc::ConcurrencyConfig {
                max_concurrent_streams: 10000,
                max_frame_size: 16384,
                initial_window_size: 1048576,
                initial_connection_window_size: 1048576,
                max_header_list_size: 8192,
            },
        }),

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        nats: Some(super::nats::NatsConfig {
            connection: super::nats::NatsConnectionConfig {
                urls: vec!["nats://localhost:4222".to_string()],
                connection_timeout_ms: 2000, // Faster timeout for performance
                reconnect_timeout_ms: 1000,
                max_reconnect_attempts: Some(3), // Fewer retries for performance
                username: None,
                password: None,
                token: None,
                nkey_file: None,
                nkey_seed: None,
                tls: crate::config::tls::TlsConfig {
                    enabled: true, // Performance with security
                    ca_cert_path: None,
                    cert_path: Some("/etc/ssl/certs/nats-client.crt".into()),
                    key_path: Some("/etc/ssl/private/nats-client.key".into()),
                    verification_mode: crate::config::tls::VerificationMode::SystemCa,
                },
                crypto_provider_strategy: None,
                custom_headers: std::collections::HashMap::new(),
                client_name: Some(network::client_names::A2A_SERVER.to_string()),
            },
            client: super::nats::NatsClientBehaviorConfig {
                request_timeout_ms: 15000,  // Shorter timeout for performance
                max_pending_messages: 1024, // Higher buffer for performance
                retry_attempts: 1,          // Minimal retries for performance
                retry_delay_ms: 500,
                connection_pool_size: 10, // Larger pool for performance
            },
            server: super::nats::NatsServerConfig {
                enabled: false, // Performance mode disables server by default
                subject_prefix: "qollective".to_string(),
                queue_group: Some("qollective-perf".to_string()),
                max_concurrent_handlers: 100, // Higher concurrency for performance
                handler_timeout_ms: 15000,    // Shorter timeout for performance
                enable_request_reply: true,
            },
            discovery: super::nats::NatsDiscoveryConfig {
                enabled: false, // Performance mode disables discovery by default
                agent_registry_subject: "qollective.agents".to_string(),
                capability_subject: "qollective.capabilities".to_string(),
                announcement_interval_ms: 60000, // Less frequent announcements
                ttl_ms: 120000,                  // Shorter TTL for performance
                auto_register: false,            // Manual registration for performance
            },
        }),

        #[cfg(feature = "tenant-extraction")]
        jwt_extraction: Some(ExtractionConfig {
            enabled: true,
            jwt_debug_logging: false, // Disable debug logging for performance
            tenant_header_names: vec!["X-Tenant-ID".to_string(), "X-Organization-ID".to_string()],
            tenant_payload_paths: vec!["tenant".to_string(), "tenant_id".to_string()],
            tenant_query_params: vec!["tenant".to_string()],
            auth_header_patterns: vec!["Bearer".to_string()],
        }),
    }
}

fn debugging_config() -> QollectiveConfig {
    development_config()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "tenant-extraction")]
    #[test]
    fn test_jwt_extraction_config_integration() {
        // Test production config
        let prod_config = ConfigPreset::Production.to_config();
        assert!(prod_config.tenant_extraction_enabled);
        assert!(prod_config.jwt_extraction.is_some());

        let jwt_config = prod_config.jwt_extraction.as_ref().unwrap();
        assert!(jwt_config.enabled);
        assert!(!jwt_config.jwt_debug_logging); // Production should have debug disabled
        assert!(!jwt_config.tenant_header_names.is_empty());
        assert!(!jwt_config.auth_header_patterns.is_empty());

        // Test development config
        let dev_config = ConfigPreset::Development.to_config();
        assert!(dev_config.tenant_extraction_enabled);
        assert!(dev_config.jwt_extraction.is_some());

        let dev_jwt_config = dev_config.jwt_extraction.as_ref().unwrap();
        assert!(dev_jwt_config.enabled);
        assert!(dev_jwt_config.jwt_debug_logging); // Development should have debug enabled

        // Test high performance config
        let perf_config = ConfigPreset::HighPerformance.to_config();
        assert!(perf_config.tenant_extraction_enabled);
        assert!(perf_config.jwt_extraction.is_some());

        let perf_jwt_config = perf_config.jwt_extraction.as_ref().unwrap();
        assert!(perf_jwt_config.enabled);
        assert!(!perf_jwt_config.jwt_debug_logging); // Performance should have debug disabled
                                                     // Performance config should have fewer headers for efficiency
        assert!(
            perf_jwt_config.tenant_header_names.len() <= dev_jwt_config.tenant_header_names.len()
        );
    }

    #[cfg(feature = "tenant-extraction")]
    #[test]
    fn test_tenant_extractor_creation_from_config() {
        let config = ConfigPreset::Development.to_config();

        // Test creating tenant extractor from config
        let extractor = config.create_tenant_extractor();
        assert!(extractor.is_some());

        // Test getting JWT extraction config
        let jwt_config = config.jwt_extraction_config();
        assert!(jwt_config.is_some());
        assert!(jwt_config.unwrap().enabled);
    }

    #[cfg(feature = "tenant-extraction")]
    #[test]
    fn test_disabled_tenant_extraction() {
        let mut config = ConfigPreset::Development.to_config();
        config.tenant_extraction_enabled = false;

        // Should return None when tenant extraction is disabled
        let extractor = config.create_tenant_extractor();
        assert!(extractor.is_none());
    }

    #[test]
    fn test_config_preset_conversion() {
        // Test that all presets can be converted to configs
        let production = ConfigPreset::Production.to_config();
        let development = ConfigPreset::Development.to_config();
        let staging = ConfigPreset::Staging.to_config();
        let high_performance = ConfigPreset::HighPerformance.to_config();
        let debugging = ConfigPreset::Debugging.to_config();

        // All configs should have tenant extraction enabled
        assert!(production.tenant_extraction_enabled);
        assert!(development.tenant_extraction_enabled);
        assert!(staging.tenant_extraction_enabled);
        assert!(high_performance.tenant_extraction_enabled);
        assert!(debugging.tenant_extraction_enabled);

        // Staging and debugging should be equivalent to development
        assert_eq!(
            staging.tenant_extraction_enabled,
            development.tenant_extraction_enabled
        );
        assert_eq!(
            debugging.tenant_extraction_enabled,
            development.tenant_extraction_enabled
        );
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_config_integration() {
        // Test that NATS config is present when features are enabled
        let config = QollectiveConfig::new();
        assert!(config.nats.is_some());

        // Test NATS config getter
        let nats_config = config.nats_config();
        assert!(nats_config.is_some());

        // Verify development defaults
        let nats = nats_config.unwrap();
        assert_eq!(nats.connection.urls, vec!["nats://localhost:4222"]);
        assert!(nats.server.enabled); // Development should have server enabled
        assert!(nats.discovery.enabled); // Development should have discovery enabled
        assert!(!nats.connection.tls.enabled); // Development should use plain connections
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_nats_preset_differences() {
        let production = ConfigPreset::Production.to_config();
        let development = ConfigPreset::Development.to_config();
        let high_performance = ConfigPreset::HighPerformance.to_config();

        // All should have NATS config
        assert!(production.nats.is_some());
        assert!(development.nats.is_some());
        assert!(high_performance.nats.is_some());

        let prod_nats = production.nats.unwrap();
        let dev_nats = development.nats.unwrap();
        let perf_nats = high_performance.nats.unwrap();

        // Production should use TLS, development should not
        assert!(prod_nats.connection.tls.enabled);
        assert!(!dev_nats.connection.tls.enabled);
        assert!(perf_nats.connection.tls.enabled);

        // Development should have server/discovery enabled
        assert!(dev_nats.server.enabled);
        assert!(dev_nats.discovery.enabled);

        // Production should have server/discovery disabled by default
        assert!(!prod_nats.server.enabled);
        assert!(!prod_nats.discovery.enabled);

        // Performance should have optimized settings
        assert!(!perf_nats.server.enabled);
        assert!(!perf_nats.discovery.enabled);
        assert_eq!(perf_nats.client.retry_attempts, 1); // Minimal retries
        assert_eq!(perf_nats.client.connection_pool_size, 10); // Larger pool
        assert!(!perf_nats.discovery.auto_register); // Manual registration
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_config_builder_with_nats() {
        let config = QollectiveConfigBuilder::new()
            .with_nats_urls(vec!["nats://custom:4222".to_string()])
            .with_nats_server_enabled(true)
            .with_nats_discovery_enabled(true)
            .with_nats_tls(true)
            .with_nats_credentials("user".to_string(), "pass".to_string())
            .build()
            .expect("Should build valid config");

        assert!(config.nats.is_some());
        let nats_config = config.nats.unwrap();
        assert_eq!(nats_config.connection.urls, vec!["nats://custom:4222"]);
        assert!(nats_config.server.enabled);
        assert!(nats_config.discovery.enabled);
        assert!(nats_config.connection.tls.enabled);
        assert_eq!(nats_config.connection.username, Some("user".to_string()));
        assert_eq!(nats_config.connection.password, Some("pass".to_string()));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_config_serialization_with_nats() {
        let config = QollectiveConfigBuilder::new()
            .with_nats_urls(vec!["nats://test:4222".to_string()])
            .with_nats_server_enabled(true)
            .build()
            .expect("Should build valid config");

        // Test JSON serialization and deserialization
        let json_str = serde_json::to_string(&config).expect("Should serialize to JSON");
        assert!(!json_str.is_empty());
        assert!(json_str.contains("nats://test:4222"));

        let deserialized_config: QollectiveConfig =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        assert!(deserialized_config.nats.is_some());
        let nats_config = deserialized_config.nats.unwrap();
        assert_eq!(nats_config.connection.urls, vec!["nats://test:4222"]);
        assert!(nats_config.server.enabled);

        // Test pretty JSON serialization
        let pretty_json =
            serde_json::to_string_pretty(&config).expect("Should serialize to pretty JSON");
        assert!(pretty_json.contains("nats://test:4222"));
        assert!(pretty_json.len() > json_str.len()); // Pretty JSON should be longer
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_config_validation_with_nats() {
        // Valid configuration should pass validation
        let valid_config = QollectiveConfig::new();
        assert!(valid_config.validate().is_ok());

        // Invalid NATS configuration should fail validation
        let mut invalid_config = QollectiveConfig::new();
        if let Some(ref mut nats_config) = invalid_config.nats {
            nats_config.connection.urls = vec![]; // Empty URLs should be invalid
        }
        assert!(invalid_config.validate().is_err());
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    #[test]
    fn test_config_without_nats_features() {
        // When NATS features are disabled, config should not have NATS field
        let config = QollectiveConfig::new();

        // This test ensures backwards compatibility
        assert!(config.tenant_extraction_enabled);
        assert!(config.validate().is_ok());

        // Test JSON serialization and deserialization without NATS features
        let json_str = serde_json::to_string(&config).expect("Should serialize to JSON");
        assert!(!json_str.is_empty());

        let deserialized_config: QollectiveConfig =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        // Verify configuration maintains its properties after serialization roundtrip
        assert_eq!(
            config.tenant_extraction_enabled,
            deserialized_config.tenant_extraction_enabled
        );
        assert!(deserialized_config.validate().is_ok());

        // Test that serialized JSON doesn't contain NATS-specific fields when features are disabled
        assert!(!json_str.contains("\"nats\""));

        // Basic functionality should work without NATS features
    }

    #[test]
    fn test_config_builder_from_preset() {
        let config = QollectiveConfigBuilder::from_preset(ConfigPreset::Production)
            .with_tenant_extraction(false)
            .build()
            .expect("Should build from preset");

        assert!(!config.tenant_extraction_enabled);

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            // Should have production NATS settings
            assert!(config.nats.is_some());
            let nats_config = config.nats.unwrap();
            assert!(nats_config.connection.tls.enabled); // Production uses TLS
        }
    }

    #[test]
    fn test_rest_client_builder_tls_system_ca() {
        let config = RestClientConfig::builder().with_tls_system_ca().build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::SystemCa
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_rest_client_builder_tls_custom_ca() {
        let config = RestClientConfig::builder()
            .with_tls_custom_ca("/path/to/ca.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::CustomCa
        );
        assert_eq!(
            config.tls.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/ca.pem"))
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
    }

    #[test]
    fn test_rest_client_builder_tls_skip_verify() {
        let config = RestClientConfig::builder().with_tls_skip_verify().build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::Skip
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_rest_client_builder_mutual_tls() {
        let config = RestClientConfig::builder()
            .with_mutual_tls("/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::MutualTls
        );
        assert_eq!(
            config.tls.cert_path,
            Some(std::path::PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(
            config.tls.key_path,
            Some(std::path::PathBuf::from("/path/to/key.pem"))
        );
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_rest_client_builder_mutual_tls_with_ca() {
        let config = RestClientConfig::builder()
            .with_mutual_tls_with_ca("/path/to/ca.pem", "/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::MutualTls
        );
        assert_eq!(
            config.tls.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/ca.pem"))
        );
        assert_eq!(
            config.tls.cert_path,
            Some(std::path::PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(
            config.tls.key_path,
            Some(std::path::PathBuf::from("/path/to/key.pem"))
        );
    }

    #[test]
    fn test_rest_server_builder_tls_system_ca() {
        let config = RestServerConfig::builder().with_tls_system_ca().build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::SystemCa
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_rest_server_builder_tls_custom_ca() {
        let config = RestServerConfig::builder()
            .with_tls_custom_ca("/path/to/ca.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::CustomCa
        );
        assert_eq!(
            config.tls.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/ca.pem"))
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
    }

    #[test]
    fn test_rest_server_builder_tls_skip_verify() {
        let config = RestServerConfig::builder().with_tls_skip_verify().build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::Skip
        );
        assert!(config.tls.cert_path.is_none());
        assert!(config.tls.key_path.is_none());
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_rest_server_builder_mutual_tls() {
        let config = RestServerConfig::builder()
            .with_mutual_tls("/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::MutualTls
        );
        assert_eq!(
            config.tls.cert_path,
            Some(std::path::PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(
            config.tls.key_path,
            Some(std::path::PathBuf::from("/path/to/key.pem"))
        );
        assert!(config.tls.ca_cert_path.is_none());
    }

    #[test]
    fn test_rest_server_builder_mutual_tls_with_ca() {
        let config = RestServerConfig::builder()
            .with_mutual_tls_with_ca("/path/to/ca.pem", "/path/to/cert.pem", "/path/to/key.pem")
            .build();

        assert!(config.tls.enabled);
        assert_eq!(
            config.tls.verification_mode,
            crate::config::tls::VerificationMode::MutualTls
        );
        assert_eq!(
            config.tls.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/ca.pem"))
        );
        assert_eq!(
            config.tls.cert_path,
            Some(std::path::PathBuf::from("/path/to/cert.pem"))
        );
        assert_eq!(
            config.tls.key_path,
            Some(std::path::PathBuf::from("/path/to/key.pem"))
        );
    }
}

// Default implementations for configuration structures
impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout_ms: 30000,
            max_connections: 100,
            user_agent: "qollective-rust/1.0".to_string(),
            default_headers: std::collections::HashMap::new(),
            retry_attempts: 3,
            tls: TlsConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            tenant_config: TenantClientConfig::default(),
        }
    }
}

impl Default for RestServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 1000,
            request_timeout_ms: 30000,
            cors: CorsConfig::default(),
            tls: TlsConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_origins: vec![],
            allowed_methods: vec![],
            allowed_headers: vec![],
            max_age_seconds: 0,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_requests: true,
            log_responses: false,
            log_headers: false,
            log_body: false,
            log_level: "info".to_string(),
            structured_logging: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_request_duration: true,
            track_response_size: true,
            track_connection_pool: true,
            benchmarking_enabled: false,
            metrics_collection: true,
        }
    }
}

impl RestClientConfig {
    /// Create a builder for REST client configuration
    pub fn builder() -> RestClientConfigBuilder {
        RestClientConfigBuilder::new()
    }
}

impl RestServerConfig {
    /// Create a builder for REST server configuration
    pub fn builder() -> RestServerConfigBuilder {
        RestServerConfigBuilder::new()
    }
}

/// REST client configuration builder
pub struct RestClientConfigBuilder {
    config: RestClientConfig,
}

impl RestClientConfigBuilder {
    /// Create a new REST client configuration builder
    pub fn new() -> Self {
        Self {
            config: RestClientConfig::default(),
        }
    }

    /// Set the base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = Some(base_url.into());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.timeout_ms = timeout_ms;
        self
    }

    /// Set the maximum connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.config.max_connections = max_connections;
        self
    }

    /// Set the user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Set the retry attempts
    pub fn with_retry_attempts(mut self, retry_attempts: u32) -> Self {
        self.config.retry_attempts = retry_attempts;
        self
    }

    /// Enable TLS with system CA verification
    pub fn with_tls_system_ca(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::SystemCa;
        self
    }

    /// Enable TLS with custom CA certificate
    pub fn with_tls_custom_ca(mut self, ca_cert_path: impl Into<std::path::PathBuf>) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::CustomCa;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self
    }

    /// Enable TLS with verification skipped (insecure)
    pub fn with_tls_skip_verify(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::Skip;
        self
    }

    /// Enable mutual TLS with client certificate and key
    pub fn with_mutual_tls(
        mut self,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Enable mutual TLS with custom CA, client certificate and key
    pub fn with_mutual_tls_with_ca(
        mut self,
        ca_cert_path: impl Into<std::path::PathBuf>,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> RestClientConfig {
        self.config
    }
}

/// REST server configuration builder
pub struct RestServerConfigBuilder {
    config: RestServerConfig,
}

impl RestServerConfigBuilder {
    /// Create a new REST server configuration builder
    pub fn new() -> Self {
        Self {
            config: RestServerConfig::default(),
        }
    }

    /// Set the bind address
    pub fn with_bind_address(mut self, address: impl Into<String>) -> Self {
        self.config.bind_address = address.into();
        self
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Set the maximum connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.config.max_connections = max_connections;
        self
    }

    /// Set the request timeout
    pub fn with_request_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.request_timeout_ms = timeout_ms;
        self
    }

    /// Enable TLS with system CA verification
    pub fn with_tls_system_ca(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::SystemCa;
        self
    }

    /// Enable TLS with custom CA certificate
    pub fn with_tls_custom_ca(mut self, ca_cert_path: impl Into<std::path::PathBuf>) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::CustomCa;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self
    }

    /// Enable TLS with verification skipped (insecure)
    pub fn with_tls_skip_verify(mut self) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::Skip;
        self
    }

    /// Enable mutual TLS with client certificate and key
    pub fn with_mutual_tls(
        mut self,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Enable mutual TLS with custom CA, client certificate and key
    pub fn with_mutual_tls_with_ca(
        mut self,
        ca_cert_path: impl Into<std::path::PathBuf>,
        cert_path: impl Into<std::path::PathBuf>,
        key_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.config.tls.enabled = true;
        self.config.tls.verification_mode = crate::config::tls::VerificationMode::MutualTls;
        self.config.tls.ca_cert_path = Some(ca_cert_path.into());
        self.config.tls.cert_path = Some(cert_path.into());
        self.config.tls.key_path = Some(key_path.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> RestServerConfig {
        self.config
    }
}
