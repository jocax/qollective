/// Card Grid Component
///
/// Generic component for displaying items in a responsive grid layout

use iocraft::prelude::*;
use crate::layout::LayoutConfig;

/// Props for the Card Grid component
#[derive(Props)]
pub struct CardGridProps<T: Clone + Send + Sync + 'static> {
    pub items: Vec<T>,
    pub columns: usize,
    pub render_card: Option<fn(&T, bool) -> Vec<String>>,  // Returns lines of the card, bool indicates if selected
    pub selected_index: usize,
    pub layout_config: LayoutConfig,
}

impl<T: Clone + Send + Sync + 'static> Default for CardGridProps<T> {
    fn default() -> Self {
        Self {
            items: vec![],
            columns: 1,
            render_card: None,
            selected_index: 0,
            layout_config: LayoutConfig::default(),
        }
    }
}

/// Card Grid Component
///
/// Displays items in a grid layout with configurable columns
#[component]
pub fn CardGrid<T: Clone + Send + Sync + 'static>(
    _hooks: Hooks,
    props: &CardGridProps<T>
) -> impl Into<AnyElement<'static>> {
    let cols = props.columns;
    let items = &props.items;
    let selected_idx = props.selected_index;

    // Group items into rows
    let mut rows: Vec<Vec<(usize, T)>> = vec![];
    for (idx, item) in items.iter().enumerate() {
        if idx % cols == 0 {
            rows.push(vec![]);
        }
        if let Some(last_row) = rows.last_mut() {
            last_row.push((idx, item.clone()));
        }
    }

    // Build row elements outside the macro
    let mut row_elements = Vec::new();
    for row in rows {
        let mut card_elements = Vec::new();
        for (idx, item) in row {
            let border_style = if idx == selected_idx {
                BorderStyle::Double
            } else {
                BorderStyle::Single
            };
            let border_color = if idx == selected_idx {
                Color::Yellow
            } else {
                Color::DarkGrey
            };

            let mut text_elements = Vec::new();
            if let Some(render_fn) = props.render_card {
                for line in render_fn(&item, idx == selected_idx) {
                    let text_color = if idx == selected_idx {
                        Color::White
                    } else {
                        Color::DarkGrey
                    };
                    text_elements.push(element! {
                        Text(content: line, color: text_color)
                    }.into_any());
                }
            } else {
                text_elements.push(element! {
                    Text(content: "No render function provided")
                }.into_any());
            }

            card_elements.push(element! {
                View(
                    flex_grow: 1.0,
                    border_style: border_style,
                    border_color: border_color,
                    margin_right: 1,
                    padding: 1
                ) {
                    #(text_elements.into_iter())
                }
            }.into_any());
        }

        row_elements.push(element! {
            View(
                flex_direction: FlexDirection::Row,
                margin_bottom: 1
            ) {
                #(card_elements.into_iter())
            }
        }.into_any());
    }

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(row_elements.into_iter())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestItem {
        name: String,
        value: i32,
    }

    fn test_render_card(item: &TestItem, _selected: bool) -> Vec<String> {
        vec![
            format!("Name: {}", item.name),
            format!("Value: {}", item.value),
        ]
    }

    #[test]
    fn test_card_grid_props_default() {
        let props: CardGridProps<TestItem> = CardGridProps::default();
        assert_eq!(props.items.len(), 0);
        assert_eq!(props.columns, 1);
        assert_eq!(props.selected_index, 0);
    }

    #[test]
    fn test_card_grid_with_items() {
        let items = vec![
            TestItem { name: "Item 1".to_string(), value: 100 },
            TestItem { name: "Item 2".to_string(), value: 200 },
            TestItem { name: "Item 3".to_string(), value: 300 },
        ];

        let props = CardGridProps {
            items: items.clone(),
            columns: 2,
            render_card: Some(test_render_card),
            selected_index: 1,
            layout_config: LayoutConfig::default(),
        };

        assert_eq!(props.items.len(), 3);
        assert_eq!(props.columns, 2);
        assert_eq!(props.selected_index, 1);
    }
}
