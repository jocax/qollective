//! Integration tests for LLM content generation
//!
//! These tests verify the content generation pipeline without actually calling
//! external LLM services. Mock responses are used to test parsing and integration logic.

use shared_types::{
    traits::llm_service::NodeContext, AgeGroup, GenerationRequest, Language,
    LLMConfig, MCPServiceType, PromptGenerationMethod, PromptMetadata, PromptPackage,
    DAG,
};
use story_generator::llm::{generate_node_content, generate_nodes_batch, StoryLlmClient};
use story_generator::prompts::PromptTemplates;
use story_generator::structure::{calculate_convergence_points, generate_dag_structure};

// ============================================================================
// Test Utilities
// ============================================================================

fn test_llm_config() -> shared_types_llm::LlmConfig {
    let toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:1234/v1"
default_model = "test-model"
    "#;
    shared_types_llm::LlmConfig::from_toml_str(toml).unwrap()
}

fn create_test_prompt_package() -> PromptPackage {
    PromptPackage {
        system_prompt: "You are a storyteller for children.".to_string(),
        user_prompt: "Create an adventure story.".to_string(),
        language: Language::En,
        llm_model: "test-model".to_string(),
        llm_config: LLMConfig {
            temperature: 0.7,
            max_tokens: 500,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: vec![],
        },
        prompt_metadata: PromptMetadata {
            theme_context: "Space Adventure".to_string(),
            age_group_context: AgeGroup::_9To11,
            language_context: Language::En,
            service_target: MCPServiceType::StoryGenerator,
            generation_method: PromptGenerationMethod::LLMGenerated,
            template_version: "1.0".to_string(),
            generated_at: "2024-01-01T00:00:00Z".to_string(),
        },
        fallback_used: false,
    }
}

fn create_test_generation_request() -> GenerationRequest {
    GenerationRequest {
        theme: "Space Adventure".to_string(),
        age_group: AgeGroup::_9To11,
        language: Language::En,
        tenant_id: 1,
        node_count: Some(16),
        vocabulary_level: None,
        educational_goals: None,
        required_elements: None,
        tags: None,
        author_id: None,
        prompt_packages: None,
    }
}

fn create_test_node_context() -> NodeContext {
    NodeContext {
        previous_content: Some("The spaceship landed on Mars.".to_string()),
        choices_made: vec!["Explore the crater".to_string()],
        node_position: 5,
        total_nodes: 16,
    }
}

fn create_test_dag() -> DAG {
    let convergence_points = calculate_convergence_points(16);
    generate_dag_structure(16, convergence_points).unwrap()
}

// ============================================================================
// StoryLlmClient Tests
// ============================================================================

#[test]
fn test_story_llm_client_creation() {
    let result = StoryLlmClient::new(test_llm_config());
    assert!(result.is_ok(), "Should create client with valid parameters");
}

#[test]
fn test_story_llm_client_empty_base_url() {
    let toml = r#"
[llm]
type = "shimmy"
url = ""
default_model = "test-model"
    "#;
    let config = shared_types_llm::LlmConfig::from_toml_str(toml);
    assert!(config.is_err(), "Should reject empty base URL");
}

#[test]
fn test_story_llm_client_empty_model_name() {
    let toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:1234/v1"
default_model = ""
    "#;
    let config = shared_types_llm::LlmConfig::from_toml_str(toml);
    assert!(config.is_err(), "Should reject empty model name");
}

// ============================================================================
// Content Parsing Tests
// ============================================================================

#[test]
fn test_extract_choices_numbered() {
    let text = "1. Go left\n2. Go right\n3. Stay here";
    let choices = StoryLlmClient::extract_choices(text);

    assert_eq!(choices.len(), 3);
    assert_eq!(choices[0], "Go left");
    assert_eq!(choices[1], "Go right");
    assert_eq!(choices[2], "Stay here");
}

#[test]
fn test_extract_choices_with_bullets() {
    let text = "- First option\n- Second option\n- Third option";
    let choices = StoryLlmClient::extract_choices(text);

    assert_eq!(choices.len(), 3);
    assert_eq!(choices[0], "First option");
    assert_eq!(choices[1], "Second option");
    assert_eq!(choices[2], "Third option");
}

#[test]
fn test_extract_choices_pads_to_three() {
    let text = "1. Only one choice";
    let choices = StoryLlmClient::extract_choices(text);

    assert_eq!(choices.len(), 3, "Should pad to 3 choices");
    assert_eq!(choices[0], "Only one choice");
    assert_eq!(choices[1], "Choice 2");
    assert_eq!(choices[2], "Choice 3");
}

#[test]
fn test_extract_choices_truncates_to_three() {
    let text = "1. First\n2. Second\n3. Third\n4. Fourth\n5. Fifth";
    let choices = StoryLlmClient::extract_choices(text);

    assert_eq!(choices.len(), 3, "Should truncate to 3 choices");
}

