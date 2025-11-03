/// JSON Request Editor Component
///
/// Multi-line text editor for editing MCP request JSON with real-time validation

use crate::components::text_editor::{TextEditor, TextEditorState};
use crate::state::McpContext;
use crate::utils::json_validator;
use iocraft::prelude::*;

/// Props for RequestEditor component
#[derive(Props)]
pub struct RequestEditorProps {
    pub mcp_context: McpContext,
    pub editor_state: TextEditorState,
}

impl Default for RequestEditorProps {
    fn default() -> Self {
        Self {
            mcp_context: McpContext::new(),
            editor_state: TextEditorState::new(String::new()),
        }
    }
}

/// JSON request editor with validation
#[component]
pub fn RequestEditor(
    _hooks: Hooks,
    props: &RequestEditorProps,
) -> impl Into<AnyElement<'static>> {
    let json_content = props.mcp_context.request_json();
    let json_error = props.mcp_context.json_error();

    // Validate JSON and show error if invalid
    let validation_message = match json_validator::validate_json(&json_content) {
        Ok(_) => {
            if json_content.trim().is_empty() {
                Some("No request loaded. Select a template or paste JSON.".to_string())
            } else {
                Some("✓ Valid JSON".to_string())
            }
        }
        Err(error) => Some(format!("✗ {}", error)),
    };

    let is_valid = json_error.is_none() && !json_content.trim().is_empty();
    let validation_color = if json_content.trim().is_empty() {
        Color::Grey
    } else if is_valid {
        Color::Green
    } else {
        Color::Red
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: "Request Editor",
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
            TextEditor(
                content: json_content.clone(),
                cursor_line: props.editor_state.cursor_line(),
                cursor_column: props.editor_state.cursor_column(),
                visible_rows: 20usize,
                show_line_numbers: true,
                error_message: json_error.clone(),
            )
            View(margin_top: 1) {
                Text(
                    content: validation_message.unwrap_or_default(),
                    color: validation_color,
                )
            }
            View(margin_top: 1) {
                Text(
                    content: "Ctrl+Enter: Send | Ctrl+S: Save | Ctrl+K: Clear | Ctrl+P: Pretty Print",
                    color: Color::Grey,
                )
            }
        }
    }
    .into_any()
}

/// Utility functions for editor operations
pub mod editor_ops {
    use crate::utils::json_validator;

    /// Validate JSON content and return error message if invalid
    pub fn validate_request_json(content: &str) -> Option<String> {
        if content.trim().is_empty() {
            return None;
        }

        match json_validator::validate_json(content) {
            Ok(_) => None,
            Err(error) => Some(error),
        }
    }

    /// Pretty-print JSON content
    pub fn pretty_print_json(content: &str) -> Result<String, String> {
        json_validator::pretty_print_json(content)
    }

    /// Minify JSON content
    pub fn minify_json(content: &str) -> Result<String, String> {
        json_validator::minify_json(content)
    }

    /// Check if content is valid JSON
    pub fn is_valid_json(content: &str) -> bool {
        json_validator::is_valid_json(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::json_validator;

    #[test]
    fn test_request_editor_json_validation() {
        let ctx = McpContext::new();

        // Test valid JSON
        let valid_json = r#"{"meta": {"tenant": "1"}, "payload": {"tool_call": {}}}"#;
        ctx.set_request_json(valid_json.to_string());

        let result = json_validator::validate_json(&ctx.request_json());
        assert!(result.is_ok());

        // Test invalid JSON
        let invalid_json = r#"{"meta": "invalid"#;
        ctx.set_request_json(invalid_json.to_string());

        let result = json_validator::validate_json(&ctx.request_json());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_request_json() {
        use super::editor_ops::validate_request_json;

        // Empty content returns None
        assert_eq!(validate_request_json(""), None);
        assert_eq!(validate_request_json("   "), None);

        // Valid JSON returns None
        let valid = r#"{"test": "value"}"#;
        assert_eq!(validate_request_json(valid), None);

        // Invalid JSON returns error
        let invalid = r#"{"test": invalid}"#;
        let error = validate_request_json(invalid);
        assert!(error.is_some());
        let error_msg = error.unwrap();
        assert!(error_msg.contains("error") || error_msg.contains("Parse") || error_msg.contains("Syntax"));
    }

    #[test]
    fn test_pretty_print_json() {
        use super::editor_ops::pretty_print_json;

        let compact = r#"{"name":"test","value":123}"#;
        let result = pretty_print_json(compact);

        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains('\n'));
        assert!(formatted.contains("  ")); // Contains indentation
    }

    #[test]
    fn test_minify_json() {
        use super::editor_ops::minify_json;

        let formatted = r#"{
            "name": "test",
            "value": 123
        }"#;

        let result = minify_json(formatted);
        assert!(result.is_ok());

        let minified = result.unwrap();
        assert!(!minified.contains('\n'));
        assert!(!minified.contains("  "));
    }

    #[test]
    fn test_is_valid_json() {
        use super::editor_ops::is_valid_json;

        assert!(is_valid_json(r#"{"valid": true}"#));
        assert!(is_valid_json(r#"[]"#));
        assert!(is_valid_json(r#"{}"#));
        assert!(is_valid_json(r#"null"#));
        assert!(is_valid_json(r#"123"#));
        assert!(is_valid_json(r#""string""#));

        assert!(!is_valid_json(r#"{"invalid": }"#));
        assert!(!is_valid_json(r#"invalid"#));
        assert!(!is_valid_json(r#""unterminated"#));
    }

    #[test]
    fn test_editor_state_integration() {
        let ctx = McpContext::new();
        let json = r#"{"test": "value"}"#;

        ctx.set_request_json(json.to_string());
        assert_eq!(ctx.request_json(), json);

        // Test error state
        ctx.set_json_error(Some("Test error".to_string()));
        assert_eq!(ctx.json_error(), Some("Test error".to_string()));

        ctx.set_json_error(None);
        assert_eq!(ctx.json_error(), None);
    }

    #[test]
    fn test_editor_ops_with_complex_json() {
        use super::editor_ops::*;

        let complex_json = r#"{
            "meta": {
                "tenant": "1",
                "request_id": "123",
                "timestamp": "2025-11-02T12:00:00Z"
            },
            "payload": {
                "tool_call": {
                    "params": {
                        "name": "test_tool",
                        "arguments": {
                            "key": "value",
                            "nested": {
                                "array": [1, 2, 3]
                            }
                        }
                    }
                }
            }
        }"#;

        // Validate complex JSON
        assert!(is_valid_json(complex_json));

        // Pretty print should work
        let pretty = pretty_print_json(complex_json);
        assert!(pretty.is_ok());

        // Minify should work
        let minified = minify_json(complex_json);
        assert!(minified.is_ok());

        // Minified should still be valid
        assert!(is_valid_json(&minified.unwrap()));
    }
}
