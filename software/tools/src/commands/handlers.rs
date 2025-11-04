// ABOUTME: Command handler implementation for CLI operations
// ABOUTME: Processes CLI commands and orchestrates schema parsing and code generation

use crate::cli::{GenerateArgs, InfoArgs, InitArgs, ValidateArgs};
use crate::codegen::DirectTypifyGenerator;
use crate::schema::{SchemaParser, SchemaValidator};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

/// Handle the generate command
pub fn handle_generate(args: &GenerateArgs, verbose: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!(
            "🔍 Generating code from schema: {}",
            args.schema_file.display()
        );
    }

    // Validate input arguments
    if !args.is_language_supported() {
        bail!(
            "Unsupported language: {}. Supported languages: rust, typescript, java",
            args.language
        );
    }

    if !args.is_format_supported() {
        bail!(
            "Unsupported format: {}. Supported formats: module, crate, single-file",
            args.format
        );
    }

    // Check if schema file exists
    if !args.schema_file.exists() {
        bail!("Schema file not found: {}", args.schema_file.display());
    }

    // Parse the schema
    if verbose {
        println!("📖 Parsing schema file...");
    }

    let mut parser = SchemaParser::new();
    let schema = parser.parse_file(&args.schema_file).with_context(|| {
        format!(
            "Failed to parse schema file: {}",
            args.schema_file.display()
        )
    })?;

    if verbose {
        println!("✅ Schema parsed successfully");
        println!("   Title: {:?}", schema.title);
        println!("   Type: {:?}", schema.schema_type);
        if let Some(desc) = &schema.description {
            println!("   Description: {}", desc);
        }
    }

    // Validate schema (unless skipped)
    if !args.skip_validation {
        if verbose {
            println!("🔍 Validating schema...");
        }

        let validator = SchemaValidator::new();
        validator
            .validate_schema(&schema)
            .context("Schema validation failed")?;

        // Show linting warnings
        let warnings = validator.lint_schema(&schema);
        if !warnings.is_empty() && !quiet {
            println!("⚠️  Linting warnings:");
            for warning in warnings {
                println!("   - {}", warning);
            }
        }

        if verbose {
            println!("✅ Schema validation passed");
        }
    } else if verbose {
        println!("⏭️  Skipping schema validation");
    }

    // Ensure output directory exists
    let output_dir = args
        .ensure_output_dir()
        .context("Failed to create output directory")?;

    if verbose {
        println!("📁 Output directory: {}", output_dir.display());
    }

    // Generate code based on target language
    match args.language.as_str() {
        "rust" => generate_rust_code(&schema, args, verbose, quiet)?,
        "typescript" => {
            if !quiet {
                println!("⚠️  TypeScript generation not yet implemented");
            }
            bail!("TypeScript code generation is not yet implemented");
        }
        "java" => {
            if !quiet {
                println!("⚠️  Java generation not yet implemented");
            }
            bail!("Java code generation is not yet implemented");
        }
        _ => unreachable!("Language validation should have caught this"),
    }

    if !quiet {
        println!("🎉 Code generation completed successfully!");
        println!("📁 Generated files in: {}", output_dir.display());
    }

    Ok(())
}

/// Generate Rust code from schema
fn generate_rust_code(
    _schema: &crate::schema::Schema,
    args: &GenerateArgs,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    if verbose {
        println!("🦀 Generating Rust code...");
    }

    let code_generator = DirectTypifyGenerator::new();

    // Use DirectTypifyGenerator directly with the original schema file
    // This avoids the round-trip through our custom IR and ensures typify gets clean JSON
    let generated_code = code_generator
        .generate_from_file(args.schema_file.to_str().unwrap())
        .context("Failed to generate Rust code")?;

    // Determine output file path
    let package_name = args.get_package_name();
    let output_file = match args.format.as_str() {
        "single-file" => args.output.join(format!("{}.rs", package_name)),
        "module" => {
            let mod_dir = args.output.join(&package_name);
            fs::create_dir_all(&mod_dir)?;
            mod_dir.join("mod.rs")
        }
        "crate" => {
            let crate_dir = args.output.join(&package_name);
            fs::create_dir_all(crate_dir.join("src"))?;

            // Create Cargo.toml
            let cargo_toml = create_cargo_toml(&package_name);
            fs::write(crate_dir.join("Cargo.toml"), cargo_toml)?;

            crate_dir.join("src").join("lib.rs")
        }
        _ => unreachable!("Format validation should have caught this"),
    };

    // Check if file exists and handle overwrite
    if output_file.exists() && !args.force {
        if !quiet {
            eprintln!("❌ Output file already exists: {}", output_file.display());
            eprintln!("   Use --force to overwrite existing files");
        }
        bail!("Output file already exists and --force not specified");
    }

    // Write generated code
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_file, &generated_code).with_context(|| {
        format!(
            "Failed to write generated code to: {}",
            output_file.display()
        )
    })?;

    if verbose {
        println!("✅ Rust code generated");
        println!("📝 Output file: {}", output_file.display());
        println!(
            "📊 Generated {} lines of code",
            generated_code.lines().count()
        );
    }

    Ok(())
}

