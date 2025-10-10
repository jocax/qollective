//! Comprehensive tests for Prompt-Helper MCP tool handlers
//!
//! Tests follow TDD principles and cover:
//! - Valid parameter extraction and successful responses
//! - LLM generation success paths with MockLlmService
//! - Template fallback when LLM fails
//! - Error handling with is_error=true responses
//! - Edge cases for all enum variants (AgeGroup, Language, VocabularyLevel)

use prompt_helper::{
    config::PromptHelperConfig,
    tool_handlers::{
        handle_generate_constraint_prompts, handle_generate_story_prompts,
        handle_generate_validation_prompts, handle_get_model_for_language,
    },
    mcp_tools::{
        GenerateConstraintPromptsParams, GenerateStoryPromptsParams,
        GenerateValidationPromptsParams, GetModelForLanguageParams,
    },
};
use rmcp::model::{CallToolRequest, CallToolRequestParam};
use shared_types::{
    traits::MockLlmService, AgeGroup, Language, PromptPackage, TaleTrailError, VocabularyLevel,
};

// ============================================================================
// Test Utilities
// ============================================================================

/// Create test LLM configuration
fn test_llm_config() -> shared_types_llm::LlmConfig {
    let toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:11434/v1"
default_model = "test-model"
    "#;
    shared_types_llm::LlmConfig::from_toml_str(toml).unwrap()
}

/// Create test configuration with defaults
fn create_test_config() -> PromptHelperConfig {
    use prompt_helper::config::{ServiceConfig, NatsConfig, PromptConfig};
    PromptHelperConfig {
        service: ServiceConfig::default(),
        nats: NatsConfig::default(),
        llm: test_llm_config(),
        prompt: PromptConfig::default(),
    }
}

/// Create CallToolRequest from parameters
fn create_request<T: serde::Serialize>(tool_name: &str, params: T) -> CallToolRequest {
    use rmcp::model::{CallToolRequestMethod, Extensions};

    let value = serde_json::to_value(params).expect("Failed to serialize params");
    let arguments = if let serde_json::Value::Object(map) = value {
        Some(map)
    } else {
        panic!("Expected object for params");
    };
    CallToolRequest {
        method: CallToolRequestMethod,
        params: CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments,
        },
        extensions: Extensions::default(),
    }
}

// ============================================================================
// Tests: handle_generate_story_prompts
// ============================================================================

#[tokio::test]
async fn test_generate_story_prompts_valid_request() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    // Mock LLM success
    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Ok(("System prompt from LLM".into(), "User prompt from LLM".into())));

    let params = GenerateStoryPromptsParams {
        theme: "Space Adventure".to_string(),
        age_group: AgeGroup::_9To11,
        language: Language::En,
        educational_goals: vec!["Learn about planets".to_string()],
    };

    let request = create_request("generate_story_prompts", params);
    let result = handle_generate_story_prompts(request, &mock_llm, &config).await;

    // Verify success
    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);

    // Verify PromptPackage deserialization
    let json = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    let package: PromptPackage = serde_json::from_str(json).expect("Failed to parse PromptPackage");

    assert_eq!(package.fallback_used, false);
    assert_eq!(package.system_prompt, "System prompt from LLM");
    assert_eq!(package.user_prompt, "User prompt from LLM");
    assert_eq!(package.language, Language::En);
}

#[tokio::test]
async fn test_generate_story_prompts_llm_failure_fallback() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    // Mock LLM failure
    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Err(TaleTrailError::LLMError("Model not available".into())));

    let params = GenerateStoryPromptsParams {
        theme: "Medieval Quest".to_string(),
        age_group: AgeGroup::_12To14,
        language: Language::De,
        educational_goals: vec!["History learning".to_string()],
    };

    let request = create_request("generate_story_prompts", params);
    let result = handle_generate_story_prompts(request, &mock_llm, &config).await;

    // Verify success with fallback
    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);

    let json = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    let package: PromptPackage = serde_json::from_str(json).expect("Failed to parse PromptPackage");

    assert_eq!(package.fallback_used, true);
    assert!(package.system_prompt.contains("Medieval Quest"));
    assert!(package.system_prompt.contains("De") || package.system_prompt.contains("German"));
}

