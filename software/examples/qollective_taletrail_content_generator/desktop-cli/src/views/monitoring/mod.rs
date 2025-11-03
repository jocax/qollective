/// NATS Monitoring Interface
///
/// Main view for real-time NATS message monitoring with filtering and diagnostics

pub mod detail_view;
pub mod diagnostics_panel;
pub mod filter_panel;
pub mod message_feed;

#[cfg(test)]
mod tests;

use crate::state::MonitorContext;
use detail_view::DetailView;
use diagnostics_panel::DiagnosticsPanel;
use filter_panel::FilterPanel;
use iocraft::prelude::*;
use message_feed::MessageFeed;

/// Represents the different views in the NATS Monitoring Interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorView {
    Feed,
    Filters,
    Diagnostics,
}

impl MonitorView {
    /// Get the display name for the view
    pub fn display_name(&self) -> &'static str {
        match self {
            MonitorView::Feed => "Message Feed",
            MonitorView::Filters => "Filters",
            MonitorView::Diagnostics => "Diagnostics",
        }
    }

    /// Get the view number (1-3)
    pub fn view_number(&self) -> usize {
        match self {
            MonitorView::Feed => 1,
            MonitorView::Filters => 2,
            MonitorView::Diagnostics => 3,
        }
    }

    /// Get view from number (1-3)
    pub fn from_number(number: usize) -> Option<Self> {
        match number {
            1 => Some(MonitorView::Feed),
            2 => Some(MonitorView::Filters),
            3 => Some(MonitorView::Diagnostics),
            _ => None,
        }
    }

    /// Get next view in order
    pub fn next(&self) -> Self {
        match self {
            MonitorView::Feed => MonitorView::Filters,
            MonitorView::Filters => MonitorView::Diagnostics,
            MonitorView::Diagnostics => MonitorView::Feed,
        }
    }

    /// Get previous view in order
    pub fn previous(&self) -> Self {
        match self {
            MonitorView::Feed => MonitorView::Diagnostics,
            MonitorView::Filters => MonitorView::Feed,
            MonitorView::Diagnostics => MonitorView::Filters,
        }
    }
}

/// Props for Monitoring view
#[derive(Props)]
pub struct MonitoringProps {
    pub monitor_ctx: MonitorContext,
    pub active_view: MonitorView,
}

impl Default for MonitoringProps {
    fn default() -> Self {
        Self {
            monitor_ctx: MonitorContext::new(),
            active_view: MonitorView::Feed,
        }
    }
}

/// Main NATS Monitoring view with sub-view navigation
#[component]
pub fn Monitoring(_hooks: Hooks, props: &MonitoringProps) -> impl Into<AnyElement<'static>> {
    let monitor_ctx = &props.monitor_ctx;
    let active_view = props.active_view;

    // Check if detail view should override everything
    let show_detail = monitor_ctx.show_detail();

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
                    content: "NATS Monitoring Interface",
                    color: Color::Magenta,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // If detail view is active, show only detail view
    let main_content = match show_detail {
        true => {
            element! {
                DetailView(monitor_ctx: monitor_ctx.clone())
            }
            .into_any()
        }
        false => {
            // Show view navigation bar and active view
            let mut view_elements: Vec<AnyElement> = Vec::new();

            // View navigation bar
            view_elements.push(render_view_bar(active_view).into());

            // Active view content
            let view_content = match active_view {
                MonitorView::Feed => {
                    element! {
                        MessageFeed(monitor_ctx: monitor_ctx.clone())
                    }
                    .into_any()
                }
                MonitorView::Filters => {
                    element! {
                        FilterPanel(monitor_ctx: monitor_ctx.clone())
                    }
                    .into_any()
                }
                MonitorView::Diagnostics => {
                    element! {
                        DiagnosticsPanel(monitor_ctx: monitor_ctx.clone())
                    }
                    .into_any()
                }
            };

            view_elements.push(view_content);

            element! {
                View(flex_direction: FlexDirection::Column) {
                    #(view_elements.into_iter())
                }
            }
            .into_any()
        }
    };

    elements.push(main_content);

    // Footer with global shortcuts (only when not in detail view)
    if !show_detail {
        elements.push(
            element! {
                View(margin_top: 1) {
                    Text(
                        content: "Tab/Shift+Tab: Switch views | 1-3: Direct view | Esc: Back to menu | Ctrl+H: Help",
                        color: Color::Grey,
                    )
                }
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            padding: 2,
        ) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Render view navigation bar
fn render_view_bar(active_view: MonitorView) -> impl Into<AnyElement<'static>> {
    let views = [
        MonitorView::Feed,
        MonitorView::Filters,
        MonitorView::Diagnostics,
    ];

    let mut view_elements: Vec<AnyElement> = Vec::new();

    for (idx, view) in views.iter().enumerate() {
        let is_active = *view == active_view;

        let label = format!("[{}] {}", view.view_number(), view.display_name());

        let text_color = if is_active {
            Color::Black
        } else {
            Color::White
        };

        let bg_color = if is_active {
            Some(Color::Magenta)
        } else {
            None
        };

        let border_color = if is_active {
            Color::Magenta
        } else {
            Color::Grey
        };

        view_elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: border_color,
                    background_color: bg_color,
                    padding: 1,
                    margin_right: if idx < views.len() - 1 { 1 } else { 0 },
                ) {
                    Text(content: label, color: text_color, weight: Weight::Bold)
                }
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Row,
            margin_bottom: 1,
        ) {
            #(view_elements.into_iter())
        }
    }
}

#[cfg(test)]
mod monitor_view_tests {
    use super::*;

    #[test]
    fn test_monitor_view_display_names() {
        assert_eq!(MonitorView::Feed.display_name(), "Message Feed");
        assert_eq!(MonitorView::Filters.display_name(), "Filters");
        assert_eq!(MonitorView::Diagnostics.display_name(), "Diagnostics");
    }

    #[test]
    fn test_monitor_view_numbers() {
        assert_eq!(MonitorView::Feed.view_number(), 1);
        assert_eq!(MonitorView::Filters.view_number(), 2);
        assert_eq!(MonitorView::Diagnostics.view_number(), 3);
    }

    #[test]
    fn test_monitor_view_from_number() {
        assert_eq!(MonitorView::from_number(1), Some(MonitorView::Feed));
        assert_eq!(MonitorView::from_number(2), Some(MonitorView::Filters));
        assert_eq!(MonitorView::from_number(3), Some(MonitorView::Diagnostics));
        assert_eq!(MonitorView::from_number(0), None);
        assert_eq!(MonitorView::from_number(4), None);
    }

    #[test]
    fn test_monitor_view_navigation() {
        assert_eq!(MonitorView::Feed.next(), MonitorView::Filters);
        assert_eq!(MonitorView::Filters.next(), MonitorView::Diagnostics);
        assert_eq!(MonitorView::Diagnostics.next(), MonitorView::Feed);

        assert_eq!(MonitorView::Feed.previous(), MonitorView::Diagnostics);
        assert_eq!(MonitorView::Filters.previous(), MonitorView::Feed);
        assert_eq!(MonitorView::Diagnostics.previous(), MonitorView::Filters);
    }
}
