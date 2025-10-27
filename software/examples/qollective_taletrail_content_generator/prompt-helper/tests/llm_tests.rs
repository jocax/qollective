//! Comprehensive tests for LLM integration in Prompt-Helper service
//!
//! Tests cover:
//! - RigLlmService initialization and configuration
//! - Meta-prompt execution with LLM response parsing
//! - Prompt separator parsing (extract SYSTEM_PROMPT and USER_PROMPT sections)
//! - Error handling for malformed LLM responses
//! - Model availability checking and listing
//!
//! Note: Tests use stubs and mocks to avoid hitting real LM Studio.
//! Integration tests with actual LM Studio are in a separate ignored test suite.

use shared_types::{
    errors::TaleTrailError, AgeGroup, GenerationRequest, Language, MCPServiceType,
    PromptGenerationRequest,
};

// ============================================================================
// Test Utilities and Stubs
// ============================================================================

/// Stub for RigLlmService that simulates rig-core behavior without actual API calls
///
/// This stub implementation allows us to test LLM integration logic without
/// requiring a running LM Studio instance.
#[derive(Debug)]
struct StubRigLlmService {
    base_url: String,
    model_name: String,
    /// Simulated LLM response for testing
    mock_response: Option<String>,
    /// Simulated model list
    mock_models: Vec<String>,
    /// Should the next operation fail?
    should_fail: bool,
}

impl StubRigLlmService {
    /// Create new stub service with configuration
    fn new(base_url: &str, model_name: &str) -> Result<Self, TaleTrailError> {
        if base_url.is_empty() {
            return Err(TaleTrailError::ConfigError(
                "Base URL cannot be empty".to_string(),
            ));
        }
        if model_name.is_empty() {
            return Err(TaleTrailError::ConfigError(
                "Model name cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            base_url: base_url.to_string(),
            model_name: model_name.to_string(),
            mock_response: None,
            mock_models: vec![model_name.to_string()],
            should_fail: false,
        })
    }

    /// Set mock response for next generate_prompt call
    fn with_mock_response(mut self, response: String) -> Self {
        self.mock_response = Some(response);
        self
    }

    /// Set mock model list for next list_models call
    fn with_mock_models(mut self, models: Vec<String>) -> Self {
        self.mock_models = models;
        self
    }

    /// Make next operation fail with LLM error
    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    /// Generate prompts using meta-prompt (simulates rig-core LLM call)
    async fn generate_prompt(
        &self,
        _meta_prompt: &str,
        _context: &PromptGenerationRequest,
    ) -> Result<(String, String), TaleTrailError> {
        if self.should_fail {
            return Err(TaleTrailError::LLMError(
                "Simulated LLM API failure".to_string(),
            ));
        }

        let response = self
            .mock_response
            .clone()
            .unwrap_or_else(|| "Default system prompt\n---SEPARATOR---\nDefault user prompt".to_string());

        // Parse response to extract system and user prompts
        Self::parse_llm_response(&response)
    }

    /// Parse LLM response to extract system and user prompts
    ///
    /// Expected format: "SYSTEM PROMPT\n---SEPARATOR---\nUSER PROMPT"
    fn parse_llm_response(response: &str) -> Result<(String, String), TaleTrailError> {
        // Try different separator variants
        let separators = vec![
            "---SEPARATOR---",
            "--- SEPARATOR ---",
            "---separator---",
            "--- separator ---",
            "\n---\n",
            "###SEPARATOR###",
        ];

        for separator in separators {
            if let Some((system, user)) = response.split_once(separator) {
                let system_prompt = system.trim().to_string();
                let user_prompt = user.trim().to_string();

                if system_prompt.is_empty() {
                    return Err(TaleTrailError::LLMError(
                        "System prompt is empty after parsing".to_string(),
                    ));
                }
                if user_prompt.is_empty() {
                    return Err(TaleTrailError::LLMError(
                        "User prompt is empty after parsing".to_string(),
                    ));
                }

                return Ok((system_prompt, user_prompt));
            }
        }

        Err(TaleTrailError::LLMError(
            "LLM response missing separator. Expected format: 'SYSTEM PROMPT\n---SEPARATOR---\nUSER PROMPT'"
                .to_string(),
        ))
    }

    /// List available models
    async fn list_models(&self) -> Result<Vec<String>, TaleTrailError> {
        if self.should_fail {
            return Err(TaleTrailError::LLMError("Failed to fetch model list".to_string()));
        }
        Ok(self.mock_models.clone())
    }

    /// Check if specific model is available
    async fn model_exists(&self, model_name: &str) -> Result<bool, TaleTrailError> {
        if self.should_fail {
            return Err(TaleTrailError::LLMError(
                "Failed to check model availability".to_string(),
            ));
        }
        Ok(self.mock_models.contains(&model_name.to_string()))
    }
}

/// Create test PromptGenerationRequest
fn create_test_request() -> PromptGenerationRequest {
    PromptGenerationRequest {
        node_context: None,
        generation_request: GenerationRequest {
            tags: None,
            node_count: Some(10),
            age_group: AgeGroup::_9To11,
            tenant_id: 1,
            prompt_packages: None,
            theme: "Space Adventure".to_string(),
            educational_goals: Some(vec!["Learn about planets".to_string()]),
            author_id: None,
            required_elements: None,
            vocabulary_level: None,
            language: Language::En,
            story_structure: None,
            dag_config: None,
            validation_policy: None,
        },
        service_target: MCPServiceType::PromptHelper,
        batch_info: None,
    }
}

// ============================================================================
// Tests: RigLlmService Initialization
// ============================================================================

#[tokio::test]
async fn test_rig_llm_service_initialization_success() {
    let base_url = "http://127.0.0.1:1234";
    let model_name = "llama-3.2-3b-instruct";

    let service = StubRigLlmService::new(base_url, model_name);

    assert!(service.is_ok());
    let service = service.unwrap();
    assert_eq!(service.base_url, base_url);
    assert_eq!(service.model_name, model_name);
}

#[tokio::test]
async fn test_rig_llm_service_initialization_empty_base_url() {
    let result = StubRigLlmService::new("", "llama-3.2-3b-instruct");

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::ConfigError(msg) => {
            assert!(msg.contains("Base URL cannot be empty"));
        }
        _ => panic!("Expected ConfigError"),
    }
}

