//! Quality Control Validation Logic Module
//!
//! This module orchestrates the three validation rubrics (age appropriateness,
//! safety, and educational value) to produce comprehensive ValidationResults
//! with correction suggestions and capability assessments.

use shared_types::*;
use crate::config::QualityControlConfig;

/// Main validation function that orchestrates all rubrics
///
/// # Arguments
/// * `content` - The ContentNode to validate
/// * `age_group` - Target age group for validation
/// * `educational_goals` - Optional educational goals to align with
/// * `language` - Language code for loading appropriate restricted words
/// * `validation_policy` - Optional validation policy for this request
/// * `config` - Quality control configuration
///
/// # Returns
/// A complete ValidationResult with scores, issues, and correction suggestions
pub async fn validate_content_node(
    content: &ContentNode,
    age_group: AgeGroup,
    educational_goals: &[String],
    language: &str,
    validation_policy: Option<&ValidationPolicy>,
    config: &QualityControlConfig,
) -> ValidationResult {
    // Check if validation is entirely disabled
    if let Some(policy) = validation_policy {
        if !policy.enable_validation {
            tracing::info!(
                node_id = %content.id,
                "Validation disabled by request policy - returning valid result"
            );
            return ValidationResult {
                is_valid: true,
                age_appropriate_score: 1.0,
                safety_issues: Vec::new(),
                educational_value_score: 1.0,
                corrections: Vec::new(),
                correction_capability: CorrectionCapability::CanFixLocally,
            };
        }
    }

    // Build restricted words list from config and policy
    let restricted_words = build_restricted_words_list(
        validation_policy,
        config,
        language,
    );

    // Log validation start
    tracing::info!(
        node_id = %content.id,
        language = %language,
        restricted_word_count = restricted_words.as_ref().map(|w| w.len()).unwrap_or(0),
        "Starting content validation"
    );

    // OPTIMIZATION: Run validation checks concurrently using tokio::task::spawn_blocking
    // to avoid blocking the async runtime with CPU-bound text analysis
    let text = content.content.text.clone();
    let age_group_clone = age_group.clone();
    let educational_goals_vec = educational_goals.to_vec();
    let restricted_words_clone = restricted_words.clone();

    let (age_score_result, safety_issues_result, edu_score_result) = tokio::join!(
        tokio::task::spawn_blocking(move || {
            crate::rubrics::validate_age_appropriateness(&text, age_group_clone) as f64
        }),
        tokio::task::spawn_blocking({
            let text = content.content.text.clone();
            move || {
                crate::rubrics::validate_safety(&text, restricted_words_clone.as_deref())
            }
        }),
        tokio::task::spawn_blocking({
            let text = content.content.text.clone();
            move || {
                crate::rubrics::validate_educational_value(&text, &educational_goals_vec) as f64
            }
        })
    );

    // Unwrap results from spawn_blocking (handles JoinError and returns inner value)
    let age_score = age_score_result.expect("Age validation task failed");
    let safety_issues = safety_issues_result.expect("Safety validation task failed");
    let edu_score = edu_score_result.expect("Educational validation task failed");

    tracing::info!(
        "Age appropriateness score for node {}: {:.2} (threshold: 0.7)",
        content.id, age_score
    );

    tracing::info!(
        "Safety validation for node {}: {} issues found",
        content.id, safety_issues.len()
    );
    if !safety_issues.is_empty() {
        tracing::warn!(
            "Safety issues for node {}: {:?}",
            content.id, safety_issues
        );
    }

    tracing::info!(
        "Educational value score for node {}: {:.2} (threshold: {:.2})",
        content.id, edu_score, config.validation.thresholds.min_educational_value_score
    );

    // Determine correction capability based on scores
    let correction_capability = determine_correction_capability(
        age_score,
        safety_issues.len(),
        edu_score,
    );

    // Generate correction suggestions
    let corrections = generate_corrections(
        &content.content,
        age_score,
        &safety_issues,
        edu_score,
        age_group,
    );

    // Determine if content is valid based on thresholds
    // Note: Educational value threshold is lower (configured in config.toml) because simple stories
    // for young children may not explicitly contain educational keywords but still have value
    let is_valid = age_score >= config.validation.thresholds.min_age_appropriate_score as f64
        && safety_issues.is_empty()
        && edu_score >= config.validation.thresholds.min_educational_value_score as f64;

    tracing::info!(
        "Validation result for node {}: is_valid={}, age_score={:.2}, safety_issues={}, edu_score={:.2}, correction_capability={:?}",
        content.id,
        is_valid,
        age_score,
        safety_issues.len(),
        edu_score,
        correction_capability
    );

    // If rejected, log WHY
    if !is_valid {
        let mut reasons = Vec::new();
        if age_score < config.validation.thresholds.min_age_appropriate_score as f64 {
            reasons.push(format!(
                "age_score {:.2} < {:.2}",
                age_score,
                config.validation.thresholds.min_age_appropriate_score
            ));
        }
        if !safety_issues.is_empty() {
            reasons.push(format!("{} safety issues", safety_issues.len()));
        }
        if edu_score < config.validation.thresholds.min_educational_value_score as f64 {
            reasons.push(format!(
                "edu_score {:.2} < {:.2}",
                edu_score,
                config.validation.thresholds.min_educational_value_score
            ));
        }

        tracing::warn!(
            "Content REJECTED for node {}: {}",
            content.id,
            reasons.join(", ")
        );
    }

    ValidationResult {
        is_valid,
        age_appropriate_score: age_score,
        safety_issues,
        educational_value_score: edu_score,
        corrections,
        correction_capability,
    }
}

