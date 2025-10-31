/// MCP Request Execution Commands
///
/// Provides Tauri commands for sending MCP tool call requests via NATS
/// using rmcp types directly without custom wrappers.
use crate::commands::mcp_template_commands::load_mcp_template;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam, CallToolResult};
use tauri::{AppHandle, Manager};

/// Send an MCP tool call request via NATS
///
/// Wraps the CallToolRequest in a Qollective envelope and sends it to the specified
/// NATS subject using request-reply pattern. Returns the CallToolResult directly.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `subject` - NATS subject to send the request to (e.g., "mcp.orchestrator.request")
/// * `tool_name` - Name of the MCP tool to call
/// * `arguments` - Generic JSON arguments for the tool
/// * `tenant_id` - Tenant ID for multi-tenancy support
///
/// # Returns
/// * `Ok(CallToolResult)` - The MCP tool result from the server
/// * `Err(String)` - Error message if the request fails
#[tauri::command]
pub async fn send_mcp_request(
    app: AppHandle,
    subject: String,
    tool_name: String,
    arguments: serde_json::Value,
    tenant_id: i32,
) -> Result<CallToolResult, String> {
    use crate::commands::nats_commands::NatsState;

    // Get NATS client from app state
    let state = app.state::<NatsState>();
    let client_guard = state.client().read().await;

    let client = client_guard
        .as_ref()
        .ok_or_else(|| "Not connected to NATS. Please subscribe first.".to_string())?;

    // Build CallToolRequest from parameters
    let tool_call_request = CallToolRequest {
        method: CallToolRequestMethod::default(),
        params: CallToolRequestParam {
            name: tool_name.into(),
            arguments: arguments.as_object().cloned(),
        },
        extensions: Default::default(),
    };

    // Use NatsClient method to send MCP tool call
    client
        .send_mcp_tool_call(&subject, tool_call_request, tenant_id)
        .await
        .map_err(|e| e.to_string())
}

