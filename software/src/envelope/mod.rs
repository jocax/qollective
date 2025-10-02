// ABOUTME: Core envelope structures and metadata handling
// ABOUTME: Implements the standard envelope pattern for cross-service communication

//! Core envelope structures and metadata handling.
//!
//! This module provides the fundamental envelope pattern that wraps all
//! service-to-service communication with consistent metadata and data sections.

pub mod builder;
pub mod context;
pub mod meta;
pub mod middleware;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub mod nats_codec;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub mod nats;

#[cfg(feature = "tenant-extraction")]
pub mod jwt_extractor;

#[cfg(feature = "tenant-extraction")]
pub mod tenant_middleware;

#[cfg(feature = "tenant-extraction")]
pub mod unified_tenant_extraction;

pub use builder::{Envelope, EnvelopeBuilder, EnvelopeError};
pub use context::{Context, ContextBuilder, ContextPropagation};
pub use meta::{Meta, MetaBuilder, MetaSection};
pub use middleware::{
    propagation, ContextMiddleware, EnvelopeMiddleware, HeaderLike, MiddlewareBuilder,
    MiddlewareConfig,
};
#[cfg(feature = "tenant-extraction")]
pub use tenant_middleware::TenantExtractionMiddleware;

#[cfg(feature = "tenant-extraction")]
pub use unified_tenant_extraction::UnifiedTenantExtractor;

// Re-exports for convenience
pub use meta::{
    DebugMeta, ExtensionsMeta, MonitoringMeta, OnBehalfOfMeta, PerformanceMeta, SecurityMeta, TracingMeta,
};

#[cfg(feature = "tenant-extraction")]
pub use jwt_extractor::{
    DefaultJwtTenantExtractor, DefaultJwtTokenLocator, HttpRequest, JwtExtractionError,
    JwtProcessingError, JwtProcessor, JwtProcessorBuilder, JwtTenantExtractor, JwtTenantInfo,
    JwtTokenLocator, OnBehalfOfInfo, TokenLocationError,
};

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use nats_codec::{CodecError, NatsEnvelopeCodec};

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
pub use nats::{SubjectPattern, SubjectPatternBuilder};
