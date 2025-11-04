// ABOUTME: Intermediate representation types for JSON Schema AST
// ABOUTME: Defines strongly-typed structures for representing parsed JSON Schema elements

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSON Schema AST representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub schema_uri: Option<String>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub schema_type: SchemaType,
    pub definitions: HashMap<String, Schema>,
    pub properties: HashMap<String, Schema>,
    pub required: Vec<String>,
    pub additional_properties: Option<Box<Schema>>,
    pub items: Option<Box<Schema>>,
    pub all_of: Vec<Schema>,
    pub any_of: Vec<Schema>,
    pub one_of: Vec<Schema>,
    pub not: Option<Box<Schema>>,
    pub reference: Option<String>,
    pub enum_values: Vec<Value>,
    pub const_value: Option<Value>,
    pub format: Option<String>,
    pub pattern: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub exclusive_minimum: Option<f64>,
    pub exclusive_maximum: Option<f64>,
    pub multiple_of: Option<f64>,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
    pub unique_items: Option<bool>,
    pub min_properties: Option<u64>,
    pub max_properties: Option<u64>,
    pub default: Option<Value>,
    pub examples: Vec<Value>,
    pub extensions: HashMap<String, Value>,
}

/// JSON Schema primitive types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
    Union(Vec<SchemaType>),
    Reference(String),
    Any,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            schema_uri: None,
            id: None,
            title: None,
            description: None,
            version: None,
            schema_type: SchemaType::Any,
            definitions: HashMap::new(),
            properties: HashMap::new(),
            required: Vec::new(),
            additional_properties: None,
            items: None,
            all_of: Vec::new(),
            any_of: Vec::new(),
            one_of: Vec::new(),
            not: None,
            reference: None,
            enum_values: Vec::new(),
            const_value: None,
            format: None,
            pattern: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            multiple_of: None,
            min_length: None,
            max_length: None,
            min_items: None,
            max_items: None,
            unique_items: None,
            min_properties: None,
            max_properties: None,
            default: None,
            examples: Vec::new(),
            extensions: HashMap::new(),
        }
    }
}

impl Schema {
    /// Create a new schema with the given type
    pub fn new(schema_type: SchemaType) -> Self {
        Self {
            schema_type,
            ..Default::default()
        }
    }

    /// Check if this schema is a reference
    pub fn is_reference(&self) -> bool {
        matches!(self.schema_type, SchemaType::Reference(_)) || self.reference.is_some()
    }

    /// Get the reference string if this is a reference schema
    pub fn get_reference(&self) -> Option<&str> {
        self.reference.as_deref().or_else(|| {
            if let SchemaType::Reference(ref r) = self.schema_type {
                Some(r)
            } else {
                None
            }
        })
    }

    /// Check if this schema represents an object type
    pub fn is_object(&self) -> bool {
        matches!(self.schema_type, SchemaType::Object)
    }

    /// Check if this schema represents an array type
    pub fn is_array(&self) -> bool {
        matches!(self.schema_type, SchemaType::Array)
    }

    /// Check if this schema represents a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self.schema_type,
            SchemaType::Null
                | SchemaType::Boolean
                | SchemaType::Integer
                | SchemaType::Number
                | SchemaType::String
        )
    }

    /// Check if this schema has enum values
    pub fn is_enum(&self) -> bool {
        !self.enum_values.is_empty()
    }

    /// Check if this schema is a union type
    pub fn is_union(&self) -> bool {
        matches!(self.schema_type, SchemaType::Union(_))
    }

    /// Get union types if this is a union
    pub fn get_union_types(&self) -> Option<&[SchemaType]> {
        if let SchemaType::Union(ref types) = self.schema_type {
            Some(types)
        } else {
            None
        }
    }
}

impl SchemaType {
    /// Convert a string type name to SchemaType
    pub fn from_str(type_name: &str) -> Result<Self, String> {
        match type_name {
            "null" => Ok(SchemaType::Null),
            "boolean" => Ok(SchemaType::Boolean),
            "integer" => Ok(SchemaType::Integer),
            "number" => Ok(SchemaType::Number),
            "string" => Ok(SchemaType::String),
            "array" => Ok(SchemaType::Array),
            "object" => Ok(SchemaType::Object),
            _ => Err(format!("Unknown type: {}", type_name)),
        }
    }

    /// Convert SchemaType to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SchemaType::Null => "null",
            SchemaType::Boolean => "boolean",
            SchemaType::Integer => "integer",
            SchemaType::Number => "number",
            SchemaType::String => "string",
            SchemaType::Array => "array",
            SchemaType::Object => "object",
            SchemaType::Union(_) => "union",
            SchemaType::Reference(_) => "reference",
            SchemaType::Any => "any",
        }
    }

    /// Check if this type is a primitive JSON type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            SchemaType::Null
                | SchemaType::Boolean
                | SchemaType::Integer
                | SchemaType::Number
                | SchemaType::String
        )
    }
}
