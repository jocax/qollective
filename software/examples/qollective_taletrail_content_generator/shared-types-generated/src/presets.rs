//! Story structure presets for DAG configuration
//!
//! This module provides Tier 1 (Simple) story structure presets that map to
//! complete DAG configurations. Users can select from predefined presets
//! instead of manually configuring all DAG parameters.
//!
//! # Examples
//!
//! ```
//! use shared_types_generated::presets::StoryStructurePreset;
//!
//! let preset = StoryStructurePreset::from_name("guided").unwrap();
//! let config = preset.to_dag_config();
//! assert_eq!(config.node_count, 12);
//! ```

use crate::constants;
use crate::generated::{ConvergencePattern, DagStructureConfig};

/// Result type for preset operations
pub type Result<T> = std::result::Result<T, PresetError>;

/// Error type for preset parsing
#[derive(Debug, Clone)]
pub struct PresetError {
    pub message: String,
}

impl std::fmt::Display for PresetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PresetError {}

/// Predefined story structure presets (Tier 1: Simple Model)
///
/// Each preset provides a complete DAG configuration optimized for
/// different storytelling styles:
///
/// - **Guided**: Linear story with occasional choices (12 nodes)
/// - **Adventure**: Branching paths with multiple convergence points (16 nodes)
/// - **Epic**: Complex branching that converges near the end (24 nodes)
/// - **ChooseYourPath**: Pure branching tree with multiple endings (16 nodes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryStructurePreset {
    /// Linear story with occasional choices and single convergence
    /// - 12 nodes, depth 8, branching factor 2
    /// - SingleConvergence at 50% through story
    Guided,

    /// Adventure story with multiple convergence points
    /// - 16 nodes, depth 10, branching factor 2
    /// - MultipleConvergence at 60% intervals
    Adventure,

    /// Epic story with complex branching that converges at the end
    /// - 24 nodes, depth 12, branching factor 2
    /// - EndOnly convergence at 90%
    Epic,

    /// Pure branching tree with multiple endings
    /// - 16 nodes, depth 10, branching factor 3
    /// - No convergence (multiple endings)
    ChooseYourPath,
}

