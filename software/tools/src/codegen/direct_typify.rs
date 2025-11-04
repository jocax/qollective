// ABOUTME: Direct typify integration for JSON Schema to Rust code generation
// ABOUTME: Simplified approach that uses typify directly without custom IR conversion

use super::integer_type_selection;
use crate::codegen::types::RustType;
use crate::schema::ir::Schema as IrSchema;
use schemars::schema::{RootSchema, Schema};
use serde_json::Value;
use std::fs;
use thiserror::Error;
use typify::{TypeSpace, TypeSpaceSettings};

/// Errors that can occur during direct typify code generation
#[derive(Debug, Error)]
pub enum DirectTypifyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Typify error: {0}")]
    Typify(#[from] typify::Error),

    #[error("Schema format error: {message}")]
    SchemaFormat { message: String },
}

/// Direct typify code generator
pub struct DirectTypifyGenerator {
    settings: TypeSpaceSettings,
}

impl DirectTypifyGenerator {
    /// Create a new generator with default settings optimized for enum generation
    pub fn new() -> Self {
        let mut settings = TypeSpaceSettings::default();
        settings
            .with_struct_builder(true)
            .with_derive("Clone".to_string())
            .with_derive("Debug".to_string())
            .with_derive("PartialEq".to_string());
        // Note: Typify automatically adds ::serde::Serialize and ::serde::Deserialize
        // so we don't need to add them here

        Self { settings }
    }

    /// Create a generator with custom settings
    pub fn with_settings(settings: TypeSpaceSettings) -> Self {
        Self { settings }
    }

    /// Generate Rust code from a JSON Schema file
    pub fn generate_from_file(&self, file_path: &str) -> Result<String, DirectTypifyError> {
        // Read the JSON schema file directly
        let schema_content = fs::read_to_string(file_path)?;
        let schema_json: Value = serde_json::from_str(&schema_content)?;

        self.generate_from_json(schema_json)
    }

    /// Generate Rust code from a JSON Value (schema)
    pub fn generate_from_json(&self, schema_json: Value) -> Result<String, DirectTypifyError> {
        // Preprocess the schema to add format hints for optimal integer types
        // For $defs-only schemas, this also adds a synthetic root type
        let preprocessed_schema = self.preprocess_schema_for_integer_types(schema_json)?;

        // Create TypeSpace with our settings
        let mut type_space = TypeSpace::new(&self.settings);

        // Convert JSON Value to schemars RootSchema by deserializing
        // This allows us to access both the root schema and definitions
        let root_schema: RootSchema =
            serde_json::from_value(preprocessed_schema.clone()).map_err(|e| {
                DirectTypifyError::SchemaFormat {
                    message: format!("Invalid schema format: {}", e),
                }
            })?;

        // First, add all definitions from $defs
        // This ensures all referenced types are available when processing the root
        if !root_schema.definitions.is_empty() {
            type_space.add_ref_types(root_schema.definitions)?;
        }

        // Then add the root schema if it has a type definition
        // For $defs-only schemas, we added a synthetic oneOf root
        if root_schema.schema.metadata.is_some()
            || root_schema.schema.instance_type.is_some()
            || root_schema.schema.subschemas.is_some()
        {
            type_space.add_type(&Schema::Object(root_schema.schema))?;
        }

        // Generate the code as token stream and convert to string
        let tokens = type_space.to_stream();
        Ok(tokens.to_string())
    }

    /// Preprocess schema to add JSON Schema format hints based on optimal integer type selection
    ///
    /// This function analyzes integer fields with min/max constraints and injects
    /// appropriate format hints (e.g., "uint8", "int32") that typify can use
    /// to generate optimal Rust types.
    ///
    /// Additionally, this function detects $defs-only schemas (schemas with no root type)
    /// and creates a synthetic root type that references all definitions, enabling typify
    /// to generate code for all types in $defs.
    fn preprocess_schema_for_integer_types(
        &self,
        mut schema_json: Value,
    ) -> Result<Value, DirectTypifyError> {
        // First, handle $defs-only schemas by creating a synthetic root
        if self.is_defs_only_schema(&schema_json) {
            self.create_synthetic_root(&mut schema_json);
        }

        // Then apply integer type optimization
        self.preprocess_value(&mut schema_json);
        Ok(schema_json)
    }

    /// Detect if schema has $defs but no root type definition
    ///
    /// A $defs-only schema has:
    /// - A $defs object with type definitions
    /// - No root type (no "type", "properties", "oneOf", "anyOf", or "allOf" at root)
    ///
    /// These schemas need a synthetic root to enable typify code generation.
    fn is_defs_only_schema(&self, schema: &Value) -> bool {
        // Must have $defs
        let has_defs = schema.get("$defs").is_some();

        // Must NOT have root type indicators
        let has_root_type = schema.get("type").is_some()
            || schema.get("properties").is_some()
            || schema.get("oneOf").is_some()
            || schema.get("anyOf").is_some()
            || schema.get("allOf").is_some();

        has_defs && !has_root_type
    }

