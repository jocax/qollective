//! NATS TLS Connection Test
//! 
//! Comprehensive test suite for validating NATS TLS connection establishment
//! with proper certificate validation, mutual authentication, and error handling.

use std::time::Duration;
use std::path::PathBuf;
use colored::Colorize;
use futures::StreamExt;

use qollective::error::Result;
use qollective_a2a_nats_enterprise::config::EnterpriseConfig;

/// Test NATS TLS connection establishment
pub async fn test_nats_tls_connection() -> Result<()> {
    println!("{}", "🔐 Testing NATS TLS Connection Establishment".bright_blue().bold());
    println!("{}", "━".repeat(80).bright_blue());
    
    // Initialize TLS crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    
    // Load configuration
    println!("{}", "📁 Loading configuration from config.toml...".bright_cyan());
    let config = EnterpriseConfig::load_default()
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    // Convert to framework config
    let nats_client_config = config.nats.to_framework_client_config(&config.tls);
    
    println!("{}", "✅ Configuration loaded successfully".bright_green());
    println!("{} {}", "🔗 NATS URLs:".bright_yellow(), format!("{:?}", nats_client_config.connection.urls));
    println!("{} {}", "🔐 TLS Enabled:".bright_yellow(), config.tls.enabled);
    println!("{} {}", "🔑 Verification Mode:".bright_yellow(), config.tls.verification_mode);
    
    // Validate certificate paths before attempting connection
    println!("\n{}", "🔍 Validating certificate paths...".bright_cyan());
    
    match config.tls.validate_certificate_paths() {
        Ok(()) => {
            println!("{}", "✅ All certificate paths are valid".bright_green());
        }
        Err(e) => {
            println!("{} {}", "❌ Certificate path validation failed:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(format!("Certificate validation failed: {}", e)));
        }
    }
    
    // Display certificate paths being used
    println!("{}", config.tls.get_certificate_paths_summary());
    
    // Test TLS connection establishment
    println!("\n{}", "🔌 Testing NATS TLS connection establishment...".bright_cyan());
    
    let connection_result = test_tls_handshake(&nats_client_config).await;
    
    match connection_result {
        Ok(()) => {
            println!("{}", "✅ NATS TLS connection established successfully".bright_green().bold());
            println!("{}", "🔐 TLS handshake completed with certificate validation".bright_green());
        }
        Err(e) => {
            println!("{} {}", "❌ NATS TLS connection failed:".bright_red().bold(), e);
            
            // Provide detailed troubleshooting information
            println!("\n{}", "🔧 Troubleshooting Information:".bright_yellow().bold());
            println!("{}", config.tls.get_env_override_summary());
            
            return Err(e);
        }
    }
    
    // Test connection resilience
    println!("\n{}", "🔄 Testing TLS connection resilience...".bright_cyan());
    test_connection_resilience(&nats_client_config).await?;
    
    println!("\n{}", "🎉 All NATS TLS connection tests passed successfully!".bright_green().bold());
    
    Ok(())
}

/// Test TLS handshake with the NATS server
async fn test_tls_handshake(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    use async_nats::ConnectOptions;
    use std::path::PathBuf;
    
    // Extract TLS configuration
    let tls_config = &nats_config.connection.tls;
    
    if !tls_config.enabled {
        return Err(qollective::error::QollectiveError::validation("TLS is not enabled in configuration".to_string()));
    }
    
    // Build NATS connection options with TLS
    let mut connect_options = ConnectOptions::new();
    
    // Set connection timeout
    connect_options = connect_options.connection_timeout(Duration::from_millis(nats_config.connection.connection_timeout_ms));
    
    // Configure TLS options
    if let (Some(ca_path), Some(cert_path), Some(key_path)) = 
        (&tls_config.ca_cert_path, &tls_config.cert_path, &tls_config.key_path) {
        
        println!("{} {}", "📜 CA Certificate:".bright_yellow(), ca_path.display());
        println!("{} {}", "🔑 Client Certificate:".bright_yellow(), cert_path.display());
        println!("{} {}", "🗝️  Private Key:".bright_yellow(), key_path.display());
        
        // Load certificates
        let ca_cert = load_certificate_file(ca_path)?;
        let client_cert = load_certificate_file(cert_path)?;
        let client_key = load_private_key_file(key_path)?;
        
        println!("{}", "✅ All certificates loaded successfully".bright_green());
        
        // Configure TLS with mutual authentication
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add(ca_cert).map_err(|e| 
            qollective::error::QollectiveError::validation(format!("Failed to add CA certificate: {}", e)))?;
        
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(vec![client_cert], client_key)
            .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to configure client auth: {}", e)))?;
        
        connect_options = connect_options.require_tls(true)
            .tls_client_config(client_config);
    } else {
        return Err(qollective::error::QollectiveError::validation("TLS certificate paths not configured".to_string()));
    }
    
    // Attempt to connect to NATS with TLS
    let url = &nats_config.connection.urls[0];
    println!("{} {}", "🔗 Connecting to:".bright_cyan(), url);
    
    let client = async_nats::connect_with_options(url, connect_options)
        .await
        .map_err(|e| qollective::error::QollectiveError::connection(format!("NATS TLS connection failed: {}", e)))?;
    
    println!("{}", "✅ TLS connection established successfully".bright_green());
    
    // Test basic operations over TLS connection
    println!("{}", "📡 Testing basic operations over TLS...".bright_cyan());
    
    // Subscribe to a test subject
    let test_subject = "tls.connection.test";
    let mut subscriber = client.subscribe(test_subject)
        .await
        .map_err(|e| qollective::error::QollectiveError::connection(format!("Failed to subscribe: {}", e)))?;
    
    // Publish a test message
    let test_message = "TLS connection test message";
    client.publish(test_subject, test_message.into())
        .await
        .map_err(|e| qollective::error::QollectiveError::connection(format!("Failed to publish: {}", e)))?;
    
    // Wait for message with timeout  
    let message = tokio::time::timeout(Duration::from_secs(5), subscriber.next())
        .await
        .map_err(|_| qollective::error::QollectiveError::nats_timeout("Timeout waiting for test message".to_string()))?
        .ok_or_else(|| qollective::error::QollectiveError::connection("No message received".to_string()))?;
    
    let received_message = String::from_utf8_lossy(&message.payload);
    if received_message == test_message {
        println!("{}", "✅ Message roundtrip test successful over TLS".bright_green());
    } else {
        return Err(qollective::error::QollectiveError::validation(
            format!("Message mismatch: sent '{}', received '{}'", test_message, received_message)
        ));
    }
    
    // Close connection gracefully
    drop(subscriber);
    // Note: async_nats::Client doesn't have a close() method - connection is managed automatically
    
    println!("{}", "✅ TLS connection closed gracefully".bright_green());
    
    Ok(())
}

/// Test connection resilience with reconnects
async fn test_connection_resilience(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    println!("{}", "🔄 Testing connection resilience patterns...".bright_cyan());
    
    // This test focuses on configuration validation rather than actual reconnection
    // since we can't simulate NATS server restarts in this environment
    
    // Validate reconnection configuration
    let max_reconnects = nats_config.connection.max_reconnect_attempts.unwrap_or(0);
    let reconnect_timeout = nats_config.connection.reconnect_timeout_ms;
    
    println!("{} {}", "🔄 Max Reconnect Attempts:".bright_yellow(), max_reconnects);
    println!("{} {}ms", "⏱️  Reconnect Timeout:".bright_yellow(), reconnect_timeout);
    
    if max_reconnects > 0 {
        println!("{}", "✅ Reconnection is properly configured".bright_green());
    } else {
        println!("{}", "⚠️  Reconnection is disabled".bright_yellow());
    }
    
    if reconnect_timeout > 1000 && reconnect_timeout < 30000 {
        println!("{}", "✅ Reconnect timeout is in acceptable range".bright_green());
    } else {
        println!("{}", "⚠️  Reconnect timeout may be too aggressive or too long".bright_yellow());
    }
    
    Ok(())
}

/// Load certificate from PEM file
fn load_certificate_file(path: &PathBuf) -> Result<rustls::pki_types::CertificateDer<'static>> {
    use std::fs::File;
    use std::io::BufReader;
    
    let file = File::open(path)
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to open certificate file {}: {}", path.display(), e)))?;
    
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to parse certificate: {}", e)))?;
    
    certs.into_iter().next()
        .ok_or_else(|| qollective::error::QollectiveError::validation(format!("No certificate found in file: {}", path.display())))
}

/// Load private key from PEM file
fn load_private_key_file(path: &PathBuf) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
    use std::fs::File;
    use std::io::BufReader;
    
    let file = File::open(path)
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to open private key file {}: {}", path.display(), e)))?;
    
    let mut reader = BufReader::new(file);
    
    // Try to read a single private key
    if let Some(key) = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to parse private key: {}", e)))? {
        return Ok(key);
    }
    
    Err(qollective::error::QollectiveError::validation(format!("No private key found in file: {}", path.display())))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    match test_nats_tls_connection().await {
        Ok(()) => {
            println!("\n{}", "🎉 NATS TLS Connection Test PASSED".bright_green().bold());
            std::process::exit(0);
        }
        Err(e) => {
            println!("\n{} {}", "❌ NATS TLS Connection Test FAILED:".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}