/// Response Viewer Component
///
/// Displays MCP response with formatted content and metadata

use crate::state::McpContext;
use iocraft::prelude::*;

#[cfg(test)]
use crate::state::ResponseMetadata;

/// Props for ResponseViewer component
#[derive(Props)]
pub struct ResponseViewerProps {
    pub mcp_context: McpContext,
}

impl Default for ResponseViewerProps {
    fn default() -> Self {
        Self {
            mcp_context: McpContext::new(),
        }
    }
}

/// Response viewer component with formatted output
#[component]
pub fn ResponseViewer(
    _hooks: Hooks,
    props: &ResponseViewerProps,
) -> impl Into<AnyElement<'static>> {
    let response = props.mcp_context.response();
    let metadata = props.mcp_context.response_metadata();
    let request_in_progress = props.mcp_context.request_in_progress();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: "Response",
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // Show metadata if available
    if let Some(meta) = metadata {
        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Grey,
                    padding: 1,
                    margin_bottom: 1,
                ) {
                    View(flex_direction: FlexDirection::Column) {
                        Text(content: format!("Status: {}", meta.status), color: Color::White)
                        Text(content: format!("Duration: {}ms", meta.duration_ms), color: Color::White)
                        Text(content: format!("Timestamp: {}", meta.timestamp), color: Color::White)
                    }
                }
            }
            .into_any(),
        );
    }

    // Show response content or status message
    if request_in_progress {
        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Yellow,
                    padding: 2,
                ) {
                    Text(content: "⏳ Request in progress...", color: Color::Yellow)
                }
            }
            .into_any(),
        );
    } else if let Some(resp) = response {
        // Format response with word wrapping
        let formatted = format_response_content(&resp);

        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Green,
                    padding: 1,
                    flex_direction: FlexDirection::Column,
                ) {
                    Text(content: formatted, color: Color::White)
                }
            }
            .into_any(),
        );
    } else {
        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Grey,
                    padding: 2,
                ) {
                    Text(
                        content: "No response yet. Send a request to see results here.",
                        color: Color::Grey,
                    )
                }
            }
            .into_any(),
        );
    }

    // Help text
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "↑/↓: Scroll | Ctrl+C: Copy | H: View History",
                    color: Color::Grey,
                )
            }
        }
        .into_any(),
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Format response content with word wrapping and structure
fn format_response_content(content: &str) -> String {
    // Try to parse as JSON and pretty-print
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
            return pretty;
        }
    }

    // Otherwise return as-is with basic formatting
    content.to_string()
}

/// Extract text content from MCP response
pub fn extract_text_content(response_json: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(response_json)
        .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

    // Navigate to payload.tool_response.content array
    let content_array = value
        .get("payload")
        .and_then(|p| p.get("tool_response"))
        .and_then(|tr| tr.get("content"))
        .and_then(|c| c.as_array())
        .ok_or_else(|| "Response does not contain content array".to_string())?;

    // Extract text from each content item
    let mut texts = Vec::new();
    for item in content_array {
        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
            texts.push(text.to_string());
        }
    }

    if texts.is_empty() {
        Err("No text content found in response".to_string())
    } else {
        Ok(texts.join("\n\n"))
    }
}

/// Check if response indicates an error
pub fn is_error_response(response_json: &str) -> bool {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(response_json) {
        value
            .get("payload")
            .and_then(|p| p.get("tool_response"))
            .and_then(|tr| tr.get("is_error"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_viewer_states() {
        let ctx = McpContext::new();

        // Initial state - no response
        assert_eq!(ctx.response(), None);
        assert_eq!(ctx.response_metadata(), None);
        assert!(!ctx.request_in_progress());

        // Set request in progress
        ctx.set_request_in_progress(true);
        assert!(ctx.request_in_progress());

        // Set response
        let response = r#"{"result": "success"}"#.to_string();
        ctx.set_response(Some(response.clone()));
        assert_eq!(ctx.response(), Some(response));

        // Set metadata
        let metadata = ResponseMetadata {
            status: "success".to_string(),
            duration_ms: 250,
            timestamp: "2025-11-02T12:00:00Z".to_string(),
        };

        ctx.set_response_metadata(Some(metadata.clone()));
        let loaded_meta = ctx.response_metadata();
        assert!(loaded_meta.is_some());
        assert_eq!(loaded_meta.unwrap().status, "success");

        // Clear request in progress
        ctx.set_request_in_progress(false);
        assert!(!ctx.request_in_progress());
    }

    #[test]
    fn test_format_response_content() {
        // Test JSON formatting
        let json = r#"{"name":"test","value":123}"#;
        let formatted = format_response_content(json);
        assert!(formatted.contains('\n'));

        // Test plain text
        let text = "Plain text response";
        let formatted = format_response_content(text);
        assert_eq!(formatted, text);
    }

    #[test]
    fn test_extract_text_content() {
        let response_json = r#"{
            "meta": {"request_id": "123"},
            "payload": {
                "tool_response": {
                    "content": [
                        {"type": "text", "text": "First paragraph"},
                        {"type": "text", "text": "Second paragraph"}
                    ],
                    "is_error": false
                }
            }
        }"#;

        let result = extract_text_content(response_json);
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(text.contains("First paragraph"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn test_extract_text_content_error_cases() {
        // Invalid JSON
        let invalid_json = "not json";
        let result = extract_text_content(invalid_json);
        assert!(result.is_err());

        // Missing content array
        let no_content = r#"{"payload": {"tool_response": {}}}"#;
        let result = extract_text_content(no_content);
        assert!(result.is_err());

        // Empty content array
        let empty_content = r#"{
            "payload": {
                "tool_response": {
                    "content": []
                }
            }
        }"#;
        let result = extract_text_content(empty_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_error_response() {
        let error_response = r#"{
            "payload": {
                "tool_response": {
                    "is_error": true,
                    "content": [{"type": "text", "text": "Error message"}]
                }
            }
        }"#;

        assert!(is_error_response(error_response));

        let success_response = r#"{
            "payload": {
                "tool_response": {
                    "is_error": false,
                    "content": [{"type": "text", "text": "Success"}]
                }
            }
        }"#;

        assert!(!is_error_response(success_response));

        // Missing is_error field defaults to false
        let no_error_field = r#"{
            "payload": {
                "tool_response": {
                    "content": []
                }
            }
        }"#;

        assert!(!is_error_response(no_error_field));
    }

    #[test]
    fn test_response_metadata() {
        let metadata = ResponseMetadata {
            status: "success".to_string(),
            duration_ms: 150,
            timestamp: "2025-11-02T12:00:00Z".to_string(),
        };

        assert_eq!(metadata.status, "success");
        assert_eq!(metadata.duration_ms, 150);
        assert_eq!(metadata.timestamp, "2025-11-02T12:00:00Z");
    }
}
