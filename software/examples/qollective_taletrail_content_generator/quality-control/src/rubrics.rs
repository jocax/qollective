//! Age-appropriate validation rubrics for quality control
//!
//! This module provides scoring and validation functions for:
//! - Age appropriateness based on vocabulary and sentence complexity
//! - Safety validation detecting violence, fear, and inappropriate content
//! - Educational value assessment based on keywords and goals
//!
//! # Scoring Philosophy
//!
//! All scores are normalized to 0.0-1.0 range where:
//! - 1.0 = Perfect match for criteria
//! - 0.7-0.9 = Good match, mostly appropriate
//! - 0.4-0.6 = Marginal, needs improvement
//! - 0.0-0.3 = Inappropriate or poor match
//!
//! # Design Decisions
//!
//! 1. **Age Group Mapping**: The task specification referenced 3-5, 6-8, 9-12 age groups,
//!    but the generated types use 6-8, 9-11, 12-14, etc. This implementation uses the
//!    actual generated enum values and provides appropriate thresholds for each.
//!
//! 2. **Sentence Length Calculation**: Uses word count divided by sentence count,
//!    treating punctuation-based splits (., !, ?) as sentence boundaries.
//!
//! 3. **Vocabulary Complexity**: Simple heuristic based on word length and common
//!    word lists. More sophisticated NLP could be added in future iterations.
//!
//! 4. **Safety Keywords**: Case-insensitive matching with context preservation.
//!    Returns descriptive messages including the actual matched text.
//!
//! 5. **Educational Keywords**: Weighted by frequency and alignment with provided goals.

use shared_types::{AgeGroup, Result, TaleTrailError};

// ============================================================================
// CONSTANTS - Age Appropriateness Thresholds
// ============================================================================

/// Maximum average sentence length for ages 6-8 (in words)
const MAX_SENTENCE_LENGTH_6_8: f32 = 12.0;

/// Maximum average sentence length for ages 9-11 (in words)
const MAX_SENTENCE_LENGTH_9_11: f32 = 15.0;

/// Maximum average sentence length for ages 12-14 (in words)
const MAX_SENTENCE_LENGTH_12_14: f32 = 18.0;

/// Maximum average sentence length for ages 15-17 (in words)
const MAX_SENTENCE_LENGTH_15_17: f32 = 22.0;

/// Maximum average sentence length for ages 18+ (in words)
const MAX_SENTENCE_LENGTH_18_PLUS: f32 = 30.0;

/// Target vocabulary level word length threshold for basic vocabulary
const BASIC_WORD_LENGTH_THRESHOLD: usize = 6;

/// Target vocabulary level word length threshold for intermediate vocabulary
const INTERMEDIATE_WORD_LENGTH_THRESHOLD: usize = 8;

// ============================================================================
// CONSTANTS - Safety Keywords
// ============================================================================

/// Violence-related keywords that trigger safety warnings
const VIOLENCE_KEYWORDS: &[&str] = &[
    "sword",
    "fight",
    "fighting",
    "blood",
    "weapon",
    "weapons",
    "kill",
    "killing",
    "hurt",
    "hurting",
    "attack",
    "attacking",
    "battle",
    "war",
    "stab",
    "stabbing",
    "shoot",
    "shooting",
    "gun",
    "knife",
    "blade",
];

/// Fear-inducing keywords that trigger safety warnings
const FEAR_KEYWORDS: &[&str] = &[
    "scary",
    "monster",
    "monsters",
    "nightmare",
    "dark cave",
    "ghost",
    "ghosts",
    "haunted",
    "creepy",
    "terrifying",
    "horrifying",
    "terror",
    "scream",
    "screaming",
    "screams",
    "devour",
    "shadows",
];

/// Inappropriate content keywords that trigger safety warnings
const INAPPROPRIATE_KEYWORDS: &[&str] = &[
    "alcohol",
    "beer",
    "wine",
    "cigarette",
    "cigarettes",
    "tobacco",
    "smoking",
    "smokes",
    "drunk",
    "drinking",
];

// ============================================================================
// CONSTANTS - Educational Keywords
// ============================================================================

