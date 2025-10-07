use chrono::Utc;
use qollective::envelope::builder::Envelope;
use shared_types::custom_metadata::TaleTrailCustomMetadata;
use shared_types::generated::enums::{
    AgeGroup, GenerationPhase, GenerationStatus, Language, VocabularyLevel,
};
use shared_types::generated::internal_api::{GenerationMetadata, GenerationRequest, GenerationResponse};
use shared_types::helpers::{
    create_request_envelope, create_response_envelope,
    extract_custom_metadata, extract_tenant_id, extract_user_id,
};
use shared_types::payloads::TaleTrailPayload;
use uuid::Uuid;

#[test]
fn test_create_request_envelope() {
    let request = GenerationRequest {
        theme: "Space Adventure".to_string(),
        age_group: AgeGroup::SixToEight,
        language: Language::En,
        educational_goals: vec!["Learn about planets".to_string()],
        node_count: 16,
        vocabulary_level: VocabularyLevel::Basic,
        required_elements: vec![],
        tags: vec![],
        author_id: Some(42),
        tenant_id: 1,
        prompt_packages: None,
    };

    let envelope = create_request_envelope(request.clone(), 1, Some(42), None);

    assert!(envelope.is_ok());
    let envelope = envelope.unwrap();

    // Verify metadata
    assert_eq!(extract_tenant_id(&envelope).unwrap(), 1);
    assert_eq!(extract_user_id(&envelope).unwrap(), Some(42));
    assert!(envelope.meta.request_id.is_some());

    // Verify payload type
    assert!(envelope.payload.is_generation_request());
}

#[test]
fn test_create_response_envelope() {
    let request_id = Uuid::now_v7();

    let response = GenerationResponse {
        request_id,
        status: GenerationStatus::Completed,
        progress_percentage: 100,
        trail: None,
        trail_steps: None,
        generation_metadata: Some(GenerationMetadata {
            generated_at: Utc::now(),
            total_word_count: 500,
            ai_model_version: "gpt-4".to_string(),
            generation_duration_seconds: 30,
            validation_rounds: 2,
            orchestrator_version: "1.0.0".to_string(),
        }),
        prompt_generation_metadata: None,
        errors: vec![],
    };

    let custom_meta = TaleTrailCustomMetadata::new().with_correlation_id(request_id);

    let envelope = create_response_envelope(response.clone(), request_id, Some(custom_meta));

    assert!(envelope.is_ok());
    let envelope = envelope.unwrap();

    // Verify correlation via custom metadata
    let extracted_meta = extract_custom_metadata(&envelope).unwrap();
    assert_eq!(extracted_meta.correlation_id, Some(request_id));

    // Verify payload type
    assert!(envelope.payload.is_generation_response());
}

#[test]
fn test_custom_metadata_with_generation_phase() {
    let request = GenerationRequest {
        theme: "Ocean Exploration".to_string(),
        age_group: AgeGroup::NineToEleven,
        language: Language::De,
        educational_goals: vec![],
        node_count: 16,
        vocabulary_level: VocabularyLevel::Basic,
        required_elements: vec![],
        tags: vec![],
        author_id: None,
        tenant_id: 2,
        prompt_packages: None,
    };

    let custom_meta = TaleTrailCustomMetadata::new()
        .with_phase(GenerationPhase::PromptGeneration)
        .with_batch_id(Uuid::now_v7());

    let envelope = create_request_envelope(request, 2, None, Some(custom_meta)).unwrap();

    // Extract and verify custom metadata
    let extracted = extract_custom_metadata(&envelope).unwrap();
    assert_eq!(
        extracted.generation_phase,
        Some(GenerationPhase::PromptGeneration)
    );
    assert!(extracted.batch_id.is_some());
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
        tenant_id: 123,
        prompt_packages: None,
    };

    let envelope = create_request_envelope(request, 123, None, None).unwrap();

    assert_eq!(extract_tenant_id(&envelope).unwrap(), 123);
    assert!(extract_custom_metadata(&envelope).is_none());
}

#[test]
fn test_user_extraction() {
    let request = GenerationRequest {
        theme: "Test".to_string(),
        age_group: AgeGroup::SixToEight,
        language: Language::En,
        educational_goals: vec![],
        node_count: 8,
        vocabulary_level: VocabularyLevel::Basic,
        required_elements: vec![],
        tags: vec![],
        author_id: Some(456),
        tenant_id: 1,
        prompt_packages: None,
    };

    // With user
    let envelope_with_user = create_request_envelope(request.clone(), 1, Some(456), None).unwrap();
    assert_eq!(extract_user_id(&envelope_with_user).unwrap(), Some(456));

    // Without user
    let envelope_without_user = create_request_envelope(request, 1, None, None).unwrap();
    assert_eq!(extract_user_id(&envelope_without_user).unwrap(), None);
}

