// ABOUTME: JSON Schema parser implementation for converting JSON Schema to internal AST
// ABOUTME: Handles complex schema features including references, composition, and validation

use super::ir::{Schema, SchemaType};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use url::Url;

/// Errors that can occur during schema parsing
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid schema format: {message}")]
    InvalidSchema { message: String },

    #[error("Schema reference error: {reference}")]
    ReferenceError { reference: String },

    #[error("Schema validation error: {message}")]
    ValidationError { message: String },

    #[error("Type conversion error: {message}")]
    TypeConversion { message: String },
}

/// JSON Schema parser
pub struct SchemaParser {
    base_uri: Option<Url>,
    resolved_refs: HashMap<String, Schema>,
}

impl SchemaParser {
    /// Create a new schema parser
    pub fn new() -> Self {
        Self {
            base_uri: None,
            resolved_refs: HashMap::new(),
        }
    }

    /// Create a new schema parser with a base URI for resolving references
    pub fn with_base_uri(base_uri: Url) -> Self {
        Self {
            base_uri: Some(base_uri),
            resolved_refs: HashMap::new(),
        }
    }

    /// Parse a JSON Schema from a file path
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Schema, SchemaError> {
        let content = std::fs::read_to_string(path)?;
        self.parse_string(&content)
    }

    /// Parse a JSON Schema from a string
    pub fn parse_string(&mut self, content: &str) -> Result<Schema, SchemaError> {
        let value: Value = serde_json::from_str(content)?;
        self.parse_value(&value)
    }

    /// Parse a JSON Schema from a serde_json::Value
    pub fn parse_value(&mut self, value: &Value) -> Result<Schema, SchemaError> {
        // First validate the JSON schema document using jsonschema crate
        self.validate_with_jsonschema(value)?;

        // Then parse into our custom AST
        self.parse_schema(value, None)
    }

    /// Validate JSON schema document using the jsonschema crate
    fn validate_with_jsonschema(&self, schema_json: &Value) -> Result<(), SchemaError> {
        use jsonschema::Validator;

        // Compile the schema to check for basic validity
        let _validator = Validator::new(schema_json).map_err(|e| SchemaError::ValidationError {
            message: format!("JSON Schema compilation failed: {}", e),
        })?;

        Ok(())
    }

