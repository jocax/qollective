// ABOUTME: Feature-gated constants and default values for the Qollective framework
// ABOUTME: Centralizes configuration defaults and provides compile-time feature control

//! Constants and default values for the Qollective framework.
//!
//! This module centralizes all hard-coded values and provides feature-gated
//! constants that can be overridden at compile time or runtime.

use std::time::Duration;

/// Default timeout values
pub mod timeouts {
    use super::*;

    /// Default timeout for agent communication
    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const DEFAULT_AGENT_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default timeout for MCP tool execution
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default timeout for transport capability detection
    pub const DEFAULT_TRANSPORT_DETECTION_TIMEOUT: Duration = Duration::from_secs(5);

    /// Default TTL for capability cache
    pub const DEFAULT_CAPABILITY_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

    /// Default circuit breaker recovery timeout
    pub const DEFAULT_CIRCUIT_BREAKER_RECOVERY: Duration = Duration::from_secs(60);

    /// Default REST request timeout in milliseconds
    #[cfg(any(feature = "rest-server", feature = "rest-client", feature = "wasm-client"))]
    pub const DEFAULT_REST_REQUEST_TIMEOUT_MS: u64 = 30000;

    /// Default gRPC timeout in milliseconds
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_TIMEOUT_MS: u64 = 30000;

    /// Default NATS connection timeout in milliseconds
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_CONNECTION_TIMEOUT_MS: u64 = 5000;

    /// Default NATS reconnect timeout in milliseconds
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_RECONNECT_TIMEOUT_MS: u64 = 2000;

    /// Default NATS request timeout in milliseconds
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_REQUEST_TIMEOUT_MS: u64 = 30000;

    /// Default WebSocket connection timeout in milliseconds
    #[cfg(feature = "websocket-client")]
    pub const DEFAULT_WEBSOCKET_CONNECTION_TIMEOUT_MS: u64 = 30000;

    /// Default WebSocket message timeout in milliseconds
    #[cfg(feature = "websocket-client")]
    pub const DEFAULT_WEBSOCKET_MESSAGE_TIMEOUT_MS: u64 = 10000;

    /// Default WebSocket ping interval in milliseconds
    #[cfg(feature = "websocket-client")]
    pub const DEFAULT_WEBSOCKET_PING_INTERVAL_MS: u64 = 30000;

    /// Default TCP keepalive duration in seconds
    #[cfg(feature = "rest-server")]
    pub const DEFAULT_TCP_KEEPALIVE_SECS: u64 = 75;

    /// Default graceful shutdown timeout in seconds
    pub const DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

    /// Default agent TTL in seconds
    pub const DEFAULT_AGENT_TTL_SECS: u64 = 300;

    /// Default agent cleanup interval in seconds
    pub const DEFAULT_AGENT_CLEANUP_INTERVAL_SECS: u64 = 60;

    /// Default JWT refresh threshold in seconds
    #[cfg(feature = "security")]
    pub const DEFAULT_JWT_REFRESH_THRESHOLD_SECS: u64 = 300;

    /// Default security TTL in seconds
    #[cfg(feature = "security")]
    pub const DEFAULT_SECURITY_TTL_SECS: u64 = 3600;

    /// Default MCP cache TTL in seconds
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_CACHE_TTL_SECS: u64 = 300;

    /// Default MCP discovery timeout in milliseconds
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_DISCOVERY_TIMEOUT_MS: u64 = 10000;

    /// Default MCP connection timeout in milliseconds
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_CONNECTION_TIMEOUT_MS: u64 = 5000;

    /// Default NATS announcement interval in milliseconds
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_ANNOUNCEMENT_INTERVAL_MS: u64 = 30000;

    /// Default NATS TTL in milliseconds
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_TTL_MS: u64 = 90000;

    /// Default gRPC idle timeout in milliseconds
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_IDLE_TIMEOUT_MS: u64 = 90000;

    /// Default gRPC keep-alive time in milliseconds
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_KEEP_ALIVE_TIME_MS: u64 = 60000;

    /// Default NATS retry delay in milliseconds
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_RETRY_DELAY_MS: u64 = 1000;

    /// Default REST retry delay in milliseconds
    #[cfg(any(feature = "rest-client", feature = "wasm-client"))]
    pub const DEFAULT_REST_RETRY_DELAY_MS: u64 = 1000;

    /// Default REST maximum retry delay in milliseconds
    #[cfg(any(feature = "rest-client", feature = "wasm-client"))]
    pub const DEFAULT_REST_MAX_RETRY_DELAY_MS: u64 = 30000;
}

/// Default endpoint patterns and URLs
pub mod endpoints {
    /// Default endpoint pattern for Qollective agents
    pub const DEFAULT_AGENT_ENDPOINT_PATTERN: &str = "https://{agent_name}.qollective.local";

    /// Default endpoint pattern for external agents
    pub const DEFAULT_EXTERNAL_AGENT_PATTERN: &str = "https://external-{agent_id}.example.com";

    /// Default MCP server endpoint pattern
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_SERVER_PATTERN: &str = "https://mcp-{server_id}.qollective.local";

    /// Default local domain for Qollective services
    pub const DEFAULT_QOLLECTIVE_DOMAIN: &str = "qollective.local";

}

