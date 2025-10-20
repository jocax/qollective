//! Response output formatting with colored display
//!
//! Provides pretty-printed, colored output for MCP responses

use crate::constants::*;
use crate::templates::TemplateInfo;
use anyhow::Result;
use colored::*;
use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use std::fs::File;
use std::io::Write;

/// Print a response envelope with colored output
///
/// # Arguments
/// * `envelope` - Response envelope to display
/// * `verbose` - Whether to include full envelope details
/// * `use_color` - Whether to use colored output
pub fn print_response(envelope: &Envelope<McpData>, verbose: bool, use_color: bool) {
    // Disable colors if requested
    if !use_color {
        colored::control::set_override(false);
    }

    // Print separator
    println!("{}", "=".repeat(80).bright_blue());

    // Print metadata section
    print_metadata(&envelope.meta, use_color);

    // Print payload section
    print_payload(&envelope.payload, use_color);

    // Print error section if present
    if let Some(error) = &envelope.error {
        println!();
        println!("{}", format!("{} === Error ===", ERROR_PREFIX).red().bold());
        println!("{}", format!("Code: {}", error.code).red());
        println!("{}", format!("Message: {}", error.message).red());

        if let Some(details) = &error.details {
            println!(
                "{}",
                format!(
                    "Details: {}",
                    serde_json::to_string_pretty(details).unwrap_or_else(|_| "N/A".to_string())
                )
                .red()
            );
        }
    }

    // Print verbose section if requested
    if verbose {
        println!();
        println!(
            "{}",
            format!("{} === Full Envelope (Verbose) ===", INFO_PREFIX)
                .yellow()
                .bold()
        );
        let json = serde_json::to_string_pretty(envelope)
            .unwrap_or_else(|_| "Failed to serialize envelope".to_string());
        println!("{}", json.dimmed());
    }

    // Print separator
    println!("{}", "=".repeat(80).bright_blue());
}

/// Print envelope metadata
fn print_metadata(meta: &qollective::envelope::meta::Meta, _use_color: bool) {
    println!();
    println!("{}", "=== Metadata ===".cyan().bold());

    if let Some(request_id) = &meta.request_id {
        println!("  {}: {}", "Request ID".cyan(), request_id);
    }

    if let Some(tenant) = &meta.tenant {
        println!("  {}: {}", "Tenant".cyan(), tenant);
    }

    if let Some(timestamp) = &meta.timestamp {
        println!("  {}: {}", "Timestamp".cyan(), timestamp);
    }

    if let Some(version) = &meta.version {
        println!("  {}: {}", "Version".cyan(), version);
    }

    if let Some(duration) = &meta.duration {
        println!("  {}: {}ms", "Duration".cyan(), duration);
    }
}

/// Print MCP payload data
fn print_payload(payload: &McpData, _use_color: bool) {
    println!();

    // Print tool response if present
    if let Some(tool_response) = &payload.tool_response {
        if tool_response.is_error == Some(true) {
            println!("{}", format!("{} === Error Result ===", ERROR_PREFIX).red().bold());

            for (i, content) in tool_response.content.iter().enumerate() {
                println!("  {} Content Block {}:", ERROR_PREFIX, i + 1);
                let json = serde_json::to_string_pretty(content)
                    .unwrap_or_else(|_| format!("{:?}", content));
                println!("{}", indent_text(&json, 4).red());
            }
        } else {
            println!(
                "{}",
                format!("{} === Tool Result ===", SUCCESS_PREFIX).green().bold()
            );

            for (i, content) in tool_response.content.iter().enumerate() {
                println!("  {} Content Block {}:", SUCCESS_PREFIX, i + 1);
                let json = serde_json::to_string_pretty(content)
                    .unwrap_or_else(|_| format!("{:?}", content));
                println!("{}", indent_text(&json, 4));
            }
        }
    }

    // Print tool call if present (for request display)
    if let Some(tool_call) = &payload.tool_call {
        println!("{}", "=== Tool Call ===".yellow().bold());
        println!("  {}: {}", "Tool Name".yellow(), tool_call.params.name);

        if let Some(args) = &tool_call.params.arguments {
            println!("  {}:", "Arguments".yellow());
            let json = serde_json::to_string_pretty(args)
                .unwrap_or_else(|_| "Failed to serialize arguments".to_string());
            println!("{}", indent_text(&json, 4));
        }
    }

    // Print discovery data if present
    if let Some(discovery) = &payload.discovery_data {
        println!("{}", "=== Discovery Data ===".magenta().bold());
        println!("  {}: {}", "Query Type".magenta(), discovery.query_type);

        if let Some(tools) = &discovery.tools {
            println!("  {}: {} tools", "Tools".magenta(), tools.len());
        }

        if let Some(server_info) = &discovery.server_info {
            println!(
                "  {}: {}",
                "Server Name".magenta(),
                server_info.server_name
            );
        }
    }
}

