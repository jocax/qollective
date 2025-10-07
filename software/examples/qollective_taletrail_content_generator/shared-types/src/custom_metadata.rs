//! TaleTrail custom metadata extensions for Qollective envelopes
//!
//! This module demonstrates how to extend Qollective's Meta structure with
//! application-specific metadata using the ExtensionsMeta pattern.

use crate::generated::enums::GenerationPhase;
use qollective::envelope::meta::ExtensionsMeta;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Custom metadata extensions for TaleTrail content generation
///
/// These fields are stored in `Meta.extensions` and provide TaleTrail-specific
/// context that augments Qollective's standard metadata sections.
///
/// # Example
/// ```
/// use shared_types::extensions::TaleTrailCustomMetadata;
/// use shared_types::generated::enums::GenerationPhase;
/// use uuid::Uuid;
///
/// let custom_meta = TaleTrailCustomMetadata {
///     generation_phase: Some(GenerationPhase::PromptGeneration),
///     batch_id: Some(Uuid::now_v7()),
///     correlation_id: Some(Uuid::now_v7()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaleTrailCustomMetadata {
    /// Current phase of the generation pipeline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_phase: Option<GenerationPhase>,

    /// Batch identifier for grouping related generation operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<Uuid>,

    /// Correlation ID for tracking request chains across services
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
}

impl TaleTrailCustomMetadata {
    /// Create a new custom metadata instance
    pub fn new() -> Self {
        Self {
            generation_phase: None,
            batch_id: None,
            correlation_id: None,
        }
    }

    /// Builder pattern: set generation phase
    pub fn with_phase(mut self, phase: GenerationPhase) -> Self {
        self.generation_phase = Some(phase);
        self
    }

    /// Builder pattern: set batch ID
    pub fn with_batch_id(mut self, batch_id: Uuid) -> Self {
        self.batch_id = Some(batch_id);
        self
    }

    /// Builder pattern: set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Convert to Qollective's ExtensionsMeta format
    ///
    /// This method serializes the TaleTrail custom metadata into the format
    /// expected by Qollective's Meta.extensions field.
    pub fn to_extensions_meta(&self) -> ExtensionsMeta {
        let mut sections = HashMap::new();

        if let Some(phase) = &self.generation_phase {
            sections.insert("generation_phase".to_string(), json!(phase));
        }

        if let Some(batch_id) = &self.batch_id {
            sections.insert("batch_id".to_string(), json!(batch_id));
        }

        if let Some(correlation_id) = &self.correlation_id {
            sections.insert("correlation_id".to_string(), json!(correlation_id));
        }

        ExtensionsMeta { sections }
    }

    /// Extract TaleTrail custom metadata from Qollective's ExtensionsMeta
    ///
    /// This method deserializes TaleTrail-specific metadata from the
    /// Meta.extensions field, returning None if parsing fails.
    pub fn from_extensions_meta(extensions: &ExtensionsMeta) -> Option<Self> {
        let generation_phase = extensions
            .sections
            .get("generation_phase")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let batch_id = extensions
            .sections
            .get("batch_id")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let correlation_id = extensions
            .sections
            .get("correlation_id")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // Return None if all fields are None (no TaleTrail metadata present)
        if generation_phase.is_none() && batch_id.is_none() && correlation_id.is_none() {
            return None;
        }

        Some(Self {
            generation_phase,
            batch_id,
            correlation_id,
        })
    }
}

impl Default for TaleTrailCustomMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_conversion() {
        let original = TaleTrailCustomMetadata {
            generation_phase: Some(GenerationPhase::Structure),
            batch_id: Some(Uuid::now_v7()),
            correlation_id: Some(Uuid::now_v7()),
        };

        let extensions = original.to_extensions_meta();
        let recovered = TaleTrailCustomMetadata::from_extensions_meta(&extensions).unwrap();

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_empty_metadata() {
        let empty = TaleTrailCustomMetadata::new();
        let extensions = empty.to_extensions_meta();

        assert!(extensions.sections.is_empty());

        let recovered = TaleTrailCustomMetadata::from_extensions_meta(&extensions);
        assert!(recovered.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let batch_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();

        let metadata = TaleTrailCustomMetadata::new()
            .with_phase(GenerationPhase::Validation)
            .with_batch_id(batch_id)
            .with_correlation_id(correlation_id);

        assert_eq!(metadata.generation_phase, Some(GenerationPhase::Validation));
        assert_eq!(metadata.batch_id, Some(batch_id));
        assert_eq!(metadata.correlation_id, Some(correlation_id));
    }
}
