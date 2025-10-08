//! Business logic handlers for Prompt-Helper MCP tools
//!
//! This module implements the handler functions that process MCP tool requests
//! and generate PromptPackage responses. Each handler:
//! - Extracts and validates parameters from CallToolRequest
//! - Attempts LLM generation using LlmService trait
//! - Falls back to template loading on LLM failure
//! - Returns CallToolResult with PromptPackage JSON or error
//!
//! # Architecture
//!
//! Handlers follow a dependency injection pattern:
//! - Accept `&impl LlmService` for testability (MockLlmService in tests)
//! - Accept `&PromptHelperConfig` for template loading
//! - Never propagate errors - always return CallToolResult with is_error flag
//!
//! # Testing
//!
//! See `tests/handler_tests.rs` for comprehensive test coverage using MockLlmService.

use rmcp::model::{CallToolRequest, CallToolResult, Content};
use shared_types::{
    traits::LlmService,
    AgeGroup, Language, LLMConfig, MCPServiceType, PromptGenerationMethod, PromptGenerationRequest,
    PromptMetadata, PromptPackage,
};
use tracing::{error, info, warn, info_span, Instrument};

use crate::config::PromptHelperConfig;
use crate::mcp_tools::{
    GenerateConstraintPromptsParams, GenerateStoryPromptsParams, GenerateValidationPromptsParams,
    GetModelForLanguageParams,
};

// ============================================================================
// Handler: generate_story_prompts
// ============================================================================

/// Handle generate_story_prompts tool request
///
/// Generates system and user prompts for story content generation based on
/// theme, age group, language, and educational goals.
///
/// # Flow
/// 1. Extract and deserialize parameters from CallToolRequest
/// 2. Try LLM generation via LlmService
/// 3. On LLM failure: load template from config
/// 4. Construct PromptPackage with prompts, metadata, and fallback_used flag
/// 5. Return CallToolResult with PromptPackage JSON
///
/// # Arguments
/// * `request` - MCP CallToolRequest containing tool parameters
/// * `llm_service` - LLM service for prompt generation (trait for testability)
/// * `config` - Configuration for template fallback and model selection
///
/// # Returns
/// CallToolResult with PromptPackage JSON content or error with is_error=true
#[tracing::instrument(
    name = "tool_execution",
    skip(request, llm_service, config),
    fields(
        tool_name = "generate_story_prompts",
        service_target = "StoryGenerator"
    )
)]
pub async fn handle_generate_story_prompts(
    request: CallToolRequest,
    llm_service: &impl LlmService,
    config: &PromptHelperConfig,
) -> CallToolResult {
    info!("Handling generate_story_prompts request");

    // Extract parameters
    let arguments = match request.params.arguments {
        Some(args) => serde_json::Value::Object(args),
        None => {
            error!("Missing arguments in request");
            return create_error_result("Missing arguments in request");
        }
    };

    let params: GenerateStoryPromptsParams = match serde_json::from_value(arguments) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to deserialize parameters: {}", e);
            return create_error_result(&format!("Invalid parameters: {}", e));
        }
    };

    // Build PromptGenerationRequest for LLM service
    let generation_request = shared_types::GenerationRequest {
        theme: params.theme.clone(),
        language: params.language.clone(),
        age_group: params.age_group.clone(),
        tenant_id: 0, // Default for prompt generation context
        node_count: None,
        educational_goals: Some(params.educational_goals.clone()),
        required_elements: None,
        vocabulary_level: None,
        author_id: None,
        prompt_packages: None,
        tags: None,
    };

    let prompt_gen_request = PromptGenerationRequest {
        generation_request,
        service_target: MCPServiceType::StoryGenerator,
        node_context: None,
        batch_info: None,
    };

    // Meta-prompt for LLM to generate story prompts
    let meta_prompt = format!(
        "Generate system and user prompts for a story generator service.\n\
         Theme: {}\n\
         Age Group: {:?}\n\
         Language: {:?}\n\
         Educational Goals: {}\n\n\
         The system prompt should instruct the LLM on how to generate engaging, \
         age-appropriate story content with educational value.\n\
         The user prompt should provide context for the current story generation task.\n\n\
         Return two prompts separated by '---SEPARATOR---':\n\
         First the system prompt, then the user prompt.",
        params.theme,
        params.age_group,
        params.language,
        params.educational_goals.join(", ")
    );

    // Try LLM generation
    let (system_prompt, user_prompt, fallback_used) =
        match llm_service.generate_prompt(&meta_prompt, &prompt_gen_request).await {
            Ok((sys, user)) => {
                info!("LLM generation successful for story prompts");
                (sys, user, false)
            }
            Err(e) => {
                warn!("LLM generation failed, using template fallback: {}", e);

                // Template fallback with tracing span
                let _fallback_span = info_span!(
                    "template_fallback",
                    fallback_reason = "llm_error",
                    service_type = ?MCPServiceType::StoryGenerator,
                    language = ?params.language
                );
                let _guard = _fallback_span.enter();

                let template_sys = format!(
                    "You are a creative story generator for {} content.\n\
                     Generate engaging, age-appropriate stories for age group: {:?}.\n\
                     Language: {:?}.\n\
                     Incorporate these educational goals: {}.",
                    params.theme,
                    params.age_group,
                    params.language,
                    params.educational_goals.join(", ")
                );
                let template_user = format!(
                    "Create a story segment about {} that is educational and engaging.",
                    params.theme
                );
                (template_sys, template_user, true)
            }
        };

    // Construct PromptPackage
    let prompt_package = PromptPackage {
        system_prompt,
        user_prompt,
        llm_model: config.prompt.models.default_model.clone(),
        language: params.language.clone(),
        llm_config: LLMConfig {
            temperature: config.prompt.models.temperature as f64,
            max_tokens: config.prompt.models.max_tokens as i64,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: vec![],
        },
        prompt_metadata: PromptMetadata {
            service_target: MCPServiceType::StoryGenerator,
            theme_context: params.theme.clone(),
            age_group_context: params.age_group,
            language_context: params.language,
            generation_method: if fallback_used {
                PromptGenerationMethod::TemplateFallback
            } else {
                PromptGenerationMethod::LLMGenerated
            },
            template_version: "1.0.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        fallback_used,
    };

    // Serialize and return
    match serde_json::to_string(&prompt_package) {
        Ok(json) => CallToolResult {
            content: vec![Content::text(json)],
            is_error: None,
            structured_content: None,
            meta: None,
        },
        Err(e) => {
            error!("Failed to serialize PromptPackage: {}", e);
            create_error_result(&format!("Serialization error: {}", e))
        }
    }
}