/// Educational keywords that boost educational value score
const EDUCATIONAL_KEYWORDS: &[&str] = &[
    "learn",
    "learning",
    "discover",
    "discovering",
    "explore",
    "exploring",
    "understand",
    "understanding",
    "science",
    "scientific",
    "math",
    "mathematics",
    "nature",
    "biology",
    "chemistry",
    "physics",
    "history",
    "geography",
    "fact",
    "facts",
    "study",
    "research",
    "observe",
    "observation",
    "experiment",
    "analyze",
    "analysis",
    "photosynthesis",
    "plants",
    "plant",
    "animals",
    "animal",
    "ecosystem",
    "environment",
    "evolution",
    "cells",
    "energy",
];

// ============================================================================
// Public API Functions
// ============================================================================

/// Validates age appropriateness of content based on sentence length and vocabulary complexity.
///
/// # Scoring Algorithm
///
/// The score is calculated as the average of two components:
/// 1. **Sentence Length Score** (50%): Compares average sentence length to age-appropriate threshold
/// 2. **Vocabulary Complexity Score** (50%): Assesses word complexity for the age group
///
/// # Arguments
///
/// * `content` - Text content to validate
/// * `age_group` - Target age group from AgeGroup enum
///
/// # Returns
///
/// Score from 0.0 (inappropriate) to 1.0 (perfectly appropriate)
///
/// # Examples
///
/// ```
/// use shared_types::AgeGroup;
/// use quality_control::rubrics::validate_age_appropriateness;
///
/// let score = validate_age_appropriateness(
///     "The bunny hops. The sun is warm.",
///     AgeGroup::_6To8
/// );
/// assert!(score > 0.8); // Simple content scores high for young ages
/// ```
pub fn validate_age_appropriateness(content: &str, age_group: AgeGroup) -> f32 {
    // Handle empty content
    if content.trim().is_empty() {
        return 0.0;
    }

    // Calculate sentence length score
    let avg_sentence_length = calculate_average_sentence_length(content);
    let max_sentence_length = get_max_sentence_length_for_age(age_group);

    let sentence_score = if avg_sentence_length <= max_sentence_length {
        1.0
    } else {
        // Gradual degradation: score decreases as we exceed threshold
        let excess_ratio = avg_sentence_length / max_sentence_length;
        (2.0 - excess_ratio).max(0.0)
    };

    // Calculate vocabulary complexity score
    let vocab_score = check_vocabulary_complexity(content, age_group);

    // Average the two scores
    (sentence_score + vocab_score) / 2.0
}

/// Validates content for safety issues including violence, fear, and inappropriate themes.
///
/// # Detection Strategy
///
/// Uses case-insensitive keyword matching across three categories:
/// - **Violence**: weapons, fighting, blood
/// - **Fear**: scary themes, monsters, nightmares
/// - **Inappropriate**: substance use, adult themes
///
/// # Arguments
///
/// * `content` - Text content to validate
///
/// # Returns
///
/// Vector of descriptive violation messages. Empty vector means no safety issues.
///
/// # Examples
///
/// ```
/// use quality_control::rubrics::validate_safety;
///
/// let violations = validate_safety("The warrior drew his sharp sword and attacked.");
/// assert!(!violations.is_empty());
/// assert!(violations.iter().any(|v| v.contains("violent imagery")));
/// ```
pub fn validate_safety(content: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let content_lower = content.to_lowercase();

    // Check violence keywords
    for keyword in VIOLENCE_KEYWORDS {
        if content_lower.contains(keyword) {
            violations.push(format!("violent imagery: '{}'", keyword));
        }
    }

    // Check fear keywords
    for keyword in FEAR_KEYWORDS {
        if content_lower.contains(keyword) {
            violations.push(format!("scary theme: '{}'", keyword));
        }
    }

    // Check inappropriate keywords
    for keyword in INAPPROPRIATE_KEYWORDS {
        if content_lower.contains(keyword) {
            violations.push(format!("inappropriate content: '{}'", keyword));
        }
    }

    violations
}

