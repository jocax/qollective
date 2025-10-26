//! Extension methods for GenerationRequest and DagStructureConfig
//!
//! This module provides validation and resolution logic for DAG configuration
//! in generation requests, implementing the two-tier configuration model:
//!
//! - Tier 1 (Simple): Story structure presets ("guided", "adventure", etc.)
//! - Tier 2 (Advanced): Full custom DagStructureConfig
//!
//! # Priority Resolution
//!
//! When resolving DAG configuration, the priority order is:
//! 1. `story_structure` preset (if provided)
//! 2. `dag_config` custom configuration (if provided)
//! 3. Orchestrator defaults (fallback)
//!
//! # Examples
//!
//! ```no_run
//! use shared_types_generated::{DagStructureConfig, ConvergencePattern};
//! use shared_types_generated::extensions::DagStructureConfigExt;
//!
//! // Validate a DAG configuration
//! let config = DagStructureConfig {
//!     node_count: 12,
//!     convergence_pattern: ConvergencePattern::SingleConvergence,
//!     convergence_point_ratio: Some(0.5),
//!     max_depth: 8,
//!     branching_factor: 2,
//! };
//! assert!(config.validate_config().is_ok());
//! ```

use crate::generated::{ConvergencePattern, DagStructureConfig, GenerationRequest};
use crate::presets::{PresetError, StoryStructurePreset};

/// Validation error for DAG configuration
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

impl From<PresetError> for ValidationError {
    fn from(err: PresetError) -> Self {
        ValidationError {
            message: err.message,
        }
    }
}

/// Extension trait for DagStructureConfig validation
pub trait DagStructureConfigExt {
    /// Validate configuration parameters against schema constraints
    fn validate_config(&self) -> Result<(), ValidationError>;
}

impl DagStructureConfigExt for DagStructureConfig {
    /// Validate configuration parameters against schema constraints
    ///
    /// # Validation Rules
    ///
    /// - `node_count`: Must be between 4 and 100 (inclusive)
    /// - `max_depth`: Must be between 3 and 20 (inclusive)
    /// - `branching_factor`: Must be between 2 and 4 (inclusive)
    /// - `convergence_point_ratio`:
    ///   - Must be `None` for `PureBranching` and `ParallelPaths` patterns
    ///   - Must be `Some(0.0..=1.0)` for all other patterns
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all validation rules pass
    /// - `Err(ValidationError { message: )` with detailed message if validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use shared_types_generated::{DagStructureConfig, ConvergencePattern};
    /// use shared_types_generated::extensions::DagStructureConfigExt;
    ///
    /// // Valid configuration
    /// let config = DagStructureConfig {
    ///     node_count: 16,
    ///     convergence_pattern: ConvergencePattern::SingleConvergence,
    ///     convergence_point_ratio: Some(0.5),
    ///     max_depth: 10,
    ///     branching_factor: 2,
    /// };
    /// assert!(config.validate_config().is_ok());
    ///
    /// // Invalid: node_count out of range
    /// let config = DagStructureConfig {
    ///     node_count: 200,
    ///     convergence_pattern: ConvergencePattern::SingleConvergence,
    ///     convergence_point_ratio: Some(0.5),
    ///     max_depth: 10,
    ///     branching_factor: 2,
    /// };
    /// assert!(config.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), ValidationError> {
        // Validate node_count range (4-100)
        if self.node_count < 4 || self.node_count > 100 {
            return Err(ValidationError {
                message: format!(
                    "node_count must be between 4 and 100, got {}",
                    self.node_count
                ),
            });
        }

        // Validate max_depth range (3-20)
        if self.max_depth < 3 || self.max_depth > 20 {
            return Err(ValidationError {
                message: format!("max_depth must be between 3 and 20, got {}", self.max_depth),
            });
        }