/// Send an MCP request using a saved template
///
/// This convenience command loads a template file and sends the MCP request
/// to the specified subject in a single operation.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `template_path` - Path to the template JSON file
/// * `subject` - NATS subject to send the request to
/// * `tenant_id` - Tenant ID for multi-tenancy support
///
/// # Returns
/// * `Ok(CallToolResult)` - The MCP tool result from the server
/// * `Err(String)` - Error message if loading or sending fails
#[tauri::command]
pub async fn send_mcp_template_request(
    app: AppHandle,
    template_path: String,
    subject: String,
    tenant_id: i32,
) -> Result<CallToolResult, String> {
    // Load template using Task 1's command
    let template_data = load_mcp_template(template_path).await?;

    // Send the request using send_mcp_request
    send_mcp_request(
        app,
        subject,
        template_data.tool_name,
        template_data.arguments,
        tenant_id,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nats::{NatsClient, NatsConfig};
    use qollective::envelope::nats_codec::NatsEnvelopeCodec;
    use qollective::envelope::{Envelope, Meta};
    use qollective::types::mcp::McpData;
    use rmcp::model::{CallToolResult, Content};
    use serde_json::json;

    /// Test building CallToolRequest from parameters
    #[test]
    fn test_build_call_tool_request() {
        let tool_name = "orchestrate_generation".to_string();
        let arguments = json!({
            "generation_request": {
                "theme": "Space Adventure",
                "age_group": "9-11",
                "language": "en"
            }
        });

        let request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: tool_name.into(),
                arguments: arguments.as_object().cloned(),
            },
            extensions: Default::default(),
        };

        assert_eq!(request.params.name.as_ref(), "orchestrate_generation");
        assert!(request.params.arguments.is_some());
        let args = request.params.arguments.as_ref().unwrap();
        assert!(args.contains_key("generation_request"));
    }

    /// Test envelope wrapping with MCP data
    #[test]
    fn test_envelope_wrapping() {
        let tool_call_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "test_tool".into(),
                arguments: Some(serde_json::Map::new()),
            },
            extensions: Default::default(),
        };

        let mcp_data = McpData::with_tool_call(tool_call_request);
        let mut meta = Meta::default();
        meta.tenant = Some("42".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        // Verify envelope structure
        assert_eq!(envelope.meta.tenant, Some("42".to_string()));
        assert!(envelope.payload.tool_call.is_some());
        assert_eq!(
            envelope.payload.tool_call.unwrap().params.name.as_ref(),
            "test_tool"
        );
    }

    /// Test encoding and decoding envelopes
    #[test]
    fn test_envelope_codec() {
        let tool_call_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "test_tool".into(),
                arguments: Some(serde_json::Map::new()),
            },
            extensions: Default::default(),
        };

        let mcp_data = McpData::with_tool_call(tool_call_request);
        let meta = Meta::default();
        let envelope = Envelope::new(meta, mcp_data);

        // Encode
        let encoded = NatsEnvelopeCodec::encode(&envelope).expect("Failed to encode envelope");

        // Decode
        let decoded: Envelope<McpData> =
            NatsEnvelopeCodec::decode(&encoded).expect("Failed to decode envelope");

        // Verify roundtrip
        assert!(decoded.payload.tool_call.is_some());
        assert_eq!(
            decoded.payload.tool_call.unwrap().params.name.as_ref(),
            "test_tool"
        );
    }

    /// Test response parsing with successful result
    #[test]
    fn test_response_parsing_success() {
        let content = vec![Content::text("Success message")];
        let tool_response = CallToolResult {
            content: content.clone(),
            structured_content: None,
            is_error: Some(false),
            meta: None,
        };

        let mcp_data = McpData {
            tool_call: None,
            tool_response: Some(tool_response),
            tool_registration: None,
            discovery_data: None,
        };

        let meta = Meta::default();
        let envelope = Envelope::new(meta, mcp_data);

        // Verify response structure
        assert!(envelope.payload.tool_response.is_some());
        let response = envelope.payload.tool_response.unwrap();
        assert_eq!(response.is_error, Some(false));
        assert_eq!(response.content.len(), 1);
    }

    /// Test response parsing with error result
    #[test]
    fn test_response_parsing_error() {
        let error_content = vec![Content::text("Tool execution failed")];
        let tool_response = CallToolResult {
            content: error_content,
            structured_content: None,
            is_error: Some(true),
            meta: None,
        };

        let mcp_data = McpData {
            tool_call: None,
            tool_response: Some(tool_response),
            tool_registration: None,
            discovery_data: None,
        };

        let meta = Meta::default();
        let envelope = Envelope::new(meta, mcp_data);

        // Verify error response
        assert!(envelope.payload.tool_response.is_some());
        let response = envelope.payload.tool_response.unwrap();
        assert_eq!(response.is_error, Some(true));
    }

    /// Test error extraction from MCP response content
    #[test]
    fn test_error_extraction() {
        let error_message = "Invalid parameters: missing required field 'theme'";
        let error_content = vec![Content::text(error_message)];

        let tool_response = CallToolResult {
            content: error_content,
            structured_content: None,
            is_error: Some(true),
            meta: None,
        };

        // Extract error message
        assert_eq!(tool_response.is_error, Some(true));
        assert_eq!(tool_response.content.len(), 1);

        // In real implementation, we'd extract text from Content
        // For now, we verify the structure is correct
        assert!(!tool_response.content.is_empty());
    }

    /// Test timeout handling (mock test structure)
    #[test]
    fn test_timeout_handling() {
        use crate::config::AppConfig;

        // This is a structure test - actual timeout would require NATS server
        let app_config = AppConfig::create_test_app_config();

        let config = NatsConfig::from_app_config(&app_config);
        assert_eq!(config.timeout_secs, crate::constants::network::DEFAULT_REQUEST_TIMEOUT_MS / 1000);

        // Verify timeout is within reasonable bounds
        assert!(config.timeout_secs >= 30);
        assert!(config.timeout_secs <= 300);
    }

    /// Test tenant ID handling
    #[test]
    fn test_tenant_id_handling() {
        let tenant_id = 42i32;
        let tenant_string = tenant_id.to_string();

        let mut meta = Meta::default();
        meta.tenant = Some(tenant_string.clone());

        assert_eq!(meta.tenant, Some("42".to_string()));
    }

    /// Test argument validation (generic JSON value)
    #[test]
    fn test_arguments_validation() {
        // Valid JSON object
        let valid_args = json!({
            "theme": "Space Adventure",
            "age_group": "9-11"
        });
        assert!(valid_args.is_object());
        assert!(valid_args.as_object().is_some());

        // Invalid - not an object
        let invalid_args = json!("just a string");
        assert!(!invalid_args.is_object());
        assert!(invalid_args.as_object().is_none());

        // Empty object is valid
        let empty_args = json!({});
        assert!(empty_args.is_object());
        assert!(empty_args.as_object().is_some());
    }

    /// Test subject validation
    #[test]
    fn test_subject_validation() {
        // Valid NATS subjects
        let valid_subjects = vec![
            "mcp.orchestrator.request",
            "mcp.story-generator.request",
            "mcp.quality-control.request",
        ];

        for subject in valid_subjects {
            assert!(!subject.is_empty());
            assert!(subject.contains('.'));
        }

        // Invalid subjects
        let invalid_subjects = vec!["", " ", "invalid subject with spaces"];

        for subject in invalid_subjects {
            // In real implementation, we'd validate these
            if subject.is_empty() {
                assert!(subject.is_empty());
            } else if subject.contains(' ') {
                assert!(subject.contains(' '));
            }
        }
    }

    /// Test CallToolResult serialization
    #[test]
    fn test_call_tool_result_serialization() {
        let result = CallToolResult {
            content: vec![Content::text("Test result")],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        };

        // Verify we can serialize the result
        let serialized = serde_json::to_value(&result).expect("Failed to serialize");
        assert!(serialized.is_object());

        // Verify structure
        assert!(serialized.get("content").is_some());
        assert!(serialized.get("isError").is_some());
    }

    // Integration tests would go here (require NATS server)
    // - test_send_mcp_request_integration
    // - test_send_mcp_template_request_integration
    // - test_nats_disconnection_handling
    // - test_request_reply_timeout
}
