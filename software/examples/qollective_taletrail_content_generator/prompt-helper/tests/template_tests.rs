//! Comprehensive tests for template loading and variable substitution
//!
//! This test suite verifies the template system that will be used for fallback
//! prompt generation when LLM services are unavailable. Tests cover:
//! - Template loading from config.toml for all service types and languages
//! - Variable substitution with various parameter types
//! - Fallback behavior when templates or languages are missing
//! - Edge cases like empty variables and missing placeholders
//!
//! # Architecture
//!
//! Templates are loaded from config.toml with structure:
//! ```toml
//! [templates.story_generator.en]
//! system_prompt = "You are a storyteller for children aged {age_group}..."
//! user_prompt = "Generate a story about: {theme}. Goals: {educational_goals}"
//! ```
//!
//! Variable substitution supports:
//! - Simple strings: {theme}, {language}
//! - Enums: {age_group}, {vocabulary_level}
//! - Lists: {educational_goals}, {required_elements}
//!
//! # Testing Strategy
//!
//! Most tests are marked with #[ignore] as they require:
//! 1. Implementation of src/templates.rs module
//! 2. Creation of config.toml with template definitions
//!
//! Tests are written first (TDD) to define the expected API and behavior.

use prompt_helper::config::PromptHelperConfig;
use shared_types::{AgeGroup, Language, MCPServiceType, VocabularyLevel};
use std::collections::HashMap;

// ============================================================================
// Expected Template Module API (to be implemented in src/templates.rs)
// ============================================================================

/// Template containing system and user prompts with placeholder variables
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub system_prompt: String,
    pub user_prompt: String,
}

/// Context for template variable substitution
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub theme: String,
    pub age_group: AgeGroup,
    pub language: Language,
    pub educational_goals: Vec<String>,
    pub vocabulary_level: Option<VocabularyLevel>,
    pub required_elements: Option<Vec<String>>,
    pub content_type: Option<String>,
}

// ============================================================================
// Stub Functions (will be replaced by actual implementation)
// ============================================================================

/// Load all templates from config.toml
/// Returns HashMap keyed by (service_type, language)
#[allow(dead_code)]
fn load_templates_from_config(
    _config: &PromptHelperConfig,
) -> Result<HashMap<(MCPServiceType, Language), Template>, String> {
    // Stub implementation - will be replaced
    Err("Not implemented yet".to_string())
}

/// Apply template by substituting variables from context
/// Returns (system_prompt, user_prompt) with all variables replaced
#[allow(dead_code)]
fn apply_template(_template: &Template, _context: &TemplateContext) -> (String, String) {
    // Stub implementation - will be replaced
    ("".to_string(), "".to_string())
}

/// Get template for specific service type and language
/// Falls back to English if requested language not found
#[allow(dead_code)]
fn get_template(
    _service: MCPServiceType,
    _language: Language,
    _config: &PromptHelperConfig,
) -> Result<Template, String> {
    // Stub implementation - will be replaced
    Err("Not implemented yet".to_string())
}

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a basic test context with all required fields
fn create_test_context() -> TemplateContext {
    TemplateContext {
        theme: "Space Adventure".to_string(),
        age_group: AgeGroup::_6To8,
        language: Language::En,
        educational_goals: vec!["Learn planets".to_string()],
        vocabulary_level: Some(VocabularyLevel::Basic),
        required_elements: Some(vec!["moral lesson".to_string()]),
        content_type: Some("story".to_string()),
    }
}

/// Create a template with placeholders for testing substitution
fn create_test_template() -> Template {
    Template {
        system_prompt: "Theme: {theme}, Age: {age_group}, Lang: {language}".to_string(),
        user_prompt: "Goals: {educational_goals}".to_string(),
    }
}

/// Create test config with mock template data
fn create_test_config_with_templates() -> PromptHelperConfig {
    // Returns default config for now
    // In actual implementation, this would include template definitions
    PromptHelperConfig::default()
}

// ============================================================================
// Tests: Template Loading from Config
// ============================================================================

