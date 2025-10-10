//! MCP Tool Definitions for Prompt Helper Service
//!
//! This module defines the MCP tools exposed by the Prompt Helper service
//! using rmcp 0.8.0 types and schemars for JSON Schema generation.

use rmcp::model::Tool;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use shared_types::{AgeGroup, Language, VocabularyLevel};
use std::sync::Arc;

// ============================================================================
// Tool Parameter Structures
// ============================================================================

/// Parameters for generating story content prompts
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateStoryPromptsParams {
    /// Story theme (e.g., "Space Adventure", "Medieval Quest")
    pub theme: String,
    /// Target age group for content appropriateness
    pub age_group: AgeGroup,
    /// Content language
    pub language: Language,
    /// Educational goals to incorporate into the story
    pub educational_goals: Vec<String>,
}

/// Parameters for generating validation prompts
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateValidationPromptsParams {
    /// Target age group for validation criteria
    pub age_group: AgeGroup,
    /// Content language for validation
    pub language: Language,
    /// Type of content being validated (e.g., "story", "choice", "educational_fact")
    pub content_type: String,
}

/// Parameters for generating constraint enforcement prompts
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateConstraintPromptsParams {
    /// Vocabulary complexity level to enforce
    pub vocabulary_level: VocabularyLevel,
    /// Content language for constraints
    pub language: Language,
    /// Required story elements that must be present
    pub required_elements: Vec<String>,
}

/// Parameters for getting LLM model identifier
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetModelForLanguageParams {
    /// Language to get model for
    pub language: Language,
}

// ============================================================================
// Tool Creation Functions
// ============================================================================

/// Create the "generate_story_prompts" tool
///
/// This tool generates system and user prompts for story content generation,
/// tailored to the specified theme, age group, language, and educational goals.
#[allow(dead_code)]
pub fn create_generate_story_prompts_tool() -> Tool {
    let schema = schema_for!(GenerateStoryPromptsParams);
    let schema_value = serde_json::to_value(schema)
        .expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "generate_story_prompts".into(),
        description: Some("Generate system and user prompts for story content generation based on theme, age group, language, and educational goals.".into()),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "generate_validation_prompts" tool
///
/// This tool generates prompts for quality control validation,
/// ensuring content meets age-appropriateness, language quality,
/// and educational value standards.
#[allow(dead_code)]
pub fn create_generate_validation_prompts_tool() -> Tool {
    let schema = schema_for!(GenerateValidationPromptsParams);
    let schema_value = serde_json::to_value(schema)
        .expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "generate_validation_prompts".into(),
        description: Some("Generate prompts for quality control validation of generated content, including age-appropriateness checks, language quality verification, and educational value assessment.".into()),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "generate_constraint_prompts" tool
///
/// This tool generates prompts for constraint enforcement,
/// ensuring vocabulary levels, required story elements,
/// and theme consistency are maintained.
#[allow(dead_code)]
pub fn create_generate_constraint_prompts_tool() -> Tool {
    let schema = schema_for!(GenerateConstraintPromptsParams);
    let schema_value = serde_json::to_value(schema)
        .expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "generate_constraint_prompts".into(),
        description: Some("Generate prompts for constraint enforcement, including vocabulary level restrictions, required story element verification, and theme consistency checks.".into()),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "get_model_for_language" tool
///
/// This tool retrieves the appropriate LLM model identifier
/// for a specific language, ensuring language-specific models
/// are used for content generation.
#[allow(dead_code)]
pub fn create_get_model_for_language_tool() -> Tool {
    let schema = schema_for!(GetModelForLanguageParams);
    let schema_value = serde_json::to_value(schema)
        .expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "get_model_for_language".into(),
        description: Some("Get the appropriate LLM model identifier for a specific language, ensuring language-specific models are used for optimal content generation quality.".into()),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Get all tools provided by the Prompt Helper service
///
/// Returns a vector of all 4 tool definitions for registration
/// with the MCP server.
#[allow(dead_code)]
pub fn get_all_tools() -> Vec<Tool> {
    vec![
        create_generate_story_prompts_tool(),
        create_generate_validation_prompts_tool(),
        create_generate_constraint_prompts_tool(),
        create_get_model_for_language_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_have_names() {
        let tools = get_all_tools();
        assert_eq!(tools.len(), 4);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"generate_story_prompts"));
        assert!(names.contains(&"generate_validation_prompts"));
        assert!(names.contains(&"generate_constraint_prompts"));
        assert!(names.contains(&"get_model_for_language"));
    }

    #[test]
    fn test_all_tools_have_descriptions() {
        let tools = get_all_tools();
        for tool in tools {
            assert!(tool.description.is_some(), "Tool {} missing description", tool.name);
            assert!(!tool.description.unwrap().is_empty(), "Tool {} has empty description", tool.name);
        }
    }

    #[test]
    fn test_all_tools_have_input_schemas() {
        let tools = get_all_tools();
        for tool in tools {
            assert!(!tool.input_schema.is_empty(), "Tool {} missing input schema", tool.name);
        }
    }
}
