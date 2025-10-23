/// Constants for TaleTrail Desktop Viewer
///
/// Following CONSTANTS FIRST principle - all hardcoded values are defined here

/// NATS subject patterns
pub mod nats {
    /// Base subject for generation events (subscribed by desktop)
    pub const GENERATION_EVENTS_BASE: &str = "taletrail.generation.events";

    /// Tenant-specific generation events pattern
    pub const GENERATION_EVENTS_TENANT_PATTERN: &str = "taletrail.generation.events.{tenant_id}";

    /// MCP Orchestrator request subject (desktop publishes to this)
    pub const ORCHESTRATOR_REQUEST_SUBJECT: &str = "mcp.orchestrator.request";
}

/// Validation constraints for generation requests
pub mod validation {
    // Note: Age groups, languages, and vocabulary levels are now enforced by enum types
    // See shared_types_generated::{AgeGroup, Language, VocabularyLevel}

    /// Valid story structure presets (Tier 1: Simple)
    /// Used for predefined DAG configurations
    pub const VALID_STORY_STRUCTURES: &[&str] = &[
        "guided",           // Linear path with minimal branching
        "adventure",        // Moderate branching with convergence
        "epic",             // Complex branching with multiple convergence points
        "choose_your_path", // Maximum branching, multiple endings
    ];

    /// Minimum allowed node count
    pub const MIN_NODE_COUNT: u32 = 5;

    /// Maximum allowed node count
    pub const MAX_NODE_COUNT: u32 = 50;

    /// Default maximum choices per node
    pub const DEFAULT_MAX_CHOICES_PER_NODE: u32 = 4;

    /// Minimum allowed choices per node
    pub const MIN_CHOICES_PER_NODE: u32 = 2;

    /// Maximum allowed choices per node
    pub const MAX_CHOICES_PER_NODE: u32 = 10;

    /// Minimum story length (characters)
    pub const MIN_STORY_LENGTH: u32 = 100;

    /// Maximum story length (characters)
    pub const MAX_STORY_LENGTH: u32 = 10000;
}

/// Default values
pub mod defaults {
    /// Default NATS server URL
    pub const NATS_URL: &str = "nats://localhost:5222";

    /// Default NATS client name
    pub const NATS_CLIENT_NAME: &str = "taletrail-desktop";

    /// Default NATS connection timeout in seconds
    pub const NATS_TIMEOUT_SECS: u64 = 5;
}

/// Request timeout configuration
pub mod timeouts {
    /// Default NATS request timeout in seconds
    /// Matches nats-cli::DEFAULT_TIMEOUT_SECS for consistency across examples
    pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 180; // 3 minutes

    /// Minimum allowed timeout in seconds
    pub const MIN_REQUEST_TIMEOUT_SECS: u64 = 30;

    /// Maximum allowed timeout in seconds
    pub const MAX_REQUEST_TIMEOUT_SECS: u64 = 300; // 5 minutes
}

// ISO 639-1 language codes are now enforced by the Language enum
// See shared_types_generated::Language for supported languages (de, en)
