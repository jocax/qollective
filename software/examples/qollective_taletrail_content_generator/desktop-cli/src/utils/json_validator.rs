/// JSON validation utilities
///
/// Provides real-time JSON validation with detailed error messages including line and column numbers

use serde_json::Value;

/// Validation result containing either the parsed JSON value or an error message
pub type ValidationResult = Result<Value, String>;

/// Validate JSON content and return parsed value or error
///
/// # Arguments
/// * `content` - JSON string to validate
///
/// # Returns
/// * `Ok(Value)` - Parsed JSON value if valid
/// * `Err(String)` - Formatted error message with line and column information
pub fn validate_json(content: &str) -> ValidationResult {
    match serde_json::from_str::<Value>(content) {
        Ok(value) => Ok(value),
        Err(e) => Err(format_json_error(&e)),
    }
}

/// Format a serde_json error into a human-readable message with line and column information
///
/// # Arguments
/// * `error` - serde_json error to format
///
/// # Returns
/// * `String` - Formatted error message
pub fn format_json_error(error: &serde_json::Error) -> String {
    let line = error.line();
    let column = error.column();

    // Get the error category and description
    let error_type = if error.is_eof() {
        "Unexpected end of file"
    } else if error.is_syntax() {
        "Syntax error"
    } else if error.is_data() {
        "Invalid data"
    } else if error.is_io() {
        "I/O error"
    } else {
        "Parse error"
    };

    format!(
        "{} at line {}, column {}: {}",
        error_type, line, column, error
    )
}

/// Pretty-print JSON with proper indentation
///
/// # Arguments
/// * `content` - JSON string to format
///
/// # Returns
/// * `Ok(String)` - Formatted JSON string
/// * `Err(String)` - Error message if JSON is invalid
pub fn pretty_print_json(content: &str) -> Result<String, String> {
    let value = validate_json(content)?;
    serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Failed to format JSON: {}", e))
}

/// Minify JSON by removing all whitespace
///
/// # Arguments
/// * `content` - JSON string to minify
///
/// # Returns
/// * `Ok(String)` - Minified JSON string
/// * `Err(String)` - Error message if JSON is invalid
pub fn minify_json(content: &str) -> Result<String, String> {
    let value = validate_json(content)?;
    serde_json::to_string(&value)
        .map_err(|e| format!("Failed to minify JSON: {}", e))
}

/// Check if a string is valid JSON
///
/// # Arguments
/// * `content` - String to validate
///
/// # Returns
/// * `bool` - True if content is valid JSON
pub fn is_valid_json(content: &str) -> bool {
    validate_json(content).is_ok()
}

/// Extract a specific field from JSON content
///
/// # Arguments
/// * `content` - JSON string
/// * `path` - Field path (e.g., "meta.request_id")
///
/// # Returns
/// * `Ok(Option<Value>)` - Field value if found, None if path doesn't exist
/// * `Err(String)` - Error message if JSON is invalid
pub fn extract_field(content: &str, path: &str) -> Result<Option<Value>, String> {
    let value = validate_json(content)?;

    let mut current = &value;
    for segment in path.split('.') {
        match current {
            Value::Object(map) => {
                match map.get(segment) {
                    Some(v) => current = v,
                    None => return Ok(None),
                }
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(current.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_json() {
        let json = r#"{"name": "test", "value": 123}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_json() {
        let json = r#"{"name": "test", invalid}"#;
        let result = validate_json(json);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("line"));
        assert!(error.contains("column"));
    }

    #[test]
    fn test_validate_empty_object() {
        let json = "{}";
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_array() {
        let json = "[]";
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid_json() {
        assert!(is_valid_json(r#"{"valid": true}"#));
        assert!(!is_valid_json(r#"{"invalid": }"#));
    }

    #[test]
    fn test_pretty_print_json() {
        let json = r#"{"name":"test","value":123}"#;
        let result = pretty_print_json(json);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains('\n'));
        assert!(formatted.contains("  "));
    }

    #[test]
    fn test_minify_json() {
        let json = r#"{
            "name": "test",
            "value": 123
        }"#;
        let result = minify_json(json);
        assert!(result.is_ok());
        let minified = result.unwrap();
        assert!(!minified.contains('\n'));
        assert!(!minified.contains("  "));
    }

    #[test]
    fn test_extract_field() {
        let json = r#"{"meta": {"request_id": "123", "tenant": "1"}}"#;

        let result = extract_field(json, "meta.request_id");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), serde_json::json!("123"));
    }

    #[test]
    fn test_extract_field_not_found() {
        let json = r#"{"meta": {"request_id": "123"}}"#;

        let result = extract_field(json, "meta.nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
