// ABOUTME: Schema parsing module providing JSON Schema parsing and intermediate representation
// ABOUTME: Converts JSON Schema definitions into strongly-typed IR for code generation

pub mod ir;
pub mod parser;
pub mod validator;

pub use ir::*;
pub use parser::*;
pub use validator::*;
