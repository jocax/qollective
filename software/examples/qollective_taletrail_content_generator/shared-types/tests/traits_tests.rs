//! Test examples for service trait mocking
//!
//! Demonstrates how to use mock implementations for unit testing.
//! Run with: `cargo test -p shared-types --features mocking`

#[cfg(feature = "mocking")]
mod mocking_tests {
    use shared_types::*;
    use shared_types::traits::llm_service::NodeContext;
    use mockall::predicate::*;

    // Helper functions for creating test data
    fn mock_prompt_generation_request() -> PromptGenerationRequest {
        PromptGenerationRequest {
            generation_request: GenerationRequest {
                theme: "space adventure".to_string(),
                age_group: AgeGroup::SixToEight,
                language: Language::En,
                node_count: 16,
                vocabulary_level: VocabularyLevel::Basic,
                educational_goals: vec!["reading comprehension".to_string()],
                required_elements: vec![],
                tenant_id: 123,
                author_id: None,
                tags: vec![],
                prompt_packages: None,
            },
            service_target: MCPServiceType::StoryGenerator,
            node_context: None,
            batch_info: None,
        }
    }

    fn mock_prompt_package(service_type: MCPServiceType) -> PromptPackage {
        PromptPackage {
            system_prompt: "You are a story generator.".to_string(),
            user_prompt: "Generate a space adventure story for ages 6-8.".to_string(),
            language: Language::En,
            llm_model: "gpt-4".to_string(),
            llm_config: LLMConfig {
                temperature: 0.7,
                max_tokens: 2000,
                top_p: 0.9,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                stop_sequences: vec![],
            },
            prompt_metadata: PromptMetadata {
                generated_at: chrono::Utc::now(),
                template_version: "1.0.0".to_string(),
                generation_method: PromptGenerationMethod::LLMGenerated,
                age_group_context: AgeGroup::SixToEight,
                language_context: Language::En,
                service_target: service_type,
                theme_context: "space adventure".to_string(),
            },
            fallback_used: false,
        }
    }

    /// Example 1: Mock LlmService for unit testing
    #[tokio::test]
    async fn test_mock_llm_service() {
        let mut mock_llm = MockLlmService::new();

        // Setup expectations
        mock_llm
            .expect_generate_prompt()
            .returning(|_, _| {
                Ok((
                    "System: Generate educational content.".to_string(),
                    "User: Create a story about space.".to_string(),
                ))
            });

        mock_llm
            .expect_generate_content()
            .returning(|_, _| {
                Ok("Once upon a time in a galaxy far away...".to_string())
            });

        // Use mock in test
        let request = mock_prompt_generation_request();
        let (system, user) = mock_llm
            .generate_prompt("meta-prompt", &request)
            .await
            .expect("Should generate prompts");

        assert!(system.contains("System"));
        assert!(user.contains("User"));

        let node_context = NodeContext {
            previous_content: None,
            choices_made: vec![],
            node_position: 1,
            total_nodes: 16,
        };
        let package = mock_prompt_package(MCPServiceType::StoryGenerator);
        let content = mock_llm
            .generate_content(&package, &node_context)
            .await
            .expect("Should generate content");

        assert!(content.contains("galaxy"));
    }

    /// Example 2: Mock PromptHelperService for orchestrator tests
    #[tokio::test]
    async fn test_mock_prompt_helper_service() {
        let mut mock_helper = MockPromptHelperService::new();

        // Setup expectations for story prompts
        mock_helper
            .expect_generate_story_prompts()
            .returning(|_| Ok(mock_prompt_package(MCPServiceType::StoryGenerator)));

        // Use mock in test
        let request = mock_prompt_generation_request();
        let prompts = mock_helper
            .generate_story_prompts(&request)
            .await
            .expect("Should generate story prompts");

        assert_eq!(prompts.language, Language::En);
        assert!(!prompts.fallback_used);
        assert_eq!(prompts.prompt_metadata.service_target, MCPServiceType::StoryGenerator);
    }

    /// Example 3: Mock McpTransport for integration tests
    #[tokio::test]
    async fn test_mock_mcp_transport() {
        let mut mock_transport = MockMcpTransport::new();

        // Setup expectations with generic types
        mock_transport
            .expect_call_mcp_tool::<PromptGenerationRequest, PromptPackage>()
            .returning(|_subject, _tool, _req| {
                Ok(mock_prompt_package(MCPServiceType::PromptHelper))
            });

        // Use mock in test
        let request = mock_prompt_generation_request();
        let response: PromptPackage = mock_transport
            .call_mcp_tool(
                "mcp.prompt.generate".to_string(),
                "generate_story_prompts".to_string(),
                request,
            )
            .await
            .expect("Should call MCP tool");

        assert_eq!(response.language, Language::En);
    }

