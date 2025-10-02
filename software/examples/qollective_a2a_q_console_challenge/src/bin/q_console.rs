// ABOUTME: Q Console - Omnipotent entity that challenges the Enterprise crew
// ABOUTME: Interactive command interface for testing the multi-agent Star Trek system

//! Q Console - The Omnipotent Entity
//!
//! Q's console for sending challenges and commands to Captain Picard and the Enterprise crew.
//! Demonstrates real NATS integration with our agent system through dramatic cosmic challenges.

use std::{
    collections::HashMap,
    io::{self, Write},
    time::{Duration, SystemTime},
};
use tokio::time::sleep;
use uuid::Uuid;
use colored::Colorize;

use qollective::{
    types::a2a::{
        AgentInfo, HealthStatus, CapabilityQuery,
    },
    client::nats::NatsClient,
    envelope::EnvelopeBuilder,
    error::Result,
    constants::subjects,
};

// Star Trek specific modules
use qollective_a2a_nats_enterprise::startrek_types::*;

// Import smart TLS path resolution
use qollective::constants::network::tls_paths;

/// Q's personality and speech patterns
struct QPersonality;

impl QPersonality {
    fn introduction() -> String {
        "🌟 Ah, the mortal Enterprise crew awakens! I am Q, and today... I have some delightful tests for you.".bright_magenta().bold().to_string()
    }

    fn challenge_intro() -> String {
        "💫 Captain, my dear Captain... let me present you with a cosmic puzzle worthy of your... limited abilities.".bright_magenta().to_string()
    }

    fn command_sent(command: &str) -> String {
        format!("⚡ {} {}", "Q sends forth:".bright_magenta().bold(), command.bright_yellow())
    }

    fn waiting() -> String {
        "✨ Q observes with cosmic amusement...".bright_magenta().dimmed().to_string()
    }

    fn farewell() -> String {
        "🌌 How... disappointing. Until we meet again, Captain. *snaps fingers*".bright_magenta().bold().to_string()
    }
}

// Note: CosmicChallenge is now imported from startrek_types

/// Q's agent information
fn create_q_agent() -> AgentInfo {
    AgentInfo {
        id: Uuid::now_v7(),
        name: "Q".to_string(),
        capabilities: vec![
            "omnipotence".to_string(),
            "testing-mortals".to_string(),
            "cosmic-challenges".to_string(),
            "reality-manipulation".to_string(),
        ],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("location".to_string(), "Continuum".to_string()),
            ("species".to_string(), "Q".to_string()),
            ("power_level".to_string(), "Omnipotent".to_string()),
        ]),
    }
}

