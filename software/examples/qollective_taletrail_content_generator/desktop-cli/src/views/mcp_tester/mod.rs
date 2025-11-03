/// MCP Testing Interface
///
/// Main view for testing MCP servers with tab-based navigation

pub mod history_panel;
pub mod request_editor;
pub mod response_viewer;
pub mod server_panel;
pub mod template_browser;

#[cfg(test)]
mod integration_tests;

use crate::components::text_editor::TextEditorState;
use crate::state::{McpContext, McpTab};
use history_panel::HistoryPanel;
use iocraft::prelude::*;
use request_editor::RequestEditor;
use response_viewer::ResponseViewer;
use template_browser::TemplateBrowser;

/// Props for McpTester view
#[derive(Props)]
pub struct McpTesterProps {
    pub mcp_context: McpContext,
    pub editor_state: TextEditorState,
}

impl Default for McpTesterProps {
    fn default() -> Self {
        Self {
            mcp_context: McpContext::new(),
            editor_state: TextEditorState::new(String::new()),
        }
    }
}

/// Main MCP Testing view with tab navigation
#[component]
pub fn McpTester(_hooks: Hooks, props: &McpTesterProps) -> impl Into<AnyElement<'static>> {
    let active_tab = props.mcp_context.active_tab();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Double,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: "MCP Testing Interface",
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // PRIMARY TABS: Server selection
    elements.push(render_server_tabs(&props.mcp_context).into());

    // SECONDARY TABS: Panel navigation
    elements.push(render_tab_bar(active_tab).into());

    // Active tab content
    let tab_content = match active_tab {
        McpTab::Templates => {
            element! {
                TemplateBrowser(
                    mcp_context: props.mcp_context.clone(),
                )
            }
            .into_any()
        }
        McpTab::Editor => {
            element! {
                RequestEditor(
                    mcp_context: props.mcp_context.clone(),
                    editor_state: props.editor_state.clone(),
                )
            }
            .into_any()
        }
        McpTab::Response => {
            element! {
                ResponseViewer(
                    mcp_context: props.mcp_context.clone(),
                )
            }
            .into_any()
        }
        McpTab::History => {
            element! {
                HistoryPanel(
                    mcp_context: props.mcp_context.clone(),
                )
            }
            .into_any()
        }
    };

    elements.push(tab_content);

    // Footer with global shortcuts
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "Tab/Shift+Tab: Switch panels | 1-4: Direct panel | Left/Right: Switch servers | Esc: Menu",
                    color: Color::DarkGrey,
                )
            }
        }
        .into_any(),
    );

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

/// Render primary server selection tabs
fn render_server_tabs(mcp_context: &McpContext) -> impl Into<AnyElement<'static>> {
    let servers = mcp_context.servers();
    let selected_index = mcp_context.selected_server_index();

    let mut server_elements: Vec<AnyElement> = Vec::new();

    for (idx, server) in servers.iter().enumerate() {
        let is_active = idx == selected_index;

        // Shorten server names for display
        let label = match server.name.as_str() {
            "orchestrator" => "Orchestrator",
            "story-generator" => "Story Gen",
            "quality-control" => "Quality",
            "constraint-enforcer" => "Constraint",
            "prompt-helper" => "Prompt",
            _ => &server.name,
        };

        let text_color = if is_active {
            Color::Black
        } else {
            Color::White
        };

        let bg_color = if is_active {
            Some(Color::Green)
        } else {
            None
        };

        let border_color = if is_active {
            Color::Green
        } else {
            Color::DarkGrey
        };

        server_elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: border_color,
                    background_color: bg_color,
                    padding: 1,
                    margin_right: if idx < servers.len() - 1 { 1 } else { 0 },
                ) {
                    Text(content: label.to_string(), color: text_color, weight: Weight::Bold)
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
            #(server_elements.into_iter())
        }
    }
}

/// Render tab navigation bar
fn render_tab_bar(active_tab: McpTab) -> impl Into<AnyElement<'static>> {
    let tabs = [
        McpTab::Templates,
        McpTab::Editor,
        McpTab::Response,
        McpTab::History,
    ];

    let mut tab_elements: Vec<AnyElement> = Vec::new();

    for (idx, tab) in tabs.iter().enumerate() {
        let is_active = *tab == active_tab;

        let label = format!(
            "[{}] {}",
            tab.tab_number(),
            tab.display_name()
        );

        let text_color = if is_active {
            Color::Black
        } else {
            Color::White
        };

        let bg_color = if is_active {
            Some(Color::Cyan)
        } else {
            None
        };

        let border_color = if is_active {
            Color::Cyan
        } else {
            Color::DarkGrey
        };

        tab_elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: border_color,
                    background_color: bg_color,
                    padding: 1,
                    margin_right: if idx < tabs.len() - 1 { 1 } else { 0 },
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
            #(tab_elements.into_iter())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::McpTab;

    #[test]
    fn test_mcp_tester_tab_navigation() {
        let ctx = McpContext::new();

        // Default tab is Templates
        assert_eq!(ctx.active_tab(), McpTab::Templates);

        // Navigate to next tab
        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Response);

        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::History);

        // Wrap around to Templates
        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Templates);

        // Navigate backwards
        ctx.previous_tab();
        assert_eq!(ctx.active_tab(), McpTab::History);
    }

    #[test]
    fn test_direct_tab_selection() {
        let ctx = McpContext::new();

        // Jump directly to Response tab
        ctx.set_active_tab(McpTab::Response);
        assert_eq!(ctx.active_tab(), McpTab::Response);

        // Jump to History
        ctx.set_active_tab(McpTab::History);
        assert_eq!(ctx.active_tab(), McpTab::History);

        // Jump to Editor
        ctx.set_active_tab(McpTab::Editor);
        assert_eq!(ctx.active_tab(), McpTab::Editor);
    }

    #[test]
    fn test_tab_numbers() {
        assert_eq!(McpTab::Templates.tab_number(), 1);
        assert_eq!(McpTab::Editor.tab_number(), 2);
        assert_eq!(McpTab::Response.tab_number(), 3);
        assert_eq!(McpTab::History.tab_number(), 4);

        // Test from_number
        assert_eq!(McpTab::from_number(1), Some(McpTab::Templates));
        assert_eq!(McpTab::from_number(2), Some(McpTab::Editor));
        assert_eq!(McpTab::from_number(3), Some(McpTab::Response));
        assert_eq!(McpTab::from_number(4), Some(McpTab::History));
        assert_eq!(McpTab::from_number(0), None);
        assert_eq!(McpTab::from_number(5), None);
    }

    #[test]
    fn test_tab_display_names() {
        assert_eq!(McpTab::Templates.display_name(), "Templates");
        assert_eq!(McpTab::Editor.display_name(), "Request Editor");
        assert_eq!(McpTab::Response.display_name(), "Response");
        assert_eq!(McpTab::History.display_name(), "History");
    }
}