#[tokio::test]
async fn test_rig_llm_service_initialization_empty_model_name() {
    let result = StubRigLlmService::new("http://127.0.0.1:1234", "");

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::ConfigError(msg) => {
            assert!(msg.contains("Model name cannot be empty"));
        }
        _ => panic!("Expected ConfigError"),
    }
}

// ============================================================================
// Tests: Meta-Prompt Execution
// ============================================================================

#[tokio::test]
async fn test_generate_prompt_success_with_separator() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response(
            "You are an educational story assistant.\n---SEPARATOR---\nGenerate a story about {theme}"
                .to_string(),
        );

    let request = create_test_request();
    let meta_prompt = "Generate prompts for a children's story about space";

    let result = service.generate_prompt(meta_prompt, &request).await;

    assert!(result.is_ok());
    let (system_prompt, user_prompt) = result.unwrap();
    assert_eq!(system_prompt, "You are an educational story assistant.");
    assert_eq!(user_prompt, "Generate a story about {theme}");
}

#[tokio::test]
async fn test_generate_prompt_success_with_variant_separators() {
    let test_cases = vec![
        (
            "System\n---SEPARATOR---\nUser",
            ("System", "User"),
        ),
        (
            "System\n--- SEPARATOR ---\nUser",
            ("System", "User"),
        ),
        (
            "System\n---separator---\nUser",
            ("System", "User"),
        ),
        (
            "System\n--- separator ---\nUser",
            ("System", "User"),
        ),
        (
            "System\n---\nUser",
            ("System", "User"),
        ),
        (
            "System\n###SEPARATOR###\nUser",
            ("System", "User"),
        ),
    ];

    for (response, expected) in test_cases {
        let service = StubRigLlmService::new("http://127.0.0.1:1234", "test-model")
            .unwrap()
            .with_mock_response(response.to_string());

        let request = create_test_request();
        let result = service.generate_prompt("meta-prompt", &request).await;

        assert!(result.is_ok(), "Failed to parse: {}", response);
        let (system, user) = result.unwrap();
        assert_eq!(system, expected.0);
        assert_eq!(user, expected.1);
    }
}

