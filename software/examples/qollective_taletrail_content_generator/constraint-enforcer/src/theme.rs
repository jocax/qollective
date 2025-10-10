//! Theme consistency validation module
//!
//! Validates thematic consistency across story nodes to detect theme drift
//! and ensure coherent storytelling.

use shared_types::ContentNode;
use std::collections::HashMap;

/// Theme keyword categories with associated keywords
const THEME_CATEGORIES: &[(&str, &[&str])] = &[
    (
        "ocean",
        &[
            "ocean",
            "water",
            "sea",
            "wave",
            "fish",
            "coral",
            "dive",
            "submarine",
            "marine",
            "whale",
            "shark",
            "dolphin",
            "reef",
            "underwater",
            "tide",
        ],
    ),
    (
        "space",
        &[
            "space",
            "star",
            "planet",
            "rocket",
            "astronaut",
            "galaxy",
            "orbit",
            "moon",
            "cosmos",
            "universe",
            "satellite",
            "asteroid",
            "nebula",
            "spacecraft",
            "alien",
        ],
    ),
    (
        "forest",
        &[
            "forest",
            "tree",
            "wood",
            "leaf",
            "animal",
            "deer",
            "bird",
            "nature",
            "wild",
            "wildlife",
            "canopy",
            "grove",
            "woodland",
            "foliage",
            "squirrel",
        ],
    ),
    (
        "medieval",
        &[
            "castle",
            "knight",
            "sword",
            "king",
            "queen",
            "dragon",
            "armor",
            "quest",
            "kingdom",
            "royal",
            "fortress",
            "battle",
            "warrior",
            "shield",
            "throne",
        ],
    ),
    (
        "modern",
        &[
            "city",
            "car",
            "computer",
            "phone",
            "internet",
            "technology",
            "building",
            "street",
            "office",
            "urban",
            "apartment",
            "digital",
            "network",
            "software",
            "device",
        ],
    ),
];

/// Conflicting theme pairs that indicate theme drift
const CONFLICTING_THEMES: &[(&str, &str)] = &[
    ("ocean", "space"),
    ("ocean", "desert"),
    ("forest", "city"),
    ("forest", "ocean"),
    ("medieval", "modern"),
    ("animals", "technology"),
    ("nature", "urban"),
];

/// Validate theme consistency across story nodes
///
/// Returns a score between 0.0 and 1.0 where:
/// - 1.0 = perfect theme consistency
/// - 0.0 = completely inconsistent or conflicting themes
///
/// # Arguments
/// * `nodes` - Story nodes to validate
/// * `theme` - Expected theme (e.g., "ocean exploration")
///
/// # Returns
/// Consistency score from 0.0 to 1.0
pub fn validate_theme_consistency(nodes: &[ContentNode], theme: &str) -> f32 {
    // Handle edge cases
    if theme.trim().is_empty() {
        return 1.0; // No theme constraint
    }

    if nodes.is_empty() {
        return 1.0; // Nothing to validate
    }

    if nodes.len() == 1 {
        return 1.0; // Can't measure consistency with single node
    }

    // Extract theme keywords from the theme parameter
    let theme_keywords = extract_theme_keywords(theme);
    if theme_keywords.is_empty() {
        return 1.0; // No recognizable theme keywords
    }

    let mut matching_nodes = 0;
    let mut drift_nodes = 0;
    let total_nodes = nodes.len();

    // Analyze each node for theme consistency
    for node in nodes {
        let content = &node.content.text;
        let content_lower = content.to_lowercase();

        // Count theme keyword matches
        let matches = count_theme_matches(&content_lower, &theme_keywords);

        if matches > 0 {
            matching_nodes += 1;
        }

        // Check for theme drift
        if detect_conflicting_themes(&content_lower, theme) {
            drift_nodes += 1;
        }
    }

    // Calculate consistency score
    let match_ratio = matching_nodes as f32 / total_nodes as f32;

    // Penalize theme drift
    let drift_penalty = drift_nodes as f32 / total_nodes as f32;

    // Final score: match ratio minus drift penalty
    // Ensure result stays in 0.0-1.0 range
    (match_ratio - drift_penalty).max(0.0).min(1.0)
}

/// Extract theme keywords from theme string
///
/// Matches theme words against predefined categories and returns
/// all associated keywords.
///
/// # Arguments
/// * `theme` - Theme description (e.g., "ocean exploration")
///
/// # Returns
/// Vector of theme-related keywords
fn extract_theme_keywords(theme: &str) -> Vec<String> {
    let theme_lower = theme.to_lowercase();
    let mut keywords = Vec::new();

    // Build keyword map from categories
    let keyword_map: HashMap<&str, Vec<&str>> = THEME_CATEGORIES
        .iter()
        .map(|(category, words)| (*category, words.to_vec()))
        .collect();

    // Check each category against the theme
    for (category, category_keywords) in &keyword_map {
        if theme_lower.contains(category) {
            // Add all keywords from matching category
            keywords.extend(category_keywords.iter().map(|s| s.to_string()));
        }
    }

    // Also extract individual words from theme as keywords
    for word in theme_lower.split_whitespace() {
        if word.len() > 3 {
            // Skip very short words
            keywords.push(word.to_string());
        }
    }

    keywords
}

