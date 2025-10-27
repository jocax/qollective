//! Required elements validation for constraint enforcement
//!
//! This module validates that content contains required educational elements
//! such as moral lessons, science facts, and other educational content.
//!
//! It supports both exact phrase matching (legacy) and hybrid keyword + LLM
//! semantic matching for more flexible content validation.

use std::collections::HashSet;
use shared_types::Result;
use shared_types_llm::DynamicLlmClient;
use crate::config::ValidationConfig;
use crate::llm_semantic::{check_semantic_match, truncate_content};
use tracing::{debug, info};
use futures::future::join_all;

/// Moral lesson detection keywords
const MORAL_KEYWORDS: &[&str] = &[
    "learn", "lesson", "moral", "teach", "value", "friendship", "kindness",
    "honesty", "courage", "sharing", "helping", "teamwork", "cooperation",
    "empathy", "respect", "responsibility", "patience", "trust", "forgiveness",
];

/// Science fact detection keywords
const SCIENCE_KEYWORDS: &[&str] = &[
    "fact", "science", "discover", "research", "study", "evidence", "data",
    "scientific", "experiment", "observe", "theory", "hypothesis", "analysis",
    "biology", "chemistry", "physics", "astronomy", "geology", "ecology",
];

/// Educational content detection keywords
const EDUCATIONAL_KEYWORDS: &[&str] = &[
    "education", "knowledge", "understand", "explain", "demonstrate", "learning",
    "teaching", "instruction", "comprehend", "grasp", "realize", "recognize",
];

/// Match method used to validate a required element
#[derive(Debug, Clone, PartialEq)]
pub enum MatchMethod {
    /// Exact substring match (legacy behavior)
    ExactMatch,
    /// Keyword-based match with percentage threshold
    KeywordMatch { percentage: f32, keywords_found: Vec<String>, keywords_missing: Vec<String> },
    /// LLM semantic fallback match
    LlmSemanticMatch,
    /// No match found
    NoMatch { attempted_methods: Vec<String> },
}

/// Detailed information about element match
#[derive(Debug, Clone)]
pub struct ElementMatchDetail {
    /// The required element that was checked
    pub element: String,
    /// Whether the element was found
    pub found: bool,
    /// Method used to determine the match
    pub method: MatchMethod,
}