#[tokio::test]
async fn test_generate_story_prompts_invalid_parameters() {
    let config = create_test_config();
    let mock_llm = MockLlmService::new();

    // Invalid JSON structure (missing required fields)
    let invalid_json = serde_json::json!({
        "theme": "Space",
        // Missing age_group, language, educational_goals
    });

    let arguments = if let serde_json::Value::Object(map) = invalid_json {
        Some(map)
    } else {
        None
    };

    let request = CallToolRequest {
        method: rmcp::model::CallToolRequestMethod,
        params: CallToolRequestParam {
            name: "generate_story_prompts".into(),
            arguments,
        },
        extensions: rmcp::model::Extensions::default(),
    };

    let result = handle_generate_story_prompts(request, &mock_llm, &config).await;

    // Verify error response
    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content.len(), 1);

    let error_msg = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    assert!(error_msg.contains("Invalid parameters"));
}

#[tokio::test]
async fn test_generate_story_prompts_all_age_groups() {
    let config = create_test_config();
    let age_groups = vec![
        AgeGroup::_6To8,
        AgeGroup::_9To11,
        AgeGroup::_12To14,
        AgeGroup::_15To17,
        AgeGroup::Plus18,
    ];

    for age_group in age_groups {
        let mut mock_llm = MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| Ok(("System".into(), "User".into())));

        let params = GenerateStoryPromptsParams {
            theme: "Test".to_string(),
            age_group: age_group.clone(),
            language: Language::En,
            educational_goals: vec!["Test goal".to_string()],
        };

        let request = create_request("generate_story_prompts", params);
        let result = handle_generate_story_prompts(request, &mock_llm, &config).await;

        assert_eq!(result.is_error, None, "Failed for age group {:?}", age_group);
    }
}

// ============================================================================
// Tests: handle_generate_validation_prompts
// ============================================================================

#[tokio::test]
async fn test_generate_validation_prompts_valid_request() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Ok(("Validation system prompt".into(), "Validation user prompt".into())));

    let params = GenerateValidationPromptsParams {
        age_group: AgeGroup::_6To8,
        language: Language::En,
        content_type: "story".to_string(),
    };

    let request = create_request("generate_validation_prompts", params);
    let result = handle_generate_validation_prompts(request, &mock_llm, &config).await;

    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);

    let json = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    let package: PromptPackage = serde_json::from_str(json).expect("Failed to parse PromptPackage");

    assert_eq!(package.fallback_used, false);
    assert_eq!(package.system_prompt, "Validation system prompt");
}

#[tokio::test]
async fn test_generate_validation_prompts_llm_failure() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Err(TaleTrailError::TimeoutError));

    let params = GenerateValidationPromptsParams {
        age_group: AgeGroup::_15To17,
        language: Language::De,
        content_type: "choice".to_string(),
    };

    let request = create_request("generate_validation_prompts", params);
    let result = handle_generate_validation_prompts(request, &mock_llm, &config).await;

    assert_eq!(result.is_error, None);

    let json = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    let package: PromptPackage = serde_json::from_str(json).expect("Failed to parse PromptPackage");

    assert_eq!(package.fallback_used, true);
    assert!(package.system_prompt.contains("quality control"));
    assert!(package.system_prompt.contains("choice"));
}

#[tokio::test]
async fn test_generate_validation_prompts_different_content_types() {
    let config = create_test_config();
    let content_types = vec!["story", "choice", "educational_fact"];

    for content_type in content_types {
        let mut mock_llm = MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| Ok(("System".into(), "User".into())));

        let params = GenerateValidationPromptsParams {
            age_group: AgeGroup::_9To11,
            language: Language::En,
            content_type: content_type.to_string(),
        };

        let request = create_request("generate_validation_prompts", params);
        let result = handle_generate_validation_prompts(request, &mock_llm, &config).await;

        assert_eq!(result.is_error, None, "Failed for content type {}", content_type);
    }
}