    /// Create a synthetic root type that references all $defs definitions
    ///
    /// This enables typify to generate code for all types in a $defs-only schema.
    /// The synthetic root uses oneOf with references to all definitions.
    fn create_synthetic_root(&self, schema: &mut Value) {
        if let Some(defs) = schema.get("$defs") {
            if let Some(defs_obj) = defs.as_object() {
                // Create oneOf with references to all $defs
                let refs: Vec<Value> = defs_obj
                    .keys()
                    .map(|key| serde_json::json!({"$ref": format!("#/$defs/{}", key)}))
                    .collect();

                // Only create synthetic root if we have definitions
                if !refs.is_empty() {
                    schema
                        .as_object_mut()
                        .unwrap()
                        .insert("oneOf".to_string(), serde_json::json!(refs));
                }
            }
        }
    }

    /// Recursively preprocess a JSON value, adding format hints to integer types
    fn preprocess_value(&self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                // Check if this is an integer type with constraints
                if let Some(Value::String(type_str)) = map.get("type") {
                    if type_str == "integer" {
                        // Try to parse this as our IR schema to determine optimal type
                        if let Ok(ir_schema) =
                            serde_json::from_value::<IrSchema>(Value::Object(map.clone()))
                        {
                            let optimal_type =
                                integer_type_selection::select_optimal_integer_type(&ir_schema);
                            let format_hint = rust_type_to_format_hint(&optimal_type);

                            // Only add format hint if we don't already have one
                            if !map.contains_key("format") {
                                map.insert("format".to_string(), Value::String(format_hint));
                            }
                        }
                    }
                }

                // Recursively process nested objects
                for value in map.values_mut() {
                    self.preprocess_value(value);
                }
            }
            Value::Array(arr) => {
                // Recursively process array elements
                for item in arr.iter_mut() {
                    self.preprocess_value(item);
                }
            }
            _ => {}
        }
    }

    /// Generate Rust code from a JSON schema string
    pub fn generate_from_string(&self, schema_str: &str) -> Result<String, DirectTypifyError> {
        let schema_json: Value = serde_json::from_str(schema_str)?;
        self.generate_from_json(schema_json)
    }
}

impl Default for DirectTypifyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a RustType to a JSON Schema format hint
///
/// Maps Rust integer types to JSON Schema format strings that typify recognizes:
/// - u8 -> "uint8"
/// - u16 -> "uint16"
/// - u32 -> "uint32"
/// - u64 -> "uint64"
/// - i8 -> "int8"
/// - i16 -> "int16"
/// - i32 -> "int32"
/// - i64 -> "int64"
fn rust_type_to_format_hint(rust_type: &RustType) -> String {
    match rust_type {
        RustType::U8 => "uint8".to_string(),
        RustType::U16 => "uint16".to_string(),
        RustType::U32 => "uint32".to_string(),
        RustType::U64 => "uint64".to_string(),
        RustType::U128 => "uint64".to_string(), // typify doesn't support u128, use u64
        RustType::I8 => "int8".to_string(),
        RustType::I16 => "int16".to_string(),
        RustType::I32 => "int32".to_string(),
        RustType::I64 => "int64".to_string(),
        RustType::I128 => "int64".to_string(), // typify doesn't support i128, use i64
        _ => "int64".to_string(),              // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_simple_struct() {
        let generator = DirectTypifyGenerator::new();

        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Person",
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                },
                "age": {
                    "type": "integer"
                }
            },
            "required": ["name"]
        });

        let result = generator.generate_from_json(schema);
        assert!(result.is_ok(), "Should generate code successfully");

        let generated_code = result.unwrap();
        assert!(
            generated_code.contains("struct Person"),
            "Should generate Person struct"
        );
        assert!(generated_code.contains("name"), "Should have name field");
        assert!(generated_code.contains("age"), "Should have age field");
    }

    #[test]
    fn test_generate_string_enum() {
        let generator = DirectTypifyGenerator::new();

        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Status",
            "type": "string",
            "enum": ["active", "inactive", "pending"]
        });

        let result = generator.generate_from_json(schema);
        assert!(result.is_ok(), "Should generate enum successfully");

        let generated_code = result.unwrap();
        // typify should generate an enum for string enums
        assert!(
            generated_code.contains("enum") || generated_code.contains("Status"),
            "Should generate Status enum or type"
        );
    }

    #[test]
    fn test_generate_from_file() {
        let generator = DirectTypifyGenerator::new();

        let temp_dir = TempDir::new().unwrap();
        let schema_file = temp_dir.path().join("test_schema.json");

        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "TestStruct",
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "value": {"type": "number"}
            }
        });

        fs::write(&schema_file, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

        let result = generator.generate_from_file(schema_file.to_str().unwrap());
        assert!(result.is_ok(), "Should generate from file successfully");

        let generated_code = result.unwrap();
        assert!(
            generated_code.contains("TestStruct"),
            "Should generate TestStruct"
        );
    }
}
