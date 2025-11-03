/// Search View Module
///
/// Execution history browser with split panel layout

use iocraft::prelude::*;
use crate::state::search_state::SearchContext;
use crate::layout::{LayoutConfig, LayoutMode};

mod execution_tree;
mod viewer_panel;

pub use execution_tree::ExecutionTree;
pub use viewer_panel::ViewerPanel;

#[derive(Props)]
pub struct SearchViewProps {
    pub search_context: SearchContext,
    pub layout_config: LayoutConfig,
}

impl Default for SearchViewProps {
    fn default() -> Self {
        Self {
            search_context: SearchContext::new(),
            layout_config: LayoutConfig::default(),
        }
    }
}

/// Search View Component
///
/// Main search view with execution history tree and request/response viewer
#[component]
pub fn SearchView(_hooks: Hooks, props: &SearchViewProps) -> impl Into<AnyElement<'static>> {
    let layout = props.layout_config;
    let ctx = &props.search_context;

    // Load execution directories on render
    let _ = ctx.load_execution_directories();

    element! {
        View(flex_direction: FlexDirection::Column) {
            // Header
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1
            ) {
                Text(
                    content: "Search - Execution History",
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
                Text(
                    content: "Browse and inspect MCP request/response execution history",
                    color: Color::DarkGrey
                )
            }

            // Search bar
            View(
                border_style: BorderStyle::Single,
                padding: 1,
                margin_bottom: 1
            ) {
                Text(content: format!("[Search: {}]", ctx.get_search_query()))
            }

            // Root directory
            View(padding: 1, margin_bottom: 1) {
                Text(content: format!("📁 Root Directory: {}", ctx.get_root_directory().display()))
            }

            // Main content: Split panel or vertical stack
            #({
                let main_content: AnyElement = if layout.supports_sidebar() {
                    // Desktop/Large: Split panel (30/70)
                    let sidebar_w = layout.sidebar_width().unwrap_or(60) as u16;
                    element! {
                        View(flex_grow: 1.0, flex_direction: FlexDirection::Row) {
                            View(
                                width: sidebar_w,
                                margin_right: 1
                            ) {
                                ExecutionTree(
                                    search_context: ctx.clone(),
                                    layout_config: layout
                                )
                            }

                            View(flex_grow: 1.0) {
                                ViewerPanel(
                                    search_context: ctx.clone(),
                                    layout_config: layout
                                )
                            }
                        }
                    }.into_any()
                } else {
                    // Medium/Small: Vertical stack
                    element! {
                        View(flex_grow: 1.0, flex_direction: FlexDirection::Column) {
                            View(flex_grow: 1.0, margin_bottom: 1) {
                                ExecutionTree(
                                    search_context: ctx.clone(),
                                    layout_config: layout
                                )
                            }
                            View(flex_grow: 1.0) {
                                ViewerPanel(
                                    search_context: ctx.clone(),
                                    layout_config: layout
                                )
                            }
                        }
                    }.into_any()
                };
                main_content
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_view_props_default() {
        let props = SearchViewProps::default();
        assert_eq!(props.search_context.get_execution_dirs().len(), 0);
        assert_eq!(props.layout_config.layout_mode, LayoutMode::FourK);
    }
}
