// ABOUTME: Library interface for the Qollective code generator
// ABOUTME: Exposes schema parsing and code generation functionality

pub mod cli;
pub mod codegen;
pub mod commands;
pub mod schema;

pub use cli::{Cli, Commands, GenerateArgs, InfoArgs, InitArgs, ValidateArgs};
pub use codegen::{DirectTypifyGenerator, DirectTypifyError};
pub use commands::{handle_generate, handle_info, handle_init, handle_validate};
pub use schema::{Schema, SchemaError, SchemaParser, SchemaType, SchemaValidator};