// ============================================================================
// Handler: generate_validation_prompts
// ============================================================================

/// Handle generate_validation_prompts tool request
///
/// Generates prompts for quality control validation, ensuring content meets
/// age-appropriateness, language quality, and educational value standards.
///
/// # Arguments
/// * `request` - MCP CallToolRequest containing tool parameters
/// * `llm_service` - LLM service for prompt generation
/// * `config` - Configuration for template fallback
///
/// # Returns
/// CallToolResult with PromptPackage JSON content or error
#[tracing::instrument(
    name = "tool_execution",
    skip(request, llm_service, config),
    fields(
        tool_name = "generate_validation_prompts",
        service_target = "QualityControl"
    )
)]
pub async fn handle_generate_validation_prompts(
    request: CallToolRequest,
    llm_service: &impl LlmService,
    config: &PromptHelperConfig,
) -> CallToolResult {
    info!("Handling generate_validation_prompts request");

    // Extract parameters
    let arguments = match request.params.arguments {
        Some(args) => serde_json::Value::Object(args),
        None => {
            error!("Missing arguments in request");
            return create_error_result("Missing arguments in request");
        }
    };

    let params: GenerateValidationPromptsParams = match serde_json::from_value(arguments) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to deserialize parameters: {}", e);
            return create_error_result(&format!("Invalid parameters: {}", e));
        }
    };

    // Build PromptGenerationRequest
    let generation_request = shared_types::GenerationRequest {
        theme: format!("Validation for {}", params.content_type),
        language: params.language.clone(),
        age_group: params.age_group.clone(),
        tenant_id: 0,
        node_count: None,
        educational_goals: None,
        required_elements: None,
        vocabulary_level: None,
        author_id: None,
        prompt_packages: None,
        tags: None,
    };

    let prompt_gen_request = PromptGenerationRequest {
        generation_request,
        service_target: MCPServiceType::QualityControl,
        node_context: None,
        batch_info: None,
    };

    // Meta-prompt for validation
    let meta_prompt = format!(
        "Generate system and user prompts for a quality control validation service.\n\
         Content Type: {}\n\
         Age Group: {:?}\n\
         Language: {:?}\n\n\
         The system prompt should instruct the LLM on validation criteria:\n\
         - Age-appropriateness checks\n\
         - Language quality verification\n\
         - Educational value assessment\n\
         - Safety and content guidelines\n\n\
         The user prompt should describe what content to validate.\n\n\
         Return two prompts separated by '---SEPARATOR---'.",
        params.content_type, params.age_group, params.language
    );

    // Try LLM generation
    let (system_prompt, user_prompt, fallback_used) =
        match llm_service.generate_prompt(&meta_prompt, &prompt_gen_request).await {
            Ok((sys, user)) => {
                info!("LLM generation successful for validation prompts");
                (sys, user, false)
            }
            Err(e) => {
                warn!("LLM generation failed, using template fallback: {}", e);

                // Template fallback with tracing span
                let _fallback_span = info_span!(
                    "template_fallback",
                    fallback_reason = "llm_error",
                    service_type = ?MCPServiceType::QualityControl,
                    language = ?params.language
                );
                let _guard = _fallback_span.enter();

                let template_sys = format!(
                    "You are a quality control validator for {} content.\n\
                     Validate content for age group: {:?}.\n\
                     Language: {:?}.\n\
                     Check for: age-appropriateness, language quality, educational value, safety.",
                    params.content_type, params.age_group, params.language
                );
                let template_user = format!(
                    "Validate the following {} content and provide a quality score with feedback.",
                    params.content_type
                );
                (template_sys, template_user, true)
            }
        };

    // Construct PromptPackage
    let prompt_package = PromptPackage {
        system_prompt,
        user_prompt,
        llm_model: config.prompt.models.default_model.clone(),
        language: params.language.clone(),
        llm_config: LLMConfig {
            temperature: config.prompt.models.temperature as f64,
            max_tokens: config.prompt.models.max_tokens as i64,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: vec![],
        },
        prompt_metadata: PromptMetadata {
            service_target: MCPServiceType::QualityControl,
            theme_context: params.content_type.clone(),
            age_group_context: params.age_group,
            language_context: params.language,
            generation_method: if fallback_used {
                PromptGenerationMethod::TemplateFallback
            } else {
                PromptGenerationMethod::LLMGenerated
            },
            template_version: "1.0.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        fallback_used,
    };

    // Serialize and return
    match serde_json::to_string(&prompt_package) {
        Ok(json) => CallToolResult {
            content: vec![Content::text(json)],
            is_error: None,
            structured_content: None,
            meta: None,
        },
        Err(e) => {
            error!("Failed to serialize PromptPackage: {}", e);
            create_error_result(&format!("Serialization error: {}", e))
        }
    }
}

