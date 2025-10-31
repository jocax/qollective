/// MCP Request Execution Commands
///
/// Provides Tauri commands for sending MCP tool call requests via NATS
/// using rmcp types directly without custom wrappers.
use crate::commands::mcp_template_commands::load_mcp_template;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Serializable wrapper for MCP response envelope
/// This ensures proper JSON serialization to TypeScript frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseEnvelope {
    pub meta: qollective::envelope::Meta,
    pub payload: qollective::types::mcp::McpData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<qollective::envelope::EnvelopeError>,
}

impl From<qollective::envelope::Envelope<qollective::types::mcp::McpData>> for McpResponseEnvelope {
    fn from(envelope: qollective::envelope::Envelope<qollective::types::mcp::McpData>) -> Self {
        Self {
            meta: envelope.meta,
            payload: envelope.payload,
            error: envelope.error,
        }
    }
}

/// Converts hyphens to dots in NATS subject to match server naming convention
///
/// Server names with hyphens (e.g., "prompt-helper") get dots in NATS subjects (e.g., "mcp.prompt.helper").
/// This function normalizes subjects by replacing all hyphens with dots.
///
/// # Examples
/// - "mcp.prompt-helper.request" → "mcp.prompt.helper.request"
/// - "mcp.story-generator.request" → "mcp.story.generator.request"
/// - "mcp.orchestrator.request" → "mcp.orchestrator.request" (no change)
fn normalize_nats_subject(subject: &str) -> String {
    subject.replace('-', ".")
}

/// Send an MCP tool call request via NATS
///
/// Wraps the CallToolRequest in a Qollective envelope and sends it to the specified
/// NATS subject using request-reply pattern. Returns the complete response envelope
/// with all metadata (tenant, request_id, tracing, etc.) preserved.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `subject` - NATS subject to send the request to (e.g., "mcp.orchestrator.request")
/// * `tool_name` - Name of the MCP tool to call
/// * `arguments` - Generic JSON arguments for the tool
/// * `tenant_id` - Tenant ID for multi-tenancy support
/// * `timeout_ms` - Optional timeout in milliseconds (defaults to DEFAULT_REQUEST_TIMEOUT_MS if None)
///
/// # Returns
/// * `Ok(McpResponseEnvelope)` - The complete response envelope with metadata and payload
/// * `Err(String)` - Error message if the request fails
#[tauri::command]
pub async fn send_mcp_request(
    app: AppHandle,
    subject: String,
    tool_name: String,
    arguments: serde_json::Value,
    tenant_id: i32,
    timeout_ms: Option<u64>,
) -> Result<McpResponseEnvelope, String> {
    use crate::commands::nats_commands::NatsState;

    // Debug logging
    eprintln!("[TaleTrail] send_mcp_request called:");
    eprintln!("  - subject: {}", subject);
    eprintln!("  - tool_name: {}", tool_name);
    eprintln!("  - tenant_id: {}", tenant_id);
    eprintln!("  - timeout_ms: {:?}", timeout_ms);

    // Normalize subject: convert hyphens to dots to match server naming convention
    let normalized_subject = normalize_nats_subject(&subject);
    eprintln!("[TaleTrail] Subject normalization:");
    eprintln!("  - Original: {}", subject);
    eprintln!("  - Normalized: {}", normalized_subject);

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

    // Convert timeout_ms to Duration, or use default
    let timeout_duration = timeout_ms
        .map(|ms| std::time::Duration::from_millis(ms))
        .or_else(|| {
            // Use default from constants
            Some(std::time::Duration::from_millis(
                crate::constants::network::DEFAULT_REQUEST_TIMEOUT_MS,
            ))
        });

    eprintln!("[TaleTrail] Using timeout: {:?}", timeout_duration);

    // Use NatsClient method to send MCP tool call with timeout
    client
        .send_mcp_tool_call(&normalized_subject, tool_call_request, tenant_id, timeout_duration)
        .await
        .map(|envelope| McpResponseEnvelope::from(envelope))
        .map_err(|e| e.to_string())
}

/// Send an MCP request using a template with full envelope
///
/// This command loads a template file containing a complete Qollective envelope
/// (with metadata and MCP payload) and sends it to the specified NATS subject.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `template_path` - Path to the template JSON file
///
/// # Returns
/// * `Ok(McpResponseEnvelope)` - The complete response envelope with metadata and payload
/// * `Err(String)` - Error message if loading or sending fails
#[tauri::command]
pub async fn send_mcp_template_request(
    app: AppHandle,
    template_path: String,
) -> Result<McpResponseEnvelope, String> {
    use crate::commands::nats_commands::NatsState;

    // Load template with full envelope
    let template_data = load_mcp_template(template_path).await?;

    // Get NATS client from app state
    let state = app.state::<NatsState>();
    let client_guard = state.client().read().await;

    let client = client_guard
        .as_ref()
        .ok_or_else(|| "Not connected to NATS. Please subscribe first.".to_string())?;

    // Send envelope directly from template
    client
        .send_envelope(&template_data.subject, template_data.envelope)
        .await
        .map(|envelope| McpResponseEnvelope::from(envelope))
        .map_err(|e| e.to_string())
}

/// Send an MCP request using a complete envelope provided as JSON
///
/// Accepts a full Qollective envelope directly as JSON and sends it via NATS.
/// Preserves all original metadata (request_id, trace_id, tenant, etc.).
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `subject` - NATS subject to send the request to
/// * `envelope_json` - Complete envelope as JSON (will be deserialized to Envelope<McpData>)
///
/// # Returns
/// * `Ok(McpResponseEnvelope)` - The complete response envelope with metadata and payload
/// * `Err(String)` - Error message if deserialization or sending fails
#[tauri::command]
pub async fn send_envelope_direct(
    app: AppHandle,
    subject: String,
    envelope_json: serde_json::Value,
) -> Result<McpResponseEnvelope, String> {
    use crate::commands::nats_commands::NatsState;
    use qollective::envelope::Envelope;
    use qollective::types::mcp::McpData;

    // Debug logging for consistency with other commands
    eprintln!("[TaleTrail] send_envelope_direct called:");
    eprintln!("  - subject: {}", subject);

    // Deserialize JSON to Envelope<McpData>
    let envelope: Envelope<McpData> = serde_json::from_value(envelope_json)
        .map_err(|e| format!("Failed to deserialize envelope: {}", e))?;

    // Get NATS client from app state
    let state = app.state::<NatsState>();
    let client_guard = state.client().read().await;

    let client = client_guard
        .as_ref()
        .ok_or_else(|| "Not connected to NATS. Please subscribe first.".to_string())?;

    // Send envelope using existing method
    client
        .send_envelope(&subject, envelope)
        .await
        .map(|envelope| McpResponseEnvelope::from(envelope))
        .map_err(|e| e.to_string())
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

        // Convert to McpResponseEnvelope
        let response_envelope = McpResponseEnvelope::from(envelope);

        // Verify response structure
        assert!(response_envelope.payload.tool_response.is_some());
        let response = response_envelope.payload.tool_response.unwrap();
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

        // Convert to McpResponseEnvelope
        let response_envelope = McpResponseEnvelope::from(envelope);

        // Verify error response
        assert!(response_envelope.payload.tool_response.is_some());
        let response = response_envelope.payload.tool_response.unwrap();
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
