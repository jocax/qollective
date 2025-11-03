/// Server Selection Panel
///
/// Displays list of MCP servers with connection status and navigation

use crate::components::list::List;
use crate::models::mcp::McpServerConfig;
use crate::state::McpContext;
use iocraft::prelude::*;

/// Props for ServerPanel component
#[derive(Props)]
pub struct ServerPanelProps {
    pub mcp_context: McpContext,
}

impl Default for ServerPanelProps {
    fn default() -> Self {
        Self {
            mcp_context: McpContext::new(),
        }
    }
}

/// Server selection panel component
#[component]
pub fn ServerPanel(_hooks: Hooks, props: &ServerPanelProps) -> impl Into<AnyElement<'static>> {
    let servers = props.mcp_context.servers();
    let selected_index = props.mcp_context.selected_server_index();

    // Render function for server items
    fn render_server(server: &McpServerConfig, is_selected: bool) -> String {
        let status_icon = if server.available { "●" } else { "○" };
        let prefix = if is_selected { "> " } else { "  " };

        format!(
            "{}{} {} - {}",
            prefix,
            status_icon,
            server.name,
            server.description.as_deref().unwrap_or("No description")
        )
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: "MCP Servers",
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
            List::<McpServerConfig>(
                items: servers.clone(),
                selected_index: selected_index,
                render_item: Some(render_server as fn(&McpServerConfig, bool) -> String),
                visible_rows: 10usize,
                show_pagination: false,
            )
            View(margin_top: 1) {
                Text(
                    content: "↑/↓: Navigate | Enter: Select | 1-5: Quick select",
                    color: Color::DarkGrey,
                )
            }
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mcp::McpServerConfig;

    #[test]
    fn test_server_panel_renders() {
        let ctx = McpContext::new();

        // Set up test servers
        let servers = vec![
            McpServerConfig {
                name: "orchestrator".to_string(),
                subject: "mcp.orchestrator.>".to_string(),
                available: true,
                description: Some("Main orchestrator".to_string()),
            },
            McpServerConfig {
                name: "story-generator".to_string(),
                subject: "mcp.story-generator.>".to_string(),
                available: false,
                description: Some("Story generator".to_string()),
            },
        ];

        ctx.set_servers(servers);

        // Verify server data is accessible
        let loaded_servers = ctx.servers();
        assert_eq!(loaded_servers.len(), 2);
        assert_eq!(loaded_servers[0].name, "orchestrator");
        assert!(loaded_servers[0].available);
    }

    #[test]
    fn test_server_navigation() {
        let ctx = McpContext::new();

        // Default has 5 servers
        assert_eq!(ctx.servers().len(), 5);
        assert_eq!(ctx.selected_server_index(), 0);

        // Navigate next
        ctx.next_server();
        assert_eq!(ctx.selected_server_index(), 1);

        // Navigate previous
        ctx.previous_server();
        assert_eq!(ctx.selected_server_index(), 0);

        // Wrap around from end
        ctx.set_selected_server_index(4);
        ctx.next_server();
        assert_eq!(ctx.selected_server_index(), 0);
    }

    #[test]
    fn test_selected_server() {
        let ctx = McpContext::new();

        let selected = ctx.selected_server();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "orchestrator");

        ctx.set_selected_server_index(1);
        let selected = ctx.selected_server();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "story-generator");
    }
}
