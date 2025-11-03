use iocraft::prelude::*;

/// Modal component properties
#[derive(Props, Default)]
pub struct ModalProps {
    pub title: String,
    pub content: Vec<String>,
    pub visible: bool,
}

/// Modal overlay component for displaying help and other overlays
#[component]
pub fn Modal(_hooks: Hooks, props: &ModalProps) -> impl Into<AnyElement<'static>> {
    if !props.visible {
        return element! {
            View {}
        };
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position: Position::Absolute,
            top: 0u16,
            left: 0u16,
            right: 0u16,
            bottom: 0u16,
        ) {
            View(
                border_style: BorderStyle::Double,
                border_color: Color::Yellow,
                background_color: Color::Black,
                width: 60u16,
            ) {
                View(flex_direction: FlexDirection::Column, padding: 1) {
                    View(margin_bottom: 1) {
                        Text(
                            content: props.title.clone(),
                            weight: Weight::Bold,
                            color: Color::Yellow,
                        )
                    }

                    #(props.content.iter().map(|line| {
                        element! {
                            Text(content: line.clone(), color: Color::White)
                        }
                    }))

                    View(margin_top: 1) {
                        Text(content: "")
                    }

                    Text(
                        content: "Press ESC to close",
                        color: Color::Grey,
                    )
                }
            }
        }
    }
}

/// Create help modal content
pub fn create_help_content() -> Vec<String> {
    vec![
        "Keyboard Shortcuts (macOS-optimized):".to_string(),
        "".to_string(),
        "Global Shortcuts:".to_string(),
        "  Ctrl+H or F1        - Show this help".to_string(),
        "  Ctrl+Q              - Quit application".to_string(),
        "  ⇧⌃T (Shift+Ctrl+T)  - Toggle theme (Dark/Light)".to_string(),
        "  ⇧⌃D (Shift+Ctrl+D)  - Toggle debug console".to_string(),
        "  ⇧⌃M (Shift+Ctrl+M)  - Toggle debug mode".to_string(),
        "  Ctrl+L              - Cycle display mode".to_string(),
        "  ESC                 - Close modal or go back".to_string(),
        "".to_string(),
        "Navigation Shortcuts:".to_string(),
        "  ⇧⌃1 or F2           - MCP Tester".to_string(),
        "  ⇧⌃2 or F3           - Trail Viewer".to_string(),
        "  ⇧⌃3 or F4           - NATS Monitor".to_string(),
        "  ⇧⌃4 or F5           - Story Generator".to_string(),
        "  ⇧⌃5 or F6           - Search & Comparison".to_string(),
        "  ⇧⌃6 or F7           - Settings".to_string(),
        "  ⇧⌃7 or F8           - Logs".to_string(),
        "".to_string(),
        "Menu Navigation:".to_string(),
        "  ↑/↓ or j/k          - Navigate menu items".to_string(),
        "  Enter               - Confirm selection".to_string(),
        "".to_string(),
        "Note: ⇧⌃ = Shift+Ctrl (primary on macOS)".to_string(),
        "      F-keys work if you press Fn or disable system shortcuts".to_string(),
    ]
}