/// NATS subject patterns with clean, consistent prefixing
pub mod subjects {
    // A2A (Agent-to-Agent) subjects with consistent prefix
    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_REGISTRATION: &str = "qollective.a2a.v1.register";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_DEREGISTRATION: &str = "qollective.a2a.v1.deregister";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_HEARTBEAT: &str = "qollective.a2a.v1.heartbeat";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_DISCOVERY: &str = "qollective.a2a.v1.discover";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_CAPABILITIES: &str = "qollective.a2a.v1.capabilities";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_HEALTH: &str = "qollective.a2a.v1.health";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_REGISTRY_ANNOUNCE: &str = "qollective.a2a.v1.registry.announce";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_REGISTRY_EVENTS: &str = "qollective.a2a.v1.registry.events";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_REGISTRY_REGISTER: &str = "qollective.a2a.v1.registry.register";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_HEALTH_UPDATE: &str = "qollective.a2a.v1.health.update";

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub const AGENT_DIRECT_PATTERN: &str = "qollective.a2a.v1.agent.{agent_id}.direct";

    // MCP (Model Context Protocol) subjects with consistent prefix
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_TOOL_DISCOVER: &str = "qollective.mcp.v1.tool.discover";

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_TOOL_EXECUTE: &str = "qollective.mcp.v1.tool.execute";

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_TOOL_CHAIN: &str = "qollective.mcp.v1.tool.chain";

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_SERVER_ANNOUNCE: &str = "qollective.mcp.v1.server.announce";

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_SERVER_DISCOVERY: &str = "qollective.mcp.v1.server.discover";

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_CAPABILITIES: &str = "qollective.mcp.v1.capabilities";

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const MCP_HEALTH: &str = "qollective.mcp.v1.health";

    // Special subjects for Enterprise examples
    pub const ENTERPRISE_BRIDGE_CHALLENGE: &str = "enterprise.bridge.challenge";
}

/// Circuit breaker configuration defaults
pub mod circuit_breaker {
    /// Default failure threshold to open circuit
    pub const DEFAULT_FAILURE_THRESHOLD: u32 = 5;

    /// Default maximum retry attempts
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Default circuit breaker enabled state
    pub const DEFAULT_ENABLED: bool = true;
}

/// Transport configuration defaults
pub mod transport {
    /// Default transport detection enabled state
    pub const DEFAULT_AUTO_DETECTION: bool = true;

    /// Default maximum detection retries
    pub const DEFAULT_MAX_DETECTION_RETRIES: u32 = 3;

    /// Default retry failed detections
    pub const DEFAULT_RETRY_FAILED_DETECTIONS: bool = true;

    /// Default performance score for unknown endpoints
    pub const DEFAULT_PERFORMANCE_SCORE: u32 = 50;

    /// Minimum performance score for high-performance requirements
    pub const HIGH_PERFORMANCE_THRESHOLD: u32 = 80;
}


/// Configuration validation constants
pub mod validation {
    /// Maximum agent name length
    pub const MAX_AGENT_NAME_LENGTH: usize = 255;

    /// Maximum capability name length  
    pub const MAX_CAPABILITY_NAME_LENGTH: usize = 128;

    /// Maximum number of capabilities per agent
    pub const MAX_CAPABILITIES_PER_AGENT: usize = 100;

    /// Maximum metadata key length
    pub const MAX_METADATA_KEY_LENGTH: usize = 64;

    /// Maximum metadata value length
    pub const MAX_METADATA_VALUE_LENGTH: usize = 1024;
}

/// Network and connection limits
pub mod limits {
    /// Default maximum request size for REST endpoints (1MB)
    #[cfg(feature = "rest-server")]
    pub const DEFAULT_REST_MAX_REQUEST_SIZE: usize = 1024 * 1024;

    /// Default maximum connections for gRPC server
    #[cfg(any(feature = "grpc-server", feature = "rest-server"))]
    pub const DEFAULT_GRPC_SERVER_MAX_CONNECTIONS: usize = 1000;

    /// Default maximum connections for gRPC client
    #[cfg(feature = "grpc-client")]
    pub const DEFAULT_GRPC_CLIENT_MAX_CONNECTIONS: usize = 100;

    /// Default maximum idle connections for gRPC
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_MAX_IDLE_CONNECTIONS: usize = 10;

    /// Default maximum concurrent streams for gRPC
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_MAX_CONCURRENT_STREAMS: u32 = 100;

    /// Default maximum frame size for gRPC
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_MAX_FRAME_SIZE: u32 = 16384;

    /// Default maximum pending messages for NATS
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_MAX_PENDING_MESSAGES: usize = 512;

    /// Default maximum concurrent handlers for NATS
    #[cfg(feature = "nats-server")]
    pub const DEFAULT_NATS_MAX_CONCURRENT_HANDLERS: usize = 50;

    /// Default maximum WebSocket message size (16MB)
    #[cfg(feature = "websocket-client")]
    pub const DEFAULT_WEBSOCKET_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

    /// Default maximum WebSocket connections
    #[cfg(feature = "websocket-client")]
    pub const DEFAULT_MAX_WEBSOCKET_CONNECTIONS: usize = 100;

    /// Default maximum number of agents in registry
    pub const DEFAULT_MAX_AGENTS: usize = 10000;

    /// Default maximum audit events in memory
    #[cfg(feature = "security")]
    pub const DEFAULT_AUDIT_MAX_EVENTS: usize = 1000;

    /// Default maximum reconnect attempts for NATS
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_MAX_RECONNECT_ATTEMPTS: u32 = 5;

    /// Default retry attempts for gRPC
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_GRPC_RETRY_ATTEMPTS: u32 = 3;

    /// Maximum agent registrations per time window (per agent ID)
    pub const DEFAULT_MAX_AGENT_REGISTRATIONS_PER_AGENT: u32 = 10;

    /// Agent rate limiting time window duration in seconds
    pub const DEFAULT_AGENT_RATE_LIMIT_WINDOW_SECS: u64 = 300; // 5 minutes

    /// Maximum total agent registrations per time window (global limit)
    pub const DEFAULT_MAX_AGENT_GLOBAL_REGISTRATIONS: u32 = 1000;

