//! TaleTrail Orchestrator Library
//!
//! Provides orchestration logic for coordinating MCP services in the TaleTrail pipeline.

pub mod config;
pub mod events;
pub mod negotiation;
pub mod orchestrator;
pub mod pipeline;
pub mod prompt_orchestration;

// Re-export key types
pub use config::OrchestratorConfig;
pub use events::{EventPublisher, PipelineEvent};
pub use negotiation::{CorrectionPlan, Negotiator, NegotiationRound};
pub use orchestrator::Orchestrator;
pub use pipeline::{PipelinePhase, PipelineState};
pub use prompt_orchestration::PromptOrchestrator;