/// Extract keywords from a phrase by splitting on whitespace and filtering
///
/// This function:
/// 1. Splits phrase into words
/// 2. Filters out stopwords
/// 3. Filters out words shorter than min_length
/// 4. Returns lowercase keywords
///
/// # Arguments
///
/// * `phrase` - The phrase to extract keywords from
/// * `stopwords` - List of stopwords to filter out
/// * `min_length` - Minimum keyword length to keep
///
/// # Returns
///
/// Vector of extracted keywords (lowercase)
pub fn extract_keywords(phrase: &str, stopwords: &[String], min_length: usize) -> Vec<String> {
    let stopwords_set: HashSet<String> = stopwords.iter().map(|s| s.to_lowercase()).collect();

    phrase
        .split_whitespace()
        .map(|word| {
            // Remove punctuation and convert to lowercase
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| {
            // Keep words that:
            // - Are long enough
            // - Are not stopwords
            word.len() >= min_length && !stopwords_set.contains(word)
        })
        .collect()
}

/// Calculate keyword match percentage
///
/// # Arguments
///
/// * `element` - The required element phrase
/// * `content` - The content to search in
/// * `stopwords` - Stopwords to filter during keyword extraction
/// * `min_length` - Minimum keyword length
///
/// # Returns
///
/// Tuple of (percentage, found_keywords, missing_keywords)
/// - `percentage`: 0.0 to 1.0 representing match percentage
/// - `found_keywords`: Keywords from element that were found in content
/// - `missing_keywords`: Keywords from element that were NOT found in content
pub fn calculate_keyword_match_percentage(
    element: &str,
    content: &str,
    stopwords: &[String],
    min_length: usize,
) -> (f32, Vec<String>, Vec<String>) {
    let element_keywords = extract_keywords(element, stopwords, min_length);

    if element_keywords.is_empty() {
        // No keywords to match (all were stopwords or too short)
        // Consider this a match since we can't validate it with keywords
        return (1.0, Vec::new(), Vec::new());
    }

    let content_lower = content.to_lowercase();
    let mut found = Vec::new();
    let mut missing = Vec::new();

    for keyword in element_keywords {
        if content_lower.contains(&keyword) {
            found.push(keyword);
        } else {
            missing.push(keyword);
        }
    }

    let percentage = found.len() as f32 / (found.len() + missing.len()) as f32;
    (percentage, found, missing)
}

/// Calculate keyword match percentage using pre-computed lowercase content (optimized)
///
/// This version avoids redundant lowercase conversion by using pre-computed lowercase content.
///
/// # Arguments
///
/// * `element` - The required element phrase
/// * `lowercase_content` - Pre-computed lowercase content to search in
/// * `stopwords` - Stopwords to filter during keyword extraction
/// * `min_length` - Minimum keyword length
///
/// # Returns
///
/// Tuple of (percentage, found_keywords, missing_keywords)
/// - `percentage`: 0.0 to 1.0 representing match percentage
/// - `found_keywords`: Keywords from element that were found in content
/// - `missing_keywords`: Keywords from element that were NOT found in content
fn calculate_keyword_match_percentage_optimized(
    element: &str,
    lowercase_content: &str,
    stopwords: &[String],
    min_length: usize,
) -> (f32, Vec<String>, Vec<String>) {
    let element_keywords = extract_keywords(element, stopwords, min_length);

    if element_keywords.is_empty() {
        // No keywords to match (all were stopwords or too short)
        // Consider this a match since we can't validate it with keywords
        return (1.0, Vec::new(), Vec::new());
    }

    // Use pre-computed lowercase_content directly - no redundant lowercasing
    let mut found = Vec::new();
    let mut missing = Vec::new();

    for keyword in element_keywords {
        if lowercase_content.contains(&keyword) {
            found.push(keyword);
        } else {
            missing.push(keyword);
        }
    }

    let percentage = found.len() as f32 / (found.len() + missing.len()) as f32;
    (percentage, found, missing)
}

/// Check required elements using hybrid keyword + LLM semantic matching
///
/// This function implements a three-tier validation strategy:
/// 1. Try keyword-based matching first (fast, no LLM calls)
/// 2. If keyword match is below threshold and LLM fallback is enabled, use LLM
/// 3. Return detailed match information for each element
///
/// # Arguments
///
/// * `content` - The original content text (for LLM processing)
/// * `lowercase_content` - Pre-computed lowercase content (for keyword matching)
/// * `word_set` - Pre-computed word set (for fast lookups)
/// * `required` - List of required elements to find
/// * `validation_config` - Configuration for keyword matching and LLM fallback
/// * `llm_client` - Optional LLM client for semantic matching (if None, LLM fallback disabled)
///
/// # Returns
///
/// Tuple of (all_present: bool, match_details: Vec<ElementMatchDetail>)
/// - `all_present`: true if all required elements were found
/// - `match_details`: Detailed information about each element's match
pub async fn check_required_elements_hybrid(
    content: &str,
    lowercase_content: &str,
    word_set: &HashSet<String>,
    required: &[String],
    validation_config: &ValidationConfig,
    llm_client: Option<&dyn DynamicLlmClient>,
) -> Result<(bool, Vec<ElementMatchDetail>)> {
    // Handle empty cases
    if required.is_empty() {
        return Ok((true, Vec::new()));
    }

    if lowercase_content.is_empty() {
        let details: Vec<ElementMatchDetail> = required
            .iter()
            .map(|element| ElementMatchDetail {
                element: element.clone(),
                found: false,
                method: MatchMethod::NoMatch {
                    attempted_methods: vec!["empty_content".to_string()],
                },
            })
            .collect();
        return Ok((false, details));
    }

    // Determine which stopwords to use based on content language heuristics
    // Simple heuristic: if content contains common German words, use German stopwords
    // Use pre-computed lowercase_content to avoid redundant lowercasing
    let stopwords = if lowercase_content.contains("und")
        || lowercase_content.contains("der")
        || lowercase_content.contains("die")
    {
        &validation_config.stopwords_de
    } else {
        &validation_config.stopwords_en
    };

    let mut match_details = Vec::new();
    let truncated_content = truncate_content(content, validation_config.max_llm_content_length);
    let mut needs_llm: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();

    // Phase 1: Keyword matching (fast path - no LLM calls)
    for element in required {
        let (keyword_percentage, found_keywords, missing_keywords) = calculate_keyword_match_percentage_optimized(
            element,
            lowercase_content,
            stopwords,
            validation_config.min_keyword_length,
        );

        debug!(
            element = %element,
            keyword_percentage = keyword_percentage,
            threshold = validation_config.keyword_match_threshold,
            found_count = found_keywords.len(),
            missing_count = missing_keywords.len(),
            "Keyword match calculation"
        );

        if keyword_percentage >= validation_config.keyword_match_threshold {
            // Keyword match succeeded!
            info!(
                element = %element,
                percentage = keyword_percentage,
                "Element matched via keywords"
            );
            match_details.push(ElementMatchDetail {
                element: element.clone(),
                found: true,
                method: MatchMethod::KeywordMatch {
                    percentage: keyword_percentage,
                    keywords_found: found_keywords,
                    keywords_missing: missing_keywords,
                },
            });
        } else {
            // Save for LLM fallback phase
            needs_llm.push((element.clone(), found_keywords, missing_keywords));
        }
    }

    // Phase 2: Concurrent LLM fallback for elements that didn't match via keywords
    if !needs_llm.is_empty() && validation_config.enable_llm_fallback && llm_client.is_some() {
        let llm_client_ref = llm_client.unwrap();

        info!(
            count = needs_llm.len(),
            "Starting concurrent LLM semantic validation"
        );

        // Build futures for all elements needing LLM validation
        let llm_futures: Vec<_> = needs_llm.iter()
            .map(|(element, _, _)| {
                check_semantic_match(
                    llm_client_ref,
                    &truncated_content,
                    element,
                    &validation_config.llm_semantic_prompt,
                )
            })
            .collect();

        // Execute ALL LLM calls concurrently
        let llm_results = join_all(llm_futures).await;

        // Process results and build match details
        for ((element, _found_kw, _missing_kw), llm_result) in
            needs_llm.into_iter().zip(llm_results.into_iter()) {

            match llm_result {
                Ok(true) => {
                    // LLM match succeeded!
                    info!(
                        element = %element,
                        "Element matched via LLM semantic analysis"
                    );
                    match_details.push(ElementMatchDetail {
                        element,
                        found: true,
                        method: MatchMethod::LlmSemanticMatch,
                    });
                }
                Ok(false) => {
                    // LLM said no match
                    debug!(
                        element = %element,
                        "LLM semantic analysis: no match"
                    );
                    match_details.push(ElementMatchDetail {
                        element,
                        found: false,
                        method: MatchMethod::NoMatch {
                            attempted_methods: vec!["keyword".to_string(), "llm_semantic".to_string()],
                        },
                    });
                }
                Err(e) => {
                    // LLM call failed - log but don't fail the whole validation
                    debug!(
                        element = %element,
                        error = %e,
                        "LLM semantic fallback failed, treating as no match"
                    );
                    match_details.push(ElementMatchDetail {
                        element,
                        found: false,
                        method: MatchMethod::NoMatch {
                            attempted_methods: vec!["keyword".to_string(), "llm_semantic".to_string()],
                        },
                    });
                }
            }
        }
    } else {
        // No LLM fallback available or disabled - mark all as not found
        for (element, _, _) in needs_llm {
            match_details.push(ElementMatchDetail {
                element,
                found: false,
                method: MatchMethod::NoMatch {
                    attempted_methods: vec!["keyword".to_string()],
                },
            });
        }
    }

    let all_present = match_details.iter().all(|detail| detail.found);
    Ok((all_present, match_details))
}

/// Check if content contains all required elements (exact substring matching - legacy)
///
/// This is the original validation function that uses exact substring matching.
/// It is kept for backward compatibility and for cases where exact matching is desired.
///
/// # Arguments
///
/// * `content` - The content to validate
/// * `required` - List of required elements to find
///
/// # Returns
///
/// Tuple of (all_present: bool, missing: Vec<String>)
/// - `all_present`: true if all required elements are found
/// - `missing`: List of required elements not found in content
///
/// # Examples
///
/// ```
/// use constraint_enforcer::requirements::check_required_elements;
///
/// let content = "This story teaches about ocean conservation and includes science facts about dolphins.";
/// let required = vec!["ocean conservation".to_string(), "science fact".to_string()];
/// let (all_present, missing) = check_required_elements(content, &required);
/// assert!(all_present);
/// assert!(missing.is_empty());
/// ```
pub fn check_required_elements(content: &str, required: &[String]) -> (bool, Vec<String>) {
    // Handle empty cases
    if required.is_empty() {
        return (true, Vec::new());
    }

    if content.is_empty() {
        return (false, required.to_vec());
    }

    let content_lower = content.to_lowercase();
    let mut missing = Vec::new();

    for element in required {
        let element_lower = element.to_lowercase();
        if !content_lower.contains(&element_lower) {
            missing.push(element.clone());
        }
    }

    let all_present = missing.is_empty();
    (all_present, missing)
}

/// Validate if content contains a moral lesson
///
/// Checks for presence of moral lesson keywords and common moral themes.
///
/// # Arguments
///
/// * `content` - The content to validate
///
/// # Returns
///
/// `true` if moral lesson content is detected, `false` otherwise
///
/// # Examples
///
/// ```
/// use constraint_enforcer::requirements::validate_moral_lesson;
///
/// let content = "The story teaches children the value of friendship and honesty.";
/// assert!(validate_moral_lesson(content));
/// ```
pub fn validate_moral_lesson(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    let content_lower = content.to_lowercase();
    let keyword_count = MORAL_KEYWORDS
        .iter()
        .filter(|&&keyword| content_lower.contains(keyword))
        .count();

    // Require at least 2 moral keywords for higher confidence
    keyword_count >= 2
}

/// Validate if content contains science facts
///
/// Checks for presence of science fact keywords and scientific content indicators.
///
/// # Arguments
///
/// * `content` - The content to validate
///
/// # Returns
///
/// `true` if science fact content is detected, `false` otherwise
///
/// # Examples
///
/// ```
/// use constraint_enforcer::requirements::validate_science_facts;
///
/// let content = "Scientific research shows that dolphins use echolocation to navigate.";
/// assert!(validate_science_facts(content));
/// ```
pub fn validate_science_facts(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    let content_lower = content.to_lowercase();
    let keyword_count = SCIENCE_KEYWORDS
        .iter()
        .filter(|&&keyword| content_lower.contains(keyword))
        .count();

    // Require at least 2 science keywords for higher confidence
    keyword_count >= 2
}

/// Extract educational elements from content
///
/// Identifies and extracts educational elements based on keyword analysis.
///
/// # Arguments
///
/// * `content` - The content to analyze
///
/// # Returns
///
/// Vector of educational element types found in the content
///
/// # Examples
///
/// ```
/// use constraint_enforcer::requirements::extract_educational_elements;
///
/// let content = "This educational story teaches a moral lesson about friendship with scientific facts about marine biology.";
/// let elements = extract_educational_elements(content);
/// assert!(elements.contains(&"moral_lesson".to_string()));
/// assert!(elements.contains(&"science_facts".to_string()));
/// assert!(elements.contains(&"educational_content".to_string()));
/// ```
pub fn extract_educational_elements(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }

    let content_lower = content.to_lowercase();
    let mut elements = HashSet::new();

    // Check for moral lessons
    if MORAL_KEYWORDS
        .iter()
        .any(|&keyword| content_lower.contains(keyword))
    {
        elements.insert("moral_lesson".to_string());
    }

    // Check for science facts
    if SCIENCE_KEYWORDS
        .iter()
        .any(|&keyword| content_lower.contains(keyword))
    {
        elements.insert("science_facts".to_string());
    }

    // Check for general educational content
    if EDUCATIONAL_KEYWORDS
        .iter()
        .any(|&keyword| content_lower.contains(keyword))
    {
        elements.insert("educational_content".to_string());
    }

    elements.into_iter().collect()
}