/// Handle the validate command
pub fn handle_validate(args: &ValidateArgs, verbose: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("🔍 Validating schema: {}", args.schema_file.display());
    }

    // Check if schema file exists
    if !args.schema_file.exists() {
        bail!("Schema file not found: {}", args.schema_file.display());
    }

    // Parse the schema
    if verbose {
        println!("📖 Parsing schema file...");
    }

    let mut parser = SchemaParser::new();
    let schema = parser.parse_file(&args.schema_file).with_context(|| {
        format!(
            "Failed to parse schema file: {}",
            args.schema_file.display()
        )
    })?;

    if verbose || args.detailed {
        println!("✅ Schema parsed successfully");
        println!("   Title: {:?}", schema.title);
        println!("   Type: {:?}", schema.schema_type);
        if let Some(desc) = &schema.description {
            println!("   Description: {}", desc);
        }
    }

    // Validate schema
    if verbose {
        println!("🔍 Validating schema structure...");
    }

    let validator = SchemaValidator::new();
    validator
        .validate_schema(&schema)
        .context("Schema validation failed")?;

    if !quiet {
        println!("✅ Schema validation passed");
    }

    // Run linting checks if requested
    if args.lint {
        if verbose {
            println!("🔍 Running linting checks...");
        }

        let warnings = validator.lint_schema(&schema);
        if warnings.is_empty() {
            if !quiet {
                println!("✅ No linting issues found");
            }
        } else {
            if !quiet {
                println!("⚠️  Linting warnings found:");
                for warning in warnings {
                    println!("   - {}", warning);
                }
            }
        }
    }

    if !quiet {
        println!("🎉 Schema validation completed successfully!");
    }

    Ok(())
}

/// Handle the info command
pub fn handle_info(args: &InfoArgs, _verbose: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("ℹ️  Schema information: {}", args.schema_file.display());
    }

    // Check if schema file exists
    if !args.schema_file.exists() {
        bail!("Schema file not found: {}", args.schema_file.display());
    }

    // Parse the schema
    let mut parser = SchemaParser::new();
    let schema = parser.parse_file(&args.schema_file).with_context(|| {
        format!(
            "Failed to parse schema file: {}",
            args.schema_file.display()
        )
    })?;

    // Display basic information
    if !quiet {
        println!("\n📋 Schema Details:");
        println!("   File: {}", args.schema_file.display());
        println!(
            "   Title: {:?}",
            schema.title.unwrap_or_else(|| "Untitled".to_string())
        );
        println!("   Type: {:?}", schema.schema_type);

        if let Some(desc) = &schema.description {
            println!("   Description: {}", desc);
        }

        // File stats
        if args.stats {
            let file_size = fs::metadata(&args.schema_file)?.len();
            println!("\n📊 File Statistics:");
            println!("   Size: {} bytes", file_size);

            let content = fs::read_to_string(&args.schema_file)?;
            println!("   Lines: {}", content.lines().count());
        }

        // Dependencies (placeholder for future implementation)
        if args.dependencies {
            println!("\n🔗 Dependencies:");
            println!("   (Dependency analysis not yet implemented)");
        }
    }

    Ok(())
}

/// Handle the init command
pub fn handle_init(args: &InitArgs, verbose: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("🆕 Initializing new project: {}", args.project_name);
    }

    if !args.is_template_supported() {
        bail!(
            "Unsupported template: {}. Supported templates: minimal, full, examples",
            args.template
        );
    }

    let target_dir = args.get_target_directory();

    if target_dir.exists() {
        bail!("Directory already exists: {}", target_dir.display());
    }

    // Create project directory
    fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "Failed to create project directory: {}",
            target_dir.display()
        )
    })?;

    if verbose {
        println!("📁 Created directory: {}", target_dir.display());
    }

    // Create template files based on template type
    match args.template.as_str() {
        "minimal" => create_minimal_template(&target_dir, &args.project_name, verbose)?,
        "full" => create_full_template(&target_dir, &args.project_name, verbose)?,
        "examples" => create_examples_template(&target_dir, &args.project_name, verbose)?,
        _ => unreachable!("Template validation should have caught this"),
    }

    if !quiet {
        println!("🎉 Project initialized successfully!");
        println!("📁 Project directory: {}", target_dir.display());
        println!("\n💡 Next steps:");
        println!("   1. cd {}", target_dir.display());
        println!("   2. Edit the schema files to match your API");
        println!("   3. Run: qollective generate schema.json");
    }

    Ok(())
}

/// Create a minimal project template
fn create_minimal_template(dir: &Path, _project_name: &str, verbose: bool) -> Result<()> {
    let schema_content = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Example API Schema",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier"
    },
    "name": {
      "type": "string",
      "description": "Display name"
    }
  },
  "required": ["id", "name"]
}"#;

    let schema_file = dir.join("schema.json");
    fs::write(&schema_file, schema_content)?;

    if verbose {
        println!("📝 Created: {}", schema_file.display());
    }

    Ok(())
}

/// Create a full project template with examples
fn create_full_template(dir: &Path, project_name: &str, verbose: bool) -> Result<()> {
    create_minimal_template(dir, project_name, verbose)?;

    // Add README
    let readme_content = format!(
        r#"# {}

Generated using Qollective framework.

## Usage

1. Edit `schema.json` to define your API structure
2. Generate code: `qollective generate schema.json`
3. Use the generated types in your application

## Files

- `schema.json` - Main API schema definition
- `generated/` - Generated code output directory
"#,
        project_name
    );

    let readme_file = dir.join("README.md");
    fs::write(&readme_file, readme_content)?;

    if verbose {
        println!("📝 Created: {}", readme_file.display());
    }

    Ok(())
}

/// Create an examples template with multiple schema files
fn create_examples_template(dir: &Path, project_name: &str, verbose: bool) -> Result<()> {
    create_full_template(dir, project_name, verbose)?;

    // Create examples directory with additional schemas
    let examples_dir = dir.join("examples");
    fs::create_dir_all(&examples_dir)?;

    if verbose {
        println!("📁 Created: {}", examples_dir.display());
    }

    // Future enhancement: Add example schema files when we have more complex examples
    // This would include comprehensive examples with all metadata sections

    Ok(())
}

/// Create a Cargo.toml for Rust crate output format
fn create_cargo_toml(package_name: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#,
        package_name
    )
}