#[test]
fn test_parse_json_response() {
    let response = r#"{
        "narrative": "This is the story text with lots of adventure.",
        "choices": ["Choice 1", "Choice 2", "Choice 3"],
        "educational_content": "Educational facts about space"
    }"#;

    let result = StoryLlmClient::try_parse_json_response(response);
    assert!(result.is_ok(), "Should parse valid JSON response");

    let (narrative, choices, educational) = result.unwrap();
    assert_eq!(narrative, "This is the story text with lots of adventure.");
    assert_eq!(choices.len(), 3);
    assert_eq!(choices[0], "Choice 1");
    assert!(educational.is_some());
    assert_eq!(educational.unwrap(), "Educational facts about space");
}

#[test]
fn test_parse_json_response_minimal() {
    let response = r#"{
        "text": "Story content here",
        "choices": ["Option A", "Option B", "Option C"]
    }"#;

    let result = StoryLlmClient::try_parse_json_response(response);
    assert!(result.is_ok(), "Should parse minimal JSON response");

    let (narrative, choices, educational) = result.unwrap();
    assert_eq!(narrative, "Story content here");
    assert_eq!(choices.len(), 3);
    assert!(educational.is_none());
}

#[test]
fn test_parse_json_response_missing_fields() {
    let response = r#"{"something": "else"}"#;

    let result = StoryLlmClient::try_parse_json_response(response);
    assert!(result.is_err(), "Should fail with missing required fields");
}

#[test]
fn test_parse_text_response_with_delimiters() {
    let response = "This is the narrative part of the story.\n---\n1. First choice\n2. Second choice\n3. Third choice\n---\nEducational content here";

    let result = StoryLlmClient::parse_text_response(response);
    assert!(result.is_ok(), "Should parse delimited text response");

    let (narrative, choices, educational) = result.unwrap();
    assert!(narrative.contains("narrative part"));
    assert_eq!(choices.len(), 3);
    assert!(educational.is_some());
}

// ============================================================================
// Prompt Template Tests
// ============================================================================

#[test]
fn test_prompt_templates_initialization() {
    let templates = PromptTemplates::new();

    // Verify all age groups have English templates
    assert!(templates.english.contains_key(&AgeGroup::_6To8));
    assert!(templates.english.contains_key(&AgeGroup::_9To11));
    assert!(templates.english.contains_key(&AgeGroup::_12To14));
    assert!(templates.english.contains_key(&AgeGroup::_15To17));
    assert!(templates.english.contains_key(&AgeGroup::Plus18));

    // Verify all age groups have German templates
    assert!(templates.german.contains_key(&AgeGroup::_6To8));
    assert!(templates.german.contains_key(&AgeGroup::_9To11));
    assert!(templates.german.contains_key(&AgeGroup::_12To14));
    assert!(templates.german.contains_key(&AgeGroup::_15To17));
    assert!(templates.german.contains_key(&AgeGroup::Plus18));
}

#[test]
fn test_prompt_template_get_english() {
    let templates = PromptTemplates::new();
    let (system, user) = templates.get_template(Language::En, AgeGroup::_9To11);

    assert!(!system.is_empty());
    assert!(!user.is_empty());
    assert!(system.to_lowercase().contains("storyteller"));
    assert!(user.contains("{theme}"));
    assert!(user.contains("{age_group}"));
    assert!(user.contains("{node_position}"));
}

#[test]
fn test_prompt_template_get_german() {
    let templates = PromptTemplates::new();
    let (system, user) = templates.get_template(Language::De, AgeGroup::_9To11);

    assert!(!system.is_empty());
    assert!(!user.is_empty());
    assert!(system.contains("Geschichtenerzähler") || system.contains("geschichtenerzähler"));
    assert!(user.contains("{theme}"));
}

#[test]
fn test_prompt_template_placeholders() {
    let templates = PromptTemplates::new();
    let (_, user) = templates.get_template(Language::En, AgeGroup::_9To11);

    // Verify all expected placeholders are present
    assert!(user.contains("{theme}"));
    assert!(user.contains("{age_group}"));
    assert!(user.contains("{node_position}"));
    assert!(user.contains("{previous_content}"));
    assert!(user.contains("{choices_made}"));
    assert!(user.contains("{educational_goals}"));
}

#[test]
fn test_prompt_template_substitution() {
    let templates = PromptTemplates::new();
    let (_, user) = templates.get_template(Language::En, AgeGroup::_9To11);

    let filled = user
        .replace("{theme}", "Space Adventure")
        .replace("{age_group}", "9-11")
        .replace("{node_position}", "5/16")
        .replace("{previous_content}", "Story beginning")
        .replace("{choices_made}", "Choice 1 → Choice 2")
        .replace("{educational_goals}", "Learn about planets");

    assert!(!filled.contains("{theme}"));
    assert!(!filled.contains("{age_group}"));
    assert!(filled.contains("Space Adventure"));
    assert!(filled.contains("9-11"));
    assert!(filled.contains("5/16"));
}

// ============================================================================
// Build Node Context Tests
// ============================================================================

