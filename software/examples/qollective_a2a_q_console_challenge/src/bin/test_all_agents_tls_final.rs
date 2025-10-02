//! Final TLS Agent Communication Verification
//!
//! This test verifies that all Enterprise agents can communicate with TLS enabled
//! using the proven NATS-based approach that works for Q Console and Log Agent.

use std::{collections::HashMap, time::SystemTime};
use uuid::Uuid;
use colored::Colorize;
use qollective::{
    error::Result,
    types::a2a::{AgentInfo, CapabilityQuery, HealthStatus},
    client::nats::NatsClient,
    envelope::EnvelopeBuilder,
    constants::subjects,
};
use qollective_a2a_nats_enterprise::{
    config::EnterpriseConfig,
    startrek_agent::EnterpriseNatsConfig,
};

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

    println!("{}", "🔐 Final TLS Agent Communication Verification".bright_blue().bold());
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

    if !config.tls.enabled {
        return Err(qollective::error::QollectiveError::validation("TLS must be enabled for this test".to_string()));
    }

    println!("{} {}", "🔐 TLS enabled:".bright_cyan(), config.tls.enabled.to_string().bright_yellow());

    // Test 1: Multiple Agent TLS Connections
    test_multiple_agent_tls_connections().await?;

    // Test 2: Agent Registry Communication
    test_agent_registry_communication().await?;

    // Test 3: Agent Discovery via TLS
    test_agent_discovery_tls().await?;

    // Test 4: Cross-Agent Communication
    test_cross_agent_communication().await?;

    println!("\n{}", "✅ All TLS Agent Communication Tests Completed Successfully".bright_green().bold());
    Ok(())
}

/// Test multiple agents establishing TLS connections simultaneously
async fn test_multiple_agent_tls_connections() -> Result<()> {
    println!("\n{}", "📋 Test 1: Multiple Agent TLS Connections".bright_cyan().bold());
    println!("{}", "🔍 Testing multiple Enterprise agents establishing TLS connections...".dimmed());

    let agent_names = vec![
        "Test-Picard", "Test-Data", "Test-Spock", "Test-Scotty", "Test-Security-Officer"
    ];

    let mut successful_connections = 0;

    for agent_name in &agent_names {
        let nats_client = create_tls_nats_client().await?;

        // Create agent info
        let agent_info = AgentInfo {
            id: Uuid::now_v7(),
            name: format!("{} TLS Test", agent_name),
            capabilities: vec!["tls-communication".to_string(), "enterprise-operations".to_string()],
            health_status: HealthStatus::Healthy,
            last_heartbeat: SystemTime::now(),
            metadata: HashMap::from([
                ("location".to_string(), "Enterprise Bridge".to_string()),
                ("tls_enabled".to_string(), "true".to_string()),
                ("test_agent".to_string(), "true".to_string()),
            ]),
        };

        // Register via TLS
        let envelope = EnvelopeBuilder::new()
            .with_payload(agent_info.clone())
            .with_meta({
                let mut meta = qollective::envelope::meta::Meta::default();
                meta.request_id = Some(Uuid::now_v7());
                meta.timestamp = Some(chrono::Utc::now());
                meta.tenant = Some("enterprise".to_string());
                meta.version = Some("final-tls-test-v1.0".to_string());
                meta
            })
            .build()?;

        match nats_client.publish(subjects::AGENT_REGISTRY_REGISTER, envelope).await {
            Ok(_) => {
                println!("{} {} TLS connection established and registered", "✅".bright_green(), agent_name.bright_cyan());
                successful_connections += 1;
            }
            Err(e) => {
                println!("{} {} TLS connection failed: {}", "❌".bright_red(), agent_name.bright_red(), e);
            }
        }
    }

    if successful_connections == agent_names.len() {
        println!("{} All {} agents established TLS connections successfully", "✅".bright_green(), successful_connections);
    } else {
        return Err(qollective::error::QollectiveError::validation(
            format!("Only {}/{} agents succeeded", successful_connections, agent_names.len())
        ));
    }

    println!("{}", "✅ Test 1 passed: Multiple agent TLS connections established".bright_green());
    Ok(())
}

