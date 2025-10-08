//! Template-based prompt generation with config loading and variable substitution
//!
//! This module provides a config-based template system for generating LLM prompts with
//! variable substitution support. It enables:
//!
//! - Loading templates from config.toml for all service types and languages
//! - Variable substitution with context values (theme, age_group, educational_goals, etc.)
//! - Hardcoded fallback templates when config is unavailable
//! - Support for multiple languages (English, German)
//! - Type-safe template retrieval by service type and language
//!
//! # Architecture
//!
//! Templates are structured in config.toml as:
//! ```toml
//! [templates.story_generator.en]
//! system_prompt = "You are a storyteller for children aged {age_group}..."
//! user_prompt = "Generate a story about: {theme}. Goals: {educational_goals}"
//! ```
//!
//! # Variable Substitution
//!
//! Supported variables:
//! - `{theme}` → context.theme
//! - `{age_group}` → format!("{:?}", context.age_group) → "6-8", "9-11", "12-14", "15-17", "18+"
//! - `{language}` → format!("{:?}", context.language) → "En", "De"
//! - `{educational_goals}` → context.educational_goals.join(", ")
//! - `{vocabulary_level}` → format!("{:?}", context.vocabulary_level.unwrap_or_default())
//! - `{required_elements}` → context.required_elements.unwrap_or_default().join(", ")
//! - `{content_type}` → context.content_type.unwrap_or_default()
//!
//! # Fallback Strategy
//!
//! 1. Try to load templates from config.toml
//! 2. If config loading fails, use hardcoded fallback templates
//! 3. If requested language not available, fallback to English
//! 4. If service type not found, return error
//!
//! # Example Usage
//!
//! ```rust
//! use prompt_helper::templates::{get_template, apply_template, TemplateContext};
//! use prompt_helper::config::PromptHelperConfig;
//! use shared_types::{MCPServiceType, Language, AgeGroup, VocabularyLevel};
//!
//! let config = PromptHelperConfig::default();
//! let template = get_template(
//!     MCPServiceType::StoryGenerator,
//!     Language::En,
//!     &config
//! ).expect("Failed to get template");
//!
//! let context = TemplateContext {
//!     theme: "Space Adventure".to_string(),
//!     age_group: AgeGroup::_6To8,
//!     language: Language::En,
//!     educational_goals: vec!["Learn planets".to_string()],
//!     vocabulary_level: Some(VocabularyLevel::Basic),
//!     required_elements: Some(vec!["moral lesson".to_string()]),
//!     content_type: Some("story".to_string()),
//! };
//!
//! let (system_prompt, user_prompt) = apply_template(&template, &context);
//! ```

use shared_types::{AgeGroup, Language, MCPServiceType, VocabularyLevel};
use crate::config::PromptHelperConfig;
use std::collections::HashMap;
use tracing::{debug, warn, info};
use serde::{Deserialize, Serialize};

// ============================================================================
// Public API Types
// ============================================================================

/// Template containing system and user prompts with placeholder variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
// Public API Functions
// ============================================================================

/// Load all templates from config.toml
///
/// Returns HashMap keyed by (service_type, language) for fast lookup.
/// If config loading fails, returns error and caller should use fallback templates.
///
/// # Arguments
///
/// * `config` - The PromptHelperConfig instance (may be extended to include templates)
///
/// # Returns
///
/// * `Ok(HashMap)` - Successfully loaded templates from config
/// * `Err(String)` - Config doesn't have templates section, use fallback
pub fn load_templates_from_config(
    _config: &PromptHelperConfig,
) -> Result<HashMap<(MCPServiceType, Language), Template>, String> {
    // TODO: When config.toml has [templates] section, load from config
    // For now, return error to trigger fallback behavior
    warn!("Templates not yet configured in config.toml, using hardcoded fallbacks");
    Err("Templates not configured in config.toml".to_string())
}

