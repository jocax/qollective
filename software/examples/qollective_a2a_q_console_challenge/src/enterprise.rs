// ABOUTME: USS Enterprise - The starship itself and its central computer system
// ABOUTME: A2A Server providing complete agent infrastructure for crew member coordination

//! USS Enterprise - Starship and Central Computer
//!
//! The USS Enterprise A2A Server that provides the complete infrastructure for
//! agent-to-agent communication, including registry, discovery, routing, and
//! health monitoring for all crew members aboard the ship.

use colored::Colorize;

use qollective::{
    error::Result,
    server::a2a::A2AServer,
};

// Import configuration
use crate::config::EnterpriseConfig;

/// Enterprise registry personality
struct EnterprisePersonality;

impl EnterprisePersonality {
    fn startup() -> String {
        "🚀 USS Enterprise NCC-1701-D Central Computer online. Crew registry systems activated.".bright_blue().bold().to_string()
    }
    
    fn registry_online() -> String {
        "📡 Distributed crew registry service operational. Ready for crew member registration.".bright_green().bold().to_string()
    }
    
    fn shutdown() -> String {
        "🖖 USS Enterprise Central Computer signing off. Registry systems secured.".bright_blue().bold().to_string()
    }
}

pub async fn main() -> Result<()> {
    // Initialize TLS crypto provider with error handling
    match rustls::crypto::aws_lc_rs::default_provider().install_default() {
        Ok(_) => {
            println!("{}", "🔒 TLS crypto provider initialized successfully".bright_green().dimmed());
        }
        Err(e) => {
            println!("{} {:?}", "❌ Failed to initialize TLS crypto provider:".bright_red().bold(), e);
            return Err(qollective::error::QollectiveError::validation(
                format!("TLS crypto provider initialization failed: {:?}", e)
            ));
        }
    }
    
    // Initialize logging
    env_logger::init();
    
    println!("{}", EnterprisePersonality::startup());
    println!("{}", "Loading USS Enterprise configuration from config.toml...".bright_blue().dimmed());
    
    // Load configuration from TOML with enhanced error handling
    let config = match EnterpriseConfig::load_default() {
        Ok(config) => config,
        Err(e) => {
            println!("{} {}", "❌ Configuration loading failed:".bright_red().bold(), e);
            println!("{}", "🔧 Suggested fixes:".bright_yellow().bold());
            println!("{}", "   - Ensure config.toml exists in the project root".dimmed());
            println!("{}", "   - Verify TOML syntax is valid".dimmed());
            println!("{}", "   - Check file permissions are readable".dimmed());
            return Err(qollective::error::QollectiveError::validation(format!("Failed to load config.toml: {}", e)));
        }
    };
    
    println!("{} {}", "✅ Configuration loaded:".bright_green().bold(), "config.toml parsed successfully".bright_yellow());
    println!("{} {}", "🚢 Server ID:".bright_cyan(), config.enterprise.server_id.bright_yellow());
    println!("{} {}", "👥 Max crew size:".bright_cyan(), config.a2a_server.registry.max_agents.to_string().bright_yellow());
    println!("{} {}", "🔐 TLS enabled:".bright_cyan(), config.tls.enabled.to_string().bright_yellow());
    
    // Validate TLS configuration if enabled
    if config.tls.enabled {
        println!("{}", "🔍 Validating TLS certificate configuration...".bright_blue().dimmed());
        
        match config.tls.validate_certificate_paths() {
            Ok(()) => {
                println!("{}", "✅ All TLS certificates validated successfully".bright_green());
            }
            Err(e) => {
                println!("{} {}", "❌ TLS certificate validation failed:".bright_red().bold(), e);
                println!("{}", "🔧 TLS Configuration Issues:".bright_yellow().bold());
                
                // Display detailed certificate path information
                let paths_summary = config.tls.get_certificate_paths_summary();
                for line in paths_summary.lines() {
                    println!("   {}", line.dimmed());
                }
                
                // Display environment variable overrides if any
                let env_summary = config.tls.get_env_override_summary();
                println!("\n{}", "🌍 Environment Variables:".bright_cyan().bold());
                for line in env_summary.lines() {
                    println!("   {}", line.dimmed());
                }
                
                println!("\n{}", "🔧 Suggested fixes:".bright_yellow().bold());
                println!("{}", "   - Verify certificate files exist at the specified paths".dimmed());
                println!("{}", "   - Check file permissions are readable".dimmed());
                println!("{}", "   - Ensure certificates are in PEM format".dimmed());
                println!("{}", "   - Verify certificate and key match".dimmed());
                println!("{}", "   - Check CA certificate can validate client certificate".dimmed());
                
                return Err(qollective::error::QollectiveError::validation(format!("TLS certificate validation failed: {}", e)));
            }
        }
        
        // Test TLS client configuration creation
        println!("{}", "🔧 Testing TLS client configuration...".bright_blue().dimmed());
        let framework_tls_config = config.tls.to_framework_tls_config();
        
        match framework_tls_config.create_client_config().await {
            Ok(_) => {
                println!("{}", "✅ TLS client configuration created successfully".bright_green());
                println!("{}", "🔐 Mutual TLS authentication ready".bright_green().dimmed());
            }
            Err(e) => {
                println!("{} {}", "❌ TLS client configuration failed:".bright_red().bold(), e);
                println!("{}", "🔧 Suggested fixes:".bright_yellow().bold());
                println!("{}", "   - Verify certificate and private key are compatible".dimmed());
                println!("{}", "   - Check certificate chain is valid".dimmed());
                println!("{}", "   - Ensure certificates are not expired".dimmed());
                println!("{}", "   - Verify certificate format is correct".dimmed());
                
                return Err(qollective::error::QollectiveError::validation(format!("TLS client configuration failed: {}", e)));
            }
        }
    } else {
        println!("{}", "⚠️  TLS is disabled - connections will not be encrypted".bright_yellow().bold());
    }
    
    // Convert configuration to framework configs
    println!("{}", "🔧 Converting configuration to framework structures...".bright_blue().dimmed());
    let nats_client_config = config.nats.to_framework_client_config(&config.tls);
    let server_config = config.a2a_server.to_framework_config(&config.enterprise, nats_client_config);
    
    // Create A2A Server with enhanced error handling
    println!("{}", "🔧 Creating A2A Server from configuration...".bright_blue().dimmed());
    let mut enterprise_server = match A2AServer::new(server_config).await {
        Ok(server) => {
            println!("{}", "✅ A2A Server created successfully".bright_green());
            server
        }
        Err(e) => {
            println!("{} {}", "❌ A2A Server creation failed:".bright_red().bold(), e);
            
            // Provide specific troubleshooting based on error type
            let error_str = e.to_string().to_lowercase();
            println!("{}", "🔧 Troubleshooting suggestions:".bright_yellow().bold());
            
            if error_str.contains("connection") || error_str.contains("timeout") {
                println!("{}", "   🌐 Network/Connection Issues:".bright_cyan());
                println!("{}", "     - Verify NATS server is running and accessible".dimmed());
                println!("{}", "     - Check network connectivity to NATS server".dimmed());
                println!("{}", "     - Verify NATS server is configured for TLS if enabled".dimmed());
                println!("{}", "     - Check firewall settings allow connection".dimmed());
            }
            
            if error_str.contains("tls") || error_str.contains("certificate") {
                println!("{}", "   🔒 TLS/Certificate Issues:".bright_red());
                println!("{}", "     - Verify NATS server TLS configuration matches client".dimmed());
                println!("{}", "     - Check mutual TLS is properly configured on server".dimmed());
                println!("{}", "     - Ensure server certificate trusts the client certificate".dimmed());
                println!("{}", "     - Verify certificate subject names match expected values".dimmed());
            }
            
            if error_str.contains("auth") || error_str.contains("permission") {
                println!("{}", "   🔑 Authentication Issues:".bright_yellow());
                println!("{}", "     - Check NATS server authentication configuration".dimmed());
                println!("{}", "     - Verify client certificates have required permissions".dimmed());
                println!("{}", "     - Ensure NATS subjects are accessible".dimmed());
            }
            
            return Err(e);
        }
    };
    
    println!("{}", EnterprisePersonality::registry_online());
    println!("{}", "🖥️  USS Enterprise A2A Server initialized successfully".bright_green().bold());
    println!("{}", "📡 Enterprise crew can connect via A2A communication channels:".bright_green().dimmed());
    println!("{}", format!("   - {} (agent registration)", config.a2a_server.subjects.agent_registration).dimmed());
    println!("{}", format!("   - {} (registry announcements)", config.a2a_server.subjects.agent_registry_announce).dimmed());
    println!("{}", format!("   - {} (agent discovery)", config.a2a_server.subjects.agent_discovery).dimmed());
    println!("{}", format!("   - {} (health monitoring)", config.a2a_server.subjects.agent_health).dimmed());
    println!("{}", format!("   - {} (capability queries)", config.a2a_server.subjects.agent_capabilities).dimmed());
    
    // Start the A2A server with connection retry logic
    println!("{}", "🚀 Starting USS Enterprise A2A Server...".bright_blue().bold());
    
    // Implement startup retry logic for transient network issues
    let max_startup_retries = 3;
    let mut startup_attempt = 1;
    
    loop {
        match enterprise_server.start().await {
            Ok(()) => {
                println!("{}", "✅ USS Enterprise A2A Server operational".bright_green().bold());
                break;
            }
            Err(e) => {
                println!("{} {} (attempt {}/{})", 
                        "❌ Server startup failed:".bright_red().bold(), 
                        e, startup_attempt, max_startup_retries);
                
                if startup_attempt >= max_startup_retries {
                    println!("{}", "💥 Maximum startup attempts exceeded".bright_red().bold());
                    println!("{}", "🔧 Final troubleshooting steps:".bright_yellow().bold());
                    println!("{}", "   - Check NATS server logs for connection issues".dimmed());
                    println!("{}", "   - Verify network connectivity with: telnet <nats-host> <port>".dimmed());
                    println!("{}", "   - Test TLS handshake with: openssl s_client -connect <host>:<port>".dimmed());
                    println!("{}", "   - Review Enterprise configuration settings".dimmed());
                    
                    return Err(e);
                }
                
                startup_attempt += 1;
                let retry_delay = std::time::Duration::from_secs(2 * startup_attempt as u64);
                println!("{} Retrying in {:?}...", "🔄".bright_yellow(), retry_delay);
                tokio::time::sleep(retry_delay).await;
            }
        }
    }
    
    println!("{}", "👀 Monitoring for crew member activities:".bright_green().dimmed());
    println!("{}", "   - Agent registrations and deregistrations".dimmed());
    println!("{}", "   - Health status updates and heartbeats".dimmed());
    println!("{}", "   - Capability queries and discoveries".dimmed());
    println!("{}", "   - Registry events and announcements".dimmed());
    
    if config.tls.enabled {
        println!("{}", "🔐 All connections secured with mutual TLS authentication".bright_green().dimmed());
    }
    
    println!("{}", "");
    println!("{}", "📝 Log levels: INFO=server events, DEBUG=detailed traces".bright_cyan().dimmed());
    
    // Create a signal handler for graceful shutdown with error handling
    tokio::select! {
        // Wait for Ctrl+C
        _ = tokio::signal::ctrl_c() => {
            println!("\n{}", "🛑 Shutdown signal received...".bright_yellow().bold());
            
            // Attempt graceful shutdown
            println!("{}", "🔄 Initiating graceful shutdown...".bright_blue().dimmed());
            
            // Give the server time to clean up connections
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            
            println!("{}", EnterprisePersonality::shutdown());
            println!("{}", "✅ USS Enterprise shutdown complete".bright_green().bold());
        }
        
        // Keep the server running indefinitely
        _ = std::future::pending::<()>() => {
            // This will never complete, keeping the server alive
        }
    }
    
    Ok(())
}