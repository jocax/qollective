//! Contract Testing Framework for MCP Tool Parameters
//!
//! This module provides utilities for validating that MCP tool parameter schemas
//! match between callers and callees, preventing parameter mismatches at runtime.
//!
//! # Purpose
//!
//! Contract testing ensures that:
//! - Parameter schemas match between orchestrator and services
//! - Serialization/deserialization is reversible (roundtrip)
//! - Envelope metadata is preserved across service boundaries
//!
//! # Usage
//!
//! ```rust,no_run
//! use shared_types::contract_tests::*;
//! use serde::{Serialize, Deserialize};
//! use schemars::JsonSchema;
//!
//! #[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
//! struct MyParams {
//!     field: String,
//! }
//!
//! // Validate schema structure
//! validate_tool_contract::<MyParams>().expect("Schema validation failed");
//!
//! // Test roundtrip serialization
//! let params = MyParams { field: "test".to_string() };
//! test_roundtrip_serialization(params).expect("Roundtrip failed");
//! ```

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Validates that a type's JSON schema is well-formed and matches expected structure.
///
/// This function generates a JSON schema from the generic type parameter using `schemars`,
/// then validates that the schema is compilable by `jsonschema`.
///
/// # Type Parameters
///
/// * `TParams` - Type implementing `JsonSchema` to validate
///
/// # Returns
///
/// * `Ok(())` - Schema is valid
/// * `Err(String)` - Schema validation failed with error message
///
/// # Example
///
/// ```rust,no_run
/// use shared_types::contract_tests::validate_tool_contract;
/// use schemars::JsonSchema;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, JsonSchema)]
/// struct ValidParams {
///     name: String,
///     age: u32,
/// }
///
/// // Validate schema
/// validate_tool_contract::<ValidParams>().expect("Schema should be valid");
/// ```
pub fn validate_tool_contract<TParams: JsonSchema>() -> Result<(), String> {
    // Generate schema from type
    let schema = schemars::schema_for!(TParams);

    // Convert to serde_json::Value for validation
    let schema_json = serde_json::to_value(&schema)
        .map_err(|e| format!("Failed to serialize schema to JSON: {}", e))?;

    // Attempt to compile schema with jsonschema validator
    jsonschema::options()
        .should_validate_formats(true)
        .build(&schema_json)
        .map_err(|e| format!("Failed to compile JSON schema: {}", e))?;

    Ok(())
}

/// Tests serialize → deserialize → serialize roundtrip for a value.
///
/// This function ensures that a value can be:
/// 1. Serialized to JSON
/// 2. Deserialized back to the original type
/// 3. Re-serialized to JSON
/// 4. All three representations are equal
///
/// # Type Parameters
///
/// * `T` - Type implementing `Serialize`, `Deserialize`, and `PartialEq`
///
/// # Arguments
///
/// * `value` - Original value to test
///
/// # Returns
///
/// * `Ok(())` - Roundtrip succeeded, all stages match
/// * `Err(String)` - Roundtrip failed with detailed error message
///
/// # Example
///
/// ```rust,no_run
/// use shared_types::contract_tests::test_roundtrip_serialization;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Data {
///     value: i32,
/// }
///
/// let data = Data { value: 42 };
/// test_roundtrip_serialization(data).expect("Roundtrip should succeed");
/// ```
pub fn test_roundtrip_serialization<T>(value: T) -> Result<(), String>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    // Stage 1: Serialize to JSON
    let json1 = serde_json::to_value(&value)
        .map_err(|e| format!("Stage 1 serialization failed: {}", e))?;

    // Stage 2: Deserialize back to type
    let deserialized: T = serde_json::from_value(json1.clone())
        .map_err(|e| format!("Stage 2 deserialization failed: {}\nJSON: {}", e, json1))?;

    // Stage 3: Re-serialize to JSON
    let json2 = serde_json::to_value(&deserialized)
        .map_err(|e| format!("Stage 3 re-serialization failed: {}", e))?;

    // Verify original value == deserialized value
    if value != deserialized {
        return Err(format!(
            "Value mismatch after deserialization:\nOriginal: {:?}\nDeserialized: {:?}",
            value, deserialized
        ));
    }

    // Verify JSON representations match
    if json1 != json2 {
        return Err(format!(
            "JSON mismatch after roundtrip:\nFirst: {}\nSecond: {}",
            json1, json2
        ));
    }

    Ok(())
}

/// Validates that two types have compatible JSON schemas.
///
/// This is useful for ensuring that request/response parameter types
/// match between different services.
///
/// # Type Parameters
///
/// * `T1` - First type implementing `JsonSchema`
/// * `T2` - Second type implementing `JsonSchema`
///
/// # Returns
///
/// * `Ok(())` - Schemas are compatible
/// * `Err(String)` - Schemas differ with detailed comparison
pub fn validate_schema_compatibility<T1: JsonSchema, T2: JsonSchema>() -> Result<(), String> {
    let schema1 = schemars::schema_for!(T1);
    let schema2 = schemars::schema_for!(T2);

    let json1 = serde_json::to_value(&schema1)
        .map_err(|e| format!("Failed to serialize first schema: {}", e))?;
    let json2 = serde_json::to_value(&schema2)
        .map_err(|e| format!("Failed to serialize second schema: {}", e))?;

    if json1 != json2 {
        return Err(format!(
            "Schema mismatch:\nSchema 1: {}\nSchema 2: {}",
            serde_json::to_string_pretty(&json1).unwrap_or_default(),
            serde_json::to_string_pretty(&json2).unwrap_or_default()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    struct TestParams {
        name: String,
        count: u32,
        optional: Option<String>,
    }

    #[test]
    fn test_validate_tool_contract_success() {
        // Valid schema should pass
        let result = validate_tool_contract::<TestParams>();
        assert!(result.is_ok(), "Schema validation should succeed");
    }

    #[test]
    fn test_roundtrip_serialization_success() {
        let params = TestParams {
            name: "test".to_string(),
            count: 42,
            optional: Some("value".to_string()),
        };

        let result = test_roundtrip_serialization(params);
        assert!(result.is_ok(), "Roundtrip should succeed: {:?}", result);
    }

    #[test]
    fn test_roundtrip_serialization_with_none() {
        let params = TestParams {
            name: "test".to_string(),
            count: 0,
            optional: None,
        };

        let result = test_roundtrip_serialization(params);
        assert!(result.is_ok(), "Roundtrip with None should succeed: {:?}", result);
    }

    #[test]
    fn test_schema_compatibility_same_type() {
        let result = validate_schema_compatibility::<TestParams, TestParams>();
        assert!(result.is_ok(), "Same type should be compatible");
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    struct DifferentParams {
        name: String,
        value: i32, // Different field name and type
    }

    #[test]
    fn test_schema_compatibility_different_types() {
        let result = validate_schema_compatibility::<TestParams, DifferentParams>();
        assert!(result.is_err(), "Different types should not be compatible");
    }
}
