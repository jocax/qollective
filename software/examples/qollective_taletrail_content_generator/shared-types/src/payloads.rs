//! TaleTrail payload types for Qollective envelope-first architecture
//!
//! This module defines the discriminated union of all possible payload types
//! used in TaleTrail content generation. These payloads are wrapped in
//! Qollective's standard Envelope<T> structure.

use crate::generated::internal_api::{GenerationRequest, GenerationResponse};
use crate::generated::validation::{ConstraintResult, QualityResult, ValidationResult};
use serde::{Deserialize, Serialize};

/// Discriminated union of all possible TaleTrail payload types
///
/// This enum represents all message types that can be transmitted through
/// the TaleTrail content generation system. Each variant is wrapped in
/// Qollective's Envelope<TaleTrailPayload> for transmission.
///
/// # Example
/// ```ignore
/// use qollective::envelope::Envelope;
/// use shared_types::TaleTrailPayload;
/// use shared_types::generated::internal_api::GenerationRequest;
///
/// // Create a payload (fields omitted for brevity)
/// let request = GenerationRequest { /* ... */ };
/// let payload = TaleTrailPayload::GenerationRequest(request);
///
/// // Wrap in Qollective envelope
/// let envelope: Envelope<TaleTrailPayload> = Envelope::new_minimal(payload);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TaleTrailPayload {
    GenerationRequest(GenerationRequest),
    GenerationResponse(GenerationResponse),
    ValidationResult(ValidationResult),
    ConstraintResult(ConstraintResult),
    QualityResult(QualityResult),
}

impl TaleTrailPayload {
    /// Check if payload is a GenerationRequest
    pub fn is_generation_request(&self) -> bool {
        matches!(self, TaleTrailPayload::GenerationRequest(_))
    }

    /// Check if payload is a GenerationResponse
    pub fn is_generation_response(&self) -> bool {
        matches!(self, TaleTrailPayload::GenerationResponse(_))
    }

    /// Check if payload is a ValidationResult
    pub fn is_validation_result(&self) -> bool {
        matches!(self, TaleTrailPayload::ValidationResult(_))
    }

    /// Check if payload is a ConstraintResult
    pub fn is_constraint_result(&self) -> bool {
        matches!(self, TaleTrailPayload::ConstraintResult(_))
    }

    /// Check if payload is a QualityResult
    pub fn is_quality_result(&self) -> bool {
        matches!(self, TaleTrailPayload::QualityResult(_))
    }
}
