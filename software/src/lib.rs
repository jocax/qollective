// ABOUTME: Main library entry point for the Qollective runtime
// ABOUTME: Provides cross-language, cross-protocol data harmonization framework

//! # Qollective Framework
//!
//! A cross-language, cross-protocol data harmonization system that provides consistent
//! envelope structures and context propagation for service-to-service communication.
//!
//! ## Features
//!
//! - **Envelope Pattern**: All communication uses standardized wrapper with `meta` and `data` sections
//! - **Context Propagation**: Metadata flows seamlessly through service layers as immutable context
//! - **Multi-Protocol**: Transport-agnostic design works with REST, gRPC, and other protocols
//! - **Configurable Metadata**: Fine-grained control over metadata inclusion per environment/endpoint/request
//!
//! ## Quick Start
//!
//! ```rust
//! use qollective::prelude::*;
//! # use serde::{Serialize, Deserialize};
//! # #[derive(Serialize, Deserialize)] struct UserCreateRequest;
//! # #[derive(Serialize, Deserialize)] struct UserCreateResponse;
//! # fn example() -> qollective::error::Result<()> {
//! # let request = Envelope::new(Meta::default(), UserCreateRequest);
//! # let result = UserCreateResponse;
//!
//! // Parse incoming request
//! let (meta, user_data) = request.extract();
//!
//! // Build response with enriched metadata
//! let response = Envelope::builder()
//!     .with_payload(result)
//!     .with_meta(meta.enrich_performance())
//!     .build()?;
//! # Ok(())
//! # }
//! ```

// Public framework traits for user business logic
use crate::envelope::Context;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Public client handler trait for framework users.
///
/// This trait defines the interface for client-side business logic handlers.
/// It receives context and data extracted from envelopes and returns response data
/// that will be wrapped back into envelopes by the framework.
#[async_trait]
pub trait ClientHandler<T, R>: Send + Sync
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
{
    /// Handle client request with context and data.
    ///
    /// # Arguments
    ///
    /// * `context` - Optional context information from the envelope
    /// * `data` - The request data extracted from the envelope
    ///
    /// # Returns
    ///
    /// Returns a `Result<R>` containing either the response data
    /// or an error if processing failed.
    async fn handle(&self, context: Option<Context>, data: T) -> Result<R>;
}

/// Public server handler trait for framework users.
///
/// This trait defines the interface for server-side business logic handlers.
/// It receives context and data extracted from envelopes and returns response data
/// that will be wrapped back into envelopes by the framework.
#[async_trait]
pub trait ServerHandler<T, R>: Send + Sync
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
{
    /// Handle server request with context and data.
    ///
    /// # Arguments
    ///
    /// * `context` - Optional context information from the envelope
    /// * `data` - The request data extracted from the envelope
    ///
    /// # Returns
    ///
    /// Returns a `Result<R>` containing either the response data
    /// or an error if processing failed.
    async fn handle(&self, context: Option<Context>, data: T) -> Result<R>;
}

// Re-export commonly used types for convenience
pub mod prelude {
    pub use crate::envelope::{
        Context, ContextPropagation, Envelope, EnvelopeBuilder, Meta, MetaBuilder,
    };
    pub use crate::error::{QollectiveError, Result};
    pub use crate::{ClientHandler, ServerHandler};

    #[cfg(feature = "config")]
    pub use crate::config::{MetaConfig, QollectiveConfig};

    #[cfg(feature = "rest-client")]
    pub use crate::client::rest::RestClient;

    // NOTE: Server exports temporarily disabled during envelope-first refactor
    // #[cfg(feature = "rest-server")]
    // pub use crate::server::rest::RestServer;

    #[cfg(feature = "grpc-client")]
    pub use crate::client::grpc::GrpcClient;

    // #[cfg(feature = "grpc-server")]
    // pub use crate::server::grpc::GrpcServer;

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub use crate::client::nats::{
        ConnectionEvent, ConnectionMetrics, ConnectionState, NatsClient,
    };

    // #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    // pub use crate::server::nats::{NatsServer, EnvelopeHandler};

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    pub use crate::envelope::nats::{SubjectPattern, SubjectPatternBuilder};

