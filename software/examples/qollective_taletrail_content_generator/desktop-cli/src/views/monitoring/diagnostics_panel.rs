/// Connection Diagnostics Panel for NATS Monitoring
///
/// Displays connection status, message statistics, and health indicators

use crate::nats::monitoring::MonitoringDiagnostics;
use crate::state::MonitorContext;
use iocraft::prelude::*;

/// Props for DiagnosticsPanel component
#[derive(Props)]
pub struct DiagnosticsPanelProps {
    pub monitor_ctx: MonitorContext,
}

impl Default for DiagnosticsPanelProps {
    fn default() -> Self {
        Self {
            monitor_ctx: MonitorContext::new(),
        }
    }
}

/// Diagnostics panel showing connection health and statistics
#[component]
pub fn DiagnosticsPanel(
    _hooks: Hooks,
    props: &DiagnosticsPanelProps,
) -> impl Into<AnyElement<'static>> {
    let monitor_ctx = &props.monitor_ctx;
    let diagnostics = monitor_ctx.diagnostics();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(margin_bottom: 1) {
                Text(
                    content: "Connection Diagnostics",
                    color: Color::Magenta,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // Connection status indicator
    elements.push(render_connection_status(&diagnostics));

    // 2x2 Grid layout for metrics
    elements.push(render_metrics_grid(&diagnostics));

    // Timestamps row
    elements.push(render_timestamps(&diagnostics));

    // Footer with shortcuts
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "N: Reconnect | D: Refresh diagnostics",
                    color: Color::DarkGrey,
                )
            }
        }
        .into_any(),
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
            padding: 1,
        ) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Render connection status indicator
fn render_connection_status(diagnostics: &MonitoringDiagnostics) -> AnyElement<'static> {
    let (status_text, status_color, status_icon) = if diagnostics.is_connected {
        ("Connected", Color::Green, "●")
    } else {
        ("Disconnected", Color::Red, "○")
    };

    element! {
        View(
            border_style: BorderStyle::Double,
            border_color: status_color,
            padding: 1,
            margin_bottom: 1,
        ) {
            Text(
                content: format!("{} Status: {}", status_icon, status_text),
                color: status_color,
                weight: Weight::Bold,
            )
        }
    }
    .into_any()
}

/// Render metrics in a 2x2 grid layout
/// Grid structure:
///   Row 1: [Messages Received] [Messages Displayed]
///   Row 2: [Emission Failures]  [Message Rate]
fn render_metrics_grid(diagnostics: &MonitoringDiagnostics) -> AnyElement<'static> {
    let rate = calculate_message_rate(diagnostics);

    // Calculate failures (difference between received and buffered)
    let failures = diagnostics.messages_received.saturating_sub(diagnostics.messages_buffered);

    element! {
        View(
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
            margin_bottom: 1,
        ) {
            Text(
                content: "Message Statistics",
                weight: Weight::Bold,
                color: Color::Cyan,
            )

            // Row 1: Received | Displayed
            View(
                flex_direction: FlexDirection::Row,
                margin_top: 1,
            ) {
                // Column 1: Messages Received
                View(
                    flex_grow: 1.0,
                    border_style: BorderStyle::Single,
                    border_color: Color::DarkGrey,
                    padding: 1,
                    margin_right: 1,
                ) {
                    Text(
                        content: "Messages Received",
                        color: Color::Yellow,
                        weight: Weight::Bold,
                    )
                    Text(
                        content: format!("{}", diagnostics.messages_received),
                        color: Color::White,
                        weight: Weight::Bold,
                    )
                }

                // Column 2: Messages Displayed
                View(
                    flex_grow: 1.0,
                    border_style: BorderStyle::Single,
                    border_color: Color::DarkGrey,
                    padding: 1,
                ) {
                    Text(
                        content: "Messages Displayed",
                        color: Color::Yellow,
                        weight: Weight::Bold,
                    )
                    Text(
                        content: format!("{}", diagnostics.messages_buffered),
                        color: Color::White,
                        weight: Weight::Bold,
                    )
                }
            }

            // Row 2: Failures | Rate
            View(
                flex_direction: FlexDirection::Row,
                margin_top: 1,
            ) {
                // Column 1: Emission Failures
                View(
                    flex_grow: 1.0,
                    border_style: BorderStyle::Single,
                    border_color: Color::DarkGrey,
                    padding: 1,
                    margin_right: 1,
                ) {
                    Text(
                        content: "Emission Failures",
                        color: Color::Yellow,
                        weight: Weight::Bold,
                    )
                    Text(
                        content: format!("{}", failures),
                        color: if failures > 0 { Color::Red } else { Color::Green },
                        weight: Weight::Bold,
                    )
                }

                // Column 2: Message Rate
                View(
                    flex_grow: 1.0,
                    border_style: BorderStyle::Single,
                    border_color: Color::DarkGrey,
                    padding: 1,
                ) {
                    Text(
                        content: "Message Rate",
                        color: Color::Yellow,
                        weight: Weight::Bold,
                    )
                    Text(
                        content: format!("{:.1}/s", rate),
                        color: Color::Green,
                        weight: Weight::Bold,
                    )
                }
            }
        }
    }
    .into_any()
}

