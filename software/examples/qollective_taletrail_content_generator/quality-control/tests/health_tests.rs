//! Unit tests for Quality Control health check endpoint
//!
//! Tests the health check implementation without requiring
//! running NATS infrastructure.

use quality_control::discovery::HealthHandler;
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
    assert!(result.is_ok(), "Health handler should succeed");

    let response = result.unwrap();
    let (_, response_data) = response.extract();

    assert!(response_data.discovery_data.is_some(), "Should have discovery data");
    let health = response_data.discovery_data.unwrap();
    assert_eq!(health.query_type, "health_response");

    let server_info = health.server_info.unwrap();
    assert!(server_info.health_status.is_healthy, "Service should be healthy");
    assert_eq!(server_info.health_status.error_count, 0, "Error count should be 0");
}

#[tokio::test]
async fn test_health_reports_correct_tools_count() {
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
    let (_, response_data) = response.extract();

    let health = response_data.discovery_data.unwrap();
    let server_info = health.server_info.unwrap();

    // Should report 2 tools in metadata tags
    assert_eq!(server_info.metadata.tags.len(), 2, "Should report 2 tools");

    let tool_names: Vec<&str> = server_info.metadata.tags
        .iter()
        .map(|s| s.as_str())
        .collect();

    assert!(tool_names.contains(&"validate_content"), "Should include validate_content");
    assert!(tool_names.contains(&"batch_validate"), "Should include batch_validate");
}

#[tokio::test]
async fn test_health_uptime_increments() {
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

    let envelope1 = Envelope::new(meta.clone(), mcp_data.clone());

    // First health check
    let result1 = handler.handle(envelope1).await;
    assert!(result1.is_ok());

    let response1 = result1.unwrap();
    let (_, response_data1) = response1.extract();
    let health1 = response_data1.discovery_data.unwrap();
    let uptime1 = health1.server_info.unwrap().health_status.uptime;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let envelope2 = Envelope::new(meta, mcp_data);

    // Second health check
    let result2 = handler.handle(envelope2).await;
    assert!(result2.is_ok());

    let response2 = result2.unwrap();
    let (_, response_data2) = response2.extract();
    let health2 = response_data2.discovery_data.unwrap();
    let uptime2 = health2.server_info.unwrap().health_status.uptime;

    // Uptime should have increased
    assert!(uptime2 >= uptime1, "Uptime should increase");
}

#[tokio::test]
async fn test_health_preserves_metadata() {
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

    let request_id = Uuid::new_v4();
    let mut meta = Meta::default();
    meta.tenant = Some("health-tenant-456".to_string());
    meta.request_id = Some(request_id);

    let envelope = Envelope::new(meta, mcp_data);

    let result = handler.handle(envelope).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let (response_meta, _) = response.extract();

    assert_eq!(response_meta.tenant, Some("health-tenant-456".to_string()));
    assert_eq!(response_meta.request_id, Some(request_id));
}

#[tokio::test]
async fn test_health_server_identification() {
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
    let (_, response_data) = response.extract();

    let health = response_data.discovery_data.unwrap();
    let server_info = health.server_info.unwrap();

    assert_eq!(server_info.server_id, "quality-control");
    assert_eq!(server_info.server_name, "Quality Control Service");
    assert_eq!(server_info.metadata.version, "0.0.1");
}

#[tokio::test]
async fn test_health_description_includes_tool_count() {
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
    let (_, response_data) = response.extract();

    let health = response_data.discovery_data.unwrap();
    let server_info = health.server_info.unwrap();

    let description = server_info.metadata.description.unwrap();
    assert!(description.contains("healthy"), "Description should mention healthy status");
    assert!(description.contains("2 tools"), "Description should mention 2 tools");
}
