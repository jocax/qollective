/// Filter Panel for NATS Monitoring
///
/// Provides UI controls for filtering messages by endpoint, type, and text search

use crate::state::{MessageFilters, MonitorContext};
use iocraft::prelude::*;

/// Props for FilterPanel component
#[derive(Props)]
pub struct FilterPanelProps {
    pub monitor_ctx: MonitorContext,
}

impl Default for FilterPanelProps {
    fn default() -> Self {
        Self {
            monitor_ctx: MonitorContext::new(),
        }
    }
}

/// Filter panel showing current filters and available options
#[component]
pub fn FilterPanel(_hooks: Hooks, props: &FilterPanelProps) -> impl Into<AnyElement<'static>> {
    let monitor_ctx = &props.monitor_ctx;
    let filters = monitor_ctx.filters();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(margin_bottom: 1) {
                Text(
                    content: "Message Filters",
                    color: Color::Yellow,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // Filter status
    let filter_desc = filters.description();
    let filter_color = if filters.is_active() {
        Color::Green
    } else {
        Color::Grey
    };

    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: filter_color,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: format!("Active Filters: {}", filter_desc),
                    color: filter_color,
                )
            }
        }
        .into_any(),
    );

    // Endpoint filter section
    elements.push(render_endpoint_section(&filters));

    // Message type filter section
    elements.push(render_message_type_section(&filters));

    // Search query section
    elements.push(render_search_section(&filters));

    // Footer with shortcuts
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "1-5: Filter by endpoint | R/E/V: Filter by type | S: Search | X: Clear filters",
                    color: Color::Grey,
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

/// Render endpoint filter section
fn render_endpoint_section(filters: &MessageFilters) -> AnyElement<'static> {
    let mut endpoint_elements: Vec<AnyElement> = Vec::new();

    endpoint_elements.push(
        element! {
            Text(
                content: "Endpoint Filter:",
                weight: Weight::Bold,
                color: Color::Cyan,
            )
        }
        .into_any(),
    );

    let endpoints = vec![
        ("1", "orchestrator"),
        ("2", "story-generator"),
        ("3", "quality-control"),
        ("4", "constraint-enforcer"),
        ("5", "prompt-helper"),
    ];

    for (key, endpoint) in endpoints {
        let is_active = filters.endpoint.as_deref() == Some(endpoint);
        let indicator = if is_active { "[X]" } else { "[ ]" };
        let color = if is_active {
            Color::Green
        } else {
            Color::White
        };

        endpoint_elements.push(
            element! {
                Text(
                    content: format!("  {} {}: {}", key, indicator, endpoint),
                    color: color,
                )
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_bottom: 1,
        ) {
            #(endpoint_elements.into_iter())
        }
    }
    .into_any()
}

/// Render message type filter section
fn render_message_type_section(filters: &MessageFilters) -> AnyElement<'static> {
    let mut type_elements: Vec<AnyElement> = Vec::new();

    type_elements.push(
        element! {
            Text(
                content: "Message Type Filter:",
                weight: Weight::Bold,
                color: Color::Cyan,
            )
        }
        .into_any(),
    );

    let types = vec![("R", "Request"), ("E", "Event"), ("V", "Response")];

    for (key, msg_type) in types {
        let is_active = filters.message_type.as_deref() == Some(msg_type);
        let indicator = if is_active { "[X]" } else { "[ ]" };
        let color = if is_active {
            Color::Green
        } else {
            Color::White
        };

        type_elements.push(
            element! {
                Text(
                    content: format!("  {} {}: {}", key, indicator, msg_type),
                    color: color,
                )
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_bottom: 1,
        ) {
            #(type_elements.into_iter())
        }
    }
    .into_any()
}

/// Render search query section
fn render_search_section(filters: &MessageFilters) -> AnyElement<'static> {
    let search_text = if let Some(ref query) = filters.search_query {
        format!("Search: {}", query)
    } else {
        "Search: (none)".to_string()
    };

    let color = if filters.search_query.is_some() {
        Color::Green
    } else {
        Color::Grey
    };

    element! {
        View(
            border_style: BorderStyle::Single,
            border_color: color,
            padding: 1,
        ) {
            Text(content: search_text, color: color)
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_panel_props_default() {
        let props = FilterPanelProps::default();
        let filters = props.monitor_ctx.filters();
        assert!(!filters.is_active());
    }

    #[test]
    fn test_message_filters_description() {
        let filters = MessageFilters::default();
        assert_eq!(filters.description(), "No filters");

        let filters = MessageFilters {
            endpoint: Some("orchestrator".to_string()),
            message_type: None,
            search_query: None,
        };
        assert_eq!(filters.description(), "Endpoint: orchestrator");

        let filters = MessageFilters {
            endpoint: Some("orchestrator".to_string()),
            message_type: Some("Request".to_string()),
            search_query: Some("test".to_string()),
        };
        assert_eq!(
            filters.description(),
            "Endpoint: orchestrator | Type: Request | Search: test"
        );
    }
}
