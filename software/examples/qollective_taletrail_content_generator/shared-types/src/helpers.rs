//! Ergonomic helper functions for creating TaleTrail envelopes
//!
//! This module provides convenience functions for creating properly-configured
//! Qollective envelopes with TaleTrail payloads and custom metadata.

use crate::custom_metadata::TaleTrailCustomMetadata;
use crate::errors::{Result, TaleTrailError};
use crate::generated::internal_api::{GenerationRequest, GenerationResponse};
use crate::generated::validation::ValidationResult;
use crate::payloads::TaleTrailPayload;
use chrono::Utc;
use qollective::envelope::builder::Envelope;
use qollective::envelope::meta::Meta;
use uuid::Uuid;

/// Create an envelope for a GenerationRequest
///
/// # Arguments
/// * `request` - The generation request payload
/// * `tenant_id` - The tenant identifier
/// * `user_id` - Optional user identifier from JWT
/// * `custom_meta` - Optional TaleTrail-specific metadata
///
/// # Example
/// ```
/// use shared_types::helpers::create_request_envelope;
/// use shared_types::generated::internal_api::GenerationRequest;
/// use shared_types::generated::enums::{AgeGroup, Language, VocabularyLevel};
///
/// let request = GenerationRequest {
///     theme: "Space Adventure".to_string(),
///     age_group: AgeGroup::SixToEight,
///     language: Language::En,
///     educational_goals: vec![],
///     node_count: 8,
///     vocabulary_level: VocabularyLevel::Basic,
///     required_elements: vec![],
///     tags: vec![],
///     author_id: None,
///     tenant_id: 1,
///     prompt_packages: None,
/// };
///
/// let envelope = create_request_envelope(request, 1, None, None).unwrap();
/// ```
pub fn create_request_envelope(
    request: GenerationRequest,
    tenant_id: i32,
    user_id: Option<i32>,
    custom_meta: Option<TaleTrailCustomMetadata>,
) -> Result<Envelope<TaleTrailPayload>> {
    let security_meta = if user_id.is_some() || true {
        // Always include security meta for tenant context
        Some(qollective::envelope::meta::SecurityMeta {
            user_id: user_id.map(|id| id.to_string()),
            permissions: vec![],
            roles: vec![],
            ..Default::default()
        })
    } else {
        None
    };

    let extensions = custom_meta.map(|meta| meta.to_extensions_meta());

    let meta = Meta {
        timestamp: Some(Utc::now()),
        request_id: Some(Uuid::now_v7()),
        version: Some("1.0".to_string()),
        tenant: Some(tenant_id.to_string()),
        security: security_meta,
        extensions,
        ..Default::default()
    };

    Ok(Envelope {
        meta,
        payload: TaleTrailPayload::GenerationRequest(request),
        error: None,
    })
}

/// Create an envelope for a GenerationResponse
///
/// # Arguments
/// * `response` - The generation response payload
/// * `request_id` - The original request ID for correlation
/// * `custom_meta` - Optional TaleTrail-specific metadata
///
/// # Example
/// ```
/// use shared_types::helpers::create_response_envelope;
/// use shared_types::generated::internal_api::GenerationResponse;
/// use shared_types::generated::enums::GenerationStatus;
/// use uuid::Uuid;
///
/// let request_id = Uuid::now_v7();
/// let response = GenerationResponse {
///     request_id,
///     status: GenerationStatus::Completed,
///     progress_percentage: 100,
///     trail: None,
///     trail_steps: None,
///     generation_metadata: None,
///     prompt_generation_metadata: None,
///     errors: vec![],
/// };
///
/// let envelope = create_response_envelope(response, request_id, None).unwrap();
/// ```
pub fn create_response_envelope(
    response: GenerationResponse,
    request_id: Uuid,
    custom_meta: Option<TaleTrailCustomMetadata>,
) -> Result<Envelope<TaleTrailPayload>> {
    let extensions = custom_meta.map(|meta| meta.to_extensions_meta());

    let meta = Meta {
        timestamp: Some(Utc::now()),
        request_id: Some(request_id),
        version: Some("1.0".to_string()),
        extensions,
        ..Default::default()
    };

    Ok(Envelope {
        meta,
        payload: TaleTrailPayload::GenerationResponse(response),
        error: None,
    })
}

