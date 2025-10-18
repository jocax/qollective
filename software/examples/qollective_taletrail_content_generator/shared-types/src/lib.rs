//! Shared types for TaleTrail Content Generator
//!
//! This crate provides common types and utilities shared
//! across all TaleTrail services.
//!
//! # Envelope-First Architecture
//!
//! TaleTrail follows Qollective's envelope-first pattern:
//! - Use `Envelope<TaleTrailPayload>` directly (not custom wrappers)
//! - Extend metadata via `Meta.extensions` using `TaleTrailCustomMetadata`
//! - Use helper functions in `helpers` module for ergonomic envelope creation
//!
//! Note: Configuration is handled per-service using Figment with
//! Defaults → config.toml → Environment variables hierarchy.

pub mod constants;
pub mod contract_tests;
pub mod custom_metadata;
pub mod errors;
pub mod extensions;
pub mod helpers;
pub mod nats_nkey;
pub mod payloads;
pub mod traits;
pub mod types;

// Test utilities (only available in test builds)
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

// Re-export commonly used items
pub use constants::*;
pub use errors::{Result, TaleTrailError};
pub use nats_nkey::{connect_with_nkey, load_nkey_from_file};

// Re-export generated types from shared-types-generated crate
pub use shared_types_generated::*;

// Re-export custom metadata extensions
pub use custom_metadata::TaleTrailCustomMetadata;

// Re-export payload types
pub use payloads::TaleTrailPayload;

// Re-export helper functions
pub use helpers::{
    create_request_envelope, create_response_envelope, create_validation_envelope,
    extract_custom_metadata, extract_tenant_id, extract_user_id,
};

// Re-export business logic extensions
pub use extensions::*;

// Re-export trait definitions
pub use traits::{
    LlmService, McpTransport, PromptHelperService, RequestMapper, StoryGeneratorService,
    ValidationService,
};

// NOTE: MCPServiceType is now only from the generated types (shared_types_generated::MCPServiceType)
// The manual version in types/mod.rs is deprecated and should not be used

// Re-export mock types when mocking feature is enabled
#[cfg(any(test, feature = "mocking"))]
pub use traits::{
    MockLlmService,
    MockPromptHelperService,
    MockMcpTransport,
    MockRequestMapper,
    MockStoryGeneratorService,
    MockValidationService,
};
