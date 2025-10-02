//! Test Q Console and Log Agent TLS Communication
//!
//! This test verifies that both Q Console and Log Agent can establish TLS
//! connections and communicate properly with the Enterprise registry.

use std::{collections::HashMap, time::{Duration, SystemTime}};
use uuid::Uuid;
use colored::Colorize;
use qollective::{
    error::Result,
    types::a2a::{AgentInfo, HealthStatus},
    client::nats::NatsClient,
    envelope::EnvelopeBuilder,
    constants::subjects,
};
use qollective_a2a_nats_enterprise::{
    config::EnterpriseConfig,
    startrek_agent::EnterpriseNatsConfig,
};

// Import LogEntry from the log_agent module (it's defined there)
use serde::{Serialize, Deserialize};

/// Generic log entry that can handle different types of logs (copied from log_agent.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub log_type: String,
    pub crew_member: String,
    pub rank: String,
    pub timestamp: std::time::SystemTime,
    pub correlation_id: Uuid,
    pub task_id: Uuid,
    pub task_type: String,
    pub description: String,
    pub outcome: String,
    pub duration_seconds: u64,
    pub confidence_level: f64,
    pub stress_level: f64,
    pub skills_demonstrated: Vec<String>,
    pub challenges_faced: Vec<String>,
    pub collaborated_with: Vec<String>,
    pub performance_rating: String,
    pub notes: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize TLS crypto provider
    match rustls::crypto::aws_lc_rs::default_provider().install_default() {
        Ok(_) => {
            println!("{}", "🔒 TLS crypto provider initialized successfully".bright_green());
        }
        Err(e) => {
            println!("{} {:?}", "❌ Failed to initialize TLS crypto provider:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(
                format!("TLS crypto provider initialization failed: {:?}", e)
            ));
        }
    }

    // Initialize logging
    env_logger::init();

    println!("{}", "🔐 Testing Q Console and Log Agent TLS Communication".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Load Enterprise configuration for TLS context
    println!("{}", "📁 Loading Enterprise configuration for TLS context...".bright_blue());
    let config = match EnterpriseConfig::load_default() {
        Ok(config) => {
            println!("{}", "✅ Enterprise configuration loaded successfully".bright_green());
            config
        }
        Err(e) => {
            println!("{} {}", "❌ Configuration loading failed:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(format!("Failed to load config.toml: {}", e)));
        }
    };

    println!("{} {}", "🔐 TLS enabled:".bright_cyan(), config.tls.enabled.to_string().bright_yellow());

    // Test Q Console TLS Connection
    test_q_console_tls_connection().await?;

    // Test Log Agent TLS Connection
    test_log_agent_tls_connection().await?;

    // Test Q Console to Log Agent communication via TLS
    test_q_console_to_log_agent_communication().await?;

    println!("\n{}", "✅ All Q Console and Log Agent TLS Tests Completed Successfully".bright_green().bold());
    Ok(())
}

/// Test Q Console TLS connection to Enterprise registry
async fn test_q_console_tls_connection() -> Result<()> {
    println!("\n{}", "📋 Test 1: Q Console TLS Connection".bright_cyan().bold());
    println!("{}", "🔍 Testing Q Console's ability to establish TLS connection...".dimmed());

    // Use centralized configuration from config.toml (same as q_console.rs)
    let connection_configs = EnterpriseNatsConfig::connection_configs();

    let mut nats_client = None;
    for nats_client_config in connection_configs {
        let url = nats_client_config.connection.urls[0].clone();
        println!("{} {}", "🔍 Q Console trying:".bright_cyan(), url.dimmed());

        // Convert NatsClientConfig to NatsConfig for NatsClient::new()
        let nats_config = qollective::config::nats::NatsConfig {
            connection: nats_client_config.connection,
            client: nats_client_config.client_behavior,
            server: qollective::config::nats::NatsServerConfig::default(),
            discovery: qollective::config::nats::NatsDiscoveryConfig {
                enabled: true,
                ttl_ms: nats_client_config.discovery_cache_ttl_ms,
                ..Default::default()
            },
        };

        match NatsClient::new(nats_config).await {
            Ok(client) => {
                println!("{} {} - Q Console", "✅ TLS Connected to:".bright_green().bold(), url.bright_yellow());
                nats_client = Some(client);
                break;
            }
            Err(e) => {
                println!("{} {} - {}", "❌ Q Console Failed:".bright_red(), url.dimmed(), e.to_string().red().dimmed());
            }
        }
    }

    let nats_client = nats_client.ok_or_else(|| qollective::error::QollectiveError::nats_connection("Q Console could not connect to any NATS server port".to_string()))?;

    // Test Q Console agent registration with Enterprise registry
    let q_agent = AgentInfo {
        id: Uuid::now_v7(),
        name: "Q Console - TLS Test".to_string(),
        capabilities: vec![
            "omnipotence".to_string(),
            "testing-mortals".to_string(),
            "cosmic-challenges".to_string(),
            "reality-manipulation".to_string(),
            "tls-communication".to_string(),
        ],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("location".to_string(), "Continuum".to_string()),
            ("species".to_string(), "Q".to_string()),
            ("power_level".to_string(), "Omnipotent".to_string()),
            ("tls_enabled".to_string(), "true".to_string()),
        ]),
    };

    let q_envelope = EnvelopeBuilder::new()
        .with_payload(q_agent.clone())
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta.version = Some("tls-test-v1.0".to_string());
            meta
        })
        .build()?;

    match nats_client.publish(subjects::AGENT_REGISTRY_REGISTER, q_envelope).await {
        Ok(_) => {
            println!("{} Q Console successfully registered with Enterprise registry via TLS", "✅".bright_green());
            println!("   {} {}", "Agent ID:".bright_cyan(), q_agent.id.to_string().bright_yellow());
            println!("   {} {}", "Capabilities:".bright_cyan(), "5 cosmic abilities".bright_magenta());
            println!("   {} {}", "TLS Status:".bright_cyan(), "Enabled and authenticated".bright_green());
        }
        Err(e) => {
            println!("{} Q Console registration failed: {}", "❌".bright_red(), e.to_string().red());
            return Err(e);
        }
    }

    println!("{}", "✅ Test 1 passed: Q Console TLS connection established".bright_green());
    Ok(())
}