/// Print a list of templates in a formatted table
///
/// # Arguments
/// * `templates` - List of template information
/// * `use_color` - Whether to use colored output
pub fn print_template_list(templates: &[TemplateInfo], use_color: bool) {
    if !use_color {
        colored::control::set_override(false);
    }

    if templates.is_empty() {
        println!(
            "{} No templates found",
            WARNING_PREFIX.yellow()
        );
        return;
    }

    println!(
        "{} Found {} template(s):",
        SUCCESS_PREFIX.green(),
        templates.len()
    );
    println!();

    // Group by server
    let mut current_server: Option<&str> = None;

    for template in templates {
        // Print server header when it changes
        if current_server != Some(&template.server_name) {
            println!();
            println!(
                "{}",
                format!("Server: {}", template.server_name).bright_blue().bold()
            );
            println!("{}", "-".repeat(60).blue());
            current_server = Some(&template.server_name);
        }

        // Print template info
        print!("  {} ", SUCCESS_PREFIX.green());
        print!("{}", template.template_name.bright_white());

        if let Some(tool_name) = &template.tool_name {
            print!(" {} {}", "→".dimmed(), tool_name.cyan());
        }

        println!();
        println!("      {}: {}", "Path".dimmed(), template.path.display().to_string().dimmed());
    }

    println!();
}

/// Print an error message
///
/// # Arguments
/// * `error` - Error message to display
/// * `use_color` - Whether to use colored output
pub fn print_error(error: &str, use_color: bool) {
    if !use_color {
        colored::control::set_override(false);
    }

    eprintln!("{} {}", ERROR_PREFIX.red().bold(), error.red());
}

/// Print a success message
///
/// # Arguments
/// * `message` - Success message to display
/// * `use_color` - Whether to use colored output
pub fn print_success(message: &str, use_color: bool) {
    if !use_color {
        colored::control::set_override(false);
    }

    println!("{} {}", SUCCESS_PREFIX.green().bold(), message.green());
}

/// Print an info message
///
/// # Arguments
/// * `message` - Info message to display
/// * `use_color` - Whether to use colored output
pub fn print_info(message: &str, use_color: bool) {
    if !use_color {
        colored::control::set_override(false);
    }

    println!("{} {}", INFO_PREFIX.blue().bold(), message);
}

/// Save response envelope to file as pretty-printed JSON
///
/// # Arguments
/// * `envelope` - Response envelope to save
/// * `path` - File path to write to
/// * `use_color` - Whether to use colored output for messages
///
/// # Returns
/// * `Result<()>` - Success or IO error
pub fn save_response_to_file(
    envelope: &Envelope<McpData>,
    path: &str,
    use_color: bool,
) -> Result<()> {
    // Pretty-print JSON
    let json = serde_json::to_string_pretty(envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))?;

    // Write to file
    let mut file = File::create(path)
        .map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", path, e))?;

    file.write_all(json.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write to file {}: {}", path, e))?;

    // Print success message
    print_success(
        &format!("Response saved to: {}", path),
        use_color,
    );

    Ok(())
}

/// Indent text by a number of spaces
fn indent_text(text: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_text() {
        let text = "line1\nline2\nline3";
        let indented = indent_text(text, 2);
        assert_eq!(indented, "  line1\n  line2\n  line3");
    }

    #[test]
    fn test_indent_single_line() {
        let text = "single line";
        let indented = indent_text(text, 4);
        assert_eq!(indented, "    single line");
    }
}
