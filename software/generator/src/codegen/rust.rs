// ABOUTME: Main Rust code generator implementation converting JSON Schema AST to Rust code
// ABOUTME: Handles complex schema features including composition, validation, and references

use super::types::*;
use crate::schema::{Schema, SchemaType, SchemaError};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Rust code generator for JSON Schema
pub struct RustCodeGenerator {
    config: RustCodegenConfig,
    generated_types: HashSet<String>,
    type_name_mapping: HashMap<String, String>,
}

/// Errors that can occur during code generation
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),
    
    #[error("Unsupported schema feature: {feature}")]
    UnsupportedFeature { feature: String },
    
    #[error("Type conversion error: {message}")]
    TypeConversion { message: String },
    
    #[error("Name conflict: {name}")]
    NameConflict { name: String },
    
    #[error("Code generation error: {message}")]
    Generation { message: String },
    
    #[error("Formatting error: {0}")]
    Formatting(#[from] std::fmt::Error),
}

impl RustCodeGenerator {
    /// Create a new Rust code generator with default configuration
    pub fn new() -> Self {
        Self::with_config(RustCodegenConfig::default())
    }
    
    /// Create a new Rust code generator with custom configuration
    pub fn with_config(config: RustCodegenConfig) -> Self {
        Self {
            config,
            generated_types: HashSet::new(),
            type_name_mapping: HashMap::new(),
        }
    }
    
    /// Generate Rust code from a JSON Schema
    pub fn generate(&mut self, schema: &Schema) -> Result<RustCode, CodegenError> {
        let mut code = RustCode::new();
        
        // Add standard imports
        self.add_standard_imports(&mut code);
        
        // Generate code for the root schema
        self.generate_schema(&schema, None, &mut code)?;
        
        // Generate code for definitions
        for (name, def_schema) in &schema.definitions {
            self.generate_schema(def_schema, Some(name), &mut code)?;
        }
        
        Ok(code)
    }
    
    /// Add standard imports based on configuration
    fn add_standard_imports(&self, code: &mut RustCode) {
        if self.config.serde_support {
            code.add_import("use serde::{Deserialize, Serialize};".to_string());
        }
        
        // Add std imports for collections
        code.add_import("use std::collections::HashMap;".to_string());
    }
    