/// Create an envelope for a ValidationResult
///
/// # Arguments
/// * `result` - The validation result payload
/// * `custom_meta` - Optional TaleTrail-specific metadata
///
/// # Example
/// ```
/// use shared_types::helpers::create_validation_envelope;
/// use shared_types::generated::validation::ValidationResult;
/// use shared_types::generated::enums::CorrectionCapability;
///
/// let result = ValidationResult {
///     is_valid: true,
///     age_appropriate_score: 0.95,
///     safety_issues: vec![],
///     educational_value_score: 0.88,
///     corrections: vec![],
///     correction_capability: CorrectionCapability::NoFixPossible,
/// };
///
/// let envelope = create_validation_envelope(result, None).unwrap();
/// ```
pub fn create_validation_envelope(
    result: ValidationResult,
    custom_meta: Option<TaleTrailCustomMetadata>,
) -> Result<Envelope<TaleTrailPayload>> {
    let extensions = custom_meta.map(|meta| meta.to_extensions_meta());

    let meta = Meta {
        timestamp: Some(Utc::now()),
        request_id: Some(Uuid::now_v7()),
        version: Some("1.0".to_string()),
        extensions,
        ..Default::default()
    };

    Ok(Envelope {
        meta,
        payload: TaleTrailPayload::ValidationResult(result),
        error: None,
    })
}

/// Extract tenant ID from envelope metadata
///
/// # Example
/// ```
/// use shared_types::helpers::{create_request_envelope, extract_tenant_id};
/// use shared_types::generated::internal_api::GenerationRequest;
/// use shared_types::generated::enums::{AgeGroup, Language, VocabularyLevel};
///
/// let request = GenerationRequest {
///     theme: "Test".to_string(),
///     age_group: AgeGroup::SixToEight,
///     language: Language::En,
///     educational_goals: vec![],
///     node_count: 8,
///     vocabulary_level: VocabularyLevel::Basic,
///     required_elements: vec![],
///     tags: vec![],
///     author_id: None,
///     tenant_id: 123,
///     prompt_packages: None,
/// };
///
/// let envelope = create_request_envelope(request, 123, None, None).unwrap();
/// assert_eq!(extract_tenant_id(&envelope).unwrap(), 123);
/// ```
pub fn extract_tenant_id<T>(envelope: &Envelope<T>) -> Result<i32> {
    envelope
        .meta
        .tenant
        .as_ref()
        .and_then(|t| t.parse::<i32>().ok())
        .ok_or_else(|| {
            TaleTrailError::ValidationError("Missing or invalid tenant_id in envelope".to_string())
        })
}

/// Extract user ID from envelope security metadata
pub fn extract_user_id<T>(envelope: &Envelope<T>) -> Result<Option<i32>> {
    Ok(envelope
        .meta
        .security
        .as_ref()
        .and_then(|s| s.user_id.as_ref())
        .and_then(|u| u.parse::<i32>().ok()))
}

/// Extract TaleTrail custom metadata from envelope
///
/// Returns `None` if no custom metadata is present or if the metadata
/// doesn't contain TaleTrail-specific fields.
pub fn extract_custom_metadata<T>(envelope: &Envelope<T>) -> Option<TaleTrailCustomMetadata> {
    envelope
        .meta
        .extensions
        .as_ref()
        .and_then(TaleTrailCustomMetadata::from_extensions_meta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::enums::{AgeGroup, GenerationPhase, Language, VocabularyLevel};

    #[test]
    fn test_create_request_envelope() {
        let request = GenerationRequest {
            theme: "Test Theme".to_string(),
            age_group: AgeGroup::SixToEight,
            language: Language::En,
            educational_goals: vec![],
            node_count: 8,
            vocabulary_level: VocabularyLevel::Basic,
            required_elements: vec![],
            tags: vec![],
            author_id: None,
            tenant_id: 1,
            prompt_packages: None,
        };

        let custom_meta = TaleTrailCustomMetadata::new()
            .with_phase(GenerationPhase::PromptGeneration)
            .with_batch_id(Uuid::now_v7());

        let envelope = create_request_envelope(request, 1, Some(42), Some(custom_meta)).unwrap();

        assert_eq!(extract_tenant_id(&envelope).unwrap(), 1);
        assert_eq!(extract_user_id(&envelope).unwrap(), Some(42));

        let extracted_meta = extract_custom_metadata(&envelope).unwrap();
        assert_eq!(
            extracted_meta.generation_phase,
            Some(GenerationPhase::PromptGeneration)
        );
    }

    #[test]
    fn test_envelope_without_custom_metadata() {
        let request = GenerationRequest {
            theme: "Test".to_string(),
            age_group: AgeGroup::SixToEight,
            language: Language::En,
            educational_goals: vec![],
            node_count: 8,
            vocabulary_level: VocabularyLevel::Basic,
            required_elements: vec![],
            tags: vec![],
            author_id: None,
            tenant_id: 1,
            prompt_packages: None,
        };

        let envelope = create_request_envelope(request, 1, None, None).unwrap();

        assert!(extract_custom_metadata(&envelope).is_none());
    }
}
