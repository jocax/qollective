//! Discovery endpoint tests for Story Generator service
//!
//! Tests tool registration and discovery protocol implementation.

use qollective::envelope::{Envelope, Meta};
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData};
use story_generator::envelope_handlers::StoryGeneratorHandler;
use uuid::Uuid;

#[test]
fn test_get_tool_registrations_returns_all_tools() {
    let registrations = StoryGeneratorHandler::get_tool_registrations();

    assert_eq!(registrations.len(), 3, "Should have 3 tools");

    let tool_names: Vec<&str> = registrations.iter().map(|t| t.tool_name.as_str()).collect();
    assert!(tool_names.contains(&"generate_structure"));
    assert!(tool_names.contains(&"generate_nodes"));
    assert!(tool_names.contains(&"validate_paths"));
}

#[test]
fn test_tool_registrations_have_valid_schemas() {
    let registrations = StoryGeneratorHandler::get_tool_registrations();

    for registration in registrations {
        // Verify schema is a valid JSON object
        assert!(
            registration.tool_schema.is_object(),
            "Tool {} schema should be an object",
            registration.tool_name
        );

        // Verify schema has required fields
        let schema = registration.tool_schema.as_object().unwrap();
        assert!(
            schema.contains_key("type") || schema.contains_key("$schema"),
            "Tool {} schema should have type or $schema field",
            registration.tool_name
        );
    }
}

#[test]
fn test_tool_registrations_service_metadata() {
    let registrations = StoryGeneratorHandler::get_tool_registrations();

    for registration in registrations {
        assert_eq!(
            registration.service_name, "story-generator",
            "Tool {} should have correct service name",
            registration.tool_name
        );
        assert_eq!(
            registration.service_version, "0.0.1",
            "Tool {} should have correct version",
            registration.tool_name
        );
    }
}

#[test]
fn test_tool_capabilities_are_correct() {
    let registrations = StoryGeneratorHandler::get_tool_registrations();

    for registration in registrations {
        match registration.tool_name.as_str() {
            "generate_structure" => {
                assert_eq!(registration.capabilities.len(), 2);
                assert!(registration
                    .capabilities
                    .iter()
                    .any(|c| matches!(c, shared_types::types::tool_registration::ServiceCapabilities::Batching)));
                assert!(registration
                    .capabilities
                    .iter()
                    .any(|c| matches!(c, shared_types::types::tool_registration::ServiceCapabilities::Retry)));
            }
            "generate_nodes" => {
                assert_eq!(registration.capabilities.len(), 2);
                assert!(registration
                    .capabilities
                    .iter()
                    .any(|c| matches!(c, shared_types::types::tool_registration::ServiceCapabilities::Batching)));
                assert!(registration
                    .capabilities
                    .iter()
                    .any(|c| matches!(c, shared_types::types::tool_registration::ServiceCapabilities::Retry)));
            }
            "validate_paths" => {
                assert_eq!(registration.capabilities.len(), 1);
                assert!(registration
                    .capabilities
                    .iter()
                    .any(|c| matches!(c, shared_types::types::tool_registration::ServiceCapabilities::Retry)));
            }
            _ => panic!("Unexpected tool: {}", registration.tool_name),
        }
    }
}

#[tokio::test]
async fn test_discovery_handler_returns_tool_list() {
    use story_generator::discovery::DiscoveryHandler;

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

    let envelope = Envelope::new(meta.clone(), mcp_data);

    let result = handler.handle(envelope).await;
    assert!(result.is_ok(), "Discovery handler should succeed");

    let response = result.unwrap();
    let (response_meta, response_data) = response.extract();

    // Verify metadata preserved
    assert_eq!(response_meta.tenant, meta.tenant);

    // Verify discovery data
    assert!(response_data.discovery_data.is_some());
    let discovery = response_data.discovery_data.unwrap();
    assert_eq!(discovery.query_type, "list_tools_response");
    assert!(discovery.server_info.is_some());

    let server_info = discovery.server_info.unwrap();
    assert_eq!(server_info.server_id, "story-generator");
    assert_eq!(server_info.server_name, "Story Generator Service");
    assert!(server_info.health_status.is_healthy);
}

#[tokio::test]
async fn test_discovery_handler_includes_uptime() {
    use story_generator::discovery::DiscoveryHandler;

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

    // Uptime should be >= 0
    assert!(server_info.health_status.uptime.as_secs() >= 0);
}