/// Apply template by substituting variables from context
///
/// Replaces all template variables with corresponding context values.
/// Unknown variables are kept as placeholders.
///
/// # Arguments
///
/// * `template` - Template with placeholder variables
/// * `context` - Context containing values for substitution
///
/// # Returns
///
/// * `(String, String)` - Tuple of (system_prompt, user_prompt) with variables replaced
pub fn apply_template(template: &Template, context: &TemplateContext) -> (String, String) {
    let system = substitute_variables(&template.system_prompt, context);
    let user = substitute_variables(&template.user_prompt, context);

    debug!(
        "Applied template: system_len={}, user_len={}, theme={}, age_group={:?}",
        system.len(),
        user.len(),
        context.theme,
        context.age_group
    );

    (system, user)
}

/// Get template for specific service type and language
///
/// First attempts to load from config, then falls back to hardcoded templates.
/// If requested language not available, falls back to English.
///
/// # Arguments
///
/// * `service` - The MCP service type (StoryGenerator, QualityControl, ConstraintEnforcer)
/// * `language` - Requested language (En, De)
/// * `config` - Configuration instance
///
/// # Returns
///
/// * `Ok(Template)` - Template for the requested service and language
/// * `Err(String)` - Service type not supported
pub fn get_template(
    service: MCPServiceType,
    language: Language,
    config: &PromptHelperConfig,
) -> Result<Template, String> {
    // Try to load from config first
    if let Ok(templates) = load_templates_from_config(config) {
        // Try exact match
        if let Some(template) = templates.get(&(service, language)) {
            info!("Loaded template from config: service={:?}, language={:?}", service, language);
            return Ok(template.clone());
        }

        // Fallback to English if requested language not found
        if language != Language::En {
            if let Some(template) = templates.get(&(service, Language::En)) {
                warn!(
                    "Language {:?} not found for service {:?}, falling back to English",
                    language, service
                );
                return Ok(template.clone());
            }
        }
    }

    // Config loading failed or template not in config, use hardcoded fallback
    debug!("Using hardcoded fallback template: service={:?}, language={:?}", service, language);
    let fallback = get_fallback_template(service, language);
    Ok(fallback)
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Substitute all variables in template string with context values
fn substitute_variables(template_str: &str, context: &TemplateContext) -> String {
    let mut result = template_str.to_string();

    // Simple string substitutions
    result = result.replace("{theme}", &context.theme);
    result = result.replace("{language}", &format_language(&context.language));

    // Enum substitutions
    result = result.replace("{age_group}", &format_age_group(&context.age_group));

    // List substitutions
    result = result.replace("{educational_goals}", &format_list(&context.educational_goals));

    // Optional substitutions
    if let Some(vocab_level) = &context.vocabulary_level {
        result = result.replace("{vocabulary_level}", &format_vocabulary_level(vocab_level));
    } else {
        result = result.replace("{vocabulary_level}", "");
    }

    if let Some(elements) = &context.required_elements {
        result = result.replace("{required_elements}", &format_list(elements));
    } else {
        result = result.replace("{required_elements}", "");
    }

    if let Some(content_type) = &context.content_type {
        result = result.replace("{content_type}", content_type);
    } else {
        result = result.replace("{content_type}", "");
    }

    result
}

/// Format age group as human-readable string
fn format_age_group(age_group: &AgeGroup) -> String {
    match age_group {
        AgeGroup::_6To8 => "6-8".to_string(),
        AgeGroup::_9To11 => "9-11".to_string(),
        AgeGroup::_12To14 => "12-14".to_string(),
        AgeGroup::_15To17 => "15-17".to_string(),
        AgeGroup::Plus18 => "18+".to_string(),
    }
}

/// Format language as two-letter code
fn format_language(language: &Language) -> String {
    match language {
        Language::En => "en".to_string(),
        Language::De => "de".to_string(),
    }
}

/// Format vocabulary level as human-readable string
fn format_vocabulary_level(level: &VocabularyLevel) -> String {
    match level {
        VocabularyLevel::Basic => "Basic".to_string(),
        VocabularyLevel::Intermediate => "Intermediate".to_string(),
        VocabularyLevel::Advanced => "Advanced".to_string(),
    }
}

/// Format list of strings as comma-separated string
fn format_list(items: &[String]) -> String {
    items.join(", ")
}

/// Get hardcoded fallback template for service and language
///
/// Provides comprehensive fallback templates when config.toml doesn't have templates.
fn get_fallback_template(service: MCPServiceType, language: Language) -> Template {
    match (service, language) {
        // ========================================================================
        // Story Generator Templates
        // ========================================================================
        (MCPServiceType::StoryGenerator, Language::En) => Template {
            system_prompt: "You are a creative story generator for {theme} content. Generate engaging, age-appropriate stories for age group: {age_group}. Language: {language}. Incorporate these educational goals: {educational_goals}.".to_string(),
            user_prompt: "Create a story segment about {theme} that is educational and engaging.".to_string(),
        },

        (MCPServiceType::StoryGenerator, Language::De) => Template {
            system_prompt: "Du bist ein kreativer Geschichtengenerator für {theme}-Inhalte. Erstelle ansprechende, altersgerechte Geschichten für Altersgruppe: {age_group}. Sprache: {language}. Integriere diese Lernziele: {educational_goals}.".to_string(),
            user_prompt: "Erstelle ein Geschichtensegment über {theme}, das lehrreich und ansprechend ist.".to_string(),
        },

        // ========================================================================
        // Quality Control Templates
        // ========================================================================
        (MCPServiceType::QualityControl, Language::En) => Template {
            system_prompt: "You are a quality control validator for {content_type} content. Validate content for age group: {age_group}. Language: {language}. Check for: age-appropriateness, language quality, educational value, safety.".to_string(),
            user_prompt: "Validate the following {content_type} content and provide a quality score with feedback.".to_string(),
        },

        (MCPServiceType::QualityControl, Language::De) => Template {
            system_prompt: "Du bist ein Qualitätsprüfer für {content_type}-Inhalte. Überprüfe Inhalte für Altersgruppe: {age_group}. Sprache: {language}. Prüfe: Altersangemessenheit, Sprachqualität, Bildungswert, Sicherheit.".to_string(),
            user_prompt: "Überprüfe den folgenden {content_type}-Inhalt und gib eine Qualitätsbewertung mit Feedback.".to_string(),
        },

        // ========================================================================
        // Constraint Enforcer Templates
        // ========================================================================
        (MCPServiceType::ConstraintEnforcer, Language::En) => Template {
            system_prompt: "You are a constraint enforcement validator. Enforce vocabulary level: {vocabulary_level}. Language: {language}. Verify presence of required elements: {required_elements}. Check theme consistency.".to_string(),
            user_prompt: "Check the following content for constraint violations and provide feedback.".to_string(),
        },

        (MCPServiceType::ConstraintEnforcer, Language::De) => Template {
            system_prompt: "Du bist ein Einschränkungsprüfer. Setze Vokabularstufe durch: {vocabulary_level}. Sprache: {language}. Überprüfe das Vorhandensein erforderlicher Elemente: {required_elements}. Prüfe Themenkonsistenz.".to_string(),
            user_prompt: "Überprüfe den folgenden Inhalt auf Verstöße gegen Einschränkungen und gib Feedback.".to_string(),
        },

        // ========================================================================
        // Prompt Helper Templates (Meta-service for prompt generation)
        // ========================================================================
        (MCPServiceType::PromptHelper, Language::En) => Template {
            system_prompt: "You are a prompt helper service that generates prompts for other MCP services. Generate prompts for theme: {theme}, age group: {age_group}, language: {language}. Educational goals: {educational_goals}.".to_string(),
            user_prompt: "Generate appropriate prompts for the requested service type.".to_string(),
        },

        (MCPServiceType::PromptHelper, Language::De) => Template {
            system_prompt: "Du bist ein Prompt-Hilfsdienst, der Prompts für andere MCP-Dienste generiert. Generiere Prompts für Thema: {theme}, Altersgruppe: {age_group}, Sprache: {language}. Lernziele: {educational_goals}.".to_string(),
            user_prompt: "Generiere geeignete Prompts für den angeforderten Diensttyp.".to_string(),
        },

        // ========================================================================
        // Orchestrator Templates (Workflow coordination)
        // ========================================================================
        (MCPServiceType::Orchestrator, Language::En) => Template {
            system_prompt: "You are an orchestration service coordinating multiple MCP services for content generation. Theme: {theme}, age group: {age_group}, language: {language}. Educational goals: {educational_goals}. Coordinate services to achieve these goals.".to_string(),
            user_prompt: "Orchestrate the content generation workflow for {content_type}.".to_string(),
        },

        (MCPServiceType::Orchestrator, Language::De) => Template {
            system_prompt: "Du bist ein Orchestrierungsdienst, der mehrere MCP-Dienste für die Inhaltserstellung koordiniert. Thema: {theme}, Altersgruppe: {age_group}, Sprache: {language}. Lernziele: {educational_goals}. Koordiniere Dienste, um diese Ziele zu erreichen.".to_string(),
            user_prompt: "Orchestriere den Workflow zur Inhaltserstellung für {content_type}.".to_string(),
        },
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_format_age_group() {
        assert_eq!(format_age_group(&AgeGroup::_6To8), "6-8");
        assert_eq!(format_age_group(&AgeGroup::_9To11), "9-11");
        assert_eq!(format_age_group(&AgeGroup::_12To14), "12-14");
        assert_eq!(format_age_group(&AgeGroup::_15To17), "15-17");
        assert_eq!(format_age_group(&AgeGroup::Plus18), "18+");
    }

    #[test]
    fn test_format_language() {
        assert_eq!(format_language(&Language::En), "en");
        assert_eq!(format_language(&Language::De), "de");
    }

    #[test]
    fn test_format_vocabulary_level() {
        assert_eq!(format_vocabulary_level(&VocabularyLevel::Basic), "Basic");
        assert_eq!(format_vocabulary_level(&VocabularyLevel::Intermediate), "Intermediate");
        assert_eq!(format_vocabulary_level(&VocabularyLevel::Advanced), "Advanced");
    }

    #[test]
    fn test_format_list_empty() {
        let empty: Vec<String> = vec![];
        assert_eq!(format_list(&empty), "");
    }

    #[test]
    fn test_format_list_single() {
        let single = vec!["Item".to_string()];
        assert_eq!(format_list(&single), "Item");
    }

    #[test]
    fn test_format_list_multiple() {
        let multiple = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert_eq!(format_list(&multiple), "A, B, C");
    }

    #[test]
    fn test_substitute_variables_basic() {
        let template = "Theme: {theme}, Age: {age_group}";
        let context = create_test_context();
        let result = substitute_variables(template, &context);
        assert_eq!(result, "Theme: Space Adventure, Age: 6-8");
    }

    #[test]
    fn test_substitute_variables_with_lists() {
        let template = "Goals: {educational_goals}";
        let mut context = create_test_context();
        context.educational_goals = vec!["A".to_string(), "B".to_string()];
        let result = substitute_variables(template, &context);
        assert_eq!(result, "Goals: A, B");
    }

    #[test]
    fn test_substitute_variables_with_optional_none() {
        let template = "Vocab: {vocabulary_level}, Type: {content_type}";
        let mut context = create_test_context();
        context.vocabulary_level = None;
        context.content_type = None;
        let result = substitute_variables(template, &context);
        assert_eq!(result, "Vocab: , Type: ");
    }

    #[test]
    fn test_substitute_variables_unknown_placeholder() {
        let template = "Known: {theme}, Unknown: {xyz}";
        let context = create_test_context();
        let result = substitute_variables(template, &context);
        assert_eq!(result, "Known: Space Adventure, Unknown: {xyz}");
    }

    #[test]
    fn test_get_fallback_template_story_generator_en() {
        let template = get_fallback_template(MCPServiceType::StoryGenerator, Language::En);
        assert!(template.system_prompt.contains("{theme}"));
        assert!(template.system_prompt.contains("{age_group}"));
        assert!(template.system_prompt.contains("story"));
    }

    #[test]
    fn test_get_fallback_template_story_generator_de() {
        let template = get_fallback_template(MCPServiceType::StoryGenerator, Language::De);
        assert!(template.system_prompt.contains("{theme}"));
        assert!(template.system_prompt.contains("Geschichte") || template.system_prompt.contains("Du bist"));
    }

    #[test]
    fn test_get_fallback_template_quality_control_en() {
        let template = get_fallback_template(MCPServiceType::QualityControl, Language::En);
        assert!(template.system_prompt.contains("quality") || template.system_prompt.contains("validat"));
        assert!(template.system_prompt.contains("{content_type}"));
    }

    #[test]
    fn test_get_fallback_template_quality_control_de() {
        let template = get_fallback_template(MCPServiceType::QualityControl, Language::De);
        assert!(template.system_prompt.contains("Qualität") || template.system_prompt.contains("prüf"));
        assert!(template.system_prompt.contains("{content_type}"));
    }

    #[test]
    fn test_get_fallback_template_constraint_enforcer_en() {
        let template = get_fallback_template(MCPServiceType::ConstraintEnforcer, Language::En);
        assert!(template.system_prompt.contains("constraint"));
        assert!(template.system_prompt.contains("{vocabulary_level}"));
    }

    #[test]
    fn test_get_fallback_template_constraint_enforcer_de() {
        let template = get_fallback_template(MCPServiceType::ConstraintEnforcer, Language::De);
        assert!(template.system_prompt.contains("Einschränkung") || template.system_prompt.contains("prüf"));
        assert!(template.system_prompt.contains("{vocabulary_level}"));
    }

    #[test]
    fn test_apply_template() {
        let template = Template {
            system_prompt: "Theme: {theme}, Age: {age_group}".to_string(),
            user_prompt: "Goals: {educational_goals}".to_string(),
        };
        let context = create_test_context();
        let (system, user) = apply_template(&template, &context);

        assert_eq!(system, "Theme: Space Adventure, Age: 6-8");
        assert_eq!(user, "Goals: Learn planets");
    }

    #[test]
    fn test_get_template_uses_fallback() {
        let config = PromptHelperConfig::default();
        let template = get_template(
            MCPServiceType::StoryGenerator,
            Language::En,
            &config
        ).expect("Should return fallback template");

        assert!(!template.system_prompt.is_empty());
        assert!(!template.user_prompt.is_empty());
    }

    #[test]
    fn test_get_template_all_service_types() {
        let config = PromptHelperConfig::default();
        let services = vec![
            MCPServiceType::StoryGenerator,
            MCPServiceType::QualityControl,
            MCPServiceType::ConstraintEnforcer,
            MCPServiceType::PromptHelper,
            MCPServiceType::Orchestrator,
        ];

        for service in services {
            let template = get_template(service, Language::En, &config)
                .expect(&format!("Should get template for {:?}", service));
            assert!(!template.system_prompt.is_empty());
            assert!(!template.user_prompt.is_empty());
        }
    }

    #[test]
    fn test_get_template_both_languages() {
        let config = PromptHelperConfig::default();

        let en_template = get_template(
            MCPServiceType::StoryGenerator,
            Language::En,
            &config
        ).expect("Should get English template");

        let de_template = get_template(
            MCPServiceType::StoryGenerator,
            Language::De,
            &config
        ).expect("Should get German template");

        // Templates should be different for different languages
        assert_ne!(en_template.system_prompt, de_template.system_prompt);
    }

    #[test]
    fn test_complete_workflow() {
        let config = PromptHelperConfig::default();

        // Get template
        let template = get_template(
            MCPServiceType::StoryGenerator,
            Language::En,
            &config
        ).expect("Failed to get template");

        // Create context
        let context = TemplateContext {
            theme: "Medieval Adventure".to_string(),
            age_group: AgeGroup::_9To11,
            language: Language::En,
            educational_goals: vec!["History".to_string(), "Courage".to_string()],
            vocabulary_level: Some(VocabularyLevel::Intermediate),
            required_elements: Some(vec!["hero".to_string(), "quest".to_string()]),
            content_type: Some("story".to_string()),
        };

        // Apply template
        let (system, user) = apply_template(&template, &context);

        // Verify results
        assert!(!system.is_empty());
        assert!(!user.is_empty());
        assert!(system.contains("Medieval Adventure"));
        assert!(system.contains("9-11"));
        assert!(system.contains("History, Courage"));
    }
}