// ============================================================================
// Tests: handle_generate_constraint_prompts
// ============================================================================

#[tokio::test]
async fn test_generate_constraint_prompts_valid_request() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Ok(("Constraint system prompt".into(), "Constraint user prompt".into())));

    let params = GenerateConstraintPromptsParams {
        vocabulary_level: VocabularyLevel::Basic,
        language: Language::En,
        required_elements: vec!["hero".to_string(), "challenge".to_string()],
    };

    let request = create_request("generate_constraint_prompts", params);
    let result = handle_generate_constraint_prompts(request, &mock_llm, &config).await;

    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);

    let json = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    let package: PromptPackage = serde_json::from_str(json).expect("Failed to parse PromptPackage");

    assert_eq!(package.fallback_used, false);
    assert_eq!(package.system_prompt, "Constraint system prompt");
}

#[tokio::test]
async fn test_generate_constraint_prompts_all_vocabulary_levels() {
    let config = create_test_config();
    let vocab_levels = vec![
        VocabularyLevel::Basic,
        VocabularyLevel::Intermediate,
        VocabularyLevel::Advanced,
    ];

    for vocab_level in vocab_levels {
        let mut mock_llm = MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| Ok(("System".into(), "User".into())));

        let params = GenerateConstraintPromptsParams {
            vocabulary_level: vocab_level.clone(),
            language: Language::En,
            required_elements: vec!["element1".to_string()],
        };

        let request = create_request("generate_constraint_prompts", params);
        let result = handle_generate_constraint_prompts(request, &mock_llm, &config).await;

        assert_eq!(result.is_error, None, "Failed for vocab level {:?}", vocab_level);
    }
}

#[tokio::test]
async fn test_generate_constraint_prompts_llm_failure() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Err(TaleTrailError::NetworkError("Connection failed".into())));

    let params = GenerateConstraintPromptsParams {
        vocabulary_level: VocabularyLevel::Advanced,
        language: Language::De,
        required_elements: vec!["element1".to_string(), "element2".to_string()],
    };

    let request = create_request("generate_constraint_prompts", params);
    let result = handle_generate_constraint_prompts(request, &mock_llm, &config).await;

    assert_eq!(result.is_error, None);

    let json = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    let package: PromptPackage = serde_json::from_str(json).expect("Failed to parse PromptPackage");

    assert_eq!(package.fallback_used, true);
    assert!(package.system_prompt.contains("constraint enforcement"));
    assert!(package.system_prompt.contains("Advanced"));
}

#[tokio::test]
async fn test_generate_constraint_prompts_empty_required_elements() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_generate_prompt()
        .returning(|_, _| Ok(("System".into(), "User".into())));

    let params = GenerateConstraintPromptsParams {
        vocabulary_level: VocabularyLevel::Basic,
        language: Language::En,
        required_elements: vec![], // Empty
    };

    let request = create_request("generate_constraint_prompts", params);
    let result = handle_generate_constraint_prompts(request, &mock_llm, &config).await;

    // Should still succeed
    assert_eq!(result.is_error, None);
}

// ============================================================================
// Tests: handle_get_model_for_language
// ============================================================================

#[tokio::test]
async fn test_get_model_for_language_english() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_model_exists()
        .returning(|_| Ok(true));

    let params = GetModelForLanguageParams {
        language: Language::En,
    };

    let request = create_request("get_model_for_language", params);
    let result = handle_get_model_for_language(request, &mock_llm, &config).await;

    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);

    let model_name = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };

    // Model should come from LLM config's default model
    assert_eq!(model_name, &config.llm.provider.default_model);
}

#[tokio::test]
async fn test_get_model_for_language_german() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    mock_llm
        .expect_model_exists()
        .returning(|_| Ok(true));

    let params = GetModelForLanguageParams {
        language: Language::De,
    };

    let request = create_request("get_model_for_language", params);
    let result = handle_get_model_for_language(request, &mock_llm, &config).await;

    assert_eq!(result.is_error, None);
    assert_eq!(result.content.len(), 1);

    // Currently returns default model for German too
    let model_name = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text) => &text.text,
        _ => panic!("Expected text content"),
    };
    // Model should come from LLM config (either language-specific or default)
    assert!(!model_name.is_empty(), "Model name should not be empty");
}

