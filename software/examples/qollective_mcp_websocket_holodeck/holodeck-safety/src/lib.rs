// ABOUTME: Library entry point for holodeck-safety - safety monitoring and emergency protocols
// ABOUTME: Provides MCP server implementation for holodeck safety analysis with rmcp integration

pub mod server;
pub use server::HolodeckSafetyServer;

// Re-export request types for main.rs
pub use server::{SafetyAnalysisRequest, ComplianceValidationRequest, RiskAssessmentRequest};