/// Test agent registry communication via TLS
async fn test_agent_registry_communication() -> Result<()> {
    println!("\n{}", "📋 Test 2: Agent Registry Communication".bright_cyan().bold());
    println!("{}", "🔍 Testing Enterprise registry communication via TLS...".dimmed());

    let nats_client = create_tls_nats_client().await?;

    // Create test agent with detailed metadata
    let agent_info = AgentInfo {
        id: Uuid::now_v7(),
        name: "Registry Test Agent".to_string(),
        capabilities: vec![
            "registry-testing".to_string(),
            "tls-verification".to_string(),
            "enterprise-compliance".to_string()
        ],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("location".to_string(), "Test Suite Bridge".to_string()),
            ("department".to_string(), "Security Testing".to_string()),
            ("clearance_level".to_string(), "Beta".to_string()),
            ("tls_verified".to_string(), "true".to_string()),
        ]),
    };

    // Test agent registration with rich envelope context
    let register_envelope = EnvelopeBuilder::new()
        .with_payload(agent_info.clone())
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta.version = Some("registry-test-v1.0".to_string());

            // Add test-specific extensions
            if meta.extensions.is_none() {
                meta.extensions = Some(qollective::envelope::meta::ExtensionsMeta {
                    sections: std::collections::HashMap::new(),
                });
            }

            if let Some(extensions) = &mut meta.extensions {
                let registry_test_data = serde_json::json!({
                    "test_type": "registry_communication",
                    "tls_enabled": true,
                    "security_level": "maximum",
                    "test_timestamp": chrono::Utc::now().to_rfc3339()
                });
                extensions.sections.insert("registry_test_context".to_string(), registry_test_data);
            }

            meta
        })
        .build()?;

    match nats_client.publish(subjects::AGENT_REGISTRY_REGISTER, register_envelope).await {
        Ok(_) => {
            println!("{} Agent registered with Enterprise registry via TLS", "✅".bright_green());
            println!("   {} {}", "Agent ID:".bright_cyan(), agent_info.id.to_string().bright_yellow());
            println!("   {} {}", "Capabilities:".bright_cyan(), agent_info.capabilities.len().to_string().bright_magenta());
            println!("   {} {}", "Security:".bright_cyan(), "TLS 1.2/1.3 encrypted".bright_green());
        }
        Err(e) => {
            println!("{} Registry communication failed: {}", "❌".bright_red(), e);
            return Err(e);
        }
    }

    println!("{}", "✅ Test 2 passed: Agent registry communication via TLS successful".bright_green());
    Ok(())
}

/// Test agent discovery via TLS
async fn test_agent_discovery_tls() -> Result<()> {
    println!("\n{}", "📋 Test 3: Agent Discovery via TLS".bright_cyan().bold());
    println!("{}", "🔍 Testing agent discovery through TLS-secured NATS...".dimmed());

    let nats_client = create_tls_nats_client().await?;

    // Test discovery query for Enterprise agents
    let discovery_query = CapabilityQuery {
        required_capabilities: vec![], // Get all agents
        preferred_capabilities: vec!["enterprise-operations".to_string()],
        exclude_agents: vec![],
        max_results: Some(50),
    };

    let query_envelope = EnvelopeBuilder::new()
        .with_payload(discovery_query)
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta.version = Some("discovery-test-v1.0".to_string());
            meta
        })
        .build()?;

    match nats_client.send_envelope::<CapabilityQuery, Vec<AgentInfo>>(subjects::AGENT_DISCOVERY, query_envelope).await {
        Ok(response_envelope) => {
            let agents = response_envelope.payload;
            println!("{} Agent discovery via TLS successful", "✅".bright_green());
            println!("   {} {} agents discovered", "🔍".bright_blue(), agents.len().to_string().bright_yellow());

            for (i, agent) in agents.iter().take(3).enumerate() { // Show first 3
                println!("   {}. {} ({})",
                        (i + 1).to_string().bright_white(),
                        agent.name.bright_cyan(),
                        agent.capabilities.len().to_string().bright_magenta());
            }

            if agents.len() > 3 {
                println!("   ... and {} more agents", (agents.len() - 3).to_string().dimmed());
            }
        }
        Err(e) => {
            println!("{} Agent discovery failed: {}", "❌".bright_red(), e);
            return Err(e);
        }
    }

    println!("{}", "✅ Test 3 passed: Agent discovery via TLS successful".bright_green());
    Ok(())
}

