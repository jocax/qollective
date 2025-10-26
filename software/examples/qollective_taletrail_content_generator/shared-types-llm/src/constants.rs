//! Constants for shared-types-llm
//!
//! This module defines all hardcoded values used throughout the LLM client library.
//! Following the CONSTANTS FIRST principle: no hardcoded values should exist in production code.

// ============================================================================
// Provider Default URLs
// ============================================================================

/// Default URL for Shimmy local LLM server
pub const SHIMMY_DEFAULT_URL: &str = "http://127.0.0.1:11435/v1";

/// Default URL for LM Studio local LLM server
pub const LMSTUDIO_DEFAULT_URL: &str = "http://127.0.0.1:1234/v1";

/// Default URL for OpenAI API
pub const OPENAI_DEFAULT_URL: &str = "https://api.openai.com/v1";

/// Default URL for Anthropic API
pub const ANTHROPIC_DEFAULT_URL: &str = "https://api.anthropic.com/v1";

/// Default URL for Google Vertex AI / Gemini API
pub const GOOGLE_DEFAULT_URL: &str = "https://generativelanguage.googleapis.com/v1";

// ============================================================================
// Default LLM Parameters
// ============================================================================

/// Default timeout for LLM requests in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Default maximum tokens for LLM completions
pub const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Default temperature for LLM completions (0.0 = deterministic, 1.0 = creative)
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

// ============================================================================
// Default Model Names
// ============================================================================

/// Default Qwen model (32B instruction-tuned, Q4 quantization)
pub const DEFAULT_MODEL_QWEN: &str = "qwen2.5-32b-instruct-q4_k_m";

/// Default Llama model (70B instruction-tuned, Q4 quantization)
pub const DEFAULT_MODEL_LLAMA: &str = "llama-3.3-70b-instruct-q4_k_m";

/// Default Magistral model (24B small model, Q8 quantization)
pub const DEFAULT_MODEL_MAGISTRAL: &str = "magistral-small-2509-q8_0";

// ============================================================================
// Configuration Keys
// ============================================================================

/// Configuration key for LLM section
pub const CONFIG_KEY_LLM: &str = "llm";

/// Configuration key for provider type
pub const CONFIG_KEY_TYPE: &str = "type";

/// Configuration key for provider URL
pub const CONFIG_KEY_URL: &str = "url";

/// Configuration key for default model
pub const CONFIG_KEY_DEFAULT_MODEL: &str = "default_model";

/// Configuration key for model fallback flag
pub const CONFIG_KEY_USE_FALLBACK: &str = "use_default_model_fallback";

/// Configuration key for language-to-model mappings
pub const CONFIG_KEY_MODELS: &str = "models";

/// Configuration key for tenant configurations
pub const CONFIG_KEY_TENANTS: &str = "tenants";

// ============================================================================
// Provider Type Strings
// ============================================================================

/// Provider type string for Shimmy
pub const PROVIDER_TYPE_SHIMMY: &str = "shimmy";

/// Provider type string for LM Studio
pub const PROVIDER_TYPE_LMSTUDIO: &str = "lmstudio";

/// Provider type string for OpenAI
pub const PROVIDER_TYPE_OPENAI: &str = "openai";

/// Provider type string for Anthropic
pub const PROVIDER_TYPE_ANTHROPIC: &str = "anthropic";

/// Provider type string for Google
pub const PROVIDER_TYPE_GOOGLE: &str = "google";

// ============================================================================
// System Prompt Style Strings
// ============================================================================

/// Native system prompt support (model handles system role natively)
pub const PROMPT_STYLE_NATIVE: &str = "native";

/// Prepend system prompt to user message
pub const PROMPT_STYLE_PREPEND: &str = "prepend";

/// ChatML format with special tokens
pub const PROMPT_STYLE_CHATML: &str = "chatml";

/// No system prompt support
pub const PROMPT_STYLE_NONE: &str = "none";

// ============================================================================
// Audit Log Messages
// ============================================================================

/// Audit message when runtime tenant config is used (premium feature)
pub const AUDIT_RUNTIME_TENANT_CONFIG: &str = "Runtime tenant LLM config used";

/// Audit message when static TOML tenant config is used
pub const AUDIT_STATIC_TENANT_CONFIG: &str = "Static tenant LLM config used";

/// Audit message when default TOML config is used
pub const AUDIT_DEFAULT_CONFIG: &str = "Default LLM config used";

/// Audit message when model fallback is triggered
pub const AUDIT_MODEL_FALLBACK: &str = "Model not found, using default model as fallback";

// ============================================================================
// Error Messages
// ============================================================================

/// Error message for empty configuration values
pub const ERROR_EMPTY_CONFIG_VALUE: &str = "Configuration value cannot be empty";

/// Error message for missing provider URL
pub const ERROR_MISSING_PROVIDER_URL: &str = "Provider URL is required";

/// Error message for missing model name
pub const ERROR_MISSING_MODEL_NAME: &str = "Model name is required";

/// Error message for missing API key
pub const ERROR_MISSING_API_KEY: &str = "API key is required for this provider";

/// Error message for invalid provider type
pub const ERROR_INVALID_PROVIDER_TYPE: &str = "Invalid provider type specified";

/// Error message for model unavailable
pub const ERROR_MODEL_UNAVAILABLE: &str = "Requested model is not available";

/// Error message for provider unreachable
pub const ERROR_PROVIDER_UNREACHABLE: &str = "Unable to reach LLM provider";

// ============================================================================
// Environment Variable Prefixes
// ============================================================================

/// Environment variable prefix for LLM configuration
pub const ENV_PREFIX_LLM: &str = "LLM_";

/// Environment variable name for API key
pub const ENV_VAR_API_KEY: &str = "API_KEY";

/// Environment variable name for OpenAI API key
pub const ENV_VAR_OPENAI_API_KEY: &str = "LLM_OPENAI_API_KEY";

/// Environment variable name for Anthropic API key
pub const ENV_VAR_ANTHROPIC_API_KEY: &str = "LLM_ANTHROPIC_API_KEY";

/// Environment variable name for Google API key
pub const ENV_VAR_GOOGLE_API_KEY: &str = "LLM_GOOGLE_API_KEY";

// ============================================================================
// Debug and Logging Constants
// ============================================================================

/// Default directory for dumping LLM raw responses
pub const DEFAULT_LLM_DUMP_DIRECTORY: &str = "/tmp/llm_responses";

/// Maximum length of LLM response preview in log messages (characters)
pub const LLM_RESPONSE_PREVIEW_LENGTH: usize = 500;

/// Filename prefix for LLM response dump files
pub const LLM_DUMP_FILENAME_PREFIX: &str = "llm_response";
