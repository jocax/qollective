//! Quality Control MCP server module
//!
//! This module re-exports the envelope handler for backward compatibility.
//! The actual server startup logic is in main.rs.

// Re-export envelope handler types and functions for backward compatibility
pub use crate::envelope_handlers::{
    QualityControlHandler,
    ValidateContentParams,
    ValidateContentResponse,
    BatchValidateParams,
    BatchValidateResponse,
    handle_validate_content,
    handle_batch_validate,
    create_validate_content_tool,
    create_batch_validate_tool,
    get_all_tools,
};
