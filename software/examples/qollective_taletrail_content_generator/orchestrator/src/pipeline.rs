//! Pipeline state machine for TaleTrail content generation
//!
//! Manages the generation pipeline phases with progress tracking and error accumulation.

use shared_types::*;
use std::collections::HashMap;

/// Orchestrator pipeline phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePhase {
    /// Generate all prompts for downstream services
    PromptGeneration,
    /// Generate DAG structure
    Structure,
    /// Generate node content in batches
    Generation,
    /// Validate content quality and constraints
    Validation,
    /// Assemble final validated DAG
    Assembly,
    /// Pipeline complete
    Complete,
}

/// Pipeline state with phase tracking, progress, and error accumulation
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// Current pipeline phase
    pub current_phase: PipelinePhase,
    /// Progress percentage (0.0 - 100.0)
    pub progress: f32,
    /// Accumulated errors during pipeline execution
    pub errors: Vec<TaleTrailError>,
    /// Original generation request
    pub request: GenerationRequest,
    /// Generated DAG structure (populated after Structure phase)
    pub dag: Option<DAG>,
    /// Prompt packages for each MCP service
    pub prompt_packages: HashMap<MCPServiceType, PromptPackage>,
    /// Current negotiation round counter (0 = not in negotiation, 1-N = active round)
    pub negotiation_round: u32,
}

impl PipelineState {
    /// Create new pipeline state from generation request
    ///
    /// # Arguments
    /// * `request` - Original generation request
    ///
    /// # Returns
    /// Pipeline state initialized to PromptGeneration phase with 0% progress
    pub fn new(request: GenerationRequest) -> Self {
        Self {
            current_phase: PipelinePhase::PromptGeneration,
            progress: 0.0,
            errors: Vec::new(),
            request,
            dag: None,
            prompt_packages: HashMap::new(),
            negotiation_round: 0,
        }
    }

    /// Advance to next pipeline phase
    ///
    /// Phase progression:
    /// PromptGeneration → Structure → Generation → Validation → Assembly → Complete
    ///
    /// # Returns
    /// Success if phase transition is valid, error if already Complete
    ///
    /// # Errors
    /// - `TaleTrailError::PipelineError`: Cannot advance past Complete phase
    pub fn advance_phase(&mut self) -> Result<()> {
        use PipelinePhase::*;

        self.current_phase = match self.current_phase {
            PromptGeneration => Structure,
            Structure => Generation,
            Generation => Validation,
            Validation => Assembly,
            Assembly => Complete,
            Complete => {
                return Err(TaleTrailError::PipelineError(
                    "Cannot advance past Complete phase".to_string(),
                ));
            }
        };

        Ok(())
    }

    /// Update progress percentage
    ///
    /// # Arguments
    /// * `percent` - Progress percentage (0.0 - 100.0)
    pub fn update_progress(&mut self, percent: f32) {
        self.progress = percent.clamp(0.0, 100.0);
    }

    /// Add error to error accumulation
    ///
    /// # Arguments
    /// * `error` - Error to accumulate
    pub fn add_error(&mut self, error: TaleTrailError) {
        self.errors.push(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> GenerationRequest {
        GenerationRequest {
            theme: "Test".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            node_count: Some(16),
            tenant_id: 1,
            educational_goals: Some(vec![]),
            vocabulary_level: Some(VocabularyLevel::Basic),
            required_elements: Some(vec![]),
            tags: None,
            prompt_packages: None,
            author_id: None,
            story_structure: None,
            dag_config: None,
            validation_policy: None,
        }
    }

    #[test]
    fn test_new_pipeline_state() {
        let request = create_test_request();

        let state = PipelineState::new(request.clone());

        assert_eq!(state.current_phase, PipelinePhase::PromptGeneration);
        assert_eq!(state.progress, 0.0);
        assert!(state.errors.is_empty());
        assert_eq!(state.request.theme, request.theme);
        assert!(state.dag.is_none());
        assert!(state.prompt_packages.is_empty());
    }

    #[test]
    fn test_phase_transitions() {
        let request = create_test_request();
        let mut state = PipelineState::new(request);

        assert_eq!(state.current_phase, PipelinePhase::PromptGeneration);

        state.advance_phase().unwrap();
        assert_eq!(state.current_phase, PipelinePhase::Structure);

        state.advance_phase().unwrap();
        assert_eq!(state.current_phase, PipelinePhase::Generation);

        state.advance_phase().unwrap();
        assert_eq!(state.current_phase, PipelinePhase::Validation);

        state.advance_phase().unwrap();
        assert_eq!(state.current_phase, PipelinePhase::Assembly);

        state.advance_phase().unwrap();
        assert_eq!(state.current_phase, PipelinePhase::Complete);

        // Should not advance past Complete
        assert!(state.advance_phase().is_err());
    }

    #[test]
    fn test_progress_tracking() {
        let request = create_test_request();
        let mut state = PipelineState::new(request);

        assert_eq!(state.progress, 0.0);

        state.update_progress(25.0);
        assert_eq!(state.progress, 25.0);

        state.update_progress(50.0);
        assert_eq!(state.progress, 50.0);

        state.update_progress(100.0);
        assert_eq!(state.progress, 100.0);

        // Test clamping
        state.update_progress(150.0);
        assert_eq!(state.progress, 100.0);

        state.update_progress(-10.0);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn test_error_accumulation() {
        let request = create_test_request();
        let mut state = PipelineState::new(request);

        assert!(state.errors.is_empty());

        state.add_error(TaleTrailError::GenerationError("Test error 1".to_string()));
        assert_eq!(state.errors.len(), 1);

        state.add_error(TaleTrailError::ValidationError("Test error 2".to_string()));
        assert_eq!(state.errors.len(), 2);
    }
}