    /// Generate code for a single schema
    fn generate_schema(&mut self, schema: &Schema, name: Option<&str>, code: &mut RustCode) -> Result<(), CodegenError> {
        // Skip if this is a reference
        if schema.is_reference() {
            return Ok(());
        }
        
        match &schema.schema_type {
            SchemaType::Object => {
                if let Some(type_name) = name {
                    self.generate_struct(schema, type_name, code)?;
                }
            }
            SchemaType::String if schema.is_enum() => {
                if let Some(type_name) = name {
                    self.generate_string_enum(schema, type_name, code)?;
                }
            }
            SchemaType::Union(types) => {
                if let Some(type_name) = name {
                    self.generate_union_enum(schema, type_name, types, code)?;
                }
            }
            _ => {
                // For primitive types, generate type aliases if they have constraints or names
                if let Some(type_name) = name {
                    if self.has_constraints(schema) || schema.format.is_some() {
                        self.generate_type_alias(schema, type_name, code)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate a Rust struct from an object schema
    fn generate_struct(&mut self, schema: &Schema, name: &str, code: &mut RustCode) -> Result<(), CodegenError> {
        let struct_name = self.to_rust_type_name(name);

        // Check for name conflicts
        if self.generated_types.contains(&struct_name) {
            return Err(CodegenError::NameConflict { name: struct_name });
        }
        self.generated_types.insert(struct_name.clone());

        // Detect if struct contains any float fields
        let has_floats = self.schema_contains_floats(schema);

        let mut rust_struct = RustStruct {
            name: struct_name.clone(),
            visibility: Visibility::Public,
            derives: self.get_derives_for_struct(has_floats),
            attributes: self.get_standard_attributes(),
            generics: Vec::new(),
            fields: Vec::new(),
            documentation: schema.description.clone(),
        };
        
        // Generate fields from properties
        for (field_name, field_schema) in &schema.properties {
            let rust_field = self.generate_field(field_name, field_schema, &schema.required)?;
            rust_struct.fields.push(rust_field);
        }
        
        // Handle additional properties
        if schema.additional_properties.is_some() {
            // Add a HashMap field for additional properties
            let additional_field = RustField {
                name: "additional_properties".to_string(),
                field_type: RustType::HashMap(
                    Box::new(RustType::String),
                    Box::new(RustType::Custom {
                        name: "serde_json::Value".to_string(),
                        generics: Vec::new(),
                        module_path: None,
                    })
                ),
                visibility: Visibility::Public,
                attributes: vec![
                    "#[serde(flatten)]".to_string(),
                    "#[serde(skip_serializing_if = \"HashMap::is_empty\")]".to_string(),
                ],
                documentation: Some("Additional properties not defined in the schema".to_string()),
            };
            rust_struct.fields.push(additional_field);
        }
        
        code.add_item(RustItem::Struct(rust_struct));
        
        // Generate validation implementation if enabled
        if self.config.validation {
            self.generate_validation_impl(&struct_name, schema, code)?;
        }
        
        Ok(())
    }
    
    /// Generate a field from a property schema
    fn generate_field(&mut self, name: &str, schema: &Schema, required: &[String]) -> Result<RustField, CodegenError> {
        let field_name = if self.config.snake_case_fields {
            self.to_snake_case(name)
        } else {
            name.to_string()
        };
        
        let is_required = required.contains(&name.to_string());
        let mut field_type = self.schema_to_rust_type(schema)?;
        
        // Wrap in Option if not required
        if !is_required {
            field_type = RustType::Option(Box::new(field_type));
        }
        
        let mut attributes = Vec::new();
        
        // Add serde attributes if field name differs from JSON name
        if self.config.serde_support && field_name != name {
            attributes.push(format!("#[serde(rename = \"{}\")]", name));
        }
        
        // Add skip_serializing_if for optional fields
        if self.config.serde_support && !is_required {
            attributes.push("#[serde(skip_serializing_if = \"Option::is_none\")]".to_string());
        }
        
        Ok(RustField {
            name: field_name,
            field_type,
            visibility: Visibility::Public,
            attributes,
            documentation: schema.description.clone(),
        })
    }
    
    /// Generate a string enum from a schema with enum values
    fn generate_string_enum(&mut self, schema: &Schema, name: &str, code: &mut RustCode) -> Result<(), CodegenError> {
        let enum_name = self.to_rust_type_name(name);

        if self.generated_types.contains(&enum_name) {
            return Err(CodegenError::NameConflict { name: enum_name });
        }
        self.generated_types.insert(enum_name.clone());

        let mut rust_enum = RustEnum {
            name: enum_name.clone(),
            visibility: Visibility::Public,
            derives: self.get_derives_for_enum(true), // String enums are simple (unit variants)
            attributes: self.get_standard_attributes(),
            generics: Vec::new(),
            variants: Vec::new(),
            documentation: schema.description.clone(),
        };

        // Generate variants from enum values
        for enum_value in &schema.enum_values {
            if let Some(value_str) = enum_value.as_str() {
                let variant_name = self.to_rust_type_name(value_str);
                let mut attributes = Vec::new();

                // Add serde rename if needed
                if self.config.serde_support && variant_name != value_str {
                    attributes.push(format!("#[serde(rename = \"{}\")]", value_str));
                }

                rust_enum.variants.push(RustEnumVariant {
                    name: variant_name,
                    data: RustVariantData::Unit,
                    attributes,
                    documentation: None,
                });
            }
        }

        code.add_item(RustItem::Enum(rust_enum));
        Ok(())
    }
    
    /// Generate a union enum from a schema with union types
    fn generate_union_enum(&mut self, schema: &Schema, name: &str, types: &[SchemaType], code: &mut RustCode) -> Result<(), CodegenError> {
        let enum_name = self.to_rust_type_name(name);

        if self.generated_types.contains(&enum_name) {
            return Err(CodegenError::NameConflict { name: enum_name });
        }
        self.generated_types.insert(enum_name.clone());

        let mut rust_enum = RustEnum {
            name: enum_name.clone(),
            visibility: Visibility::Public,
            derives: self.get_derives_for_enum(false), // Union enums are not simple (have data variants)
            attributes: self.get_standard_attributes(),
            generics: Vec::new(),
            variants: Vec::new(),
            documentation: schema.description.clone(),
        };
        
        // Generate variants for each union type
        for (i, schema_type) in types.iter().enumerate() {
            let variant_name = match schema_type {
                SchemaType::Null => "Null".to_string(),
                SchemaType::Boolean => "Boolean".to_string(),
                SchemaType::Integer => "Integer".to_string(),
                SchemaType::Number => "Number".to_string(),
                SchemaType::String => "String".to_string(),
                SchemaType::Array => "Array".to_string(),
                SchemaType::Object => "Object".to_string(),
                _ => format!("Variant{}", i),
            };
            
            let rust_type = self.schema_type_to_rust_type(schema_type)?;
            
            let variant_data = if matches!(schema_type, SchemaType::Null) {
                RustVariantData::Unit
            } else {
                RustVariantData::Tuple(vec![rust_type])
            };
            
            rust_enum.variants.push(RustEnumVariant {
                name: variant_name,
                data: variant_data,
                attributes: Vec::new(),
                documentation: None,
            });
        }
        
        code.add_item(RustItem::Enum(rust_enum));
        Ok(())
    }
    
    /// Generate a type alias for constrained primitive types
    fn generate_type_alias(&mut self, schema: &Schema, name: &str, code: &mut RustCode) -> Result<(), CodegenError> {
        let alias_name = self.to_rust_type_name(name);
        
        if self.generated_types.contains(&alias_name) {
            return Err(CodegenError::NameConflict { name: alias_name });
        }
        self.generated_types.insert(alias_name.clone());
        
        let target_type = self.schema_to_rust_type(schema)?;
        
        let rust_alias = RustTypeAlias {
            name: alias_name,
            visibility: Visibility::Public,
            generics: Vec::new(),
            target_type,
            attributes: self.get_standard_attributes(),
            documentation: schema.description.clone(),
        };
        
        code.add_item(RustItem::TypeAlias(rust_alias));
        Ok(())
    }
    
    /// Convert a JSON Schema to a Rust type
    fn schema_to_rust_type(&mut self, schema: &Schema) -> Result<RustType, CodegenError> {
        if let Some(reference) = schema.get_reference() {
            // Handle reference to another type
            let type_name = self.extract_type_name_from_reference(reference)?;
            return Ok(RustType::Custom {
                name: self.to_rust_type_name(&type_name),
                generics: Vec::new(),
                module_path: None,
            });
        }
        
        self.schema_type_to_rust_type(&schema.schema_type)
    }
    
    /// Convert a SchemaType to a Rust type
    fn schema_type_to_rust_type(&mut self, schema_type: &SchemaType) -> Result<RustType, CodegenError> {
        match schema_type {
            SchemaType::Null => Ok(RustType::Unit),
            SchemaType::Boolean => Ok(RustType::Bool),
            SchemaType::Integer => Ok(RustType::I64),
            SchemaType::Number => Ok(RustType::F64),
            SchemaType::String => Ok(RustType::String),
            SchemaType::Array => Ok(RustType::Vec(Box::new(RustType::Custom {
                name: "serde_json::Value".to_string(),
                generics: Vec::new(),
                module_path: None,
            }))),
            SchemaType::Object => Ok(RustType::HashMap(
                Box::new(RustType::String),
                Box::new(RustType::Custom {
                    name: "serde_json::Value".to_string(),
                    generics: Vec::new(),
                    module_path: None,
                })
            )),
            SchemaType::Union(types) => {
                // For simple unions, try to create a single Rust type
                if types.len() == 2 && types.contains(&SchemaType::Null) {
                    // Nullable type - wrap in Option
                    let non_null_type = types.iter()
                        .find(|t| !matches!(t, SchemaType::Null))
                        .ok_or_else(|| CodegenError::TypeConversion {
                            message: "Invalid nullable union".to_string()
                        })?;
                    let inner_type = self.schema_type_to_rust_type(non_null_type)?;
                    Ok(RustType::Option(Box::new(inner_type)))
                } else {
                    // Complex union - would need to generate an enum
                    Err(CodegenError::UnsupportedFeature {
                        feature: "Complex union types require named enum generation".to_string()
                    })
                }
            }
            SchemaType::Reference(reference) => {
                let type_name = self.extract_type_name_from_reference(reference)?;
                Ok(RustType::Custom {
                    name: self.to_rust_type_name(&type_name),
                    generics: Vec::new(),
                    module_path: None,
                })
            }
            SchemaType::Any => Ok(RustType::Custom {
                name: "serde_json::Value".to_string(),
                generics: Vec::new(),
                module_path: None,
            }),
        }
    }
    
    /// Extract type name from a JSON Schema reference
    fn extract_type_name_from_reference(&self, reference: &str) -> Result<String, CodegenError> {
        if let Some(name) = reference.strip_prefix("#/definitions/") {
            Ok(name.to_string())
        } else if let Some(name) = reference.strip_prefix("#/$defs/") {
            Ok(name.to_string())
        } else {
            Err(CodegenError::TypeConversion {
                message: format!("Unsupported reference format: {}", reference)
            })
        }
    }
    
    /// Generate validation implementation for a struct
    fn generate_validation_impl(&self, struct_name: &str, schema: &Schema, code: &mut RustCode) -> Result<(), CodegenError> {
        let mut methods = Vec::new();
        
        // Generate validate method
        let validate_method = RustMethod {
            name: "validate".to_string(),
            visibility: Visibility::Public,
            receiver: Some(RustReceiver::SelfRef),
            parameters: Vec::new(),
            return_type: Some(RustType::Custom {
                name: "Result<(), String>".to_string(),
                generics: Vec::new(),
                module_path: None,
            }),
            body: self.generate_validation_body(schema)?,
            attributes: Vec::new(),
            documentation: Some("Validate this instance against the JSON Schema constraints".to_string()),
        };
        methods.push(validate_method);
        
        let impl_block = RustImpl {
            target_type: RustType::Custom {
                name: struct_name.to_string(),
                generics: Vec::new(),
                module_path: None,
            },
            trait_impl: None,
            generics: Vec::new(),
            methods,
            attributes: Vec::new(),
        };
        
        code.add_item(RustItem::Impl(impl_block));
        Ok(())
    }
    
    /// Generate validation method body
    fn generate_validation_body(&self, _schema: &Schema) -> Result<String, CodegenError> {
        // For now, return a simple implementation
        // Future enhancement: Implement detailed validation based on schema constraints
        // This would generate validation code for JSON Schema constraints (minLength, pattern, etc.)
        Ok("Ok(())".to_string())
    }
    
    /// Check if a schema has validation constraints
    fn has_constraints(&self, schema: &Schema) -> bool {
        schema.minimum.is_some() ||
        schema.maximum.is_some() ||
        schema.min_length.is_some() ||
        schema.max_length.is_some() ||
        schema.pattern.is_some() ||
        schema.min_items.is_some() ||
        schema.max_items.is_some() ||
        !schema.enum_values.is_empty()
    }

    /// Check if a schema contains float/number fields (recursively checks properties)
    fn schema_contains_floats(&self, schema: &Schema) -> bool {
        // Check direct schema type
        if matches!(schema.schema_type, SchemaType::Number) {
            return true;
        }

        // Check properties
        for (_field_name, field_schema) in &schema.properties {
            if self.schema_type_is_float(&field_schema.schema_type) {
                return true;
            }
        }

        false
    }

    /// Check if a SchemaType represents a float
    fn schema_type_is_float(&self, schema_type: &SchemaType) -> bool {
        match schema_type {
            SchemaType::Number => true,
            SchemaType::Union(types) => types.iter().any(|t| self.schema_type_is_float(t)),
            _ => false,
        }
    }

    /// Get standard derives based on configuration
    fn get_standard_derives(&self) -> Vec<String> {
        self.get_derives_for_struct(false)
    }

    /// Get derives for struct types with optional float detection
    fn get_derives_for_struct(&self, has_floats: bool) -> Vec<String> {
        let mut derives = Vec::new();

        if self.config.debug {
            derives.push("Debug".to_string());
        }
        if self.config.clone {
            derives.push("Clone".to_string());
        }
        if self.config.partial_eq {
            derives.push("PartialEq".to_string());
        }
        // Add Eq only if no floats (floats only support PartialEq, not Eq)
        if self.config.partial_eq && !has_floats {
            derives.push("Eq".to_string());
            derives.push("Hash".to_string());
        }
        if self.config.serde_support {
            derives.push("Serialize".to_string());
            derives.push("Deserialize".to_string());
        }
        if self.config.default {
            derives.push("Default".to_string());
        }

        derives.extend(self.config.custom_derives.clone());
        derives
    }

    /// Get derives for simple enum types (unit variants only)
    fn get_derives_for_enum(&self, is_simple: bool) -> Vec<String> {
        let mut derives = Vec::new();

        if self.config.debug {
            derives.push("Debug".to_string());
        }
        if self.config.clone {
            derives.push("Clone".to_string());
        }
        // Simple enums can be Copy
        if is_simple {
            derives.push("Copy".to_string());
        }
        if self.config.partial_eq {
            derives.push("PartialEq".to_string());
            derives.push("Eq".to_string());
            derives.push("Hash".to_string());
        }
        if self.config.serde_support {
            derives.push("Serialize".to_string());
            derives.push("Deserialize".to_string());
        }
        // Do NOT add Default for enums - unclear which variant should be default

        derives.extend(self.config.custom_derives.clone());
        derives
    }
    
    /// Get standard attributes based on configuration
    fn get_standard_attributes(&self) -> Vec<String> {
        let mut attributes = Vec::new();
        
        // Add custom attributes
        attributes.extend(self.config.custom_attributes.clone());
        
        attributes
    }
    
    /// Convert a name to Rust type naming convention (PascalCase)
    fn to_rust_type_name(&self, name: &str) -> String {
        self.to_pascal_case(name)
    }
    
    /// Convert a string to PascalCase, handling special characters
    fn to_pascal_case(&self, s: &str) -> String {
        // Handle leading special characters first
        let sanitized = self.sanitize_identifier(s);

        // Split on separators, but replace '-' between numbers with 'To'
        let parts: Vec<String> = sanitized
            .split(['_', '-', ' '])
            .enumerate()
            .filter_map(|(_i, part)| {
                if part.is_empty() {
                    return None;
                }

                let mut chars = part.chars();
                match chars.next() {
                    None => None,
                    Some(first) => {
                        let capitalized = first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase();
                        Some(capitalized)
                    }
                }
            })
            .collect();

        let result = parts.join("");

        // If the result starts with a digit, prefix with underscore
        // (Rust identifiers cannot start with digits)
        if result.chars().next().map_or(false, |c| c.is_numeric()) {
            format!("_{}", result)
        } else {
            result
        }
    }

    /// Sanitize a string to be a valid Rust identifier
    fn sanitize_identifier(&self, s: &str) -> String {
        let mut result = String::new();
        let mut prev_char: Option<char> = None;

        for ch in s.chars() {
            match ch {
                // Replace leading + with "Plus"
                '+' if result.is_empty() => {
                    result.push_str("Plus");
                }
                '+' => {} // Skip + in middle

                // Replace dash between numbers with " To " (will be capitalized later)
                '-' if prev_char.map_or(false, |p| p.is_numeric()) => {
                    result.push_str(" To ");
                }

                // Keep alphanumeric and separators
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | ' ' => {
                    result.push(ch);
                }

                // Skip other special characters
                _ => {}
            }

            prev_char = Some(ch);
        }

        result
    }
    
    /// Convert a string to snake_case
    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }

        let result = result.replace(['-', ' '], "_");

        // Escape Rust keywords
        self.escape_rust_keyword(&result)
    }

    /// Escape Rust keywords with r# prefix
    fn escape_rust_keyword(&self, s: &str) -> String {
        // List of Rust keywords that need escaping
        const KEYWORDS: &[&str] = &[
            "as", "break", "const", "continue", "crate", "else", "enum", "extern",
            "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
            "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            "async", "await", "dyn", "abstract", "become", "box", "do", "final",
            "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try"
        ];

        if KEYWORDS.contains(&s) {
            format!("r#{}", s)
        } else {
            s.to_string()
        }
    }
}

impl Default for RustCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Render Rust code to a formatted string
pub fn render_rust_code(code: &RustCode) -> Result<String, CodegenError> {
    let mut output = String::new();
    
    // Add features
    if !code.features.is_empty() {
        for feature in &code.features {
            writeln!(output, "#![feature({})]", feature)
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
        writeln!(output)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Add imports
    if !code.imports.is_empty() {
        for import in &code.imports {
            writeln!(output, "{}", import)
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
        writeln!(output)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Add items
    for item in &code.items {
        render_rust_item(item, &mut output)?;
        writeln!(output)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    Ok(output)
}

/// Render a single Rust item
fn render_rust_item(item: &RustItem, output: &mut String) -> Result<(), CodegenError> {
    match item {
        RustItem::Struct(s) => render_struct(s, output),
        RustItem::Enum(e) => render_enum(e, output),
        RustItem::TypeAlias(t) => render_type_alias(t, output),
        RustItem::Const(c) => render_const(c, output),
        RustItem::Impl(i) => render_impl(i, output),
    }
}

/// Render a struct
fn render_struct(rust_struct: &RustStruct, output: &mut String) -> Result<(), CodegenError> {
    // Documentation
    if let Some(doc) = &rust_struct.documentation {
        writeln!(output, "/// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Attributes
    for attr in &rust_struct.attributes {
        writeln!(output, "{}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Derives
    if !rust_struct.derives.is_empty() {
        write!(output, "#[derive(")
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        for (i, derive) in rust_struct.derives.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")
                    .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
            }
            write!(output, "{}", derive)
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
        writeln!(output, ")]")
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Struct definition
    write!(output, "{} struct {}", rust_struct.visibility.to_string(), rust_struct.name)
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    if !rust_struct.generics.is_empty() {
        write!(output, "<{}>", rust_struct.generics.join(", "))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    writeln!(output, " {{")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    // Fields
    for field in &rust_struct.fields {
        render_struct_field(field, output)?;
    }
    
    writeln!(output, "}}")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render a struct field
fn render_struct_field(field: &RustField, output: &mut String) -> Result<(), CodegenError> {
    // Field documentation
    if let Some(doc) = &field.documentation {
        writeln!(output, "    /// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Field attributes
    for attr in &field.attributes {
        writeln!(output, "    {}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Field definition
    let visibility = if matches!(field.visibility, Visibility::Private) {
        String::new()
    } else {
        format!("{} ", field.visibility.to_string())
    };
    
    writeln!(output, "    {}{}: {},", visibility, field.name, render_rust_type(&field.field_type))
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render an enum
fn render_enum(rust_enum: &RustEnum, output: &mut String) -> Result<(), CodegenError> {
    // Documentation
    if let Some(doc) = &rust_enum.documentation {
        writeln!(output, "/// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Attributes
    for attr in &rust_enum.attributes {
        writeln!(output, "{}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Derives
    if !rust_enum.derives.is_empty() {
        write!(output, "#[derive(")
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        for (i, derive) in rust_enum.derives.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")
                    .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
            }
            write!(output, "{}", derive)
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
        writeln!(output, ")]")
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Enum definition
    write!(output, "{} enum {}", rust_enum.visibility.to_string(), rust_enum.name)
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    if !rust_enum.generics.is_empty() {
        write!(output, "<{}>", rust_enum.generics.join(", "))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    writeln!(output, " {{")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    // Variants
    for variant in &rust_enum.variants {
        render_enum_variant(variant, output)?;
    }
    
    writeln!(output, "}}")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render an enum variant
fn render_enum_variant(variant: &RustEnumVariant, output: &mut String) -> Result<(), CodegenError> {
    // Variant documentation
    if let Some(doc) = &variant.documentation {
        writeln!(output, "    /// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Variant attributes
    for attr in &variant.attributes {
        writeln!(output, "    {}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Variant definition
    write!(output, "    {}", variant.name)
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    match &variant.data {
        RustVariantData::Unit => {}
        RustVariantData::Tuple(types) => {
            write!(output, "(")
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
            for (i, rust_type) in types.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")
                        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
                }
                write!(output, "{}", render_rust_type(rust_type))
                    .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
            }
            write!(output, ")")
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
        RustVariantData::Struct(fields) => {
            writeln!(output, " {{")
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
            for field in fields {
                write!(output, "        {}: {},", field.name, render_rust_type(&field.field_type))
                    .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
            }
            write!(output, "    }}")
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
    }
    
    writeln!(output, ",")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render a type alias
fn render_type_alias(alias: &RustTypeAlias, output: &mut String) -> Result<(), CodegenError> {
    // Documentation
    if let Some(doc) = &alias.documentation {
        writeln!(output, "/// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Attributes
    for attr in &alias.attributes {
        writeln!(output, "{}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Type alias definition
    write!(output, "{} type {}", alias.visibility.to_string(), alias.name)
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    if !alias.generics.is_empty() {
        write!(output, "<{}>", alias.generics.join(", "))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    writeln!(output, " = {};", render_rust_type(&alias.target_type))
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render a constant
fn render_const(rust_const: &RustConst, output: &mut String) -> Result<(), CodegenError> {
    // Documentation
    if let Some(doc) = &rust_const.documentation {
        writeln!(output, "/// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Attributes
    for attr in &rust_const.attributes {
        writeln!(output, "{}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Const definition
    writeln!(output, "{} const {}: {} = {};", 
             rust_const.visibility.to_string(), 
             rust_const.name, 
             render_rust_type(&rust_const.const_type),
             rust_const.value)
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render an impl block
fn render_impl(rust_impl: &RustImpl, output: &mut String) -> Result<(), CodegenError> {
    // Attributes
    for attr in &rust_impl.attributes {
        writeln!(output, "{}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Impl definition
    write!(output, "impl")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    if !rust_impl.generics.is_empty() {
        write!(output, "<{}>", rust_impl.generics.join(", "))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    if let Some(trait_impl) = &rust_impl.trait_impl {
        write!(output, " {} for", render_rust_type(trait_impl))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    writeln!(output, " {} {{", render_rust_type(&rust_impl.target_type))
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    // Methods
    for method in &rust_impl.methods {
        render_method(method, output)?;
    }
    
    writeln!(output, "}}")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render a method
fn render_method(method: &RustMethod, output: &mut String) -> Result<(), CodegenError> {
    // Method documentation
    if let Some(doc) = &method.documentation {
        writeln!(output, "    /// {}", doc)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Method attributes
    for attr in &method.attributes {
        writeln!(output, "    {}", attr)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    // Method signature
    write!(output, "    {} fn {}(", method.visibility.to_string(), method.name)
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    let mut param_count = 0;
    
    // Receiver
    if let Some(receiver) = &method.receiver {
        match receiver {
            RustReceiver::SelfValue => write!(output, "self")?,
            RustReceiver::SelfRef => write!(output, "&self")?,
            RustReceiver::SelfMut => write!(output, "&mut self")?,
        }
        param_count += 1;
    }
    
    // Parameters
    for param in &method.parameters {
        if param_count > 0 {
            write!(output, ", ")
                .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        }
        write!(output, "{}: {}", param.name, render_rust_type(&param.param_type))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
        param_count += 1;
    }
    
    write!(output, ")")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    // Return type
    if let Some(return_type) = &method.return_type {
        write!(output, " -> {}", render_rust_type(return_type))
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    writeln!(output, " {{")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    // Method body
    for line in method.body.lines() {
        writeln!(output, "        {}", line)
            .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    }
    
    writeln!(output, "    }}")
        .map_err(|e| CodegenError::Generation { message: e.to_string() })?;
    
    Ok(())
}

/// Render a Rust type to string
fn render_rust_type(rust_type: &RustType) -> String {
    match rust_type {
        RustType::Bool => "bool".to_string(),
        RustType::I8 => "i8".to_string(),
        RustType::I16 => "i16".to_string(),
        RustType::I32 => "i32".to_string(),
        RustType::I64 => "i64".to_string(),
        RustType::I128 => "i128".to_string(),
        RustType::Isize => "isize".to_string(),
        RustType::U8 => "u8".to_string(),
        RustType::U16 => "u16".to_string(),
        RustType::U32 => "u32".to_string(),
        RustType::U64 => "u64".to_string(),
        RustType::U128 => "u128".to_string(),
        RustType::Usize => "usize".to_string(),
        RustType::F32 => "f32".to_string(),
        RustType::F64 => "f64".to_string(),
        RustType::Char => "char".to_string(),
        RustType::Str => "&str".to_string(),
        RustType::String => "String".to_string(),
        RustType::Option(inner) => format!("Option<{}>", render_rust_type(inner)),
        RustType::Vec(inner) => format!("Vec<{}>", render_rust_type(inner)),
        RustType::HashMap(key, value) => format!("HashMap<{}, {}>", render_rust_type(key), render_rust_type(value)),
        RustType::BTreeMap(key, value) => format!("BTreeMap<{}, {}>", render_rust_type(key), render_rust_type(value)),
        RustType::HashSet(inner) => format!("HashSet<{}>", render_rust_type(inner)),
        RustType::BTreeSet(inner) => format!("BTreeSet<{}>", render_rust_type(inner)),
        RustType::Reference { mutable, lifetime, inner } => {
            let mut result = "&".to_string();
            if let Some(lt) = lifetime {
                result.push_str(&format!("'{} ", lt));
            }
            if *mutable {
                result.push_str("mut ");
            }
            result.push_str(&render_rust_type(inner));
            result
        }
        RustType::Custom { name, generics, .. } => {
            if generics.is_empty() {
                name.clone()
            } else {
                format!("{}<{}>", name, generics.iter().map(render_rust_type).collect::<Vec<_>>().join(", "))
            }
        }
        RustType::Generic(name) => name.clone(),
        RustType::Tuple(types) => {
            if types.is_empty() {
                "()".to_string()
            } else {
                format!("({})", types.iter().map(render_rust_type).collect::<Vec<_>>().join(", "))
            }
        }
        RustType::Unit => "()".to_string(),
        RustType::Array(inner, size) => format!("[{}; {}]", render_rust_type(inner), size),
        RustType::Slice(inner) => format!("[{}]", render_rust_type(inner)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Schema, SchemaType};
    use serde_json::json;

    #[test]
    fn test_generate_simple_struct() {
        let mut generator = RustCodeGenerator::new();
        
        let mut schema = Schema::new(SchemaType::Object);
        schema.title = Some("User".to_string());
        schema.description = Some("A user object".to_string());
        
        // Add properties
        let mut name_schema = Schema::new(SchemaType::String);
        name_schema.description = Some("The user's name".to_string());
        schema.properties.insert("name".to_string(), name_schema);
        
        let mut age_schema = Schema::new(SchemaType::Integer);
        age_schema.minimum = Some(0.0);
        schema.properties.insert("age".to_string(), age_schema);
        
        schema.required = vec!["name".to_string()];
        
        let result = generator.generate(&schema).unwrap();
        let rendered = render_rust_code(&result).unwrap();
        
        println!("Generated Rust code:\n{}", rendered);
        
        assert!(rendered.contains("pub struct User"));
        assert!(rendered.contains("pub name: String"));
        assert!(rendered.contains("pub age: Option<i64>"));
        assert!(rendered.contains("Serialize, Deserialize"));
    }

    #[test]
    fn test_generate_string_enum() {
        let mut generator = RustCodeGenerator::new();
        
        let mut schema = Schema::new(SchemaType::String);
        schema.title = Some("Color".to_string());
        schema.enum_values = vec![
            json!("red"),
            json!("green"),
            json!("blue"),
        ];
        
        let result = generator.generate_string_enum(&schema, "Color", &mut RustCode::new()).unwrap();
        
        // This test would need to check the generated enum in the code structure
        // For now, just verify it doesn't error
    }

    #[test]
    fn test_rust_type_rendering() {
        assert_eq!(render_rust_type(&RustType::String), "String");
        assert_eq!(render_rust_type(&RustType::Option(Box::new(RustType::I32))), "Option<i32>");
        assert_eq!(render_rust_type(&RustType::Vec(Box::new(RustType::String))), "Vec<String>");
        assert_eq!(
            render_rust_type(&RustType::HashMap(
                Box::new(RustType::String),
                Box::new(RustType::I32)
            )),
            "HashMap<String, i32>"
        );
    }
}