/// Available cosmic challenges Q can send with rich context
fn get_available_challenges() -> Vec<(&'static str, CosmicChallenge)> {
    vec![
        ("1", CosmicChallenge {
            challenge_id: Uuid::now_v7().to_string(),
            challenge_type: "unknown-contact".to_string(),
            description: "An unknown vessel of impossible design approaches! Its energy signature defies all known physics.".to_string(),
            urgency: "high".to_string(),
            requires_capabilities: vec!["tactical-analysis".to_string(), "science".to_string(), "command".to_string()],
            timestamp: SystemTime::now(),
            // Rich context additions
            threat_level: ThreatLevel::High,
            affected_sectors: vec!["Alpha-7".to_string(), "Beta-2".to_string()],
            estimated_duration: Duration::from_secs(3600), // 1 hour
            resource_requirements: ResourceManifest {
                power_level: 0.85,
                personnel_available: 850,
                auxiliary_craft: 2,
                torpedo_count: 20,
                ..Default::default()
            },
            previous_encounters: vec!["Borg first contact".to_string(), "Q Continuum visits".to_string()],
            similar_challenges: vec!["Species 8472 encounter".to_string()],
            q_test_parameters: Some(QTestParameters {
                test_focus: "Human curiosity vs caution balance".to_string(),
                expected_behaviors: vec!["Diplomatic approach".to_string(), "Scientific analysis".to_string()],
                amusement_level: 0.73,
                is_series_test: true,
                prior_test_influence: Some("Previous Q tests show crew learns from experience".to_string()),
            }),
        }),
        ("2", CosmicChallenge {
            challenge_id: Uuid::now_v7().to_string(),
            challenge_type: "warp-core-anomaly".to_string(),
            description: "The warp core is exhibiting quantum fluctuations that shouldn't exist in this reality!".to_string(),
            urgency: "critical".to_string(),
            requires_capabilities: vec!["engineering".to_string(), "science".to_string(), "systems-diagnostics".to_string()],
            timestamp: SystemTime::now(),
            // Rich context additions
            threat_level: ThreatLevel::Extreme,
            affected_sectors: vec!["Ship Internal".to_string()],
            estimated_duration: Duration::from_secs(1800), // 30 minutes to resolve
            resource_requirements: ResourceManifest {
                power_level: 0.30, // Power critically low
                personnel_available: 1012,
                emergency_reserves: 0.95, // Need emergency power
                ..Default::default()
            },
            previous_encounters: vec!["Enterprise NCC-1701 core breach".to_string()],
            similar_challenges: vec!["Dilithium recrystallization failure".to_string()],
            q_test_parameters: Some(QTestParameters {
                test_focus: "Crisis leadership and technical problem solving".to_string(),
                expected_behaviors: vec!["Immediate evacuation protocols".to_string(), "Engineering expertise".to_string()],
                amusement_level: 0.91, // Q finds engineering crises amusing
                is_series_test: false,
                prior_test_influence: None,
            }),
        }),
        ("3", CosmicChallenge {
            challenge_id: Uuid::now_v7().to_string(),
            challenge_type: "temporal-anomaly".to_string(),
            description: "Time itself seems to be... hiccupping. Past and future are becoming entangled.".to_string(),
            urgency: "extreme".to_string(),
            requires_capabilities: vec!["data-analysis".to_string(), "logic".to_string(), "risk-assessment".to_string()],
            timestamp: SystemTime::now(),
            // Rich context additions
            threat_level: ThreatLevel::CosmicScale,
            affected_sectors: vec!["Local space-time continuum".to_string(), "Sector 001".to_string()],
            estimated_duration: Duration::from_secs(7200), // 2 hours of temporal effects
            resource_requirements: ResourceManifest {
                power_level: 0.75,
                personnel_available: 900, // Some crew may be temporally displaced
                ..Default::default()
            },
            previous_encounters: vec!["Guardian of Forever".to_string(), "Temporal causality loop".to_string()],
            similar_challenges: vec!["Chroniton particle exposure".to_string()],
            q_test_parameters: Some(QTestParameters {
                test_focus: "Linear thinking vs temporal logic adaptation".to_string(),
                expected_behaviors: vec!["Vulcan logic application".to_string(), "Data's computational analysis".to_string()],
                amusement_level: 0.89,
                is_series_test: true,
                prior_test_influence: Some("Crew has shown ability to handle temporal paradoxes".to_string()),
            }),
        }),
        ("4", CosmicChallenge {
            challenge_id: Uuid::now_v7().to_string(),
            challenge_type: "diplomatic-crisis".to_string(),
            description: "Representatives from 12 species have arrived simultaneously, each claiming this sector as their own!".to_string(),
            urgency: "high".to_string(),
            requires_capabilities: vec!["command".to_string(), "decision-making".to_string(), "crew-coordination".to_string()],
            timestamp: SystemTime::now(),
            // Rich context additions
            threat_level: ThreatLevel::Moderate,
            affected_sectors: vec!["Neutral Zone border".to_string(), "Trade routes".to_string()],
            estimated_duration: Duration::from_secs(14400), // 4 hours of negotiations
            resource_requirements: ResourceManifest {
                power_level: 0.95,
                personnel_available: 1012,
                auxiliary_craft: 3, // May need diplomatic shuttles
                ..Default::default()
            },
            previous_encounters: vec!["Klingon-Federation peace talks".to_string(), "Cardassian treaty negotiations".to_string()],
            similar_challenges: vec!["Multi-party territorial disputes".to_string()],
            q_test_parameters: Some(QTestParameters {
                test_focus: "Federation diplomacy vs human nature".to_string(),
                expected_behaviors: vec!["Peaceful mediation".to_string(), "Cultural sensitivity".to_string()],
                amusement_level: 0.67,
                is_series_test: false,
                prior_test_influence: Some("Crew excels at diplomatic solutions".to_string()),
            }),
        }),
        ("5", CosmicChallenge {
            challenge_id: Uuid::now_v7().to_string(),
            challenge_type: "data-cascade".to_string(),
            description: "An alien data stream is overwhelming our computers with information that seems... alive.".to_string(),
            urgency: "medium".to_string(),
            requires_capabilities: vec!["data-analysis".to_string(), "computation".to_string(), "systems-diagnostics".to_string()],
            timestamp: SystemTime::now(),
            // Rich context additions
            threat_level: ThreatLevel::Moderate,
            affected_sectors: vec!["Computer core".to_string(), "Data networks".to_string()],
            estimated_duration: Duration::from_secs(5400), // 1.5 hours
            resource_requirements: ResourceManifest {
                power_level: 0.80,
                personnel_available: 950, // Some crew may lose computer access
                replicator_capacity: 0.60, // Computer-dependent systems affected
                ..Default::default()
            },
            previous_encounters: vec!["V'Ger entity".to_string(), "Borg collective consciousness".to_string()],
            similar_challenges: vec!["Sentient computer virus".to_string(), "AI emergence".to_string()],
            q_test_parameters: Some(QTestParameters {
                test_focus: "Artificial vs organic intelligence interaction".to_string(),
                expected_behaviors: vec!["Data's unique perspective".to_string(), "Cautious AI protocols".to_string()],
                amusement_level: 0.82,
                is_series_test: false,
                prior_test_influence: Some("Data's presence makes this particularly interesting".to_string()),
            }),
        }),
    ]
}