    /// Internal method to parse a schema value recursively
    fn parse_schema(
        &mut self,
        value: &Value,
        parent_path: Option<&str>,
    ) -> Result<Schema, SchemaError> {
        let obj = value
            .as_object()
            .ok_or_else(|| SchemaError::InvalidSchema {
                message: "Schema must be an object".to_string(),
            })?;

        // Handle $ref first
        if let Some(ref_value) = obj.get("$ref") {
            let reference = ref_value
                .as_str()
                .ok_or_else(|| SchemaError::InvalidSchema {
                    message: "$ref must be a string".to_string(),
                })?;
            return Ok(Schema {
                schema_type: SchemaType::Reference(reference.to_string()),
                reference: Some(reference.to_string()),
                ..Default::default()
            });
        }

        let mut schema = Schema {
            schema_uri: obj
                .get("$schema")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            id: obj
                .get("$id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            title: obj
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: obj
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            version: obj
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            schema_type: self.parse_type(obj)?,
            required: self.parse_required(obj)?,
            enum_values: self.parse_enum(obj)?,
            const_value: obj.get("const").cloned(),
            format: obj
                .get("format")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            pattern: obj
                .get("pattern")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            minimum: obj.get("minimum").and_then(|v| v.as_f64()),
            maximum: obj.get("maximum").and_then(|v| v.as_f64()),
            exclusive_minimum: obj.get("exclusiveMinimum").and_then(|v| v.as_f64()),
            exclusive_maximum: obj.get("exclusiveMaximum").and_then(|v| v.as_f64()),
            multiple_of: obj.get("multipleOf").and_then(|v| v.as_f64()),
            min_length: obj.get("minLength").and_then(|v| v.as_u64()),
            max_length: obj.get("maxLength").and_then(|v| v.as_u64()),
            min_items: obj.get("minItems").and_then(|v| v.as_u64()),
            max_items: obj.get("maxItems").and_then(|v| v.as_u64()),
            unique_items: obj.get("uniqueItems").and_then(|v| v.as_bool()),
            min_properties: obj.get("minProperties").and_then(|v| v.as_u64()),
            max_properties: obj.get("maxProperties").and_then(|v| v.as_u64()),
            default: obj.get("default").cloned(),
            examples: self.parse_examples(obj)?,
            extensions: self.parse_extensions(obj)?,
            ..Default::default()
        };

        // Parse definitions
        if let Some(defs) = obj.get("definitions").or_else(|| obj.get("$defs")) {
            schema.definitions = self.parse_definitions(defs)?;
        }

        // Parse properties
        if let Some(props) = obj.get("properties") {
            schema.properties = self.parse_properties(props)?;
        }

        // Parse additionalProperties
        if let Some(additional) = obj.get("additionalProperties") {
            if additional.is_boolean() && additional.as_bool() == Some(false) {
                // additionalProperties: false
                schema.additional_properties = None;
            } else if additional.is_object() {
                schema.additional_properties =
                    Some(Box::new(self.parse_schema(additional, parent_path)?));
            }
        }

        // Parse items for arrays
        if let Some(items) = obj.get("items") {
            schema.items = Some(Box::new(self.parse_schema(items, parent_path)?));
        }

        // Parse composition keywords
        if let Some(all_of) = obj.get("allOf") {
            schema.all_of = self.parse_schema_array(all_of, parent_path)?;
        }
        if let Some(any_of) = obj.get("anyOf") {
            schema.any_of = self.parse_schema_array(any_of, parent_path)?;
        }
        if let Some(one_of) = obj.get("oneOf") {
            schema.one_of = self.parse_schema_array(one_of, parent_path)?;
        }
        if let Some(not) = obj.get("not") {
            schema.not = Some(Box::new(self.parse_schema(not, parent_path)?));
        }

        Ok(schema)
    }

    /// Parse the type field of a schema
    fn parse_type(&self, obj: &serde_json::Map<String, Value>) -> Result<SchemaType, SchemaError> {
        match obj.get("type") {
            Some(Value::String(type_str)) => SchemaType::from_str(type_str)
                .map_err(|e| SchemaError::TypeConversion { message: e }),
            Some(Value::Array(type_array)) => {
                let mut types = Vec::new();
                for type_val in type_array {
                    if let Some(type_str) = type_val.as_str() {
                        types.push(
                            SchemaType::from_str(type_str)
                                .map_err(|e| SchemaError::TypeConversion { message: e })?,
                        );
                    } else {
                        return Err(SchemaError::TypeConversion {
                            message: "Type array must contain strings".to_string(),
                        });
                    }
                }
                Ok(SchemaType::Union(types))
            }
            None => Ok(SchemaType::Any),
            _ => Err(SchemaError::TypeConversion {
                message: "Type must be a string or array of strings".to_string(),
            }),
        }
    }

    /// Parse required field
    fn parse_required(
        &self,
        obj: &serde_json::Map<String, Value>,
    ) -> Result<Vec<String>, SchemaError> {
        match obj.get("required") {
            Some(Value::Array(arr)) => arr
                .iter()
                .map(|v| {
                    v.as_str()
                        .ok_or_else(|| SchemaError::InvalidSchema {
                            message: "Required array must contain strings".to_string(),
                        })
                        .map(|s| s.to_string())
                })
                .collect(),
            None => Ok(Vec::new()),
            _ => Err(SchemaError::InvalidSchema {
                message: "Required must be an array".to_string(),
            }),
        }
    }

    /// Parse enum values
    fn parse_enum(&self, obj: &serde_json::Map<String, Value>) -> Result<Vec<Value>, SchemaError> {
        match obj.get("enum") {
            Some(Value::Array(arr)) => Ok(arr.clone()),
            None => Ok(Vec::new()),
            _ => Err(SchemaError::InvalidSchema {
                message: "Enum must be an array".to_string(),
            }),
        }
    }

    /// Parse examples
    fn parse_examples(
        &self,
        obj: &serde_json::Map<String, Value>,
    ) -> Result<Vec<Value>, SchemaError> {
        match obj.get("examples") {
            Some(Value::Array(arr)) => Ok(arr.clone()),
            None => Ok(Vec::new()),
            _ => Err(SchemaError::InvalidSchema {
                message: "Examples must be an array".to_string(),
            }),
        }
    }

    /// Parse extension properties (properties starting with x-)
    fn parse_extensions(
        &self,
        obj: &serde_json::Map<String, Value>,
    ) -> Result<HashMap<String, Value>, SchemaError> {
        let mut extensions = HashMap::new();
        for (key, value) in obj {
            if key.starts_with("x-") {
                extensions.insert(key.clone(), value.clone());
            }
        }
        Ok(extensions)
    }

    /// Parse definitions section
    fn parse_definitions(&mut self, value: &Value) -> Result<HashMap<String, Schema>, SchemaError> {
        let obj = value
            .as_object()
            .ok_or_else(|| SchemaError::InvalidSchema {
                message: "Definitions must be an object".to_string(),
            })?;

        let mut definitions = HashMap::new();
        for (name, schema_value) in obj {
            definitions.insert(name.clone(), self.parse_schema(schema_value, Some(name))?);
        }
        Ok(definitions)
    }

    /// Parse properties section
    fn parse_properties(&mut self, value: &Value) -> Result<HashMap<String, Schema>, SchemaError> {
        let obj = value
            .as_object()
            .ok_or_else(|| SchemaError::InvalidSchema {
                message: "Properties must be an object".to_string(),
            })?;

        let mut properties = HashMap::new();
        for (name, schema_value) in obj {
            properties.insert(name.clone(), self.parse_schema(schema_value, Some(name))?);
        }
        Ok(properties)
    }

    /// Parse an array of schemas (for allOf, anyOf, oneOf)
    fn parse_schema_array(
        &mut self,
        value: &Value,
        parent_path: Option<&str>,
    ) -> Result<Vec<Schema>, SchemaError> {
        let arr = value.as_array().ok_or_else(|| SchemaError::InvalidSchema {
            message: "Schema composition must be an array".to_string(),
        })?;

        arr.iter()
            .map(|v| self.parse_schema(v, parent_path))
            .collect()
    }

    /// Resolve a schema reference
    pub fn resolve_reference(
        &mut self,
        reference: &str,
        base_schema: &Schema,
    ) -> Result<Schema, SchemaError> {
        // Check if already resolved
        if let Some(resolved) = self.resolved_refs.get(reference) {
            return Ok(resolved.clone());
        }

        // Handle different types of references
        if reference.starts_with("#/") {
            // Local reference within the same document
            self.resolve_local_reference(reference, base_schema)
        } else if reference.starts_with("http://") || reference.starts_with("https://") {
            // Remote reference
            self.resolve_remote_reference(reference)
        } else {
            // Relative reference
            self.resolve_relative_reference(reference)
        }
    }

    /// Resolve a local reference (e.g., #/definitions/User)
    fn resolve_local_reference(
        &mut self,
        reference: &str,
        base_schema: &Schema,
    ) -> Result<Schema, SchemaError> {
        let path = reference.strip_prefix("#/").unwrap_or(reference);
        let parts: Vec<&str> = path.split('/').collect();

        if parts.is_empty() {
            return Err(SchemaError::ReferenceError {
                reference: reference.to_string(),
            });
        }

        match parts[0] {
            "definitions" => {
                if parts.len() != 2 {
                    return Err(SchemaError::ReferenceError {
                        reference: reference.to_string(),
                    });
                }
                let def_name = parts[1];
                base_schema
                    .definitions
                    .get(def_name)
                    .cloned()
                    .ok_or_else(|| SchemaError::ReferenceError {
                        reference: reference.to_string(),
                    })
            }
            "$defs" => {
                if parts.len() != 2 {
                    return Err(SchemaError::ReferenceError {
                        reference: reference.to_string(),
                    });
                }
                let def_name = parts[1];
                base_schema
                    .definitions
                    .get(def_name)
                    .cloned()
                    .ok_or_else(|| SchemaError::ReferenceError {
                        reference: reference.to_string(),
                    })
            }
            "properties" => {
                if parts.len() != 2 {
                    return Err(SchemaError::ReferenceError {
                        reference: reference.to_string(),
                    });
                }
                let prop_name = parts[1];
                base_schema
                    .properties
                    .get(prop_name)
                    .cloned()
                    .ok_or_else(|| SchemaError::ReferenceError {
                        reference: reference.to_string(),
                    })
            }
            _ => Err(SchemaError::ReferenceError {
                reference: reference.to_string(),
            }),
        }
    }

    /// Resolve a remote reference (HTTP/HTTPS URL)
    fn resolve_remote_reference(&mut self, _reference: &str) -> Result<Schema, SchemaError> {
        // For now, return an error as remote resolution requires HTTP client
        Err(SchemaError::ReferenceError {
            reference: "Remote references not yet implemented".to_string(),
        })
    }

    /// Resolve a relative reference
    fn resolve_relative_reference(&mut self, _reference: &str) -> Result<Schema, SchemaError> {
        // For now, return an error as relative resolution requires file system operations
        Err(SchemaError::ReferenceError {
            reference: "Relative references not yet implemented".to_string(),
        })
    }
}

impl Default for SchemaParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_string_schema() {
        let mut parser = SchemaParser::new();
        let schema_json = json!({
            "type": "string",
            "title": "Simple String",
            "description": "A simple string schema"
        });

