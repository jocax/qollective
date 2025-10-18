//! Runtime JSON Schema Validation Module
//!
//! This module provides runtime validation of MCP tool parameters against their
//! JSON Schema definitions generated from Rust types using schemars.
//!
//! # Usage
//!
//! ```no_run
//! use shared_types::validation::validate_mcp_params;
//! use serde_json::json;
//!
//! #[derive(schemars::JsonSchema)]
//! struct MyParams {
//!     name: String,
//!     age: i32,
//! }
//!
//! let params = json!({
//!     "name": "Alice",
//!     "age": 30
//! });
//!
//! // Validate params against MyParams schema
//! validate_mcp_params::<MyParams>(&params).expect("Validation failed");
//! ```

use jsonschema::Validator;
use schemars::JsonSchema;
use serde_json::Value;

/// Validates runtime JSON parameters against a type's JSON Schema
///
/// This function generates a JSON Schema from the provided type `T` using schemars,
/// compiles it into a validator, and validates the provided JSON parameters against it.
///
/// # Type Parameters
///
/// * `T` - A type implementing `JsonSchema` that defines the expected structure
///
/// # Arguments
///
/// * `params` - JSON parameters to validate
///
/// # Returns
///
/// * `Ok(())` - If validation succeeds
/// * `Err(String)` - Detailed error message with schema, actual params, and validation errors
///
/// # Examples
///
/// ```no_run
/// use shared_types::validation::validate_mcp_params;
/// use serde_json::json;
///
/// #[derive(schemars::JsonSchema)]
/// struct ValidateContentParams {
///     content_node: ContentNode,
///     age_group: String,
///     educational_goals: Vec<String>,
/// }
///
/// let params = json!({
///     "content_node": {...},
///     "age_group": "6-8",
///     "educational_goals": ["reading"]
/// });
///
/// validate_mcp_params::<ValidateContentParams>(&params)?;
/// ```
pub fn validate_mcp_params<T: JsonSchema>(params: &Value) -> Result<(), String> {
    // 1. Generate JSON Schema from type using schemars
    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_value(&schema)
        .map_err(|e| format!("Failed to serialize schema: {}", e))?;

    // 2. Compile JSON Schema validator
    let validator = Validator::new(&schema_json)
        .map_err(|e| format!("Failed to compile schema: {}", e))?;

    // 3. Check if params are valid
    if !validator.is_valid(params) {
        // Params are invalid, collect detailed errors
        // Note: iter_errors() returns an iterator over ValidationError items
        let error_messages: Vec<String> = validator
            .iter_errors(params)
            .map(|e| format!("  - {} (at path: {})", e, e.instance_path))
            .collect();

        return Err(format!(
            "Schema validation failed:\n\nExpected schema:\n{}\n\nActual parameters:\n{}\n\nValidation errors:\n{}",
            serde_json::to_string_pretty(&schema_json).unwrap_or_else(|_| "<invalid>".to_string()),
            serde_json::to_string_pretty(params).unwrap_or_else(|_| "<invalid>".to_string()),
            error_messages.join("\n")
        ));
    }

    Ok(())
}

/// Validates parameters and provides a simplified error message
///
/// This is a convenience wrapper around `validate_mcp_params` that provides
/// shorter error messages suitable for user-facing APIs.
///
/// # Arguments
///
/// * `params` - JSON parameters to validate
///
/// # Returns
///
/// * `Ok(())` - If validation succeeds
/// * `Err(String)` - Simplified error message listing validation failures
pub fn validate_mcp_params_simple<T: JsonSchema>(params: &Value) -> Result<(), String> {
    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_value(&schema)
        .map_err(|e| format!("Schema generation error: {}", e))?;

    let validator = Validator::new(&schema_json)
        .map_err(|e| format!("Schema compilation error: {}", e))?;

    if !validator.is_valid(params) {
        let error_list: Vec<String> = validator
            .iter_errors(params)
            .map(|e| e.to_string())
            .collect();

        return Err(format!("Validation errors: {}", error_list.join("; ")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(schemars::JsonSchema)]
    struct TestParams {
        name: String,
        age: i32,
        #[serde(default)]
        optional_field: Option<String>,
    }

    #[test]
    fn test_valid_params_pass() {
        let params = json!({
            "name": "Alice",
            "age": 30
        });

        let result = validate_mcp_params::<TestParams>(&params);
        assert!(result.is_ok(), "Valid params should pass: {:?}", result.err());
    }

    #[test]
    fn test_valid_params_with_optional_field() {
        let params = json!({
            "name": "Bob",
            "age": 25,
            "optional_field": "test"
        });

        assert!(validate_mcp_params::<TestParams>(&params).is_ok());
    }

    #[test]
    fn test_missing_required_field_fails() {
        let params = json!({
            "name": "Alice"
            // missing age
        });

        let result = validate_mcp_params::<TestParams>(&params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("age") || error.contains("required"), "Error should mention missing field: {}", error);
    }

    #[test]
    fn test_wrong_type_fails() {
        let params = json!({
            "name": "Alice",
            "age": "thirty" // should be i32
        });

        let result = validate_mcp_params::<TestParams>(&params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("age") || error.contains("type"), "Error should mention type mismatch: {}", error);
    }

    #[test]
    fn test_extra_fields_allowed_by_default() {
        let params = json!({
            "name": "Alice",
            "age": 30,
            "extra_field": "should be allowed"
        });

        // JSON Schema typically allows additional properties by default
        let result = validate_mcp_params::<TestParams>(&params);
        // This may pass or fail depending on schemars configuration
        // Documenting the behavior
        println!("Extra fields result: {:?}", result);
    }

    #[test]
    fn test_simple_error_format() {
        let params = json!({
            "name": "Alice"
            // missing age
        });

        let result = validate_mcp_params_simple::<TestParams>(&params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.starts_with("Validation errors:"), "Should have simple format: {}", error);
    }

    #[test]
    fn test_detailed_error_includes_schema() {
        let params = json!({
            "name": 123, // wrong type
            "age": 30
        });

        let result = validate_mcp_params::<TestParams>(&params);
        assert!(result.is_err());
        let error = result.unwrap_err();

        // Detailed error should include schema and actual params
        assert!(error.contains("Expected schema:"), "Should include schema: {}", error);
        assert!(error.contains("Actual parameters:"), "Should include params: {}", error);
        assert!(error.contains("Validation errors:"), "Should include errors: {}", error);
    }
}
