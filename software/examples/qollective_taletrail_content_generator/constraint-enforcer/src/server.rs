//! Constraint Enforcer MCP server module
//!
//! This module re-exports the envelope handler for backward compatibility.
//! The actual server startup logic is in main.rs.

// Re-export envelope handler types and functions for backward compatibility
pub use crate::envelope_handlers::{
    ConstraintEnforcerHandler,
    EnforceConstraintsParams,
    EnforceConstraintsResponse,
    SuggestCorrectionsParams,
    SuggestCorrectionsResponse,
    handle_enforce_constraints,
    handle_suggest_corrections,
    create_enforce_constraints_tool,
    create_suggest_corrections_tool,
    get_all_tools,
};
