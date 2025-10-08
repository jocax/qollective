// ABOUTME: Type definitions and intermediate representation for Rust code generation
// ABOUTME: Defines structures for representing Rust types, structs, enums, and validation rules

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rust code generation output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustCode {
    pub items: Vec<RustItem>,
    pub imports: Vec<String>,
    pub features: Vec<String>,
}

/// Top-level Rust code items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustItem {
    Struct(RustStruct),
    Enum(RustEnum),
    TypeAlias(RustTypeAlias),
    Const(RustConst),
    Impl(RustImpl),
}

/// Rust struct definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustStruct {
    pub name: String,
    pub visibility: Visibility,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,
    pub generics: Vec<String>,
    pub fields: Vec<RustField>,
    pub documentation: Option<String>,
}

/// Rust enum definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustEnum {
    pub name: String,
    pub visibility: Visibility,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,
    pub generics: Vec<String>,
    pub variants: Vec<RustEnumVariant>,
    pub documentation: Option<String>,
}

/// Rust enum variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustEnumVariant {
    pub name: String,
    pub data: RustVariantData,
    pub attributes: Vec<String>,
    pub documentation: Option<String>,
}

/// Rust enum variant data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustVariantData {
    Unit,
    Tuple(Vec<RustType>),
    Struct(Vec<RustField>),
}

/// Rust field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustField {
    pub name: String,
    pub field_type: RustType,
    pub visibility: Visibility,
    pub attributes: Vec<String>,
    pub documentation: Option<String>,
}

/// Rust type representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustType {
    /// Primitive types
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Char,
    Str,
    String,
    
    /// Container types
    Option(Box<RustType>),
    Vec(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),
    BTreeMap(Box<RustType>, Box<RustType>),
    HashSet(Box<RustType>),
    BTreeSet(Box<RustType>),
    
    /// Reference types
    Reference {
        mutable: bool,
        lifetime: Option<String>,
        inner: Box<RustType>,
    },
    
    /// Custom types
    Custom {
        name: String,
        generics: Vec<RustType>,
        module_path: Option<String>,
    },
    
    /// Generic type parameter
    Generic(String),
    
    /// Tuple type
    Tuple(Vec<RustType>),
    
    /// Unit type
    Unit,
    
    /// Array type with fixed size
    Array(Box<RustType>, usize),
    
    /// Slice type
    Slice(Box<RustType>),
}

/// Rust type alias definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustTypeAlias {
    pub name: String,
    pub visibility: Visibility,
    pub generics: Vec<String>,
    pub target_type: RustType,
    pub attributes: Vec<String>,
    pub documentation: Option<String>,
}

/// Rust constant definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustConst {
    pub name: String,
    pub visibility: Visibility,
    pub const_type: RustType,
    pub value: String,
    pub attributes: Vec<String>,
    pub documentation: Option<String>,
}

/// Rust impl block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustImpl {
    pub target_type: RustType,
    pub trait_impl: Option<RustType>,
    pub generics: Vec<String>,
    pub methods: Vec<RustMethod>,
    pub attributes: Vec<String>,
}

/// Rust method definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustMethod {
    pub name: String,
    pub visibility: Visibility,
    pub receiver: Option<RustReceiver>,
    pub parameters: Vec<RustParameter>,
    pub return_type: Option<RustType>,
    pub body: String,
    pub attributes: Vec<String>,
    pub documentation: Option<String>,
}

/// Rust method receiver (self parameter)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustReceiver {
    SelfValue,
    SelfRef,
    SelfMut,
}

/// Rust method parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustParameter {
    pub name: String,
    pub param_type: RustType,
    pub attributes: Vec<String>,
}

/// Visibility levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Crate,
    Super,
    Private,
    Module(String),
}

/// Configuration for Rust code generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustCodegenConfig {
    /// Whether to generate serde derives
    pub serde_support: bool,
    /// Whether to generate validation methods
    pub validation: bool,
    /// Whether to generate builder patterns
    pub builders: bool,
    /// Whether to generate Debug derive
    pub debug: bool,
    /// Whether to generate Clone derive
    pub clone: bool,
    /// Whether to generate PartialEq derive
    pub partial_eq: bool,
    /// Whether to generate Default derive when possible
    pub default: bool,
    /// Custom derives to add to all generated types
    pub custom_derives: Vec<String>,
    /// Custom attributes to add to all generated types
    pub custom_attributes: Vec<String>,
    /// Module name for generated code
    pub module_name: Option<String>,
    /// Whether to use camelCase field names or snake_case
    pub snake_case_fields: bool,
    /// Whether to generate documentation
    pub documentation: bool,
}

impl Default for RustCodegenConfig {
    fn default() -> Self {
        Self {
            serde_support: true,
            validation: true,
            builders: false,
            debug: true,
            clone: true,
            partial_eq: true,
            default: false,  // Disabled by default - enums may not have Default
            custom_derives: vec!["JsonSchema".to_string()],  // Add JsonSchema for MCP support
            custom_attributes: Vec::new(),
            module_name: None,
            snake_case_fields: true,
            documentation: true,
        }
    }
}

impl RustCode {
    /// Create a new empty Rust code structure
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            imports: Vec::new(),
            features: Vec::new(),
        }
    }
    
    /// Add an item to the code
    pub fn add_item(&mut self, item: RustItem) {
        self.items.push(item);
    }
    
    /// Add an import
    pub fn add_import(&mut self, import: String) {
        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }
    
    /// Add multiple imports
    pub fn add_imports(&mut self, imports: &[String]) {
        for import in imports {
            self.add_import(import.clone());
        }
    }
    
    /// Add a feature
    pub fn add_feature(&mut self, feature: String) {
        if !self.features.contains(&feature) {
            self.features.push(feature);
        }
    }
}

impl Default for RustCode {
    fn default() -> Self {
        Self::new()
    }
}

impl RustType {
    /// Check if this type is optional (wrapped in Option)
    pub fn is_optional(&self) -> bool {
        matches!(self, RustType::Option(_))
    }
    
    /// Check if this type is a collection
    pub fn is_collection(&self) -> bool {
        matches!(
            self,
            RustType::Vec(_) | 
            RustType::HashMap(_, _) | 
            RustType::BTreeMap(_, _) |
            RustType::HashSet(_) |
            RustType::BTreeSet(_) |
            RustType::Array(_, _) |
            RustType::Slice(_)
        )
    }
    
    /// Check if this type is a primitive
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            RustType::Bool |
            RustType::I8 | RustType::I16 | RustType::I32 | RustType::I64 | RustType::I128 | RustType::Isize |
            RustType::U8 | RustType::U16 | RustType::U32 | RustType::U64 | RustType::U128 | RustType::Usize |
            RustType::F32 | RustType::F64 |
            RustType::Char | RustType::Str | RustType::String
        )
    }
    
    /// Check if this type needs lifetime parameters
    pub fn needs_lifetime(&self) -> bool {
        match self {
            RustType::Str => true,
            RustType::Reference { .. } => true,
            RustType::Slice(_) => true,
            RustType::Vec(inner) |
            RustType::Option(inner) => inner.needs_lifetime(),
            RustType::Tuple(types) => types.iter().any(|t| t.needs_lifetime()),
            _ => false,
        }
    }
}

impl Visibility {
    /// Convert visibility to Rust code string
    pub fn to_string(&self) -> String {
        match self {
            Visibility::Public => "pub".to_string(),
            Visibility::Crate => "pub(crate)".to_string(),
            Visibility::Super => "pub(super)".to_string(),
            Visibility::Private => "".to_string(),
            Visibility::Module(module) => format!("pub(in {})", module),
        }
    }
}