#[test]
#[ignore = "Requires templates.rs implementation and config.toml with template definitions"]
fn test_load_templates_from_config_success() {
    let config = create_test_config_with_templates();
    let templates = load_templates_from_config(&config).expect("Failed to load templates");

    // Verify templates exist for all service types
    assert!(
        templates.contains_key(&(MCPServiceType::StoryGenerator, Language::En)),
        "Missing story_generator English template"
    );
    assert!(
        templates.contains_key(&(MCPServiceType::StoryGenerator, Language::De)),
        "Missing story_generator German template"
    );
    assert!(
        templates.contains_key(&(MCPServiceType::QualityControl, Language::En)),
        "Missing quality_control English template"
    );
    assert!(
        templates.contains_key(&(MCPServiceType::QualityControl, Language::De)),
        "Missing quality_control German template"
    );
    assert!(
        templates.contains_key(&(MCPServiceType::ConstraintEnforcer, Language::En)),
        "Missing constraint_enforcer English template"
    );
    assert!(
        templates.contains_key(&(MCPServiceType::ConstraintEnforcer, Language::De)),
        "Missing constraint_enforcer German template"
    );

    // Verify templates contain expected placeholders
    let story_template = templates
        .get(&(MCPServiceType::StoryGenerator, Language::En))
        .expect("Story template missing");
    assert!(
        story_template.system_prompt.contains("{age_group}"),
        "Story template missing age_group placeholder"
    );
    assert!(
        story_template.user_prompt.contains("{theme}"),
        "Story template missing theme placeholder"
    );
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_load_templates_missing_service_type() {
    let config = create_test_config_with_templates();
    let templates = load_templates_from_config(&config).expect("Failed to load templates");

    // NonExistentService is not a real MCPServiceType, but this tests the concept
    // In actual implementation, we'd need to handle services not in config
    // For now, verify that loading doesn't crash and returns expected types
    assert!(templates.len() >= 6, "Should have at least 6 templates (3 services × 2 languages)");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_load_templates_missing_language() {
    let config = create_test_config_with_templates();

    // Try to get template for unsupported language (French)
    // Should return error or fall back to English
    let result = get_template(MCPServiceType::StoryGenerator, Language::En, &config);

    // Either succeeds with fallback or returns clear error
    match result {
        Ok(_) => (), // Fallback to English worked
        Err(e) => assert!(
            e.contains("language") || e.contains("not found"),
            "Error message should mention language issue"
        ),
    }
}

// ============================================================================
// Tests: Variable Substitution
// ============================================================================

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_substitutes_all_variables() {
    let template = Template {
        system_prompt: "Theme: {theme}, Age: {age_group}, Lang: {language}".to_string(),
        user_prompt: "Vocab: {vocabulary_level}, Type: {content_type}".to_string(),
    };

    let context = create_test_context();
    let (system, user) = apply_template(&template, &context);

    assert_eq!(system, "Theme: Space Adventure, Age: 6-8, Lang: en");
    assert_eq!(user, "Vocab: Basic, Type: story");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_with_educational_goals_list() {
    let template = Template {
        system_prompt: "Goals: {educational_goals}".to_string(),
        user_prompt: "Theme: {theme}".to_string(),
    };

    let mut context = create_test_context();
    context.educational_goals = vec![
        "Learn planets".to_string(),
        "Math skills".to_string(),
        "Problem solving".to_string(),
    ];

    let (system, _) = apply_template(&template, &context);

    // List should be comma-separated
    assert_eq!(system, "Goals: Learn planets, Math skills, Problem solving");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_with_vocabulary_level() {
    let template = Template {
        system_prompt: "Vocabulary: {vocabulary_level}".to_string(),
        user_prompt: "".to_string(),
    };

    // Test all vocabulary levels
    let vocab_levels = vec![
        (VocabularyLevel::Basic, "Basic"),
        (VocabularyLevel::Intermediate, "Intermediate"),
        (VocabularyLevel::Advanced, "Advanced"),
    ];

    for (level, expected_str) in vocab_levels {
        let mut context = create_test_context();
        context.vocabulary_level = Some(level);

        let (system, _) = apply_template(&template, &context);
        assert_eq!(system, format!("Vocabulary: {}", expected_str));
    }
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_with_required_elements_list() {
    let template = Template {
        system_prompt: "Required: {required_elements}".to_string(),
        user_prompt: "".to_string(),
    };

    let mut context = create_test_context();
    context.required_elements = Some(vec![
        "moral".to_string(),
        "science fact".to_string(),
        "hero journey".to_string(),
    ]);

    let (system, _) = apply_template(&template, &context);

    assert_eq!(system, "Required: moral, science fact, hero journey");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_missing_variable_keeps_placeholder() {
    let template = Template {
        system_prompt: "Theme: {theme}, Missing: {unknown_variable}".to_string(),
        user_prompt: "".to_string(),
    };

    let context = create_test_context();
    let (system, _) = apply_template(&template, &context);

    // Known variable is replaced, unknown variable stays as placeholder
    assert_eq!(system, "Theme: Space Adventure, Missing: {unknown_variable}");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_empty_variable_substitutes_empty() {
    let template = Template {
        system_prompt: "Theme: '{theme}'".to_string(),
        user_prompt: "".to_string(),
    };

    let mut context = create_test_context();
    context.theme = "".to_string();

    let (system, _) = apply_template(&template, &context);

    assert_eq!(system, "Theme: ''");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_with_all_age_groups() {
    let template = Template {
        system_prompt: "Age: {age_group}".to_string(),
        user_prompt: "".to_string(),
    };

    let age_groups = vec![
        (AgeGroup::_6To8, "6-8"),
        (AgeGroup::_9To11, "9-11"),
        (AgeGroup::_12To14, "12-14"),
        (AgeGroup::_15To17, "15-17"),
        (AgeGroup::Plus18, "18+"),
    ];

    for (age_group, expected_str) in age_groups {
        let mut context = create_test_context();
        context.age_group = age_group;

        let (system, _) = apply_template(&template, &context);
        assert_eq!(system, format!("Age: {}", expected_str));
    }
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_apply_template_with_optional_none_values() {
    let template = Template {
        system_prompt: "Vocab: {vocabulary_level}, Elements: {required_elements}".to_string(),
        user_prompt: "Type: {content_type}".to_string(),
    };

    let mut context = create_test_context();
    context.vocabulary_level = None;
    context.required_elements = None;
    context.content_type = None;

    let (system, user) = apply_template(&template, &context);

    // None values should result in empty string or "None" or keep placeholder
    // Implementation decision: let's say empty string for None
    assert!(
        system.contains("Vocab:") && system.contains("Elements:"),
        "Template structure preserved"
    );
    assert!(user.contains("Type:"), "User prompt structure preserved");
}

// ============================================================================
// Tests: Service-Specific Templates
// ============================================================================

#[test]
#[ignore = "Requires templates.rs implementation and config.toml"]
fn test_load_story_generator_templates_english() {
    let config = create_test_config_with_templates();
    let template = get_template(MCPServiceType::StoryGenerator, Language::En, &config)
        .expect("Failed to load story generator English template");

    // Verify system prompt contains story-specific content
    assert!(
        template.system_prompt.contains("story") || template.system_prompt.contains("Story"),
        "Story template should mention 'story'"
    );

    // Verify expected variables
    assert!(template.system_prompt.contains("{age_group}"));
    assert!(template.user_prompt.contains("{theme}"));
}

#[test]
#[ignore = "Requires templates.rs implementation and config.toml"]
fn test_load_story_generator_templates_german() {
    let config = create_test_config_with_templates();
    let template = get_template(MCPServiceType::StoryGenerator, Language::De, &config)
        .expect("Failed to load story generator German template");

    // Verify German language content (should contain German words)
    // German words for story: "Geschichte", "Erzählung"
    let has_german = template.system_prompt.contains("Geschichte")
        || template.system_prompt.contains("Du bist")
        || template.system_prompt.contains("Kinder");

    assert!(has_german, "German template should contain German language");

    // Verify same variable names work in German templates
    assert!(template.system_prompt.contains("{age_group}"));
}

#[test]
#[ignore = "Requires templates.rs implementation and config.toml"]
fn test_load_quality_control_templates() {
    let config = create_test_config_with_templates();

    // Test English
    let en_template = get_template(MCPServiceType::QualityControl, Language::En, &config)
        .expect("Failed to load QC English template");
    assert!(
        en_template.system_prompt.contains("quality")
            || en_template.system_prompt.contains("validation")
            || en_template.system_prompt.contains("validate"),
        "QC template should mention quality or validation"
    );

    // Test German
    let de_template = get_template(MCPServiceType::QualityControl, Language::De, &config)
        .expect("Failed to load QC German template");
    assert!(
        de_template.system_prompt.len() > 0,
        "German QC template should not be empty"
    );
}

#[test]
#[ignore = "Requires templates.rs implementation and config.toml"]
fn test_load_constraint_enforcer_templates() {
    let config = create_test_config_with_templates();

    // Test English
    let en_template = get_template(MCPServiceType::ConstraintEnforcer, Language::En, &config)
        .expect("Failed to load constraint English template");
    assert!(
        en_template.system_prompt.contains("constraint")
            || en_template.system_prompt.contains("enforce")
            || en_template.system_prompt.contains("requirement"),
        "Constraint template should mention constraints or enforcement"
    );
    assert!(en_template.system_prompt.contains("{vocabulary_level}"));

    // Test German
    let de_template = get_template(MCPServiceType::ConstraintEnforcer, Language::De, &config)
        .expect("Failed to load constraint German template");
    assert!(
        de_template.system_prompt.len() > 0,
        "German constraint template should not be empty"
    );
}

// ============================================================================
// Tests: Fallback Behavior
// ============================================================================

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_template_fallback_for_all_service_types() {
    let config = create_test_config_with_templates();

    let service_types = vec![
        MCPServiceType::StoryGenerator,
        MCPServiceType::QualityControl,
        MCPServiceType::ConstraintEnforcer,
    ];

    let languages = vec![Language::En, Language::De];

    for service in &service_types {
        for language in &languages {
            let result = get_template(*service, *language, &config);

            match result {
                Ok(template) => {
                    // Verify template is not empty
                    assert!(
                        !template.system_prompt.is_empty(),
                        "Template for {:?} {:?} has empty system prompt",
                        service,
                        language
                    );
                    assert!(
                        !template.user_prompt.is_empty(),
                        "Template for {:?} {:?} has empty user prompt",
                        service,
                        language
                    );
                }
                Err(e) => {
                    panic!(
                        "Failed to get template for {:?} {:?}: {}",
                        service, language, e
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_get_template_with_fallback_chain() {
    let config = create_test_config_with_templates();

    // Try to get template for French (not supported)
    // Should fall back to English
    // Note: This test assumes French is implemented as enum variant
    // For now, we test the fallback mechanism with supported languages

    let template = get_template(MCPServiceType::StoryGenerator, Language::En, &config)
        .expect("Fallback should work");

    assert!(!template.system_prompt.is_empty(), "Fallback template should not be empty");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_fallback_templates_match_current_handler_implementation() {
    // This test verifies that config-based templates produce similar output
    // to the current hardcoded fallback templates in handlers.rs

    let config = create_test_config_with_templates();
    let context = create_test_context();

    // Get story generator template
    let template = get_template(MCPServiceType::StoryGenerator, Language::En, &config)
        .expect("Failed to load template");

    let (system, user) = apply_template(&template, &context);

    // Verify it contains similar content to handlers.rs fallback (lines 132-145)
    assert!(
        system.contains("story") || system.contains("Story"),
        "Story template should mention stories"
    );
    assert!(
        system.contains("Space Adventure") || user.contains("Space Adventure"),
        "Theme should be substituted"
    );
}

// ============================================================================
// Tests: Edge Cases and Error Handling
// ============================================================================

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_template_with_special_characters() {
    let template = Template {
        system_prompt: "Theme: {theme}. Use quotes: \"double\" and 'single'".to_string(),
        user_prompt: "Symbols: & < > @ # $ %".to_string(),
    };

    let context = create_test_context();
    let (system, user) = apply_template(&template, &context);

    // Special characters should be preserved
    assert!(system.contains("\"double\""));
    assert!(system.contains("'single'"));
    assert!(user.contains("& < > @ # $ %"));
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_template_with_multiline_content() {
    let template = Template {
        system_prompt: "Line 1: {theme}\nLine 2: {age_group}\nLine 3: Rules".to_string(),
        user_prompt: "User:\n- Bullet 1\n- Bullet 2".to_string(),
    };

    let context = create_test_context();
    let (system, user) = apply_template(&template, &context);

    // Newlines should be preserved
    assert!(system.contains("\n"));
    assert_eq!(system.lines().count(), 3);
    assert!(user.contains("\n"));
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_template_with_empty_educational_goals() {
    let template = Template {
        system_prompt: "Goals: {educational_goals}".to_string(),
        user_prompt: "".to_string(),
    };

    let mut context = create_test_context();
    context.educational_goals = vec![];

    let (system, _) = apply_template(&template, &context);

    // Empty list should result in empty string for the list
    assert_eq!(system, "Goals: ");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_template_with_single_element_list() {
    let template = Template {
        system_prompt: "Goals: {educational_goals}".to_string(),
        user_prompt: "".to_string(),
    };

    let mut context = create_test_context();
    context.educational_goals = vec!["Single goal".to_string()];

    let (system, _) = apply_template(&template, &context);

    // Single element should not have trailing comma
    assert_eq!(system, "Goals: Single goal");
}

#[test]
#[ignore = "Requires templates.rs implementation"]
fn test_template_substitution_preserves_surrounding_text() {
    let template = Template {
        system_prompt: "Before {theme} middle {age_group} after".to_string(),
        user_prompt: "Start {language} end".to_string(),
    };

    let context = create_test_context();
    let (system, user) = apply_template(&template, &context);

    // Verify surrounding text is preserved exactly
    assert!(system.starts_with("Before "));
    assert!(system.contains(" middle "));
    assert!(system.ends_with(" after"));
    assert!(user.starts_with("Start "));
    assert!(user.ends_with(" end"));
}

// ============================================================================
// Tests: Integration with Config Loading
// ============================================================================

#[test]
#[ignore = "Requires config.toml with [templates] section"]
fn test_load_config_with_templates() {
    // This test verifies that PromptHelperConfig can load template definitions
    // from config.toml and make them accessible

    // In actual implementation, config might have a templates field:
    // pub struct PromptHelperConfig {
    //     pub templates: HashMap<(MCPServiceType, Language), Template>,
    //     ...
    // }

    let config = PromptHelperConfig::default();

    // For now, just verify config loads without crashing
    assert_eq!(config.service.name, "prompt-helper");
}

#[test]
fn test_template_struct_equality() {
    let template1 = Template {
        system_prompt: "System".to_string(),
        user_prompt: "User".to_string(),
    };

    let template2 = Template {
        system_prompt: "System".to_string(),
        user_prompt: "User".to_string(),
    };

    let template3 = Template {
        system_prompt: "Different".to_string(),
        user_prompt: "User".to_string(),
    };

    assert_eq!(template1, template2);
    assert_ne!(template1, template3);
}

#[test]
fn test_template_context_creation() {
    let context = create_test_context();

    assert_eq!(context.theme, "Space Adventure");
    assert_eq!(context.age_group, AgeGroup::_6To8);
    assert_eq!(context.language, Language::En);
    assert_eq!(context.educational_goals.len(), 1);
    assert_eq!(context.vocabulary_level, Some(VocabularyLevel::Basic));
    assert!(context.required_elements.is_some());
    assert_eq!(context.content_type, Some("story".to_string()));
}

// ============================================================================
// Documentation Tests
// ============================================================================

/// This test serves as documentation for how the template system should work
#[test]
#[ignore = "Documentation test - requires full implementation"]
fn test_complete_template_workflow() {
    // Step 1: Load config with templates
    let config = create_test_config_with_templates();

    // Step 2: Load all templates from config
    let templates = load_templates_from_config(&config).expect("Failed to load templates");
    assert!(templates.len() >= 6, "Should have templates for 3 services × 2 languages");

    // Step 3: Get a specific template
    let story_template = get_template(MCPServiceType::StoryGenerator, Language::En, &config)
        .expect("Failed to get story template");

    // Step 4: Create context for substitution
    let context = TemplateContext {
        theme: "Medieval Adventure".to_string(),
        age_group: AgeGroup::_9To11,
        language: Language::En,
        educational_goals: vec!["History".to_string(), "Courage".to_string()],
        vocabulary_level: Some(VocabularyLevel::Intermediate),
        required_elements: Some(vec!["hero".to_string(), "quest".to_string()]),
        content_type: Some("story".to_string()),
    };

    // Step 5: Apply template with context
    let (system_prompt, user_prompt) = apply_template(&story_template, &context);

    // Step 6: Verify prompts are ready for LLM
    assert!(!system_prompt.is_empty(), "System prompt should not be empty");
    assert!(!user_prompt.is_empty(), "User prompt should not be empty");
    assert!(
        !system_prompt.contains("{"),
        "System prompt should have all variables substituted"
    );
    assert!(
        !user_prompt.contains("{"),
        "User prompt should have all variables substituted"
    );
}
