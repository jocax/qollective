// ABOUTME: Metadata configuration structures and builders
// ABOUTME: Provides granular control over metadata section inclusion and properties

//! Metadata configuration structures and builders.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for metadata inclusion and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    pub security: Option<MetaSectionConfig>,
    pub debug: Option<MetaSectionConfig>,
    pub performance: Option<MetaSectionConfig>,
    pub monitoring: Option<MetaSectionConfig>,
    pub tracing: Option<MetaSectionConfig>,
    pub extensions: Option<HashMap<String, MetaSectionConfig>>,
}

/// Configuration for a specific metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSectionConfig {
    pub enabled: bool,
    pub properties: PropertyConfig,
}

/// Configuration for properties within a metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyConfig {
    All,
    None,
    Specific(HashMap<String, bool>),
}

impl MetaConfig {
    pub fn new() -> Self {
        Self {
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        }
    }
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self::new()
    }
}
