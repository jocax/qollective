//! Constants for NATS CLI
//!
//! Following CONSTANTS FIRST principle - all hardcoded values defined here

// ============================================================================
// CONFIGURATION CONSTANTS
// ============================================================================

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "config.toml";

/// Environment variable prefix for configuration overrides
pub const ENV_PREFIX: &str = "NATS_CLI";

// ============================================================================
// TEMPLATE CONSTANTS
// ============================================================================

/// Templates directory name (relative to CLI executable)
pub const TEMPLATES_DIR: &str = "templates";

/// Template file extension
pub const TEMPLATE_EXTENSION: &str = ".json";

/// Template README filename
pub const TEMPLATE_README: &str = "README.md";

// ============================================================================
// ENVELOPE CONSTANTS
// ============================================================================

/// Default envelope version
pub const DEFAULT_ENVELOPE_VERSION: &str = "1.0";

/// Default tenant ID when not specified
pub const DEFAULT_TENANT_ID: i32 = 1;

// ============================================================================
// TIMEOUT CONSTANTS
// ============================================================================

/// Default request timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Minimum allowed timeout in seconds
pub const MIN_TIMEOUT_SECS: u64 = 1;

/// Maximum allowed timeout in seconds
pub const MAX_TIMEOUT_SECS: u64 = 300;

// ============================================================================
// CLI OUTPUT CONSTANTS
// ============================================================================

/// Success indicator prefix
pub const SUCCESS_PREFIX: &str = "✅";

/// Error indicator prefix
pub const ERROR_PREFIX: &str = "❌";

/// Warning indicator prefix
pub const WARNING_PREFIX: &str = "⚠️";

/// Info indicator prefix
pub const INFO_PREFIX: &str = "ℹ️";

// ============================================================================
// SUBJECT CONSTANTS (Common NATS subjects)
// ============================================================================

/// Default MCP prompt helper subject
pub const SUBJECT_MCP_PROMPT_HELPER: &str = "mcp.prompt.helper";

/// MCP discovery subject pattern
pub const SUBJECT_MCP_DISCOVERY: &str = "mcp.discovery";

// ============================================================================
// LOGGING CONSTANTS
// ============================================================================

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

// ============================================================================
// ERROR MESSAGE TEMPLATES
// ============================================================================

/// Template not found error template
pub const ERR_TEMPLATE_NOT_FOUND: &str = "Template not found";

/// Invalid template format error template
pub const ERR_INVALID_TEMPLATE: &str = "Invalid template format";

/// Connection error template
pub const ERR_CONNECTION_FAILED: &str = "Failed to connect to NATS";

/// Request timeout error template
pub const ERR_REQUEST_TIMEOUT: &str = "Request timed out";

/// Configuration error template
pub const ERR_CONFIG_ERROR: &str = "Configuration error";