        // Validate branching_factor range (2-4)
        if self.branching_factor < 2 || self.branching_factor > 4 {
            return Err(ValidationError {
                message: format!(
                    "branching_factor must be between 2 and 4, got {}",
                    self.branching_factor
                ),
            });
        }

        // Validate convergence_point_ratio requirements by pattern
        match self.convergence_pattern {
            ConvergencePattern::PureBranching | ConvergencePattern::ParallelPaths => {
                if self.convergence_point_ratio.is_some() {
                    return Err(ValidationError {
                        message: "convergence_point_ratio must be None for PureBranching/ParallelPaths patterns".into()
                    });
                }
            }
            _ => {
                if let Some(ratio) = self.convergence_point_ratio {
                    if ratio < 0.0 || ratio > 1.0 {
                        return Err(ValidationError {
                            message: format!(
                                "convergence_point_ratio must be between 0.0 and 1.0, got {}",
                                ratio
                            ),
                        });
                    }
                } else {
                    return Err(ValidationError {
                        message: "convergence_point_ratio is required for this convergence pattern"
                            .into(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for DagStructureConfig {
    /// Provides sensible defaults for DAG configuration
    ///
    /// Default configuration represents a balanced story structure:
    /// - 16 nodes with moderate complexity
    /// - Single convergence point at story midpoint
    /// - Depth of 10 levels
    /// - Binary choices (2-way branching)
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

impl GenerationRequest {
    /// Resolve DAG configuration from request or orchestrator defaults
    ///
    /// Implements the priority resolution model with explicit node_count override:
    ///
    /// # Priority 1: Explicit node_count (HIGHEST - Always Wins)
    ///
    /// If `node_count` field is explicitly provided in the request, it ALWAYS
    /// overrides the node count from any preset, custom config, or defaults.
    /// This allows users to request specific node counts while still benefiting
    /// from preset structural settings (convergence_pattern, branching_factor, etc.).
    ///
    /// # Priority 2: Story Structure Preset
    ///
    /// If `story_structure` field is present, parse the preset name and use
    /// its DAG configuration. When both `node_count` and `story_structure` are
    /// provided, the preset's structural settings are used but with the explicit
    /// node count.
    ///
    /// # Priority 3: Custom DAG Configuration
    ///
    /// If `dag_config` field is present (and no preset), validate the
    /// configuration and use it. Explicit `node_count` still overrides.
    ///
    /// # Priority 4: Orchestrator Defaults
    ///
    /// If neither preset nor custom config is provided, use the orchestrator's
    /// default configuration. Explicit `node_count` still overrides.
    ///
    /// # Arguments
    ///
    /// * `orchestrator_defaults` - Default DAG configuration from orchestrator
    ///
    /// # Returns
    ///
    /// - `Ok(DagStructureConfig)` with resolved and validated configuration
    /// - `Err(ValidationError { message: )` if:
    ///   - Preset name is invalid
    ///   - Custom config fails validation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shared_types_generated::{GenerationRequest, DagStructureConfig};
    /// # use shared_types_generated::AgeGroup;
    /// # use shared_types_generated::Language;
    ///
    /// // Explicit node_count overrides preset - this is the key fix for the bug
    /// # let request = GenerationRequest {
    /// #     story_structure: Some("epic".to_string()),
    /// #     node_count: Some(9),
    /// #     dag_config: None,
    /// #     tenant_id: 1,
    /// #     age_group: AgeGroup::_9To11,
    /// #     language: Language::En,
    /// #     theme: "test".to_string(),
    /// #     tags: None,
    /// #     prompt_packages: None,
    /// #     educational_goals: None,
    /// #     author_id: None,
    /// #     required_elements: None,
    /// #     vocabulary_level: None,
    /// # };
    /// # let defaults = DagStructureConfig::default();
    /// let resolved = request.resolve_dag_config(&defaults).unwrap();
    /// assert_eq!(resolved.node_count, 9); // Explicit count wins over epic's 24
    /// ```
    pub fn resolve_dag_config(
        &self,
        orchestrator_defaults: &DagStructureConfig,
    ) -> Result<DagStructureConfig, ValidationError> {
        // Start with base config from preset, custom, or defaults
        let mut resolved_config = if let Some(preset_name) = &self.story_structure {
            // Priority 2: story_structure preset (for structural settings)
            if self.dag_config.is_some() {
                tracing::warn!(
                    "Both story_structure and dag_config provided. Using story_structure preset '{}' (takes priority for structure)",
                    preset_name
                );
            }
            let preset = StoryStructurePreset::from_name(preset_name)?;
            preset.to_dag_config()
        } else if let Some(config) = &self.dag_config {
            // Priority 3: Custom dag_config
            config.validate_config()?;
            config.clone()
        } else {
            // Priority 4: Orchestrator defaults
            orchestrator_defaults.clone()
        };

        // PRIORITY 1 OVERRIDE: Explicit node_count always wins (even over preset)
        if let Some(explicit_count) = self.node_count {
            if explicit_count != resolved_config.node_count {
                tracing::info!(
                    requested_count = explicit_count,
                    preset_count = resolved_config.node_count,
                    preset = ?self.story_structure,
                    "Explicit node_count overrides story_structure preset"
                );
            }
            resolved_config.node_count = explicit_count;
        }

        Ok(resolved_config)
    }
}

#[cfg(test)]
mod tests {
    use super::DagStructureConfigExt;
    use super::*;
    use crate::generated::{AgeGroup, Language}; // Import trait for extension methods

    // ========================================================================
    // TEST GROUP 1: DagStructureConfig Validation - Node Count
    // ========================================================================

    #[test]
    fn test_validate_node_count_valid_minimum() {
        let config = DagStructureConfig {
            node_count: 4,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 3,
            branching_factor: 2,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_node_count_valid_maximum() {
        let config = DagStructureConfig {
            node_count: 100,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 20,
            branching_factor: 2,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_node_count_below_minimum() {
        let config = DagStructureConfig {
            node_count: 3,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("node_count"));
    }

    #[test]
    fn test_validate_node_count_above_maximum() {
        let config = DagStructureConfig {
            node_count: 101,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("node_count"));
    }

    // ========================================================================
    // TEST GROUP 2: DagStructureConfig Validation - Max Depth
    // ========================================================================

    #[test]
    fn test_validate_max_depth_valid_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 3,
            branching_factor: 2,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_max_depth_valid_maximum() {
        let config = DagStructureConfig {
            node_count: 50,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 20,
            branching_factor: 2,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_max_depth_below_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 2,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_depth"));
    }

    #[test]
    fn test_validate_max_depth_above_maximum() {
        let config = DagStructureConfig {
            node_count: 50,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 21,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_depth"));
    }

    // ========================================================================
    // TEST GROUP 3: DagStructureConfig Validation - Branching Factor
    // ========================================================================

    #[test]
    fn test_validate_branching_factor_valid_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 2,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_branching_factor_valid_maximum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 4,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_branching_factor_below_minimum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 1,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("branching_factor"));
    }

    #[test]
    fn test_validate_branching_factor_above_maximum() {
        let config = DagStructureConfig {
            node_count: 10,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 5,
            branching_factor: 5,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("branching_factor"));
    }

    // ========================================================================
    // TEST GROUP 4: DagStructureConfig Validation - Convergence Ratio
    // ========================================================================

    #[test]
    fn test_validate_convergence_ratio_required_for_single_convergence() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: None,
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required"));
    }

    #[test]
    fn test_validate_convergence_ratio_must_be_none_for_pure_branching() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::PureBranching,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 3,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be None"));
    }

    #[test]
    fn test_validate_convergence_ratio_range_valid() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_validate_convergence_ratio_below_minimum() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(-0.1),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("between 0.0 and 1.0"));
    }

    #[test]
    fn test_validate_convergence_ratio_above_maximum() {
        let config = DagStructureConfig {
            node_count: 16,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(1.1),
            max_depth: 10,
            branching_factor: 2,
        };
        let result = config.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("between 0.0 and 1.0"));
    }

    // ========================================================================
    // TEST GROUP 5: GenerationRequest Resolution - Priority Order
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

        let request = GenerationRequest {
            story_structure: Some("guided".to_string()),
            dag_config: Some(custom),
            tenant_id: 1,
            age_group: AgeGroup::_9To11,
            language: Language::En,
            theme: "test".to_string(),
            tags: None,
            node_count: None,
            prompt_packages: None,
            educational_goals: None,
            author_id: None,
            required_elements: None,
            vocabulary_level: None,
        };

        let resolved = request.resolve_dag_config(&defaults).unwrap();

        // Should match guided preset, not custom
        assert_eq!(resolved.node_count, 12);
        assert_eq!(
            resolved.convergence_pattern,
            ConvergencePattern::SingleConvergence
        );
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

        let request = GenerationRequest {
            story_structure: None,
            dag_config: Some(custom.clone()),
            tenant_id: 1,
            age_group: AgeGroup::_9To11,
            language: Language::En,
            theme: "test".to_string(),
            tags: None,
            node_count: None,
            prompt_packages: None,
            educational_goals: None,
            author_id: None,
            required_elements: None,
            vocabulary_level: None,
        };

        let resolved = request.resolve_dag_config(&defaults).unwrap();

        assert_eq!(resolved.node_count, 50);
        assert_eq!(resolved.convergence_pattern, ConvergencePattern::EndOnly);
        assert_eq!(resolved.convergence_point_ratio, Some(0.8));
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

        let request = GenerationRequest {
            story_structure: None,
            dag_config: None,
            tenant_id: 1,
            age_group: AgeGroup::_9To11,
            language: Language::En,
            theme: "test".to_string(),
            tags: None,
            node_count: None,
            prompt_packages: None,
            educational_goals: None,
            author_id: None,
            required_elements: None,
            vocabulary_level: None,
        };

        let resolved = request.resolve_dag_config(&defaults).unwrap();

        assert_eq!(resolved.node_count, 20);
        assert_eq!(
            resolved.convergence_pattern,
            ConvergencePattern::MultipleConvergence
        );
        assert_eq!(resolved.convergence_point_ratio, Some(0.6));
    }

    #[test]
    fn test_resolve_invalid_preset_returns_error() {
        let defaults = DagStructureConfig::default();

        let request = GenerationRequest {
            story_structure: Some("invalid_preset".to_string()),
            dag_config: None,
            tenant_id: 1,
            age_group: AgeGroup::_9To11,
            language: Language::En,
            theme: "test".to_string(),
            tags: None,
            node_count: None,
            prompt_packages: None,
            educational_goals: None,
            author_id: None,
            required_elements: None,
            vocabulary_level: None,
        };

        let result = request.resolve_dag_config(&defaults);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown story_structure preset"));
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

        let request = GenerationRequest {
            story_structure: None,
            dag_config: Some(invalid_custom),
            tenant_id: 1,
            age_group: AgeGroup::_9To11,
            language: Language::En,
            theme: "test".to_string(),
            tags: None,
            node_count: None,
            prompt_packages: None,
            educational_goals: None,
            author_id: None,
            required_elements: None,
            vocabulary_level: None,
        };

        let result = request.resolve_dag_config(&defaults);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("node_count"));
    }

    // ========================================================================
    // TEST GROUP 6: DagStructureConfig Default
    // ========================================================================

    #[test]
    fn test_default_config_is_valid() {
        let config = DagStructureConfig::default();
        assert!(config.validate_config().is_ok());
        assert_eq!(config.node_count, 16);
        assert_eq!(
            config.convergence_pattern,
            ConvergencePattern::SingleConvergence
        );
        assert_eq!(config.convergence_point_ratio, Some(0.5));
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.branching_factor, 2);
    }
}
