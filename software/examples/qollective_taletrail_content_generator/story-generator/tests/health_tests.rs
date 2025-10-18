//! Health check endpoint tests for Story Generator service

use qollective::envelope::{Envelope, Meta};
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData};
use story_generator::discovery::HealthHandler;
use uuid::Uuid;

#[tokio::test]
async fn test_health_endpoint_returns_healthy() {
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
    assert!(result.is_ok(), "Health check should succeed");

    let response = result.unwrap();
    let (_, response_data) = response.extract();

    assert!(response_data.discovery_data.is_some());
    let health = response_data.discovery_data.unwrap();
    assert_eq!(health.query_type, "health_response");

    let server_info = health.server_info.unwrap();
    assert!(server_info.health_status.is_healthy);
    assert_eq!(server_info.health_status.error_count, 0);
}

#[tokio::test]
async fn test_health_endpoint_returns_tool_count() {
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
    meta.request_id = Some(Uuid::new_v4());

    let envelope = Envelope::new(meta, mcp_data);

    let result = handler.handle(envelope).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let (_, response_data) = response.extract();

    let health = response_data.discovery_data.unwrap();
    let server_info = health.server_info.unwrap();

    // Should list tool names in tags
    assert_eq!(server_info.metadata.tags.len(), 3); // 3 tools
    assert!(server_info.metadata.tags.contains(&"generate_structure".to_string()));
    assert!(server_info.metadata.tags.contains(&"generate_nodes".to_string()));
    assert!(server_info.metadata.tags.contains(&"validate_paths".to_string()));
}

#[tokio::test]
async fn test_health_endpoint_uptime_increments() {
    let handler = HealthHandler::new();

    // First health check
    let mcp_data1 = McpData {
        tool_call: None,
        tool_response: None,
        tool_registration: None,
        discovery_data: Some(McpDiscoveryData {
            query_type: "health".to_string(),
            tools: None,
            server_info: None,
        }),
    };

    let envelope1 = Envelope::new(Meta::default(), mcp_data1);
    let result1 = handler.handle(envelope1).await.unwrap();
    let (_, data1) = result1.extract();
    let uptime1 = data1.discovery_data.unwrap().server_info.unwrap().health_status.uptime;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Second health check
    let mcp_data2 = McpData {
        tool_call: None,
        tool_response: None,
        tool_registration: None,
        discovery_data: Some(McpDiscoveryData {
            query_type: "health".to_string(),
            tools: None,
            server_info: None,
        }),
    };

    let envelope2 = Envelope::new(Meta::default(), mcp_data2);
    let result2 = handler.handle(envelope2).await.unwrap();
    let (_, data2) = result2.extract();
    let uptime2 = data2.discovery_data.unwrap().server_info.unwrap().health_status.uptime;

    assert!(uptime2 >= uptime1, "Uptime should increase over time");
}

#[tokio::test]
async fn test_health_endpoint_preserves_metadata() {
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
    let request_id = Uuid::new_v4();
    meta.tenant = Some("test-tenant".to_string());
    meta.request_id = Some(request_id);

    let envelope = Envelope::new(meta, mcp_data);

    let result = handler.handle(envelope).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let (response_meta, _) = response.extract();

    // Metadata should be preserved
    assert_eq!(response_meta.tenant, Some("test-tenant".to_string()));
    assert_eq!(response_meta.request_id, Some(request_id));
}
