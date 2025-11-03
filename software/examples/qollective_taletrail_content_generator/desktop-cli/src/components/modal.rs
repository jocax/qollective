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
                        color: Color::DarkGrey,
                    )
                }
            }
        }
    }
}

/// Create help modal content
pub fn create_help_content() -> Vec<String> {
    vec![
        "Keyboard Shortcuts:".to_string(),
        "".to_string(),
        "Global Shortcuts:".to_string(),
        "  Ctrl+H or F1    - Show this help".to_string(),
        "  Ctrl+Q          - Quit application".to_string(),
        "  ESC             - Close modal or go back".to_string(),
        "".to_string(),
        "Navigation Shortcuts:".to_string(),
        "  Ctrl+1          - MCP Tester".to_string(),
        "  Ctrl+2          - Trail Viewer".to_string(),
        "  Ctrl+3          - NATS Monitor".to_string(),
        "  Ctrl+4          - Story Generator".to_string(),
        "  Ctrl+5          - Search & Comparison".to_string(),
        "  Ctrl+6          - Settings".to_string(),
        "".to_string(),
        "Menu Navigation:".to_string(),
        "  ↑/↓ or j/k      - Navigate menu items".to_string(),
        "  1-7             - Select menu item by number".to_string(),
        "  Enter           - Confirm selection".to_string(),
    ]
}
