// ABOUTME: Schema validation functionality for ensuring schema correctness
// ABOUTME: Validates schemas against JSON Schema specification rules and conventions

use super::ir::{Schema, SchemaType};
use super::parser::SchemaError;
use serde_json::Value;

/// Schema validator for checking schema correctness
pub struct SchemaValidator;

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a raw JSON schema document against the JSON Schema specification
    pub fn validate_json_schema(&self, schema_json: &Value) -> Result<(), SchemaError> {
        // Try to compile the schema with format validation enabled
        let _validator = jsonschema::options()
            .should_validate_formats(true)
            .build(schema_json)
            .map_err(|e| SchemaError::ValidationError {
                message: format!("Schema compilation failed: {}", e),
            })?;

        Ok(())
    }

    /// Validate that the schema follows JSON Schema specification
    pub fn validate_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        // Basic validation rules

        // If it's a reference, it should only have the reference field
        if schema.reference.is_some()
            && (!schema.properties.is_empty() || !schema.definitions.is_empty())
        {
            return Err(SchemaError::ValidationError {
                message: "Schema with $ref should not have other properties".to_string(),
            });
        }

        // Object type should have properties or additionalProperties
        if matches!(schema.schema_type, SchemaType::Object) {
            if schema.properties.is_empty() && schema.additional_properties.is_none() {
                // This is allowed but might be worth warning about
            }
        }

        // Array type should have items
        if matches!(schema.schema_type, SchemaType::Array) && schema.items.is_none() {
            // This is allowed (any items) but might be worth warning about
        }

        // Validate required fields are in properties (root level only)
        self.validate_required_fields(schema, &schema.required)?;

        // Validate numeric constraints
        if let (Some(min), Some(max)) = (schema.minimum, schema.maximum) {
            if min > max {
                return Err(SchemaError::ValidationError {
                    message: "Minimum value cannot be greater than maximum value".to_string(),
                });
            }
        }

        if let (Some(min), Some(max)) = (schema.exclusive_minimum, schema.exclusive_maximum) {
            if min >= max {
                return Err(SchemaError::ValidationError {
                    message: "Exclusive minimum value must be less than exclusive maximum value"
                        .to_string(),
                });
            }
        }

        // Validate string constraints
        if let (Some(min_len), Some(max_len)) = (schema.min_length, schema.max_length) {
            if min_len > max_len {
                return Err(SchemaError::ValidationError {
                    message: "Minimum length cannot be greater than maximum length".to_string(),
                });
            }
        }

        // Validate array constraints
        if let (Some(min_items), Some(max_items)) = (schema.min_items, schema.max_items) {
            if min_items > max_items {
                return Err(SchemaError::ValidationError {
                    message: "Minimum items cannot be greater than maximum items".to_string(),
                });
            }
        }

        // Validate object constraints
        if let (Some(min_props), Some(max_props)) = (schema.min_properties, schema.max_properties) {
            if min_props > max_props {
                return Err(SchemaError::ValidationError {
                    message: "Minimum properties cannot be greater than maximum properties"
                        .to_string(),
                });
            }
        }

        // Validate that multipleOf is positive
        if let Some(multiple_of) = schema.multiple_of {
            if multiple_of <= 0.0 {
                return Err(SchemaError::ValidationError {
                    message: "multipleOf must be greater than 0".to_string(),
                });
            }
        }

        // Validate composition schemas and their requirements
        self.validate_composition_schemas(schema)?;

        // Recursively validate nested schemas
        for property_schema in schema.properties.values() {
            self.validate_schema(property_schema)?;
        }

        for def_schema in schema.definitions.values() {
            self.validate_schema(def_schema)?;
        }

        if let Some(items) = &schema.items {
            self.validate_schema(items)?;
        }

        if let Some(additional) = &schema.additional_properties {
            self.validate_schema(additional)?;
        }

        if let Some(not_schema) = &schema.not {
            // For "not" schemas, only validate structure, not logical requirements
            self.validate_schema_structure_only(not_schema)?;
        }

        Ok(())
    }

    /// Validate schema structure without logical requirements (for "not" schemas)
    fn validate_schema_structure_only(&self, schema: &Schema) -> Result<(), SchemaError> {
        // Validate basic structural constraints without requirement logic

        // Validate numeric constraints
        if let (Some(min), Some(max)) = (schema.minimum, schema.maximum) {
            if min > max {
                return Err(SchemaError::ValidationError {
                    message: "Minimum value cannot be greater than maximum value".to_string(),
                });
            }
        }

        if let (Some(min), Some(max)) = (schema.exclusive_minimum, schema.exclusive_maximum) {
            if min >= max {
                return Err(SchemaError::ValidationError {
                    message: "Exclusive minimum value must be less than exclusive maximum value"
                        .to_string(),
                });
            }
        }

        // Validate string constraints
        if let (Some(min_len), Some(max_len)) = (schema.min_length, schema.max_length) {
            if min_len > max_len {
                return Err(SchemaError::ValidationError {
                    message: "Minimum length cannot be greater than maximum length".to_string(),
                });
            }
        }

        // Validate array constraints
        if let (Some(min_items), Some(max_items)) = (schema.min_items, schema.max_items) {
            if min_items > max_items {
                return Err(SchemaError::ValidationError {
                    message: "Minimum items cannot be greater than maximum items".to_string(),
                });
            }
        }

        // Validate object constraints
        if let (Some(min_props), Some(max_props)) = (schema.min_properties, schema.max_properties) {
            if min_props > max_props {
                return Err(SchemaError::ValidationError {
                    message: "Minimum properties cannot be greater than maximum properties"
                        .to_string(),
                });
            }
        }

        // Validate that multipleOf is positive
        if let Some(multiple_of) = schema.multiple_of {
            if multiple_of <= 0.0 {
                return Err(SchemaError::ValidationError {
                    message: "multipleOf must be greater than 0".to_string(),
                });
            }
        }

        // Recursively validate nested schemas (structure only)
        for property_schema in schema.properties.values() {
            self.validate_schema_structure_only(property_schema)?;
        }

        for def_schema in schema.definitions.values() {
            self.validate_schema_structure_only(def_schema)?;
        }

        if let Some(items) = &schema.items {
            self.validate_schema_structure_only(items)?;
        }

        if let Some(additional) = &schema.additional_properties {
            self.validate_schema_structure_only(additional)?;
        }

        // For composition schemas in "not" context, just validate their structure
        for composition_schema in &schema.all_of {
            self.validate_schema_structure_only(composition_schema)?;
        }
        for composition_schema in &schema.any_of {
            self.validate_schema_structure_only(composition_schema)?;
        }
        for composition_schema in &schema.one_of {
            self.validate_schema_structure_only(composition_schema)?;
        }

        if let Some(not_schema) = &schema.not {
            self.validate_schema_structure_only(not_schema)?;
        }

        Ok(())
    }

    /// Validate required fields against available properties
    fn validate_required_fields(
        &self,
        schema: &Schema,
        required_fields: &[String],
    ) -> Result<(), SchemaError> {
        for required_field in required_fields {
            if !schema.properties.contains_key(required_field) {
                return Err(SchemaError::ValidationError {
                    message: format!(
                        "Required field '{}' not found in properties",
                        required_field
                    ),
                });
            }
        }
        Ok(())
    }

    /// Validate composition schemas (anyOf, oneOf, allOf) and their requirements
    fn validate_composition_schemas(&self, schema: &Schema) -> Result<(), SchemaError> {
        // Validate allOf schemas
        for (i, composition_schema) in schema.all_of.iter().enumerate() {
            // Recursively validate the composed schema structure, but skip requirement validation
            // if it's just a requirement constraint (no properties of its own)
            if !composition_schema.properties.is_empty()
                || composition_schema.schema_type != SchemaType::Any
            {
                self.validate_schema(composition_schema).map_err(|e| {
                    SchemaError::ValidationError {
                        message: format!("allOf schema {} is invalid: {}", i, e),
                    }
                })?;
            }

            // Validate requirements in allOf (all must be satisfiable together)
            // Check that required fields exist in the parent schema's properties
            for required_field in &composition_schema.required {
                if !schema.properties.contains_key(required_field) {
                    return Err(SchemaError::ValidationError {
                        message: format!(
                            "allOf schema {}: required field '{}' not found in parent properties",
                            i, required_field
                        ),
                    });
                }
            }
        }

        // Validate anyOf schemas
        for (i, composition_schema) in schema.any_of.iter().enumerate() {
            // For anyOf, only validate if the schema has its own structure
            if !composition_schema.properties.is_empty()
                || composition_schema.schema_type != SchemaType::Any
            {
                self.validate_schema(composition_schema).map_err(|e| {
                    SchemaError::ValidationError {
                        message: format!("anyOf schema {} is invalid: {}", i, e),
                    }
                })?;
            }

            // For anyOf, validate that required fields exist in parent properties
            // This is more lenient since only one alternative needs to match
            for required_field in &composition_schema.required {
                if !schema.properties.contains_key(required_field) {
                    return Err(SchemaError::ValidationError {
                        message: format!(
                            "anyOf schema {}: required field '{}' not found in parent properties",
                            i, required_field
                        ),
                    });
                }
            }
        }

        // Validate oneOf schemas
        for (i, composition_schema) in schema.one_of.iter().enumerate() {
            // For oneOf, only validate if the schema has its own structure
            if !composition_schema.properties.is_empty()
                || composition_schema.schema_type != SchemaType::Any
            {
                self.validate_schema(composition_schema).map_err(|e| {
                    SchemaError::ValidationError {
                        message: format!("oneOf schema {} is invalid: {}", i, e),
                    }
                })?;
            }

            // For oneOf, same logic as anyOf - validate individual schemas
            for required_field in &composition_schema.required {
                if !schema.properties.contains_key(required_field) {
                    return Err(SchemaError::ValidationError {
                        message: format!(
                            "oneOf schema {}: required field '{}' not found in parent properties",
                            i, required_field
                        ),
                    });
                }
            }
        }

        // Note: Advanced validation of anyOf/oneOf mutual exclusivity and consistency
        // is complex and may be too strict for real-world schemas. The main goal
        // was to handle conditional requirements properly, which we now do.

        Ok(())
    }

    /// Validate that anyOf alternatives are logically consistent
    fn validate_any_of_consistency(&self, any_of_schemas: &[Schema]) -> Result<(), SchemaError> {
        // Check for obviously contradictory type requirements
        let mut primitive_types = Vec::new();

        for schema in any_of_schemas {
            match &schema.schema_type {
                SchemaType::String
                | SchemaType::Number
                | SchemaType::Integer
                | SchemaType::Boolean => {
                    // Only track primitive types for conflict detection
                    if !primitive_types.iter().any(|t| {
                        std::mem::discriminant(t) == std::mem::discriminant(&schema.schema_type)
                    }) {
                        primitive_types.push(schema.schema_type.clone());
                    }
                }
                _ => {
                    // Other types (Object, Array, Any, etc.) are generally compatible in anyOf
                }
            }
        }

        // Warn if we have many conflicting primitive types (may indicate design issue)
        if primitive_types.len() > 3 {
            return Err(SchemaError::ValidationError {
                message: format!(
                    "anyOf contains many conflicting primitive types ({}), which may indicate a design issue",
                    primitive_types.len()
                ),
            });
        }

        Ok(())
    }

    /// Validate that enum values match the schema type
    pub fn validate_enum_values(&self, schema: &Schema) -> Result<(), SchemaError> {
        if schema.enum_values.is_empty() {
            return Ok(());
        }

        for enum_value in &schema.enum_values {
            if !self.value_matches_type(enum_value, &schema.schema_type) {
                return Err(SchemaError::ValidationError {
                    message: format!(
                        "Enum value {:?} does not match schema type {:?}",
                        enum_value, schema.schema_type
                    ),
                });
            }
        }

        Ok(())
    }

    /// Check if a JSON value matches a schema type
    fn value_matches_type(&self, value: &serde_json::Value, schema_type: &SchemaType) -> bool {
        use serde_json::Value;

        match (value, schema_type) {
            (Value::Null, SchemaType::Null) => true,
            (Value::Bool(_), SchemaType::Boolean) => true,
            (Value::Number(n), SchemaType::Integer) => n.is_i64(),
            (Value::Number(_), SchemaType::Number) => true,
            (Value::String(_), SchemaType::String) => true,
            (Value::Array(_), SchemaType::Array) => true,
            (Value::Object(_), SchemaType::Object) => true,
            (_, SchemaType::Any) => true,
            (val, SchemaType::Union(types)) => {
                types.iter().any(|t| self.value_matches_type(val, t))
            }
            _ => false,
        }
    }

    /// Check for common schema design issues and provide warnings
    pub fn lint_schema(&self, schema: &Schema) -> Vec<String> {
        let mut warnings = Vec::new();

        // Warning for object without properties or additionalProperties
        if matches!(schema.schema_type, SchemaType::Object)
            && schema.properties.is_empty()
            && schema.additional_properties.is_none()
        {
            warnings.push(
                "Object type without properties or additionalProperties allows any object shape"
                    .to_string(),
            );
        }

        // Warning for array without items constraint
        if matches!(schema.schema_type, SchemaType::Array) && schema.items.is_none() {
            warnings.push("Array type without items constraint allows any item types".to_string());
        }

        // Warning for missing description
        if schema.description.is_none() {
            warnings.push("Schema missing description".to_string());
        }

        // Warning for overly permissive string constraints
        if matches!(schema.schema_type, SchemaType::String) {
            if schema.min_length.is_none()
                && schema.max_length.is_none()
                && schema.pattern.is_none()
                && schema.format.is_none()
            {
                warnings.push(
                    "String type without length, pattern, or format constraints is very permissive"
                        .to_string(),
                );
            }
        }

        // Warning for very large maximum values
        if let Some(max_len) = schema.max_length {
            if max_len > 10_000 {
                warnings.push(format!("Very large maxLength: {}", max_len));
            }
        }

        if let Some(max_items) = schema.max_items {
            if max_items > 1_000 {
                warnings.push(format!("Very large maxItems: {}", max_items));
            }
        }

        // Recursively check nested schemas
        for (prop_name, prop_schema) in &schema.properties {
            let prop_warnings = self.lint_schema(prop_schema);
            for warning in prop_warnings {
                warnings.push(format!("Property '{}': {}", prop_name, warning));
            }
        }

        warnings
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_validate_simple_schema() {
        let validator = SchemaValidator::new();
        let schema = Schema::new(SchemaType::String);

        assert!(validator.validate_schema(&schema).is_ok());
    }

    #[test]
    fn test_validate_required_field_not_in_properties() {
        let validator = SchemaValidator::new();
        let mut schema = Schema::new(SchemaType::Object);
        schema.required = vec!["missing_field".to_string()];

        let result = validator.validate_schema(&schema);
        assert!(result.is_err());
        match result.unwrap_err() {
            SchemaError::ValidationError { message } => {
                assert!(message.contains("missing_field"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_validate_numeric_constraints() {
        let validator = SchemaValidator::new();
        let mut schema = Schema::new(SchemaType::Number);
        schema.minimum = Some(10.0);
        schema.maximum = Some(5.0); // Invalid: min > max

        let result = validator.validate_schema(&schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_string_length_constraints() {
        let validator = SchemaValidator::new();
        let mut schema = Schema::new(SchemaType::String);
        schema.min_length = Some(10);
        schema.max_length = Some(5); // Invalid: min > max

        let result = validator.validate_schema(&schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_enum_values() {
        let validator = SchemaValidator::new();
        let mut schema = Schema::new(SchemaType::String);
        schema.enum_values = vec![
            Value::String("valid".to_string()),
            Value::Number(serde_json::Number::from(42)), // Invalid for string type
        ];

        let result = validator.validate_enum_values(&schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_lint_permissive_schemas() {
        let validator = SchemaValidator::new();

        // Object without properties
        let object_schema = Schema::new(SchemaType::Object);
        let warnings = validator.lint_schema(&object_schema);
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("without properties")));

        // Array without items
        let array_schema = Schema::new(SchemaType::Array);
        let warnings = validator.lint_schema(&array_schema);
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("without items")));

        // String without constraints
        let string_schema = Schema::new(SchemaType::String);
        let warnings = validator.lint_schema(&string_schema);
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("very permissive")));
    }

    #[test]
    fn test_validate_any_of_composition() {
        let validator = SchemaValidator::new();

        // Create a schema similar to envelope schema with anyOf
        let mut schema = Schema::new(SchemaType::Object);

        // Add properties for data and error
        let data_prop = Schema::new(SchemaType::Any);
        let error_prop = Schema::new(SchemaType::Object);

        schema.properties.insert("data".to_string(), data_prop);
        schema.properties.insert("error".to_string(), error_prop);

        // Create anyOf alternatives
        let mut success_schema = Schema::new(SchemaType::Any);
        success_schema.required = vec!["data".to_string()];

        let mut error_schema = Schema::new(SchemaType::Any);
        error_schema.required = vec!["error".to_string()];

        schema.any_of = vec![success_schema, error_schema];

        // This should validate successfully now
        let result = validator.validate_schema(&schema);
        assert!(
            result.is_ok(),
            "anyOf schema validation failed: {:?}",
            result
        );
    }

    #[test]
    fn test_validate_any_of_missing_property() {
        let validator = SchemaValidator::new();

        let mut schema = Schema::new(SchemaType::Object);

        // Only add data property, not error
        let data_prop = Schema::new(SchemaType::Any);
        schema.properties.insert("data".to_string(), data_prop);

        // Create anyOf that requires missing property
        let mut success_schema = Schema::new(SchemaType::Any);
        success_schema.required = vec!["data".to_string()];

        let mut error_schema = Schema::new(SchemaType::Any);
        error_schema.required = vec!["missing_error".to_string()]; // This property doesn't exist

        schema.any_of = vec![success_schema, error_schema];

        // This should fail validation
        let result = validator.validate_schema(&schema);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            SchemaError::ValidationError { message } => {
                assert!(message.contains("missing_error"));
                assert!(message.contains("not found in parent properties"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_one_of_composition() {
        let validator = SchemaValidator::new();

        let mut schema = Schema::new(SchemaType::Object);

        // Add properties
        let data_prop = Schema::new(SchemaType::String);
        let error_prop = Schema::new(SchemaType::Object);

        schema.properties.insert("data".to_string(), data_prop);
        schema.properties.insert("error".to_string(), error_prop);

        // Create oneOf alternatives
        let mut success_schema = Schema::new(SchemaType::Any);
        success_schema.required = vec!["data".to_string()];

        let mut error_schema = Schema::new(SchemaType::Any);
        error_schema.required = vec!["error".to_string()];

        schema.one_of = vec![success_schema, error_schema];

        // This should validate successfully
        let result = validator.validate_schema(&schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_all_of_composition() {
        let validator = SchemaValidator::new();

        let mut schema = Schema::new(SchemaType::Object);

        // Add properties that satisfy both allOf requirements
        let name_prop = Schema::new(SchemaType::String);
        let age_prop = Schema::new(SchemaType::Integer);

        schema.properties.insert("name".to_string(), name_prop);
        schema.properties.insert("age".to_string(), age_prop);

        // Create allOf that requires both fields
        let mut name_requirement = Schema::new(SchemaType::Any);
        name_requirement.required = vec!["name".to_string()];

        let mut age_requirement = Schema::new(SchemaType::Any);
        age_requirement.required = vec!["age".to_string()];

        schema.all_of = vec![name_requirement, age_requirement];

        // This should validate successfully
        let result = validator.validate_schema(&schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_one_of_basic_functionality() {
        let validator = SchemaValidator::new();

        let mut schema = Schema::new(SchemaType::Object);

        // Create two different schemas in oneOf (should pass)
        let mut schema1 = Schema::new(SchemaType::Any);
        schema1.required = vec!["data".to_string()];

        let mut schema2 = Schema::new(SchemaType::Any);
        schema2.required = vec!["error".to_string()]; // Different requirement

        schema
            .properties
            .insert("data".to_string(), Schema::new(SchemaType::String));
        schema
            .properties
            .insert("error".to_string(), Schema::new(SchemaType::Object));
        schema.one_of = vec![schema1, schema2];

        // This should pass because schemas have different requirements
        let result = validator.validate_schema(&schema);
        assert!(result.is_ok());
    }
}
