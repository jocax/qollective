use iocraft::prelude::*;

/// Validation result type
pub type ValidationResult = Result<(), String>;

/// Validate required field
pub fn validate_required(value: &str) -> ValidationResult {
    if value.trim().is_empty() {
        Err("This field is required".to_string())
    } else {
        Ok(())
    }
}

/// Validate numeric field
pub fn validate_numeric(value: &str) -> ValidationResult {
    value
        .parse::<f64>()
        .map(|_| ())
        .map_err(|_| "Must be a valid number".to_string())
}

/// Validate range
pub fn validate_range(value: &str, min: f64, max: f64) -> ValidationResult {
    match value.parse::<f64>() {
        Ok(num) => {
            if num >= min && num <= max {
                Ok(())
            } else {
                Err(format!("Must be between {} and {}", min, max))
            }
        }
        Err(_) => Err("Must be a valid number".to_string()),
    }
}

/// Props for TextInput component
#[derive(Props)]
pub struct TextInputProps {
    pub label: String,
    pub value: String,
    pub error: Option<String>,
    pub placeholder: Option<String>,
    pub is_focused: bool,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            value: String::new(),
            error: None,
            placeholder: None,
            is_focused: false,
        }
    }
}

/// Text input field with label and validation error display
#[component]
pub fn TextInput(_hooks: Hooks, props: &TextInputProps) -> impl Into<AnyElement<'static>> {
    let border_color = if props.error.is_some() {
        Color::Red
    } else if props.is_focused {
        Color::Cyan
    } else {
        Color::DarkGrey
    };

    let display_value = if props.value.is_empty() {
        props.placeholder.as_ref().map(|s| s.as_str()).unwrap_or("")
    } else {
        &props.value
    };

    let value_color = if props.value.is_empty() && props.placeholder.is_some() {
        Color::DarkGrey
    } else {
        Color::White
    };

    let mut elements: Vec<AnyElement> = Vec::new();

    // Label
    elements.push(
        element! {
            Text(content: &props.label, color: Color::Cyan, weight: Weight::Bold)
        }
        .into_any(),
    );

    // Input box
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: border_color,
                padding: 1,
                margin_top: 1,
            ) {
                Text(content: display_value, color: value_color)
            }
        }
        .into_any(),
    );

    // Error message
    if let Some(error) = &props.error {
        elements.push(
            element! {
                View(margin_top: 1) {
                    Text(content: error, color: Color::Red)
                }
            }
            .into_any(),
        );
    }

    element! {
        View(flex_direction: FlexDirection::Column, margin_bottom: 1) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Props for Select/Dropdown component
#[derive(Props)]
pub struct SelectProps {
    pub label: String,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub is_focused: bool,
    pub is_expanded: bool,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            options: Vec::new(),
            selected_index: 0,
            is_focused: false,
            is_expanded: false,
        }
    }
}

/// Select/Dropdown component for choosing from options
#[component]
pub fn Select(_hooks: Hooks, props: &SelectProps) -> impl Into<AnyElement<'static>> {
    let border_color = if props.is_focused {
        Color::Cyan
    } else {
        Color::DarkGrey
    };

    let mut elements: Vec<AnyElement> = Vec::new();

    // Label
    elements.push(
        element! {
            Text(content: &props.label, color: Color::Cyan, weight: Weight::Bold)
        }
        .into_any(),
    );

    if props.is_expanded {
        // Show all options when expanded
        for (idx, option) in props.options.iter().enumerate() {
            let is_selected = idx == props.selected_index;
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
                        Text(content: option, color: text_color)
                    }
                }
                .into_any(),
            );
        }
    } else {
        // Show only selected option when collapsed
        let selected_value = props
            .options
            .get(props.selected_index)
            .map(|s| s.as_str())
            .unwrap_or("(none)");

        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: border_color,
                    padding: 1,
                    margin_top: 1,
                ) {
                    Text(content: format!("{} ▼", selected_value), color: Color::White)
                }
            }
            .into_any(),
        );
    }

    element! {
        View(flex_direction: FlexDirection::Column, margin_bottom: 1) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// State management for form fields
#[derive(Clone)]
pub struct FormState {
    fields: Vec<(String, String)>, // (name, value) pairs
    errors: Vec<(String, String)>, // (name, error) pairs
    focused_field: usize,
}

impl FormState {
    pub fn new(field_names: Vec<String>) -> Self {
        let fields = field_names.into_iter().map(|name| (name, String::new())).collect();
        Self {
            fields,
            errors: Vec::new(),
            focused_field: 0,
        }
    }

    pub fn get_field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn set_field(&mut self, name: &str, value: String) {
        if let Some((_, v)) = self.fields.iter_mut().find(|(n, _)| n == name) {
            *v = value;
        }
    }

    pub fn get_error(&self, name: &str) -> Option<&str> {
        self.errors
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, e)| e.as_str())
    }

    pub fn set_error(&mut self, name: &str, error: String) {
        // Remove existing error
        self.errors.retain(|(n, _)| n != name);
        // Add new error
        self.errors.push((name.to_string(), error));
    }

    pub fn clear_error(&mut self, name: &str) {
        self.errors.retain(|(n, _)| n != name);
    }

    pub fn clear_all_errors(&mut self) {
        self.errors.clear();
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn focused_field(&self) -> usize {
        self.focused_field
    }

    pub fn focus_next(&mut self) {
        if !self.fields.is_empty() {
            self.focused_field = (self.focused_field + 1) % self.fields.len();
        }
    }

    pub fn focus_previous(&mut self) {
        if !self.fields.is_empty() {
            if self.focused_field == 0 {
                self.focused_field = self.fields.len() - 1;
            } else {
                self.focused_field -= 1;
            }
        }
    }

    pub fn validate_field<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> ValidationResult,
    {
        if let Some(value) = self.get_field(name) {
            match validator(value) {
                Ok(()) => self.clear_error(name),
                Err(error) => self.set_error(name, error),
            }
        }
    }
}