    /// Default maximum MCP (rmcp) clients to cache
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_MAX_CACHED_CLIENTS: usize = 100;

    /// Default maximum connections per MCP client
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_MAX_CONNECTIONS_PER_CLIENT: u32 = 10;

    /// Default MCP retry attempts for failed requests
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_RETRY_ATTEMPTS: u32 = 3;

    /// Default retry attempts for REST clients and general purpose retries
    #[cfg(any(feature = "rest-client", feature = "wasm-client"))]
    pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;
}

/// Network addresses and ports
pub mod network {
    /// Default NATS server URL
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";

    /// Default gRPC server port
    #[cfg(feature = "grpc-server")]
    pub const DEFAULT_GRPC_SERVER_PORT: u16 = 50051;

    /// Default REST server port
    #[cfg(any(feature = "rest-server", feature = "rest-client"))]
    pub const DEFAULT_REST_SERVER_PORT: u16 = 8080;

    /// Default NATS port
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_PORT: u16 = 4222;

    /// Default NATS TLS port
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_TLS_PORT: u16 = 4443;

    /// Default protobuf protocol version
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    pub const DEFAULT_PROTOBUF_VERSION: &str = "1.0.0";

    /// Default NATS TLS URL
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const DEFAULT_NATS_TLS_URL: &str = "nats://localhost:4443";

    /// Default bind address for all interfaces
    pub const DEFAULT_BIND_ALL_INTERFACES: &str = "0.0.0.0";

    /// Default bind address for localhost only
    pub const DEFAULT_BIND_LOCALHOST: &str = "127.0.0.1";

    /// Default Redis connection string
    #[cfg(feature = "security")]
    pub const DEFAULT_REDIS_CONNECTION_STRING: &str = "redis://localhost:6379";

    /// Default localhost hostname
    pub const DEFAULT_LOCALHOST: &str = "localhost";

    /// Standard NATS client names for component identification
    pub mod client_names {
        /// A2A server component
        pub const A2A_SERVER: &str = "qollective-a2a-server";

        /// A2A client component
        pub const A2A_CLIENT: &str = "qollective-a2a-client";

        /// NATS client component
        pub const NATS_CLIENT: &str = "qollective-nats-client";

        /// MCP server component
        pub const MCP_SERVER: &str = "qollective-mcp-server";

        /// MCP client component
        pub const MCP_CLIENT: &str = "qollective-mcp-client";

        /// REST server component
        pub const REST_SERVER: &str = "qollective-rest-server";

        /// REST client component
        pub const REST_CLIENT: &str = "qollective-rest-client";

        /// gRPC server component
        pub const GRPC_SERVER: &str = "qollective-grpc-server";

        /// gRPC client component
        pub const GRPC_CLIENT: &str = "qollective-grpc-client";
    }

    /// Smart TLS certificate path resolution
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub mod tls_paths {
        use super::super::env_vars;
        use std::env;
        use std::path::PathBuf;

        /// Fallback TLS certificate base path relative to Cargo.toml
        const FALLBACK_TLS_CERT_BASE_PATH: &str = "tests/certs";

        /// Legacy absolute path for backwards compatibility
        const LEGACY_TLS_CERT_BASE_PATH: &str = "/Users/ms/development/docker/nats/certs/server";

        /// Smart resolution of TLS certificate base path
        /// 1. Check environment variable QOLLECTIVE_TLS_CERT_BASE_PATH
        /// 2. Check for software/runtime/rust/tests/certs relative to Cargo.toml
        /// 3. Fall back to legacy absolute path
        pub fn resolve_tls_cert_base_path() -> String {
            // 1. Check environment variable first
            if let Ok(env_path) = env::var(env_vars::QOLLECTIVE_TLS_CERT_BASE_PATH) {
                return env_path;
            }

            // 2. Check for relative path from Cargo.toml location
            let cargo_manifest_dir =
                env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

            let relative_cert_path =
                PathBuf::from(&cargo_manifest_dir).join(FALLBACK_TLS_CERT_BASE_PATH);

            if relative_cert_path.exists() {
                return relative_cert_path.to_string_lossy().to_string();
            }

            // 3. Fall back to legacy absolute path
            LEGACY_TLS_CERT_BASE_PATH.to_string()
        }

        /// Helper functions to build TLS certificate paths from base path
        pub fn ca_file_path(base_path: &str) -> String {
            format!("{}/ca.pem", base_path)
        }

        pub fn cert_file_path(base_path: &str) -> String {
            format!("{}/client-cert.pem", base_path)
        }

        pub fn key_file_path(base_path: &str) -> String {
            format!("{}/client-key.pem", base_path)
        }

        /// Smart TLS file paths (using smart base path resolution)
        pub fn default_ca_file() -> String {
            ca_file_path(&resolve_tls_cert_base_path())
        }

        pub fn default_cert_file() -> String {
            cert_file_path(&resolve_tls_cert_base_path())
        }

        pub fn default_key_file() -> String {
            key_file_path(&resolve_tls_cert_base_path())
        }

        /// Get the resolved TLS certificate base path for logging/debugging
        pub fn get_resolved_base_path() -> String {
            resolve_tls_cert_base_path()
        }
    }
}

/// HTTP headers and security constants
pub mod http {
    /// Default Content-Type header for JSON
    pub const CONTENT_TYPE_JSON: &str = "application/json";

    /// Default Content-Type header for protobuf
    pub const CONTENT_TYPE_PROTOBUF: &str = "application/x-protobuf";

    /// Default User-Agent header
    pub const DEFAULT_USER_AGENT: &str = "qollective-client/1.0";

    /// Authorization header name
    pub const HEADER_AUTHORIZATION: &str = "Authorization";

    /// Content-Type header name
    pub const HEADER_CONTENT_TYPE: &str = "Content-Type";

