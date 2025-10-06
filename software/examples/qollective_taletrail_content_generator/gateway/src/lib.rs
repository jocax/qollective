//! Gateway library using Qollective REST server

pub mod config;
pub mod routes;
pub mod nats_client;
pub mod orchestrator_client;

// Re-export for convenience
pub use config::{get_gateway_config, GatewayConfig};
pub use routes::*;
pub use nats_client::connect_nats_with_tls;
