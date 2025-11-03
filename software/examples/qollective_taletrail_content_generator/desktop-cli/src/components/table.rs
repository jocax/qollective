use iocraft::prelude::*;

/// Column definition for table
#[derive(Clone, Debug)]
pub struct ColumnDefinition {
    pub title: String,
    pub width: usize,
}

impl ColumnDefinition {
    pub fn new(title: impl Into<String>, width: usize) -> Self {
        Self {
            title: title.into(),
            width,
        }
    }
}

/// Truncate text to fit within a given width
fn truncate_text(text: &str, width: usize) -> String {
    if text.len() <= width {
        format!("{:<width$}", text, width = width)
    } else if width > 3 {
        format!("{:<width$}...", &text[..width - 3], width = width - 3)
    } else {
        "...".to_string()
    }
}

/// Pad text to fit within a given width
fn pad_text(text: &str, width: usize) -> String {
    format!("{:<width$}", text, width = width)
}

/// Props for Table component
#[derive(Props)]
pub struct TableProps {
    pub columns: Vec<ColumnDefinition>,
    pub rows: Vec<Vec<String>>,
    pub selected_row: Option<usize>,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            selected_row: None,
        }
    }
}

/// Table component with column headers and row highlighting
#[component]
pub fn Table(_hooks: Hooks, props: &TableProps) -> impl Into<AnyElement<'static>> {
    let mut elements: Vec<AnyElement> = Vec::new();

    let columns = &props.columns;
    let rows = &props.rows;

    // Render header row
    let header_parts: Vec<String> = columns
        .iter()
        .map(|col| pad_text(&col.title, col.width))
        .collect();
    let header_line = header_parts.join(" | ");

    elements.push(
        element! {
            View {
                Text(content: header_line, color: Color::Cyan, weight: Weight::Bold)
            }
        }
        .into_any(),
    );

    // Render separator line
    let separator_parts: Vec<String> = columns
        .iter()
        .map(|col| "-".repeat(col.width))
        .collect();
    let separator_line = separator_parts.join("-+-");

    elements.push(
        element! {
            Text(content: separator_line, color: Color::Grey)
        }
        .into_any(),
    );

    // Render data rows
    for (row_idx, row) in rows.iter().enumerate() {
        let is_selected = props.selected_row == Some(row_idx);

        let row_parts: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(col_idx, col)| {
                let cell_value = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                truncate_text(cell_value, col.width)
            })
            .collect();

        let row_line = row_parts.join(" | ");

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

        elements.push(
            element! {
                View(background_color: bg_color) {
                    Text(content: row_line, color: text_color)
                }
            }
            .into_any(),
        );
    }

    // If no rows, show empty message
    if rows.is_empty() {
        elements.push(
            element! {
                View(margin_top: 1) {
                    Text(content: "No data to display", color: Color::Grey)
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
            #(elements.into_iter())
        }
    }
    .into_any()
}

// Helper function to create a simple table (not a component)
pub fn create_simple_table(
    headers: &[&str],
    rows: &[Vec<&str>],
    default_width: usize,
    selected_row: Option<usize>,
) -> TableProps {
    let columns: Vec<ColumnDefinition> = headers
        .iter()
        .map(|header| ColumnDefinition::new(*header, default_width))
        .collect();

    let owned_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|s| s.to_string()).collect())
        .collect();

    TableProps {
        columns,
        rows: owned_rows,
        selected_row,
    }
}
