//! TaleTrail Orchestrator Library
//!
//! Provides orchestration logic for coordinating MCP services in the TaleTrail pipeline.

pub mod config;
pub mod discovery;
pub mod envelope_handlers;
pub mod events;
pub mod mcp_client;
pub mod negotiation;
pub mod negotiation_state;
pub mod orchestrator;
pub mod pipeline;
pub mod prompt_orchestration;
pub mod retry;
pub mod validation_issues;

// Re-export key types
pub use config::OrchestratorConfig;
pub use discovery::DiscoveryClient;
pub use envelope_handlers::OrchestratorHandler;
pub use events::{EventPublisher, PipelineEvent};
pub use negotiation::{CorrectionPlan, Negotiator, NegotiationRound};
pub use negotiation_state::{NegotiationState, MAX_NEGOTIATION_ROUNDS};
pub use orchestrator::Orchestrator;
pub use pipeline::{PipelinePhase, PipelineState};
pub use prompt_orchestration::PromptOrchestrator;
pub use validation_issues::{
    aggregate_all_issues, aggregate_constraint_issues, aggregate_quality_issues,
    severity_from_quality_score, severity_from_violation_count, ConstraintIssueKind,
    IssueSeverity, IssueType, QualityIssueKind, ValidationIssue,
};