    /// Example 4: Mock RequestMapper for gateway tests
    #[test]
    fn test_mock_request_mapper() {
        let mut mock_mapper = MockRequestMapper::new();

        // Setup expectations
        mock_mapper
            .expect_external_to_internal()
            .returning(|ext, tenant_id| {
                Ok(GenerationRequest {
                    theme: ext.theme,
                    age_group: ext.age_group,
                    language: ext.language,
                    node_count: 16,
                    vocabulary_level: VocabularyLevel::Basic,
                    educational_goals: vec!["reading comprehension".to_string()],
                    required_elements: vec![],
                    tenant_id: tenant_id.try_into().unwrap(),
                    author_id: None,
                    tags: vec![],
                    prompt_packages: None,
                })
            });

        // Use mock in test
        let external = ExternalGenerationRequestV1 {
            theme: "space".to_string(),
            age_group: AgeGroup::SixToEight,
            language: Language::En,
        };

        let internal = mock_mapper
            .external_to_internal(external, 123)
            .expect("Should map external to internal");

        assert_eq!(internal.tenant_id, 123);
        assert_eq!(internal.node_count, 16);
        assert_eq!(internal.vocabulary_level, VocabularyLevel::Basic);
    }

    /// Example 5: Mock StoryGeneratorService for orchestrator tests
    #[tokio::test]
    async fn test_mock_story_generator_service() {
        let mut mock_generator = MockStoryGeneratorService::new();

        // Setup expectations
        mock_generator
            .expect_generate_structure()
            .returning(|_req, _prompts| {
                Ok(DAG {
                    nodes: std::collections::HashMap::new(),
                    edges: vec![],
                    start_node_id: uuid::Uuid::new_v4(),
                    convergence_points: vec![],
                })
            });

        // Use mock in test
        let request = GenerationRequest {
            theme: "space".to_string(),
            age_group: AgeGroup::SixToEight,
            language: Language::En,
            node_count: 16,
            vocabulary_level: VocabularyLevel::Basic,
            educational_goals: vec![],
            required_elements: vec![],
            tenant_id: 123,
            author_id: None,
            tags: vec![],
            prompt_packages: None,
        };
        let prompts = mock_prompt_package(MCPServiceType::StoryGenerator);

        let dag = mock_generator
            .generate_structure(&request, &prompts)
            .await
            .expect("Should generate structure");

        assert_eq!(dag.nodes.len(), 0); // Mock returns empty DAG
    }

    /// Example 6: Mock ValidationService for quality control tests
    #[tokio::test]
    async fn test_mock_validation_service() {
        let mut mock_validator = MockValidationService::new();

        // Setup expectations
        mock_validator
            .expect_validate_quality()
            .returning(|_content, _prompts, _request| {
                Ok(ValidationResult {
                    is_valid: true,
                    age_appropriate_score: 0.95,
                    safety_issues: vec![],
                    educational_value_score: 0.90,
                    corrections: vec![],
                    correction_capability: CorrectionCapability::NoFixPossible,
                })
            });

        // Use mock in test
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id: uuid::Uuid::new_v4(),
            text: "A friendly astronaut explores the moon.".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let prompts = mock_prompt_package(MCPServiceType::QualityControl);
        let request = GenerationRequest {
            theme: "space".to_string(),
            age_group: AgeGroup::SixToEight,
            language: Language::En,
            node_count: 16,
            vocabulary_level: VocabularyLevel::Basic,
            educational_goals: vec![],
            required_elements: vec![],
            tenant_id: 123,
            author_id: None,
            tags: vec![],
            prompt_packages: None,
        };

        let result = mock_validator
            .validate_quality(&content, &prompts, &request)
            .await
            .expect("Should validate quality");

        assert!(result.is_valid);
        assert!(result.safety_issues.is_empty());
    }
}

#[cfg(not(feature = "mocking"))]
#[test]
fn mocking_feature_not_enabled() {
    // This test runs when mocking feature is NOT enabled
    // to verify that the feature flag works correctly
    println!("Mocking feature is not enabled. Run with --features mocking to test mocks.");
}