    /// Request ID header name
    pub const HEADER_REQUEST_ID: &str = "x-request-id";

    /// Request duration header name
    pub const HEADER_REQUEST_DURATION: &str = "x-request-duration-ms";

    /// Tenant ID header name
    pub const HEADER_TENANT_ID: &str = "x-tenant-id";

    /// On behalf of header name
    pub const HEADER_ON_BEHALF_OF: &str = "x-on-behalf-of";

    /// Qollective envelope metadata headers for REST transport
    #[cfg(any(feature = "rest-server", feature = "rest-client"))]
    pub mod envelope_headers {
        /// Qollective request ID header (envelope metadata)
        pub const QOLLECTIVE_REQUEST_ID: &str = "X-Qollective-Request-Id";

        /// Qollective tenant header (envelope metadata)
        pub const QOLLECTIVE_TENANT: &str = "X-Qollective-Tenant";

        /// Qollective version header (envelope metadata)
        pub const QOLLECTIVE_VERSION: &str = "X-Qollective-Version";

        /// Qollective timestamp header (envelope metadata)
        pub const QOLLECTIVE_TIMESTAMP: &str = "X-Qollective-Timestamp";

        /// Qollective complex metadata header (base64 encoded JSON)
        pub const QOLLECTIVE_META: &str = "X-Qollective-Meta";

        /// Qollective trace ID header for distributed tracing
        pub const QOLLECTIVE_TRACE_ID: &str = "X-Qollective-Trace-Id";

        /// Qollective span ID header for distributed tracing
        pub const QOLLECTIVE_SPAN_ID: &str = "X-Qollective-Span-Id";

        /// Qollective user ID header (security context)
        pub const QOLLECTIVE_USER_ID: &str = "X-Qollective-User-Id";

        /// Qollective session ID header (security context)
        pub const QOLLECTIVE_SESSION_ID: &str = "X-Qollective-Session-Id";

        /// Qollective correlation ID header (request correlation)
        pub const QOLLECTIVE_CORRELATION_ID: &str = "X-Qollective-Correlation-Id";
    }

    /// Qollective query parameters for REST transport (fallback when headers too large)
    #[cfg(any(feature = "rest-server", feature = "rest-client"))]
    pub mod envelope_query_params {
        /// Tenant query parameter (fallback for GET/DELETE)
        pub const TENANT: &str = "tenant";

        /// Version query parameter (fallback for GET/DELETE)
        pub const VERSION: &str = "version";

        /// Trace ID query parameter (fallback for GET/DELETE)
        pub const TRACE_ID: &str = "trace_id";

        /// User ID query parameter (fallback for GET/DELETE)
        pub const USER_ID: &str = "user_id";

        /// Session ID query parameter (fallback for GET/DELETE)
        pub const SESSION_ID: &str = "session_id";

        /// Correlation ID query parameter (fallback for GET/DELETE)
        pub const CORRELATION_ID: &str = "correlation_id";

        /// Request ID query parameter (fallback for GET/DELETE)
        pub const REQUEST_ID: &str = "request_id";
    }

    /// Default HSTS header value
    #[cfg(feature = "rest-server")]
    pub const DEFAULT_HSTS_HEADER_VALUE: &str = "max-age=31536000; includeSubDomains";

    /// Default X-Content-Type-Options header value
    #[cfg(feature = "rest-server")]
    pub const DEFAULT_CONTENT_TYPE_OPTIONS: &str = "nosniff";

    /// Default X-Frame-Options header value
    #[cfg(feature = "rest-server")]
    pub const DEFAULT_FRAME_OPTIONS: &str = "DENY";

    /// Default X-XSS-Protection header value
    #[cfg(feature = "rest-server")]
    pub const DEFAULT_XSS_PROTECTION: &str = "1; mode=block";
}

/// TLS and security configuration constants
pub mod tls {
    /// Default TLS cipher suites for production use
    pub const DEFAULT_CIPHER_SUITES: &[&str] = &[
        "TLS_AES_256_GCM_SHA384",
        "TLS_AES_128_GCM_SHA256",
        "TLS_CHACHA20_POLY1305_SHA256",
    ];

    /// Default TLS protocols for production use
    pub const DEFAULT_TLS_PROTOCOLS: &[&str] = &["TLSv1.3", "TLSv1.2"];

    /// Default cipher suites for high-performance configurations (TLS 1.3 only)
    pub const HIGH_PERFORMANCE_CIPHER_SUITES: &[&str] = &["TLS_AES_256_GCM_SHA384"];

    /// Default protocols for high-performance configurations (TLS 1.3 only)
    pub const HIGH_PERFORMANCE_TLS_PROTOCOLS: &[&str] = &["TLSv1.3"];

    /// Default cipher suites for development (broader compatibility)
    pub const DEVELOPMENT_CIPHER_SUITES: &[&str] = &[
        "TLS_AES_256_GCM_SHA384",
        "TLS_AES_128_GCM_SHA256",
        "TLS_CHACHA20_POLY1305_SHA256",
        "TLS_AES_128_CCM_SHA256",
    ];

    /// Default protocols for development (broader compatibility)
    pub const DEVELOPMENT_TLS_PROTOCOLS: &[&str] = &["TLSv1.3", "TLSv1.2"];

    /// Default certificate file extensions
    pub const CERT_FILE_EXTENSIONS: &[&str] = &[".pem", ".crt", ".cert"];

    /// Default private key file extensions
    pub const KEY_FILE_EXTENSIONS: &[&str] = &[".pem", ".key", ".priv"];

    /// Default CA certificate file extensions
    pub const CA_FILE_EXTENSIONS: &[&str] = &[".pem", ".crt", ".ca"];

