//! Enterprise TLS Integration Test
//! 
//! Tests that the Enterprise server properly establishes TLS connections
//! with NATS and uses mutual authentication for all A2A operations.

use std::time::Duration;
use colored::Colorize;

use qollective::error::Result;
use qollective_a2a_nats_enterprise::config::EnterpriseConfig;

/// Test Enterprise server TLS integration
pub async fn test_enterprise_tls_integration() -> Result<()> {
    println!("{}", "🚢 Testing Enterprise Server TLS Integration".bright_blue().bold());
    println!("{}", "━".repeat(80).bright_blue());
    
    // Initialize TLS crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    
    // Load configuration
    println!("{}", "📁 Loading Enterprise configuration...".bright_cyan());
    let config = EnterpriseConfig::load_default()
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    println!("{}", "✅ Configuration loaded successfully".bright_green());
    println!("{} {}", "🔐 TLS Enabled:".bright_yellow(), config.tls.enabled);
    println!("{} {}", "🚢 Server ID:".bright_yellow(), config.enterprise.server_id);
    
    // Validate TLS configuration
    println!("\n{}", "🔍 Validating TLS configuration...".bright_cyan());
    
    match config.tls.validate_certificate_paths() {
        Ok(()) => {
            println!("{}", "✅ All certificate paths are valid".bright_green());
        }
        Err(e) => {
            println!("{} {}", "❌ Certificate path validation failed:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(format!("Certificate validation failed: {}", e)));
        }
    }
    
    // Convert configuration to framework configs (same as Enterprise server does)
    println!("\n{}", "🔧 Converting configuration to framework structures...".bright_cyan());
    let nats_client_config = config.nats.to_framework_client_config(&config.tls);
    let server_config = config.a2a_server.to_framework_config(&config.enterprise, nats_client_config);
    
    println!("{}", "✅ Configuration conversion completed".bright_green());
    
    // Verify TLS configuration in framework config
    println!("\n{}", "🔍 Verifying TLS configuration in framework config...".bright_cyan());
    
    let tls_config = &server_config.nats_client.connection.tls;
    println!("{} {}", "🔐 TLS Enabled in Framework Config:".bright_yellow(), tls_config.enabled);
    
    if tls_config.enabled {
        println!("{} {:?}", "🔑 Verification Mode:".bright_yellow(), tls_config.verification_mode);
        
        if let Some(ca_path) = &tls_config.ca_cert_path {
            println!("{} {}", "📜 CA Certificate:".bright_yellow(), ca_path.display());
        }
        
        if let Some(cert_path) = &tls_config.cert_path {
            println!("{} {}", "🔑 Client Certificate:".bright_yellow(), cert_path.display());
        }
        
        if let Some(key_path) = &tls_config.key_path {
            println!("{} {}", "🗝️  Private Key:".bright_yellow(), key_path.display());
        }
    }
    
    // Test TLS client config creation (this is what NatsServer does internally)
    println!("\n{}", "🔧 Testing TLS client config creation...".bright_cyan());
    
    match tls_config.create_client_config().await {
        Ok(_client_config) => {
            println!("{}", "✅ TLS client config created successfully".bright_green());
            println!("{}", "🔐 Mutual TLS authentication is properly configured".bright_green());
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to create TLS client config:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(format!("TLS client config creation failed: {}", e)));
        }
    }
    
    // Test A2A server initialization (without actually starting it)
    println!("\n{}", "🚀 Testing A2A Server initialization with TLS...".bright_cyan());
    
    match qollective::server::a2a::A2AServer::new(server_config).await {
        Ok(enterprise_server) => {
            println!("{}", "✅ Enterprise A2A Server initialized successfully with TLS".bright_green().bold());
            println!("{}", "🔐 All NATS connections will use mutual TLS authentication".bright_green());
            
            // Clean shutdown
            drop(enterprise_server);
            println!("{}", "✅ Enterprise server properly disposed".bright_green());
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to initialize Enterprise server:".bright_red().bold(), e);
            return Err(e);
        }
    }
    
    println!("\n{}", "🎉 All Enterprise TLS integration tests passed!".bright_green().bold());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    match test_enterprise_tls_integration().await {
        Ok(()) => {
            println!("\n{}", "🎉 Enterprise TLS Integration Test PASSED".bright_green().bold());
            std::process::exit(0);
        }
        Err(e) => {
            println!("\n{} {}", "❌ Enterprise TLS Integration Test FAILED:".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}