/// Render timestamps
fn render_timestamps(diagnostics: &MonitoringDiagnostics) -> AnyElement<'static> {
    let mut timestamp_elements: Vec<AnyElement> = Vec::new();

    timestamp_elements.push(
        element! {
            Text(
                content: "Timestamps:",
                weight: Weight::Bold,
                color: Color::Cyan,
            )
        }
        .into_any(),
    );

    // Connection timestamp
    let conn_time = format_timestamp(&diagnostics.connection_timestamp);
    timestamp_elements.push(
        element! {
            Text(
                content: format!("  Connected At: {}", conn_time),
                color: Color::White,
            )
        }
        .into_any(),
    );

    // Last message timestamp
    let last_msg = if let Some(ref timestamp) = diagnostics.last_message_timestamp {
        format_timestamp(timestamp)
    } else {
        "No messages yet".to_string()
    };

    timestamp_elements.push(
        element! {
            Text(
                content: format!("  Last Message: {}", last_msg),
                color: Color::White,
            )
        }
        .into_any(),
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
        ) {
            #(timestamp_elements.into_iter())
        }
    }
    .into_any()
}

/// Calculate message rate (simplified)
/// In a real implementation, this would track messages over a time window
fn calculate_message_rate(diagnostics: &MonitoringDiagnostics) -> f64 {
    // Parse connection timestamp
    if let Ok(conn_time) = chrono::DateTime::parse_from_rfc3339(&diagnostics.connection_timestamp)
    {
        if let Some(ref last_msg_str) = diagnostics.last_message_timestamp {
            if let Ok(last_msg_time) = chrono::DateTime::parse_from_rfc3339(last_msg_str) {
                let duration = last_msg_time.signed_duration_since(conn_time);
                let seconds = duration.num_seconds() as f64;

                if seconds > 0.0 {
                    return diagnostics.messages_received as f64 / seconds;
                }
            }
        }
    }

    0.0
}

/// Format timestamp for display (convert ISO 8601 to human-readable)
fn format_timestamp(timestamp: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        timestamp.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_panel_props_default() {
        let props = DiagnosticsPanelProps::default();
        let diagnostics = props.monitor_ctx.diagnostics();
        assert!(diagnostics.is_connected);
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = "2025-11-02T10:30:45Z";
        let formatted = format_timestamp(timestamp);
        assert!(formatted.contains("2025-11-02"));
        assert!(formatted.contains("10:30:45"));
    }

    #[test]
    fn test_calculate_message_rate() {
        let mut diagnostics = MonitoringDiagnostics::new();
        diagnostics.messages_received = 0;

        let rate = calculate_message_rate(&diagnostics);
        assert_eq!(rate, 0.0);

        // Test with actual timestamps
        let conn_time = chrono::Utc::now();
        let last_msg_time = conn_time + chrono::Duration::seconds(10);

        diagnostics.connection_timestamp = conn_time.to_rfc3339();
        diagnostics.last_message_timestamp = Some(last_msg_time.to_rfc3339());
        diagnostics.messages_received = 100;

        let rate = calculate_message_rate(&diagnostics);
        assert!(rate > 9.0 && rate < 11.0); // ~10 messages per second
    }
}
