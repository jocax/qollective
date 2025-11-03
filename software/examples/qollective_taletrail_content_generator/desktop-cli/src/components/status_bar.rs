use iocraft::prelude::*;

use crate::layout::{LayoutConfig, LayoutMode};
use crate::state::{AppContext, ThemeMode};
use crate::environment::Environment;

/// Status bar component properties
#[derive(Props)]
pub struct StatusBarProps {
    pub help_hint: Option<String>,
    pub nats_connected: bool,
    pub active_requests: usize,
    pub view_name: String,
    pub layout_config: LayoutConfig,
    pub app_context: Option<AppContext>,
    pub manual_display_mode: Option<LayoutMode>,
}

impl Default for StatusBarProps {
    fn default() -> Self {
        Self {
            help_hint: None,
            nats_connected: false,
            active_requests: 0,
            view_name: String::from("Unknown"),
            layout_config: LayoutConfig::default(),
            app_context: None,
            manual_display_mode: None,
        }
    }
}

/// Status bar component displayed at the bottom of the screen
#[component]
pub fn StatusBar(_hooks: Hooks, props: &StatusBarProps) -> impl Into<AnyElement<'static>> {
    let view_name = &props.view_name;
    let nats_connected = props.nats_connected;
    let active_requests = props.active_requests;
    let layout = props.layout_config;

    let nats_indicator = if nats_connected {
        "● NATS Connected"
    } else {
        "○ NATS Disconnected"
    };
    let nats_color = if nats_connected {
        Color::Green
    } else {
        Color::Red
    };

    let requests_text = format!("Active: {}", active_requests);

    let help_text = props
        .help_hint
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Ctrl+H: Help | Ctrl+Q: Quit");

    // Get environment and mode info if app_context is available
    let environment_badge = if let Some(ctx) = &props.app_context {
        let env = ctx.environment();
        let env_text = match env {
            Environment::ITerm2 => "[iTerm2]",
            Environment::RustRover => "[IDE]",
            Environment::Unknown => "[Unknown]",
        };

        let mode_badge = if let Some(manual_mode) = props.manual_display_mode {
            format!(" [Manual: {}]", match manual_mode {
                LayoutMode::Classic => "Classic",
                LayoutMode::Modern => "Modern",
                LayoutMode::FullHD => "FullHD",
                LayoutMode::FourK => "4K",
            })
        } else {
            String::new()
        };

        let debug_badge = if ctx.is_debug_mode() {
            " [DEBUG MODE: ON]"
        } else {
            ""
        };

        let theme_badge = match ctx.theme_mode() {
            ThemeMode::Dark => " [DARK]",
            ThemeMode::Light => " [LIGHT]",
        };

        format!("{}{}{}{}", env_text, mode_badge, debug_badge, theme_badge)
    } else {
        String::new()
    };

    // Determine status bar border color based on debug mode
    let status_border_color = if let Some(ctx) = &props.app_context {
        if ctx.is_debug_mode() {
            Color::Yellow
        } else {
            Color::Green
        }
    } else {
        Color::Green
    };

    let status_border_color_compact = if let Some(ctx) = &props.app_context {
        if ctx.is_debug_mode() {
            Color::Yellow
        } else {
            Color::Grey
        }
    } else {
        Color::Grey
    };

    // Desktop mode: Rich multi-row status (7 rows)
    if layout.layout_mode == LayoutMode::FourK {
        element! {
            View(
                border_style: BorderStyle::Round,
                border_color: status_border_color,
            ) {
                // Row 1: View and NATS status
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    View(margin_right: 2) {
                        Text(
                            content: format!(" View: {} ", view_name),
                            weight: Weight::Bold,
                            color: Color::Cyan,
                        )
                    }
                    View(margin_right: 2) {
                        Text(content: " │ ")
                    }
                    View(margin_right: 2) {
                        Text(
                            content: format!("{} ", nats_indicator),
                            color: nats_color,
                        )
                    }
                    View(margin_right: 2) {
                        Text(content: " │ ")
                    }
                    View {
                        Text(
                            content: format!("{} ", requests_text),
                            color: Color::Yellow,
                        )
                    }
                }

                // Row 2: Metrics and environment info
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    View(margin_right: 2) {
                        Text(content: format!(" Messages: 1,234 │ Trails: 1,247 │ Memory: 45MB │ {} ", environment_badge))
                    }
                }

                // Row 3: Help hints
                View(padding: 0) {
                    Text(
                        content: format!(" [F1:Help] [⇧⌃D:Debug] [⇧⌃1-7:Jump] [⇧⌃T:Theme] [Ctrl+Q:Quit] {} ", help_text),
                        color: Color::Grey,
                    )
                }

                // Rows 4-7: Debug console
                #({
                    let debug_elements: Vec<AnyElement> = if let Some(ctx) = &props.app_context {
                        if ctx.is_debug_console_expanded() {
                            // Expanded debug console
                            let mut elements = vec![
                                element! {
                                    Text(
                                        content: " DEBUG CONSOLE [Press Ctrl+D to collapse] ",
                                        weight: Weight::Bold,
                                        color: Color::Yellow
                                    )
                                }.into_any()
                            ];

                            // Show last 3 log entries
                            for log in ctx.get_debug_logs().iter().rev().take(3) {
                                elements.push(element! {
                                    Text(content: format!(" {} ", log), color: Color::Grey)
                                }.into_any());
                            }

                            vec![element! {
                                View(
                                    border_style: BorderStyle::Single,
                                    border_color: Color::Yellow,
                                    padding: 1,
                                    margin_top: 1
                                ) {
                                    #(elements.into_iter())
                                }
                            }.into_any()]
                        } else {
                            // Collapsed debug console
                            vec![element! {
                                View(padding: 0, margin_top: 1) {
                                    Text(
                                        content: format!(
                                            " [Debug Console: Collapsed] Last: {} ",
                                            ctx.get_debug_logs().last().unwrap_or(&"No logs".to_string())
                                        ),
                                        color: Color::Grey
                                    )
                                }
                            }.into_any()]
                        }
                    } else {
                        // No app context - show placeholder
                        vec![element! {
                            View(padding: 0, margin_top: 1) {
                                Text(content: " Press Ctrl+D to toggle debug console ", color: Color::Grey)
                            }
                        }.into_any()]
                    };
                    debug_elements.into_iter()
                })
            }
        }
    }
    // Large mode: Compact status (3 rows)
    else if layout.layout_mode == LayoutMode::FullHD {
        element! {
            View(
                border_style: BorderStyle::Round,
                border_color: status_border_color,
            ) {
                // Line 1: View and NATS status
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    View(margin_right: 2) {
                        Text(
                            content: format!(" View: {} ", view_name),
                            weight: Weight::Bold,
                            color: Color::Cyan,
                        )
                    }
                    View(margin_right: 2) {
                        Text(content: " │ ")
                    }
                    View {
                        Text(
                            content: format!("{} ", nats_indicator),
                            color: nats_color,
                        )
                    }
                }

                // Line 2: Metrics and environment info
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    View(margin_right: 2) {
                        Text(
                            content: format!(" {} ", requests_text),
                            color: Color::Yellow,
                        )
                    }
                    View(margin_right: 2) {
                        Text(content: format!(" │ Trails: 1,247 │ Memory: 45MB │ {} ", environment_badge))
                    }
                }

                // Line 3: Help hints
                View(padding: 0) {
                    Text(
                        content: format!(" [F1:Help] [⇧⌃1-7:Views] [⇧⌃T:Theme] {} ", help_text),
                        color: Color::Grey,
                    )
                }
            }
        }
    }
    // Medium mode: Compact two-line status
    else if layout.layout_mode == LayoutMode::Modern {
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: status_border_color_compact,
            ) {
                // Line 1: Main status info
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    View(margin_right: 2) {
                        Text(
                            content: format!(" {} ", view_name),
                            weight: Weight::Bold,
                            color: Color::Cyan,
                        )
                    }
                    View(margin_right: 2) {
                        Text(content: " │ ")
                    }
                    View(margin_right: 2) {
                        Text(
                            content: format!("{} ", nats_indicator),
                            color: nats_color,
                        )
                    }
                    View(margin_right: 2) {
                        Text(content: " │ ")
                    }
                    View(flex_grow: 1.0) {
                        Text(
                            content: format!("{} ", requests_text),
                            color: Color::Yellow,
                        )
                    }
                }

                // Line 2: Help hints and environment badge
                View(padding: 0) {
                    Text(
                        content: format!(" {} {} ", help_text, if !environment_badge.is_empty() { format!("│ {}", environment_badge) } else { String::new() }),
                        color: Color::Grey,
                    )
                }
            }
        }
    }
    // Small mode: Minimal single line
    else {
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: status_border_color_compact,
            ) {
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                    height: 1u16,
                ) {
                    View(margin_right: 1) {
                        Text(
                            content: format!(" {} ", view_name),
                            color: Color::Cyan,
                        )
                    }
                    View(margin_right: 1) {
                        Text(
                            content: if nats_connected { "●" } else { "○" },
                            color: nats_color,
                        )
                    }
                    View(flex_grow: 1.0) {
                        Text(
                            content: format!(" {} ", active_requests),
                            color: Color::Yellow,
                        )
                    }
                }
            }
        }
    }
}