#[test]
fn test_full_custom_metadata_roundtrip() {
    let request = GenerationRequest {
        theme: "Custom Metadata Test".to_string(),
        age_group: AgeGroup::TwelveToFourteen,
        language: Language::En,
        educational_goals: vec![],
        node_count: 24,
        vocabulary_level: VocabularyLevel::Intermediate,
        required_elements: vec![],
        tags: vec![],
        author_id: None,
        tenant_id: 1,
        prompt_packages: None,
    };

    let batch_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();

    let custom_meta = TaleTrailCustomMetadata::new()
        .with_phase(GenerationPhase::Structure)
        .with_batch_id(batch_id)
        .with_correlation_id(correlation_id);

    let envelope = create_request_envelope(request, 1, None, Some(custom_meta)).unwrap();

    // Retrieve and verify custom metadata
    let extracted = extract_custom_metadata(&envelope).unwrap();
    assert_eq!(extracted.generation_phase, Some(GenerationPhase::Structure));
    assert_eq!(extracted.batch_id, Some(batch_id));
    assert_eq!(extracted.correlation_id, Some(correlation_id));
}

#[test]
fn test_payload_type_discrimination() {
    // Test GenerationRequest payload
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

    let request_envelope = create_request_envelope(request, 1, None, None).unwrap();
    assert!(request_envelope.payload.is_generation_request());
    assert!(!request_envelope.payload.is_generation_response());
    assert!(!request_envelope.payload.is_validation_result());

    // Test GenerationResponse payload
    let response = GenerationResponse {
        request_id: Uuid::now_v7(),
        status: GenerationStatus::Completed,
        progress_percentage: 100,
        trail: None,
        trail_steps: None,
        generation_metadata: None,
        prompt_generation_metadata: None,
        errors: vec![],
    };

    let response_envelope = create_response_envelope(response, Uuid::now_v7(), None).unwrap();
    assert!(!response_envelope.payload.is_generation_request());
    assert!(response_envelope.payload.is_generation_response());
    assert!(!response_envelope.payload.is_validation_result());
}

#[test]
fn test_envelope_cloning() {
    let request = GenerationRequest {
        theme: "Clone Test".to_string(),
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
    let cloned = envelope.clone();

    assert_eq!(
        extract_tenant_id(&envelope).unwrap(),
        extract_tenant_id(&cloned).unwrap()
    );
    assert_eq!(envelope.meta.request_id, cloned.meta.request_id);
}

#[test]
fn test_direct_envelope_usage() {
    // Demonstrate direct usage of Envelope<TaleTrailPayload>
    let request = GenerationRequest {
        theme: "Direct Usage Test".to_string(),
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

    let payload = TaleTrailPayload::GenerationRequest(request);
    let envelope: Envelope<TaleTrailPayload> = Envelope::new_minimal(payload);

    assert!(envelope.payload.is_generation_request());
    assert!(envelope.meta.request_id.is_some());
    assert!(envelope.meta.timestamp.is_some());
}

#[test]
fn test_serialization_basics() {
    let request = GenerationRequest {
        theme: "Serialization Test".to_string(),
        age_group: AgeGroup::SixToEight,
        language: Language::En,
        educational_goals: vec!["Test goal".to_string()],
        node_count: 8,
        vocabulary_level: VocabularyLevel::Basic,
        required_elements: vec!["Test element".to_string()],
        tags: vec!["test".to_string()],
        author_id: Some(999),
        tenant_id: 7,
        prompt_packages: None,
    };

    let custom_meta = TaleTrailCustomMetadata::new().with_phase(GenerationPhase::Generation);

    let envelope = create_request_envelope(request.clone(), 7, Some(999), Some(custom_meta)).unwrap();

    // Verify serialization works
    let json = serde_json::to_string(&envelope).expect("Failed to serialize");
    assert!(!json.is_empty());
    assert!(json.contains("Serialization Test"));

    // Verify we can access all key fields
    assert_eq!(extract_tenant_id(&envelope).unwrap(), 7);
    assert_eq!(extract_user_id(&envelope).unwrap(), Some(999));

    let extracted_meta = extract_custom_metadata(&envelope).unwrap();
    assert_eq!(extracted_meta.generation_phase, Some(GenerationPhase::Generation));
}
