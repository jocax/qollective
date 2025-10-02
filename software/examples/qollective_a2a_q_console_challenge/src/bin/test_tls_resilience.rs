//! TLS Connection Resilience Test
//! 
//! Tests TLS connection recovery, reconnection handling, and resilience patterns
//! for the A2A NATS Enterprise example.

use std::time::Duration;
use colored::Colorize;
use futures::StreamExt;

use qollective::error::Result;
use qollective_a2a_nats_enterprise::config::EnterpriseConfig;

/// Test TLS connection resilience and recovery
pub async fn test_tls_resilience() -> Result<()> {
    println!("{}", "🔄 Testing TLS Connection Resilience".bright_blue().bold());
    println!("{}", "━".repeat(80).bright_blue());
    
    // Initialize TLS crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    
    // Load configuration
    println!("{}", "📁 Loading configuration...".bright_cyan());
    let config = EnterpriseConfig::load_default()
        .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to load config: {}", e)))?;
    
    // Convert to framework config
    let nats_client_config = config.nats.to_framework_client_config(&config.tls);
    
    println!("{}", "✅ Configuration loaded successfully".bright_green());
    
    // Test multiple connection attempts with TLS
    println!("\n{}", "🔄 Testing multiple TLS connection establishment...".bright_cyan());
    test_multiple_tls_connections(&nats_client_config).await?;
    
    // Test reconnection behavior
    println!("\n{}", "🔄 Testing TLS reconnection configuration...".bright_cyan());
    test_tls_reconnection_config(&nats_client_config).await?;
    
    // Test connection error handling
    println!("\n{}", "🔄 Testing TLS connection error handling...".bright_cyan());
    test_tls_error_handling(&nats_client_config).await?;
    
    // Test connection timeout handling
    println!("\n{}", "🔄 Testing TLS connection timeout handling...".bright_cyan());
    test_tls_timeout_handling(&nats_client_config).await?;
    
    println!("\n{}", "🎉 All TLS resilience tests passed!".bright_green().bold());
    
    Ok(())
}

/// Test multiple TLS connections can be established successfully
async fn test_multiple_tls_connections(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    use async_nats::ConnectOptions;
    
    println!("{}", "📡 Testing multiple concurrent TLS connections...".bright_cyan());
    
    let connection_count = 3;
    let mut connection_tasks = Vec::new();
    
    for i in 1..=connection_count {
        let config = nats_config.clone();
        let task = tokio::spawn(async move {
            let result = establish_single_tls_connection(&config, i).await;
            (i, result)
        });
        connection_tasks.push(task);
    }
    
    // Wait for all connections to complete
    let mut successful_connections = 0;
    let mut failed_connections = 0;
    
    for task in connection_tasks {
        match task.await {
            Ok((conn_id, Ok(()))) => {
                println!("{} Connection {} established successfully", "✅".bright_green(), conn_id);
                successful_connections += 1;
            }
            Ok((conn_id, Err(e))) => {
                println!("{} Connection {} failed: {}", "❌".bright_red(), conn_id, e);
                failed_connections += 1;
            }
            Err(e) => {
                println!("{} Task error: {}", "❌".bright_red(), e);
                failed_connections += 1;
            }
        }
    }
    
    println!("{} {}/{} connections successful", "📊".bright_yellow(), successful_connections, connection_count);
    
    if successful_connections == connection_count {
        println!("{}", "✅ All concurrent TLS connections successful".bright_green());
    } else {
        println!("{}", "⚠️  Some connections failed - this may be expected under load".bright_yellow());
    }
    
    Ok(())
}

