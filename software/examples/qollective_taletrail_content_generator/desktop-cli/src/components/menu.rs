use iocraft::prelude::*;

use crate::state::View as AppView;

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub number: usize,
    pub label: &'static str,
    pub hotkey: &'static str,
    pub view: Option<AppView>,
}

impl MenuItem {
    pub const fn new(
        number: usize,
        label: &'static str,
        hotkey: &'static str,
        view: Option<AppView>,
    ) -> Self {
        Self {
            number,
            label,
            hotkey,
            view,
        }
    }
}

/// Menu component properties
#[derive(Props, Default)]
pub struct MenuProps {
    pub selected_index: usize,
}

/// Main navigation menu component
#[component]
pub fn Menu(_hooks: Hooks, props: &MenuProps) -> impl Into<AnyElement<'static>> {
    let selected = props.selected_index;

    let menu_items = [
        MenuItem::new(1, "MCP Tester", "Ctrl+1", Some(AppView::McpTester)),
        MenuItem::new(2, "Trail Viewer", "Ctrl+2", Some(AppView::TrailViewer)),
        MenuItem::new(3, "NATS Monitor", "Ctrl+3", Some(AppView::NatsMonitor)),
        MenuItem::new(4, "Story Generator", "Ctrl+4", Some(AppView::StoryGenerator)),
        MenuItem::new(5, "Search & Comparison", "Ctrl+5", Some(AppView::Search)),
        MenuItem::new(6, "Settings", "Ctrl+6", Some(AppView::Settings)),
        MenuItem::new(7, "Quit", "Ctrl+Q", None),
    ];

    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(flex_direction: FlexDirection::Column, padding: 1) {
                View(margin_bottom: 1) {
                    Text(
                        content: "TaleTrail Desktop CLI - Main Menu",
                        weight: Weight::Bold,
                        color: Color::Cyan,
                    )
                }

                #(menu_items.iter().enumerate().map(|(idx, item)| {
                    let is_selected = idx == selected;
                    let prefix = if is_selected { "→ " } else { "  " };
                    let color = if is_selected { Color::Yellow } else { Color::White };
                    let weight = if is_selected { Weight::Bold } else { Weight::Normal };

                    element! {
                        View(flex_direction: FlexDirection::Row) {
                            View(flex_grow: 1.0) {
                                Text(
                                    content: format!("{}{}. {}", prefix, item.number, item.label),
                                    color: color,
                                    weight: weight,
                                )
                            }
                            Text(
                                content: format!("[{}]", item.hotkey),
                                color: Color::DarkGrey,
                            )
                        }
                    }
                }))

                View(margin_top: 1) {
                    Text(content: "")
                }

                Text(
                    content: "Use ↑/↓ arrows or numbers 1-7 to select, Enter to confirm",
                    color: Color::DarkGrey,
                )
            }
        }
    }
}

/// Get the menu item at a specific index
pub fn get_menu_item(index: usize) -> Option<MenuItem> {
    let items = [
        MenuItem::new(1, "MCP Tester", "Ctrl+1", Some(AppView::McpTester)),
        MenuItem::new(2, "Trail Viewer", "Ctrl+2", Some(AppView::TrailViewer)),
        MenuItem::new(3, "NATS Monitor", "Ctrl+3", Some(AppView::NatsMonitor)),
        MenuItem::new(4, "Story Generator", "Ctrl+4", Some(AppView::StoryGenerator)),
        MenuItem::new(5, "Search & Comparison", "Ctrl+5", Some(AppView::Search)),
        MenuItem::new(6, "Settings", "Ctrl+6", Some(AppView::Settings)),
        MenuItem::new(7, "Quit", "Ctrl+Q", None),
    ];

    items.get(index).cloned()
}

/// Get total number of menu items
pub const fn menu_item_count() -> usize {
    7
}
