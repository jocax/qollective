//! Gateway library using Qollective REST server

pub mod config;
pub mod routes;

// Re-export for convenience
pub use config::get_gateway_config;
pub use routes::*;