#[tokio::test]
async fn test_generate_prompt_parsing_extracts_prompts() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response(
            "You are an AI assistant specialized in creating educational content for children.\n\
             Your role is to generate age-appropriate stories.\n\
             ---SEPARATOR---\n\
             Please create a story about {theme} for age group {age_group}.\n\
             Include educational elements about {educational_goals}."
                .to_string(),
        );

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_ok());
    let (system_prompt, user_prompt) = result.unwrap();

    // Verify system prompt
    assert!(system_prompt.contains("AI assistant"));
    assert!(system_prompt.contains("educational content"));
    assert!(system_prompt.contains("children"));

    // Verify user prompt
    assert!(user_prompt.contains("story about {theme}"));
    assert!(user_prompt.contains("{age_group}"));
    assert!(user_prompt.contains("{educational_goals}"));
}

// ============================================================================
// Tests: Error Handling
// ============================================================================

#[tokio::test]
async fn test_generate_prompt_missing_separator_returns_error() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response("This is a response without any separator".to_string());

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::LLMError(msg) => {
            assert!(msg.contains("missing separator"));
        }
        _ => panic!("Expected LLMError"),
    }
}

#[tokio::test]
async fn test_generate_prompt_empty_system_prompt_returns_error() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response("\n---SEPARATOR---\nUser prompt here".to_string());

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::LLMError(msg) => {
            assert!(msg.contains("System prompt is empty"));
        }
        _ => panic!("Expected LLMError for empty system prompt"),
    }
}

#[tokio::test]
async fn test_generate_prompt_empty_user_prompt_returns_error() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response("System prompt here\n---SEPARATOR---\n".to_string());

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::LLMError(msg) => {
            assert!(msg.contains("User prompt is empty"));
        }
        _ => panic!("Expected LLMError for empty user prompt"),
    }
}

#[tokio::test]
async fn test_generate_prompt_llm_api_failure() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_failure();

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::LLMError(msg) => {
            assert!(msg.contains("Simulated LLM API failure"));
        }
        _ => panic!("Expected LLMError"),
    }
}

// ============================================================================
// Tests: Model Availability Checking
// ============================================================================

#[tokio::test]
async fn test_model_exists_returns_true_for_available_model() {
    let model_name = "llama-3.2-3b-instruct";
    let service = StubRigLlmService::new("http://127.0.0.1:1234", model_name)
        .unwrap()
        .with_mock_models(vec![
            model_name.to_string(),
            "gpt-4".to_string(),
            "claude-3-opus".to_string(),
        ]);

    let result = service.model_exists(model_name).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_model_exists_returns_false_for_unavailable_model() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_models(vec!["gpt-4".to_string(), "claude-3-opus".to_string()]);

    let result = service.model_exists("nonexistent-model").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[tokio::test]
async fn test_model_exists_api_failure() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_failure();

    let result = service.model_exists("llama-3.2-3b-instruct").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::LLMError(msg) => {
            assert!(msg.contains("Failed to check model availability"));
        }
        _ => panic!("Expected LLMError"),
    }
}

// ============================================================================
// Tests: Model Listing
// ============================================================================

