// ABOUTME: Main entry point for the Qollective code generator CLI
// ABOUTME: Parses command line arguments and dispatches to appropriate command handlers

use anyhow::Result;
use qollective_cli::{handle_generate, handle_info, handle_init, handle_validate, Cli, Commands};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("❌ Error: {}", e);

        // Print error chain for better debugging
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("   Caused by: {}", err);
            source = err.source();
        }

        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_args();

    let verbose = cli.is_verbose();
    let quiet = cli.is_quiet();

    if verbose {
        println!(
            "🚀 Qollective Code Generator v{}",
            env!("CARGO_PKG_VERSION")
        );
    }

    match &cli.command {
        Commands::Generate(args) => {
            handle_generate(args, verbose, quiet)?;
        }
        Commands::Validate(args) => {
            handle_validate(args, verbose, quiet)?;
        }
        Commands::Info(args) => {
            handle_info(args, verbose, quiet)?;
        }
        Commands::Init(args) => {
            handle_init(args, verbose, quiet)?;
        }
    }

    Ok(())
}
