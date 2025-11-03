use crate::{
    components::{create_help_content, get_menu_item, menu_item_count},
    state::{AppContext, View as AppView},
};

#[test]
fn test_menu_item_count() {
    assert_eq!(menu_item_count(), 7);
}

#[test]
fn test_get_menu_items() {
    // Test valid menu items
    let item1 = get_menu_item(0).unwrap();
    assert_eq!(item1.number, 1);
    assert_eq!(item1.label, "MCP Tester");
    assert_eq!(item1.view, Some(AppView::McpTester));

    let item2 = get_menu_item(1).unwrap();
    assert_eq!(item2.number, 2);
    assert_eq!(item2.label, "Trail Viewer");
    assert_eq!(item2.view, Some(AppView::TrailViewer));

    let item7 = get_menu_item(6).unwrap();
    assert_eq!(item7.number, 7);
    assert_eq!(item7.label, "Quit");
    assert_eq!(item7.view, None); // Quit has no view

    // Test invalid index
    assert!(get_menu_item(7).is_none());
    assert!(get_menu_item(100).is_none());
}

#[test]
fn test_help_content_generation() {
    let help = create_help_content();

    // Should have multiple lines
    assert!(help.len() > 10);

    // Should contain key sections
    let content = help.join("\n");
    assert!(content.contains("Keyboard Shortcuts"));
    assert!(content.contains("Global Shortcuts"));
    assert!(content.contains("Navigation Shortcuts"));
    assert!(content.contains("Menu Navigation"));
    assert!(content.contains("Ctrl+H"));
    assert!(content.contains("Ctrl+Q"));
    assert!(content.contains("ESC"));
}

#[test]
fn test_app_context_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let ctx = Arc::new(AppContext::new());

    // Clone context for multiple threads
    let ctx1 = Arc::clone(&ctx);
    let ctx2 = Arc::clone(&ctx);

    // Spawn threads that modify state
    let handle1 = thread::spawn(move || {
        for _ in 0..100 {
            ctx1.increment_requests();
        }
    });

    let handle2 = thread::spawn(move || {
        for _ in 0..100 {
            ctx2.increment_requests();
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    // Should have 200 total increments
    assert_eq!(ctx.active_requests(), 200);
}

// Task Group 6: Reusable UI Components Tests

mod ui_components {
    use crate::components::list::{ListState, calculate_viewport};
    use crate::components::table::ColumnDefinition;
    use crate::components::text_editor::TextEditorState;
    use crate::components::form::validate_required;

    #[test]
    fn test_list_navigation() {
        let mut state = ListState::new(vec!["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"]);

        // Initial state
        assert_eq!(state.selected(), 0);

        // Navigate down
        state.next();
        assert_eq!(state.selected(), 1);

        state.next();
        assert_eq!(state.selected(), 2);

        // Navigate up
        state.previous();
        assert_eq!(state.selected(), 1);

        // Navigate to end (wraps around)
        state.previous();
        state.previous();
        assert_eq!(state.selected(), 4); // Wraps to last item
    }

    #[test]
    fn test_list_viewport_calculation() {
        // Test viewport with 5 items visible, 10 total items
        let items = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];

        // Selected index 0 (first item)
        let (start, end) = calculate_viewport(0, items.len(), 5);
        assert_eq!(start, 0);
        assert_eq!(end, 5);

        // Selected index 3 (middle of viewport)
        let (start, end) = calculate_viewport(3, items.len(), 5);
        assert_eq!(start, 0);
        assert_eq!(end, 5);

        // Selected index 7 (should scroll)
        let (start, end) = calculate_viewport(7, items.len(), 5);
        assert_eq!(start, 5);
        assert_eq!(end, 10);

        // Selected index 9 (last item)
        let (start, end) = calculate_viewport(9, items.len(), 5);
        assert_eq!(start, 5);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_table_column_definitions() {
        let cols = vec![
            ColumnDefinition {
                title: "Name".to_string(),
                width: 20,
            },
            ColumnDefinition {
                title: "Age".to_string(),
                width: 5,
            },
            ColumnDefinition {
                title: "City".to_string(),
                width: 15,
            },
        ];

        // Test total width calculation
        let total_width: usize = cols.iter().map(|c| c.width).sum();
        assert_eq!(total_width, 40);

        // Test column titles
        assert_eq!(cols[0].title, "Name");
        assert_eq!(cols[1].title, "Age");
        assert_eq!(cols[2].title, "City");
    }

    #[test]
    fn test_text_editor_basic_operations() {
        let mut editor = TextEditorState::new("Hello\nWorld".to_string());

        // Initial state
        assert_eq!(editor.line_count(), 2);
        assert_eq!(editor.cursor_line(), 0);
        assert_eq!(editor.cursor_column(), 0);

        // Move cursor
        editor.move_cursor_right();
        assert_eq!(editor.cursor_column(), 1);

        editor.move_cursor_down();
        assert_eq!(editor.cursor_line(), 1);
        // Cursor column should be preserved when moving down
        assert_eq!(editor.cursor_column(), 1);

        // Insert character at column 1 (between 'W' and 'o')
        editor.insert_char('!');
        assert_eq!(editor.get_line(1).unwrap(), "W!orld");
        assert_eq!(editor.cursor_column(), 2); // cursor moves to after the inserted char

        // Delete character at column 2 (the 'o' after cursor)
        editor.delete_char();
        assert_eq!(editor.get_line(1).unwrap(), "W!rld");
    }

    #[test]
    fn test_text_editor_multiline_content() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4";
        let editor = TextEditorState::new(content.to_string());

        assert_eq!(editor.line_count(), 4);
        assert_eq!(editor.get_line(0).unwrap(), "Line 1");
        assert_eq!(editor.get_line(3).unwrap(), "Line 4");
        assert!(editor.get_line(4).is_none());
    }

    #[test]
    fn test_form_validation() {
        // Test required field validation
        assert!(validate_required("").is_err());
        assert!(validate_required("   ").is_err());
        assert!(validate_required("valid").is_ok());

        // Test validation error message
        let result = validate_required("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("required"));
    }
}
