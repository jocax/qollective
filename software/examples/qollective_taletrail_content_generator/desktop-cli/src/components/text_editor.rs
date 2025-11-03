use iocraft::prelude::*;

/// State management for text editor
#[derive(Clone)]
pub struct TextEditorState {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_column: usize,
    scroll_offset: usize,
}

impl TextEditorState {
    pub fn new(content: String) -> Self {
        let lines: Vec<String> = content.split('\n').map(|s| s.to_string()).collect();
        Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
            cursor_line: 0,
            cursor_column: 0,
            scroll_offset: 0,
        }
    }

    pub fn from_lines(lines: Vec<String>) -> Self {
        Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
            cursor_line: 0,
            cursor_column: 0,
            scroll_offset: 0,
        }
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_content(&mut self, content: String) {
        self.lines = content.split('\n').map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.clamp_cursor();
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn cursor_column(&self) -> usize {
        self.cursor_column
    }

    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    fn current_line(&self) -> &String {
        &self.lines[self.cursor_line]
    }

    fn current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_line]
    }

    fn clamp_cursor(&mut self) {
        // Clamp line
        if self.cursor_line >= self.lines.len() {
            self.cursor_line = self.lines.len().saturating_sub(1);
        }

        // Clamp column
        let line_len = self.current_line().len();
        if self.cursor_column > line_len {
            self.cursor_column = line_len;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_column > 0 {
            self.cursor_column -= 1;
        } else if self.cursor_line > 0 {
            // Move to end of previous line
            self.cursor_line -= 1;
            self.cursor_column = self.current_line().len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_column < line_len {
            self.cursor_column += 1;
        } else if self.cursor_line < self.lines.len() - 1 {
            // Move to start of next line
            self.cursor_line += 1;
            self.cursor_column = 0;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.clamp_cursor();
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_column = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_column = self.current_line().len();
    }

    pub fn insert_char(&mut self, ch: char) {
        let line_len = self.current_line().len();
        if self.cursor_column > line_len {
            self.cursor_column = line_len;
        }
        let cursor_col = self.cursor_column;
        self.current_line_mut().insert(cursor_col, ch);
        self.cursor_column += 1;
    }

    pub fn insert_newline(&mut self) {
        let cursor_col = self.cursor_column;
        let remainder = self.current_line_mut().split_off(cursor_col);
        self.cursor_line += 1;
        self.lines.insert(self.cursor_line, remainder);
        self.cursor_column = 0;
    }

    pub fn delete_char(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_column < line_len {
            let cursor_col = self.cursor_column;
            self.current_line_mut().remove(cursor_col);
        } else if self.cursor_line < self.lines.len() - 1 {
            // Merge with next line
            let next_line = self.lines.remove(self.cursor_line + 1);
            self.current_line_mut().push_str(&next_line);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_column > 0 {
            self.cursor_column -= 1;
            let cursor_col = self.cursor_column;
            self.current_line_mut().remove(cursor_col);
        } else if self.cursor_line > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_column = self.current_line().len();
            self.current_line_mut().push_str(&current_line);
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.cursor_line = self.cursor_line.saturating_sub(page_size);
        self.clamp_cursor();
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.cursor_line = (self.cursor_line + page_size).min(self.lines.len() - 1);
        self.clamp_cursor();
    }

    pub fn calculate_scroll(&mut self, visible_rows: usize) {
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.cursor_line - visible_rows + 1;
        }
    }

    pub fn get_visible_lines(&self, visible_rows: usize) -> &[String] {
        let end = (self.scroll_offset + visible_rows).min(self.lines.len());
        &self.lines[self.scroll_offset..end]
    }
}

/// Props for TextEditor component
#[derive(Props)]
pub struct TextEditorProps {
    pub content: String,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub visible_rows: usize,
    pub show_line_numbers: bool,
    pub error_message: Option<String>,
}

impl Default for TextEditorProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            cursor_line: 0,
            cursor_column: 0,
            visible_rows: 20,
            show_line_numbers: false,
            error_message: None,
        }
    }
}

/// Text editor component with multi-line editing
#[component]
pub fn TextEditor(_hooks: Hooks, props: &TextEditorProps) -> impl Into<AnyElement<'static>> {
    let lines: Vec<&str> = props.content.split('\n').collect();
    let total_lines = lines.len();

    // Calculate visible range
    let mut scroll_offset = 0;
    if props.cursor_line >= props.visible_rows {
        scroll_offset = props.cursor_line - props.visible_rows + 1;
    }

    let end = (scroll_offset + props.visible_rows).min(total_lines);
    let visible_lines = &lines[scroll_offset..end];

    let mut elements: Vec<AnyElement> = Vec::new();

    // Render visible lines
    for (idx, line) in visible_lines.iter().enumerate() {
        let actual_line_num = scroll_offset + idx;
        let is_cursor_line = actual_line_num == props.cursor_line;

        let line_content = if props.show_line_numbers {
            format!("{:4} | {}", actual_line_num + 1, line)
        } else {
            line.to_string()
        };

        let text_color = if is_cursor_line {
            Color::White
        } else {
            Color::DarkGrey
        };

        elements.push(
            element! {
                Text(content: line_content, color: text_color)
            }
            .into_any(),
        );
    }

    // Render cursor position indicator
    let cursor_info = format!(
        "Line {}, Col {} | {} lines",
        props.cursor_line + 1,
        props.cursor_column + 1,
        total_lines
    );

    elements.push(
        element! {
            View(margin_top: 1) {
                Text(content: cursor_info, color: Color::Cyan)
            }
        }
        .into_any(),
    );

    // Render error message if present
    if let Some(error) = &props.error_message {
        elements.push(
            element! {
                View(margin_top: 1) {
                    Text(content: error, color: Color::Red, weight: Weight::Bold)
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