    /// Default certificate validation modes
    pub mod verification {
        /// Verify certificates using system CA store
        pub const SYSTEM_CA: &str = "system_ca";

        /// Verify certificates using custom CA file
        pub const CUSTOM_CA: &str = "custom_ca";

        /// Skip certificate verification (insecure, development only)
        pub const SKIP: &str = "skip";

        /// Verify certificates and require client certificates (mTLS)
        pub const MUTUAL_TLS: &str = "mutual_tls";
    }

    /// TLS environment variable names
    pub mod env_vars {
        /// Global TLS enabled environment variable
        pub const QOLLECTIVE_TLS_ENABLED: &str = "QOLLECTIVE_TLS_ENABLED";

        /// TLS certificate path environment variable
        pub const QOLLECTIVE_TLS_CERT_PATH: &str = "QOLLECTIVE_TLS_CERT_PATH";

        /// TLS private key path environment variable
        pub const QOLLECTIVE_TLS_KEY_PATH: &str = "QOLLECTIVE_TLS_KEY_PATH";

        /// TLS CA certificate path environment variable
        pub const QOLLECTIVE_TLS_CA_PATH: &str = "QOLLECTIVE_TLS_CA_PATH";

        /// TLS certificate verification mode environment variable
        pub const QOLLECTIVE_TLS_VERIFY_MODE: &str = "QOLLECTIVE_TLS_VERIFY_MODE";

        /// TLS cipher suites override environment variable
        pub const QOLLECTIVE_TLS_CIPHER_SUITES: &str = "QOLLECTIVE_TLS_CIPHER_SUITES";

        /// TLS protocols override environment variable
        pub const QOLLECTIVE_TLS_PROTOCOLS: &str = "QOLLECTIVE_TLS_PROTOCOLS";

        /// mTLS client certificate required environment variable
        pub const QOLLECTIVE_TLS_CLIENT_CERT_REQUIRED: &str = "QOLLECTIVE_TLS_CLIENT_CERT_REQUIRED";

        /// TLS certificate base path environment variable (already exists but referenced here)
        pub const QOLLECTIVE_TLS_CERT_BASE_PATH: &str =
            super::super::env_vars::QOLLECTIVE_TLS_CERT_BASE_PATH;
    }
}

/// Environment variable names
pub mod env_vars {
    /// Tenant extraction enabled environment variable
    pub const QOLLECTIVE_TENANT_EXTRACTION: &str = "QOLLECTIVE_TENANT_EXTRACTION";

    /// gRPC timeout environment variable
    pub const QOLLECTIVE_GRPC_TIMEOUT: &str = "QOLLECTIVE_GRPC_TIMEOUT";

    /// REST timeout environment variable
    pub const QOLLECTIVE_REST_TIMEOUT: &str = "QOLLECTIVE_REST_TIMEOUT";

    /// NATS URL environment variable
    pub const QOLLECTIVE_NATS_URL: &str = "QOLLECTIVE_NATS_URL";

    // NATS Connection environment variables
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_URLS: &str = "QOLLECTIVE_NATS_URLS";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_CONNECTION_TIMEOUT: &str = "QOLLECTIVE_NATS_CONNECTION_TIMEOUT";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_RECONNECT_TIMEOUT: &str = "QOLLECTIVE_NATS_RECONNECT_TIMEOUT";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_MAX_RECONNECT_ATTEMPTS: &str =
        "QOLLECTIVE_NATS_MAX_RECONNECT_ATTEMPTS";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_USERNAME: &str = "QOLLECTIVE_NATS_USERNAME";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_PASSWORD: &str = "QOLLECTIVE_NATS_PASSWORD";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_TOKEN: &str = "QOLLECTIVE_NATS_TOKEN";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_TLS_ENABLED: &str = "QOLLECTIVE_NATS_TLS_ENABLED";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_TLS_CA_FILE: &str = "QOLLECTIVE_NATS_TLS_CA_FILE";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_TLS_CERT_FILE: &str = "QOLLECTIVE_NATS_TLS_CERT_FILE";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_TLS_KEY_FILE: &str = "QOLLECTIVE_NATS_TLS_KEY_FILE";

    // NATS Client environment variables
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_CLIENT_TIMEOUT: &str = "QOLLECTIVE_NATS_CLIENT_TIMEOUT";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_CLIENT_MAX_PENDING: &str = "QOLLECTIVE_NATS_CLIENT_MAX_PENDING";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_CLIENT_RETRY_ATTEMPTS: &str = "QOLLECTIVE_NATS_CLIENT_RETRY_ATTEMPTS";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_CLIENT_RETRY_DELAY: &str = "QOLLECTIVE_NATS_CLIENT_RETRY_DELAY";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_CLIENT_POOL_SIZE: &str = "QOLLECTIVE_NATS_CLIENT_POOL_SIZE";

    // NATS Server environment variables
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_SERVER_ENABLED: &str = "QOLLECTIVE_NATS_SERVER_ENABLED";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_SERVER_PREFIX: &str = "QOLLECTIVE_NATS_SERVER_PREFIX";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_SERVER_QUEUE_GROUP: &str = "QOLLECTIVE_NATS_SERVER_QUEUE_GROUP";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_SERVER_MAX_HANDLERS: &str = "QOLLECTIVE_NATS_SERVER_MAX_HANDLERS";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_SERVER_HANDLER_TIMEOUT: &str =
        "QOLLECTIVE_NATS_SERVER_HANDLER_TIMEOUT";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_SERVER_REQUEST_REPLY: &str = "QOLLECTIVE_NATS_SERVER_REQUEST_REPLY";

