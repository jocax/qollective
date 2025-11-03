# Iocraft v0.7 API Guide

Quick reference for working with Iocraft v0.7 in the TaleTrail Desktop CLI project.

## Props Definition

### Basic Props

```rust
use iocraft::prelude::*;

#[derive(Props)]
pub struct MyComponentProps {
    pub title: String,
    pub count: usize,
    pub is_active: bool,
}

impl Default for MyComponentProps {
    fn default() -> Self {
        Self {
            title: String::new(),
            count: 0,
            is_active: false,
        }
    }
}
```

### Props with Optional Fields

```rust
#[derive(Props)]
pub struct ComponentProps {
    pub label: String,
    pub value: String,
    pub error: Option<String>,      // Optional field
    pub placeholder: Option<String>, // Optional field
}

impl Default for ComponentProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            value: String::new(),
            error: None,
            placeholder: None,
        }
    }
}
```

### Generic Props

```rust
#[derive(Props)]
pub struct ListProps<T: Clone + Send + Sync + 'static> {
    pub items: Vec<T>,
    pub selected_index: usize,
    pub render_item: Option<fn(&T, bool) -> String>,
}

impl<T: Clone + Send + Sync + 'static> Default for ListProps<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            render_item: None,
        }
    }
}
```

**Important**: Generic types MUST have `Clone + Send + Sync + 'static` bounds.

## Component Definition

### Basic Component

```rust
#[component]
pub fn MyComponent(_hooks: Hooks, props: &MyComponentProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
        ) {
            Text(content: &props.title, color: Color::White)
        }
    }
}
```

### Component with State

```rust
#[component]
pub fn StatefulComponent(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let counter = hooks.use_state(|| 0usize);

    element! {
        View {
            Text(content: format!("Count: {}", counter.get()), color: Color::White)
        }
    }
}
```

### Generic Component

```rust
#[component]
pub fn GenericList<T: Clone + Send + Sync + 'static>(
    _hooks: Hooks,
    props: &ListProps<T>,
) -> impl Into<AnyElement<'static>> {
    let items_vec: Vec<AnyElement> = props.items
        .iter()
        .map(|item| {
            element! {
                Text(content: format!("{:?}", item), color: Color::White)
            }
            .into_any()
        })
        .collect();

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(items_vec.into_iter())
        }
    }
}
```

## Common Patterns

### Building Dynamic Element Lists

```rust
let mut elements: Vec<AnyElement> = Vec::new();

// Add elements
elements.push(
    element! {
        Text(content: "Header", color: Color::Cyan)
    }
    .into_any()
);

// Conditional element
if show_footer {
    elements.push(
        element! {
            Text(content: "Footer", color: Color::DarkGrey)
        }
        .into_any()
    );
}

// Render all elements
element! {
    View(flex_direction: FlexDirection::Column) {
        #(elements.into_iter())
    }
}
```

### Conditional Rendering

```rust
let content: AnyElement = if condition {
    element! {
        View { Text(content: "True branch") }
    }
    .into_any()
} else {
    element! {
        View { Text(content: "False branch") }
    }
    .into_any()
};

element! {
    View { #(content) }
}
```

### Iterating Over Collections

```rust
let items: Vec<AnyElement> = props.rows
    .iter()
    .enumerate()
    .map(|(idx, row)| {
        let is_selected = Some(idx) == props.selected_row;
        let color = if is_selected { Color::Cyan } else { Color::White };

        element! {
            View {
                Text(content: row, color: color)
            }
        }
        .into_any()
    })
    .collect();

element! {
    View(flex_direction: FlexDirection::Column) {
        #(items.into_iter())
    }
}
```

## Styling

### Colors

```rust
use iocraft::prelude::Color;

// Standard colors
Color::Black
Color::White
Color::DarkGrey
Color::Cyan
Color::Red
Color::Green
Color::Yellow
Color::Blue
Color::Magenta
```

### Border Styles

```rust
use iocraft::prelude::BorderStyle;

BorderStyle::Single    // ┌─┐│└┘
BorderStyle::Double    // ╔═╗║╚╝
BorderStyle::Rounded   // ╭─╮│╰╯
BorderStyle::Thick     // ┏━┓┃┗┛
```