/// Check if content contains specific educational phrases
///
/// More strict validation that requires exact phrase matching.
///
/// # Arguments
///
/// * `content` - The content to validate
/// * `phrases` - List of exact phrases to find
///
/// # Returns
///
/// Vector of phrases that were found in the content
pub fn find_educational_phrases(content: &str, phrases: &[String]) -> Vec<String> {
    if content.is_empty() || phrases.is_empty() {
        return Vec::new();
    }

    let content_lower = content.to_lowercase();
    phrases
        .iter()
        .filter(|phrase| {
            let phrase_lower = phrase.to_lowercase();
            content_lower.contains(&phrase_lower)
        })
        .cloned()
        .collect()
}

/// Validate social skills content
///
/// Checks for content related to social skills development.
///
/// # Arguments
///
/// * `content` - The content to validate
///
/// # Returns
///
/// `true` if social skills content is detected, `false` otherwise
pub fn validate_social_skills(content: &str) -> bool {
    const SOCIAL_SKILLS_KEYWORDS: &[&str] = &[
        "teamwork",
        "cooperation",
        "empathy",
        "communication",
        "problem-solving",
        "collaborate",
        "social",
        "interact",
        "relationship",
        "together",
    ];

    if content.is_empty() {
        return false;
    }

    let content_lower = content.to_lowercase();
    SOCIAL_SKILLS_KEYWORDS
        .iter()
        .filter(|&&keyword| content_lower.contains(keyword))
        .count()
        >= 2
}

