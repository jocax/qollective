/// Execution Tree Component
///
/// Displays execution directories and their server entries in a tree structure

use crate::state::search_state::SearchContext;
use crate::layout::LayoutConfig;
use iocraft::prelude::*;

#[derive(Props)]
pub struct ExecutionTreeProps {
    pub search_context: SearchContext,
    pub layout_config: LayoutConfig,
}

impl Default for ExecutionTreeProps {
    fn default() -> Self {
        Self {
            search_context: SearchContext::new(),
            layout_config: LayoutConfig::default(),
        }
    }
}

/// Execution Tree Component
///
/// Shows a collapsible tree of execution directories and their server entries
#[component]
pub fn ExecutionTree(_hooks: Hooks, props: &ExecutionTreeProps) -> impl Into<AnyElement<'static>> {
    let ctx = &props.search_context;
    let dirs = ctx.get_execution_dirs();
    let selected_dir = ctx.selected_dir_index();
    let selected_server = ctx.selected_server_index();

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1
        ) {
            // Header
            Text(
                content: format!("Execution Directories ({})", dirs.len()),
                weight: Weight::Bold,
                color: Color::Cyan
            )

            #({
                let tree_elements: Vec<AnyElement> = if dirs.is_empty() {
                    vec![element! {
                        Text(
                            content: "No execution history found",
                            color: Color::Grey
                        )
                    }.into_any()]
                } else {
                    // Directory tree
                    let mut elements = Vec::new();
                    for (i, dir) in dirs.iter().enumerate() {
                        let mut dir_elements = vec![
                            element! {
                                Text(
                                    content: format!(
                                        "{} {} {}",
                                        if i == selected_dir { ">" } else { " " },
                                        if dir.expanded { "▼" } else { "▶" },
                                        dir.name
                                    ),
                                    color: if i == selected_dir { Color::Yellow } else { Color::White },
                                    weight: if i == selected_dir { Weight::Bold } else { Weight::Normal }
                                )
                            }.into_any()
                        ];

                        // Server entries (if expanded)
                        if dir.expanded {
                            for (j, server) in dir.servers.iter().enumerate() {
                                dir_elements.push(element! {
                                    Text(
                                        content: format!(
                                            "  {} ├─ {}",
                                            if i == selected_dir && j == selected_server { ">" } else { " " },
                                            server.name
                                        ),
                                        color: if i == selected_dir && j == selected_server {
                                            Color::Green
                                        } else {
                                            Color::Grey
                                        }
                                    )
                                }.into_any());
                            }
                        }

                        elements.push(element! {
                            View {
                                #(dir_elements.into_iter())
                            }
                        }.into_any());
                    }
                    elements
                };

                element! {
                    View(margin_top: 1) {
                        #(tree_elements.into_iter())
                    }
                }.into_any()
            })

            // Help text
            View(margin_top: 1) {
                Text(
                    content: "↑/↓: Navigate | Space: Expand/Collapse",
                    color: Color::Grey
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_tree_props_default() {
        let props = ExecutionTreeProps::default();
        assert_eq!(props.search_context.get_execution_dirs().len(), 0);
    }
}
