//! NATS CLI for sending envelope-wrapped MCP requests
//!
//! Command-line tool for testing and interacting with MCP servers via NATS
//! using Qollective's envelope-first architecture.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod client;
mod config;
mod constants;
mod errors;
mod output;
mod templates;

use client::NatsClient;
use config::NatsCliConfig;
use constants::*;
use output::{print_error, print_info, print_response, print_success, print_template_list};
use templates::TemplateManager;

/// NATS CLI for envelope-wrapped MCP communication
#[derive(Parser, Debug)]
#[command(name = "nats-cli")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
    config: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Override log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Send an MCP request to a NATS subject
    Send {
        /// NATS subject to send request to
        #[arg(short, long)]
        subject: String,

        /// Template file path (absolute or relative)
        #[arg(short, long)]
        template: String,

        /// Tenant ID (overrides config default)
        #[arg(long)]
        tenant: Option<i32>,

        /// Request timeout in seconds (overrides config default)
        #[arg(long)]
        timeout: Option<u64>,

        /// Output file path for pretty-printed JSON response
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Template management commands
    Template {
        #[command(subcommand)]
        action: TemplateCommands,
    },
}

/// Template subcommands
#[derive(Subcommand, Debug)]
enum TemplateCommands {
    /// List available templates
    List {
        /// Filter by server name
        #[arg(short, long)]
        server: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider (aws-lc-rs)
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli)?;

    // Load configuration
    let config = if cli.config == DEFAULT_CONFIG_PATH {
        NatsCliConfig::load()?
    } else {
        NatsCliConfig::load_from_file(&cli.config)?
    };

    let use_color = !cli.no_color && config.client.colored_output;

    // Execute command
    match cli.command {
        Commands::Send {
            subject,
            template,
            tenant,
            timeout,
            output,
        } => {
            handle_send(
                &config,
                &subject,
                &template,
                tenant,
                timeout,
                output,
                cli.verbose,
                use_color,
            )
            .await?;
        }
        Commands::Template { action } => match action {
            TemplateCommands::List { server } => {
                handle_template_list(server.as_deref(), use_color)?;
            }
        },
    }

    Ok(())
}

/// Handle send command
async fn handle_send(
    config: &NatsCliConfig,
    subject: &str,
    template_path: &str,
    tenant: Option<i32>,
    timeout: Option<u64>,
    output: Option<String>,
    verbose: bool,
    use_color: bool,
) -> Result<()> {
    print_info(
        &format!("Sending MCP request to subject: {}", subject),
        use_color,
    );

    // Load template
    let template_manager = TemplateManager::with_default_dir()?;
    let request = template_manager.load_template(template_path)?;

    print_info(
        &format!("Loaded template: tool={}", request.params.name),
        use_color,
    );

    // Create NATS client with custom timeout if provided
    let mut client_config = config.clone();
    if let Some(timeout_secs) = timeout {
        if timeout_secs < MIN_TIMEOUT_SECS || timeout_secs > MAX_TIMEOUT_SECS {
            print_error(
                &format!(
                    "Timeout must be between {} and {} seconds",
                    MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS
                ),
                use_color,
            );
            std::process::exit(1);
        }
        client_config.client.default_timeout_secs = timeout_secs;
    }

    let client = NatsClient::new(&client_config).await?;

    // Determine tenant ID
    let tenant_id = tenant.unwrap_or(config.client.default_tenant_id);

    print_info(
        &format!("Using tenant ID: {}", tenant_id),
        use_color,
    );

    // Send request
    print_info("Waiting for response...", use_color);

    match client.send_request(subject, request, tenant_id).await {
        Ok(response) => {
            if let Some(output_path) = output {
                // Save to file with pretty-print
                output::save_response_to_file(&response, &output_path, use_color)?;
            } else {
                // Console output (existing behavior)
                print_success("Received response", use_color);
                println!();
                print_response(&response, verbose, use_color);
            }
        }
        Err(e) => {
            print_error(&format!("Request failed: {}", e), use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle template list command
fn handle_template_list(server_filter: Option<&str>, use_color: bool) -> Result<()> {
    let template_manager = TemplateManager::with_default_dir()?;

    if let Some(server) = server_filter {
        print_info(&format!("Listing templates for server: {}", server), use_color);
    } else {
        print_info("Listing all available templates", use_color);
    }

    let templates = template_manager.list_templates(server_filter)?;

    println!();
    print_template_list(&templates, use_color);

    Ok(())
}

/// Initialize logging based on CLI arguments and config
fn init_logging(cli: &Cli) -> Result<()> {
    // Determine log level
    let log_level = cli
        .log_level
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_LOG_LEVEL);

    let level_filter = match log_level {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => {
            eprintln!("Invalid log level: {}, defaulting to info", log_level);
            LevelFilter::INFO
        }
    };

    // Build filter
    let filter = EnvFilter::builder()
        .with_default_directive(level_filter.into())
        .from_env_lossy();

    // Initialize subscriber
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    Ok(())
}
