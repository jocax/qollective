//! Constants for TaleTrail Content Generator
//!
//! CONSTANTS FIRST PRINCIPLE:
//! All hardcoded values must be defined here before use in production code.
//! This ensures maintainable code, simplified deployment, and configuration consistency.

// ============================================================================
// NATS CONNECTION CONSTANTS
// ============================================================================

/// Default NATS server URL (TLS-enabled)
pub const NATS_DEFAULT_URL: &str = "nats://localhost:5222";

/// NATS TLS certificate paths
pub const NATS_TLS_CA_CERT_PATH: &str = "./certs/ca.pem";
pub const NATS_TLS_CLIENT_CERT_PATH: &str = "./certs/client-cert.pem";
pub const NATS_TLS_CLIENT_KEY_PATH: &str = "./certs/client-key.pem";

// ============================================================================
// MCP PROMPT-HELPER SUBJECTS
// ============================================================================

/// MCP subject for story prompt generation
pub const MCP_PROMPT_STORY: &str = "mcp.prompt.generate_story";

/// MCP subject for validation prompt generation
pub const MCP_PROMPT_VALIDATION: &str = "mcp.prompt.generate_validation";

/// MCP subject for constraint prompt generation
pub const MCP_PROMPT_CONSTRAINT: &str = "mcp.prompt.generate_constraint";

/// MCP subject for model-for-language lookup
pub const MCP_PROMPT_MODEL: &str = "mcp.prompt.get_model";

/// MCP subject for health checks (optional)
pub const MCP_PROMPT_HEALTH: &str = "mcp.prompt.health";

// ============================================================================
// NATS QUEUE GROUPS
// ============================================================================

/// Queue group for prompt-helper service (for load balancing)
pub const PROMPT_HELPER_GROUP: &str = "prompt-helper-group";

/// Queue group for story-generator service
pub const STORY_GENERATOR_GROUP: &str = "story-generator-group";

/// Queue group for quality-control service
pub const QUALITY_CONTROL_GROUP: &str = "quality-control-group";

/// Queue group for constraint-enforcer service
pub const CONSTRAINT_ENFORCER_GROUP: &str = "constraint-enforcer-group";

// ============================================================================
// MCP ORCHESTRATOR SUBJECTS
// ============================================================================

/// MCP orchestrator request subject
pub const MCP_ORCHESTRATOR_REQUEST: &str = "mcp.orchestrator.request";

/// MCP story structure generation subject
pub const MCP_STORY_GENERATE: &str = "mcp.story.generate";

/// MCP quality validation subject
pub const MCP_QUALITY_VALIDATE: &str = "mcp.quality.validate";

/// MCP constraint enforcement subject
pub const MCP_CONSTRAINT_ENFORCE: &str = "mcp.constraint.enforce";

/// MCP event publishing subject
pub const MCP_EVENTS: &str = "mcp.events";

// ============================================================================
// TIMEOUTS (in seconds)
// ============================================================================

/// Default timeout for generation requests
pub const GENERATION_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// Default timeout for validation requests
pub const VALIDATION_TIMEOUT_SECS: u64 = 60; // 1 minute

/// Maximum retry attempts
pub const RETRY_MAX_ATTEMPTS: u32 = 3;

// ============================================================================
// BATCH PROCESSING
// ============================================================================

/// Minimum batch size for node generation
pub const BATCH_SIZE_MIN: usize = 4;

/// Maximum batch size for node generation
pub const BATCH_SIZE_MAX: usize = 6;

/// Number of concurrent batches
pub const CONCURRENT_BATCHES: usize = 3;

// ============================================================================
// LM STUDIO DEFAULTS
// ============================================================================

/// Default LM Studio URL
pub const LM_STUDIO_DEFAULT_URL: &str = "http://127.0.0.1:1234";

/// Default LM Studio model name
pub const LM_STUDIO_MODEL_NAME: &str = "meta-llama-3.1-8b-instruct";

// ============================================================================
// DAG GENERATION DEFAULTS
// ============================================================================

/// Default number of nodes in DAG
pub const DEFAULT_NODE_COUNT: usize = 16;

/// Ratio for convergence point placement (0.0-1.0)
pub const CONVERGENCE_POINT_RATIO: f64 = 0.5;