#[tokio::test]
async fn test_get_model_for_language_model_not_found() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    // Mock model_exists returns false
    mock_llm
        .expect_model_exists()
        .returning(|_| Ok(false));

    let params = GetModelForLanguageParams {
        language: Language::En,
    };

    let request = create_request("get_model_for_language", params);
    let result = handle_get_model_for_language(request, &mock_llm, &config).await;

    // Should still succeed and return model name even if not found
    assert_eq!(result.is_error, None);
}

#[tokio::test]
async fn test_get_model_for_language_check_fails() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    // Mock model_exists fails
    mock_llm
        .expect_model_exists()
        .returning(|_| Err(TaleTrailError::LLMError("API error".into())));

    let params = GetModelForLanguageParams {
        language: Language::En,
    };

    let request = create_request("get_model_for_language", params);
    let result = handle_get_model_for_language(request, &mock_llm, &config).await;

    // Should still succeed and return model name
    assert_eq!(result.is_error, None);
}

#[tokio::test]
async fn test_get_model_for_language_invalid_parameters() {
    let config = create_test_config();
    let mock_llm = MockLlmService::new();

    let invalid_json = serde_json::json!({
        // Missing language field
    });

    let arguments = if let serde_json::Value::Object(map) = invalid_json {
        Some(map)
    } else {
        None
    };

    let request = CallToolRequest {
        method: rmcp::model::CallToolRequestMethod,
        params: CallToolRequestParam {
            name: "get_model_for_language".into(),
            arguments,
        },
        extensions: rmcp::model::Extensions::default(),
    };

    let result = handle_get_model_for_language(request, &mock_llm, &config).await;

    // Verify error response
    assert_eq!(result.is_error, Some(true));
}

// ============================================================================
// Integration Tests: Multiple Handlers
// ============================================================================

#[tokio::test]
async fn test_all_handlers_with_same_llm_service() {
    let config = create_test_config();
    let mut mock_llm = MockLlmService::new();

    // Setup mock to handle multiple calls
    mock_llm
        .expect_generate_prompt()
        .times(3)
        .returning(|_, _| Ok(("System".into(), "User".into())));

    mock_llm
        .expect_model_exists()
        .returning(|_| Ok(true));

    // Test story prompts
    let story_params = GenerateStoryPromptsParams {
        theme: "Adventure".to_string(),
        age_group: AgeGroup::_9To11,
        language: Language::En,
        educational_goals: vec!["Learn".to_string()],
    };
    let story_request = create_request("generate_story_prompts", story_params);
    let story_result = handle_generate_story_prompts(story_request, &mock_llm, &config).await;
    assert_eq!(story_result.is_error, None);

    // Test validation prompts
    let validation_params = GenerateValidationPromptsParams {
        age_group: AgeGroup::_9To11,
        language: Language::En,
        content_type: "story".to_string(),
    };
    let validation_request = create_request("generate_validation_prompts", validation_params);
    let validation_result = handle_generate_validation_prompts(validation_request, &mock_llm, &config).await;
    assert_eq!(validation_result.is_error, None);

    // Test constraint prompts
    let constraint_params = GenerateConstraintPromptsParams {
        vocabulary_level: VocabularyLevel::Basic,
        language: Language::En,
        required_elements: vec!["hero".to_string()],
    };
    let constraint_request = create_request("generate_constraint_prompts", constraint_params);
    let constraint_result = handle_generate_constraint_prompts(constraint_request, &mock_llm, &config).await;
    assert_eq!(constraint_result.is_error, None);

    // Test get model
    let model_params = GetModelForLanguageParams {
        language: Language::En,
    };
    let model_request = create_request("get_model_for_language", model_params);
    let model_result = handle_get_model_for_language(model_request, &mock_llm, &config).await;
    assert_eq!(model_result.is_error, None);
}