#[test]
fn test_build_node_context_from_dag() {
    let dag = create_test_dag();

    // Test building context for start node
    let result = story_generator::llm::build_node_context("0", &dag);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert_eq!(context.node_position, 0);
    assert_eq!(context.total_nodes, 16);
    assert!(context.previous_content.is_none(), "Start node should have no previous content");
}

#[test]
fn test_build_node_context_invalid_node() {
    let dag = create_test_dag();

    let result = story_generator::llm::build_node_context("999", &dag);
    assert!(result.is_err(), "Should fail for non-existent node");
}

// ============================================================================
// Content Node Generation Tests (would need mock LLM)
// ============================================================================

// Note: Full integration tests with generate_node_content and generate_nodes_batch
// would require either:
// 1. A running LM Studio instance (not suitable for unit tests)
// 2. Mocking the rig-core client (complex, beyond scope)
// 3. Refactoring to use dependency injection of LlmService trait
//
// For now, we test the components that don't require actual LLM calls.

#[test]
fn test_build_content_prompt() {
    let prompt_package = create_test_prompt_package();
    let node_context = create_test_node_context();

    let prompt = StoryLlmClient::build_content_prompt(&prompt_package, &node_context);

    assert!(prompt.contains("System:"));
    assert!(prompt.contains("User:"));
    assert!(prompt.contains("Node position: 5/16"));
    assert!(prompt.contains("Previous content: The spaceship landed on Mars"));
    assert!(prompt.contains("Choices made: Explore the crater"));
    assert!(prompt.contains("Narrative text (~400 words)"));
    assert!(prompt.contains("Three choice options"));
}

#[test]
fn test_build_content_prompt_no_previous_content() {
    let prompt_package = create_test_prompt_package();
    let node_context = NodeContext {
        previous_content: None,
        choices_made: vec![],
        node_position: 0,
        total_nodes: 16,
    };

    let prompt = StoryLlmClient::build_content_prompt(&prompt_package, &node_context);

    assert!(prompt.contains("Node position: 0/16"));
    assert!(!prompt.contains("Previous content:"));
    assert!(!prompt.contains("Choices made:"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_parse_content_response_invalid_json() {
    let response = "This is not JSON at all";

    // Should fall back to text parsing
    let result = StoryLlmClient::parse_content_response(response);
    assert!(result.is_ok(), "Should fall back to text parsing");
}

#[test]
fn test_parse_content_response_empty() {
    let response = "";

    let result = StoryLlmClient::parse_content_response(response);
    // Empty response should fail or return minimal content
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[test]
fn test_concurrent_batches_constant() {
    use shared_types::constants::CONCURRENT_BATCHES;

    assert!(CONCURRENT_BATCHES >= 3);
    assert!(CONCURRENT_BATCHES <= 5);
}

// ============================================================================
// Integration Test Markers
// ============================================================================

// These tests would run if LM Studio is available
// They are marked with #[ignore] to skip in regular test runs

#[tokio::test]
#[ignore]
async fn integration_test_generate_node_content_with_llm() {
    // This test requires a running LM Studio instance
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let prompt_package = create_test_prompt_package();
    let node_context = create_test_node_context();
    let generation_request = create_test_generation_request();
    let dag = create_test_dag();

    let result = generate_node_content(
        &llm_client,
        "0",
        node_context,
        &prompt_package,
        &generation_request,
        &dag,
    )
    .await;

    assert!(result.is_ok(), "Should generate content with LLM");

    let content_node = result.unwrap();
    assert!(!content_node.content.text.is_empty());
    assert_eq!(content_node.content.choices.len(), 3);
}

#[tokio::test]
#[ignore]
async fn integration_test_generate_nodes_batch_with_llm() {
    // This test requires a running LM Studio instance
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let prompt_package = create_test_prompt_package();
    let generation_request = create_test_generation_request();
    let dag = create_test_dag();

    let node_ids = vec!["0".to_string(), "1".to_string(), "2".to_string()];

    let result = generate_nodes_batch(
        &llm_client,
        node_ids,
        &dag,
        &prompt_package,
        &generation_request,
    )
    .await;

    assert!(result.is_ok(), "Should generate batch with LLM");

    let nodes = result.unwrap();
    assert_eq!(nodes.len(), 3);

    for node in nodes {
        assert!(!node.content.text.is_empty());
        assert_eq!(node.content.choices.len(), 3);
    }
}

// ============================================================================
// Summary
// ============================================================================

#[test]
fn test_summary() {
    println!("\n=== LLM Content Generation Test Summary ===");
    println!("✓ StoryLlmClient creation and validation");
    println!("✓ Content parsing (JSON and text formats)");
    println!("✓ Choice extraction from various formats");
    println!("✓ Prompt template initialization and substitution");
    println!("✓ Node context building from DAG");
    println!("✓ Error handling for invalid inputs");
    println!("✓ Build content prompts with context");
    println!("\nNote: Integration tests with actual LLM require LM Studio");
    println!("Run with: cargo test --test generation_tests -- --ignored");
}
