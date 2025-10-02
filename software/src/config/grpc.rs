// ABOUTME: gRPC-specific configuration structures and utilities
// ABOUTME: Provides comprehensive gRPC client and server configuration with TLS, health checks, and reflection

//! gRPC-specific configuration structures and utilities.

use super::presets::{LoggingConfig, PerformanceConfig};
use super::tls::TlsConfig;
use crate::client::common::TenantClientConfig;
use crate::constants::{limits, network, timeouts};
use crate::error::{QollectiveError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT configuration for gRPC authentication metadata handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcJwtConfig {
    /// Metadata key name for JWT token (default: "authorization")
    pub header_name: String,
    /// Metadata value prefix (default: "bearer ")
    pub header_prefix: String,
    /// Custom tenant metadata key when no JWT is available (default: "x-tenant-id")
    pub tenant_header_name: String,
    /// Custom onBehalfOf metadata key when no JWT is available (default: "x-on-behalf-of")
    pub on_behalf_of_header_name: String,
}

impl Default for GrpcJwtConfig {
    fn default() -> Self {
        Self {
            header_name: "authorization".to_string(),
            header_prefix: "bearer ".to_string(),
            tenant_header_name: "x-tenant-id".to_string(),
            on_behalf_of_header_name: "x-on-behalf-of".to_string(),
        }
    }
}

/// gRPC client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcClientConfig {
    pub base_url: Option<String>,
    pub timeout_ms: u64,
    pub max_connections: usize,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
    pub retry_attempts: u32,
    pub tls: TlsConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
    pub connection_pool: ConnectionPoolConfig,
    pub health_check: HealthCheckConfig,
    pub jwt_config: GrpcJwtConfig,
    pub tenant_config: TenantClientConfig,
}

/// gRPC server configuration  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
    pub tls: TlsConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
    pub health_check: HealthCheckConfig,
    pub reflection: ReflectionConfig,
    pub concurrency: ConcurrencyConfig,
}

/// Connection pool configuration for gRPC client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    pub enabled: bool,
    pub max_idle_connections: usize,
    pub idle_timeout_ms: u64,
    pub connection_timeout_ms: u64,
    pub keep_alive_time_ms: u64,
    pub keep_alive_timeout_ms: u64,
    pub keep_alive_while_idle: bool,
}

/// Health check configuration for gRPC client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
    pub service_names: Vec<String>,
}

/// gRPC reflection configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionConfig {
    pub enabled: bool,
    pub include_services: Vec<String>,
    pub exclude_services: Vec<String>,
}

/// Concurrency configuration for gRPC server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    pub max_concurrent_streams: u32,
    pub max_frame_size: u32,
    pub initial_window_size: u32,
    pub initial_connection_window_size: u32,
    pub max_header_list_size: u32,
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout_ms: timeouts::DEFAULT_GRPC_TIMEOUT_MS,
            max_connections: limits::DEFAULT_GRPC_CLIENT_MAX_CONNECTIONS,
            user_agent: "qollective-grpc-client/1.0".to_string(),
            default_headers: HashMap::new(),
            retry_attempts: 3,
            tls: TlsConfig::default(),
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
            connection_pool: ConnectionPoolConfig::default(),
            health_check: HealthCheckConfig::default(),
            jwt_config: GrpcJwtConfig::default(),
            tenant_config: TenantClientConfig::default(),
        }
    }
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address: network::DEFAULT_BIND_LOCALHOST.to_string(),
            port: network::DEFAULT_GRPC_SERVER_PORT,
            max_connections: limits::DEFAULT_GRPC_SERVER_MAX_CONNECTIONS,
            request_timeout_ms: timeouts::DEFAULT_GRPC_TIMEOUT_MS,
            tls: TlsConfig::default(),
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
            health_check: HealthCheckConfig::default(),
            reflection: ReflectionConfig::default(),
            concurrency: ConcurrencyConfig::default(),
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_idle_connections: limits::DEFAULT_GRPC_MAX_IDLE_CONNECTIONS,
            idle_timeout_ms: timeouts::DEFAULT_GRPC_IDLE_TIMEOUT_MS,
            connection_timeout_ms: 10000,
            keep_alive_time_ms: timeouts::DEFAULT_GRPC_KEEP_ALIVE_TIME_MS,
            keep_alive_timeout_ms: 5000,
            keep_alive_while_idle: true,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_ms: 30000,
            timeout_ms: 5000,
            healthy_threshold: 2,
            unhealthy_threshold: 3,
            service_names: vec![],
        }
    }
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            include_services: vec![],
            exclude_services: vec![],
        }
    }
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: limits::DEFAULT_GRPC_MAX_CONCURRENT_STREAMS,
            max_frame_size: 16384,
            initial_window_size: 65535,
            initial_connection_window_size: 65535,
            max_header_list_size: 16384,
        }
    }
}

