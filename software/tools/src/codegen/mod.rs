// ABOUTME: Rust code generation module for converting JSON Schema to Rust code
// ABOUTME: Provides both direct typify integration and custom Rust code generation

pub mod direct_typify;
pub mod integer_type_selection;
pub mod rust;
pub mod types;

#[cfg(test)]
mod tests;

// Export both implementations
pub use direct_typify::*;
pub use rust::*;
pub use types::*;
