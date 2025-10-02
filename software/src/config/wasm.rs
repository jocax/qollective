// ABOUTME: WASM client configuration structure
// ABOUTME: Provides configuration for browser-based envelope communication

//! WASM client configuration structures.
//!
//! This module provides configuration structures for WASM clients that
//! support multiple protocols while maintaining configuration inheritance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "rest-client")]
use crate::config::rest::RestClientConfig;

#[cfg(feature = "websocket-client")]
use crate::config::websocket::WebSocketClientConfig;

/// Main WASM client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmClientConfig {
    /// Enable REST client
    pub rest_enabled: bool,

    /// REST client configuration
    #[cfg(feature = "rest-client")]
    pub rest_config: RestClientConfig,

    /// Enable WebSocket client
    pub websocket_enabled: bool,

    /// WebSocket client configuration
    #[cfg(feature = "websocket-client")]
    pub websocket_config: WebSocketClientConfig,

    /// Enable MCP adapter
    pub mcp_enabled: bool,

    /// MCP adapter configuration
    pub mcp_config: McpAdapterConfig,

    /// Certificate management configuration
    pub certificate_config: CertificateConfig,

    /// Global timeout settings
    pub timeouts: WasmTimeoutConfig,

    /// Bundle optimization settings
    pub bundle_config: BundleConfig,

    /// Browser-specific settings
    pub browser_config: BrowserConfig,
}

/// MCP adapter configuration for WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAdapterConfig {
    /// Default MCP server URLs
    pub default_servers: Vec<String>,

    /// Context injection settings
    pub context_injection: ContextInjectionConfig,

    /// Tool execution timeouts
    pub tool_timeout_ms: u64,

    /// Maximum concurrent tool calls
    pub max_concurrent_calls: usize,

    /// Error handling policy
    pub error_policy: McpErrorPolicy,

    /// Cache tool definitions
    pub cache_tools: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

/// Certificate management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    /// Embedded certificates for mTLS
    pub embedded_certificates: HashMap<String, EmbeddedCertificate>,

    /// Certificate validation settings
    pub validation: CertificateValidation,

    /// Auto-refresh certificates
    pub auto_refresh: bool,

    /// Refresh threshold in seconds
    pub refresh_threshold_secs: u64,
}

/// Embedded certificate for WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedCertificate {
    /// Certificate PEM data (base64 encoded for WASM)
    pub cert_data: String,

    /// Private key PEM data (base64 encoded for WASM)
    pub key_data: String,

    /// CA certificate PEM data (optional)
    pub ca_data: Option<String>,

    /// Certificate name/identifier
    pub name: String,

    /// Valid domains for this certificate
    pub domains: Vec<String>,
}

/// Certificate validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidation {
    /// Verify certificate chain
    pub verify_chain: bool,

    /// Verify certificate hostname
    pub verify_hostname: bool,

    /// Allow self-signed certificates (development only)
    pub allow_self_signed: bool,

    /// Custom CA certificates
    pub custom_ca_certs: Vec<String>,
}

/// Context injection configuration for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInjectionConfig {
    /// Inject tenant information
    pub inject_tenant: bool,

    /// Inject user information
    pub inject_user: bool,

    /// Inject session information
    pub inject_session: bool,

    /// Inject trace information
    pub inject_trace: bool,

    /// Custom context fields to inject
    pub custom_fields: HashMap<String, String>,

    /// Security context injection
    pub security_context: SecurityContextConfig,
}

/// Security context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContextConfig {
    /// Include permissions in context
    pub include_permissions: bool,

    /// Include scopes in context
    pub include_scopes: bool,

    /// Include role information
    pub include_roles: bool,

    /// Sanitize sensitive information
    pub sanitize_sensitive: bool,
}

/// MCP error handling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpErrorPolicy {
    /// Fail fast on any error
    FailFast,

    /// Retry with exponential backoff
    RetryExponential {
        max_retries: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
    },

    /// Retry with linear backoff
    RetryLinear { max_retries: u32, delay_ms: u64 },

    /// Continue on error (best effort)
    BestEffort,
}

/// WASM-specific timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTimeoutConfig {
    /// Default request timeout in milliseconds
    pub default_request_timeout_ms: u64,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// WebSocket ping interval in milliseconds
    pub websocket_ping_interval_ms: u64,

    /// Certificate validation timeout in milliseconds
    pub cert_validation_timeout_ms: u64,

    /// Tool execution timeout in milliseconds
    pub tool_execution_timeout_ms: u64,
}