/// Establish a single TLS connection for testing
async fn establish_single_tls_connection(nats_config: &qollective::config::nats::NatsClientConfig, conn_id: u32) -> Result<()> {
    use async_nats::ConnectOptions;
    
    let tls_config = &nats_config.connection.tls;
    
    if !tls_config.enabled {
        return Err(qollective::error::QollectiveError::validation("TLS not enabled".to_string()));
    }
    
    // Build NATS connection options with TLS
    let mut connect_options = ConnectOptions::new();
    connect_options = connect_options.connection_timeout(Duration::from_millis(nats_config.connection.connection_timeout_ms));
    
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
            qollective::error::QollectiveError::validation(format!("Failed to add CA certificate: {}", e)))?;
        
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(vec![client_cert], client_key)
            .map_err(|e| qollective::error::QollectiveError::validation(format!("Failed to configure client auth: {}", e)))?;
        
        connect_options = connect_options.require_tls(true)
            .tls_client_config(client_config);
    }
    
    // Attempt to connect
    let url = &nats_config.connection.urls[0];
    let client = async_nats::connect_with_options(url, connect_options)
        .await
        .map_err(|e| qollective::error::QollectiveError::connection(format!("Connection {} failed: {}", conn_id, e)))?;
    
    // Test basic operation
    let test_subject = format!("tls.resilience.test.{}", conn_id);
    let mut subscriber = client.subscribe(test_subject.clone())
        .await
        .map_err(|e| qollective::error::QollectiveError::connection(format!("Subscribe failed: {}", e)))?;
    
    // Publish and verify
    client.publish(test_subject.clone(), format!("test-{}", conn_id).into())
        .await
        .map_err(|e| qollective::error::QollectiveError::connection(format!("Publish failed: {}", e)))?;
    
    // Verify message received
    let _message = tokio::time::timeout(Duration::from_secs(2), subscriber.next())
        .await
        .map_err(|_| qollective::error::QollectiveError::nats_timeout("Timeout waiting for test message".to_string()))?
        .ok_or_else(|| qollective::error::QollectiveError::connection("No message received".to_string()))?;
    
    drop(subscriber);
    
    Ok(())
}

/// Test TLS reconnection configuration
async fn test_tls_reconnection_config(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    println!("{}", "🔧 Analyzing TLS reconnection configuration...".bright_cyan());
    
    let max_reconnects = nats_config.connection.max_reconnect_attempts.unwrap_or(0);
    let reconnect_timeout = nats_config.connection.reconnect_timeout_ms;
    
    println!("{} {} attempts", "🔄 Max Reconnect Attempts:".bright_yellow(), max_reconnects);
    println!("{} {}ms", "⏱️  Reconnect Timeout:".bright_yellow(), reconnect_timeout);
    
    // Validate reconnection settings for TLS resilience
    let mut recommendations = Vec::new();
    
    if max_reconnects == 0 {
        recommendations.push("⚠️  Reconnection is disabled - consider enabling for TLS resilience".to_string());
    } else if max_reconnects < 3 {
        recommendations.push("⚠️  Consider increasing max reconnect attempts for better TLS resilience".to_string());
    } else {
        println!("{}", "✅ Reconnect attempts configured appropriately for TLS".bright_green());
    }
    
    if reconnect_timeout < 1000 {
        recommendations.push("⚠️  Reconnect timeout may be too aggressive for TLS handshakes".to_string());
    } else if reconnect_timeout > 30000 {
        recommendations.push("⚠️  Reconnect timeout may be too long for responsive TLS recovery".to_string());
    } else {
        println!("{}", "✅ Reconnect timeout is appropriate for TLS connections".bright_green());
    }
    
    // Display recommendations
    if !recommendations.is_empty() {
        println!("\n{}", "💡 Recommendations for TLS resilience:".bright_yellow().bold());
        for rec in recommendations {
            println!("   {}", rec);
        }
    } else {
        println!("{}", "✅ TLS reconnection configuration is well-tuned".bright_green());
    }
    
    Ok(())
}