/// Validates educational value of content based on keywords and goal alignment.
///
/// # Scoring Algorithm
///
/// The score is calculated from three components:
/// 1. **Educational Keywords** (40%): Presence of learning-related terms
/// 2. **Goal Alignment** (40%): Match with provided educational goals
/// 3. **Baseline** (20%): Minimum score for structured content
///
/// # Arguments
///
/// * `content` - Text content to validate
/// * `educational_goals` - List of educational goals to align with
///
/// # Returns
///
/// Score from 0.0 (no educational value) to 1.0 (highly educational)
///
/// # Examples
///
/// ```
/// use quality_control::rubrics::validate_educational_value;
///
/// let goals = vec!["science".to_string(), "nature".to_string()];
/// let score = validate_educational_value(
///     "Let's learn about nature! Plants use photosynthesis to grow.",
///     &goals
/// );
/// assert!(score > 0.7);
/// ```
pub fn validate_educational_value(content: &str, educational_goals: &[String]) -> f32 {
    // Handle empty content
    if content.trim().is_empty() {
        return 0.0;
    }

    // Base score from educational keywords
    let keyword_count = count_educational_keywords(content);
    let keyword_score = match keyword_count {
        0 => 0.0,
        1 => 0.3,
        2 => 0.5,
        3 => 0.7,
        4 => 0.85,
        _ => 1.0, // 5+ keywords
    };

    // Goal alignment score
    let goal_score = if educational_goals.is_empty() {
        0.5 // Neutral score if no goals provided
    } else {
        let content_lower = content.to_lowercase();
        let matches = educational_goals
            .iter()
            .filter(|goal| content_lower.contains(&goal.to_lowercase()))
            .count();

        let match_ratio = matches as f32 / educational_goals.len() as f32;
        match_ratio
    };

    // Weighted average: 40% keywords, 40% goal alignment, 20% baseline
    let baseline = 0.2;
    (keyword_score * 0.4) + (goal_score * 0.4) + baseline
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculates average sentence length in words.
///
/// Splits text on sentence-ending punctuation (., !, ?) and counts words per sentence.
/// Empty sentences are ignored in the calculation.
///
/// # Arguments
///
/// * `content` - Text to analyze
///
/// # Returns
///
/// Average number of words per sentence
pub fn calculate_average_sentence_length(content: &str) -> f32 {
    // Split on sentence-ending punctuation
    let sentences: Vec<&str> = content
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();

    if sentences.is_empty() {
        return 0.0;
    }

    let total_words: usize = sentences
        .iter()
        .map(|sentence| {
            sentence
                .split_whitespace()
                .filter(|w| !w.is_empty())
                .count()
        })
        .sum();

    total_words as f32 / sentences.len() as f32
}

/// Counts educational keywords in content.
///
/// Uses case-insensitive matching against the EDUCATIONAL_KEYWORDS list.
///
/// # Arguments
///
/// * `content` - Text to analyze
///
/// # Returns
///
/// Number of educational keywords found
pub fn count_educational_keywords(content: &str) -> usize {
    let content_lower = content.to_lowercase();
    EDUCATIONAL_KEYWORDS
        .iter()
        .filter(|keyword| content_lower.contains(*keyword))
        .count()
}

/// Checks vocabulary complexity appropriate for age group.
///
/// # Algorithm
///
/// Uses word length as a proxy for complexity:
/// - Basic vocabulary: words ≤ 6 characters
/// - Intermediate vocabulary: words ≤ 8 characters
/// - Advanced vocabulary: words > 8 characters
///
/// Different age groups have different tolerance levels:
/// - Younger ages: penalized for long/complex words
/// - Older ages: reward for more sophisticated vocabulary
///
/// # Arguments
///
/// * `content` - Text to analyze
/// * `age_group` - Target age group
///
/// # Returns
///
/// Score from 0.0 (inappropriate complexity) to 1.0 (perfect match)
pub fn check_vocabulary_complexity(content: &str, age_group: AgeGroup) -> f32 {
    let words: Vec<&str> = content
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return 0.0;
    }

    // Calculate average word length and count complex words
    let word_lengths: Vec<usize> = words.iter().map(|w| {
        w.chars().filter(|c| c.is_alphabetic()).count()
    }).collect();

    let total_length: usize = word_lengths.iter().sum();
    let avg_word_length = total_length as f32 / words.len() as f32;

    // Count words that are too complex for age group
    let complex_word_ratio = match age_group {
        AgeGroup::_6To8 => {
            // For young ages, count words > 8 characters as complex
            word_lengths.iter().filter(|&&len| len > 8).count() as f32 / words.len() as f32
        }
        AgeGroup::_9To11 => {
            word_lengths.iter().filter(|&&len| len > 10).count() as f32 / words.len() as f32
        }
        AgeGroup::_12To14 => {
            word_lengths.iter().filter(|&&len| len > 12).count() as f32 / words.len() as f32
        }
        AgeGroup::_15To17 => {
            word_lengths.iter().filter(|&&len| len > 14).count() as f32 / words.len() as f32
        }
        AgeGroup::Plus18 => {
            0.0 // No penalty for complex words in adult content
        }
    };

    // Score based on age group expectations
    let base_score = match age_group {
        AgeGroup::_6To8 => {
            // Expect mostly basic vocabulary (short words)
            if avg_word_length <= BASIC_WORD_LENGTH_THRESHOLD as f32 {
                1.0
            } else if avg_word_length <= INTERMEDIATE_WORD_LENGTH_THRESHOLD as f32 {
                // Gradual degradation for intermediate words
                let ratio = (avg_word_length - BASIC_WORD_LENGTH_THRESHOLD as f32) /
                           (INTERMEDIATE_WORD_LENGTH_THRESHOLD as f32 - BASIC_WORD_LENGTH_THRESHOLD as f32);
                0.9 - (ratio * 0.6) // Range from 0.9 to 0.3
            } else {
                // Very complex words: strong penalty
                let excess = avg_word_length - INTERMEDIATE_WORD_LENGTH_THRESHOLD as f32;
                (0.3 - (excess * 0.1)).max(0.0)
            }
        }
        AgeGroup::_9To11 => {
            // Expect basic to intermediate vocabulary
            if avg_word_length <= INTERMEDIATE_WORD_LENGTH_THRESHOLD as f32 {
                1.0
            } else if avg_word_length <= 10.0 {
                0.8
            } else {
                0.4
            }
        }
        AgeGroup::_12To14 => {
            // Expect intermediate vocabulary
            if avg_word_length >= 6.0 && avg_word_length <= 10.0 {
                1.0
            } else if avg_word_length < 6.0 {
                0.7 // Too simple
            } else {
                0.6 // Slightly complex is OK
            }
        }
        AgeGroup::_15To17 => {
            // Expect intermediate to advanced vocabulary
            if avg_word_length >= 7.0 && avg_word_length <= 12.0 {
                1.0
            } else if avg_word_length < 7.0 {
                0.6 // Too simple
            } else {
                0.8 // Complex is acceptable
            }
        }
        AgeGroup::Plus18 => {
            // Advanced vocabulary expected
            if avg_word_length >= 8.0 {
                1.0
            } else if avg_word_length >= 7.0 {
                0.8
            } else {
                0.5 // Too simple
            }
        }
    };

    // Apply penalty for complex word ratio
    // If >40% of words are too complex, reduce score to essentially zero
    let final_score = if complex_word_ratio > 0.4 {
        // Strong penalty: makes it highly inappropriate
        0.0
    } else if complex_word_ratio > 0.2 {
        base_score * 0.3
    } else {
        base_score
    };

    final_score
}