        let schema = parser.parse_value(&schema_json).unwrap();

        assert_eq!(schema.schema_type, SchemaType::String);
        assert_eq!(schema.title, Some("Simple String".to_string()));
        assert_eq!(
            schema.description,
            Some("A simple string schema".to_string())
        );
    }

    #[test]
    fn test_parse_object_schema_with_properties() {
        let mut parser = SchemaParser::new();
        let schema_json = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                },
                "age": {
                    "type": "integer",
                    "minimum": 0
                }
            },
            "required": ["name"]
        });

        let schema = parser.parse_value(&schema_json).unwrap();

        assert_eq!(schema.schema_type, SchemaType::Object);
        assert_eq!(schema.properties.len(), 2);
        assert!(schema.properties.contains_key("name"));
        assert!(schema.properties.contains_key("age"));
        assert_eq!(schema.required, vec!["name"]);

        let age_prop = &schema.properties["age"];
        assert_eq!(age_prop.schema_type, SchemaType::Integer);
        assert_eq!(age_prop.minimum, Some(0.0));
    }

    #[test]
    fn test_parse_array_schema() {
        let mut parser = SchemaParser::new();
        let schema_json = json!({
            "type": "array",
            "items": {
                "type": "string"
            },
            "minItems": 1,
            "maxItems": 10
        });

        let schema = parser.parse_value(&schema_json).unwrap();

        assert_eq!(schema.schema_type, SchemaType::Array);
        assert_eq!(schema.min_items, Some(1));
        assert_eq!(schema.max_items, Some(10));

        let items = schema.items.as_ref().unwrap();
        assert_eq!(items.schema_type, SchemaType::String);
    }

    #[test]
    fn test_parse_reference_schema() {
        let mut parser = SchemaParser::new();
        let schema_json = json!({
            "type": "object",
            "properties": {
                "user": {
                    "$ref": "#/definitions/User"
                }
            },
            "definitions": {
                "User": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string"
                        }
                    }
                }
            }
        });

        let schema = parser.parse_value(&schema_json).unwrap();

        // Check that the reference property was parsed correctly
        let user_prop = &schema.properties["user"];
        assert_eq!(
            user_prop.schema_type,
            SchemaType::Reference("#/definitions/User".to_string())
        );
        assert_eq!(user_prop.reference, Some("#/definitions/User".to_string()));

        // Check that the definition was parsed
        assert!(schema.definitions.contains_key("User"));
    }

    #[test]
    fn test_parse_enum_schema() {
        let mut parser = SchemaParser::new();
        let schema_json = json!({
            "type": "string",
            "enum": ["red", "green", "blue"]
        });

        let schema = parser.parse_value(&schema_json).unwrap();

        assert_eq!(schema.schema_type, SchemaType::String);
        assert_eq!(schema.enum_values.len(), 3);
        assert_eq!(schema.enum_values[0], Value::String("red".to_string()));
    }

    #[test]
    fn test_parse_union_type() {
        let mut parser = SchemaParser::new();
        let schema_json = json!({
            "type": ["string", "null"]
        });

        let schema = parser.parse_value(&schema_json).unwrap();

        match schema.schema_type {
            SchemaType::Union(types) => {
                assert_eq!(types.len(), 2);
                assert!(types.contains(&SchemaType::String));
                assert!(types.contains(&SchemaType::Null));
            }
            _ => panic!("Expected union type"),
        }
    }
}