    // Transport traits for envelope communication
    pub use crate::traits::handlers::{
        ContextDataHandler, DefaultContextDataHandler, DefaultEnvelopeHandler, EnvelopeHandler,
    };
    pub use crate::traits::receivers::UnifiedEnvelopeReceiver;
    pub use crate::traits::senders::{UnifiedEnvelopeSender, UnifiedSender};

    #[cfg(feature = "a2a-client")]
    pub use crate::client::a2a::{A2AClient, A2AMessageType, A2AMetadata};

    // #[cfg(feature = "a2a-server")]
    // pub use crate::server::a2a::A2AServer;

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub use crate::types::a2a::{
        AgentId, AgentInfo, CapabilityQuery, HealthStatus as AgentHealthStatus,
    };

    // Note: AgentRegistry and AgentMetadata were simplified in envelope-first refactor

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    pub use crate::config::a2a::{
        A2AClientConfig, A2AServerConfig, A2ASubjectConfig, HealthConfig, RegistryConfig,
        RoutingConfig,
    };

    #[cfg(any(feature = "a2a-client", feature = "a2a-server"))]
    // pub use crate::transport::a2a::AgentTransportClient;
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub use crate::types::mcp::{
        AsyncConfig, McpData, McpDiscoveryData, McpServerInfo, ServerMetadata, SslConfig,
    };

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    pub use crate::types::mcp::HealthStatus as McpHealthStatus;

    // #[cfg(feature = "mcp-server")]
    // pub use crate::server::mcp::{McpServer, McpResource, McpPrompt};

    #[cfg(feature = "tenant-extraction")]
    pub use crate::tenant::{JwtClaims, JwtParser, TenantExtractor, TenantInfo};

    #[cfg(feature = "security")]
    pub use crate::security::{
        DefaultJwtValidator, DefaultTokenScopeValidator, InMemoryTokenStorage, JwtValidator,
        RoleBasedScopeValidator, SecureTokenStorage, SecurityConfig, Token, TokenScopeValidator,
        TokenValidationError,
    };

    pub use crate::monitoring::{
        get_metrics_summary, record_envelope_operation, record_http_request, start_operation_timer,
        OperationTimer,
    };

    pub use crate::transport::{
        HybridTransportClient, ServerInfo, TransportCapabilities, TransportDetectionConfig,
        TransportMetrics, TransportProtocol, TransportRequirements,
    };

    // WASM client exports (only for wasm32 target with wasm-client feature)
    #[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
    pub use crate::wasm::{ErrorTranslator, WasmClient, WasmContext, WasmEnvelope, WasmMeta};

    pub use crate::constants;
}

// Core modules (always available)
pub mod crypto;
pub mod envelope;
pub mod error;

// Generated protobuf types for gRPC support
#[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
pub mod generated;

// Optional modules based on features
#[cfg(feature = "config")]
pub mod config;

// Client module - supports any client-side protocols
// Examples: #[cfg(any(feature = "rest-client", feature = "grpc-client"))]
#[cfg(any(
    feature = "rest-client",
    feature = "grpc-client",
    feature = "nats-client",
    feature = "mcp-client"
))]
pub mod client;

// Server module - supports any server-side protocols
#[cfg(any(
    feature = "rest-server",
    feature = "grpc-server",
    feature = "nats-server"
))]
pub mod server;

#[cfg(feature = "tenant-extraction")]
pub mod tenant;

// Security modules for token propagation and validation
#[cfg(feature = "security")]
pub mod security;

// Monitoring and observability
pub mod monitoring;

// Core transport traits (internal framework use only)
mod traits;

// Hybrid transport architecture
// NOTE: Re-enabled for Step 2 - HybridTransportClient trait implementation
pub mod transport;

// Framework constants and default values
pub mod constants;

// Shared types across modules
pub mod types;

// WASM client module (only for wasm32 target with wasm-client feature)
#[cfg(all(target_arch = "wasm32", feature = "wasm-client"))]
pub mod wasm;

/// OpenAPI utilities and schema generation
#[cfg(feature = "openapi")]
pub mod openapi;

// Crypto provider initialization (prominently exported for discoverability)
pub use crypto::ensure_crypto_provider;

// Top-level server re-exports for convenience
// NOTE: Temporarily disabled during envelope-first refactor
// #[cfg(feature = "rest-server")]
// pub use server::rest::{RestServer, RestServerBuilder, TlsConfig};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }
}