/// Display the Q console menu
fn display_menu() {
    println!("\n{}", "═══════════════════════════════════════".bright_magenta());
    println!("{}", "           Q'S COSMIC CONSOLE".bright_magenta().bold());
    println!("{}", "═══════════════════════════════════════".bright_magenta());

    let challenges = get_available_challenges();
    for (key, challenge) in challenges {
        println!("{} {} - {} ({})",
                 key.bright_yellow().bold(),
                 challenge.challenge_type.bright_cyan(),
                 challenge.description.white(),
                 format!("Urgency: {}", challenge.urgency).bright_red());
    }

    println!("\n{} {}", "a".bright_yellow().bold(), "- View available AgentCards".white());
    println!("{} {}", "c".bright_yellow().bold(), "- Custom challenge".white());
    println!("{} {}", "s".bright_yellow().bold(), "- Check crew status".white());
    println!("{} {}", "q".bright_yellow().bold(), "- Quit (snap fingers)".white());
    println!("{}", "═══════════════════════════════════════".bright_magenta());
}

/// Send challenge through NATS to the Enterprise crew with rich qollective envelope context
async fn send_challenge(
    nats_client: &NatsClient,
    challenge: CosmicChallenge,
) -> Result<()> {
    println!("{}", QPersonality::command_sent(&challenge.description));

    // Display rich challenge context to show what we're packing into the envelope
    println!("\n{}", "📦 Packing rich context into qollective envelope:".bright_cyan().bold());
    println!("   ⚠️  Threat Level: {:?}", challenge.threat_level);
    println!("   ⏱️  Est. Duration: {:.1} minutes", challenge.estimated_duration.as_secs_f64() / 60.0);
    println!("   🎯 Q Test Focus: {}",
             challenge.q_test_parameters.as_ref()
                .map(|q| q.test_focus.as_str())
                .unwrap_or("Standard challenge"));
    println!("   🚀 Resource Impact: {:.1}% power required",
             challenge.resource_requirements.power_level * 100.0);

    // Create envelope with enhanced metadata using qollective's extension system
    let mut meta = qollective::envelope::meta::Meta::default();
    meta.request_id = Some(Uuid::now_v7());
    meta.timestamp = Some(chrono::Utc::now());
    meta.tenant = Some("enterprise".to_string());
    meta.version = Some("enhanced-v1.0".to_string());

    // Pack rich context into qollective envelope extensions
    // This demonstrates the framework's capability to transport complex data
    if meta.extensions.is_none() {
        meta.extensions = Some(qollective::envelope::meta::ExtensionsMeta {
            sections: std::collections::HashMap::new(),
        });
    }

    if let Some(extensions) = &mut meta.extensions {
        // Pack Q's test parameters as extension data
        if let Some(q_params) = &challenge.q_test_parameters {
            let q_data = serde_json::json!({
                "test_focus": q_params.test_focus,
                "amusement_level": q_params.amusement_level,
                "expected_behaviors": q_params.expected_behaviors,
                "is_series_test": q_params.is_series_test
            });
            extensions.sections.insert("q_test_parameters".to_string(), q_data);
        }

        // Pack resource requirements for Enterprise planning
        let resource_data = serde_json::json!({
            "power_level": challenge.resource_requirements.power_level,
            "personnel_available": challenge.resource_requirements.personnel_available,
            "emergency_reserves": challenge.resource_requirements.emergency_reserves,
            "auxiliary_craft": challenge.resource_requirements.auxiliary_craft
        });
        extensions.sections.insert("resource_requirements".to_string(), resource_data);

        // Pack threat and tactical information
        let threat_data = serde_json::json!({
            "threat_level": challenge.threat_level,
            "affected_sectors": challenge.affected_sectors,
            "estimated_duration_seconds": challenge.estimated_duration.as_secs(),
            "previous_encounters": challenge.previous_encounters
        });
        extensions.sections.insert("threat_analysis".to_string(), threat_data);

        // Pack mission correlation data for tracking
        let correlation_data = serde_json::json!({
            "challenge_id": challenge.challenge_id,
            "similar_challenges": challenge.similar_challenges,
            "urgency_level": challenge.urgency,
            "timestamp": challenge.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        });
        extensions.sections.insert("mission_correlation".to_string(), correlation_data);
    }

    // Create the envelope with our rich metadata
    let envelope = EnvelopeBuilder::new()
        .with_payload(challenge)
        .with_meta(meta.clone())
        .build()?;

    // Display qollective envelope metadata to showcase the framework
    println!("\n{}", "🎭 QOLLECTIVE ENVELOPE METADATA:".bright_yellow().bold());
    println!("   📋 Request ID: {}", meta.request_id.as_ref().unwrap().to_string().bright_green());
    println!("   ⏰ Timestamp: {}", meta.timestamp.as_ref().unwrap().format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    println!("   🏢 Tenant: {}", meta.tenant.as_ref().unwrap().bright_cyan());
    println!("   📦 Version: {}", meta.version.as_ref().unwrap().bright_magenta());

    if let Some(extensions) = &meta.extensions {
        println!("   🔧 Extensions: {} sections packed", extensions.sections.len().to_string().bright_yellow());
        for (key, _) in &extensions.sections {
            println!("      └─ {}", key.bright_white());
        }
    }

    println!("   ✅ Rich context packed into qollective envelope extensions!");
    println!("   📡 Sending to enterprise.bridge.challenge...\n");

    // Send to bridge command subject
    nats_client.publish(subjects::ENTERPRISE_BRIDGE_CHALLENGE, envelope).await?;

    println!("{}", QPersonality::waiting());
    Ok(())
}

/// Check crew status by querying the Enterprise Registry directly via NATS
async fn check_crew_status(
    nats_client: &NatsClient,
) -> Result<()> {
    println!("{}", "🔍 Q peers into the mortal realm...".bright_magenta());

    // Query for all crew capabilities
    let capabilities = vec!["command", "engineering", "data-analysis", "logic"];

    for capability in capabilities {
        let query = CapabilityQuery {
            required_capabilities: vec![capability.to_string()],
            preferred_capabilities: vec![],
            exclude_agents: vec![],
            max_results: Some(10),
        };

        // Send discovery request directly to Enterprise Registry
        let envelope = EnvelopeBuilder::new()
            .with_payload(query)
            .with_meta({
                let mut meta = qollective::envelope::meta::Meta::default();
                meta.request_id = Some(Uuid::now_v7());
                meta.timestamp = Some(chrono::Utc::now());
                meta.tenant = Some("enterprise".to_string());
                meta
            })
            .build()?;

        match nats_client.send_envelope(subjects::AGENT_DISCOVERY, envelope).await {
            Ok(response) => {
                let agents: Vec<AgentInfo> = response.payload;
                if agents.is_empty() {
                    println!("  {} {} {}",
                             "⚠️".bright_red(),
                             capability.bright_cyan(),
                             "- No crew member available".bright_red());
                } else {
                    for agent in agents {
                        let status_color = match agent.health_status {
                            HealthStatus::Healthy => "🟢".green(),
                            HealthStatus::Warning => "🟡".yellow(),
                            HealthStatus::Unhealthy => "🔴".red(),
                            HealthStatus::Unknown => "⚪".white(),
                        };
                        println!("  {} {} {} ({})",
                                 status_color,
                                 capability.bright_cyan(),
                                 agent.name.bright_white(),
                                 format!("{:?}", agent.health_status).dimmed());
                    }
                }
            }
            Err(e) => {
                println!("  {} {} - {}", "❌".bright_red(), capability.bright_cyan(), e.to_string().red());
            }
        }
    }
    Ok(())
}

/// Display available AgentCards using A2A discovery
async fn display_agent_cards(nats_client: &NatsClient) -> Result<()> {
    println!("{}", "🎯 Q examines the A2A Agent Registry...".bright_magenta().bold());
    println!("{}", "═══════════════════════════════════════".bright_magenta());

    // Use the existing working NATS client instead of creating a new A2A client

    // Create a broad query to discover all available agents
    let query = CapabilityQuery {
        required_capabilities: vec![], // Empty to get all agents
        preferred_capabilities: vec![],
        exclude_agents: vec![],
        max_results: Some(50), // Get up to 50 agents
    };

    // Query the centralized registry directly instead of local registry
    let query_envelope = EnvelopeBuilder::new()
        .with_payload(query)
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta
        })
        .build()?;

    match nats_client.send_envelope::<CapabilityQuery, Vec<AgentInfo>>(subjects::AGENT_DISCOVERY, query_envelope).await {
        Ok(response_envelope) => {
            let agents = response_envelope.payload;

            if agents.is_empty() {
                println!("{}", "📋 No agents found in the registry.".bright_yellow());
                println!("{}", "💡 Agents may not be running or registered yet.".dimmed());
            } else {
                println!("{} {} {}",
                         "📋".bright_blue(),
                         "Agents discovered in Enterprise Registry:".bright_white().bold(),
                         format!("({} agents)", agents.len()).bright_cyan());

                for (index, agent) in agents.iter().enumerate() {
                    println!("\n{} {} {}",
                             format!("{}.", index + 1).bright_yellow().bold(),
                             agent.name.bright_cyan().bold(),
                             format!("({})", agent.id).dimmed());

                    println!("   🔧 Capabilities: {}",
                             agent.capabilities.join(", ").bright_magenta());
                    println!("   💚 Health: {:?}", agent.health_status);
                    println!("   🌐 Endpoint: nats://agent.{}", agent.id.to_string().bright_green());

                    if !agent.metadata.is_empty() {
                        println!("   📊 Metadata:");
                        for (key, value) in &agent.metadata {
                            println!("      • {}: {}", key.bright_white(), value.dimmed());
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to query Enterprise Registry:".bright_red(), e.to_string().dimmed());
            println!("{}", "💡 Make sure the enterprise-registry service is running.".dimmed());
        }
    }


    println!("{}", "═══════════════════════════════════════".bright_magenta());
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Initialize TLS crypto provider
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| qollective::error::QollectiveError::nats_connection("Failed to install TLS crypto provider".to_string()))?;

    // Initialize logging
    env_logger::init();

    println!("{}", QPersonality::introduction());
    println!("{}", "Attempting to connect to NATS server...".bright_magenta().dimmed());

    // Configure mTLS connection with client certificates
    println!("{} {}", "📁 TLS cert base path:".bright_cyan(), tls_paths::get_resolved_base_path().bright_yellow());

    // Use centralized configuration from config.toml
    let connection_configs = qollective_a2a_nats_enterprise::startrek_agent::EnterpriseNatsConfig::connection_configs();

    let mut nats_client = None;
    for nats_client_config in connection_configs {
        let url = nats_client_config.connection.urls[0].clone();
        println!("{} {}", "🔍 Trying:".bright_cyan(), url.dimmed());

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
                println!("{} {}", "✅ Connected to:".bright_green().bold(), url.bright_yellow());
                nats_client = Some(client);
                break;
            }
            Err(e) => {
                println!("{} {} - {}", "❌ Failed:".bright_red(), url.dimmed(), e.to_string().red().dimmed());
            }
        }
    }

    let nats_client = nats_client.ok_or_else(|| qollective::error::QollectiveError::nats_connection("Could not connect to any NATS server port".to_string()))?;
    // nats_client is already created above

    // Q Console does not need its own registry - it queries the Enterprise Registry via NATS
    // This eliminates the registry isolation problem

    // Q Console doesn't need A2AClient since it queries Enterprise Registry directly

    // Register Q with the Enterprise Registry
    let q_agent = create_q_agent();
    let q_envelope = EnvelopeBuilder::new()
        .with_payload(q_agent)
        .with_meta({
            let mut meta = qollective::envelope::meta::Meta::default();
            meta.request_id = Some(Uuid::now_v7());
            meta.timestamp = Some(chrono::Utc::now());
            meta.tenant = Some("enterprise".to_string());
            meta
        })
        .build()?;

    nats_client.publish(subjects::AGENT_REGISTRY_REGISTER, q_envelope).await?;

    println!("{}", "✨ Q is ready to test the mortal crew!".bright_magenta().bold());
    sleep(Duration::from_millis(1000)).await;

    // Main interaction loop
    loop {
        display_menu();
        print!("\n{} ", "Q's will:".bright_magenta().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "1" | "2" | "3" | "4" | "5" => {
                let challenges = get_available_challenges();
                if let Some((_, challenge)) = challenges.iter().find(|(key, _)| *key == input) {
                    if let Err(e) = send_challenge(&nats_client, challenge.clone()).await {
                        println!("{} {}", "Error sending challenge:".red(), e);
                    }
                    sleep(Duration::from_millis(2000)).await;
                }
            }
            "c" => {
                print!("{} ", "Enter your cosmic challenge:".bright_magenta());
                io::stdout().flush().unwrap();
                let mut custom_input = String::new();
                io::stdin().read_line(&mut custom_input).unwrap();
                let custom_challenge = CosmicChallenge {
                    challenge_id: Uuid::now_v7().to_string(),
                    challenge_type: "custom".to_string(),
                    description: custom_input.trim().to_string(),
                    urgency: "medium".to_string(),
                    requires_capabilities: vec!["command".to_string()],
                    timestamp: SystemTime::now(),
                    // Rich context for custom challenges
                    threat_level: ThreatLevel::Moderate,
                    affected_sectors: vec!["Unknown".to_string()],
                    estimated_duration: Duration::from_secs(3600), // 1 hour default
                    resource_requirements: ResourceManifest::default(),
                    previous_encounters: vec!["Custom Q challenge".to_string()],
                    similar_challenges: vec!["User-defined scenario".to_string()],
                    q_test_parameters: Some(QTestParameters {
                        test_focus: "Crew adaptability to unknown scenarios".to_string(),
                        expected_behaviors: vec!["Creative problem solving".to_string()],
                        amusement_level: 0.85, // Q enjoys custom challenges
                        is_series_test: false,
                        prior_test_influence: Some("Custom challenge from Q Console user".to_string()),
                    }),
                };
                if let Err(e) = send_challenge(&nats_client, custom_challenge).await {
                    println!("{} {}", "Error sending challenge:".red(), e);
                }
                sleep(Duration::from_millis(2000)).await;
            }
            "a" => {
                if let Err(e) = display_agent_cards(&nats_client).await {
                    println!("{} {}", "Error displaying AgentCards:".red(), e);
                }
                sleep(Duration::from_millis(1000)).await;
            }
            "s" => {
                if let Err(e) = check_crew_status(&nats_client).await {
                    println!("{} {}", "Error checking crew status:".red(), e);
                }
                sleep(Duration::from_millis(1000)).await;
            }
            "q" => {
                println!("{}", QPersonality::farewell());
                break;
            }
            _ => {
                println!("{}", "Q does not understand mortal input. Try again.".bright_red());
            }
        }
    }

    Ok(())
}
