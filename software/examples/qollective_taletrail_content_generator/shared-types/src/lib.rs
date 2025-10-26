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

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

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
pub mod validation;

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

// ============================================================================
// Validation Policy Types
// ============================================================================

/// Validation policy for content generation requests
///
/// Provides flexible control over validation behavior including:
/// - Enable/disable validation entirely
/// - Custom restricted words per language
/// - Merge strategies for combining custom words with config
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct ValidationPolicy {
    /// Enable/disable all validation for this request
    #[serde(default = "default_enable_validation")]
    pub enable_validation: bool,

    /// Custom restricted words for this request
    /// Key: Language code (en, de, es, fr)
    /// Value: List of words to avoid in generation and check in validation
    #[serde(default)]
    pub custom_restricted_words: HashMap<String, Vec<String>>,

    /// How to combine custom words with config file words
    #[serde(default)]
    pub merge_mode: RestrictedWordsMergeMode,
}

/// Strategy for combining custom restricted words with config file words
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
pub enum RestrictedWordsMergeMode {
    /// Use custom words only, ignore config file
    Replace,

    /// Merge custom words with config file words (default)
    #[default]
    Merge,

    /// Use config file words only, ignore custom words
    ConfigOnly,
}

fn default_enable_validation() -> bool {
    true
}
