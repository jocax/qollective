use iocraft::prelude::*;

use crate::layout::{LayoutConfig, LayoutMode};
use crate::state::View as AppView;

/// Navbar component properties
#[derive(Props)]
pub struct NavbarProps {
    pub current_view: AppView,
    pub layout_config: LayoutConfig,
}

impl Default for NavbarProps {
    fn default() -> Self {
        Self {
            current_view: AppView::McpTester,
            layout_config: LayoutConfig::default(),
        }
    }
}

/// Persistent navigation bar component displayed at the top of the screen
#[component]
pub fn Navbar(_hooks: Hooks, props: &NavbarProps) -> impl Into<AnyElement<'static>> {
    let layout = props.layout_config;
    let current = props.current_view;

    // Desktop mode: Rich multi-line navbar with logo (3 rows)
    if layout.layout_mode == LayoutMode::FourK {
        let logo_line = format!(
            " {} TaleTrail Desktop CLI {} v{} ",
            "🚀",
            " ",
            env!("CARGO_PKG_VERSION")
        );

        let nav_line = format!(
            " {}  {}  {}  {}  {}  {}  {} ",
            nav_item(1, "MCP Tester", current == AppView::McpTester),
            nav_item(2, "Trail Viewer", current == AppView::TrailViewer),
            nav_item(3, "NATS Monitor", current == AppView::NatsMonitor),
            nav_item(4, "Story Gen", current == AppView::StoryGenerator),
            nav_item(5, "Search", current == AppView::Search),
            nav_item(6, "Settings", current == AppView::Settings),
            nav_item(7, "Logs", current == AppView::Logs)
        );

        element! {
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Cyan,
            ) {
                // Line 1: Logo + title + version
                View(padding: 0) {
                    Text(
                        content: logo_line,
                        weight: Weight::Bold,
                        color: Color::Cyan,
                    )
                }

                // Line 2: Navigation tabs
                View(padding: 0) {
                    Text(content: nav_line)
                }
            }
        }
    }
    // Large mode: Rich multi-line navbar with logo (3 rows)
    else if layout.layout_mode == LayoutMode::FullHD {
        let logo_line = format!(
            " {} TaleTrail Desktop CLI {} v{} ",
            "🚀",
            " ",
            env!("CARGO_PKG_VERSION")
        );

        let nav_line = format!(
            " {}  {}  {}  {}  {}  {}  {} ",
            nav_item(1, "MCP Tester", current == AppView::McpTester),
            nav_item(2, "Trail Viewer", current == AppView::TrailViewer),
            nav_item(3, "NATS Monitor", current == AppView::NatsMonitor),
            nav_item(4, "Story Gen", current == AppView::StoryGenerator),
            nav_item(5, "Search", current == AppView::Search),
            nav_item(6, "Settings", current == AppView::Settings),
            nav_item(7, "Logs", current == AppView::Logs)
        );

        element! {
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Cyan,
            ) {
                // Line 1: Logo + title + version
                View(padding: 0) {
                    Text(
                        content: logo_line,
                        weight: Weight::Bold,
                        color: Color::Cyan,
                    )
                }

                // Line 2: Navigation tabs
                View(padding: 0) {
                    Text(content: nav_line)
                }
            }
        }
    }
    // Medium mode: Compact single-line navbar (2 rows)
    else if layout.layout_mode == LayoutMode::Modern {
        let nav_line = format!(
            " [1:MCP] [2:Trails] [3:Monitor] [4:Gen] [5:Search] [6:Settings] [7:Logs] ",
        );

        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Blue,
            ) {
                Text(content: nav_line)
            }
        }
    }
    // Small mode: No navbar (hidden)
    else {
        element! {
            View(height: 0u16)
        }
    }
}

/// Format a navigation item with active/inactive styling
fn nav_item(num: usize, title: &str, active: bool) -> String {
    if active {
        format!("[{}:{}]", num, title) // Active: with brackets
    } else {
        format!(" {}:{} ", num, title) // Inactive: with spaces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navbar_props_creation() {
        let props = NavbarProps {
            current_view: AppView::McpTester,
            layout_config: LayoutConfig::from_terminal_size(200, 60),
        };
        assert_eq!(props.current_view, AppView::McpTester);
        assert_eq!(props.layout_config.layout_mode, LayoutMode::FullHD);
    }

    #[test]
    fn test_navbar_props_default() {
        let props = NavbarProps::default();
        assert_eq!(props.current_view, AppView::McpTester);
    }

    #[test]
    fn test_nav_item_active() {
        let active = nav_item(1, "Test", true);
        assert_eq!(active, "[1:Test]");
    }

    #[test]
    fn test_nav_item_inactive() {
        let inactive = nav_item(1, "Test", false);
        assert_eq!(inactive, " 1:Test ");
    }
}