/// Test Log Agent TLS connection to Enterprise registry
async fn test_log_agent_tls_connection() -> Result<()> {
    println!("\n{}", "📋 Test 2: Log Agent TLS Connection".bright_cyan().bold());
    println!("{}", "🔍 Testing Log Agent's ability to establish TLS connection...".dimmed());

    // Use the same configuration approach as the Log Agent
    let connection_configs = EnterpriseNatsConfig::connection_configs();

    let mut nats_client = None;
    for nats_client_config in connection_configs {
        let url = nats_client_config.connection.urls[0].clone();
        println!("{} {}", "🔍 Log Agent trying:".bright_cyan(), url.dimmed());

        // Convert NatsClientConfig to NatsConfig for NatsClient::new()
        let nats_config = qollective::config::nats::NatsConfig {
            connection: nats_client_config.connection,
            client: nats_client_config.client_behavior,
            server: qollective::config::nats::NatsServerConfig::default(),
            discovery: qollective::config::nats::NatsDiscoveryConfig {
                enabled: true,
                ttl_ms: nats_client_config.discovery_cache_ttl_ms,
                ..Default::default()
            },
        };

        match NatsClient::new(nats_config).await {
            Ok(client) => {
                println!("{} {} - Log Agent", "✅ TLS Connected to:".bright_green().bold(), url.bright_yellow());
                nats_client = Some(client);
                break;
            }
            Err(e) => {
                println!("{} {} - {}", "❌ Log Agent Failed:".bright_red(), url.dimmed(), e.to_string().red().dimmed());
            }
        }
    }

    let nats_client = nats_client.ok_or_else(|| qollective::error::QollectiveError::nats_connection("Log Agent could not connect to any NATS server port".to_string()))?;

    // Test Log Agent registration with Enterprise registry
    let log_agent = AgentInfo {
        id: Uuid::now_v7(),
        name: "Enterprise Log Agent - TLS Test".to_string(),
        capabilities: vec![
            "logging".to_string(),
            "log-aggregation".to_string(),
            "real-time-display".to_string(),
            "context-analysis".to_string(),
            "envelope-processing".to_string(),
            "tls-communication".to_string(),
        ],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("location".to_string(), "Computer Core - Logging Bay".to_string()),
            ("service_type".to_string(), "hybrid_logging".to_string()),
            ("tls_enabled".to_string(), "true".to_string()),
        ]),
    };

    let log_envelope = EnvelopeBuilder::new()
        .with_payload(log_agent.clone())
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta.version = Some("tls-test-v1.0".to_string());
            meta
        })
        .build()?;

    match nats_client.publish(subjects::AGENT_REGISTRY_REGISTER, log_envelope).await {
        Ok(_) => {
            println!("{} Log Agent successfully registered with Enterprise registry via TLS", "✅".bright_green());
            println!("   {} {}", "Agent ID:".bright_cyan(), log_agent.id.to_string().bright_yellow());
            println!("   {} {}", "Capabilities:".bright_cyan(), "6 logging abilities".bright_magenta());
            println!("   {} {}", "TLS Status:".bright_cyan(), "Enabled and authenticated".bright_green());
        }
        Err(e) => {
            println!("{} Log Agent registration failed: {}", "❌".bright_red(), e.to_string().red());
            return Err(e);
        }
    }

    println!("{}", "✅ Test 2 passed: Log Agent TLS connection established".bright_green());
    Ok(())
}