/// Bundle optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfig {
    /// Target bundle size limit in bytes
    pub size_limit_bytes: usize,

    /// Optimize for size vs. speed
    pub optimize_for_size: bool,

    /// Enable tree shaking
    pub tree_shaking: bool,

    /// Compression settings
    pub compression: CompressionConfig,
}

/// Compression configuration for WASM bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable gzip compression
    pub gzip: bool,

    /// Enable brotli compression
    pub brotli: bool,

    /// Compression level (1-9)
    pub level: u8,
}

/// Browser-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Console logging configuration
    pub console_logging: ConsoleLoggingConfig,

    /// Local storage settings
    pub local_storage: LocalStorageConfig,

    /// Service worker settings
    pub service_worker: ServiceWorkerConfig,

    /// Browser compatibility settings
    pub compatibility: CompatibilityConfig,
}

/// Console logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLoggingConfig {
    /// Enable console logging
    pub enabled: bool,

    /// Log level (error, warn, info, debug, trace)
    pub level: String,

    /// Include stack traces
    pub include_stack_traces: bool,

    /// Log to browser console
    pub log_to_console: bool,

    /// Buffer logs for inspection
    pub buffer_logs: bool,

    /// Maximum log buffer size
    pub max_buffer_size: usize,
}

/// Local storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    /// Use local storage for caching
    pub enabled: bool,

    /// Storage key prefix
    pub key_prefix: String,

    /// Cache certificates locally
    pub cache_certificates: bool,

    /// Cache tool definitions
    pub cache_tools: bool,

    /// Cache timeout in seconds
    pub cache_timeout_secs: u64,

    /// Maximum storage size in bytes
    pub max_storage_bytes: usize,
}

/// Service worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerConfig {
    /// Enable service worker integration
    pub enabled: bool,

    /// Service worker script URL
    pub script_url: Option<String>,

    /// Cache API responses
    pub cache_responses: bool,

    /// Offline mode support
    pub offline_mode: bool,

    /// Background sync support
    pub background_sync: bool,
}

/// Browser compatibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityConfig {
    /// Minimum supported browser versions
    pub min_versions: HashMap<String, String>,

    /// Polyfill configuration
    pub polyfills: PolyfillConfig,

    /// Feature detection settings
    pub feature_detection: FeatureDetectionConfig,
}

/// Polyfill configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyfillConfig {
    /// Enable fetch polyfill
    pub fetch: bool,

    /// Enable WebSocket polyfill
    pub websocket: bool,

    /// Enable Promise polyfill
    pub promise: bool,

    /// Enable AbortController polyfill
    pub abort_controller: bool,
}

/// Feature detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDetectionConfig {
    /// Detect WebAssembly support
    pub webassembly: bool,

    /// Detect BigInt support
    pub bigint: bool,

    /// Detect dynamic import support
    pub dynamic_import: bool,

    /// Detect Worker support
    pub worker: bool,
}

impl Default for WasmClientConfig {
    fn default() -> Self {
        use crate::constants::{limits, timeouts};

        Self {
            rest_enabled: true,

            #[cfg(feature = "rest-client")]
            rest_config: RestClientConfig::default(),

            websocket_enabled: true,

            #[cfg(feature = "websocket-client")]
            websocket_config: WebSocketClientConfig::default(),

            mcp_enabled: true,
            mcp_config: McpAdapterConfig::default(),
            certificate_config: CertificateConfig::default(),
            timeouts: WasmTimeoutConfig::default(),
            bundle_config: BundleConfig::default(),
            browser_config: BrowserConfig::default(),
        }
    }
}

impl Default for McpAdapterConfig {
    fn default() -> Self {
        use crate::constants::{limits, timeouts};

        Self {
            default_servers: Vec::new(),
            context_injection: ContextInjectionConfig::default(),

            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            tool_timeout_ms: timeouts::DEFAULT_MCP_TIMEOUT.as_millis() as u64,
            #[cfg(not(any(feature = "mcp-client", feature = "mcp-server")))]
            tool_timeout_ms: 60000,

            max_concurrent_calls: 10,
            error_policy: McpErrorPolicy::RetryExponential {
                max_retries: 3,
                base_delay_ms: 1000,
                max_delay_ms: 30000,
            },
            cache_tools: true,

            #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
            cache_ttl_secs: timeouts::DEFAULT_MCP_CACHE_TTL_SECS,
            #[cfg(not(any(feature = "mcp-client", feature = "mcp-server")))]
            cache_ttl_secs: 300,
        }
    }
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            embedded_certificates: HashMap::new(),
            validation: CertificateValidation::default(),
            auto_refresh: true,
            refresh_threshold_secs: 300, // 5 minutes
        }
    }
}

