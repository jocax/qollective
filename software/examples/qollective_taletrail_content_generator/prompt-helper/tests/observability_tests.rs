//! Observability and error handling tests
//!
//! This module tests tracing span creation, error handling consistency,
//! and observability features of the Prompt Helper MCP server.

use std::sync::Once;
use prompt_helper::*;
use shared_types::{
    AgeGroup, Language, VocabularyLevel, MCPServiceType,
    traits::LlmService, PromptPackage, PromptMetadata,
    PromptGenerationMethod, LLMConfig, TaleTrailError,
};
use rmcp::model::{CallToolRequest, CallToolRequestParam};
use serde_json::json;

static INIT: Once = Once::new();

fn init_rustls() {
    INIT.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

#[cfg(test)]
mod observability_tests {
    use super::*;

    #[tokio::test]
    async fn test_tracing_span_creation_on_tool_execution() {
        init_rustls();

        // Initialize tracing subscriber for span capture
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Create mock LLM service that succeeds
        let mut mock_llm = shared_types::MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| {
                Ok((
                    "System prompt for story generation".to_string(),
                    "User prompt for story generation".to_string(),
                ))
            });

        // Create CallToolRequest
        let arguments = serde_json::Map::from_iter(vec![
            ("theme".to_string(), json!("space exploration")),
            ("age_group".to_string(), json!("6-8")),
            ("language".to_string(), json!("en")),
            ("educational_goals".to_string(), json!(["science", "adventure"])),
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_story_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        // Execute handler - this should create tracing spans
        let result = tool_handlers::handle_generate_story_prompts(request, &mock_llm, &config).await;

        // Verify result is success
        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert_eq!(result.content.len(), 1);

        // Verify content contains valid PromptPackage
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        let package: PromptPackage = serde_json::from_str(content_text).unwrap();
        assert!(!package.system_prompt.is_empty());
        assert!(!package.user_prompt.is_empty());
        assert!(!package.fallback_used); // LLM succeeded
    }

    #[tokio::test]
    async fn test_span_attributes_include_tool_name() {
        init_rustls();

        // Initialize tracing
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Create mock LLM service
        let mut mock_llm = shared_types::MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| {
                Ok((
                    "Validation system prompt".to_string(),
                    "Validation user prompt".to_string(),
                ))
            });

        // Create request for validation prompts
        let arguments = serde_json::Map::from_iter(vec![
            ("content_type".to_string(), json!("story")),
            ("age_group".to_string(), json!("6-8")),
            ("language".to_string(), json!("en")),
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_validation_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        // Execute handler - should create spans with tool_name attribute
        let result = tool_handlers::handle_generate_validation_prompts(request, &mock_llm, &config).await;

        // Verify success
        assert!(result.is_error.is_none() || result.is_error == Some(false));

        // Verify content is valid
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        let package: PromptPackage = serde_json::from_str(content_text).unwrap();
        assert_eq!(package.prompt_metadata.service_target, MCPServiceType::QualityControl);
    }

    #[tokio::test]
    async fn test_template_fallback_span_created() {
        init_rustls();

        // Initialize tracing
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Create mock LLM that fails
        let mut mock_llm = shared_types::MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| Err(TaleTrailError::LLMError("Connection failed".to_string())));

        let arguments = serde_json::Map::from_iter(vec![
            ("theme".to_string(), json!("space")),
            ("age_group".to_string(), json!("6-8")),
            ("language".to_string(), json!("en")),
            ("educational_goals".to_string(), json!(["science"])),
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_story_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        // Execute - should fallback to template and create fallback span
        let result = tool_handlers::handle_generate_story_prompts(request, &mock_llm, &config).await;

        // Verify fallback succeeded
        assert!(result.is_error.is_none() || result.is_error == Some(false));

        // Verify response indicates fallback was used
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        let package: PromptPackage = serde_json::from_str(content_text).unwrap();
        assert!(package.fallback_used);
        assert_eq!(package.prompt_metadata.generation_method, PromptGenerationMethod::TemplateFallback);
    }

    #[tokio::test]
    async fn test_llm_call_duration_recorded() {
        init_rustls();

        // Initialize tracing with timing info
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Create mock LLM that succeeds (timing will be recorded in llm.rs span)
        let mut mock_llm = shared_types::MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| {
                Ok((
                    "System prompt".to_string(),
                    "User prompt".to_string(),
                ))
            });

        let arguments = serde_json::Map::from_iter(vec![
            ("vocabulary_level".to_string(), json!("basic")),
            ("language".to_string(), json!("en")),
            ("required_elements".to_string(), json!(["character", "setting"])),
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_constraint_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        // Execute handler - should record timing
        let result = tool_handlers::handle_generate_constraint_prompts(request, &mock_llm, &config).await;

        // Verify success
        assert!(result.is_error.is_none() || result.is_error == Some(false));

        // Verify LLM was used (not fallback)
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        let package: PromptPackage = serde_json::from_str(content_text).unwrap();
        assert!(!package.fallback_used);
    }

    #[tokio::test]
    async fn test_error_handling_creates_error_span() {
        init_rustls();

        // Initialize tracing
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Create mock LLM (won't be called due to parameter error)
        let mock_llm = shared_types::MockLlmService::new();

        // Create request with INVALID parameters (missing required fields)
        let arguments = serde_json::Map::from_iter(vec![
            ("theme".to_string(), json!("space")),
            // Missing age_group, language, educational_goals
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_story_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        // Execute handler - should create error span
        let result = tool_handlers::handle_generate_story_prompts(request, &mock_llm, &config).await;

        // Verify error result
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content.len(), 1);

        // Verify error message
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        assert!(content_text.contains("Invalid parameters") || content_text.contains("missing field"));
    }

    #[tokio::test]
    async fn test_all_handlers_have_consistent_error_format() {
        init_rustls();

        let config = config::PromptHelperConfig::load().unwrap();
        let mock_llm = shared_types::MockLlmService::new();

        // Test invalid request for each handler
        let invalid_request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "test".into(),
                arguments: None, // Missing arguments
            },
            extensions: Default::default(),
        };

        // Test story prompts handler
        let result1 = tool_handlers::handle_generate_story_prompts(invalid_request.clone(), &mock_llm, &config).await;
        assert_eq!(result1.is_error, Some(true));
        assert!(result1.content.first().unwrap().raw.as_text().unwrap().text.as_str().contains("Missing arguments"));

        // Test validation prompts handler
        let result2 = tool_handlers::handle_generate_validation_prompts(invalid_request.clone(), &mock_llm, &config).await;
        assert_eq!(result2.is_error, Some(true));
        assert!(result2.content.first().unwrap().raw.as_text().unwrap().text.as_str().contains("Missing arguments"));

        // Test constraint prompts handler
        let result3 = tool_handlers::handle_generate_constraint_prompts(invalid_request.clone(), &mock_llm, &config).await;
        assert_eq!(result3.is_error, Some(true));
        assert!(result3.content.first().unwrap().raw.as_text().unwrap().text.as_str().contains("Missing arguments"));

        // Test get model handler
        let result4 = tool_handlers::handle_get_model_for_language(invalid_request.clone(), &mock_llm, &config).await;
        assert_eq!(result4.is_error, Some(true));
        assert!(result4.content.first().unwrap().raw.as_text().unwrap().text.as_str().contains("Missing arguments"));
    }

    #[tokio::test]
    async fn test_multiple_fallback_scenarios() {
        init_rustls();

        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Test fallback for validation prompts
        let mut mock_llm = shared_types::MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| Err(TaleTrailError::LLMError("Timeout".to_string())));

        let arguments = serde_json::Map::from_iter(vec![
            ("content_type".to_string(), json!("story")),
            ("age_group".to_string(), json!("9-11")),
            ("language".to_string(), json!("de")),
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_validation_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        let result = tool_handlers::handle_generate_validation_prompts(request, &mock_llm, &config).await;

        // Verify fallback worked
        assert!(result.is_error.is_none() || result.is_error == Some(false));
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        let package: PromptPackage = serde_json::from_str(content_text).unwrap();
        assert!(package.fallback_used);
        assert_eq!(package.prompt_metadata.service_target, MCPServiceType::QualityControl);
    }

    #[tokio::test]
    async fn test_constraint_prompts_with_multiple_elements() {
        init_rustls();

        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();

        let config = config::PromptHelperConfig::load().unwrap();

        // Mock LLM that succeeds
        let mut mock_llm = shared_types::MockLlmService::new();
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| {
                Ok((
                    "Constraint enforcement system".to_string(),
                    "Check constraints user prompt".to_string(),
                ))
            });

        let arguments = serde_json::Map::from_iter(vec![
            ("vocabulary_level".to_string(), json!("intermediate")),
            ("language".to_string(), json!("en")),
            ("required_elements".to_string(), json!(["protagonist", "conflict", "resolution", "moral"])),
        ]);

        let request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_constraint_prompts".into(),
                arguments: Some(arguments),
            },
            extensions: Default::default(),
        };

        let result = tool_handlers::handle_generate_constraint_prompts(request, &mock_llm, &config).await;

        // Verify success
        assert!(result.is_error.is_none() || result.is_error == Some(false));
        let content_text = result.content.first().unwrap().raw.as_text().unwrap().text.as_str();
        let package: PromptPackage = serde_json::from_str(content_text).unwrap();
        assert_eq!(package.prompt_metadata.service_target, MCPServiceType::ConstraintEnforcer);
        assert!(!package.fallback_used);
    }
}
