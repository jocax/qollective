//! Comprehensive tests for MCP tool definitions
//!
//! These tests verify that all tool definitions are correct,
//! have valid schemas, and properly validate input parameters.

use prompt_helper::mcp_tools::*;
use serde_json::json;
use shared_types::{AgeGroup, Language, VocabularyLevel};

// ============================================================================
// Tool Name and Description Tests
// ============================================================================

#[test]
fn test_generate_story_prompts_tool_definition() {
    let tool = create_generate_story_prompts_tool();

    assert_eq!(tool.name, "generate_story_prompts");
    assert!(tool.description.is_some());
    assert!(tool.description.as_ref().unwrap().contains("story content generation"));
    assert!(!tool.input_schema.is_empty());
}

#[test]
fn test_generate_validation_prompts_tool_definition() {
    let tool = create_generate_validation_prompts_tool();

    assert_eq!(tool.name, "generate_validation_prompts");
    assert!(tool.description.is_some());
    assert!(tool.description.as_ref().unwrap().contains("quality control validation"));
    assert!(!tool.input_schema.is_empty());
}

#[test]
fn test_generate_constraint_prompts_tool_definition() {
    let tool = create_generate_constraint_prompts_tool();

    assert_eq!(tool.name, "generate_constraint_prompts");
    assert!(tool.description.is_some());
    assert!(tool.description.as_ref().unwrap().contains("constraint enforcement"));
    assert!(!tool.input_schema.is_empty());
}

#[test]
fn test_get_model_for_language_tool_definition() {
    let tool = create_get_model_for_language_tool();

    assert_eq!(tool.name, "get_model_for_language");
    assert!(tool.description.is_some());
    assert!(tool.description.as_ref().unwrap().contains("LLM model identifier"));
    assert!(!tool.input_schema.is_empty());
}

// ============================================================================
// Input Schema Validation Tests
// ============================================================================

#[test]
fn test_generate_story_prompts_schema_structure() {
    let tool = create_generate_story_prompts_tool();
    let schema = &tool.input_schema;

    // Verify schema contains required properties
    assert!(schema.contains_key("properties") || schema.contains_key("$ref"));

    // Schema should be valid JSON Schema
    let schema_json = serde_json::to_value(schema.as_ref()).unwrap();
    assert!(schema_json.is_object());
}

#[test]
fn test_generate_validation_prompts_schema_structure() {
    let tool = create_generate_validation_prompts_tool();
    let schema = &tool.input_schema;

    // Verify schema contains required properties
    assert!(schema.contains_key("properties") || schema.contains_key("$ref"));

    // Schema should be valid JSON Schema
    let schema_json = serde_json::to_value(schema.as_ref()).unwrap();
    assert!(schema_json.is_object());
}

#[test]
fn test_generate_constraint_prompts_schema_structure() {
    let tool = create_generate_constraint_prompts_tool();
    let schema = &tool.input_schema;

    // Verify schema contains required properties
    assert!(schema.contains_key("properties") || schema.contains_key("$ref"));

    // Schema should be valid JSON Schema
    let schema_json = serde_json::to_value(schema.as_ref()).unwrap();
    assert!(schema_json.is_object());
}

#[test]
fn test_get_model_for_language_schema_structure() {
    let tool = create_get_model_for_language_tool();
    let schema = &tool.input_schema;

    // Verify schema contains required properties
    assert!(schema.contains_key("properties") || schema.contains_key("$ref"));

    // Schema should be valid JSON Schema
    let schema_json = serde_json::to_value(schema.as_ref()).unwrap();
    assert!(schema_json.is_object());
}

// ============================================================================
// Parameter Serialization Tests
// ============================================================================

#[test]
fn test_generate_story_prompts_params_serialization() {
    let params = GenerateStoryPromptsParams {
        theme: "Space Adventure".to_string(),
        age_group: AgeGroup::_9To11,
        language: Language::En,
        educational_goals: vec![
            "Learn about planets".to_string(),
            "Understand gravity".to_string(),
        ],
        required_elements: None,
        node_choice_counts: None,
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["theme"], "Space Adventure");
    assert_eq!(json["age_group"], "9-11");
    assert_eq!(json["language"], "en");
    assert!(json["educational_goals"].is_array());
    assert_eq!(json["educational_goals"].as_array().unwrap().len(), 2);
}

#[test]
fn test_generate_validation_prompts_params_serialization() {
    let params = GenerateValidationPromptsParams {
        age_group: AgeGroup::_12To14,
        language: Language::De,
        content_type: "story".to_string(),
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["age_group"], "12-14");
    assert_eq!(json["language"], "de");
    assert_eq!(json["content_type"], "story");
}

#[test]
fn test_generate_constraint_prompts_params_serialization() {
    let params = GenerateConstraintPromptsParams {
        vocabulary_level: VocabularyLevel::Intermediate,
        language: Language::En,
        required_elements: vec![
            "hero character".to_string(),
            "conflict".to_string(),
            "resolution".to_string(),
        ],
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["vocabulary_level"], "intermediate");
    assert_eq!(json["language"], "en");
    assert!(json["required_elements"].is_array());
    assert_eq!(json["required_elements"].as_array().unwrap().len(), 3);
}

#[test]
fn test_get_model_for_language_params_serialization() {
    let params = GetModelForLanguageParams {
        language: Language::De,
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["language"], "de");
}

// ============================================================================
// Parameter Deserialization Tests
// ============================================================================

