//! TaleTrail Content Generator Constants
//!
//! Following CONSTANTS FIRST principle - all hardcoded values defined here

// ============================================================================
// NATS Connection Configuration
// ============================================================================

/// Default NATS server URL (TLS-enabled)
pub const NATS_DEFAULT_URL: &str = "nats://localhost:5222";

/// NATS TLS CA certificate path
pub const NATS_TLS_CA_CERT_PATH: &str = "./certs/ca.pem";

/// NATS TLS client certificate path
pub const NATS_TLS_CLIENT_CERT_PATH: &str = "./certs/client-cert.pem";

/// NATS TLS client key path
pub const NATS_TLS_CLIENT_KEY_PATH: &str = "./certs/client-key.pem";

/// NATS monitoring URL
pub const NATS_MONITOR_URL: &str = "http://localhost:9222";

// ============================================================================
// NATS Subject Hierarchy
// ============================================================================

/// MCP orchestrator request subject
pub const MCP_ORCHESTRATOR_REQUEST: &str = "mcp.orchestrator.request";

/// MCP story generation subject base
pub const MCP_STORY_GENERATE: &str = "mcp.story.generate";

/// MCP story structure generation subject
pub const MCP_STORY_GENERATE_STRUCTURE: &str = "mcp.story.generate.structure";

/// MCP story nodes generation subject
pub const MCP_STORY_GENERATE_NODES: &str = "mcp.story.generate.nodes";

/// MCP story validation subject
pub const MCP_STORY_VALIDATE_PATHS: &str = "mcp.story.validate.paths";

/// MCP quality validation subject
pub const MCP_QUALITY_VALIDATE: &str = "mcp.quality.validate";

/// MCP quality batch validation subject
pub const MCP_QUALITY_VALIDATE_BATCH: &str = "mcp.quality.validate.batch";

/// MCP constraint enforcement subject
pub const MCP_CONSTRAINT_ENFORCE: &str = "mcp.constraint.enforce";

/// MCP constraint correction subject
pub const MCP_CONSTRAINT_CORRECT: &str = "mcp.constraint.correct";

/// MCP events stream base subject
pub const MCP_EVENTS: &str = "mcp.events";

/// MCP events structure created
pub const MCP_EVENTS_STRUCTURE_CREATED: &str = "mcp.events.structure.created";

/// MCP events batch started
pub const MCP_EVENTS_BATCH_STARTED: &str = "mcp.events.batch.started";

/// MCP events batch completed
pub const MCP_EVENTS_BATCH_COMPLETED: &str = "mcp.events.batch.completed";

/// MCP events validation started
pub const MCP_EVENTS_VALIDATION_STARTED: &str = "mcp.events.validation.started";

/// MCP events negotiation round
pub const MCP_EVENTS_NEGOTIATION_ROUND: &str = "mcp.events.negotiation.round";

/// MCP events generation complete
pub const MCP_EVENTS_COMPLETE: &str = "mcp.events.complete";

// ============================================================================
// NATS Queue Groups (for load balancing)
// ============================================================================

/// Story generator queue group
pub const STORY_GENERATOR_GROUP: &str = "story-generator";

/// Quality control queue group
pub const QUALITY_CONTROL_GROUP: &str = "quality-control";

/// Constraint enforcer queue group
pub const CONSTRAINT_ENFORCER_GROUP: &str = "constraint-enforcer";

/// Orchestrator queue group
pub const ORCHESTRATOR_GROUP: &str = "orchestrator";

// ============================================================================
// Timeout Configuration
// ============================================================================

/// Generation timeout in seconds
pub const GENERATION_TIMEOUT_SECS: u64 = 60;

/// Validation timeout in seconds
pub const VALIDATION_TIMEOUT_SECS: u64 = 10;

/// NATS connection timeout in seconds
pub const NATS_CONNECT_TIMEOUT_SECS: u64 = 10;

/// Request timeout in seconds
pub const REQUEST_TIMEOUT_SECS: u64 = 120;

// ============================================================================
// Retry Configuration
// ============================================================================

/// Maximum retry attempts
pub const RETRY_MAX_ATTEMPTS: u32 = 3;

/// Exponential backoff base delay in seconds
pub const RETRY_BASE_DELAY_SECS: u64 = 1;

/// Maximum retry delay in seconds
pub const RETRY_MAX_DELAY_SECS: u64 = 30;

// ============================================================================
// Batch Processing Configuration
// ============================================================================

/// Minimum batch size for node generation
pub const BATCH_SIZE_MIN: usize = 4;

/// Maximum batch size for node generation
pub const BATCH_SIZE_MAX: usize = 6;

/// Number of concurrent batches
pub const CONCURRENT_BATCHES: usize = 3;

/// Maximum concurrent batches (upper limit)
pub const CONCURRENT_BATCHES_MAX: usize = 5;

// ============================================================================
// LM Studio Configuration
// ============================================================================

/// Default LM Studio URL
pub const LM_STUDIO_DEFAULT_URL: &str = "http://127.0.0.1:1234";

/// Default LM Studio model name
pub const LM_STUDIO_MODEL_NAME: &str = "local-model";

/// Maximum tokens per story generation
pub const MAX_TOKENS_PER_STORY: u32 = 50_000;

/// Maximum tokens per node
pub const MAX_TOKENS_PER_NODE: u32 = 600;

/// Target word count per node
pub const TARGET_WORDS_PER_NODE: usize = 400;

// ============================================================================
// DAG Structure Configuration
// ============================================================================

/// Default node count for generated stories
pub const DEFAULT_NODE_COUNT: usize = 16;

/// Convergence point ratio (fraction of nodes that are convergence points)
pub const CONVERGENCE_POINT_RATIO: f32 = 0.25;

/// Number of choices per node
pub const CHOICES_PER_NODE: usize = 3;

/// Maximum depth of story DAG
pub const MAX_DAG_DEPTH: usize = 10;

// ============================================================================
// HTTP Gateway Configuration
// ============================================================================

/// Default HTTPS gateway port (TLS-enabled)
pub const GATEWAY_DEFAULT_PORT: u16 = 8443;

/// API version prefix
pub const API_VERSION_PREFIX: &str = "/api/v1";

/// Health check endpoint
pub const HEALTH_ENDPOINT: &str = "/health";

/// Metrics endpoint
pub const METRICS_ENDPOINT: &str = "/metrics";

// ============================================================================
// Validation Configuration
// ============================================================================

/// Minimum content quality score (0.0 - 1.0)
pub const MIN_QUALITY_SCORE: f32 = 0.7;

/// Maximum negotiation rounds
pub const MAX_NEGOTIATION_ROUNDS: u32 = 3;

// ============================================================================
// Rate Limiting
// ============================================================================

/// Generation requests per minute
pub const RATE_LIMIT_GENERATION: u32 = 10;

/// Status requests per minute
pub const RATE_LIMIT_STATUS: u32 = 60;