/// Determines correction capability based on validation scores
///
/// # Thresholds:
/// - `CanFixLocally`: age_score >= 0.6, 1-3 safety issues, edu_score >= 0.5
/// - `NeedsRevision`: age_score 0.3-0.6, 4-6 safety issues
/// - `NoFixPossible`: age_score < 0.3, 7+ safety issues
fn determine_correction_capability(
    age_score: f64,
    safety_count: usize,
    edu_score: f64,
) -> CorrectionCapability {
    // NoFixPossible: Fundamental problems
    if age_score < 0.3 || safety_count >= 7 {
        return CorrectionCapability::NoFixPossible;
    }

    // NeedsRevision: Moderate to severe issues requiring structural changes
    if (age_score >= 0.3 && age_score < 0.6) || (safety_count >= 4 && safety_count < 7) {
        return CorrectionCapability::NeedsRevision;
    }

    // CanFixLocally: Minor issues fixable with word replacements
    if age_score >= 0.6 && safety_count <= 3 && edu_score >= 0.5 {
        return CorrectionCapability::CanFixLocally;
    }

    // Default to NeedsRevision for edge cases
    CorrectionCapability::NeedsRevision
}

/// Generates specific correction suggestions based on detected issues
fn generate_corrections(
    content: &Content,
    age_score: f64,
    safety_issues: &[String],
    edu_score: f64,
    age_group: AgeGroup,
) -> Vec<CorrectionSuggestion> {
    let mut corrections = Vec::new();

    // Generate age appropriateness corrections
    if age_score < 0.7 {
        corrections.extend(generate_age_corrections(content, age_score, age_group));
    }

    // Generate safety corrections
    if !safety_issues.is_empty() {
        corrections.extend(generate_safety_corrections(content, safety_issues));
    }

    // Generate educational value corrections
    if edu_score < 0.4 {
        corrections.extend(generate_educational_corrections(content, edu_score));
    }

    corrections
}

