//! Tests for DAG Configuration Types
//!
//! Validates the two-tier DAG configuration model:
//! - Tier 1: Simple story structure presets ("guided", "adventure", "epic", "choose_your_path")
//! - Tier 2: Full custom DagStructureConfig with all parameters
//!
//! Tests cover:
//! - ConvergencePattern enum serialization
//! - DagStructureConfig validation (ranges and requirements)
//! - StoryStructurePreset parsing and mapping
//! - Configuration resolution priority (preset > custom > defaults)

// ============================================================================
// NOTE: These are expected types from the implementation plan
// When the actual types are generated, these will need to reference them
// ============================================================================

/// Convergence pattern for story DAG structure
///
/// Expected to be generated from JSON schema and support serde serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize, serde::Deserialize))]
pub enum ConvergencePattern {
    /// Single major convergence point (default) - mid-story reconvergence
    SingleConvergence,
    /// Multiple convergence points at regular intervals
    MultipleConvergence,
    /// All paths converge only near the end (90%+)
    EndOnly,
    /// Pure branching tree with multiple endings (no convergence)
    PureBranching,
    /// Separate parallel storylines that never merge
    ParallelPaths,
}

/// Complete DAG structure configuration
///
/// Expected to be generated from JSON schema with validation
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize, serde::Deserialize))]
pub struct DagStructureConfig {
    /// Total number of nodes in story DAG (4-100)
    pub node_count: usize,
    /// Pattern for how story branches converge
    pub convergence_pattern: ConvergencePattern,
    /// Position of convergence as ratio (0.0-1.0), None for PureBranching/ParallelPaths
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convergence_point_ratio: Option<f64>,
    /// Maximum depth of DAG tree (3-20)
    pub max_depth: usize,
    /// Number of choices per decision node (2-4)
    pub branching_factor: usize,
}

/// Predefined story structure presets (Tier 1: Simple Model)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryStructurePreset {
    Guided,
    Adventure,
    Epic,
    ChooseYourPath,
}

