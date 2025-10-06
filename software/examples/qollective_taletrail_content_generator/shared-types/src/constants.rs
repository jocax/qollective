//! TaleTrail Content Generator Constants
//!
//! Following CONSTANTS FIRST principle with lazy_static pattern for runtime configuration

use lazy_static::lazy_static;
use dotenvy::dotenv;
use std::env as std_env;

// ============================================================================
// Environment Variable Names
// ============================================================================

/// Environment variable names for runtime configuration
pub mod env {
    pub const NATS_URL: &str = "NATS_URL";
    pub const NATS_TLS_CA_CERT: &str = "NATS_TLS_CA_CERT";
    pub const NATS_TLS_CLIENT_CERT: &str = "NATS_TLS_CLIENT_CERT";
    pub const NATS_TLS_CLIENT_KEY: &str = "NATS_TLS_CLIENT_KEY";
    pub const NATS_MONITOR_URL: &str = "NATS_MONITOR_URL";

    pub const LM_STUDIO_URL: &str = "LM_STUDIO_URL";
    pub const LM_STUDIO_MODEL_NAME: &str = "LM_STUDIO_MODEL_NAME";

    pub const GENERATION_TIMEOUT_SECS: &str = "GENERATION_TIMEOUT_SECS";
    pub const VALIDATION_TIMEOUT_SECS: &str = "VALIDATION_TIMEOUT_SECS";
    pub const NATS_CONNECT_TIMEOUT_SECS: &str = "NATS_CONNECT_TIMEOUT_SECS";
    pub const REQUEST_TIMEOUT_SECS: &str = "REQUEST_TIMEOUT_SECS";

    pub const RETRY_MAX_ATTEMPTS: &str = "RETRY_MAX_ATTEMPTS";
    pub const RETRY_BASE_DELAY_SECS: &str = "RETRY_BASE_DELAY_SECS";
    pub const RETRY_MAX_DELAY_SECS: &str = "RETRY_MAX_DELAY_SECS";

    pub const BATCH_SIZE_MIN: &str = "BATCH_SIZE_MIN";
    pub const BATCH_SIZE_MAX: &str = "BATCH_SIZE_MAX";
    pub const CONCURRENT_BATCHES: &str = "CONCURRENT_BATCHES";
    pub const CONCURRENT_BATCHES_MAX: &str = "CONCURRENT_BATCHES_MAX";

    pub const MAX_TOKENS_PER_STORY: &str = "MAX_TOKENS_PER_STORY";
    pub const MAX_TOKENS_PER_NODE: &str = "MAX_TOKENS_PER_NODE";
    pub const TARGET_WORDS_PER_NODE: &str = "TARGET_WORDS_PER_NODE";

    pub const DEFAULT_NODE_COUNT: &str = "DEFAULT_NODE_COUNT";
    pub const CHOICES_PER_NODE: &str = "CHOICES_PER_NODE";
    pub const MAX_DAG_DEPTH: &str = "MAX_DAG_DEPTH";

    pub const GATEWAY_PORT: &str = "GATEWAY_PORT";
    pub const MIN_QUALITY_SCORE: &str = "MIN_QUALITY_SCORE";
    pub const MAX_NEGOTIATION_ROUNDS: &str = "MAX_NEGOTIATION_ROUNDS";

    pub const RATE_LIMIT_GENERATION: &str = "RATE_LIMIT_GENERATION";
    pub const RATE_LIMIT_STATUS: &str = "RATE_LIMIT_STATUS";
}

// ============================================================================
// Runtime Configuration (Lazy Static)
// ============================================================================