/// Count how many theme keywords appear in content
///
/// # Arguments
/// * `content` - Content text (should be lowercase)
/// * `keywords` - Theme keywords to search for
///
/// # Returns
/// Number of keyword matches found
fn count_theme_matches(content: &str, keywords: &[String]) -> usize {
    keywords
        .iter()
        .filter(|keyword| content.contains(keyword.as_str()))
        .count()
}

/// Detect if content contains themes that conflict with the expected theme
///
/// # Arguments
/// * `content` - Content text (should be lowercase)
/// * `theme` - Expected theme
///
/// # Returns
/// True if conflicting themes are detected
fn detect_conflicting_themes(content: &str, theme: &str) -> bool {
    let theme_lower = theme.to_lowercase();

    // Identify primary theme category
    let primary_category = THEME_CATEGORIES
        .iter()
        .find(|(category, _)| theme_lower.contains(category))
        .map(|(category, _)| *category);

    if let Some(primary) = primary_category {
        // Check for conflicting themes
        for (theme1, theme2) in CONFLICTING_THEMES {
            let is_primary_conflict = (*theme1 == primary || *theme2 == primary)
                && (content.contains(theme1) || content.contains(theme2));

            if is_primary_conflict {
                // Check if the conflict theme appears significantly
                let conflict_category = if *theme1 == primary {
                    theme2
                } else {
                    theme1
                };

                // Get keywords for conflicting category
                if let Some((_, keywords)) = THEME_CATEGORIES
                    .iter()
                    .find(|(cat, _)| cat == conflict_category)
                {
                    let conflict_matches =
                        keywords.iter().filter(|kw| content.contains(*kw)).count();

                    // If multiple conflicting keywords appear, it's drift
                    if conflict_matches >= 2 {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{Choice, Content};

    fn create_test_node(id: &str, text: &str) -> ContentNode {
        ContentNode {
            id: id.to_string(),
            content: Content {
                node_id: id.to_string(),
                text: text.to_string(),
                r#type: "narrative".to_string(),
                choices: vec![],
                next_nodes: vec![],
                convergence_point: false,
                educational_content: None,
            },
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        }
    }

    #[test]
    fn test_empty_theme_returns_perfect_score() {
        let nodes = vec![create_test_node("1", "some content")];
        assert_eq!(validate_theme_consistency(&nodes, ""), 1.0);
    }

    #[test]
    fn test_empty_nodes_returns_perfect_score() {
        assert_eq!(validate_theme_consistency(&[], "ocean"), 1.0);
    }

    #[test]
    fn test_single_node_returns_perfect_score() {
        let nodes = vec![create_test_node("1", "ocean waves")];
        assert_eq!(validate_theme_consistency(&nodes, "ocean"), 1.0);
    }

    #[test]
    fn test_perfect_ocean_consistency() {
        let nodes = vec![
            create_test_node("1", "The ocean waves crashed against the shore"),
            create_test_node("2", "A submarine descended into the deep sea"),
            create_test_node("3", "Dolphins swam alongside the coral reef"),
        ];
        let score = validate_theme_consistency(&nodes, "ocean exploration");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_partial_consistency() {
        let nodes = vec![
            create_test_node("1", "The ocean waves were beautiful"),
            create_test_node("2", "Walking through the park"),
            create_test_node("3", "Marine life is fascinating"),
        ];
        let score = validate_theme_consistency(&nodes, "ocean");
        // 2 out of 3 nodes have ocean keywords = 0.666...
        assert!((score - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_theme_drift_detection() {
        let nodes = vec![
            create_test_node("1", "The ocean was calm and peaceful"),
            create_test_node(
                "2",
                "Suddenly they launched into space with rockets and stars",
            ),
            create_test_node("3", "The spacecraft orbited the planet"),
        ];
        let score = validate_theme_consistency(&nodes, "ocean");
        // Should be penalized for space drift
        assert!(score < 0.5);
    }

    #[test]
    fn test_extract_theme_keywords() {
        let keywords = extract_theme_keywords("ocean exploration");
        assert!(keywords.contains(&"ocean".to_string()));
        assert!(keywords.contains(&"water".to_string()));
        assert!(keywords.contains(&"exploration".to_string()));
    }

    #[test]
    fn test_count_theme_matches() {
        let content = "the ocean waves and marine life";
        let keywords = vec!["ocean".to_string(), "marine".to_string(), "space".to_string()];
        assert_eq!(count_theme_matches(content, &keywords), 2);
    }

    #[test]
    fn test_detect_conflicting_themes_ocean_space() {
        let content = "rockets and stars in space with planets and astronauts";
        assert!(detect_conflicting_themes(content, "ocean"));
    }

    #[test]
    fn test_no_conflict_detection() {
        let content = "the ocean waves crashed against the reef";
        assert!(!detect_conflicting_themes(content, "ocean"));
    }

    #[test]
    fn test_medieval_modern_conflict() {
        let nodes = vec![
            create_test_node("1", "The knight drew his sword"),
            create_test_node("2", "He checked his phone and computer"),
            create_test_node("3", "The castle stood tall"),
        ];
        let score = validate_theme_consistency(&nodes, "medieval");
        // Should be penalized for modern drift
        assert!(score < 0.7);
    }
}