/// Error type for configuration validation
#[derive(Debug)]
pub enum ValidationError {
    InvalidRange { field: String, value: String, range: String },
    MissingRequired { field: String, reason: String },
    InvalidPreset { name: String, valid_options: Vec<String> },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidRange { field, value, range } => {
                write!(f, "{} must be {}, got {}", field, range, value)
            }
            ValidationError::MissingRequired { field, reason } => {
                write!(f, "{} is required: {}", field, reason)
            }
            ValidationError::InvalidPreset { name, valid_options } => {
                write!(
                    f,
                    "Unknown story_structure preset: '{}'. Valid options: {}",
                    name,
                    valid_options.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl StoryStructurePreset {
    /// Parse preset from string name
    pub fn from_name(name: &str) -> Result<Self, ValidationError> {
        match name.to_lowercase().as_str() {
            "guided" => Ok(Self::Guided),
            "adventure" => Ok(Self::Adventure),
            "epic" => Ok(Self::Epic),
            "choose_your_path" => Ok(Self::ChooseYourPath),
            _ => Err(ValidationError::InvalidPreset {
                name: name.to_string(),
                valid_options: vec![
                    "guided".to_string(),
                    "adventure".to_string(),
                    "epic".to_string(),
                    "choose_your_path".to_string(),
                ],
            }),
        }
    }

    /// Convert preset to full DagStructureConfig
    pub fn to_dag_config(&self) -> DagStructureConfig {
        match self {
            Self::Guided => DagStructureConfig {
                node_count: 12,
                convergence_pattern: ConvergencePattern::SingleConvergence,
                convergence_point_ratio: Some(0.5),
                max_depth: 8,
                branching_factor: 2,
            },
            Self::Adventure => DagStructureConfig {
                node_count: 16,
                convergence_pattern: ConvergencePattern::MultipleConvergence,
                convergence_point_ratio: Some(0.6),
                max_depth: 10,
                branching_factor: 2,
            },
            Self::Epic => DagStructureConfig {
                node_count: 24,
                convergence_pattern: ConvergencePattern::EndOnly,
                convergence_point_ratio: Some(0.9),
                max_depth: 12,
                branching_factor: 2,
            },
            Self::ChooseYourPath => DagStructureConfig {
                node_count: 16,
                convergence_pattern: ConvergencePattern::PureBranching,
                convergence_point_ratio: None,
                max_depth: 10,
                branching_factor: 3,
            },
        }
    }
}

impl DagStructureConfig {
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate node_count range (4-100)
        if self.node_count < 4 || self.node_count > 100 {
            return Err(ValidationError::InvalidRange {
                field: "node_count".to_string(),
                value: self.node_count.to_string(),
                range: "between 4 and 100".to_string(),
            });
        }

        // Validate max_depth range (3-20)
        if self.max_depth < 3 || self.max_depth > 20 {
            return Err(ValidationError::InvalidRange {
                field: "max_depth".to_string(),
                value: self.max_depth.to_string(),
                range: "between 3 and 20".to_string(),
            });
        }

        // Validate branching_factor range (2-4)
        if self.branching_factor < 2 || self.branching_factor > 4 {
            return Err(ValidationError::InvalidRange {
                field: "branching_factor".to_string(),
                value: self.branching_factor.to_string(),
                range: "between 2 and 4".to_string(),
            });
        }

        // Validate convergence_point_ratio requirements by pattern
        match self.convergence_pattern {
            ConvergencePattern::PureBranching | ConvergencePattern::ParallelPaths => {
                if self.convergence_point_ratio.is_some() {
                    return Err(ValidationError::MissingRequired {
                        field: "convergence_point_ratio".to_string(),
                        reason: "must be None for PureBranching/ParallelPaths patterns".to_string(),
                    });
                }
            }
            _ => {
                if let Some(ratio) = self.convergence_point_ratio {
                    if !(0.0..=1.0).contains(&ratio) {
                        return Err(ValidationError::InvalidRange {
                            field: "convergence_point_ratio".to_string(),
                            value: ratio.to_string(),
                            range: "between 0.0 and 1.0".to_string(),
                        });
                    }
                } else {
                    return Err(ValidationError::MissingRequired {
                        field: "convergence_point_ratio".to_string(),
                        reason: "is required for this convergence pattern".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for DagStructureConfig {
    fn default() -> Self {
        Self {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        }
    }
}

/// Resolve DAG configuration with priority: preset > custom > defaults
pub fn resolve_dag_config(
    story_structure: Option<&str>,
    dag_config: Option<DagStructureConfig>,
    defaults: &DagStructureConfig,
) -> Result<DagStructureConfig, ValidationError> {
    // PRIORITY 1: story_structure preset
    if let Some(preset_name) = story_structure {
        let preset = StoryStructurePreset::from_name(preset_name)?;
        return Ok(preset.to_dag_config());
    }

    // PRIORITY 2: Custom dag_config
    if let Some(config) = dag_config {
        config.validate()?;
        return Ok(config);
    }

    // PRIORITY 3: Defaults
    Ok(defaults.clone())
}

// ============================================================================
// TEST SUITE
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TEST GROUP 1: ConvergencePattern Enum Serialization
    // ========================================================================

    #[test]
    fn test_convergence_pattern_single_convergence_serialization() {
        let pattern = ConvergencePattern::SingleConvergence;
        let json_str = serde_json::to_string(&pattern).expect("Should serialize");
        assert_eq!(json_str, "\"SingleConvergence\"");

        let deserialized: ConvergencePattern =
            serde_json::from_str(&json_str).expect("Should deserialize");
        assert_eq!(pattern, deserialized);
    }

    #[test]
    fn test_convergence_pattern_multiple_convergence_serialization() {
        let pattern = ConvergencePattern::MultipleConvergence;
        let json_str = serde_json::to_string(&pattern).expect("Should serialize");
        assert_eq!(json_str, "\"MultipleConvergence\"");

        let deserialized: ConvergencePattern =
            serde_json::from_str(&json_str).expect("Should deserialize");
        assert_eq!(pattern, deserialized);
    }

    #[test]
    fn test_convergence_pattern_end_only_serialization() {
        let pattern = ConvergencePattern::EndOnly;
        let json_str = serde_json::to_string(&pattern).expect("Should serialize");
        assert_eq!(json_str, "\"EndOnly\"");

        let deserialized: ConvergencePattern =
            serde_json::from_str(&json_str).expect("Should deserialize");
        assert_eq!(pattern, deserialized);
    }

    #[test]
    fn test_convergence_pattern_pure_branching_serialization() {
        let pattern = ConvergencePattern::PureBranching;
        let json_str = serde_json::to_string(&pattern).expect("Should serialize");
        assert_eq!(json_str, "\"PureBranching\"");

        let deserialized: ConvergencePattern =
            serde_json::from_str(&json_str).expect("Should deserialize");
        assert_eq!(pattern, deserialized);
    }

    #[test]
    fn test_convergence_pattern_parallel_paths_serialization() {
        let pattern = ConvergencePattern::ParallelPaths;
        let json_str = serde_json::to_string(&pattern).expect("Should serialize");
        assert_eq!(json_str, "\"ParallelPaths\"");

        let deserialized: ConvergencePattern =
            serde_json::from_str(&json_str).expect("Should deserialize");
        assert_eq!(pattern, deserialized);
    }

    #[test]
    fn test_convergence_pattern_all_variants_roundtrip() {
        let all_patterns = vec![
            ConvergencePattern::SingleConvergence,
            ConvergencePattern::MultipleConvergence,
            ConvergencePattern::EndOnly,
            ConvergencePattern::PureBranching,
            ConvergencePattern::ParallelPaths,
        ];

        for pattern in all_patterns {
            let json_str = serde_json::to_string(&pattern).expect("Should serialize");
            let deserialized: ConvergencePattern =
                serde_json::from_str(&json_str).expect("Should deserialize");
            assert_eq!(pattern, deserialized);
        }
    }

    // ========================================================================
    // TEST GROUP 2: DagStructureConfig Validation - Node Count
    // ========================================================================

    #[test]
    fn test_dag_config_node_count_valid_minimum() {
        let config = DagStructureConfig {
            node_count: 4,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 3,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_node_count_valid_maximum() {
        let config = DagStructureConfig {
            node_count: 100,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 20,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_node_count_below_minimum() {
        let config = DagStructureConfig {
            node_count: 3,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("node_count"));
        assert!(err.contains("between 4 and 100"));
    }

    #[test]
    fn test_dag_config_node_count_above_maximum() {
        let config = DagStructureConfig {
            node_count: 101,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("node_count"));
        assert!(err.contains("between 4 and 100"));
    }

    #[test]
    fn test_dag_config_node_count_zero() {
        let config = DagStructureConfig {
            node_count: 0,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 2,
        };
        assert!(config.validate().is_err());
    }

    // ========================================================================
    // TEST GROUP 3: DagStructureConfig Validation - Max Depth
    // ========================================================================

    #[test]
    fn test_dag_config_max_depth_valid_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 3,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_max_depth_valid_maximum() {
        let config = DagStructureConfig {
            node_count: 50,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 20,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_max_depth_below_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 2,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("max_depth"));
        assert!(err.contains("between 3 and 20"));
    }

    #[test]
    fn test_dag_config_max_depth_above_maximum() {
        let config = DagStructureConfig {
            node_count: 50,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 21,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("max_depth"));
        assert!(err.contains("between 3 and 20"));
    }

    // ========================================================================
    // TEST GROUP 4: DagStructureConfig Validation - Branching Factor
    // ========================================================================

    #[test]
    fn test_dag_config_branching_factor_valid_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_branching_factor_valid_maximum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 4,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_branching_factor_below_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 1,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("branching_factor"));
        assert!(err.contains("between 2 and 4"));
    }

    #[test]
    fn test_dag_config_branching_factor_above_maximum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 5,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("branching_factor"));
        assert!(err.contains("between 2 and 4"));
    }

    // ========================================================================
    // TEST GROUP 5: DagStructureConfig Validation - Convergence Ratio
    // ========================================================================

    #[test]
    fn test_dag_config_convergence_ratio_valid_for_single_convergence() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_convergence_ratio_required_for_single_convergence() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: None,
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("required"));
    }

    #[test]
    fn test_dag_config_convergence_ratio_required_for_multiple_convergence() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::MultipleConvergence,
            convergence_point_ratio: None,
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("required"));
    }

    #[test]
    fn test_dag_config_convergence_ratio_required_for_end_only() {
        let config = DagStructureConfig {
            node_count: 24,
            convergence_pattern: ConvergencePattern::EndOnly,
            convergence_point_ratio: None,
            max_depth: 12,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("required"));
    }

    #[test]
    fn test_dag_config_convergence_ratio_must_be_none_for_pure_branching() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::PureBranching,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 3,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("must be None"));
    }

    #[test]
    fn test_dag_config_convergence_ratio_must_be_none_for_parallel_paths() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::ParallelPaths,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("must be None"));
    }

