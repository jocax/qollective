// ABOUTME: Direct typify integration for JSON Schema to Rust code generation
// ABOUTME: Simplified approach that uses typify directly without custom IR conversion

use schemars::schema::Schema;
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
            .with_derive("PartialEq".to_string())
            .with_derive("Serialize".to_string())
            .with_derive("Deserialize".to_string());

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
        // Create TypeSpace with our settings
        let mut type_space = TypeSpace::new(&self.settings);
        
        // Convert JSON Value to schemars Schema by deserializing
        let schema: Schema = serde_json::from_value(schema_json)
            .map_err(|e| DirectTypifyError::SchemaFormat { 
                message: format!("Invalid schema format: {}", e)
            })?;
        
        // Add the schema to typify
        type_space.add_type(&schema)?;
        
        // Generate the code as token stream and convert to string
        let tokens = type_space.to_stream();
        Ok(tokens.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;
    use std::fs;

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
        assert!(generated_code.contains("struct Person"), "Should generate Person struct");
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
        assert!(generated_code.contains("enum") || generated_code.contains("Status"), "Should generate Status enum or type");
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
        assert!(generated_code.contains("TestStruct"), "Should generate TestStruct");
    }
}