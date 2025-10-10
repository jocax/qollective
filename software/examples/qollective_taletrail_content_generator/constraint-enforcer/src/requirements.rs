//! Required elements validation for constraint enforcement
//!
//! This module validates that content contains required educational elements
//! such as moral lessons, science facts, and other educational content.

use std::collections::HashSet;

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

/// Check if content contains all required elements
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
}