/// Validation utilities for gRPC configurations
impl GrpcClientConfig {
    pub fn validate(&self) -> Result<()> {
        if self.timeout_ms == 0 {
            return Err(QollectiveError::config(
                "Client timeout must be greater than 0",
            ));
        }

        if self.max_connections == 0 {
            return Err(QollectiveError::config(
                "Max connections must be greater than 0",
            ));
        }

        if self.retry_attempts > 10 {
            return Err(QollectiveError::config(
                "Retry attempts should not exceed 10",
            ));
        }

        // Validate TLS configuration
        if self.tls.enabled {
            if let Some(ref cert_path) = self.tls.cert_path {
                if cert_path.as_os_str().is_empty() {
                    return Err(QollectiveError::config(
                        "TLS cert path cannot be empty when TLS is enabled",
                    ));
                }
            }
        }

        Ok(())
    }
}

impl GrpcServerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(QollectiveError::config(
                "Server port must be greater than 0",
            ));
        }

        if self.max_connections == 0 {
            return Err(QollectiveError::config(
                "Max connections must be greater than 0",
            ));
        }

        if self.request_timeout_ms == 0 {
            return Err(QollectiveError::config(
                "Request timeout must be greater than 0",
            ));
        }

        if self.bind_address.is_empty() {
            return Err(QollectiveError::config("Bind address cannot be empty"));
        }

        // Validate concurrency settings
        if self.concurrency.max_concurrent_streams == 0 {
            return Err(QollectiveError::config(
                "Max concurrent streams must be greater than 0",
            ));
        }

        // Validate TLS configuration
        if self.tls.enabled {
            if let Some(ref cert_path) = self.tls.cert_path {
                if cert_path.as_os_str().is_empty() {
                    return Err(QollectiveError::config(
                        "TLS cert path cannot be empty when TLS is enabled",
                    ));
                }
            }
            if let Some(ref key_path) = self.tls.key_path {
                if key_path.as_os_str().is_empty() {
                    return Err(QollectiveError::config(
                        "TLS key path cannot be empty when TLS is enabled",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Create a builder for gRPC server configuration
    pub fn builder() -> GrpcServerConfigBuilder {
        GrpcServerConfigBuilder::new()
    }
}

impl GrpcClientConfig {
    /// Create a builder for gRPC client configuration
    pub fn builder() -> GrpcClientConfigBuilder {
        GrpcClientConfigBuilder::new()
    }
}

/// gRPC client configuration builder
pub struct GrpcClientConfigBuilder {
    config: GrpcClientConfig,
}

impl GrpcClientConfigBuilder {
    /// Create a new gRPC client configuration builder
    pub fn new() -> Self {
        Self {
            config: GrpcClientConfig::default(),
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
    pub fn build(self) -> GrpcClientConfig {
        self.config
    }
}

/// gRPC server configuration builder
pub struct GrpcServerConfigBuilder {
    config: GrpcServerConfig,
}

impl GrpcServerConfigBuilder {
    /// Create a new gRPC server configuration builder
    pub fn new() -> Self {
        Self {
            config: GrpcServerConfig::default(),
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
    pub fn build(self) -> GrpcServerConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_client_config_default() {
        let config = GrpcClientConfig::default();

        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.user_agent, "qollective-grpc-client/1.0");
        assert_eq!(config.retry_attempts, 3);
        assert!(!config.tls.enabled);
        assert!(config.logging.enabled);
        assert!(config.performance.enabled);
        assert!(config.connection_pool.enabled);
        assert!(!config.health_check.enabled);
    }

    #[test]
    fn test_grpc_server_config_default() {
        let config = GrpcServerConfig::default();

        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 50051);
        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.request_timeout_ms, 30000);
        assert!(!config.tls.enabled);
        assert!(config.logging.enabled);
        assert!(config.performance.enabled);
        assert!(!config.health_check.enabled);
        assert!(!config.reflection.enabled);
    }

    #[test]
    fn test_grpc_client_config_validation() {
        let mut config = GrpcClientConfig::default();
        assert!(config.validate().is_ok());

        config.timeout_ms = 0;
        assert!(config.validate().is_err());

        config.timeout_ms = 30000;
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 100;
        config.retry_attempts = 15;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_grpc_server_config_validation() {
        let mut config = GrpcServerConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 50051;
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 1000;
        config.bind_address = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_pool_config() {
        let config = ConnectionPoolConfig::default();

        assert!(config.enabled);
        assert_eq!(config.max_idle_connections, 10);
        assert_eq!(config.idle_timeout_ms, 90000);
        assert_eq!(config.keep_alive_time_ms, 60000);
        assert!(config.keep_alive_while_idle);
    }

    #[test]
    fn test_health_check_config() {
        let config = HealthCheckConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.interval_ms, 30000);
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.healthy_threshold, 2);
        assert_eq!(config.unhealthy_threshold, 3);
        assert!(config.service_names.is_empty());
    }

    #[test]
    fn test_reflection_config() {
        let config = ReflectionConfig::default();

        assert!(!config.enabled);
        assert!(config.include_services.is_empty());
        assert!(config.exclude_services.is_empty());
    }

    #[test]
    fn test_concurrency_config() {
        let config = ConcurrencyConfig::default();

        assert_eq!(config.max_concurrent_streams, 100);
        assert_eq!(config.max_frame_size, 16384);
        assert_eq!(config.initial_window_size, 65535);
        assert_eq!(config.initial_connection_window_size, 65535);
        assert_eq!(config.max_header_list_size, 16384);
    }

    #[test]
    fn test_grpc_client_builder_tls_system_ca() {
        let config = GrpcClientConfig::builder().with_tls_system_ca().build();

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
    fn test_grpc_client_builder_tls_custom_ca() {
        let config = GrpcClientConfig::builder()
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
    fn test_grpc_client_builder_tls_skip_verify() {
        let config = GrpcClientConfig::builder().with_tls_skip_verify().build();

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
    fn test_grpc_client_builder_mutual_tls() {
        let config = GrpcClientConfig::builder()
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
    fn test_grpc_client_builder_mutual_tls_with_ca() {
        let config = GrpcClientConfig::builder()
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
    fn test_grpc_server_builder_tls_system_ca() {
        let config = GrpcServerConfig::builder().with_tls_system_ca().build();

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
    fn test_grpc_server_builder_tls_custom_ca() {
        let config = GrpcServerConfig::builder()
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
    fn test_grpc_server_builder_tls_skip_verify() {
        let config = GrpcServerConfig::builder().with_tls_skip_verify().build();

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
    fn test_grpc_server_builder_mutual_tls() {
        let config = GrpcServerConfig::builder()
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
    fn test_grpc_server_builder_mutual_tls_with_ca() {
        let config = GrpcServerConfig::builder()
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
