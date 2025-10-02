// ABOUTME: TLS Analysis Binary - Q Console Challenge diagnostics tool
// ABOUTME: Runs comprehensive TLS configuration analysis and generates detailed report

//! TLS Analysis Binary
//! 
//! This binary performs comprehensive analysis of TLS configuration issues
//! in the Q Console Challenge example and generates a detailed diagnostic report.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --bin tls_analysis
//! ```
//! 
//! ## Output
//! 
//! The tool generates a comprehensive report covering:
//! - Certificate path resolution analysis
//! - Configuration validation
//! - NATS connection compatibility
//! - Specific issue identification with resolutions

use colored::Colorize;
use qollective::error::Result;
use qollective_a2a_nats_enterprise::tls_analysis::TlsAnalysisReport;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🔍 Q Console Challenge - TLS Configuration Analysis".bright_blue().bold());
    println!("{}", "USS Enterprise NCC-1701-D TLS Diagnostics".bright_blue().dimmed());
    println!();
    
    // Initialize TLS crypto provider (required for framework)
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    
    // Perform comprehensive analysis
    println!("{}", "⚡ Analyzing TLS configuration...".bright_yellow());
    let report = TlsAnalysisReport::analyze();
    
    // Count issues by severity
    let critical_count = report.issues.iter().filter(|i| matches!(i.severity, qollective_a2a_nats_enterprise::tls_analysis::IssueSeverity::Critical)).count();
    let high_count = report.issues.iter().filter(|i| matches!(i.severity, qollective_a2a_nats_enterprise::tls_analysis::IssueSeverity::High)).count();
    
    // Display quick summary
    if critical_count > 0 {
        println!("{} {} critical issues found", "🚨".red(), critical_count.to_string().red().bold());
    }
    if high_count > 0 {
        println!("{} {} high-priority issues found", "⚠️".yellow(), high_count.to_string().yellow().bold());
    }
    
    if critical_count == 0 && high_count == 0 {
        println!("{}", "✅ No critical or high-priority issues found".bright_green());
    }
    
    println!();
    
    // Generate and display full report
    println!("{}", "📋 Generating detailed analysis report...".bright_cyan());
    let report_text = report.generate_report();
    
    println!("{}", "=".repeat(80).bright_blue());
    println!("{}", report_text);
    println!("{}", "=".repeat(80).bright_blue());
    
    // Quick action summary
    if critical_count > 0 || high_count > 0 {
        println!();
        println!("{}", "🎯 Quick Fix Command:".bright_green().bold());
        println!("{}", "export QOLLECTIVE_TLS_CERT_BASE_PATH=/Users/ms/development/qollective/software/tests/certs".bright_yellow());
        println!();
        println!("{}", "Then rerun the analysis to verify:".bright_green());
        println!("{}", "cargo run --bin tls_analysis".bright_yellow());
    }
    
    println!();
    println!("{}", "🖖 Analysis complete. Live long and prosper.".bright_blue().dimmed());
    
    Ok(())
}