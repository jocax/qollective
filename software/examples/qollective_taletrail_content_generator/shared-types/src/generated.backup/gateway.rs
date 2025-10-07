//! Gateway mapping configuration and error types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for gateway mapping layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMappingConfig {
    /// Age-appropriate defaults and enrichment rules
    pub defaults: MappingDefaults,

    /// Validation rules for external API
    pub validation: MappingValidation,
}

/// Age-appropriate defaults per age group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDefaults {
    // Node counts per age group
    #[serde(default = "default_node_count_6_8")]
    pub node_count_6_8: usize,
    #[serde(default = "default_node_count_9_11")]
    pub node_count_9_11: usize,
    #[serde(default = "default_node_count_12_14")]
    pub node_count_12_14: usize,
    #[serde(default = "default_node_count_15_17")]
    pub node_count_15_17: usize,
    #[serde(default = "default_node_count_18_plus")]
    pub node_count_18_plus: usize,

    // Vocabulary levels per age group
    #[serde(default = "default_vocab_basic")]
    pub vocabulary_level_6_8: String,
    #[serde(default = "default_vocab_basic")]
    pub vocabulary_level_9_11: String,
    #[serde(default = "default_vocab_intermediate")]
    pub vocabulary_level_12_14: String,
    #[serde(default = "default_vocab_intermediate")]
    pub vocabulary_level_15_17: String,
    #[serde(default = "default_vocab_advanced")]
    pub vocabulary_level_18_plus: String,

    /// Educational goals per age group
    #[serde(default)]
    pub educational_goals: HashMap<String, Vec<String>>,

    /// Required elements per age group
    #[serde(default)]
    pub required_elements: HashMap<String, Vec<String>>,
}

fn default_node_count_6_8() -> usize {
    8
}
fn default_node_count_9_11() -> usize {
    12
}
fn default_node_count_12_14() -> usize {
    16
}
fn default_node_count_15_17() -> usize {
    20
}
fn default_node_count_18_plus() -> usize {
    24
}
fn default_vocab_basic() -> String {
    "basic".to_string()
}
fn default_vocab_intermediate() -> String {
    "intermediate".to_string()
}
fn default_vocab_advanced() -> String {
    "advanced".to_string()
}

/// Validation rules for external API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingValidation {
    /// Maximum theme length
    #[serde(default = "default_max_theme_length")]
    pub max_theme_length: usize,

    /// Minimum theme length
    #[serde(default = "default_min_theme_length")]
    pub min_theme_length: usize,

    /// Allowed languages
    #[serde(default = "default_allowed_languages")]
    pub allowed_languages: Vec<String>,

    /// Allowed age groups
    #[serde(default = "default_allowed_age_groups")]
    pub allowed_age_groups: Vec<String>,
}

fn default_max_theme_length() -> usize {
    200
}
fn default_min_theme_length() -> usize {
    5
}
fn default_allowed_languages() -> Vec<String> {
    vec!["de".to_string(), "en".to_string()]
}
fn default_allowed_age_groups() -> Vec<String> {
    vec![
        "6-8".to_string(),
        "9-11".to_string(),
        "12-14".to_string(),
        "15-17".to_string(),
        "+18".to_string(),
    ]
}

/// Gateway mapping error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingError {
    /// Error type
    pub error_type: MappingErrorType,

    /// Error message
    pub message: String,
}

/// Types of mapping errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MappingErrorType {
    ValidationError,
    ConfigurationError,
    InternalResponseError,
    FormatConversionError,
}
