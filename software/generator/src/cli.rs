// ABOUTME: Command-line interface module for the Qollective code generator
// ABOUTME: Provides CLI argument parsing, command routing, and user interaction

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Qollective Code Generator CLI
///
/// A powerful code generation tool for the Qollective framework that creates
/// type-safe client libraries from JSON schemas.
#[derive(Parser)]
#[command(name = "qollective")]
#[command(about = "Qollective framework code generator")]
#[command(version = "0.1.0")]
#[command(
    long_about = "Generate type-safe client libraries and server stubs from Qollective JSON schemas"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate code from a schema file
    Generate(GenerateArgs),

    /// Validate a schema file without generating code
    Validate(ValidateArgs),

    /// Show information about a schema file
    Info(InfoArgs),

    /// Initialize a new project with example schemas
    Init(InitArgs),
}

#[derive(Args)]
pub struct GenerateArgs {
    /// Path to the JSON schema file
    #[arg(value_name = "SCHEMA_FILE")]
    pub schema_file: PathBuf,

    /// Output directory for generated code
    #[arg(short, long, default_value = "./generated")]
    pub output: PathBuf,

    /// Target language for code generation
    #[arg(short, long, default_value = "rust")]
    pub language: String,

    /// Output format (module, crate, single-file)
    #[arg(short, long, default_value = "module")]
    pub format: String,

    /// Package/module name for generated code
    #[arg(short, long)]
    pub package_name: Option<String>,

    /// Skip validation and generate code directly
    #[arg(long)]
    pub skip_validation: bool,

    /// Overwrite existing files without confirmation
    #[arg(long)]
    pub force: bool,

    /// Enable schemars::JsonSchema derive and dependency
    #[arg(long, default_value_t = false)]
    pub schemars: bool,

    /// Additional derive traits (comma-separated, e.g., "PartialEq,Hash,Eq")
    #[arg(long)]
    pub additional_derives: Option<String>,
}

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the JSON schema file
    #[arg(value_name = "SCHEMA_FILE")]
    pub schema_file: PathBuf,

    /// Show detailed validation information
    #[arg(short, long)]
    pub detailed: bool,

    /// Run linting checks
    #[arg(short, long)]
    pub lint: bool,
}

#[derive(Args)]
pub struct InfoArgs {
    /// Path to the JSON schema file
    #[arg(value_name = "SCHEMA_FILE")]
    pub schema_file: PathBuf,

    /// Show schema statistics
    #[arg(short, long)]
    pub stats: bool,

    /// Show schema dependencies
    #[arg(short, long)]
    pub dependencies: bool,
}

#[derive(Args)]
pub struct InitArgs {
    /// Project name
    #[arg(value_name = "PROJECT_NAME")]
    pub project_name: String,

    /// Target directory (defaults to project name)
    #[arg(short, long)]
    pub directory: Option<PathBuf>,

    /// Project template (minimal, full, examples)
    #[arg(short, long, default_value = "minimal")]
    pub template: String,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }
}

impl GenerateArgs {
    /// Get the output directory, creating it if necessary
    pub fn ensure_output_dir(&self) -> anyhow::Result<PathBuf> {
        if !self.output.exists() {
            std::fs::create_dir_all(&self.output)?;
        }
        Ok(self.output.clone())
    }

    /// Generate package name from schema file if not provided
    pub fn get_package_name(&self) -> String {
        match &self.package_name {
            Some(name) => name.clone(),
            None => self
                .schema_file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .replace(['-', ' '], "_")
                .to_lowercase(),
        }
    }

    /// Check if the target language is supported
    pub fn is_language_supported(&self) -> bool {
        matches!(self.language.as_str(), "rust" | "typescript" | "java")
    }

    /// Check if the output format is supported
    pub fn is_format_supported(&self) -> bool {
        matches!(self.format.as_str(), "module" | "crate" | "single-file")
    }
}

impl ValidateArgs {
    /// Check if any additional validation flags are set
    pub fn has_additional_checks(&self) -> bool {
        self.detailed || self.lint
    }
}

impl InitArgs {
    /// Get the target directory, defaulting to project name
    pub fn get_target_directory(&self) -> PathBuf {
        match &self.directory {
            Some(dir) => dir.clone(),
            None => PathBuf::from(&self.project_name),
        }
    }

    /// Check if the template is supported
    pub fn is_template_supported(&self) -> bool {
        matches!(self.template.as_str(), "minimal" | "full" | "examples")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_args_package_name_from_file() {
        let args = GenerateArgs {
            schema_file: PathBuf::from("my-api-schema.json"),
            output: PathBuf::from("./generated"),
            language: "rust".to_string(),
            format: "module".to_string(),
            package_name: None,
            skip_validation: false,
            force: false,
            schemars: false,
            additional_derives: None,
        };

        assert_eq!(args.get_package_name(), "my_api_schema");
    }

    #[test]
    fn test_generate_args_explicit_package_name() {
        let args = GenerateArgs {
            schema_file: PathBuf::from("schema.json"),
            output: PathBuf::from("./generated"),
            language: "rust".to_string(),
            format: "module".to_string(),
            package_name: Some("custom_package".to_string()),
            skip_validation: false,
            force: false,
            schemars: false,
            additional_derives: None,
        };

        assert_eq!(args.get_package_name(), "custom_package");
    }

    #[test]
    fn test_language_support_validation() {
        let mut args = GenerateArgs {
            schema_file: PathBuf::from("schema.json"),
            output: PathBuf::from("./generated"),
            language: "rust".to_string(),
            format: "module".to_string(),
            package_name: None,
            skip_validation: false,
            force: false,
            schemars: false,
            additional_derives: None,
        };

        assert!(args.is_language_supported());

        args.language = "python".to_string();
        assert!(!args.is_language_supported());
    }

    #[test]
    fn test_init_args_target_directory() {
        let args = InitArgs {
            project_name: "my_project".to_string(),
            directory: None,
            template: "minimal".to_string(),
        };

        assert_eq!(args.get_target_directory(), PathBuf::from("my_project"));

        let args_with_dir = InitArgs {
            project_name: "my_project".to_string(),
            directory: Some(PathBuf::from("custom_dir")),
            template: "minimal".to_string(),
        };

        assert_eq!(
            args_with_dir.get_target_directory(),
            PathBuf::from("custom_dir")
        );
    }
}
