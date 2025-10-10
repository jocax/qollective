//! Tests for pipeline state machine

use shared_types::*;
use orchestrator::pipeline::{PipelinePhase, PipelineState};

fn create_test_request(theme: &str, node_count: i64) -> GenerationRequest {
    GenerationRequest {
        theme: theme.to_string(),
        age_group: AgeGroup::_6To8,
        language: Language::En,
        node_count: Some(node_count),
        tenant_id: 1,
        educational_goals: Some(vec!["Learn about planets".to_string()]),
        vocabulary_level: Some(VocabularyLevel::Basic),
        required_elements: Some(vec!["Jupiter".to_string()]),
        tags: None,
        prompt_packages: None,
        author_id: None,
    }
}

#[test]
fn test_new_pipeline_state() {
    let request = create_test_request("Space Adventure", 12);
    let state = PipelineState::new(request.clone());

    assert_eq!(state.current_phase, PipelinePhase::PromptGeneration);
    assert_eq!(state.progress, 0.0);
    assert!(state.errors.is_empty());
    assert_eq!(state.request.theme, request.theme);
    assert!(state.dag.is_none());
    assert!(state.prompt_packages.is_empty());
}

#[test]
fn test_phase_transitions() {
    let request = create_test_request("Ocean Quest", 8);
    let mut state = PipelineState::new(request);

    // Test phase progression
    assert_eq!(state.current_phase, PipelinePhase::PromptGeneration);

    state.advance_phase().expect("Should advance to Structure");
    assert_eq!(state.current_phase, PipelinePhase::Structure);

    state.advance_phase().expect("Should advance to Generation");
    assert_eq!(state.current_phase, PipelinePhase::Generation);

    state.advance_phase().expect("Should advance to Validation");
    assert_eq!(state.current_phase, PipelinePhase::Validation);

    state.advance_phase().expect("Should advance to Assembly");
    assert_eq!(state.current_phase, PipelinePhase::Assembly);

    state.advance_phase().expect("Should advance to Complete");
    assert_eq!(state.current_phase, PipelinePhase::Complete);

    // Should not advance past Complete
    let result = state.advance_phase();
    assert!(result.is_err());
}

#[test]
fn test_progress_tracking() {
    let request = create_test_request("Test", 16);
    let mut state = PipelineState::new(request);

    assert_eq!(state.progress, 0.0);

    state.update_progress(25.0);
    assert_eq!(state.progress, 25.0);

    state.update_progress(50.0);
    assert_eq!(state.progress, 50.0);

    state.update_progress(100.0);
    assert_eq!(state.progress, 100.0);
}

#[test]
fn test_error_accumulation() {
    let request = create_test_request("Test", 16);
    let mut state = PipelineState::new(request);

    assert!(state.errors.is_empty());

    state.add_error(TaleTrailError::GenerationError("Test error 1".to_string()));
    assert_eq!(state.errors.len(), 1);

    state.add_error(TaleTrailError::ValidationError("Test error 2".to_string()));
    assert_eq!(state.errors.len(), 2);
}

#[test]
fn test_prompt_packages_storage() {
    let request = create_test_request("Test", 16);
    let mut state = PipelineState::new(request);

    assert!(state.prompt_packages.is_empty());

    // Add prompt packages
    let story_prompts = PromptPackage {
        system_prompt: "Story system".to_string(),
        user_prompt: "Story user".to_string(),
        llm_model: "test-model".to_string(),
        language: Language::En,
        llm_config: LLMConfig {
            temperature: 0.7,
            max_tokens: 2000,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: vec![],
        },
        prompt_metadata: PromptMetadata {
            service_target: MCPServiceType::StoryGenerator,
            theme_context: "Test".to_string(),
            age_group_context: AgeGroup::_6To8,
            language_context: Language::En,
            generation_method: PromptGenerationMethod::TemplateFallback,
            template_version: "1.0".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
        },
        fallback_used: false,
    };

    let validation_prompts = PromptPackage {
        system_prompt: "Validation system".to_string(),
        user_prompt: "Validation user".to_string(),
        llm_model: "test-model".to_string(),
        language: Language::En,
        llm_config: LLMConfig {
            temperature: 0.7,
            max_tokens: 2000,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: vec![],
        },
        prompt_metadata: PromptMetadata {
            service_target: MCPServiceType::QualityControl,
            theme_context: "Test".to_string(),
            age_group_context: AgeGroup::_6To8,
            language_context: Language::En,
            generation_method: PromptGenerationMethod::TemplateFallback,
            template_version: "1.0".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
        },
        fallback_used: false,
    };

    state.prompt_packages.insert(MCPServiceType::StoryGenerator, story_prompts.clone());
    state.prompt_packages.insert(MCPServiceType::QualityControl, validation_prompts.clone());

    assert_eq!(state.prompt_packages.len(), 2);
    assert_eq!(
        state.prompt_packages.get(&MCPServiceType::StoryGenerator).unwrap().system_prompt,
        "Story system"
    );
}