/// Validate environmental content
///
/// Checks for content related to environmental education.
///
/// # Arguments
///
/// * `content` - The content to validate
///
/// # Returns
///
/// `true` if environmental content is detected, `false` otherwise
pub fn validate_environmental_content(content: &str) -> bool {
    const ENVIRONMENTAL_KEYWORDS: &[&str] = &[
        "conservation",
        "recycling",
        "sustainability",
        "ecosystem",
        "nature",
        "environment",
        "protect",
        "preserve",
        "wildlife",
        "habitat",
    ];

    if content.is_empty() {
        return false;
    }

    let content_lower = content.to_lowercase();
    ENVIRONMENTAL_KEYWORDS
        .iter()
        .filter(|&&keyword| content_lower.contains(keyword))
        .count()
        >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_required_elements_all_present() {
        let content = "This story teaches about ocean conservation and includes science facts about dolphins.";
        let required = vec!["ocean conservation".to_string(), "science fact".to_string()];
        let (all_present, missing) = check_required_elements(content, &required);
        assert!(all_present);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_check_required_elements_some_missing() {
        let content = "This story is about dolphins.";
        let required = vec![
            "ocean conservation".to_string(),
            "science fact".to_string(),
        ];
        let (all_present, missing) = check_required_elements(content, &required);
        assert!(!all_present);
        assert_eq!(missing.len(), 2);
    }

    #[test]
    fn test_check_required_elements_case_insensitive() {
        let content = "OCEAN CONSERVATION is important";
        let required = vec!["ocean conservation".to_string()];
        let (all_present, missing) = check_required_elements(content, &required);
        assert!(all_present);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_check_required_elements_empty_content() {
        let required = vec!["test".to_string()];
        let (all_present, missing) = check_required_elements("", &required);
        assert!(!all_present);
        assert_eq!(missing, required);
    }

    #[test]
    fn test_check_required_elements_empty_required() {
        let (all_present, missing) = check_required_elements("some content", &[]);
        assert!(all_present);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_validate_moral_lesson() {
        let content = "This story teaches children the value of friendship and honesty.";
        assert!(validate_moral_lesson(content));
    }

    #[test]
    fn test_validate_moral_lesson_insufficient_keywords() {
        let content = "This story is about dolphins.";
        assert!(!validate_moral_lesson(content));
    }

    #[test]
    fn test_validate_science_facts() {
        let content = "Scientific research shows that dolphins use echolocation.";
        assert!(validate_science_facts(content));
    }

    #[test]
    fn test_validate_science_facts_insufficient_keywords() {
        let content = "Dolphins are interesting.";
        assert!(!validate_science_facts(content));
    }

    #[test]
    fn test_extract_educational_elements() {
        let content = "This educational story teaches a moral lesson with scientific facts.";
        let elements = extract_educational_elements(content);
        assert!(elements.contains(&"moral_lesson".to_string()));
        assert!(elements.contains(&"science_facts".to_string()));
        assert!(elements.contains(&"educational_content".to_string()));
    }

    #[test]
    fn test_extract_educational_elements_empty() {
        let elements = extract_educational_elements("");
        assert!(elements.is_empty());
    }

    #[test]
    fn test_find_educational_phrases() {
        let content = "Ocean conservation is important for marine life.";
        let phrases = vec!["ocean conservation".to_string(), "marine life".to_string()];
        let found = find_educational_phrases(content, &phrases);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_validate_social_skills() {
        let content = "The story emphasizes teamwork and cooperation among friends.";
        assert!(validate_social_skills(content));
    }

    #[test]
    fn test_validate_environmental_content() {
        let content = "Learn about conservation and protecting wildlife habitats.";
        assert!(validate_environmental_content(content));
    }

    // Tests for hybrid validation functions

    #[test]
    fn test_extract_keywords_basic() {
        let stopwords = vec!["the".to_string(), "and".to_string(), "or".to_string()];
        let keywords = extract_keywords("Industrie und Natur Dilemma", &stopwords, 3);
        assert_eq!(keywords, vec!["industrie", "natur", "dilemma"]);
    }

    #[test]
    fn test_extract_keywords_filters_stopwords() {
        let stopwords = vec!["the".to_string(), "and".to_string(), "of".to_string()];
        let keywords = extract_keywords("The beauty of nature and wildlife", &stopwords, 3);
        assert_eq!(keywords, vec!["beauty", "nature", "wildlife"]);
    }

    #[test]
    fn test_extract_keywords_min_length() {
        let stopwords = vec![];
        let keywords = extract_keywords("A big elephant or a small ant", &stopwords, 4);
        // Only "elephant" and "small" are >= 4 chars
        assert_eq!(keywords, vec!["elephant", "small"]);
    }

    #[test]
    fn test_extract_keywords_with_punctuation() {
        let stopwords = vec![];
        let keywords = extract_keywords("Hello, world! This is great.", &stopwords, 3);
        assert_eq!(keywords, vec!["hello", "world", "this", "great"]);
    }

    #[test]
    fn test_calculate_keyword_match_percentage_full_match() {
        let stopwords = vec!["und".to_string()];
        let (percentage, found, missing) = calculate_keyword_match_percentage(
            "Industrie und Natur",
            "Die Fabrik (Industrie) vergiftet die Natur",
            &stopwords,
            3,
        );
        assert_eq!(percentage, 1.0); // Both keywords found
        assert_eq!(found, vec!["industrie", "natur"]);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_calculate_keyword_match_percentage_partial_match() {
        let stopwords = vec!["und".to_string()];
        let (percentage, found, missing) = calculate_keyword_match_percentage(
            "Industrie und Natur Dilemma",
            "Die Fabrik (Industrie) vergiftet die Natur",
            &stopwords,
            3,
        );
        // 2 out of 3 keywords found (industrie, natur) missing (dilemma)
        assert_eq!(percentage, 2.0 / 3.0);
        assert_eq!(found.len(), 2);
        assert_eq!(missing, vec!["dilemma"]);
    }

    #[test]
    fn test_calculate_keyword_match_percentage_no_match() {
        let stopwords = vec![];
        let (percentage, found, missing) = calculate_keyword_match_percentage(
            "ocean conservation",
            "The cat sat on the mat",
            &stopwords,
            3,
        );
        assert_eq!(percentage, 0.0);
        assert!(found.is_empty());
        assert_eq!(missing, vec!["ocean", "conservation"]);
    }

    #[test]
    fn test_calculate_keyword_match_percentage_all_stopwords() {
        let stopwords = vec!["the".to_string(), "and".to_string(), "or".to_string()];
        let (percentage, found, missing) = calculate_keyword_match_percentage(
            "the and or",
            "some content here",
            &stopwords,
            3,
        );
        // No keywords after filtering stopwords - should return 1.0 (match by default)
        assert_eq!(percentage, 1.0);
        assert!(found.is_empty());
        assert!(missing.is_empty());
    }
}
