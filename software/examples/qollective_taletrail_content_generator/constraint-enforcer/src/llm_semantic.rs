//! LLM-based semantic matching for constraint validation
//!
//! This module provides LLM fallback functionality when keyword-based matching
//! is insufficient. It uses the LLM client to perform semantic understanding
//! of whether content addresses a required element.

use shared_types::{Result, TaleTrailError};
use shared_types_llm::DynamicLlmClient;
use tracing::{debug, warn};

/// Check if content semantically matches a required element using LLM
///
/// This function sends a yes/no question to the LLM asking if the content
/// conveys or addresses the concept described by the required element.
///
/// # Arguments
///
/// * `llm_client` - LLM client for making API calls
/// * `content` - The content to check (should be truncated before calling)
/// * `element` - The required element phrase to check for
/// * `prompt_template` - Template for the LLM prompt with {element} and {content} placeholders
///
/// # Returns
///
/// `true` if LLM determines content semantically matches, `false` otherwise
///
/// # Errors
///
/// Returns error if LLM API call fails or response cannot be parsed
pub async fn check_semantic_match(
    llm_client: &dyn DynamicLlmClient,
    content: &str,
    element: &str,
    prompt_template: &str,
) -> Result<bool> {
    // Build prompt from template
    let prompt = prompt_template
        .replace("{element}", element)
        .replace("{content}", content);

    debug!(
        element = %element,
        content_length = content.len(),
        "Checking semantic match with LLM"
    );

    // Call LLM with simple prompt (no additional context needed for yes/no)
    // The LLM client is already configured with appropriate parameters
    match llm_client.prompt(&prompt, None).await {
        Ok(response) => {
            let response_lower = response.trim().to_lowercase();

            // Parse response - look for yes/no
            let is_match = if response_lower.contains("yes") {
                true
            } else if response_lower.contains("no") {
                false
            } else {
                // Unclear response - log warning and default to false
                warn!(
                    response = %response,
                    element = %element,
                    "LLM returned unclear response for semantic match, defaulting to no match"
                );
                false
            };

            debug!(
                element = %element,
                is_match = is_match,
                response = %response,
                "LLM semantic match result"
            );

            Ok(is_match)
        }
        Err(e) => {
            warn!(
                error = %e,
                element = %element,
                "LLM API call failed for semantic matching"
            );
            Err(TaleTrailError::LLMError(format!(
                "Semantic match check failed: {}",
                e
            )))
        }
    }
}

/// Truncate content to maximum length for LLM processing
///
/// Truncates at word boundaries to avoid cutting words in half.
/// Uses character-boundary-aware truncation to safely handle UTF-8 multi-byte characters.
///
/// This is a thin wrapper around the centralized `truncate_with_ellipsis` utility
/// from `shared_types::text_utils`.
///
/// # Arguments
///
/// * `content` - The content to truncate
/// * `max_length` - Maximum character length (in bytes, not Unicode characters)
///
/// # Returns
///
/// Truncated content string (or original if shorter than max_length)
pub fn truncate_content(content: &str, max_length: usize) -> String {
    shared_types::truncate_with_ellipsis(content, max_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration test - verify wrapper delegates correctly to shared_types::truncate_with_ellipsis
    #[test]
    fn test_truncate_content_delegates_to_shared_types() {
        let content = "This is a test";
        let result = truncate_content(content, 10);
        let expected = shared_types::truncate_with_ellipsis(content, 10);
        assert_eq!(result, expected);
    }

    // Comprehensive tests for truncate_with_ellipsis are in shared-types/src/text_utils.rs
}
