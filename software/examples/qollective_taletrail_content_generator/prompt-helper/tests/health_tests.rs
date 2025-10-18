//! Unit tests for Prompt Helper health check endpoint

use prompt_helper::discovery::HealthHandler;
use qollective::envelope::{Envelope, Meta};
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData};
use uuid::Uuid;

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
    let health = data.discovery_data.unwrap();
    assert_eq!(health.query_type, "health_response");
    let server_info = health.server_info.unwrap();
    assert!(server_info.health_status.is_healthy);
    assert_eq!(server_info.health_status.error_count, 0);
}

#[tokio::test]
async fn test_health_reports_four_tools() {
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
    let health = data.discovery_data.unwrap();
    let server_info = health.server_info.unwrap();
    assert_eq!(server_info.metadata.tags.len(), 4);
}
