use iocraft::prelude::*;

/// State management for scrollable list component
#[derive(Clone)]
pub struct ListState<T: Clone> {
    items: Vec<T>,
    selected_index: usize,
}

impl<T: Clone> ListState<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            selected_index: 0,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected_index
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected_index)
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = index;
        }
    }

    pub fn update_items(&mut self, items: Vec<T>) {
        self.items = items;
        // Reset selection if it's out of bounds
        if self.selected_index >= self.items.len() && !self.items.is_empty() {
            self.selected_index = self.items.len() - 1;
        } else if self.items.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Calculate viewport range for scrollable list
pub fn calculate_viewport(selected: usize, total: usize, visible: usize) -> (usize, usize) {
    if total <= visible {
        // All items fit on screen
        return (0, total);
    }

    // Calculate start position to keep selected item visible
    // Keep at top until we need to scroll
    let start = if selected < visible {
        0
    } else if selected >= total - visible {
        total - visible
    } else {
        selected - visible + 1
    };

    let end = (start + visible).min(total);

    (start, end)
}

/// Props for the List component
#[derive(Props)]
pub struct ListProps<T: Clone + Send + Sync + 'static> {
    pub items: Vec<T>,
    pub selected_index: usize,
    pub render_item: Option<fn(&T, bool) -> String>,
    pub visible_rows: usize,
    pub show_pagination: bool,
}

impl<T: Clone + Send + Sync + 'static> Default for ListProps<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            render_item: None,
            visible_rows: 20,
            show_pagination: false,
        }
    }
}

/// Scrollable list component with selection and viewport
#[component]
pub fn List<T: Clone + Send + Sync + 'static>(
    _hooks: Hooks,
    props: &ListProps<T>,
) -> impl Into<AnyElement<'static>> {
    let items = &props.items;
    let total_items = items.len();

    if total_items == 0 {
        return element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Grey,
                padding: 1,
            ) {
                Text(content: "No items to display", color: Color::Grey)
            }
        }
        .into_any();
    }

    // Get render function or use default
    let render_fn = props.render_item.unwrap_or(|_item: &T, selected: bool| {
        format!("{}{}", if selected { "> " } else { "  " }, std::any::type_name::<T>())
    });

    // Calculate viewport
    let (start, end) = calculate_viewport(props.selected_index, total_items, props.visible_rows);

    // Render visible items
    let mut items_vec: Vec<AnyElement> = Vec::new();

    for (idx, item) in items[start..end].iter().enumerate() {
        let actual_index = start + idx;
        let is_selected = actual_index == props.selected_index;
        let content = render_fn(item, is_selected);

        let text_color = if is_selected {
            Color::Black
        } else {
            Color::White
        };

        let bg_color = if is_selected {
            Some(Color::Cyan)
        } else {
            None
        };

        items_vec.push(
            element! {
                View(background_color: bg_color) {
                    Text(content: content, color: text_color)
                }
            }
            .into_any(),
        );
    }

    // Add pagination info if requested
    if props.show_pagination {
        let pagination_text = format!(
            "Showing {}-{} of {}",
            start + 1,
            end,
            total_items
        );
        items_vec.push(
            element! {
                View(margin_top: 1) {
                    Text(content: pagination_text, color: Color::Grey)
                }
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
        ) {
            #(items_vec.into_iter())
        }
    }
    .into_any()
}
