use jsonschema;
use serde_json::json;

#[test]
fn test_simple_validation() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "number"}
        },
        "required": ["name"]
    });

    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .expect("Schema should be valid");

    let valid_data = json!({
        "name": "John",
        "age": 30
    });

    let invalid_data = json!({
        "age": 30
    });

    // Test valid data
    let result = validator.validate(&valid_data);
    assert!(result.is_ok(), "Valid data should pass");

    // Test invalid data
    let result = validator.validate(&invalid_data);
    assert!(result.is_err(), "Invalid data should fail");

    // Check how to handle errors
    if let Err(error) = result {
        // Try to understand the error type
        println!("Error: {:?}", error);
        println!("Error type: {}", std::any::type_name_of_val(&error));
    }
}
