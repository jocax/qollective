/// Template Browser Component
///
/// Displays and filters MCP request templates with search functionality

use crate::components::form::TextInput;
use crate::components::list::List;
use crate::models::mcp::TemplateInfo;
use crate::state::McpContext;
use iocraft::prelude::*;

/// Props for TemplateBrowser component
#[derive(Props)]
pub struct TemplateBrowserProps {
    pub mcp_context: McpContext,
}

impl Default for TemplateBrowserProps {
    fn default() -> Self {
        Self {
            mcp_context: McpContext::new(),
        }
    }
}

/// Template browser component with search and filtering
#[component]
pub fn TemplateBrowser(
    _hooks: Hooks,
    props: &TemplateBrowserProps,
) -> impl Into<AnyElement<'static>> {
    let filtered_templates = props.mcp_context.filtered_templates();
    let selected_index = props.mcp_context.selected_template_index();
    let search_filter = props.mcp_context.template_filter();

    // Group templates by server for display
    let mut grouped_display: Vec<(String, Vec<&TemplateInfo>)> = Vec::new();
    let mut current_server: Option<String> = None;
    let mut current_group: Vec<&TemplateInfo> = Vec::new();

    for template in &filtered_templates {
        if current_server.as_ref() != Some(&template.server_name) {
            // Start a new group
            if let Some(server) = current_server {
                grouped_display.push((server, current_group));
            }
            current_server = Some(template.server_name.clone());
            current_group = vec![template];
        } else {
            current_group.push(template);
        }
    }

    // Push the last group
    if let Some(server) = current_server {
        grouped_display.push((server, current_group));
    }

    // Render function for template items
    fn render_template(template: &TemplateInfo, is_selected: bool) -> String {
        let prefix = if is_selected { "> " } else { "  " };
        let desc = template
            .description
            .as_ref()
            .map(|d| format!(" - {}", d))
            .unwrap_or_default();

        format!(
            "{}[{}] {}: {}{}",
            prefix, template.server_name, template.template_name, template.tool_name, desc
        )
    }

    let template_count = filtered_templates.len();
    let total_count = props.mcp_context.templates().len();

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
                    content: "Template Browser",
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
            TextInput(
                label: "Search Templates".to_string(),
                value: search_filter.clone(),
                error: None,
                placeholder: Some("Type to filter templates...".to_string()),
                is_focused: false,
            )
            View(margin_top: 1, margin_bottom: 1) {
                Text(
                    content: format!("Showing {} of {} templates", template_count, total_count),
                    color: Color::DarkGrey,
                )
            }
            List::<TemplateInfo>(
                items: filtered_templates.clone(),
                selected_index: selected_index,
                render_item: Some(render_template as fn(&TemplateInfo, bool) -> String),
                visible_rows: 15usize,
                show_pagination: true,
            )
            View(margin_top: 1) {
                Text(
                    content: "↑/↓: Navigate | Enter: Load Template | /: Search",
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
    use crate::models::mcp::TemplateInfo;

    #[test]
    fn test_template_browser_filtering() {
        let ctx = McpContext::new();

        // Set up test templates
        let templates = vec![
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "start_workflow".to_string(),
                file_path: "/path/to/start_workflow.json".to_string(),
                description: Some("Start a new workflow".to_string()),
                tool_name: "start_workflow".to_string(),
            },
            TemplateInfo {
                server_name: "story-generator".to_string(),
                template_name: "generate_scene".to_string(),
                file_path: "/path/to/generate_scene.json".to_string(),
                description: Some("Generate a story scene".to_string()),
                tool_name: "generate_scene".to_string(),
            },
            TemplateInfo {
                server_name: "quality-control".to_string(),
                template_name: "validate_content".to_string(),
                file_path: "/path/to/validate_content.json".to_string(),
                description: Some("Validate generated content".to_string()),
                tool_name: "validate_content".to_string(),
            },
        ];

        ctx.set_templates(templates);

        // Test initial state - all templates visible
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 3);

        // Test filtering by server name
        ctx.set_template_filter("story".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].server_name, "story-generator");

        // Test filtering by tool name
        ctx.set_template_filter("validate".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tool_name, "validate_content");

        // Test case-insensitive filtering
        ctx.set_template_filter("WORKFLOW".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].template_name, "start_workflow");

        // Test clearing filter
        ctx.set_template_filter(String::new());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_template_navigation() {
        let ctx = McpContext::new();

        let templates = vec![
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "template1".to_string(),
                file_path: "/path/to/template1.json".to_string(),
                description: None,
                tool_name: "tool1".to_string(),
            },
            TemplateInfo {
                server_name: "story-generator".to_string(),
                template_name: "template2".to_string(),
                file_path: "/path/to/template2.json".to_string(),
                description: None,
                tool_name: "tool2".to_string(),
            },
        ];

        ctx.set_templates(templates);

        // Test initial selection
        assert_eq!(ctx.selected_template_index(), 0);

        // Navigate next
        ctx.next_template();
        assert_eq!(ctx.selected_template_index(), 1);

        // Navigate previous
        ctx.previous_template();
        assert_eq!(ctx.selected_template_index(), 0);

        // Test wrap around
        ctx.previous_template();
        assert_eq!(ctx.selected_template_index(), 1);

        ctx.next_template();
        assert_eq!(ctx.selected_template_index(), 0);
    }

    #[test]
    fn test_selected_template() {
        let ctx = McpContext::new();

        let templates = vec![TemplateInfo {
            server_name: "orchestrator".to_string(),
            template_name: "test_template".to_string(),
            file_path: "/path/to/test.json".to_string(),
            description: Some("Test description".to_string()),
            tool_name: "test_tool".to_string(),
        }];

        ctx.set_templates(templates);

        let selected = ctx.selected_template();
        assert!(selected.is_some());

        let template = selected.unwrap();
        assert_eq!(template.template_name, "test_template");
        assert_eq!(template.tool_name, "test_tool");
        assert_eq!(
            template.description,
            Some("Test description".to_string())
        );
    }

    #[test]
    fn test_filter_resets_selection() {
        let ctx = McpContext::new();

        let templates = vec![
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "template1".to_string(),
                file_path: "/path/to/template1.json".to_string(),
                description: None,
                tool_name: "tool1".to_string(),
            },
            TemplateInfo {
                server_name: "story-generator".to_string(),
                template_name: "template2".to_string(),
                file_path: "/path/to/template2.json".to_string(),
                description: None,
                tool_name: "tool2".to_string(),
            },
        ];

        ctx.set_templates(templates);

        // Navigate to second template
        ctx.next_template();
        assert_eq!(ctx.selected_template_index(), 1);

        // Apply filter - selection should reset to 0
        ctx.set_template_filter("story".to_string());
        assert_eq!(ctx.selected_template_index(), 0);
    }
}