### Layout

```rust
element! {
    View(
        flex_direction: FlexDirection::Column,  // or Row
        padding: 1,
        margin_top: 1,
        margin_bottom: 1,
        border_style: BorderStyle::Single,
        border_color: Color::Cyan,
        background_color: Some(Color::Black),
    ) {
        // children
    }
}
```

### Text Styling

```rust
element! {
    Text(
        content: "Styled text",
        color: Color::Cyan,
        weight: Weight::Bold,  // or Normal
    )
}
```

## State Management

### use_state Hook

```rust
#[component]
pub fn Counter(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let count = hooks.use_state(|| 0usize);

    // Read value
    let value = count.get();

    // Update value (in event handler)
    // count.set(value + 1);

    element! {
        Text(content: format!("Count: {}", value))
    }
}
```

### use_ref Hook

```rust
#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let app_ctx = hooks.use_ref(|| AppContext::new());

    // Read reference
    let current_view = app_ctx.read().current_view();

    // Write (in event handler)
    // app_ctx.write().set_view(View::Settings);

    element! {
        Text(content: format!("View: {:?}", current_view))
    }
}
```

## Common Mistakes to Avoid

### ❌ DON'T: Use #[prop] attributes

```rust
// WRONG
#[derive(Props, Default)]
pub struct MyProps {
    #[prop(default = "hello")]  // ❌ Not supported in v0.7
    pub message: String,
}
```

### ✅ DO: Implement Default manually

```rust
// CORRECT
#[derive(Props)]
pub struct MyProps {
    pub message: String,
}

impl Default for MyProps {
    fn default() -> Self {
        Self {
            message: "hello".to_string(),
        }
    }
}
```

### ❌ DON'T: Forget Hooks parameter

```rust
// WRONG
#[component]
pub fn MyComponent(props: &MyProps) -> impl Into<AnyElement<'static>> {
    // ❌ Missing Hooks parameter
}
```

### ✅ DO: Always include Hooks parameter

```rust
// CORRECT
#[component]
pub fn MyComponent(_hooks: Hooks, props: &MyProps) -> impl Into<AnyElement<'static>> {
    // ✅ Even if unused, prefix with underscore
}
```

### ❌ DON'T: Forget generic bounds

```rust
// WRONG
#[derive(Props)]
pub struct ListProps<T: Clone> {  // ❌ Missing Send + Sync + 'static
    pub items: Vec<T>,
}
```

### ✅ DO: Include all required bounds

```rust
// CORRECT
#[derive(Props)]
pub struct ListProps<T: Clone + Send + Sync + 'static> {
    pub items: Vec<T>,
}
```

## Troubleshooting

### "cannot find attribute `prop` in this scope"

**Problem**: Using `#[prop(...)]` attributes on struct fields.

**Solution**: Remove the attributes and implement `Default` manually.

### "the trait bound `T: Send` is not satisfied"

**Problem**: Generic type missing `Send + Sync + 'static` bounds.

**Solution**: Add bounds to the generic parameter: `T: Clone + Send + Sync + 'static`

### "this function takes 2 parameters but 1 parameter was supplied"

**Problem**: Component missing `Hooks` parameter.

**Solution**: Add `_hooks: Hooks` (or `mut hooks: Hooks` if using hooks) as first parameter.

## Testing Components

### Testing Component Logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_state() {
        let mut state = ComponentState::new();

        // Test state operations
        state.next();
        assert_eq!(state.selected(), 1);

        state.previous();
        assert_eq!(state.selected(), 0);
    }
}
```

### Testing Helper Functions

```rust
#[test]
fn test_viewport_calculation() {
    let (start, end) = calculate_viewport(5, 10, 5);
    assert_eq!(start, 5);
    assert_eq!(end, 10);
}
```

## References

- [Iocraft GitHub](https://github.com/ccbrown/iocraft)
- [Iocraft Examples](https://github.com/ccbrown/iocraft/tree/main/examples)
- Project Components: `desktop-cli/src/components/`

---

Generated: 2025-11-02
Version: Iocraft v0.7