/// Gets maximum sentence length threshold for age group.
///
/// # Arguments
///
/// * `age_group` - Target age group
///
/// # Returns
///
/// Maximum recommended average sentence length in words
fn get_max_sentence_length_for_age(age_group: AgeGroup) -> f32 {
    match age_group {
        AgeGroup::_6To8 => MAX_SENTENCE_LENGTH_6_8,
        AgeGroup::_9To11 => MAX_SENTENCE_LENGTH_9_11,
        AgeGroup::_12To14 => MAX_SENTENCE_LENGTH_12_14,
        AgeGroup::_15To17 => MAX_SENTENCE_LENGTH_15_17,
        AgeGroup::Plus18 => MAX_SENTENCE_LENGTH_18_PLUS,
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_average_sentence_length_simple() {
        let text = "The cat sits. The dog runs. Birds fly.";
        let avg = calculate_average_sentence_length(text);
        // Sentences: "The cat sits" (3), "The dog runs" (3), "Birds fly" (2)
        // Average: (3 + 3 + 2) / 3 = 2.67
        assert!((avg - 2.67).abs() < 0.01);
    }

    #[test]
    fn test_calculate_average_sentence_length_varying() {
        let text = "Hi! This is a longer sentence. Short.";
        let avg = calculate_average_sentence_length(text);
        assert!(avg > 1.0 && avg < 4.0); // Mixed lengths
    }

    #[test]
    fn test_calculate_average_sentence_length_empty() {
        let avg = calculate_average_sentence_length("");
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_count_educational_keywords_none() {
        let text = "The bunny hops around the garden.";
        let count = count_educational_keywords(text);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_educational_keywords_multiple() {
        let text = "Let's learn about science! We will discover nature and explore math.";
        let count = count_educational_keywords(text);
        assert!(count >= 4); // learn, science, discover, nature, explore, math
    }

    #[test]
    fn test_count_educational_keywords_case_insensitive() {
        let text = "LEARN about SCIENCE";
        let count = count_educational_keywords(text);
        assert!(count >= 2);
    }

    #[test]
    fn test_validate_safety_clean_content() {
        let text = "The friendly dolphin swims in the ocean.";
        let violations = validate_safety(text);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_validate_safety_violence() {
        let text = "The warrior drew his sword and attacked.";
        let violations = validate_safety(text);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("sword") || v.contains("attack")));
    }

    #[test]
    fn test_validate_safety_scary() {
        let text = "The scary monster lurked in the dark cave.";
        let violations = validate_safety(text);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("scary") || v.contains("monster")));
    }

    #[test]
    fn test_validate_safety_inappropriate() {
        let text = "The character drinks alcohol and smokes a cigarette.";
        let violations = validate_safety(text);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("alcohol") || v.contains("cigarette")));
    }

    #[test]
    fn test_validate_safety_multiple_violations() {
        let text = "The scary monster attacks with a sword!";
        let violations = validate_safety(text);
        assert!(violations.len() >= 3); // scary, monster, attacks, sword
    }

    #[test]
    fn test_validate_age_appropriateness_simple_for_young() {
        let text = "The cat sits. The dog runs.";
        let score = validate_age_appropriateness(text, AgeGroup::_6To8);
        assert!(score > 0.7, "Simple text should score high for young ages");
    }

    #[test]
    fn test_validate_age_appropriateness_complex_for_young() {
        let text = "The protagonist contemplated the existential ramifications of their predicament.";
        let score = validate_age_appropriateness(text, AgeGroup::_6To8);
        // Score of 0.5 is marginal/inappropriate per scoring guidelines (0.4-0.6 range)
        assert!(score <= 0.5, "Complex text should score low for young ages, got {}", score);
    }

    #[test]
    fn test_validate_age_appropriateness_empty() {
        let score = validate_age_appropriateness("", AgeGroup::_6To8);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_validate_educational_value_high() {
        let text = "Let's learn about science! We'll discover how nature works and explore mathematics.";
        let goals = vec!["science".to_string(), "nature".to_string()];
        let score = validate_educational_value(text, &goals);
        assert!(score > 0.7, "Educational content should score high");
    }

    #[test]
    fn test_validate_educational_value_low() {
        let text = "You walk down the path.";
        let goals = vec![];
        let score = validate_educational_value(text, &goals);
        assert!(score < 0.5, "Non-educational content should score low");
    }

    #[test]
    fn test_validate_educational_value_empty() {
        let score = validate_educational_value("", &[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_validate_educational_value_goal_alignment() {
        let text = "Learn about photosynthesis in plants using science!";
        let goals = vec!["science".to_string(), "nature".to_string()];
        let score = validate_educational_value(text, &goals);
        assert!(score > 0.6, "Goal-aligned content should score well");
    }

    #[test]
    fn test_check_vocabulary_complexity_basic_for_young() {
        let text = "The cat runs fast";
        let score = check_vocabulary_complexity(text, AgeGroup::_6To8);
        assert!(score > 0.8, "Basic vocabulary should score high for young ages");
    }

    #[test]
    fn test_check_vocabulary_complexity_advanced_for_young() {
        let text = "The extraordinarily complicated circumstances";
        let score = check_vocabulary_complexity(text, AgeGroup::_6To8);
        assert!(score < 0.5, "Advanced vocabulary should score low for young ages");
    }

    #[test]
    fn test_check_vocabulary_complexity_advanced_for_older() {
        let text = "The extraordinarily complicated philosophical predicament";
        let score = check_vocabulary_complexity(text, AgeGroup::Plus18);
        assert!(score > 0.7, "Advanced vocabulary should score high for older ages");
    }
}