    // NATS Discovery environment variables
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_DISCOVERY_ENABLED: &str = "QOLLECTIVE_NATS_DISCOVERY_ENABLED";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_DISCOVERY_REGISTRY_SUBJECT: &str =
        "QOLLECTIVE_NATS_DISCOVERY_REGISTRY_SUBJECT";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_DISCOVERY_CAPABILITY_SUBJECT: &str =
        "QOLLECTIVE_NATS_DISCOVERY_CAPABILITY_SUBJECT";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_DISCOVERY_ANNOUNCEMENT_INTERVAL: &str =
        "QOLLECTIVE_NATS_DISCOVERY_ANNOUNCEMENT_INTERVAL";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_DISCOVERY_TTL: &str = "QOLLECTIVE_NATS_DISCOVERY_TTL";
    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub const QOLLECTIVE_NATS_DISCOVERY_AUTO_REGISTER: &str =
        "QOLLECTIVE_NATS_DISCOVERY_AUTO_REGISTER";

    /// Redis URL environment variable
    pub const QOLLECTIVE_REDIS_URL: &str = "QOLLECTIVE_REDIS_URL";

    /// Log level environment variable
    pub const QOLLECTIVE_LOG_LEVEL: &str = "QOLLECTIVE_LOG_LEVEL";

    /// TLS enabled environment variable (defined in tls::env_vars module)
    pub const QOLLECTIVE_TLS_ENABLED: &str = "QOLLECTIVE_TLS_ENABLED";

    /// TLS certificate base path environment variable
    pub const QOLLECTIVE_TLS_CERT_BASE_PATH: &str = "QOLLECTIVE_TLS_CERT_BASE_PATH";
}

/// Version and metadata constants
pub mod metadata {
    /// Default capability version
    pub const DEFAULT_CAPABILITY_VERSION: &str = "1.0";

    /// Default API version
    pub const DEFAULT_API_VERSION: &str = "v1";

    /// Default MCP protocol version
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub const DEFAULT_MCP_PROTOCOL_VERSION: &str = "1.0.0";

    /// Extension key for protocol metadata in envelope extensions
    pub const PROTOCOL_EXTENSION_KEY: &str = "protocol";

    /// Framework version
    pub const QOLLECTIVE_VERSION: &str = "0.1.0";

    /// Default service name for health checks
    pub const DEFAULT_SERVICE_NAME: &str = "qollective-service";

    /// Health check status: healthy
    pub const HEALTH_STATUS_HEALTHY: &str = "healthy";

    /// Health check status: unhealthy
    pub const HEALTH_STATUS_UNHEALTHY: &str = "unhealthy";

    /// Health check status: serving (gRPC)
    pub const HEALTH_STATUS_SERVING: i32 = 1;

    /// Health check status: not serving (gRPC)
    pub const HEALTH_STATUS_NOT_SERVING: i32 = 0;
}

/// WASM-specific constants and limits
#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod wasm {
    /// Maximum WASM bundle size in bytes (500KB as per PRP requirements)
    pub const MAX_BUNDLE_SIZE: usize = 500_000;

    /// Default WASM client timeouts
    pub mod timeouts {
        /// Default WebAssembly request timeout in milliseconds
        pub const DEFAULT_WASM_REQUEST_TIMEOUT_MS: u64 = 30000;

        /// Default WASM connection timeout in milliseconds
        pub const DEFAULT_WASM_CONNECTION_TIMEOUT_MS: u64 = 10000;

        /// Default WASM tool execution timeout in milliseconds
        pub const DEFAULT_WASM_TOOL_TIMEOUT_MS: u64 = 60000;

        /// Default certificate validation timeout in milliseconds
        pub const DEFAULT_WASM_CERT_TIMEOUT_MS: u64 = 5000;

        /// Default WebSocket ping interval in WASM (milliseconds)
        pub const DEFAULT_WASM_WS_PING_INTERVAL_MS: u64 = 30000;
    }

    /// WASM bundle optimization constants
    pub mod bundle {
        /// Target bundle size for optimization
        pub const TARGET_BUNDLE_SIZE: usize = 400_000; // 400KB target, 500KB max

        /// Minimum required free memory for WASM operation (bytes)
        pub const MIN_FREE_MEMORY: usize = 1_048_576; // 1MB

        /// Maximum concurrent WebSocket connections in WASM
        pub const MAX_WEBSOCKET_CONNECTIONS: usize = 10;

        /// Maximum concurrent MCP tool calls in WASM
        pub const MAX_CONCURRENT_MCP_CALLS: usize = 5;

        /// Maximum certificate cache size in WASM (bytes)
        pub const MAX_CERT_CACHE_SIZE: usize = 102_400; // 100KB
    }

    /// Browser storage constants
    pub mod storage {
        /// Default local storage key prefix for WASM
        pub const LOCAL_STORAGE_PREFIX: &str = "qollective_wasm_";

        /// Maximum local storage usage (bytes)
        pub const MAX_LOCAL_STORAGE_SIZE: usize = 10_485_760; // 10MB

        /// Certificate cache key in local storage
        pub const CERT_CACHE_KEY: &str = "certificates";

        /// Tool definition cache key in local storage
        pub const TOOL_CACHE_KEY: &str = "mcp_tools";

        /// Configuration cache key in local storage
        pub const CONFIG_CACHE_KEY: &str = "client_config";

        /// Default cache TTL in seconds
        pub const DEFAULT_CACHE_TTL_SECS: u64 = 3600; // 1 hour
    }

    /// WASM envelope constants
    pub mod envelope {
        /// Maximum envelope size for WASM (bytes)
        pub const MAX_ENVELOPE_SIZE: usize = 1_048_576; // 1MB

        /// Maximum metadata size for WASM (bytes)
        pub const MAX_METADATA_SIZE: usize = 65_536; // 64KB

        /// Maximum data payload size for WASM (bytes)
        pub const MAX_DATA_SIZE: usize = 983_040; // ~960KB (1MB - 64KB metadata overhead)

        /// Default WASM envelope version
        pub const DEFAULT_WASM_VERSION: &str = "wasm-1.0";
    }

    /// JavaScript interop constants
    pub mod js_interop {
        /// Maximum string length for JS conversion
        pub const MAX_JS_STRING_LENGTH: usize = 1_048_576; // 1MB

        /// Maximum array length for JS conversion
        pub const MAX_JS_ARRAY_LENGTH: usize = 10_000;

        /// Maximum object property count for JS conversion
        pub const MAX_JS_OBJECT_PROPERTIES: usize = 1_000;

        /// Default UTF-8 buffer size for string conversion
        pub const JS_STRING_BUFFER_SIZE: usize = 4_096; // 4KB
    }

    /// Error handling constants for WASM
    pub mod errors {
        /// Maximum error message length for user display
        pub const MAX_USER_ERROR_LENGTH: usize = 200;

        /// Maximum error details length for logging
        pub const MAX_ERROR_DETAILS_LENGTH: usize = 1000;

        /// Default retry attempts for transient errors
        pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;

        /// Default retry delay in milliseconds
        pub const DEFAULT_RETRY_DELAY_MS: u64 = 1000;

        /// Maximum retry delay in milliseconds
        pub const MAX_RETRY_DELAY_MS: u64 = 30000;
    }

    /// Browser compatibility constants
    pub mod compatibility {
        /// Minimum Chrome version for WASM support
        pub const MIN_CHROME_VERSION: u32 = 88;

        /// Minimum Firefox version for WASM support
        pub const MIN_FIREFOX_VERSION: u32 = 85;

        /// Minimum Safari version for WASM support
        pub const MIN_SAFARI_VERSION: u32 = 14;

        /// Minimum Edge version for WASM support
        pub const MIN_EDGE_VERSION: u32 = 88;

        /// Features required for full WASM functionality
        pub const REQUIRED_FEATURES: &[&str] =
            &["wasm", "fetch", "websocket", "promise", "async", "bigint"];
    }
}

