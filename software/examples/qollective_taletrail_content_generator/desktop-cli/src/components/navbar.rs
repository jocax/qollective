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

        // Individual nav items with colors
        let nav1_content = if current == AppView::McpTester { "[1:MCP Tester]" } else { " 1:MCP Tester " };
        let nav1_color = if current == AppView::McpTester { Color::Cyan } else { Color::White };
        let nav1_weight = if current == AppView::McpTester { Weight::Bold } else { Weight::Normal };

        let nav2_content = if current == AppView::TrailViewer { "[2:Trail Viewer]" } else { " 2:Trail Viewer " };
        let nav2_color = if current == AppView::TrailViewer { Color::Cyan } else { Color::White };
        let nav2_weight = if current == AppView::TrailViewer { Weight::Bold } else { Weight::Normal };

        let nav3_content = if current == AppView::NatsMonitor { "[3:NATS Monitor]" } else { " 3:NATS Monitor " };
        let nav3_color = if current == AppView::NatsMonitor { Color::Cyan } else { Color::White };
        let nav3_weight = if current == AppView::NatsMonitor { Weight::Bold } else { Weight::Normal };

        let nav4_content = if current == AppView::StoryGenerator { "[4:Story Gen]" } else { " 4:Story Gen " };
        let nav4_color = if current == AppView::StoryGenerator { Color::Cyan } else { Color::White };
        let nav4_weight = if current == AppView::StoryGenerator { Weight::Bold } else { Weight::Normal };

        let nav5_content = if current == AppView::Search { "[5:Search]" } else { " 5:Search " };
        let nav5_color = if current == AppView::Search { Color::Cyan } else { Color::White };
        let nav5_weight = if current == AppView::Search { Weight::Bold } else { Weight::Normal };

        let nav6_content = if current == AppView::Settings { "[6:Settings]" } else { " 6:Settings " };
        let nav6_color = if current == AppView::Settings { Color::Cyan } else { Color::White };
        let nav6_weight = if current == AppView::Settings { Weight::Bold } else { Weight::Normal };

        let nav7_content = if current == AppView::Logs { "[7:Logs]" } else { " 7:Logs " };
        let nav7_color = if current == AppView::Logs { Color::Cyan } else { Color::White };
        let nav7_weight = if current == AppView::Logs { Weight::Bold } else { Weight::Normal };

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
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    Text(content: " ")
                    Text(content: nav1_content, color: nav1_color, weight: nav1_weight)
                    Text(content: "  ")
                    Text(content: nav2_content, color: nav2_color, weight: nav2_weight)
                    Text(content: "  ")
                    Text(content: nav3_content, color: nav3_color, weight: nav3_weight)
                    Text(content: "  ")
                    Text(content: nav4_content, color: nav4_color, weight: nav4_weight)
                    Text(content: "  ")
                    Text(content: nav5_content, color: nav5_color, weight: nav5_weight)
                    Text(content: "  ")
                    Text(content: nav6_content, color: nav6_color, weight: nav6_weight)
                    Text(content: "  ")
                    Text(content: nav7_content, color: nav7_color, weight: nav7_weight)
                    Text(content: " ")
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

        // Individual nav items with colors
        let nav1_content = if current == AppView::McpTester { "[1:MCP Tester]" } else { " 1:MCP Tester " };
        let nav1_color = if current == AppView::McpTester { Color::Cyan } else { Color::White };
        let nav1_weight = if current == AppView::McpTester { Weight::Bold } else { Weight::Normal };

        let nav2_content = if current == AppView::TrailViewer { "[2:Trail Viewer]" } else { " 2:Trail Viewer " };
        let nav2_color = if current == AppView::TrailViewer { Color::Cyan } else { Color::White };
        let nav2_weight = if current == AppView::TrailViewer { Weight::Bold } else { Weight::Normal };

        let nav3_content = if current == AppView::NatsMonitor { "[3:NATS Monitor]" } else { " 3:NATS Monitor " };
        let nav3_color = if current == AppView::NatsMonitor { Color::Cyan } else { Color::White };
        let nav3_weight = if current == AppView::NatsMonitor { Weight::Bold } else { Weight::Normal };

        let nav4_content = if current == AppView::StoryGenerator { "[4:Story Gen]" } else { " 4:Story Gen " };
        let nav4_color = if current == AppView::StoryGenerator { Color::Cyan } else { Color::White };
        let nav4_weight = if current == AppView::StoryGenerator { Weight::Bold } else { Weight::Normal };

        let nav5_content = if current == AppView::Search { "[5:Search]" } else { " 5:Search " };
        let nav5_color = if current == AppView::Search { Color::Cyan } else { Color::White };
        let nav5_weight = if current == AppView::Search { Weight::Bold } else { Weight::Normal };

        let nav6_content = if current == AppView::Settings { "[6:Settings]" } else { " 6:Settings " };
        let nav6_color = if current == AppView::Settings { Color::Cyan } else { Color::White };
        let nav6_weight = if current == AppView::Settings { Weight::Bold } else { Weight::Normal };

        let nav7_content = if current == AppView::Logs { "[7:Logs]" } else { " 7:Logs " };
        let nav7_color = if current == AppView::Logs { Color::Cyan } else { Color::White };
        let nav7_weight = if current == AppView::Logs { Weight::Bold } else { Weight::Normal };

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
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    Text(content: " ")
                    Text(content: nav1_content, color: nav1_color, weight: nav1_weight)
                    Text(content: "  ")
                    Text(content: nav2_content, color: nav2_color, weight: nav2_weight)
                    Text(content: "  ")
                    Text(content: nav3_content, color: nav3_color, weight: nav3_weight)
                    Text(content: "  ")
                    Text(content: nav4_content, color: nav4_color, weight: nav4_weight)
                    Text(content: "  ")
                    Text(content: nav5_content, color: nav5_color, weight: nav5_weight)
                    Text(content: "  ")
                    Text(content: nav6_content, color: nav6_color, weight: nav6_weight)
                    Text(content: "  ")
                    Text(content: nav7_content, color: nav7_color, weight: nav7_weight)
                    Text(content: " ")
                }
            }
        }
    }
    // Medium mode: Compact single-line navbar (2 rows)
    else if layout.layout_mode == LayoutMode::Modern {
        // Individual nav items with colors (compact version)
        let nav1_content = if current == AppView::McpTester { "[1:MCP]" } else { " 1:MCP " };
        let nav1_color = if current == AppView::McpTester { Color::Cyan } else { Color::White };
        let nav1_weight = if current == AppView::McpTester { Weight::Bold } else { Weight::Normal };

        let nav2_content = if current == AppView::TrailViewer { "[2:Trails]" } else { " 2:Trails " };
        let nav2_color = if current == AppView::TrailViewer { Color::Cyan } else { Color::White };
        let nav2_weight = if current == AppView::TrailViewer { Weight::Bold } else { Weight::Normal };

        let nav3_content = if current == AppView::NatsMonitor { "[3:Monitor]" } else { " 3:Monitor " };
        let nav3_color = if current == AppView::NatsMonitor { Color::Cyan } else { Color::White };
        let nav3_weight = if current == AppView::NatsMonitor { Weight::Bold } else { Weight::Normal };

        let nav4_content = if current == AppView::StoryGenerator { "[4:Gen]" } else { " 4:Gen " };
        let nav4_color = if current == AppView::StoryGenerator { Color::Cyan } else { Color::White };
        let nav4_weight = if current == AppView::StoryGenerator { Weight::Bold } else { Weight::Normal };

        let nav5_content = if current == AppView::Search { "[5:Search]" } else { " 5:Search " };
        let nav5_color = if current == AppView::Search { Color::Cyan } else { Color::White };
        let nav5_weight = if current == AppView::Search { Weight::Bold } else { Weight::Normal };

        let nav6_content = if current == AppView::Settings { "[6:Settings]" } else { " 6:Settings " };
        let nav6_color = if current == AppView::Settings { Color::Cyan } else { Color::White };
        let nav6_weight = if current == AppView::Settings { Weight::Bold } else { Weight::Normal };

        let nav7_content = if current == AppView::Logs { "[7:Logs]" } else { " 7:Logs " };
        let nav7_color = if current == AppView::Logs { Color::Cyan } else { Color::White };
        let nav7_weight = if current == AppView::Logs { Weight::Bold } else { Weight::Normal };

        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Blue,
            ) {
                View(
                    flex_direction: FlexDirection::Row,
                    padding: 0,
                ) {
                    Text(content: " ")
                    Text(content: nav1_content, color: nav1_color, weight: nav1_weight)
                    Text(content: " ")
                    Text(content: nav2_content, color: nav2_color, weight: nav2_weight)
                    Text(content: " ")
                    Text(content: nav3_content, color: nav3_color, weight: nav3_weight)
                    Text(content: " ")
                    Text(content: nav4_content, color: nav4_color, weight: nav4_weight)
                    Text(content: " ")
                    Text(content: nav5_content, color: nav5_color, weight: nav5_weight)
                    Text(content: " ")
                    Text(content: nav6_content, color: nav6_color, weight: nav6_weight)
                    Text(content: " ")
                    Text(content: nav7_content, color: nav7_color, weight: nav7_weight)
                    Text(content: " ")
                }
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
            layout_config: LayoutConfig::from_terminal_size(240, 60),
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