/// Generates corrections for age appropriateness issues
fn generate_age_corrections(
    content: &Content,
    age_score: f64,
    age_group: AgeGroup,
) -> Vec<CorrectionSuggestion> {
    let mut corrections = Vec::new();

    if age_score < 0.3 {
        corrections.push(CorrectionSuggestion {
            field: "content.text".to_string(),
            issue: format!("Content complexity far exceeds {:?} age group capabilities", age_group),
            suggestion: "Completely rewrite with simpler vocabulary and shorter sentences appropriate for target age".to_string(),
            severity: "high".to_string(),
        });
    } else if age_score < 0.5 {
        corrections.push(CorrectionSuggestion {
            field: "content.text".to_string(),
            issue: format!("Vocabulary and sentence structure too complex for {:?}", age_group),
            suggestion: "Simplify vocabulary by replacing advanced words with age-appropriate alternatives. Break long sentences into shorter ones.".to_string(),
            severity: "high".to_string(),
        });
    } else if age_score < 0.7 {
        corrections.push(CorrectionSuggestion {
            field: "content.text".to_string(),
            issue: format!("Some vocabulary or concepts may be challenging for {:?}", age_group),
            suggestion: "Review and simplify complex words. Ensure concepts are explained clearly.".to_string(),
            severity: "medium".to_string(),
        });
    }

    // Check choices for age appropriateness
    // OPTIMIZATION: Direct computation without intermediate Vec allocation
    if !content.choices.is_empty() {
        let total_choice_length: usize = content.choices.iter().map(|c| c.text.len()).sum();
        let avg_choice_length = total_choice_length / content.choices.len();

        let expected_max_length = match age_group {
            AgeGroup::_6To8 => 50,
            AgeGroup::_9To11 => 80,
            AgeGroup::_12To14 => 120,
            _ => 150,
        };

        if avg_choice_length > expected_max_length {
            corrections.push(CorrectionSuggestion {
                field: "content.choices".to_string(),
                issue: "Choice text too long for age group".to_string(),
                suggestion: format!("Shorten choice descriptions to under {} characters", expected_max_length),
                severity: "medium".to_string(),
            });
        }
    }

    corrections
}

/// Generates corrections for safety issues
fn generate_safety_corrections(
    _content: &Content,
    safety_issues: &[String],
) -> Vec<CorrectionSuggestion> {
    let mut corrections = Vec::new();

    for issue in safety_issues {
        let (suggestion, severity) = if issue.contains("violent") || issue.contains("violence") {
            (
                "Replace violent imagery with peaceful alternatives. Change 'sword' to 'stick', 'attack' to 'approach', 'blood' to 'paint' or remove entirely.".to_string(),
                "high".to_string(),
            )
        } else if issue.contains("scary") || issue.contains("terrifying") || issue.contains("horrifying") {
            (
                "Replace scary descriptions with gentle alternatives. Change 'terrifying' to 'surprising', 'horrifying' to 'unusual', 'monster' to 'creature'.".to_string(),
                "high".to_string(),
            )
        } else if issue.contains("weapon") || issue.contains("sword") || issue.contains("knife") {
            (
                "Remove weapon references or replace with non-violent tools: 'magic wand', 'flashlight', 'compass'.".to_string(),
                "high".to_string(),
            )
        } else if issue.contains("alcohol") || issue.contains("cigarette") || issue.contains("drug") {
            (
                "Remove all references to substances. Replace with healthy alternatives or remove the content entirely.".to_string(),
                "high".to_string(),
            )
        } else if issue.contains("dark") || issue.contains("shadow") {
            (
                "Reduce emphasis on darkness. Add light sources or change setting to daytime.".to_string(),
                "medium".to_string(),
            )
        } else {
            (
                format!("Address safety concern: {}", issue),
                "medium".to_string(),
            )
        };

        corrections.push(CorrectionSuggestion {
            field: "content.text".to_string(),
            issue: issue.clone(),
            suggestion,
            severity,
        });
    }

    corrections
}

