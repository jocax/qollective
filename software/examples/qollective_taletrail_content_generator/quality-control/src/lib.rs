//! Quality Control library (stub for Phase 0)

pub mod config;
pub mod discovery;
pub mod envelope_handlers;
pub mod execution_logger;
pub mod rubrics;
pub mod server;
pub mod validation;

// Re-export main handler type
pub use envelope_handlers::QualityControlHandler;