// ============================================================================
// Handler: generate_constraint_prompts
// ============================================================================

/// Handle generate_constraint_prompts tool request
///
/// Generates prompts for constraint enforcement, ensuring vocabulary levels,
/// required story elements, and theme consistency are maintained.
///
/// # Arguments
/// * `request` - MCP CallToolRequest containing tool parameters
/// * `llm_service` - LLM service for prompt generation
/// * `config` - Configuration for template fallback
///
/// # Returns
/// CallToolResult with PromptPackage JSON content or error
#[tracing::instrument(
    name = "tool_execution",
    skip(request, llm_service, config),
    fields(
        tool_name = "generate_constraint_prompts",
        service_target = "ConstraintEnforcer"
    )
)]
pub async fn handle_generate_constraint_prompts(
    request: CallToolRequest,
    llm_service: &impl LlmService,
    config: &PromptHelperConfig,
) -> CallToolResult {
    info!("Handling generate_constraint_prompts request");

    // Extract parameters
    let arguments = match request.params.arguments {
        Some(args) => serde_json::Value::Object(args),
        None => {
            error!("Missing arguments in request");
            return create_error_result("Missing arguments in request");
        }
    };

    let params: GenerateConstraintPromptsParams = match serde_json::from_value(arguments) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to deserialize parameters: {}", e);
            return create_error_result(&format!("Invalid parameters: {}", e));
        }
    };

    // Build PromptGenerationRequest
    let generation_request = shared_types::GenerationRequest {
        theme: "Constraint Enforcement".to_string(),
        language: params.language.clone(),
        age_group: AgeGroup::_6To8, // Default, constraints apply to all ages
        tenant_id: 0,
        node_count: None,
        educational_goals: None,
        required_elements: Some(params.required_elements.clone()),
        vocabulary_level: Some(params.vocabulary_level.clone()),
        author_id: None,
        prompt_packages: None,
        tags: None,
    };

    let prompt_gen_request = PromptGenerationRequest {
        generation_request,
        service_target: MCPServiceType::ConstraintEnforcer,
        node_context: None,
        batch_info: None,
    };

    // Meta-prompt for constraint enforcement
    let meta_prompt = format!(
        "Generate system and user prompts for a constraint enforcement service.\n\
         Vocabulary Level: {:?}\n\
         Language: {:?}\n\
         Required Elements: {}\n\n\
         The system prompt should instruct the LLM on constraint checking:\n\
         - Vocabulary level restrictions\n\
         - Required story element verification\n\
         - Theme consistency checks\n\n\
         The user prompt should describe what content to check for constraints.\n\n\
         Return two prompts separated by '---SEPARATOR---'.",
        params.vocabulary_level,
        params.language,
        params.required_elements.join(", ")
    );

    // Try LLM generation
    let (system_prompt, user_prompt, fallback_used) =
        match llm_service.generate_prompt(&meta_prompt, &prompt_gen_request).await {
            Ok((sys, user)) => {
                info!("LLM generation successful for constraint prompts");
                (sys, user, false)
            }
            Err(e) => {
                warn!("LLM generation failed, using template fallback: {}", e);

                // Template fallback with tracing span
                let _fallback_span = info_span!(
                    "template_fallback",
                    fallback_reason = "llm_error",
                    service_type = ?MCPServiceType::ConstraintEnforcer,
                    language = ?params.language
                );
                let _guard = _fallback_span.enter();

                let template_sys = format!(
                    "You are a constraint enforcement validator.\n\
                     Enforce vocabulary level: {:?}.\n\
                     Language: {:?}.\n\
                     Verify presence of required elements: {}.\n\
                     Check theme consistency.",
                    params.vocabulary_level,
                    params.language,
                    params.required_elements.join(", ")
                );
                let template_user =
                    "Check the following content for constraint violations and provide feedback."
                        .to_string();
                (template_sys, template_user, true)
            }
        };

    // Construct PromptPackage
    let prompt_package = PromptPackage {
        system_prompt,
        user_prompt,
        llm_model: config.prompt.models.default_model.clone(),
        language: params.language.clone(),
        llm_config: LLMConfig {
            temperature: config.prompt.models.temperature as f64,
            max_tokens: config.prompt.models.max_tokens as i64,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: vec![],
        },
        prompt_metadata: PromptMetadata {
            service_target: MCPServiceType::ConstraintEnforcer,
            theme_context: "Constraint Enforcement".to_string(),
            age_group_context: AgeGroup::_6To8, // Default
            language_context: params.language,
            generation_method: if fallback_used {
                PromptGenerationMethod::TemplateFallback
            } else {
                PromptGenerationMethod::LLMGenerated
            },
            template_version: "1.0.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        fallback_used,
    };

    // Serialize and return
    match serde_json::to_string(&prompt_package) {
        Ok(json) => CallToolResult {
            content: vec![Content::text(json)],
            is_error: None,
            structured_content: None,
            meta: None,
        },
        Err(e) => {
            error!("Failed to serialize PromptPackage: {}", e);
            create_error_result(&format!("Serialization error: {}", e))
        }
    }
}

