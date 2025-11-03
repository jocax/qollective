/// Message Feed View for NATS Monitoring
///
/// Displays a scrollable list of NATS messages with auto-scroll support

use crate::components::List;
use crate::nats::monitoring::NatsMessage;
use crate::state::MonitorContext;
use iocraft::prelude::*;

/// Props for MessageFeed component
#[derive(Props)]
pub struct MessageFeedProps {
    pub monitor_ctx: MonitorContext,
}

impl Default for MessageFeedProps {
    fn default() -> Self {
        Self {
            monitor_ctx: MonitorContext::new(),
        }
    }
}

/// Message feed component showing real-time NATS messages
#[component]
pub fn MessageFeed(_hooks: Hooks, props: &MessageFeedProps) -> impl Into<AnyElement<'static>> {
    let monitor_ctx = &props.monitor_ctx;

    // Get filtered messages and selection state
    let messages = monitor_ctx.filtered_messages();
    let selected_index = monitor_ctx.selected_index();
    let auto_scroll = monitor_ctx.auto_scroll();
    let total_count = monitor_ctx.message_count();
    let filtered_count = messages.len();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header with status
    let header_text = if filtered_count < total_count {
        format!(
            "NATS Message Feed (Showing {} of {} messages){}",
            filtered_count,
            total_count,
            if auto_scroll { " [AUTO-SCROLL]" } else { "" }
        )
    } else {
        format!(
            "NATS Message Feed ({} messages){}",
            total_count,
            if auto_scroll { " [AUTO-SCROLL]" } else { "" }
        )
    };

    elements.push(
        element! {
            View(margin_bottom: 1) {
                Text(
                    content: header_text,
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // Message list
    if messages.is_empty() {
        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Grey,
                    padding: 2,
                    height: 25u16,
                ) {
                    Text(
                        content: "No messages to display",
                        color: Color::Grey,
                    )
                }
            }
            .into_any(),
        );
    } else {
        let render_fn: fn(&NatsMessage, bool) -> String = render_message_item;
        elements.push(
            element! {
                List::<NatsMessage>(
                    items: messages.clone(),
                    selected_index: selected_index,
                    visible_rows: 25usize,
                    render_item: Some(render_fn),
                    show_pagination: true,
                )
            }
            .into_any(),
        );
    }

    // Footer with shortcuts
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "Up/Down: Navigate | Enter: View details | A: Toggle auto-scroll | F: Filters | C: Clear",
                    color: Color::Grey,
                )
            }
        }
        .into_any(),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Render a single message item in the list
fn render_message_item(msg: &NatsMessage, selected: bool) -> String {
    // Color prefix based on subject
    let prefix = if msg.subject.starts_with("mcp.") {
        "[MCP]"
    } else if msg.subject.starts_with("taletrail.") {
        "[TT] "
    } else {
        "[???]"
    };

    // Parse timestamp and format as HH:MM:SS
    let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&msg.timestamp) {
        dt.format("%H:%M:%S").to_string()
    } else {
        msg.timestamp.chars().take(8).collect()
    };

    // Truncate payload for preview
    let payload_preview: String = msg.payload.chars().take(80).collect();
    let payload_preview = payload_preview.replace('\n', " ").replace('\r', "");

    // Format: [PREFIX] HH:MM:SS | endpoint | subject | payload preview
    let indicator = if selected { ">" } else { " " };

    format!(
        "{} {} {} | {:12} | {:30} | {}",
        indicator,
        prefix,
        time_str,
        truncate_str(&msg.endpoint, 12),
        truncate_str(&msg.subject, 30),
        payload_preview
    )
}

/// Helper to truncate string and add ellipsis if needed
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_message_item() {
        let msg = NatsMessage {
            timestamp: "2025-11-02T10:30:45Z".to_string(),
            subject: "mcp.orchestrator.request".to_string(),
            endpoint: "orchestrator".to_string(),
            message_type: "Request".to_string(),
            payload: r#"{"tool": "test_tool", "params": {}}"#.to_string(),
            request_id: Some("req-123".to_string()),
        };

        let rendered = render_message_item(&msg, false);
        assert!(rendered.contains("[MCP]"));
        assert!(rendered.contains("orchestrator"));
        assert!(rendered.contains("mcp.orchestrator.request"));

        let rendered_selected = render_message_item(&msg, true);
        assert!(rendered_selected.starts_with(">"));
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short     ");
        assert_eq!(truncate_str("this is a very long string", 10), "this is...");
        assert_eq!(truncate_str("exactly10c", 10), "exactly10c");
    }

    #[test]
    fn test_message_feed_props_default() {
        let props = MessageFeedProps::default();
        assert_eq!(props.monitor_ctx.message_count(), 0);
    }
}
