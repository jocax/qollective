use iocraft::prelude::*;

use crate::state::View as AppView;

/// Props for PlaceholderView
#[derive(Props, Default)]
pub struct PlaceholderViewProps {
    pub view: AppView,
}

/// Placeholder component for views not yet implemented
#[component]
pub fn PlaceholderView(_hooks: Hooks, props: &PlaceholderViewProps) -> impl Into<AnyElement<'static>> {
    let view_name = props.view.display_name();

    // Unique styling per view
    let (emoji, border_color) = match props.view {
        AppView::McpTester => ("🧪", Color::Magenta),
        AppView::TrailViewer => ("🎬", Color::Cyan),
        AppView::NatsMonitor => ("📊", Color::Yellow),
        AppView::StoryGenerator => ("✍️", Color::Green),
        AppView::Settings => ("⚙️", Color::Blue),
        _ => ("🚧", Color::Magenta),  // Fallback for other views
    };

    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: border_color,
        ) {
            View(
                flex_direction: FlexDirection::Column,
                padding: 2,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: 20u16,
            ) {
                View(margin_bottom: 2) {
                    Text(
                        content: format!("{} {} {}", emoji, view_name, emoji),
                        weight: Weight::Bold,
                        color: border_color,
                    )
                }

                View(margin_bottom: 1) {
                    Text(
                        content: "This view is under construction.",
                        color: Color::White,
                    )
                }

                View(margin_bottom: 1) {
                    Text(
                        content: "It will be implemented in a future task group.",
                        color: Color::Grey,
                    )
                }

                Text(
                    content: "Try Ctrl+5 for Search or Ctrl+7 for Logs (fully implemented!)",
                    color: Color::Green,
                )
            }
        }
    }
}