/// Generates corrections for educational value issues
fn generate_educational_corrections(
    content: &Content,
    edu_score: f64,
) -> Vec<CorrectionSuggestion> {
    let mut corrections = Vec::new();

    if content.educational_content.is_none() {
        corrections.push(CorrectionSuggestion {
            field: "content.educational_content".to_string(),
            issue: "Missing educational content structure".to_string(),
            suggestion: "Add educational_content with topic, learning_objective, vocabulary_words, and educational_facts.".to_string(),
            severity: "medium".to_string(),
        });
    } else if edu_score < 0.4 {
        corrections.push(CorrectionSuggestion {
            field: "content.educational_content".to_string(),
            issue: "Weak educational value - content lacks meaningful learning opportunities".to_string(),
            suggestion: "Strengthen educational content by adding clear learning objectives, relevant vocabulary, and factual information.".to_string(),
            severity: "medium".to_string(),
        });
    } else if edu_score < 0.6 {
        corrections.push(CorrectionSuggestion {
            field: "content.educational_content".to_string(),
            issue: "Educational content present but could be enhanced".to_string(),
            suggestion: "Add more specific learning objectives or increase vocabulary richness. Include interesting facts related to the topic.".to_string(),
            severity: "low".to_string(),
        });
    }

    // Check if text content aligns with educational goals
    if let Some(edu_content) = &content.educational_content {
        if let Some(topic) = &edu_content.topic {
            if !content.text.to_lowercase().contains(&topic.to_lowercase()) {
                corrections.push(CorrectionSuggestion {
                    field: "content.text".to_string(),
                    issue: format!("Story text doesn't mention educational topic '{}'", topic),
                    suggestion: format!("Integrate the topic '{}' more naturally into the narrative", topic),
                    severity: "low".to_string(),
                });
            }
        }

        // Check vocabulary words usage
        // OPTIMIZATION: Pre-compute lowercase text and add early exit for performance
        if let Some(vocab_words) = &edu_content.vocabulary_words {
            let text_lower = content.text.to_lowercase();
            let threshold = vocab_words.len() / 2;
            let mut unused_count = 0;

            let unused_words: Vec<_> = vocab_words.iter()
                .filter(|word| {
                    // Early exit if already over threshold
                    if unused_count > threshold {
                        return false;
                    }
                    let is_unused = !text_lower.contains(&word.to_lowercase());
                    if is_unused {
                        unused_count += 1;
                    }
                    is_unused
                })
                .collect();

            if !unused_words.is_empty() && unused_words.len() > threshold {
                corrections.push(CorrectionSuggestion {
                    field: "content.text".to_string(),
                    issue: "Many vocabulary words not used in the text".to_string(),
                    suggestion: format!("Use these vocabulary words in context: {}",
                        unused_words.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                    severity: "low".to_string(),
                });
            }
        }
    }

    corrections
}

/// Build restricted words list from config and validation policy
///
/// Merges config file words and custom words based on merge mode.
fn build_restricted_words_list(
    validation_policy: Option<&ValidationPolicy>,
    config: &QualityControlConfig,
    language: &str,
) -> Option<Vec<String>> {
    let policy = validation_policy?;

    // Load config file words for this language
    let config_words = config.load_restricted_words(language)
        .unwrap_or_else(|e| {
            tracing::warn!(
                error = %e,
                language = %language,
                "Failed to load restricted words from config, using empty list"
            );
            Vec::new()
        });

    // Get custom words for this language from policy
    let custom_words = policy.custom_restricted_words
        .get(language)
        .cloned()
        .unwrap_or_default();

    // Merge based on merge mode
    let words = match policy.merge_mode {
        RestrictedWordsMergeMode::Replace => {
            tracing::debug!(
                custom_count = custom_words.len(),
                language = %language,
                "Using custom restricted words only (Replace mode)"
            );
            custom_words
        }
        RestrictedWordsMergeMode::Merge => {
            let mut merged = config_words.clone();
            merged.extend(custom_words.clone());
            merged.sort();
            merged.dedup();

            tracing::debug!(
                config_count = config_words.len(),
                custom_count = custom_words.len(),
                merged_count = merged.len(),
                language = %language,
                "Merged config and custom restricted words"
            );

            merged
        }
        RestrictedWordsMergeMode::ConfigOnly => {
            tracing::debug!(
                config_count = config_words.len(),
                language = %language,
                "Using config restricted words only (ConfigOnly mode)"
            );
            config_words
        }
    };

    if words.is_empty() {
        None
    } else {
        Some(words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_correction_capability_can_fix_locally() {
        let capability = determine_correction_capability(0.7, 2, 0.6);
        assert_eq!(capability, CorrectionCapability::CanFixLocally);
    }

    #[test]
    fn test_determine_correction_capability_needs_revision() {
        let capability = determine_correction_capability(0.4, 5, 0.5);
        assert_eq!(capability, CorrectionCapability::NeedsRevision);
    }

    #[test]
    fn test_determine_correction_capability_no_fix_possible() {
        let capability = determine_correction_capability(0.2, 8, 0.3);
        assert_eq!(capability, CorrectionCapability::NoFixPossible);
    }

    #[test]
    fn test_generate_corrections_returns_vec() {
        let content = Content {
            r#type: "test".to_string(),
            node_id: "test-1".to_string(),
            text: "Test content".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let safety_issues = vec!["violent imagery".to_string()];
        let corrections = generate_corrections(&content, 0.5, &safety_issues, 0.4, AgeGroup::_6To8);
        assert!(!corrections.is_empty());
    }
}
