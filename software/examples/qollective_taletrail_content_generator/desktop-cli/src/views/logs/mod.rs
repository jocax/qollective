/// Logs View - Display application logs with filtering and search
use iocraft::prelude::*;

use crate::layout::LayoutConfig;
use crate::state::AppContext;

/// Props for the Logs view
#[derive(Props)]
pub struct LogsViewProps {
    pub layout_config: LayoutConfig,
    pub app_context: AppContext,
}

impl Default for LogsViewProps {
    fn default() -> Self {
        Self {
            layout_config: LayoutConfig::default(),
            app_context: AppContext::new(),
        }
    }
}

/// Logs view component showing application debug logs
#[component]
pub fn LogsView(_hooks: Hooks, props: &LogsViewProps) -> impl Into<AnyElement<'static>> {
    let logs = props.app_context.get_debug_logs();
    let layout = props.layout_config;
    let debug_mode = props.app_context.is_debug_mode();

    // Calculate how many logs we can display
    let visible_rows = layout.list_visible_rows();
    let logs_to_show = logs.len().min(visible_rows);

    // Take the most recent logs (from the end)
    let recent_logs: Vec<String> = logs.iter()
        .rev()  // Reverse to show most recent first
        .take(logs_to_show)
        .cloned()
        .collect();

    // Prepare log display elements
    let log_elements: Vec<AnyElement> = if recent_logs.is_empty() {
        vec![element! {
            Text(
                content: "No logs yet. Debug logs will appear here.",
                color: Color::Grey
            )
        }.into()]
    } else {
        recent_logs.iter().map(|log_entry| {
            let text_color = if log_entry.contains("ERROR") {
                Color::Red
            } else if log_entry.contains("WARN") {
                Color::Yellow
            } else if log_entry.contains("INFO") {
                Color::Green
            } else {
                Color::White
            };

            element! {
                Text(content: log_entry.clone(), color: text_color)
            }.into()
        }).collect()
    };

    element! {
        View(flex_direction: FlexDirection::Column) {
            // Header
            View(border_style: BorderStyle::Single, border_color: Color::Cyan) {
                Text(
                    content: format!("Application Logs ({})", logs.len()),
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
                Text(
                    content: format!("Debug Mode: {} | Showing {} most recent",
                        if debug_mode { "ON" } else { "OFF" },
                        logs_to_show
                    )
                )
            }

            // Controls hint
            Text(
                content: "[F12: Toggle Debug Mode] [C: Clear Logs] [ESC: Back]",
                color: Color::Grey
            )

            // Logs display area
            View(
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Single
            ) {
                #(log_elements)
            }

            // Footer stats
            View(border_style: BorderStyle::Single, border_color: Color::Blue) {
                Text(
                    content: format!(
                        "Total logs: {} | Layout: {} ({}×{}) | Environment: {}",
                        logs.len(),
                        layout.layout_mode.display_name(),
                        layout.width,
                        layout.height,
                        props.app_context.environment().name()
                    ),
                    color: Color::Blue
                )
            }
        }
    }
}
