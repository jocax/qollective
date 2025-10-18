//! Unit tests for Constraint Enforcer health check endpoint

use constraint_enforcer::discovery::HealthHandler;
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
    let (_, response_data) = response.extract();
    assert!(response_data.discovery_data.is_some());
    let health = response_data.discovery_data.unwrap();
    assert_eq!(health.query_type, "health_response");
    let server_info = health.server_info.unwrap();
    assert!(server_info.health_status.is_healthy);
    assert_eq!(server_info.health_status.error_count, 0);
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
    assert_eq!(server_info.metadata.tags.len(), 2);
    let tool_names: Vec<&str> = server_info.metadata.tags.iter().map(|s| s.as_str()).collect();
    assert!(tool_names.contains(&"enforce_constraints"));
    assert!(tool_names.contains(&"suggest_corrections"));
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
    let result1 = handler.handle(envelope1).await;
    assert!(result1.is_ok());
    let response1 = result1.unwrap();
    let (_, response_data1) = response1.extract();
    let health1 = response_data1.discovery_data.unwrap();
    let uptime1 = health1.server_info.unwrap().health_status.uptime;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let envelope2 = Envelope::new(meta, mcp_data);
    let result2 = handler.handle(envelope2).await;
    assert!(result2.is_ok());
    let response2 = result2.unwrap();
    let (_, response_data2) = response2.extract();
    let health2 = response_data2.discovery_data.unwrap();
    let uptime2 = health2.server_info.unwrap().health_status.uptime;
    assert!(uptime2 >= uptime1);
}