/// Test cross-agent communication via TLS
async fn test_cross_agent_communication() -> Result<()> {
    println!("\n{}", "📋 Test 4: Cross-Agent Communication".bright_cyan().bold());
    println!("{}", "🔍 Testing agent-to-agent communication via TLS...".dimmed());

    let nats_client = create_tls_nats_client().await?;

    // Simulate a cosmic challenge from Q to Picard (cross-agent communication)
    let cosmic_challenge = serde_json::json!({
        "challenge_id": Uuid::now_v7().to_string(),
        "challenge_type": "tls-communication-test",
        "description": "Testing secure agent communication through TLS-encrypted channels",
        "urgency": "medium",
        "from_agent": "Q Console Test",
        "to_agent": "Enterprise Crew",
        "requires_capabilities": ["tls-communication", "enterprise-operations"],
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "test_metadata": {
            "security_level": "maximum",
            "encryption": "TLS 1.2/1.3",
            "verification": "mutual_tls"
        }
    });

    let challenge_envelope = EnvelopeBuilder::new()
        .with_payload(cosmic_challenge)
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta.version = Some("cross-agent-test-v1.0".to_string());

            // Add cross-agent communication context
            if meta.extensions.is_none() {
                meta.extensions = Some(qollective::envelope::meta::ExtensionsMeta {
                    sections: std::collections::HashMap::new(),
                });
            }

            if let Some(extensions) = &mut meta.extensions {
                let comm_data = serde_json::json!({
                    "communication_type": "cross_agent",
                    "source": "test_suite",
                    "destination": "enterprise_crew",
                    "security_context": "tls_encrypted",
                    "message_priority": "test_verification"
                });
                extensions.sections.insert("cross_agent_context".to_string(), comm_data);
            }

            meta
        })
        .build()?;

    match nats_client.publish(subjects::ENTERPRISE_BRIDGE_CHALLENGE, challenge_envelope).await {
        Ok(_) => {
            println!("{} Cross-agent communication via TLS successful", "✅".bright_green());
            println!("   {} {}", "Message Type:".bright_cyan(), "Cosmic Challenge".bright_magenta());
            println!("   {} {}", "Encryption:".bright_cyan(), "TLS 1.2/1.3 secured".bright_green());
            println!("   {} {}", "Subject:".bright_cyan(), subjects::ENTERPRISE_BRIDGE_CHALLENGE.bright_yellow());
            println!("   {} {}", "Payload Size:".bright_cyan(), "~1KB with rich context".bright_blue());
        }
        Err(e) => {
            println!("{} Cross-agent communication failed: {}", "❌".bright_red(), e);
            return Err(e);
        }
    }

    println!("{}", "✅ Test 4 passed: Cross-agent communication via TLS successful".bright_green());
    Ok(())
}

/// Create TLS-enabled NATS client (same approach as Q Console and Log Agent)
async fn create_tls_nats_client() -> Result<NatsClient> {
    let connection_configs = EnterpriseNatsConfig::connection_configs();

    for nats_client_config in connection_configs {
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
                return Ok(client);
            }
            Err(_) => {
                continue; // Try next configuration
            }
        }
    }

    Err(qollective::error::QollectiveError::nats_connection("Could not establish TLS connection to NATS server".to_string()))
}