lazy_static! {
    // NATS Connection Configuration
    pub static ref NATS_URL: String = get_nats_url();
    pub static ref NATS_TLS_CA_CERT_PATH: String = get_nats_tls_ca_cert();
    pub static ref NATS_TLS_CLIENT_CERT_PATH: String = get_nats_tls_client_cert();
    pub static ref NATS_TLS_CLIENT_KEY_PATH: String = get_nats_tls_client_key();
    pub static ref NATS_MONITOR_URL: String = get_nats_monitor_url();

    // LM Studio Configuration
    pub static ref LM_STUDIO_URL: String = get_lm_studio_url();
    pub static ref LM_STUDIO_MODEL_NAME: String = get_lm_studio_model_name();

    // Timeout Configuration
    pub static ref GENERATION_TIMEOUT_SECS: u64 = get_generation_timeout();
    pub static ref VALIDATION_TIMEOUT_SECS: u64 = get_validation_timeout();
    pub static ref NATS_CONNECT_TIMEOUT_SECS: u64 = get_nats_connect_timeout();
    pub static ref REQUEST_TIMEOUT_SECS: u64 = get_request_timeout();

    // Retry Configuration
    pub static ref RETRY_MAX_ATTEMPTS: u32 = get_retry_max_attempts();
    pub static ref RETRY_BASE_DELAY_SECS: u64 = get_retry_base_delay();
    pub static ref RETRY_MAX_DELAY_SECS: u64 = get_retry_max_delay();

    // Batch Processing Configuration
    pub static ref BATCH_SIZE_MIN: usize = get_batch_size_min();
    pub static ref BATCH_SIZE_MAX: usize = get_batch_size_max();
    pub static ref CONCURRENT_BATCHES: usize = get_concurrent_batches();
    pub static ref CONCURRENT_BATCHES_MAX: usize = get_concurrent_batches_max();

    // Token Limits
    pub static ref MAX_TOKENS_PER_STORY: u32 = get_max_tokens_per_story();
    pub static ref MAX_TOKENS_PER_NODE: u32 = get_max_tokens_per_node();
    pub static ref TARGET_WORDS_PER_NODE: usize = get_target_words_per_node();

    // DAG Structure Configuration
    pub static ref DEFAULT_NODE_COUNT: usize = get_default_node_count();
    pub static ref CHOICES_PER_NODE: usize = get_choices_per_node();
    pub static ref MAX_DAG_DEPTH: usize = get_max_dag_depth();

    // HTTP Gateway Configuration
    pub static ref GATEWAY_PORT: u16 = get_gateway_port();

    // Validation Configuration
    pub static ref MIN_QUALITY_SCORE: f32 = get_min_quality_score();
    pub static ref MAX_NEGOTIATION_ROUNDS: u32 = get_max_negotiation_rounds();

    // Rate Limiting
    pub static ref RATE_LIMIT_GENERATION: u32 = get_rate_limit_generation();
    pub static ref RATE_LIMIT_STATUS: u32 = get_rate_limit_status();
}

// ============================================================================
// Getter Functions with Defaults
// ============================================================================

fn get_nats_url() -> String {
    dotenv().ok();
    std_env::var(env::NATS_URL)
        .unwrap_or_else(|_| "nats://localhost:5222".to_string())
}

fn get_nats_tls_ca_cert() -> String {
    dotenv().ok();
    std_env::var(env::NATS_TLS_CA_CERT)
        .unwrap_or_else(|_| "./certs/ca.pem".to_string())
}

fn get_nats_tls_client_cert() -> String {
    dotenv().ok();
    std_env::var(env::NATS_TLS_CLIENT_CERT)
        .unwrap_or_else(|_| "./certs/client-cert.pem".to_string())
}

fn get_nats_tls_client_key() -> String {
    dotenv().ok();
    std_env::var(env::NATS_TLS_CLIENT_KEY)
        .unwrap_or_else(|_| "./certs/client-key.pem".to_string())
}

fn get_nats_monitor_url() -> String {
    dotenv().ok();
    std_env::var(env::NATS_MONITOR_URL)
        .unwrap_or_else(|_| "http://localhost:9222".to_string())
}

fn get_lm_studio_url() -> String {
    dotenv().ok();
    std_env::var(env::LM_STUDIO_URL)
        .unwrap_or_else(|_| "http://127.0.0.1:1234".to_string())
}

fn get_lm_studio_model_name() -> String {
    dotenv().ok();
    std_env::var(env::LM_STUDIO_MODEL_NAME)
        .unwrap_or_else(|_| "local-model".to_string())
}

fn get_generation_timeout() -> u64 {
    dotenv().ok();
    std_env::var(env::GENERATION_TIMEOUT_SECS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60)
}