// ============================================================================
// Handler: get_model_for_language
// ============================================================================

/// Handle get_model_for_language tool request
///
/// Retrieves the appropriate LLM model identifier for a specific language,
/// ensuring language-specific models are used for optimal content generation.
///
/// # Arguments
/// * `request` - MCP CallToolRequest containing tool parameters
/// * `llm_service` - LLM service for model availability checking
/// * `config` - Configuration for model selection
///
/// # Returns
/// CallToolResult with model name string or error
#[tracing::instrument(
    name = "tool_execution",
    skip(request, llm_service, config),
    fields(
        tool_name = "get_model_for_language"
    )
)]
pub async fn handle_get_model_for_language(
    request: CallToolRequest,
    llm_service: &impl LlmService,
    config: &PromptHelperConfig,
) -> CallToolResult {
    info!("Handling get_model_for_language request");

    // Extract parameters
    let arguments = match request.params.arguments {
        Some(args) => serde_json::Value::Object(args),
        None => {
            error!("Missing arguments in request");
            return create_error_result("Missing arguments in request");
        }
    };

    let params: GetModelForLanguageParams = match serde_json::from_value(arguments) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to deserialize parameters: {}", e);
            return create_error_result(&format!("Invalid parameters: {}", e));
        }
    };

    // Language-specific model mapping (can be moved to config in future)
    let model_name = match params.language {
        Language::En => config.prompt.models.default_model.clone(),
        Language::De => {
            // For German, we might want a multilingual model
            // For now, use default and let config override
            config.prompt.models.default_model.clone()
        }
    };

    // Verify model exists (optional check)
    match llm_service.model_exists(&model_name).await {
        Ok(true) => {
            info!("Model {} exists and is available", model_name);
        }
        Ok(false) => {
            warn!("Model {} not found, returning anyway", model_name);
        }
        Err(e) => {
            warn!("Could not verify model existence: {}, proceeding", e);
        }
    }

    // Return model name as simple text
    CallToolResult {
        content: vec![Content::text(model_name)],
        is_error: None,
        structured_content: None,
        meta: None,
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create error CallToolResult
///
/// # Arguments
/// * `message` - Error message to include in result
///
/// # Returns
/// CallToolResult with is_error=true and error message in content
fn create_error_result(message: &str) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(message.to_string())],
        is_error: Some(true),
        structured_content: None,
        meta: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_error_result() {
        let result = create_error_result("Test error message");
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content.len(), 1);
    }
}
