/// Constants for TaleTrail Desktop Viewer
///
/// Following CONSTANTS FIRST principle - all hardcoded values are defined here

// ============================================================================
// NETWORK CONFIGURATION
// ============================================================================

pub mod network {
    /// Default NATS server URL (relative to localhost development)
    pub const DEFAULT_NATS_URL: &str = "nats://localhost:5222";

    /// Default NATS connection timeout in milliseconds
    pub const DEFAULT_NATS_TIMEOUT_MS: u64 = 5000;

    /// Default NATS request timeout in milliseconds (3 minutes for content generation)
    pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 180000;
}

// ============================================================================
// PATH CONFIGURATION (RELATIVE PATHS ONLY)
// ============================================================================

pub mod paths {
    /// Relative path to TLS certificates directory from project root
    pub const CERTS_DIR: &str = "certs";

    /// CA certificate filename
    pub const CA_CERT_FILE: &str = "ca.pem";

    /// Relative path to NKeys directory from project root
    pub const NKEYS_DIR: &str = "nkeys";

    /// Desktop NKey filename
    pub const DESKTOP_NKEY_FILE: &str = "desktop.nk";

    // ========================================================================
    // Source Templates (Read-Only, Shipped with Application)
    // ========================================================================

    /// Source templates directory (read-only, shipped with app)
    /// Located at: desktop/src-tauri/templates/
    pub const SOURCE_TEMPLATES_DIR: &str = "desktop/src-tauri/templates";

    /// DEPRECATED: Use SOURCE_TEMPLATES_DIR instead
    /// Kept for backward compatibility
    pub const TEMPLATES_DIR: &str = "templates";

    // ========================================================================
    // Runtime Directory Structure (User Persistent Workspace)
    // ========================================================================

    /// Runtime templates subdirectory name (relative to root_directory)
    /// User workspace for editable templates
    pub const RUNTIME_TEMPLATES_SUBDIR: &str = "templates";

    /// Trails subdirectory name (relative to root_directory)
    /// User workspace for generated story trails
    pub const TRAILS_SUBDIR: &str = "trails";

    /// Execution subdirectory name (relative to root_directory)
    /// User workspace for request execution tracking
    pub const EXECUTION_SUBDIR: &str = "execution";

    // ========================================================================
    // Template Management
    // ========================================================================

    /// Suffix added to example templates when copying from source
    /// Original: "template.json" -> Runtime: "template_example.json"
    pub const TEMPLATE_EXAMPLE_SUFFIX: &str = "_example";

    /// Template file extension
    pub const TEMPLATE_FILE_EXTENSION: &str = ".json";

    /// DEPRECATED: Use TRAILS_SUBDIR instead
    /// Kept for backward compatibility
    pub const DEFAULT_TRAILS_DIR_NAME: &str = "trails";

    /// MCP server templates subdirectories
    pub const ORCHESTRATOR_TEMPLATES: &str = "orchestrator";
    pub const STORY_GENERATOR_TEMPLATES: &str = "story-generator";
    pub const QUALITY_CONTROL_TEMPLATES: &str = "quality-control";
    pub const CONSTRAINT_ENFORCER_TEMPLATES: &str = "constraint-enforcer";
    pub const PROMPT_HELPER_TEMPLATES: &str = "prompt-helper";
}

// ============================================================================
// MCP CONFIGURATION
// ============================================================================

pub mod mcp {
    /// Available MCP servers
    pub const AVAILABLE_SERVERS: &[&str] = &[
        "orchestrator",
        "story-generator",
        "quality-control",
        "constraint-enforcer",
        "prompt-helper",
    ];

    /// Default MCP request timeout in milliseconds
    pub const DEFAULT_TIMEOUT_MS: u64 = 180000;
}

// ============================================================================
// MONITORING CONFIGURATION
// ============================================================================

pub mod monitoring {
    /// Maximum number of events to keep in memory
    pub const MAX_EVENT_BUFFER_SIZE: usize = 1000;

    /// Request cleanup timeout in seconds (1 hour)
    pub const REQUEST_CLEANUP_TIMEOUT_SECS: u64 = 3600;

    /// Event types available for filtering
    pub const EVENT_TYPES: &[&str] = &[
        "Started",
        "Progress",
        "Completed",
        "Failed",
        "ToolExecution",
    ];
}

// ============================================================================
// NATS SUBJECTS
// ============================================================================

/// NATS subject patterns
pub mod nats {
    /// Base subject for generation events (subscribed by desktop)
    pub const GENERATION_EVENTS_BASE: &str = "taletrail.generation.events";

    /// Tenant-specific generation events pattern
    pub const GENERATION_EVENTS_TENANT_PATTERN: &str = "taletrail.generation.events.{tenant_id}";

    /// MCP Orchestrator request subject (desktop publishes to this)
    pub const ORCHESTRATOR_REQUEST_SUBJECT: &str = "mcp.orchestrator.request";

    /// MCP Server request subject patterns
    pub mod mcp_subjects {
        /// Orchestrator MCP server request subject
        pub const ORCHESTRATOR: &str = "mcp.orchestrator.request";

        /// Story generator MCP server request subject
        pub const STORY_GENERATOR: &str = "mcp.story-generator.request";

        /// Quality control MCP server request subject
        pub const QUALITY_CONTROL: &str = "mcp.quality-control.request";

        /// Constraint enforcer MCP server request subject
        pub const CONSTRAINT_ENFORCER: &str = "mcp.constraint-enforcer.request";

        /// Prompt helper MCP server request subject
        pub const PROMPT_HELPER: &str = "mcp.prompt-helper.request";
    }
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

    /// Envelope protocol version
    pub const ENVELOPE_VERSION: &str = "1.0.0";
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

/// MCP template management constants
pub mod templates {
    /// Base directory for MCP templates (relative to desktop app)
    pub const TEMPLATE_BASE_DIR: &str = "../nats-cli/templates";

    /// Template file extension
    pub const TEMPLATE_FILE_EXTENSION: &str = ".json";

    /// MCP server names (must match directory structure)
    pub mod servers {
        /// Orchestrator server directory name
        pub const ORCHESTRATOR: &str = "orchestrator";

        /// Story generator server directory name
        pub const STORY_GENERATOR: &str = "story-generator";

        /// Quality control server directory name
        pub const QUALITY_CONTROL: &str = "quality-control";

        /// Constraint enforcer server directory name
        pub const CONSTRAINT_ENFORCER: &str = "constraint-enforcer";

        /// Prompt helper server directory name
        pub const PROMPT_HELPER: &str = "prompt-helper";

        /// All valid server names
        pub const ALL_SERVERS: &[&str] = &[
            ORCHESTRATOR,
            STORY_GENERATOR,
            QUALITY_CONTROL,
            CONSTRAINT_ENFORCER,
            PROMPT_HELPER,
        ];
    }
}

/// MCP request history management constants
pub mod history {
    /// Tauri store key for MCP history
    pub const STORE_KEY: &str = "mcp_history";

    /// Tauri store file name
    pub const STORE_FILE: &str = "mcp-history.json";

    /// Default number of entries per page
    pub const DEFAULT_PAGE_SIZE: usize = 50;

    /// Maximum allowed page size
    pub const MAX_PAGE_SIZE: usize = 200;

    /// Minimum allowed page size
    pub const MIN_PAGE_SIZE: usize = 10;
}
