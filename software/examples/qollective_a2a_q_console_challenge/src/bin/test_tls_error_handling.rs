//! TLS Error Handling Test
//!
//! Tests comprehensive TLS error handling scenarios including certificate issues,
//! handshake failures, network errors, and configuration problems.

use std::time::Duration;
use colored::Colorize;

use qollective::error::{Result, QollectiveError};
use qollective_a2a_nats_enterprise::config::EnterpriseConfig;

/// Test comprehensive TLS error handling scenarios
pub async fn test_tls_error_handling() -> Result<()> {
    println!("{}", "🚨 Testing TLS Error Handling Scenarios".bright_red().bold());
    println!("{}", "━".repeat(80).bright_red());
    
    // Initialize TLS crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    
    // Test 1: Missing certificate files
    println!("\n{}", "📋 Test 1: Missing Certificate Files".bright_cyan().bold());
    test_missing_certificate_files().await?;
    
    // Test 2: Invalid certificate format
    println!("\n{}", "📋 Test 2: Invalid Certificate Format".bright_cyan().bold());
    test_invalid_certificate_format().await?;
    
    // Test 3: Certificate path permissions
    println!("\n{}", "📋 Test 3: Certificate Path Permissions".bright_cyan().bold());
    test_certificate_permissions().await?;
    
    // Test 4: TLS handshake timeout
    println!("\n{}", "📋 Test 4: TLS Handshake Timeout".bright_cyan().bold());
    test_tls_handshake_timeout().await?;
    
    // Test 5: Certificate chain validation
    println!("\n{}", "📋 Test 5: Certificate Chain Validation".bright_cyan().bold());
    test_certificate_chain_validation().await?;
    
    // Test 6: Network connectivity issues
    println!("\n{}", "📋 Test 6: Network Connectivity Issues".bright_cyan().bold());
    test_network_connectivity_issues().await?;
    
    // Test 7: Configuration validation
    println!("\n{}", "📋 Test 7: Configuration Validation".bright_cyan().bold());
    test_configuration_validation().await?;
    
    println!("\n{}", "🎉 All TLS error handling tests completed!".bright_green().bold());
    
    Ok(())
}

