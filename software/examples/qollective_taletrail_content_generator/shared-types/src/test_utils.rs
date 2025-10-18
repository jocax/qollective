//! Test utilities for TaleTrail Content Generator
//!
//! Provides helper functions for creating test envelopes, metadata, and payloads
//! following the envelope-first architecture pattern.

use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};
use serde_json::{Map, Value};
use uuid::Uuid;
use chrono::Utc;

/// Creates a test envelope with MCP tool call
///
/// # Arguments
///
/// * `tool_name` - Name of the MCP tool to call
/// * `params` - Tool parameters as JSON Value
/// * `tenant_id` - Optional tenant ID for multi-tenancy testing
/// * `request_id` - Optional request ID for correlation
///
/// # Returns
///
/// Envelope<McpData> ready for testing
///
/// # Example
///
/// ```no_run
/// use shared_types::test_utils::create_test_mcp_call;
/// use serde_json::json;
///
/// let envelope = create_test_mcp_call(
///     "validate_content",
///     json!({
///         "content_node": { "id": "test-1" },
///         "age_group": "_6To8"
///     }),
///     Some("tenant-123".to_string()),
///     None
/// );
/// ```
pub fn create_test_mcp_call(
    tool_name: &str,
    params: Value,
    tenant_id: Option<String>,
    request_id: Option<Uuid>,
) -> Envelope<McpData> {
    let meta = create_test_meta(
        tenant_id.as_deref(),
        request_id.unwrap_or_else(Uuid::new_v4),
    );

    let arguments = if let Value::Object(map) = params {
        Some(map)
    } else {
        panic!("MCP tool parameters must be JSON object");
    };

    let tool_call = CallToolRequest {
        method: CallToolRequestMethod,
        params: CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments,
        },
        extensions: Default::default(),
    };

    let mcp_data = McpData {
        tool_call: Some(tool_call),
        tool_response: None,
        tool_registration: None,
        discovery_data: None,
    };

    Envelope::new(meta, mcp_data)
}

/// Creates test metadata with realistic values
///
/// # Arguments
///
/// * `tenant_id` - Optional tenant ID
/// * `request_id` - Request UUID for correlation
///
/// # Returns
///
/// Meta structure with populated fields for testing
pub fn create_test_meta(tenant_id: Option<&str>, request_id: Uuid) -> Meta {
    let mut meta = Meta::default();

    meta.tenant = tenant_id.map(String::from);
    meta.request_id = Some(request_id);
    meta.timestamp = Some(Utc::now());

    // Add tracing metadata with default values
    meta.tracing = Some(qollective::envelope::TracingMeta {
        trace_id: Some(Uuid::new_v4().to_string()),
        span_id: Some(Uuid::new_v4().to_string()),
        parent_span_id: None,
        baggage: Default::default(),
        sampling_rate: None,
        sampled: Some(true),
        trace_state: None,
        operation_name: Some("test_operation".to_string()),
        span_kind: None,
        span_status: None,
        tags: Default::default(),
    });

    meta
}

/// Creates test MCP parameters as JSON Value
///
/// Useful for creating parameters without type constraints
///
/// # Example
///
/// ```no_run
/// use shared_types::test_utils::create_test_params;
/// use serde_json::json;
///
/// let params = create_test_params(vec![
///     ("content_node", json!({"id": "node-1"})),
///     ("age_group", json!("_6To8")),
/// ]);
/// ```
pub fn create_test_params(fields: Vec<(&str, Value)>) -> Value {
    let mut map = Map::new();
    for (key, value) in fields {
        map.insert(key.to_string(), value);
    }
    Value::Object(map)
}

/// Extracts tool response content from MCP response envelope
///
/// # Arguments
///
/// * `envelope` - Response envelope from MCP handler
///
/// # Returns
///
/// Tuple of (is_error, content_text)
pub fn extract_tool_response(envelope: Envelope<McpData>) -> (bool, String) {
    let (_, data) = envelope.extract();

    if let Some(response) = data.tool_response {
        let is_error = response.is_error.unwrap_or(false);

        // Serialize content to JSON and extract text field
        let content = response
            .content
            .first()
            .and_then(|c| {
                // Serialize to JSON to extract text content
                serde_json::to_value(c).ok()
                    .and_then(|v| v.get("text").cloned())
                    .and_then(|t| t.as_str().map(String::from))
            })
            .unwrap_or_default();

        (is_error, content)
    } else {
        (true, "No tool response in envelope".to_string())
    }
}

/// Asserts that an envelope response is successful
///
/// # Panics
///
/// Panics if the response indicates an error
pub fn assert_success_response(envelope: Envelope<McpData>) {
    let (is_error, content) = extract_tool_response(envelope);
    assert!(!is_error, "Expected success but got error: {}", content);
}

/// Asserts that an envelope response is an error
///
/// # Panics
///
/// Panics if the response does not indicate an error
pub fn assert_error_response(envelope: Envelope<McpData>) {
    let (is_error, _) = extract_tool_response(envelope);
    assert!(is_error, "Expected error but got success");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_test_mcp_call() {
        let envelope = create_test_mcp_call(
            "test_tool",
            json!({"param1": "value1"}),
            Some("tenant-123".to_string()),
            None,
        );

        let (meta, data) = envelope.extract();

        assert_eq!(meta.tenant, Some("tenant-123".to_string()));
        assert!(meta.request_id.is_some());
        assert!(meta.timestamp.is_some());
        assert!(meta.tracing.is_some());

        assert!(data.tool_call.is_some());
        let tool_call = data.tool_call.unwrap();
        assert_eq!(tool_call.params.name, "test_tool");
    }

    #[test]
    fn test_create_test_meta() {
        let request_id = Uuid::new_v4();
        let meta = create_test_meta(Some("tenant-abc"), request_id);

        assert_eq!(meta.tenant, Some("tenant-abc".to_string()));
        assert_eq!(meta.request_id, Some(request_id));
        assert!(meta.tracing.is_some());
    }

    #[test]
    fn test_create_test_params() {
        let params = create_test_params(vec![
            ("field1", json!("value1")),
            ("field2", json!(42)),
        ]);

        assert!(params.is_object());
        let obj = params.as_object().unwrap();
        assert_eq!(obj.get("field1").unwrap(), &json!("value1"));
        assert_eq!(obj.get("field2").unwrap(), &json!(42));
    }
}