fn get_validation_timeout() -> u64 {
    dotenv().ok();
    std_env::var(env::VALIDATION_TIMEOUT_SECS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn get_nats_connect_timeout() -> u64 {
    dotenv().ok();
    std_env::var(env::NATS_CONNECT_TIMEOUT_SECS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn get_request_timeout() -> u64 {
    dotenv().ok();
    std_env::var(env::REQUEST_TIMEOUT_SECS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120)
}

fn get_retry_max_attempts() -> u32 {
    dotenv().ok();
    std_env::var(env::RETRY_MAX_ATTEMPTS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn get_retry_base_delay() -> u64 {
    dotenv().ok();
    std_env::var(env::RETRY_BASE_DELAY_SECS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

fn get_retry_max_delay() -> u64 {
    dotenv().ok();
    std_env::var(env::RETRY_MAX_DELAY_SECS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
}

fn get_batch_size_min() -> usize {
    dotenv().ok();
    std_env::var(env::BATCH_SIZE_MIN)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4)
}

fn get_batch_size_max() -> usize {
    dotenv().ok();
    std_env::var(env::BATCH_SIZE_MAX)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6)
}

fn get_concurrent_batches() -> usize {
    dotenv().ok();
    std_env::var(env::CONCURRENT_BATCHES)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn get_concurrent_batches_max() -> usize {
    dotenv().ok();
    std_env::var(env::CONCURRENT_BATCHES_MAX)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

fn get_max_tokens_per_story() -> u32 {
    dotenv().ok();
    std_env::var(env::MAX_TOKENS_PER_STORY)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50_000)
}

fn get_max_tokens_per_node() -> u32 {
    dotenv().ok();
    std_env::var(env::MAX_TOKENS_PER_NODE)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600)
}

fn get_target_words_per_node() -> usize {
    dotenv().ok();
    std_env::var(env::TARGET_WORDS_PER_NODE)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400)
}

fn get_default_node_count() -> usize {
    dotenv().ok();
    std_env::var(env::DEFAULT_NODE_COUNT)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16)
}

fn get_choices_per_node() -> usize {
    dotenv().ok();
    std_env::var(env::CHOICES_PER_NODE)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn get_max_dag_depth() -> usize {
    dotenv().ok();
    std_env::var(env::MAX_DAG_DEPTH)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn get_gateway_port() -> u16 {
    dotenv().ok();
    std_env::var(env::GATEWAY_PORT)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8443)
}

fn get_min_quality_score() -> f32 {
    dotenv().ok();
    std_env::var(env::MIN_QUALITY_SCORE)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.7)
}

fn get_max_negotiation_rounds() -> u32 {
    dotenv().ok();
    std_env::var(env::MAX_NEGOTIATION_ROUNDS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn get_rate_limit_generation() -> u32 {
    dotenv().ok();
    std_env::var(env::RATE_LIMIT_GENERATION)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn get_rate_limit_status() -> u32 {
    dotenv().ok();
    std_env::var(env::RATE_LIMIT_STATUS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60)
}

// ============================================================================
// Truly Constant Values (Static Identifiers and Fixed Ratios)
// ============================================================================

// NATS Subject Hierarchy (static protocol identifiers)
pub const MCP_ORCHESTRATOR_REQUEST: &str = "mcp.orchestrator.request";
pub const MCP_STORY_GENERATE: &str = "mcp.story.generate";
pub const MCP_STORY_GENERATE_STRUCTURE: &str = "mcp.story.generate.structure";
pub const MCP_STORY_GENERATE_NODES: &str = "mcp.story.generate.nodes";
pub const MCP_STORY_VALIDATE_PATHS: &str = "mcp.story.validate.paths";
pub const MCP_QUALITY_VALIDATE: &str = "mcp.quality.validate";
pub const MCP_QUALITY_VALIDATE_BATCH: &str = "mcp.quality.validate.batch";
pub const MCP_CONSTRAINT_ENFORCE: &str = "mcp.constraint.enforce";
pub const MCP_CONSTRAINT_CORRECT: &str = "mcp.constraint.correct";
pub const MCP_EVENTS: &str = "mcp.events";
pub const MCP_EVENTS_STRUCTURE_CREATED: &str = "mcp.events.structure.created";
pub const MCP_EVENTS_BATCH_STARTED: &str = "mcp.events.batch.started";
pub const MCP_EVENTS_BATCH_COMPLETED: &str = "mcp.events.batch.completed";
pub const MCP_EVENTS_VALIDATION_STARTED: &str = "mcp.events.validation.started";
pub const MCP_EVENTS_NEGOTIATION_ROUND: &str = "mcp.events.negotiation.round";
pub const MCP_EVENTS_COMPLETE: &str = "mcp.events.complete";

// NATS Queue Groups (static load balancing identifiers)
pub const STORY_GENERATOR_GROUP: &str = "story-generator";
pub const QUALITY_CONTROL_GROUP: &str = "quality-control";
pub const CONSTRAINT_ENFORCER_GROUP: &str = "constraint-enforcer";
pub const ORCHESTRATOR_GROUP: &str = "orchestrator";

// HTTP API Constants (static path identifiers)
pub const API_VERSION_PREFIX: &str = "/api/v1";
pub const HEALTH_ENDPOINT: &str = "/health";
pub const METRICS_ENDPOINT: &str = "/metrics";

// Fixed Ratios and Percentages
pub const CONVERGENCE_POINT_RATIO: f32 = 0.25;