impl StoryStructurePreset {
    /// Parse preset from string name (case-insensitive)
    ///
    /// # Arguments
    ///
    /// * `name` - Preset name ("guided", "adventure", "epic", "choose_your_path")
    ///
    /// # Returns
    ///
    /// * `Ok(StoryStructurePreset)` if name is valid
    /// * `Err(TaleTrailError::ValidationError)` with helpful message listing valid options
    ///
    /// # Examples
    ///
    /// ```
    /// use shared_types_generated::presets::StoryStructurePreset;
    ///
    /// let preset = StoryStructurePreset::from_name("guided").unwrap();
    /// assert_eq!(preset, StoryStructurePreset::Guided);
    ///
    /// let preset = StoryStructurePreset::from_name("ADVENTURE").unwrap();
    /// assert_eq!(preset, StoryStructurePreset::Adventure);
    /// ```
    pub fn from_name(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "guided" => Ok(Self::Guided),
            "adventure" => Ok(Self::Adventure),
            "epic" => Ok(Self::Epic),
            "choose_your_path" => Ok(Self::ChooseYourPath),
            _ => Err(PresetError {
                message: format!(
                    "Unknown story_structure preset: '{}'. Valid options: guided, adventure, epic, choose_your_path",
                    name
                ),
            }),
        }
    }

    /// Convert preset to full DagStructureConfig
    ///
    /// Each preset maps to a validated DAG configuration optimized for
    /// the storytelling style.
    ///
    /// # Returns
    ///
    /// Complete `DagStructureConfig` that will pass validation
    ///
    /// # Examples
    ///
    /// ```
    /// use shared_types_generated::presets::StoryStructurePreset;
    ///
    /// let preset = StoryStructurePreset::Guided;
    /// let config = preset.to_dag_config();
    ///
    /// assert_eq!(config.node_count, 12);
    /// assert_eq!(config.max_depth, 8);
    /// assert_eq!(config.branching_factor, 2);
    /// ```
    pub fn to_dag_config(&self) -> DagStructureConfig {
        match self {
            Self::Guided => DagStructureConfig {
                node_count: constants::PRESET_GUIDED_NODE_COUNT,
                convergence_pattern: ConvergencePattern::SingleConvergence,
                convergence_point_ratio: Some(0.5),
                max_depth: 8,
                branching_factor: 2,
            },
            Self::Adventure => DagStructureConfig {
                node_count: constants::PRESET_ADVENTURE_NODE_COUNT,
                convergence_pattern: ConvergencePattern::MultipleConvergence,
                convergence_point_ratio: Some(0.6),
                max_depth: 10,
                branching_factor: 2,
            },
            Self::Epic => DagStructureConfig {
                node_count: constants::PRESET_EPIC_NODE_COUNT,
                convergence_pattern: ConvergencePattern::EndOnly,
                convergence_point_ratio: Some(0.9),
                max_depth: 12,
                branching_factor: 2,
            },
            Self::ChooseYourPath => DagStructureConfig {
                node_count: constants::PRESET_CHOOSE_YOUR_PATH_NODE_COUNT,
                convergence_pattern: ConvergencePattern::PureBranching,
                convergence_point_ratio: None,
                max_depth: 10,
                branching_factor: 3,
            },
        }
    }

    /// Get all available preset names as a vector
    ///
    /// Useful for displaying available options to users
    pub fn all_names() -> Vec<&'static str> {
        vec!["guided", "adventure", "epic", "choose_your_path"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name_guided_lowercase() {
        let preset = StoryStructurePreset::from_name("guided").unwrap();
        assert_eq!(preset, StoryStructurePreset::Guided);
    }

    #[test]
    fn test_from_name_guided_uppercase() {
        let preset = StoryStructurePreset::from_name("GUIDED").unwrap();
        assert_eq!(preset, StoryStructurePreset::Guided);
    }

    #[test]
    fn test_from_name_adventure() {
        let preset = StoryStructurePreset::from_name("adventure").unwrap();
        assert_eq!(preset, StoryStructurePreset::Adventure);
    }

    #[test]
    fn test_from_name_epic() {
        let preset = StoryStructurePreset::from_name("epic").unwrap();
        assert_eq!(preset, StoryStructurePreset::Epic);
    }

    #[test]
    fn test_from_name_choose_your_path() {
        let preset = StoryStructurePreset::from_name("choose_your_path").unwrap();
        assert_eq!(preset, StoryStructurePreset::ChooseYourPath);
    }

    #[test]
    fn test_from_name_invalid() {
        let result = StoryStructurePreset::from_name("invalid");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown story_structure preset"));
        assert!(err_msg.contains("guided"));
    }

    #[test]
    fn test_guided_to_dag_config() {
        let config = StoryStructurePreset::Guided.to_dag_config();
        assert_eq!(config.node_count, 12);
        assert_eq!(
            config.convergence_pattern,
            ConvergencePattern::SingleConvergence
        );
        assert_eq!(config.convergence_point_ratio, Some(0.5));
        assert_eq!(config.max_depth, 8);
        assert_eq!(config.branching_factor, 2);
    }

    #[test]
    fn test_adventure_to_dag_config() {
        let config = StoryStructurePreset::Adventure.to_dag_config();
        assert_eq!(config.node_count, 16);
        assert_eq!(
            config.convergence_pattern,
            ConvergencePattern::MultipleConvergence
        );
        assert_eq!(config.convergence_point_ratio, Some(0.6));
    }

    #[test]
    fn test_all_names() {
        let names = StoryStructurePreset::all_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"guided"));
        assert!(names.contains(&"adventure"));
        assert!(names.contains(&"epic"));
        assert!(names.contains(&"choose_your_path"));
    }
}