    #[test]
    fn test_dag_config_convergence_ratio_range_minimum() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.0),
            max_depth: 10,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_convergence_ratio_range_maximum() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(1.0),
            max_depth: 10,
            branching_factor: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_dag_config_convergence_ratio_below_minimum() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(-0.1),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("between 0.0 and 1.0"));
    }

    #[test]
    fn test_dag_config_convergence_ratio_above_maximum() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(1.1),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("convergence_point_ratio"));
        assert!(err.contains("between 0.0 and 1.0"));
    }

    // ========================================================================
    // TEST GROUP 6: StoryStructurePreset Parsing
    // ========================================================================

    #[test]
    fn test_preset_parse_guided_lowercase() {
        let preset = StoryStructurePreset::from_name("guided").expect("Should parse");
        assert_eq!(preset, StoryStructurePreset::Guided);
    }

    #[test]
    fn test_preset_parse_guided_uppercase() {
        let preset = StoryStructurePreset::from_name("GUIDED").expect("Should parse");
        assert_eq!(preset, StoryStructurePreset::Guided);
    }

    #[test]
    fn test_preset_parse_guided_mixedcase() {
        let preset = StoryStructurePreset::from_name("Guided").expect("Should parse");
        assert_eq!(preset, StoryStructurePreset::Guided);
    }

    #[test]
    fn test_preset_parse_adventure() {
        let preset = StoryStructurePreset::from_name("adventure").expect("Should parse");
        assert_eq!(preset, StoryStructurePreset::Adventure);
    }

    #[test]
    fn test_preset_parse_epic() {
        let preset = StoryStructurePreset::from_name("epic").expect("Should parse");
        assert_eq!(preset, StoryStructurePreset::Epic);
    }

    #[test]
    fn test_preset_parse_choose_your_path() {
        let preset = StoryStructurePreset::from_name("choose_your_path").expect("Should parse");
        assert_eq!(preset, StoryStructurePreset::ChooseYourPath);
    }

    #[test]
    fn test_preset_parse_invalid_empty() {
        let result = StoryStructurePreset::from_name("");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown story_structure preset"));
    }

    #[test]
    fn test_preset_parse_invalid_unknown() {
        let result = StoryStructurePreset::from_name("unknown_preset");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown story_structure preset"));
        assert!(err.contains("guided"));
        assert!(err.contains("adventure"));
        assert!(err.contains("epic"));
        assert!(err.contains("choose_your_path"));
    }

    #[test]
    fn test_preset_parse_invalid_typo() {
        let result = StoryStructurePreset::from_name("gided"); // typo
        assert!(result.is_err());
    }

    // ========================================================================
    // TEST GROUP 7: Preset to Config Mapping
    // ========================================================================

    #[test]
    fn test_preset_guided_mapping() {
        let preset = StoryStructurePreset::Guided;
        let config = preset.to_dag_config();

        assert_eq!(config.node_count, 12);
        assert_eq!(config.convergence_pattern, ConvergencePattern::SingleConvergence);
        assert_eq!(config.convergence_point_ratio, Some(0.5));
        assert_eq!(config.max_depth, 8);
        assert_eq!(config.branching_factor, 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_adventure_mapping() {
        let preset = StoryStructurePreset::Adventure;
        let config = preset.to_dag_config();

        assert_eq!(config.node_count, 16);
        assert_eq!(config.convergence_pattern, ConvergencePattern::MultipleConvergence);
        assert_eq!(config.convergence_point_ratio, Some(0.6));
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.branching_factor, 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_epic_mapping() {
        let preset = StoryStructurePreset::Epic;
        let config = preset.to_dag_config();

        assert_eq!(config.node_count, 24);
        assert_eq!(config.convergence_pattern, ConvergencePattern::EndOnly);
        assert_eq!(config.convergence_point_ratio, Some(0.9));
        assert_eq!(config.max_depth, 12);
        assert_eq!(config.branching_factor, 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_choose_your_path_mapping() {
        let preset = StoryStructurePreset::ChooseYourPath;
        let config = preset.to_dag_config();

        assert_eq!(config.node_count, 16);
        assert_eq!(config.convergence_pattern, ConvergencePattern::PureBranching);
        assert_eq!(config.convergence_point_ratio, None);
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.branching_factor, 3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_all_configs_are_valid() {
        // Ensure all preset configurations pass validation
        let presets = vec![
            StoryStructurePreset::Guided,
            StoryStructurePreset::Adventure,
            StoryStructurePreset::Epic,
            StoryStructurePreset::ChooseYourPath,
        ];

        for preset in presets {
            let config = preset.to_dag_config();
            assert!(
                config.validate().is_ok(),
                "Preset {:?} should produce valid config",
                preset
            );
        }
    }

    // ========================================================================
    // TEST GROUP 8: Resolution Priority Order
    // ========================================================================

    #[test]
    fn test_resolve_priority_preset_wins() {
        let defaults = DagStructureConfig::default();
        let custom = DagStructureConfig {
            node_count: 50,
            convergence_pattern: ConvergencePattern::EndOnly,
            convergence_point_ratio: Some(0.8),
            max_depth: 15,
            branching_factor: 3,
        };

        // Both preset and custom provided - preset should win
        let resolved = resolve_dag_config(
            Some("guided"),
            Some(custom),
            &defaults,
        ).expect("Should resolve");

        // Should match guided preset, not custom
        assert_eq!(resolved.node_count, 12);
        assert_eq!(resolved.convergence_pattern, ConvergencePattern::SingleConvergence);
        assert_eq!(resolved.convergence_point_ratio, Some(0.5));
    }

    #[test]
    fn test_resolve_priority_custom_wins_over_defaults() {
        let defaults = DagStructureConfig::default();
        let custom = DagStructureConfig {
            node_count: 50,
            convergence_pattern: ConvergencePattern::EndOnly,
            convergence_point_ratio: Some(0.8),
            max_depth: 15,
            branching_factor: 3,
        };

        // No preset, custom provided - custom should win
        let resolved = resolve_dag_config(
            None,
            Some(custom.clone()),
            &defaults,
        ).expect("Should resolve");

        assert_eq!(resolved.node_count, 50);
        assert_eq!(resolved.convergence_pattern, ConvergencePattern::EndOnly);
        assert_eq!(resolved.convergence_point_ratio, Some(0.8));
        assert_eq!(resolved.max_depth, 15);
        assert_eq!(resolved.branching_factor, 3);
    }

    #[test]
    fn test_resolve_priority_defaults_used() {
        let defaults = DagStructureConfig {
            node_count: 20,
            convergence_pattern: ConvergencePattern::MultipleConvergence,
            convergence_point_ratio: Some(0.6),
            max_depth: 12,
            branching_factor: 2,
        };

        // No preset, no custom - defaults should be used
        let resolved = resolve_dag_config(
            None,
            None,
            &defaults,
        ).expect("Should resolve");

        assert_eq!(resolved.node_count, 20);
        assert_eq!(resolved.convergence_pattern, ConvergencePattern::MultipleConvergence);
        assert_eq!(resolved.convergence_point_ratio, Some(0.6));
        assert_eq!(resolved.max_depth, 12);
        assert_eq!(resolved.branching_factor, 2);
    }

    #[test]
    fn test_resolve_invalid_preset_returns_error() {
        let defaults = DagStructureConfig::default();

        let result = resolve_dag_config(
            Some("invalid_preset"),
            None,
            &defaults,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown story_structure preset"));
    }

    #[test]
    fn test_resolve_invalid_custom_config_returns_error() {
        let defaults = DagStructureConfig::default();
        let invalid_custom = DagStructureConfig {
            node_count: 200, // Invalid: over maximum
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };

        let result = resolve_dag_config(
            None,
            Some(invalid_custom),
            &defaults,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("node_count"));
    }

    // ========================================================================
    // TEST GROUP 9: DagStructureConfig Serialization
    // ========================================================================

    #[test]
    fn test_dag_config_json_serialization_with_ratio() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };

        let json_str = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: DagStructureConfig =
            serde_json::from_str(&json_str).expect("Should deserialize");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_dag_config_json_serialization_without_ratio() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::PureBranching,
            convergence_point_ratio: None,
            max_depth: 10,
            branching_factor: 3,
        };

        let json_str = serde_json::to_string(&config).expect("Should serialize");

        // convergence_point_ratio should be omitted when None
        assert!(!json_str.contains("convergence_point_ratio"));

        let deserialized: DagStructureConfig =
            serde_json::from_str(&json_str).expect("Should deserialize");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_dag_config_default_values() {
        let config = DagStructureConfig::default();

        assert_eq!(config.node_count, 16);
        assert_eq!(config.convergence_pattern, ConvergencePattern::SingleConvergence);
        assert_eq!(config.convergence_point_ratio, Some(0.5));
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.branching_factor, 2);
        assert!(config.validate().is_ok());
    }

    // ========================================================================
    // TEST GROUP 10: Integration Tests
    // ========================================================================

    #[test]
    fn test_full_workflow_with_preset() {
        // Simulate full workflow: parse preset -> get config -> validate
        let preset_name = "epic";
        let preset = StoryStructurePreset::from_name(preset_name).expect("Should parse");
        let config = preset.to_dag_config();

        assert!(config.validate().is_ok());
        assert_eq!(config.node_count, 24);
        assert_eq!(config.convergence_pattern, ConvergencePattern::EndOnly);
    }

    #[test]
    fn test_full_workflow_with_custom_config() {
        // Create custom config -> validate -> serialize -> deserialize -> validate
        let config = DagStructureConfig {
            node_count: 32,
            convergence_pattern: ConvergencePattern::MultipleConvergence,
            convergence_point_ratio: Some(0.7),
            max_depth: 14,
            branching_factor: 3,
        };

        assert!(config.validate().is_ok());

        let json_str = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: DagStructureConfig =
            serde_json::from_str(&json_str).expect("Should deserialize");

        assert!(deserialized.validate().is_ok());
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_all_presets_serialize_and_validate() {
        let preset_names = vec!["guided", "adventure", "epic", "choose_your_path"];

        for name in preset_names {
            let preset = StoryStructurePreset::from_name(name).expect("Should parse");
            let config = preset.to_dag_config();

            assert!(config.validate().is_ok());

            let json_str = serde_json::to_string(&config).expect("Should serialize");
            let deserialized: DagStructureConfig =
                serde_json::from_str(&json_str).expect("Should deserialize");

            assert_eq!(config, deserialized);
            assert!(deserialized.validate().is_ok());
        }
    }
}
