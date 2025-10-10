//! Model selection logic based on language codes
//!
//! This module provides functionality to map language codes to appropriate
//! LLM models based on configuration, with fallback support to default models.

use crate::constants::*;
use crate::error::LlmError;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Select the appropriate model for a given language code
///
/// This function implements the model selection logic with the following priority:
/// 1. Explicit model name override (from parameters)
/// 2. Language-specific model mapping (from config)
/// 3. Default model (from config, if fallback is enabled)
/// 4. Error if no model found and fallback is disabled
///
/// # Arguments
///
/// * `language_code` - Language code for content generation (e.g., "en", "es", "fr")
/// * `explicit_model` - Optional explicit model name override
/// * `language_models` - Language code to model name mapping
/// * `default_model` - Default model to use as fallback
/// * `use_fallback` - Whether to use default model as fallback
///
/// # Returns
///
/// The selected model name or an error if no suitable model is found
///
/// # Example
///
/// ```
/// use shared_types_llm::select_model_for_language;
/// use std::collections::HashMap;
///
/// let models = HashMap::from([
///     ("en".to_string(), "gpt-4".to_string()),
///     ("es".to_string(), "gpt-4".to_string()),
/// ]);
///
/// // Use explicit model
/// let model = select_model_for_language("en", Some("gpt-3.5-turbo"), &models, "gpt-4", true).unwrap();
/// assert_eq!(model, "gpt-3.5-turbo");
///
/// // Use language mapping
/// let model = select_model_for_language("es", None, &models, "gpt-4", true).unwrap();
/// assert_eq!(model, "gpt-4");
///
/// // Use fallback
/// let model = select_model_for_language("fr", None, &models, "gpt-4", true).unwrap();
/// assert_eq!(model, "gpt-4");
/// ```
pub fn select_model_for_language(
    language_code: &str,
    explicit_model: Option<&str>,
    language_models: &HashMap<String, String>,
    default_model: &str,
    use_fallback: bool,
) -> Result<String, LlmError> {
    // Priority 1: Explicit model override
    if let Some(model) = explicit_model {
        debug!(
            language_code = language_code,
            model = model,
            "Using explicit model override"
        );
        return Ok(model.to_string());
    }

    // Priority 2: Language-specific mapping
    if let Some(model) = language_models.get(language_code) {
        debug!(
            language_code = language_code,
            model = model,
            "Using language-specific model mapping"
        );
        return Ok(model.clone());
    }

    // Priority 3: Default model (if fallback enabled)
    if use_fallback {
        warn!(
            language_code = language_code,
            default_model = default_model,
            "{}",
            AUDIT_MODEL_FALLBACK
        );
        return Ok(default_model.to_string());
    }

    // No model found and fallback disabled
    Err(LlmError::model_not_available(language_code))
}

/// Merge language model mappings with priority to runtime overrides
///
/// This function merges multiple model mapping sources, giving priority to
/// runtime configuration over static configuration.
///
/// # Arguments
///
/// * `base_models` - Base model mappings (e.g., from default config)
/// * `override_models` - Override model mappings (e.g., from tenant config)
///
/// # Returns
///
/// A new HashMap with merged mappings where overrides take precedence
///
/// # Example
///
/// ```
/// use shared_types_llm::merge_model_mappings;
/// use std::collections::HashMap;
///
/// let base = HashMap::from([
///     ("en".to_string(), "base-model".to_string()),
///     ("es".to_string(), "base-model".to_string()),
/// ]);
///
/// let overrides = HashMap::from([
///     ("en".to_string(), "override-model".to_string()),
/// ]);
///
/// let merged = merge_model_mappings(&base, &overrides);
/// assert_eq!(merged.get("en"), Some(&"override-model".to_string()));
/// assert_eq!(merged.get("es"), Some(&"base-model".to_string()));
/// ```
pub fn merge_model_mappings(
    base_models: &HashMap<String, String>,
    override_models: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = base_models.clone();
    merged.extend(override_models.clone());
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_models() -> HashMap<String, String> {
        HashMap::from([
            ("en".to_string(), "english-model".to_string()),
            ("es".to_string(), "spanish-model".to_string()),
            ("fr".to_string(), "french-model".to_string()),
        ])
    }

    #[test]
    fn test_select_explicit_model() {
        let models = create_test_models();
        let result = select_model_for_language(
            "en",
            Some("explicit-model"),
            &models,
            "default-model",
            true,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "explicit-model");
    }

    #[test]
    fn test_select_language_specific_model() {
        let models = create_test_models();
        let result = select_model_for_language("es", None, &models, "default-model", true);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "spanish-model");
    }

    #[test]
    fn test_select_default_model_with_fallback() {
        let models = create_test_models();
        let result = select_model_for_language("de", None, &models, "default-model", true);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default-model");
    }

    #[test]
    fn test_select_model_without_fallback_fails() {
        let models = create_test_models();
        let result = select_model_for_language("de", None, &models, "default-model", false);

        assert!(result.is_err());
        match result {
            Err(LlmError::ModelNotAvailable { model_name, .. }) => {
                assert_eq!(model_name, "de");
            }
            _ => panic!("Expected ModelNotAvailable error"),
        }
    }

    #[test]
    fn test_explicit_model_overrides_everything() {
        let models = create_test_models();
        let result = select_model_for_language(
            "en",
            Some("override-model"),
            &models,
            "default-model",
            false,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "override-model");
    }

    #[test]
    fn test_merge_model_mappings_basic() {
        let base = HashMap::from([
            ("en".to_string(), "base-en".to_string()),
            ("es".to_string(), "base-es".to_string()),
        ]);

        let overrides = HashMap::from([("en".to_string(), "override-en".to_string())]);

        let merged = merge_model_mappings(&base, &overrides);

        assert_eq!(merged.get("en"), Some(&"override-en".to_string()));
        assert_eq!(merged.get("es"), Some(&"base-es".to_string()));
    }

    #[test]
    fn test_merge_model_mappings_new_language() {
        let base = HashMap::from([("en".to_string(), "base-en".to_string())]);

        let overrides = HashMap::from([("fr".to_string(), "override-fr".to_string())]);

        let merged = merge_model_mappings(&base, &overrides);

        assert_eq!(merged.get("en"), Some(&"base-en".to_string()));
        assert_eq!(merged.get("fr"), Some(&"override-fr".to_string()));
    }

    #[test]
    fn test_merge_model_mappings_empty_override() {
        let base = HashMap::from([("en".to_string(), "base-en".to_string())]);

        let overrides = HashMap::new();

        let merged = merge_model_mappings(&base, &overrides);

        assert_eq!(merged, base);
    }

    #[test]
    fn test_merge_model_mappings_empty_base() {
        let base = HashMap::new();

        let overrides = HashMap::from([("en".to_string(), "override-en".to_string())]);

        let merged = merge_model_mappings(&base, &overrides);

        assert_eq!(merged, overrides);
    }
}