impl Default for CertificateValidation {
    fn default() -> Self {
        Self {
            verify_chain: true,
            verify_hostname: true,
            allow_self_signed: false,
            custom_ca_certs: Vec::new(),
        }
    }
}

impl Default for ContextInjectionConfig {
    fn default() -> Self {
        Self {
            inject_tenant: true,
            inject_user: true,
            inject_session: true,
            inject_trace: true,
            custom_fields: HashMap::new(),
            security_context: SecurityContextConfig::default(),
        }
    }
}

impl Default for SecurityContextConfig {
    fn default() -> Self {
        Self {
            include_permissions: true,
            include_scopes: true,
            include_roles: true,
            sanitize_sensitive: true,
        }
    }
}

impl Default for WasmTimeoutConfig {
    fn default() -> Self {
        use crate::constants::timeouts;

        Self {
            #[cfg(feature = "rest-client")]
            default_request_timeout_ms: timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS,
            #[cfg(not(feature = "rest-client"))]
            default_request_timeout_ms: 30000,

            connection_timeout_ms: 10000,

            #[cfg(feature = "websocket-client")]
            websocket_ping_interval_ms: timeouts::DEFAULT_WEBSOCKET_PING_INTERVAL_MS,
            #[cfg(not(feature = "websocket-client"))]
            websocket_ping_interval_ms: 30000,

            cert_validation_timeout_ms: 5000,
            tool_execution_timeout_ms: 60000,
        }
    }
}

impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            size_limit_bytes: 500_000, // 500KB as per PRP requirements
            optimize_for_size: true,
            tree_shaking: true,
            compression: CompressionConfig::default(),
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            gzip: true,
            brotli: true,
            level: 6,
        }
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            console_logging: ConsoleLoggingConfig::default(),
            local_storage: LocalStorageConfig::default(),
            service_worker: ServiceWorkerConfig::default(),
            compatibility: CompatibilityConfig::default(),
        }
    }
}

impl Default for ConsoleLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            include_stack_traces: false,
            log_to_console: true,
            buffer_logs: true,
            max_buffer_size: 1000,
        }
    }
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_prefix: "qollective_wasm_".to_string(),
            cache_certificates: true,
            cache_tools: true,
            cache_timeout_secs: 3600,      // 1 hour
            max_storage_bytes: 10_485_760, // 10MB
        }
    }
}

impl Default for ServiceWorkerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            script_url: None,
            cache_responses: true,
            offline_mode: false,
            background_sync: false,
        }
    }
}

impl Default for CompatibilityConfig {
    fn default() -> Self {
        let mut min_versions = HashMap::new();
        min_versions.insert("chrome".to_string(), "88".to_string());
        min_versions.insert("firefox".to_string(), "85".to_string());
        min_versions.insert("safari".to_string(), "14".to_string());
        min_versions.insert("edge".to_string(), "88".to_string());

        Self {
            min_versions,
            polyfills: PolyfillConfig::default(),
            feature_detection: FeatureDetectionConfig::default(),
        }
    }
}

impl Default for PolyfillConfig {
    fn default() -> Self {
        Self {
            fetch: true,
            websocket: false,
            promise: true,
            abort_controller: true,
        }
    }
}

impl Default for FeatureDetectionConfig {
    fn default() -> Self {
        Self {
            webassembly: true,
            bigint: true,
            dynamic_import: true,
            worker: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WasmClientConfig::default();
        assert!(config.rest_enabled);
        assert!(config.websocket_enabled);
        assert!(config.mcp_enabled);
        assert_eq!(config.bundle_config.size_limit_bytes, 500_000);
    }

    #[test]
    fn test_mcp_error_policy_serialization() {
        let policy = McpErrorPolicy::RetryExponential {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
        };

        let serialized = serde_json::to_string(&policy).unwrap();
        let deserialized: McpErrorPolicy = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            McpErrorPolicy::RetryExponential { max_retries, .. } => {
                assert_eq!(max_retries, 3);
            }
            _ => panic!("Unexpected error policy type"),
        }
    }

    #[test]
    fn test_certificate_config() {
        let mut config = CertificateConfig::default();

        let cert = EmbeddedCertificate {
            cert_data: "cert_pem_data".to_string(),
            key_data: "key_pem_data".to_string(),
            ca_data: None,
            name: "test_cert".to_string(),
            domains: vec!["example.com".to_string()],
        };

        config
            .embedded_certificates
            .insert("test".to_string(), cert);

        assert!(config.embedded_certificates.contains_key("test"));
        assert!(config.validation.verify_chain);
        assert!(!config.validation.allow_self_signed);
    }
}
