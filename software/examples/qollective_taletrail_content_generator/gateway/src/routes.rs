//! Gateway HTTP routes using Qollective envelope-first architecture

use qollective::{
    prelude::{ContextDataHandler},
    envelope::Context,
    error::Result,
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tracing::info;

// =============================================================================
// Health Check Endpoint
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: String,
}

pub struct HealthHandler;

#[async_trait]
impl ContextDataHandler<HealthCheckRequest, HealthCheckResponse> for HealthHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        _request: HealthCheckRequest,
    ) -> Result<HealthCheckResponse> {
        info!("📋 REST REQUEST - Health check");

        Ok(HealthCheckResponse {
            status: "healthy".to_string(),
            service: "taletrail-gateway".to_string(),
            version: "0.1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}