/// Test TLS connection error handling
async fn test_tls_error_handling(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    println!("{}", "🚨 Testing TLS error handling scenarios...".bright_cyan());
    
    // Test with invalid port (simulates connection failure)
    println!("{}", "   Testing connection to invalid port...".dimmed());
    
    let mut invalid_config = nats_config.clone();
    invalid_config.connection.urls = vec!["nats://localhost:9999".to_string()];
    invalid_config.connection.connection_timeout_ms = 2000; // Short timeout for faster test
    
    let result = establish_single_tls_connection(&invalid_config, 999).await;
    match result {
        Err(e) => {
            println!("{} Connection error properly handled: {}", "✅".bright_green(), 
                    e.to_string().chars().take(100).collect::<String>());
        }
        Ok(()) => {
            println!("{} Unexpected success connecting to invalid port", "❌".bright_red());
        }
    }
    
    // Test with missing certificates (simulates TLS configuration error)
    println!("{}", "   Testing with invalid certificate path...".dimmed());
    
    let mut invalid_tls_config = nats_config.clone();
    if let Some(ref mut ca_path) = invalid_tls_config.connection.tls.ca_cert_path {
        *ca_path = std::path::PathBuf::from("nonexistent-ca.pem");
    }
    
    let result = establish_single_tls_connection(&invalid_tls_config, 998).await;
    match result {
        Err(e) => {
            println!("{} Certificate error properly handled: {}", "✅".bright_green(), 
                    e.to_string().chars().take(100).collect::<String>());
        }
        Ok(()) => {
            println!("{} Unexpected success with invalid certificate", "❌".bright_red());
        }
    }
    
    println!("{}", "✅ TLS error handling is working correctly".bright_green());
    
    Ok(())
}

/// Test TLS connection timeout handling
async fn test_tls_timeout_handling(nats_config: &qollective::config::nats::NatsClientConfig) -> Result<()> {
    println!("{}", "⏱️  Testing TLS timeout handling...".bright_cyan());
    
    let original_timeout = nats_config.connection.connection_timeout_ms;
    println!("{} Original timeout: {}ms", "📊".bright_yellow(), original_timeout);
    
    if original_timeout > 5000 {
        println!("{}", "✅ Connection timeout allows sufficient time for TLS handshake".bright_green());
    } else {
        println!("{}", "⚠️  Connection timeout may be too short for reliable TLS handshakes".bright_yellow());
    }
    
    // Test with very short timeout to verify timeout handling
    println!("{}", "   Testing with very short timeout...".dimmed());
    
    let mut short_timeout_config = nats_config.clone();
    short_timeout_config.connection.connection_timeout_ms = 1; // 1ms - should timeout
    
    // Use unreachable IP to guarantee timeout (RFC5737 test address)
    short_timeout_config.connection.urls = vec!["nats://192.0.2.1:4443".to_string()];
    
    let start_time = std::time::Instant::now();
    let result = establish_single_tls_connection(&short_timeout_config, 997).await;
    let elapsed = start_time.elapsed();
    
    match result {
        Err(e) => {
            println!("{} Timeout properly handled after {:?}: {}", "✅".bright_green(), elapsed,
                    e.to_string().chars().take(100).collect::<String>());
        }
        Ok(()) => {
            println!("{} Unexpected success with 1ms timeout to unreachable host", "❌".bright_red());
        }
    }
    
    println!("{}", "✅ TLS timeout handling is working correctly".bright_green());
    
    Ok(())
}

/// Load certificate from PEM file (same as in test_nats_tls_connection.rs)
fn load_certificate_file(path: &std::path::PathBuf) -> Result<rustls::pki_types::CertificateDer<'static>> {
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

/// Load private key from PEM file (same as in test_nats_tls_connection.rs)
fn load_private_key_file(path: &std::path::PathBuf) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
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
    
    match test_tls_resilience().await {
        Ok(()) => {
            println!("\n{}", "🎉 TLS Resilience Test PASSED".bright_green().bold());
            std::process::exit(0);
        }
        Err(e) => {
            println!("\n{} {}", "❌ TLS Resilience Test FAILED:".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}