/// Helper functions for endpoint generation
pub mod helpers {
    use super::endpoints::*;

    /// Generate agent endpoint URL from name
    pub fn agent_endpoint_url(agent_name: &str) -> String {
        DEFAULT_AGENT_ENDPOINT_PATTERN.replace("{agent_name}", agent_name)
    }

    /// Generate external agent endpoint URL from ID
    pub fn external_agent_endpoint_url(agent_id: &str) -> String {
        DEFAULT_EXTERNAL_AGENT_PATTERN.replace("{agent_id}", agent_id)
    }

    /// Generate MCP server endpoint URL from ID
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub fn mcp_server_endpoint_url(server_id: &str) -> String {
        DEFAULT_MCP_SERVER_PATTERN.replace("{server_id}", server_id)
    }

    /// Check if URL looks like a Qollective endpoint
    pub fn is_qollective_endpoint(url: &str) -> bool {
        url.contains(DEFAULT_QOLLECTIVE_DOMAIN) || url.contains("qollective")
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_constants() {
        use timeouts::*;

        assert!(DEFAULT_TRANSPORT_DETECTION_TIMEOUT.as_secs() > 0);
        assert!(DEFAULT_CAPABILITY_CACHE_TTL.as_secs() > 0);
        assert!(DEFAULT_CIRCUIT_BREAKER_RECOVERY.as_secs() > 0);
    }

    #[test]
    fn test_subject_patterns() {
        use subjects::*;

        // Verify all subjects follow shared pattern
        #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
        {
            assert!(AGENT_REGISTRATION.starts_with("qollective.a2a.v1."));
            assert!(AGENT_DISCOVERY.starts_with("qollective.a2a.v1."));
            assert!(AGENT_HEARTBEAT.starts_with("qollective.a2a.v1."));
            assert!(AGENT_CAPABILITIES.starts_with("qollective.a2a.v1."));
            assert!(AGENT_HEALTH.starts_with("qollective.a2a.v1."));
            assert!(AGENT_REGISTRY_EVENTS.starts_with("qollective.a2a.v1."));
            assert!(AGENT_REGISTRY_ANNOUNCE.starts_with("qollective.a2a.v1."));
        }

        #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
        {
            assert!(MCP_TOOL_DISCOVER.starts_with("qollective.mcp.v1."));
            assert!(MCP_TOOL_EXECUTE.starts_with("qollective.mcp.v1."));
            assert!(MCP_TOOL_CHAIN.starts_with("qollective.mcp.v1."));
            assert!(MCP_SERVER_ANNOUNCE.starts_with("qollective.mcp.v1."));
            assert!(MCP_CAPABILITIES.starts_with("qollective.mcp.v1."));
            assert!(MCP_HEALTH.starts_with("qollective.mcp.v1."));
        }
    }

    #[test]
    fn test_helper_functions() {
        use helpers::*;

        #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
        {
            let url = agent_endpoint_url("test-agent");
            assert!(url.contains("test-agent"));
            assert!(url.contains("qollective.local"));
        }

        assert!(is_qollective_endpoint("https://service.qollective.local"));
        assert!(is_qollective_endpoint(
            "https://qollective-service.example.com"
        ));
        assert!(!is_qollective_endpoint(
            "https://external-service.example.com"
        ));
    }

    #[test]
    fn test_validation_constants() {
        use validation::*;

        assert!(MAX_AGENT_NAME_LENGTH > 0);
        assert!(MAX_CAPABILITY_NAME_LENGTH > 0);
        assert!(MAX_CAPABILITIES_PER_AGENT > 0);
        assert!(MAX_METADATA_KEY_LENGTH > 0);
        assert!(MAX_METADATA_VALUE_LENGTH > MAX_METADATA_KEY_LENGTH);
    }

    #[test]
    fn test_network_constants() {
        use network::*;

        assert!(!DEFAULT_BIND_ALL_INTERFACES.is_empty());
        assert!(!DEFAULT_BIND_LOCALHOST.is_empty());
        assert!(!DEFAULT_LOCALHOST.is_empty());

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            assert!(DEFAULT_NATS_URL.starts_with("nats://"));
            assert!(DEFAULT_NATS_PORT > 0);
        }

        #[cfg(feature = "grpc-server")]
        assert!(DEFAULT_GRPC_SERVER_PORT > 0);

        #[cfg(feature = "rest-server")]
        assert!(DEFAULT_REST_SERVER_PORT > 0);
    }

