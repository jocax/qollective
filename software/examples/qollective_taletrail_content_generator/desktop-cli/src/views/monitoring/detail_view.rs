/// Message Detail View for NATS Monitoring
///
/// Shows expanded view of selected message with formatted JSON and metadata

use crate::nats::monitoring::NatsMessage;
use crate::state::MonitorContext;
use iocraft::prelude::*;

/// Props for DetailView component
#[derive(Props)]
pub struct DetailViewProps {
    pub monitor_ctx: MonitorContext,
}

impl Default for DetailViewProps {
    fn default() -> Self {
        Self {
            monitor_ctx: MonitorContext::new(),
        }
    }
}

/// Detail view component showing full message information
#[component]
pub fn DetailView(_hooks: Hooks, props: &DetailViewProps) -> impl Into<AnyElement<'static>> {
    let monitor_ctx = &props.monitor_ctx;

    // Check if detail view should be shown
    if !monitor_ctx.show_detail() {
        return element! {
            View {}
        }
        .into_any();
    }

    // Get selected message
    let selected_msg = monitor_ctx.selected_message();

    let content = match selected_msg {
        Some(msg) => render_message_detail(&msg),
        None => render_no_selection(),
    };

    content
}

/// Render detailed view of a message
fn render_message_detail(msg: &NatsMessage) -> AnyElement<'static> {
    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Double,
                border_color: Color::Magenta,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: "Message Details",
                    color: Color::Magenta,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // Metadata section
    elements.push(render_metadata_section(msg));

    // Payload section
    elements.push(render_payload_section(msg));

    // Footer
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "ESC: Close detail view",
                    color: Color::Grey,
                )
            }
        }
        .into_any(),
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
            padding: 2,
            border_style: BorderStyle::Double,
            border_color: Color::Magenta,
        ) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Render metadata section
fn render_metadata_section(msg: &NatsMessage) -> AnyElement<'static> {
    let mut metadata_elements: Vec<AnyElement> = Vec::new();

    metadata_elements.push(
        element! {
            Text(
                content: "Envelope Metadata",
                weight: Weight::Bold,
                color: Color::Cyan,
            )
        }
        .into_any(),
    );

    // Timestamp
    let timestamp_formatted = format_timestamp(&msg.timestamp);
    metadata_elements.push(
        element! {
            Text(
                content: format!("  Timestamp: {}", timestamp_formatted),
                color: Color::White,
            )
        }
        .into_any(),
    );

    // Subject
    metadata_elements.push(
        element! {
            Text(
                content: format!("  Subject: {}", msg.subject),
                color: Color::Yellow,
            )
        }
        .into_any(),
    );

    // Endpoint
    metadata_elements.push(
        element! {
            Text(
                content: format!("  Endpoint: {}", msg.endpoint),
                color: Color::White,
            )
        }
        .into_any(),
    );

    // Message type
    metadata_elements.push(
        element! {
            Text(
                content: format!("  Type: {}", msg.message_type),
                color: Color::White,
            )
        }
        .into_any(),
    );

    // Request ID (if available)
    if let Some(ref request_id) = msg.request_id {
        metadata_elements.push(
            element! {
                Text(
                    content: format!("  Request ID: {}", request_id),
                    color: Color::Green,
                )
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
            margin_bottom: 1,
        ) {
            #(metadata_elements.into_iter())
        }
    }
    .into_any()
}

/// Render payload section with formatted JSON
fn render_payload_section(msg: &NatsMessage) -> AnyElement<'static> {
    let mut payload_elements: Vec<AnyElement> = Vec::new();

    payload_elements.push(
        element! {
            Text(
                content: "Payload",
                weight: Weight::Bold,
                color: Color::Cyan,
            )
        }
        .into_any(),
    );

    // Try to format as JSON, otherwise show raw
    let formatted_payload = format_json_payload(&msg.payload);

    // Split into lines and render each
    for line in formatted_payload.lines().take(30) {
        // Limit to 30 lines
        payload_elements.push(
            element! {
                Text(
                    content: format!("  {}", line),
                    color: Color::White,
                )
            }
            .into_any(),
        );
    }

    // Show truncation warning if payload is very long
    if formatted_payload.lines().count() > 30 {
        payload_elements.push(
            element! {
                Text(
                    content: "  ... (payload truncated, showing first 30 lines)",
                    color: Color::Yellow,
                )
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
        ) {
            #(payload_elements.into_iter())
        }
    }
    .into_any()
}

/// Render "no selection" message
fn render_no_selection() -> AnyElement<'static> {
    element! {
        View(
            border_style: BorderStyle::Single,
            border_color: Color::Grey,
            padding: 2,
        ) {
            Text(
                content: "No message selected",
                color: Color::Grey,
            )
        }
    }
    .into_any()
}

/// Format timestamp for display
fn format_timestamp(timestamp: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()
    } else {
        timestamp.to_string()
    }
}

/// Format JSON payload with indentation
fn format_json_payload(payload: &str) -> String {
    // Try to parse and pretty-print JSON
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(payload) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
            return pretty;
        }
    }

    // Fall back to raw payload if not valid JSON
    payload.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detail_view_props_default() {
        let props = DetailViewProps::default();
        assert!(!props.monitor_ctx.show_detail());
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = "2025-11-02T10:30:45.123Z";
        let formatted = format_timestamp(timestamp);
        assert!(formatted.contains("2025-11-02"));
        assert!(formatted.contains("10:30:45"));
    }

    #[test]
    fn test_format_json_payload() {
        // Valid JSON
        let json = r#"{"key":"value","number":42}"#;
        let formatted = format_json_payload(json);
        assert!(formatted.contains("key"));
        assert!(formatted.contains("value"));
        assert!(formatted.len() > json.len()); // Should be pretty-printed

        // Invalid JSON - should return as-is
        let invalid = "not json at all";
        let formatted = format_json_payload(invalid);
        assert_eq!(formatted, invalid);
    }

    #[test]
    fn test_render_message_detail() {
        let msg = NatsMessage {
            timestamp: "2025-11-02T10:30:45Z".to_string(),
            subject: "mcp.orchestrator.request".to_string(),
            endpoint: "orchestrator".to_string(),
            message_type: "Request".to_string(),
            payload: r#"{"tool":"test","params":{}}"#.to_string(),
            request_id: Some("req-123".to_string()),
        };

        // Just verify it doesn't panic
        let _detail = render_message_detail(&msg);
    }
}
