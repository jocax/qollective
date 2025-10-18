//! Discovery and Health Check handlers for Quality Control service
//!
//! Implements MCP discovery protocol endpoints allowing the orchestrator
//! to discover available tools and check service health before execution.

use qollective::envelope::{Envelope, Meta};
use qollective::error::Result as QollectiveResult;
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData, HealthStatus};
use shared_types::types::tool_registration::DiscoveryInfo;
use std::future::Future;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use crate::envelope_handlers::QualityControlHandler;

/// Discovery request handler for listing available tools
#[derive(Clone)]
pub struct DiscoveryHandler {
    start_time: Arc<RwLock<SystemTime>>,
}

impl DiscoveryHandler {
    /// Create a new discovery handler
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(RwLock::new(SystemTime::now())),
        }
    }

    /// Get uptime in seconds
    async fn get_uptime(&self) -> u64 {
        let start = self.start_time.read().await;
        SystemTime::now()
            .duration_since(*start)
            .unwrap_or_default()
            .as_secs()
    }
}

impl EnvelopeHandler<McpData, McpData> for DiscoveryHandler {
    fn handle(
        &self,
        envelope: Envelope<McpData>,
    ) -> impl Future<Output = QollectiveResult<Envelope<McpData>>> + Send {
        async move {
            // Extract metadata
            let (meta, _data) = envelope.extract();

            tracing::info!(
                "Discovery request received (tenant: {:?}, request_id: {:?})",
                meta.tenant,
                meta.request_id
            );

            // Get tool registrations
            let tool_registrations = QualityControlHandler::get_tool_registrations();
            let uptime = self.get_uptime().await;

            // Create discovery info
            let discovery_info = DiscoveryInfo::healthy(tool_registrations.clone(), uptime);

            tracing::info!(
                "Returning discovery info: {} tools, uptime: {}s",
                discovery_info.available_tools.len(),
                uptime
            );

            // Convert to McpDiscoveryData format
            let tools_json = serde_json::to_value(&discovery_info)
                .map_err(|e| {
                    qollective::error::QollectiveError::mcp_tool_execution(format!(
                        "Failed to serialize discovery info: {}",
                        e
                    ))
                })?;

            let discovery_data = McpDiscoveryData {
                query_type: "list_tools_response".to_string(),
                tools: None, // Tools are in server_info for this implementation
                server_info: Some(qollective::types::mcp::McpServerInfo {
                    server_id: "quality-control".to_string(),
                    server_name: "Quality Control Service".to_string(),
                    tools: vec![], // Full details in custom field
                    capabilities: tool_registrations
                        .iter()
                        .flat_map(|t| t.capabilities.clone())
                        .map(|c| format!("{:?}", c))
                        .collect(),
                    metadata: qollective::types::mcp::ServerMetadata {
                        description: Some(
                            "Validates narrative content against age appropriateness, safety, and educational rubrics"
                                .to_string(),
                        ),
                        version: "0.0.1".to_string(),
                        contact: None,
                        documentation_url: None,
                        tags: vec!["validation".to_string(), "quality-control".to_string()],
                    },
                    async_config: None,
                    health_status: HealthStatus {
                        is_healthy: true,
                        last_check: SystemTime::now(),
                        response_time: std::time::Duration::from_millis(0),
                        error_count: 0,
                        uptime: std::time::Duration::from_secs(uptime),
                    },
                }),
            };

            // Create response McpData
            let response_data = McpData {
                tool_call: None,
                tool_response: None,
                tool_registration: None,
                discovery_data: Some(discovery_data),
            };

            // Create response envelope (preserving metadata)
            Ok(Envelope::new(meta, response_data))
        }
    }
}

/// Health check handler
#[derive(Clone)]
pub struct HealthHandler {
    start_time: Arc<RwLock<SystemTime>>,
}

impl HealthHandler {
    /// Create a new health handler
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(RwLock::new(SystemTime::now())),
        }
    }

    /// Get uptime in seconds
    async fn get_uptime(&self) -> u64 {
        let start = self.start_time.read().await;
        SystemTime::now()
            .duration_since(*start)
            .unwrap_or_default()
            .as_secs()
    }
}

impl EnvelopeHandler<McpData, McpData> for HealthHandler {
    fn handle(
        &self,
        envelope: Envelope<McpData>,
    ) -> impl Future<Output = QollectiveResult<Envelope<McpData>>> + Send {
        async move {
            // Extract metadata
            let (meta, _data) = envelope.extract();

            tracing::debug!("Health check request received");

            let tool_registrations = QualityControlHandler::get_tool_registrations();
            let uptime = self.get_uptime().await;

            // Create health response
            let health_data = McpDiscoveryData {
                query_type: "health_response".to_string(),
                tools: None,
                server_info: Some(qollective::types::mcp::McpServerInfo {
                    server_id: "quality-control".to_string(),
                    server_name: "Quality Control Service".to_string(),
                    tools: vec![],
                    capabilities: vec!["healthy".to_string()],
                    metadata: qollective::types::mcp::ServerMetadata {
                        description: Some(format!(
                            "healthy - {} tools available",
                            tool_registrations.len()
                        )),
                        version: "0.0.1".to_string(),
                        contact: None,
                        documentation_url: None,
                        tags: tool_registrations
                            .iter()
                            .map(|t| t.tool_name.clone())
                            .collect(),
                    },
                    async_config: None,
                    health_status: HealthStatus {
                        is_healthy: true,
                        last_check: SystemTime::now(),
                        response_time: std::time::Duration::from_millis(0),
                        error_count: 0,
                        uptime: std::time::Duration::from_secs(uptime),
                    },
                }),
            };

            // Create response McpData
            let response_data = McpData {
                tool_call: None,
                tool_response: None,
                tool_registration: None,
                discovery_data: Some(health_data),
            };

            Ok(Envelope::new(meta, response_data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qollective::envelope::Meta;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_discovery_handler_returns_tools() {
        let handler = DiscoveryHandler::new();

        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: Some(McpDiscoveryData {
                query_type: "list_tools".to_string(),
                tools: None,
                server_info: None,
            }),
        };

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());

        let envelope = Envelope::new(meta, mcp_data);

        let result = handler.handle(envelope).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let (_, data) = response.extract();

        assert!(data.discovery_data.is_some());
        let discovery = data.discovery_data.unwrap();
        assert_eq!(discovery.query_type, "list_tools_response");
        assert!(discovery.server_info.is_some());

        let server_info = discovery.server_info.unwrap();
        assert_eq!(server_info.server_id, "quality-control");
        assert!(server_info.health_status.is_healthy);
    }

    #[tokio::test]
    async fn test_health_handler_returns_healthy() {
        let handler = HealthHandler::new();

        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: Some(McpDiscoveryData {
                query_type: "health".to_string(),
                tools: None,
                server_info: None,
            }),
        };

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());

        let envelope = Envelope::new(meta, mcp_data);

        let result = handler.handle(envelope).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let (_, data) = response.extract();

        assert!(data.discovery_data.is_some());
        let health = data.discovery_data.unwrap();
        assert_eq!(health.query_type, "health_response");

        let server_info = health.server_info.unwrap();
        assert!(server_info.health_status.is_healthy);
        assert_eq!(server_info.health_status.error_count, 0);
    }

    #[tokio::test]
    async fn test_discovery_handler_uptime_increases() {
        let handler = DiscoveryHandler::new();

        // Get uptime immediately
        let uptime1 = handler.get_uptime().await;

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get uptime again
        let uptime2 = handler.get_uptime().await;

        assert!(uptime2 >= uptime1);
    }
}