    #[test]
    fn test_limits_constants() {
        use limits::*;

        assert!(DEFAULT_MAX_AGENTS > 0);

        #[cfg(feature = "rest-server")]
        assert!(DEFAULT_REST_MAX_REQUEST_SIZE > 0);

        #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
        {
            assert!(DEFAULT_GRPC_MAX_FRAME_SIZE > 0);
            assert!(DEFAULT_GRPC_MAX_CONCURRENT_STREAMS > 0);
            assert!(DEFAULT_GRPC_MAX_IDLE_CONNECTIONS > 0);
        }

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            assert!(DEFAULT_NATS_MAX_PENDING_MESSAGES > 0);
            assert!(DEFAULT_NATS_MAX_RECONNECT_ATTEMPTS > 0);
        }
    }

    #[test]
    fn test_http_constants() {
        use http::*;

        assert!(!CONTENT_TYPE_JSON.is_empty());
        assert!(!CONTENT_TYPE_PROTOBUF.is_empty());
        assert!(!DEFAULT_USER_AGENT.is_empty());
        assert!(!HEADER_AUTHORIZATION.is_empty());
        assert!(!HEADER_CONTENT_TYPE.is_empty());
        assert!(!HEADER_REQUEST_ID.is_empty());

        #[cfg(feature = "rest-server")]
        {
            assert!(!DEFAULT_HSTS_HEADER_VALUE.is_empty());
            assert!(!DEFAULT_CONTENT_TYPE_OPTIONS.is_empty());
            assert!(!DEFAULT_FRAME_OPTIONS.is_empty());
            assert!(!DEFAULT_XSS_PROTECTION.is_empty());
        }
    }

    #[test]
    fn test_metadata_constants() {
        use metadata::*;

        assert!(!DEFAULT_CAPABILITY_VERSION.is_empty());
        assert!(!DEFAULT_API_VERSION.is_empty());
        assert!(!QOLLECTIVE_VERSION.is_empty());
        assert!(!DEFAULT_SERVICE_NAME.is_empty());
        assert!(!HEALTH_STATUS_HEALTHY.is_empty());
        assert!(!HEALTH_STATUS_UNHEALTHY.is_empty());
        assert!(HEALTH_STATUS_SERVING >= 0);
        assert!(HEALTH_STATUS_NOT_SERVING >= 0);
    }

    #[test]
    fn test_env_vars_constants() {
        use env_vars::*;

        assert!(!QOLLECTIVE_TENANT_EXTRACTION.is_empty());
        assert!(!QOLLECTIVE_GRPC_TIMEOUT.is_empty());
        assert!(!QOLLECTIVE_REST_TIMEOUT.is_empty());
        assert!(!QOLLECTIVE_NATS_URL.is_empty());
        assert!(!QOLLECTIVE_REDIS_URL.is_empty());
        assert!(!QOLLECTIVE_LOG_LEVEL.is_empty());
        assert!(!QOLLECTIVE_TLS_ENABLED.is_empty());

        // Verify all environment variables start with QOLLECTIVE_
        assert!(QOLLECTIVE_TENANT_EXTRACTION.starts_with("QOLLECTIVE_"));
        assert!(QOLLECTIVE_GRPC_TIMEOUT.starts_with("QOLLECTIVE_"));
        assert!(QOLLECTIVE_REST_TIMEOUT.starts_with("QOLLECTIVE_"));
        assert!(QOLLECTIVE_NATS_URL.starts_with("QOLLECTIVE_"));
        assert!(QOLLECTIVE_REDIS_URL.starts_with("QOLLECTIVE_"));
        assert!(QOLLECTIVE_LOG_LEVEL.starts_with("QOLLECTIVE_"));
        assert!(QOLLECTIVE_TLS_ENABLED.starts_with("QOLLECTIVE_"));
    }

    #[test]
    fn test_timeout_values_consistency() {
        use timeouts::*;

        // Verify timeout values are reasonable
        #[cfg(feature = "rest-server")]
        assert!(DEFAULT_REST_REQUEST_TIMEOUT_MS >= 1000); // At least 1 second

        #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
        assert!(DEFAULT_GRPC_TIMEOUT_MS >= 1000); // At least 1 second

        #[cfg(any(feature = "nats-client", feature = "nats-server"))]
        {
            assert!(DEFAULT_NATS_CONNECTION_TIMEOUT_MS >= 1000); // At least 1 second
            assert!(DEFAULT_NATS_REQUEST_TIMEOUT_MS >= DEFAULT_NATS_CONNECTION_TIMEOUT_MS);
        }

        assert!(DEFAULT_AGENT_TTL_SECS >= DEFAULT_AGENT_CLEANUP_INTERVAL_SECS);

        #[cfg(feature = "security")]
        {
            assert!(DEFAULT_SECURITY_TTL_SECS > DEFAULT_JWT_REFRESH_THRESHOLD_SECS);
        }
    }
}