#[test]
fn test_generate_story_prompts_params_deserialization() {
    let json = json!({
        "theme": "Medieval Quest",
        "age_group": "6-8",
        "language": "en",
        "educational_goals": ["Learn about castles", "Understand medieval life"]
    });

    let params: GenerateStoryPromptsParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.theme, "Medieval Quest");
    assert_eq!(params.age_group, AgeGroup::_6To8);
    assert_eq!(params.language, Language::En);
    assert_eq!(params.educational_goals.len(), 2);
}

#[test]
fn test_generate_validation_prompts_params_deserialization() {
    let json = json!({
        "age_group": "15-17",
        "language": "de",
        "content_type": "choice"
    });

    let params: GenerateValidationPromptsParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.age_group, AgeGroup::_15To17);
    assert_eq!(params.language, Language::De);
    assert_eq!(params.content_type, "choice");
}

#[test]
fn test_generate_constraint_prompts_params_deserialization() {
    let json = json!({
        "vocabulary_level": "advanced",
        "language": "en",
        "required_elements": ["protagonist", "antagonist", "climax"]
    });

    let params: GenerateConstraintPromptsParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.vocabulary_level, VocabularyLevel::Advanced);
    assert_eq!(params.language, Language::En);
    assert_eq!(params.required_elements.len(), 3);
}

#[test]
fn test_get_model_for_language_params_deserialization() {
    let json = json!({
        "language": "en"
    });

    let params: GetModelForLanguageParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.language, Language::En);
}

// ============================================================================
// All Tools Collection Tests
// ============================================================================

#[test]
fn test_get_all_tools_count() {
    let tools = get_all_tools();
    assert_eq!(tools.len(), 4, "Expected exactly 4 tools");
}

#[test]
fn test_get_all_tools_unique_names() {
    let tools = get_all_tools();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();

    // Check for duplicates
    for name in &names {
        let count = names.iter().filter(|&n| n == name).count();
        assert_eq!(count, 1, "Duplicate tool name found: {}", name);
    }
}

#[test]
fn test_get_all_tools_expected_names() {
    let tools = get_all_tools();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();

    assert!(names.contains(&"generate_story_prompts"));
    assert!(names.contains(&"generate_validation_prompts"));
    assert!(names.contains(&"generate_constraint_prompts"));
    assert!(names.contains(&"get_model_for_language"));
}

#[test]
fn test_all_tools_have_non_empty_descriptions() {
    let tools = get_all_tools();

    for tool in tools {
        assert!(
            tool.description.is_some(),
            "Tool '{}' is missing description",
            tool.name
        );
        assert!(
            !tool.description.as_ref().unwrap().is_empty(),
            "Tool '{}' has empty description",
            tool.name
        );
    }
}

#[test]
fn test_all_tools_have_valid_input_schemas() {
    let tools = get_all_tools();

    for tool in tools {
        assert!(
            !tool.input_schema.is_empty(),
            "Tool '{}' has empty input schema",
            tool.name
        );

        // Verify schema can be serialized to JSON
        let schema_json = serde_json::to_value(tool.input_schema.as_ref()).unwrap();
        assert!(
            schema_json.is_object(),
            "Tool '{}' input schema is not a valid JSON object",
            tool.name
        );
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_educational_goals_serialization() {
    let params = GenerateStoryPromptsParams {
        theme: "Simple Story".to_string(),
        age_group: AgeGroup::_6To8,
        language: Language::En,
        educational_goals: vec![],
        required_elements: None,
        node_choice_counts: None,
    };

    let json = serde_json::to_value(&params).unwrap();
    assert!(json["educational_goals"].is_array());
    assert_eq!(json["educational_goals"].as_array().unwrap().len(), 0);
}

#[test]
fn test_empty_required_elements_serialization() {
    let params = GenerateConstraintPromptsParams {
        vocabulary_level: VocabularyLevel::Basic,
        language: Language::En,
        required_elements: vec![],
    };

    let json = serde_json::to_value(&params).unwrap();
    assert!(json["required_elements"].is_array());
    assert_eq!(json["required_elements"].as_array().unwrap().len(), 0);
}

#[test]
fn test_all_age_groups_serialization() {
    let age_groups = vec![
        AgeGroup::_6To8,
        AgeGroup::_9To11,
        AgeGroup::_12To14,
        AgeGroup::_15To17,
        AgeGroup::Plus18,
    ];

    for age_group in age_groups {
        let params = GenerateValidationPromptsParams {
            age_group,
            language: Language::En,
            content_type: "test".to_string(),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert!(json["age_group"].is_string());
    }
}

#[test]
fn test_all_languages_serialization() {
    let languages = vec![Language::En, Language::De];

    for language in languages {
        let params = GetModelForLanguageParams { language };

        let json = serde_json::to_value(&params).unwrap();
        assert!(json["language"].is_string());
    }
}

#[test]
fn test_all_vocabulary_levels_serialization() {
    let vocab_levels = vec![
        VocabularyLevel::Basic,
        VocabularyLevel::Intermediate,
        VocabularyLevel::Advanced,
    ];

    for vocab_level in vocab_levels {
        let params = GenerateConstraintPromptsParams {
            vocabulary_level: vocab_level,
            language: Language::En,
            required_elements: vec![],
        };

        let json = serde_json::to_value(&params).unwrap();
        assert!(json["vocabulary_level"].is_string());
    }
}
