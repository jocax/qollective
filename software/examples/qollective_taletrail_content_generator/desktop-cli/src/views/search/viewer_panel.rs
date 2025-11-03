/// Viewer Panel Component
///
/// Displays request and response data for selected execution

use crate::state::search_state::SearchContext;
use crate::layout::LayoutConfig;
use iocraft::prelude::*;

#[derive(Props)]
pub struct ViewerPanelProps {
    pub search_context: SearchContext,
    pub layout_config: LayoutConfig,
}

impl Default for ViewerPanelProps {
    fn default() -> Self {
        Self {
            search_context: SearchContext::new(),
            layout_config: LayoutConfig::default(),
        }
    }
}

/// Viewer Panel Component
///
/// Shows request and response data in a split vertical layout
#[component]
pub fn ViewerPanel(_hooks: Hooks, props: &ViewerPanelProps) -> impl Into<AnyElement<'static>> {
    let ctx = &props.search_context;

    // Load content for selected server
    let (request, response) = ctx
        .load_selected_content()
        .unwrap_or_else(|_| ("Error loading request".to_string(), "Error loading response".to_string()));

    element! {
        View(flex_direction: FlexDirection::Column) {
            // Request panel (top half)
            View(
                flex_grow: 1.0,
                border_style: BorderStyle::Single,
                border_color: Color::Blue,
                padding: 1,
                margin_bottom: 1
            ) {
                Text(
                    content: "REQUEST",
                    weight: Weight::Bold,
                    color: Color::Blue
                )
                #({
                    let request_lines: Vec<AnyElement> = request.lines().map(|line| {
                        element! {
                            Text(content: line.to_string())
                        }.into_any()
                    }).collect();

                    element! {
                        View(margin_top: 1) {
                            #(request_lines.into_iter())
                        }
                    }.into_any()
                })
            }

            // Response panel (bottom half)
            View(
                flex_grow: 1.0,
                border_style: BorderStyle::Single,
                border_color: Color::Green,
                padding: 1
            ) {
                Text(
                    content: "RESPONSE",
                    weight: Weight::Bold,
                    color: Color::Green
                )
                #({
                    let response_lines: Vec<AnyElement> = response.lines().map(|line| {
                        element! {
                            Text(content: line.to_string())
                        }.into_any()
                    }).collect();

                    element! {
                        View(margin_top: 1) {
                            #(response_lines.into_iter())
                        }
                    }.into_any()
                })
            }

            // Footer help
            View(margin_top: 1, padding: 1) {
                Text(
                    content: "Enter: Select | ESC: Back",
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
    fn test_viewer_panel_props_default() {
        let props = ViewerPanelProps::default();
        assert_eq!(props.search_context.get_execution_dirs().len(), 0);
    }
}