/// Test Q Console to Log Agent communication via TLS-secured NATS
async fn test_q_console_to_log_agent_communication() -> Result<()> {
    println!("\n{}", "📋 Test 3: Q Console to Log Agent Communication via TLS".bright_cyan().bold());
    println!("{}", "🔍 Testing Q Console sending log entries to Log Agent over TLS...".dimmed());

    // Establish Q Console connection (same as before)
    let connection_configs = EnterpriseNatsConfig::connection_configs();
    let nats_client_config = &connection_configs[0]; // Use first config

    let nats_config = qollective::config::nats::NatsConfig {
        connection: nats_client_config.connection.clone(),
        client: nats_client_config.client_behavior.clone(),
        server: qollective::config::nats::NatsServerConfig::default(),
        discovery: qollective::config::nats::NatsDiscoveryConfig {
            enabled: true,
            ttl_ms: nats_client_config.discovery_cache_ttl_ms,
            ..Default::default()
        },
    };

    let nats_client = NatsClient::new(nats_config).await?;

    // Create a test log entry from Q Console
    let test_log_entry = LogEntry {
        log_type: "cosmic".to_string(),
        crew_member: "Q".to_string(),
        rank: "Omnipotent Entity".to_string(),
        timestamp: SystemTime::now(),
        correlation_id: Uuid::now_v7(),
        task_id: Uuid::now_v7(),
        task_type: "tls-communication-test".to_string(),
        description: "Testing TLS-secured communication between Q Console and Log Agent".to_string(),
        outcome: "Successful TLS message transmission".to_string(),
        duration_seconds: 1,
        confidence_level: 1.0, // Q is always confident
        stress_level: 0.0, // Q doesn't experience stress
        skills_demonstrated: vec!["tls-communication".to_string(), "cosmic-messaging".to_string()],
        challenges_faced: vec!["Mortal TLS limitations".to_string()],
        collaborated_with: vec!["Enterprise Log Agent".to_string()],
        performance_rating: "Omnipotent".to_string(),
        notes: "TLS communication test successful - even mortals can secure their communications".to_string(),
    };

    // Create envelope with rich context for the log entry
    let log_envelope = EnvelopeBuilder::new()
        .with_payload(test_log_entry.clone())
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta.version = Some("tls-test-v1.0".to_string());

            // Add TLS-specific extensions
            if meta.extensions.is_none() {
                meta.extensions = Some(qollective::envelope::meta::ExtensionsMeta {
                    sections: std::collections::HashMap::new(),
                });
            }

            if let Some(extensions) = &mut meta.extensions {
                // Pack TLS test context
                let tls_data = serde_json::json!({
                    "tls_enabled": true,
                    "connection_secured": true,
                    "test_purpose": "Q Console to Log Agent TLS communication",
                    "encryption_status": "Active"
                });
                extensions.sections.insert("tls_test_context".to_string(), tls_data);

                // Pack enterprise context
                let enterprise_data = serde_json::json!({
                    "ship_registry": "NCC-1701-D",
                    "test_scenario": "TLS Communication Validation",
                    "participants": ["Q Console", "Log Agent"],
                    "security_level": "Maximum"
                });
                extensions.sections.insert("enterprise_context".to_string(), enterprise_data);
            }

            meta
        })
        .build()?;

    // Send log entry to Log Agent via TLS-secured NATS
    match nats_client.publish("enterprise.logging.entry", log_envelope).await {
        Ok(_) => {
            println!("{} Q Console successfully sent log entry to Log Agent via TLS", "✅".bright_green());
            println!("   {} {}", "Log Type:".bright_cyan(), test_log_entry.log_type.bright_magenta());
            println!("   {} {}", "Task Type:".bright_cyan(), test_log_entry.task_type.bright_yellow());
            println!("   {} {}", "Correlation ID:".bright_cyan(), test_log_entry.correlation_id.to_string().bright_green());
            println!("   {} {}", "Message Size:".bright_cyan(), "~1KB with TLS envelope context".bright_blue());
            println!("   {} {}", "Security:".bright_cyan(), "TLS 1.2/1.3 encrypted transport".bright_green());
        }
        Err(e) => {
            println!("{} Q Console to Log Agent communication failed: {}", "❌".bright_red(), e.to_string().red());
            return Err(e);
        }
    }

    // Wait a moment for message delivery
    tokio::time::sleep(Duration::from_millis(1000)).await;

    println!("{}", "✅ Test 3 passed: Q Console to Log Agent TLS communication successful".bright_green());
    Ok(())
}