#[tokio::test]
async fn test_list_models_returns_available_models() {
    let expected_models = vec![
        "llama-3.2-3b-instruct".to_string(),
        "gpt-4".to_string(),
        "claude-3-opus".to_string(),
        "mistral-7b".to_string(),
    ];

    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_models(expected_models.clone());

    let result = service.list_models().await;

    assert!(result.is_ok());
    let models = result.unwrap();
    assert_eq!(models.len(), 4);
    assert_eq!(models, expected_models);
}

#[tokio::test]
async fn test_list_models_returns_empty_list_when_no_models() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_models(vec![]);

    let result = service.list_models().await;

    assert!(result.is_ok());
    let models = result.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_list_models_api_failure() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_failure();

    let result = service.list_models().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TaleTrailError::LLMError(msg) => {
            assert!(msg.contains("Failed to fetch model list"));
        }
        _ => panic!("Expected LLMError"),
    }
}

// ============================================================================
// Tests: Separator Parsing Edge Cases
// ============================================================================

#[tokio::test]
async fn test_parse_llm_response_with_separator_in_content() {
    // LLM response that contains separator-like text in the actual content
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response(
            "System: Use --- as markdown separator\n---SEPARATOR---\nUser: Explain separators"
                .to_string(),
        );

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_ok());
    let (system, user) = result.unwrap();
    assert_eq!(system, "System: Use --- as markdown separator");
    assert_eq!(user, "User: Explain separators");
}

#[tokio::test]
async fn test_parse_llm_response_with_multiline_prompts() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response(
            "You are an AI assistant.\n\
             You help with educational content.\n\
             You are patient and kind.\n\
             ---SEPARATOR---\n\
             Generate a story.\n\
             Make it engaging.\n\
             Include educational elements."
                .to_string(),
        );

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_ok());
    let (system, user) = result.unwrap();
    assert!(system.contains("AI assistant"));
    assert!(system.contains("patient and kind"));
    assert!(user.contains("Generate a story"));
    assert!(user.contains("educational elements"));
}

#[tokio::test]
async fn test_parse_llm_response_with_extra_whitespace() {
    let service = StubRigLlmService::new("http://127.0.0.1:1234", "llama-3.2-3b-instruct")
        .unwrap()
        .with_mock_response(
            "\n\n  System prompt with extra whitespace  \n\n\
             ---SEPARATOR---\n\n\
             User prompt with extra whitespace  \n\n"
                .to_string(),
        );

    let request = create_test_request();
    let result = service.generate_prompt("meta-prompt", &request).await;

    assert!(result.is_ok());
    let (system, user) = result.unwrap();
    // trim() should remove extra whitespace
    assert_eq!(system, "System prompt with extra whitespace");
    assert_eq!(user, "User prompt with extra whitespace");
}

// ============================================================================
// Integration Tests (Ignored - Require Real LM Studio)
// ============================================================================

#[tokio::test]
#[ignore = "Requires running LM Studio instance"]
async fn test_real_llm_integration_with_lm_studio() {
    // This test would use actual RigLlmService implementation
    // with rig-core to hit a real LM Studio instance
    //
    // Run with: cargo test --test llm_tests -- --ignored
    //
    // Prerequisites:
    // 1. LM Studio running at http://127.0.0.1:1234
    // 2. Model loaded (e.g., llama-3.2-3b-instruct)

    todo!("Implement when RigLlmService is fully implemented");
}

#[tokio::test]
#[ignore = "Requires running LM Studio instance"]
async fn test_real_llm_model_listing() {
    // Test actual model listing from LM Studio API
    //
    // This would verify:
    // - HTTP client connection to LM Studio
    // - API response parsing
    // - Model list deserialization

    todo!("Implement when RigLlmService is fully implemented");
}

// ============================================================================
// Performance Tests (Ignored - Long Running)
// ============================================================================

#[tokio::test]
#[ignore = "Long running performance test"]
async fn test_generate_prompt_performance() {
    // Test LLM performance with different meta-prompt sizes
    // Measure response times and throughput

    todo!("Implement performance benchmarks");
}
