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

    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::Magenta,
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
                        content: format!("🚧 {} 🚧", view_name),
                        weight: Weight::Bold,
                        color: Color::Magenta,
                    )
                }

                View(margin_bottom: 1) {
                    Text(
                        content: "This view is under construction.",
                        color: Color::White,
                    )
                }

                Text(
                    content: "It will be implemented in a future task group.",
                    color: Color::DarkGrey,
                )
            }
        }
    }
}