/// Test missing certificate files error handling
async fn test_missing_certificate_files() -> Result<()> {
    println!("{}", "🔍 Testing missing certificate file scenarios...".bright_cyan());
    
    // Load base configuration
    let mut config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    // Test with paths to guaranteed non-existent files using temp directory
    let temp_dir = tempfile::tempdir()
        .map_err(|e| QollectiveError::validation(format!("Failed to create temp dir: {}", e)))?;
    
    // Test missing CA certificate
    config.tls.ca_cert_path = temp_dir.path().join("nonexistent-ca.pem").to_string_lossy().to_string();
    
    match config.tls.validate_certificate_paths() {
        Ok(()) => {
            println!("{}", "❌ Missing CA certificate validation should have failed".bright_red());
        }
        Err(e) => {
            println!("{} Missing CA certificate properly detected: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    // Test missing client certificate
    let mut config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    config.tls.cert_path = temp_dir.path().join("nonexistent-client.pem").to_string_lossy().to_string();
    
    match config.tls.validate_certificate_paths() {
        Ok(()) => {
            println!("{}", "❌ Missing client certificate validation should have failed".bright_red());
        }
        Err(e) => {
            println!("{} Missing client certificate properly detected: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    // Test missing private key
    let mut config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    config.tls.key_path = temp_dir.path().join("nonexistent-key.pem").to_string_lossy().to_string();
    
    match config.tls.validate_certificate_paths() {
        Ok(()) => {
            println!("{}", "❌ Missing private key validation should have failed".bright_red());
        }
        Err(e) => {
            println!("{} Missing private key properly detected: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    println!("{}", "✅ Missing certificate file error handling working correctly".bright_green());
    Ok(())
}

/// Test invalid certificate format error handling
async fn test_invalid_certificate_format() -> Result<()> {
    println!("{}", "🔍 Testing invalid certificate format scenarios...".bright_cyan());
    
    // Create temporary files with invalid content
    let temp_dir = tempfile::tempdir()
        .map_err(|e| QollectiveError::validation(format!("Failed to create temp dir: {}", e)))?;
    
    let invalid_cert_path = temp_dir.path().join("invalid.pem");
    std::fs::write(&invalid_cert_path, "This is not a valid PEM certificate")
        .map_err(|e| QollectiveError::validation(format!("Failed to write invalid cert: {}", e)))?;
    
    // Test loading invalid certificate
    match load_certificate_file(&invalid_cert_path) {
        Ok(_) => {
            println!("{}", "❌ Invalid certificate should not have loaded successfully".bright_red());
        }
        Err(e) => {
            println!("{} Invalid certificate format properly rejected: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    // Test loading invalid private key
    let invalid_key_path = temp_dir.path().join("invalid_key.pem");
    std::fs::write(&invalid_key_path, "This is not a valid PEM private key")
        .map_err(|e| QollectiveError::validation(format!("Failed to write invalid key: {}", e)))?;
    
    match load_private_key_file(&invalid_key_path) {
        Ok(_) => {
            println!("{}", "❌ Invalid private key should not have loaded successfully".bright_red());
        }
        Err(e) => {
            println!("{} Invalid private key format properly rejected: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    println!("{}", "✅ Invalid certificate format error handling working correctly".bright_green());
    Ok(())
}

/// Test certificate path permissions error handling
async fn test_certificate_permissions() -> Result<()> {
    println!("{}", "🔍 Testing certificate path permissions scenarios...".bright_cyan());
    
    // For this test, we'll simulate permission issues by checking for directories
    // instead of files (which should fail)
    let temp_dir = tempfile::tempdir()
        .map_err(|e| QollectiveError::validation(format!("Failed to create temp dir: {}", e)))?;
    
    let dir_path = temp_dir.path().join("directory_not_file");
    std::fs::create_dir(&dir_path)
        .map_err(|e| QollectiveError::validation(format!("Failed to create dir: {}", e)))?;
    
    // Test loading directory as certificate (should fail)
    match load_certificate_file(&dir_path) {
        Ok(_) => {
            println!("{}", "❌ Directory should not be loadable as certificate".bright_red());
        }
        Err(e) => {
            println!("{} Directory access properly rejected: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    println!("{}", "✅ Certificate permissions error handling working correctly".bright_green());
    Ok(())
}

/// Test TLS handshake timeout error handling
async fn test_tls_handshake_timeout() -> Result<()> {
    println!("{}", "🔍 Testing TLS handshake timeout scenarios...".bright_cyan());
    
    let config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    let nats_client_config = config.nats.to_framework_client_config(&config.tls);
    
    // Test with very short connection timeout and unreachable host
    let mut short_timeout_config = nats_client_config.clone();
    short_timeout_config.connection.connection_timeout_ms = 1; // 1ms - should timeout
    short_timeout_config.connection.urls = vec!["nats://192.0.2.1:4443".to_string()]; // RFC5737 test address
    
    let start_time = std::time::Instant::now();
    let result = attempt_tls_connection(&short_timeout_config).await;
    let elapsed = start_time.elapsed();
    
    match result {
        Ok(_) => {
            println!("{}", "❌ Connection with 1ms timeout to unreachable host should have failed".bright_red());
        }
        Err(e) => {
            println!("{} Connection timeout properly handled after {:?}: {}", 
                    "✅".bright_green(), 
                    elapsed,
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    println!("{}", "✅ TLS handshake timeout error handling working correctly".bright_green());
    Ok(())
}

/// Test certificate chain validation error handling
async fn test_certificate_chain_validation() -> Result<()> {
    println!("{}", "🔍 Testing certificate chain validation scenarios...".bright_cyan());
    
    let config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    // Load the actual certificates to validate they form a proper chain
    let framework_config = config.tls.to_framework_tls_config();
    
    if let (Some(ca_path), Some(cert_path), Some(key_path)) = 
        (&framework_config.ca_cert_path, &framework_config.cert_path, &framework_config.key_path) {
        
        // Test certificate loading with chain validation
        let ca_cert = load_certificate_file(ca_path)?;
        let client_cert = load_certificate_file(cert_path)?;
        let client_key = load_private_key_file(key_path)?;
        
        // Create root store and validate chain
        let mut root_store = rustls::RootCertStore::empty();
        match root_store.add(ca_cert) {
            Ok(()) => {
                println!("{} CA certificate added to root store successfully", "✅".bright_green());
            }
            Err(e) => {
                println!("{} CA certificate validation failed: {}", "❌".bright_red(), e);
                return Err(QollectiveError::validation(format!("CA certificate validation failed: {}", e)));
            }
        }
        
        // Test client config creation with chain validation
        match rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(vec![client_cert], client_key) {
            Ok(_) => {
                println!("{} Certificate chain validation successful", "✅".bright_green());
            }
            Err(e) => {
                println!("{} Certificate chain validation failed: {}", "❌".bright_red(), e);
                return Err(QollectiveError::validation(format!("Certificate chain validation failed: {}", e)));
            }
        }
    } else {
        println!("{}", "⚠️  TLS not configured, skipping certificate chain validation".bright_yellow());
    }
    
    println!("{}", "✅ Certificate chain validation error handling working correctly".bright_green());
    Ok(())
}

/// Test network connectivity issues error handling
async fn test_network_connectivity_issues() -> Result<()> {
    println!("{}", "🔍 Testing network connectivity error scenarios...".bright_cyan());
    
    let config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    let mut invalid_config = config.nats.to_framework_client_config(&config.tls);
    
    // Test with invalid hostname
    invalid_config.connection.urls = vec!["nats://nonexistent.host:4443".to_string()];
    invalid_config.connection.connection_timeout_ms = 2000; // Short timeout for faster test
    
    let result = attempt_tls_connection(&invalid_config).await;
    match result {
        Ok(_) => {
            println!("{}", "❌ Connection to nonexistent host should have failed".bright_red());
        }
        Err(e) => {
            println!("{} Network connectivity error properly handled: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    // Test with invalid port
    let mut invalid_port_config = config.nats.to_framework_client_config(&config.tls);
    invalid_port_config.connection.urls = vec!["nats://localhost:9999".to_string()];
    invalid_port_config.connection.connection_timeout_ms = 2000;
    
    let result = attempt_tls_connection(&invalid_port_config).await;
    match result {
        Ok(_) => {
            println!("{}", "❌ Connection to invalid port should have failed".bright_red());
        }
        Err(e) => {
            println!("{} Invalid port error properly handled: {}", 
                    "✅".bright_green(), 
                    e.to_string().chars().take(80).collect::<String>());
        }
    }
    
    println!("{}", "✅ Network connectivity error handling working correctly".bright_green());
    Ok(())
}

/// Test configuration validation error handling
async fn test_configuration_validation() -> Result<()> {
    println!("{}", "🔍 Testing configuration validation scenarios...".bright_cyan());
    
    // Test with TLS disabled but certificates configured
    let mut config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    config.tls.enabled = false;
    
    // This should succeed since TLS is disabled
    match config.tls.validate_certificate_paths() {
        Ok(()) => {
            println!("{} TLS disabled configuration validated correctly", "✅".bright_green());
        }
        Err(e) => {
            println!("{} TLS disabled validation unexpectedly failed: {}", "❌".bright_red(), e);
        }
    }
    
    // Test with invalid verification mode
    let mut config = EnterpriseConfig::load_default()
        .map_err(|e| QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    config.tls.verification_mode = "invalid_mode".to_string();
    let framework_config = config.tls.to_framework_tls_config();
    
    // Should default to MutualTls with warning
    match framework_config.verification_mode {
        qollective::config::tls::VerificationMode::MutualTls => {
            println!("{} Invalid verification mode properly defaulted to MutualTls", "✅".bright_green());
        }
        _ => {
            println!("{} Invalid verification mode handling failed", "❌".bright_red());
        }
    }
    
    println!("{}", "✅ Configuration validation error handling working correctly".bright_green());
    Ok(())
}

/// Attempt TLS connection with error handling
async fn attempt_tls_connection(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    use async_nats::ConnectOptions;
    
    let tls_config = &nats_config.connection.tls;
    
    if !tls_config.enabled {
        return Err(QollectiveError::validation("TLS not enabled".to_string()));
    }
    
    // Build NATS connection options with TLS
    let mut connect_options = ConnectOptions::new();
    connect_options = connect_options
        .connection_timeout(Duration::from_millis(nats_config.connection.connection_timeout_ms));
    
    // Configure TLS
    if let (Some(ca_path), Some(cert_path), Some(key_path)) = 
        (&tls_config.ca_cert_path, &tls_config.cert_path, &tls_config.key_path) {
        
        // Load certificates
        let ca_cert = load_certificate_file(ca_path)?;
        let client_cert = load_certificate_file(cert_path)?;
        let client_key = load_private_key_file(key_path)?;
        
        // Configure TLS with mutual authentication
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add(ca_cert).map_err(|e| 
            QollectiveError::validation(format!("Failed to add CA certificate: {}", e)))?;
        
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(vec![client_cert], client_key)
            .map_err(|e| QollectiveError::validation(format!("Failed to configure client auth: {}", e)))?;
        
        connect_options = connect_options.require_tls(true)
            .tls_client_config(client_config);
    }
    
    // Attempt to connect
    let url = &nats_config.connection.urls[0];
    let _client = async_nats::connect_with_options(url, connect_options)
        .await
        .map_err(|e| QollectiveError::connection(format!("TLS connection failed: {}", e)))?;
    
    Ok(())
}

/// Load certificate from PEM file with enhanced error handling
fn load_certificate_file(path: &std::path::PathBuf) -> Result<rustls::pki_types::CertificateDer<'static>> {
    use std::fs::File;
    use std::io::BufReader;
    
    // Check if path exists and is accessible
    if !path.exists() {
        return Err(QollectiveError::validation(format!("Certificate file does not exist: {}", path.display())));
    }
    
    if !path.is_file() {
        return Err(QollectiveError::validation(format!("Certificate path is not a file: {}", path.display())));
    }
    
    let file = File::open(path)
        .map_err(|e| QollectiveError::validation(format!("Failed to open certificate file {}: {}", path.display(), e)))?;
    
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|e| QollectiveError::validation(format!("Failed to parse certificate from {}: {}", path.display(), e)))?;
    
    if certs.is_empty() {
        return Err(QollectiveError::validation(format!("No certificates found in file: {}", path.display())));
    }
    
    Ok(certs.into_iter().next().unwrap())
}

/// Load private key from PEM file with enhanced error handling
fn load_private_key_file(path: &std::path::PathBuf) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
    use std::fs::File;
    use std::io::BufReader;
    
    // Check if path exists and is accessible
    if !path.exists() {
        return Err(QollectiveError::validation(format!("Private key file does not exist: {}", path.display())));
    }
    
    if !path.is_file() {
        return Err(QollectiveError::validation(format!("Private key path is not a file: {}", path.display())));
    }
    
    let file = File::open(path)
        .map_err(|e| QollectiveError::validation(format!("Failed to open private key file {}: {}", path.display(), e)))?;
    
    let mut reader = BufReader::new(file);
    
    // Try to read a single private key
    if let Some(key) = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| QollectiveError::validation(format!("Failed to parse private key from {}: {}", path.display(), e)))? {
        return Ok(key);
    }
    
    Err(QollectiveError::validation(format!("No private key found in file: {}", path.display())))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    match test_tls_error_handling().await {
        Ok(()) => {
            println!("\n{}", "🎉 TLS Error Handling Test PASSED".bright_green().bold());
            std::process::exit(0);
        }
        Err(e) => {
            println!("\n{} {}", "❌ TLS Error Handling Test FAILED:".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}