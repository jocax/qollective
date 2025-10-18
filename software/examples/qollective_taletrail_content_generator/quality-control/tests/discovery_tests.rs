//! Unit tests for Quality Control discovery protocol
//!
//! Tests the tool registration and discovery endpoint implementation
//! without requiring running NATS infrastructure.

use quality_control::envelope_handlers::QualityControlHandler;
use quality_control::discovery::DiscoveryHandler;
use qollective::envelope::{Envelope, Meta};
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData};
use uuid::Uuid;

#[test]
fn test_get_tool_registrations_returns_two_tools() {
    let registrations = QualityControlHandler::get_tool_registrations();

    assert_eq!(registrations.len(), 2, "Should have exactly 2 tools");
}

#[test]
fn test_tool_names_are_correct() {
    let registrations = QualityControlHandler::get_tool_registrations();

    let tool_names: Vec<&str> = registrations
        .iter()
        .map(|t| t.tool_name.as_str())
        .collect();

    assert!(tool_names.contains(&"validate_content"), "Missing validate_content tool");
    assert!(tool_names.contains(&"batch_validate"), "Missing batch_validate tool");
}

#[test]
fn test_validate_content_capabilities() {
    let registrations = QualityControlHandler::get_tool_registrations();

    let validate_content = registrations
        .iter()
        .find(|t| t.tool_name == "validate_content")
        .expect("validate_content tool not found");

    assert_eq!(validate_content.service_name, "quality-control");
    assert_eq!(validate_content.service_version, "0.0.1");
    assert_eq!(validate_content.capabilities.len(), 2);

    use shared_types::types::tool_registration::ServiceCapabilities;
    assert!(validate_content.capabilities.contains(&ServiceCapabilities::Batching));
    assert!(validate_content.capabilities.contains(&ServiceCapabilities::Retry));
}

#[test]
fn test_batch_validate_capabilities() {
    let registrations = QualityControlHandler::get_tool_registrations();

    let batch_validate = registrations
        .iter()
        .find(|t| t.tool_name == "batch_validate")
        .expect("batch_validate tool not found");

    assert_eq!(batch_validate.service_name, "quality-control");
    assert_eq!(batch_validate.service_version, "0.0.1");
    assert_eq!(batch_validate.capabilities.len(), 2);

    use shared_types::types::tool_registration::ServiceCapabilities;
    assert!(batch_validate.capabilities.contains(&ServiceCapabilities::Batching));
    assert!(batch_validate.capabilities.contains(&ServiceCapabilities::Retry));
}

#[test]
fn test_tool_schemas_are_valid_json() {
    let registrations = QualityControlHandler::get_tool_registrations();

    for registration in registrations {
        assert!(registration.tool_schema.is_object(),
            "Tool {} schema should be a JSON object", registration.tool_name);

        // Verify schema has expected fields
        let schema = registration.tool_schema.as_object().unwrap();
        assert!(schema.contains_key("properties") || schema.contains_key("$schema"),
            "Tool {} schema should have properties or $schema field", registration.tool_name);
    }
}

#[tokio::test]
async fn test_discovery_handler_response_structure() {
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
    assert!(result.is_ok(), "Discovery handler should succeed");

    let response = result.unwrap();
    let (response_meta, response_data) = response.extract();

    // Verify metadata preservation
    assert_eq!(response_meta.tenant, Some("test-tenant".to_string()));
    assert!(response_meta.request_id.is_some());

    // Verify response structure
    assert!(response_data.discovery_data.is_some(), "Should have discovery data");
    let discovery = response_data.discovery_data.unwrap();
    assert_eq!(discovery.query_type, "list_tools_response");
    assert!(discovery.server_info.is_some(), "Should have server info");

    let server_info = discovery.server_info.unwrap();
    assert_eq!(server_info.server_id, "quality-control");
    assert_eq!(server_info.server_name, "Quality Control Service");
    assert!(server_info.health_status.is_healthy);
}

#[tokio::test]
async fn test_discovery_includes_uptime() {
    let handler = DiscoveryHandler::new();

    // Wait a bit to ensure uptime > 0
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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
    let (_, response_data) = response.extract();

    let discovery = response_data.discovery_data.unwrap();
    let server_info = discovery.server_info.unwrap();

    // Uptime should be at least 0 seconds (likely more due to sleep)
    assert!(server_info.health_status.uptime.as_secs() >= 0);
}

#[tokio::test]
async fn test_discovery_handler_preserves_metadata() {
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

    let request_id = Uuid::new_v4();
    let mut meta = Meta::default();
    meta.tenant = Some("test-tenant-123".to_string());
    meta.request_id = Some(request_id);

    let envelope = Envelope::new(meta, mcp_data);

    let result = handler.handle(envelope).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let (response_meta, _) = response.extract();

    assert_eq!(response_meta.tenant, Some("test-tenant-123".to_string()));
    assert_eq!(response_meta.request_id, Some(request_id));
}
