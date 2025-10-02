// ABOUTME: Captain Picard - Bridge Command Center using StarTrek agent patterns
// ABOUTME: Coordinates crew responses to Q's challenges through hybrid agent communication

//! Captain Jean-Luc Picard - Bridge Command
//!
//! The commanding officer who receives challenges from Q and coordinates
//! the Enterprise crew response through our agent system. Built using the
//! StarTrek Enterprise agent patterns for maximum reliability and reusability.

use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use uuid::Uuid;
use colored::Colorize;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use qollective::{
    types::a2a::{CapabilityQuery},
    envelope::{Envelope, EnvelopeBuilder, Context},
    error::Result,
    constants::subjects,
};

// Import StarTrek Enterprise agent patterns
use qollective_a2a_nats_enterprise::startrek_agent::{
    EnterpriseAgent,
    HybridMessageHandler,
};

// Star Trek specific modules
use qollective_a2a_nats_enterprise::startrek_types::{CosmicChallenge, ThreatLevel};

/// Picard's personality and speech patterns
struct PicardPersonality;

impl PicardPersonality {
    fn startup() -> String {
        "🏛️ Captain Jean-Luc Picard reporting for duty. Bridge systems online.".bright_blue().bold().to_string()
    }

    fn challenge_received(challenge_type: &str) -> String {
        match challenge_type {
            "unknown-contact" => "🚨 Red alert! Unknown contact detected. All hands to battle stations!".bright_red().bold(),
            "warp-core-anomaly" => "⚠️ Engineering emergency. Mr. La Forge, report to the bridge immediately!".bright_yellow().bold(),
            "temporal-anomaly" => "🌀 Temporal distortion detected. This requires careful analysis.".bright_cyan().bold(),
            "diplomatic-crisis" => "🤝 Diplomatic situation developing. Prepare for first contact protocols.".bright_green().bold(),
            "data-cascade" => "💾 Computer systems under stress. Data, your expertise is required.".bright_magenta().bold(),
            _ => "🎯 New situation developing. Assess and respond accordingly.".bright_white().bold(),
        }.to_string()
    }

    fn coordinating_crew() -> String {
        "📡 Coordinating crew response. Make it so.".bright_blue().to_string()
    }

    fn crew_report(crew_member: &str, response: &str) -> String {
        format!("{} {}: {}",
                "📋".bright_blue(),
                crew_member.bright_cyan().bold(),
                response.white())
    }

    fn final_decision(decision: &str) -> String {
        format!("{} {}",
                "⭐ Command Decision:".bright_blue().bold(),
                decision.bright_yellow())
    }
}

/// Crew response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrewResponse {
    pub crew_member: String,
    pub capability: String,
    pub response: String,
    pub recommendation: String,
    pub confidence: f64,
    pub timestamp: SystemTime,
}

/// Command decision structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandDecision {
    pub challenge_id: String,
    pub decision: String,
    pub reasoning: String,
    pub crew_involved: Vec<String>,
    pub timestamp: SystemTime,
}

/// Log entry for Enterprise Log Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub log_type: String,
    pub crew_member: String,
    pub rank: String,
    pub timestamp: SystemTime,
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

/// Hybrid message handler for Picard agent supporting both envelope and raw messages
pub struct PicardMessageHandler {
    agent: EnterpriseAgent,
}

impl PicardMessageHandler {
    pub fn new(agent: EnterpriseAgent) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl HybridMessageHandler for PicardMessageHandler {
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
        // Try to parse as CosmicChallenge first
        if let Ok(challenge) = serde_json::from_value::<CosmicChallenge>(envelope.payload.clone()) {
            return handle_challenge(&self.agent, challenge, context).await;
        }

        // Try to parse as CrewResponse
        if let Ok(crew_response) = serde_json::from_value::<CrewResponse>(envelope.payload.clone()) {
            return handle_crew_response(crew_response, context).await;
        }

        // Generic envelope message handling
        tracing::info!("📨 Picard: Processing generic Qollective envelope message");

        // Display envelope context
        println!("\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("{}", "📨 COMMAND BRIDGE - ENVELOPE RECEIVED:".bright_cyan().bold());

        if let Some(request_id) = &context.meta().request_id {
            println!("   📋 Request ID: {}", request_id.to_string().bright_green());
        }
        if let Some(timestamp) = &context.meta().timestamp {
            println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
        }
        if let Some(tenant) = &context.meta().tenant {
            println!("   🏢 Tenant: {}", tenant.bright_cyan());
        }

        println!("   📄 Data: {}", envelope.payload.to_string().chars().take(200).collect::<String>().bright_white());
        println!("{}", "═══════════════════════════════════════════════════════════════".bright_yellow().dimmed());

        Ok(())
    }

    async fn handle_raw_message(&self, payload: Vec<u8>) -> Result<()> {
        tracing::info!("📦 Picard: Processing raw NATS message");

        println!("\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("📦 COMMAND BRIDGE - RAW TRANSMISSION:");
        println!("   📄 Raw payload size: {} bytes", payload.len());

        // Try to parse as JSON for better display
        match serde_json::from_slice::<serde_json::Value>(&payload) {
            Ok(json_value) => {
                println!("   📋 JSON content: {}",
                        serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| "Invalid JSON".to_string()));
            }
            Err(_) => {
                // Handle binary or non-JSON data
                let text = String::from_utf8_lossy(&payload);
                println!("   📋 Text content: {}", text.chars().take(200).collect::<String>());
            }
        }

        println!("{}", "═══════════════════════════════════════════════════════════════".bright_yellow().dimmed());

        Ok(())
    }
}

/// Handle cosmic challenges from Q
async fn handle_challenge(
    agent: &EnterpriseAgent,
    challenge: CosmicChallenge,
    context: Context,
) -> Result<()> {
    println!("\n{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());
    println!("{}", "🎭 Q'S COSMIC CHALLENGE DETECTED! 🎭".bright_magenta().bold());
    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Q's Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Challenge Time: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", PicardPersonality::challenge_received(&challenge.challenge_type));
    println!("🎯 Challenge: {}", challenge.description.bright_white().bold());
    println!("⚡ Threat Level: {:?}", challenge.threat_level);
    println!("⏰ Estimated Duration: {} seconds", challenge.estimated_duration.as_secs().to_string().bright_yellow());

    if !challenge.affected_sectors.is_empty() {
        println!("🚨 Affected Sectors: {}", challenge.affected_sectors.join(", ").bright_red());
    }

    println!("\n{}", PicardPersonality::coordinating_crew());

    // Query available crew with relevant capabilities
    let capabilities_needed = determine_capabilities_needed(&challenge.challenge_type);
    let mut crew_responses = Vec::new();

    for capability in &capabilities_needed {
        println!("🔍 Requesting {} expertise...", capability.bright_cyan());

        // Query for agents with this capability
        let query = CapabilityQuery {
            required_capabilities: vec![capability.clone()],
            preferred_capabilities: Vec::new(),
            exclude_agents: Vec::new(),
            max_results: Some(5),
        };

        match agent.a2a_client.discover_agents(query).await {
            Ok(envelope) => {
                let agents = envelope.payload;
                if !agents.is_empty() {
                    let agent_info = &agents[0]; // Use first available agent
                    println!("   ✅ {} available for {}", agent_info.name.bright_cyan().bold(), capability);

                    // Create mock crew response for demonstration
                    let response = create_mock_crew_response(&agent_info.name, capability, &challenge);
                    crew_responses.push(response);
                } else {
                    println!("   ❌ No agents available for {}", capability.bright_red());
                }
            }
            Err(e) => {
                println!("   ⚠️ Error querying {}: {}", capability, e.to_string().dimmed());
            }
        }

        sleep(Duration::from_millis(500)).await; // Brief pause for realism
    }

    // Make command decision
    let decision = make_command_decision(&challenge, &crew_responses);
    println!("\n{}", PicardPersonality::final_decision(&decision.decision));
    println!("💭 Reasoning: {}", decision.reasoning.bright_white());

    if !decision.crew_involved.is_empty() {
        println!("👥 Crew Involved: {}", decision.crew_involved.join(", ").bright_green());
    }

    // Send log entry to Enterprise Log Agent
    if let Err(e) = send_command_log_entry(agent, &challenge, &decision, context).await {
        println!("⚠️ Failed to log command decision: {}", e.to_string().dimmed());
    }

    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().dimmed());

    Ok(())
}

/// Handle crew responses
async fn handle_crew_response(
    response: CrewResponse,
    context: Context,
) -> Result<()> {
    println!("\n{}", "═══════════════════════════════════════════════════════════════".bright_blue());
    println!("{}", PicardPersonality::crew_report(&response.crew_member, &response.response));
    println!("   🎯 Capability: {}", response.capability.bright_magenta());
    println!("   💡 Recommendation: {}", response.recommendation.bright_green());
    println!("   📊 Confidence: {:.1}%", response.confidence * 100.0);

    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Response ID: {}", request_id.to_string().bright_green());
    }

    println!("{}", "═══════════════════════════════════════════════════════════════".bright_blue().dimmed());

    Ok(())
}

/// Determine what capabilities are needed for a challenge type
fn determine_capabilities_needed(challenge_type: &str) -> Vec<String> {
    match challenge_type {
        "unknown-contact" => vec!["sensors".to_string(), "communications".to_string(), "tactical".to_string()],
        "warp-core-anomaly" => vec!["engineering".to_string(), "physics".to_string()],
        "temporal-anomaly" => vec!["science".to_string(), "temporal-mechanics".to_string()],
        "diplomatic-crisis" => vec!["diplomacy".to_string(), "linguistics".to_string()],
        "data-cascade" => vec!["data-analysis".to_string(), "computer-systems".to_string()],
        _ => vec!["science".to_string(), "engineering".to_string()],
    }
}

/// Create mock crew response for demonstration
fn create_mock_crew_response(crew_name: &str, capability: &str, challenge: &CosmicChallenge) -> CrewResponse {
    let (response, recommendation, confidence) = match capability {
        "engineering" => ("Warp core is stable, but I recommend reducing power by 15%", "Implement emergency protocols", 0.85),
        "science" => ("Analyzing quantum fluctuations. Fascinating patterns detected", "Recommend detailed sensor sweep", 0.92),
        "tactical" => ("Shields at maximum. Weapons systems ready", "Maintain defensive posture", 0.88),
        "sensors" => ("Long-range sensors detecting unusual energy signatures", "Increase sensor resolution", 0.90),
        _ => ("Standing by for orders, Captain", "Await further instructions", 0.75),
    };

    CrewResponse {
        crew_member: crew_name.to_string(),
        capability: capability.to_string(),
        response: response.to_string(),
        recommendation: recommendation.to_string(),
        confidence,
        timestamp: SystemTime::now(),
    }
}

/// Make command decision based on challenge and crew input
fn make_command_decision(challenge: &CosmicChallenge, crew_responses: &[CrewResponse]) -> CommandDecision {
    let decision = match challenge.challenge_type.as_str() {
        "unknown-contact" => "Proceed with first contact protocols. Maintain defensive readiness.",
        "warp-core-anomaly" => "Implement emergency engineering protocols. Reduce to impulse power.",
        "temporal-anomaly" => "Avoid direct interaction. Gather data from safe distance.",
        "diplomatic-crisis" => "Open diplomatic channels. Prepare cultural database.",
        "data-cascade" => "Implement data isolation protocols. Backup critical systems.",
        _ => "Proceed with caution. Monitor all systems.",
    };

    let reasoning = format!(
        "Based on analysis of {} challenge with severity {} and input from {} crew members, this decision balances safety with mission objectives.",
        challenge.challenge_type,
        format!("{:?}", challenge.threat_level),
        crew_responses.len()
    );

    let crew_involved: Vec<String> = crew_responses.iter()
        .map(|r| r.crew_member.clone())
        .collect();

    CommandDecision {
        challenge_id: challenge.challenge_id.clone(),
        decision: decision.to_string(),
        reasoning,
        crew_involved,
        timestamp: SystemTime::now(),
    }
}

/// Send command log entry to Enterprise Log Agent
async fn send_command_log_entry(
    agent: &EnterpriseAgent,
    challenge: &CosmicChallenge,
    decision: &CommandDecision,
    context: Context,
) -> Result<()> {
    let log_entry = LogEntry {
        log_type: "command".to_string(),
        crew_member: "Captain Jean-Luc Picard".to_string(),
        rank: "Captain".to_string(),
        timestamp: SystemTime::now(),
        correlation_id: Uuid::now_v7(),
        task_id: Uuid::now_v7(),
        task_type: format!("Q Challenge: {}", challenge.challenge_type),
        description: challenge.description.clone(),
        outcome: decision.decision.clone(),
        duration_seconds: 30, // Assume 30 seconds for command decision
        confidence_level: 0.95, // Picard is very confident in command decisions
        stress_level: match challenge.threat_level {
            ThreatLevel::Minimal => 0.1,
            ThreatLevel::Low => 0.2,
            ThreatLevel::Moderate => 0.4,
            ThreatLevel::High => 0.7,
            ThreatLevel::Extreme => 0.9,
            ThreatLevel::CosmicScale => 1.0,
        },
        skills_demonstrated: vec!["leadership".to_string(), "decision-making".to_string(), "crisis-management".to_string()],
        challenges_faced: vec![format!("Q Challenge: {}", challenge.challenge_type)],
        collaborated_with: decision.crew_involved.clone(),
        performance_rating: "Excellent".to_string(),
        notes: decision.reasoning.clone(),
    };

    // Create envelope with enterprise context
    let mut meta = qollective::envelope::Meta::default();
    meta.version = Some("1.0".to_string());
    meta.request_id = Some(Uuid::now_v7());
    meta.timestamp = Some(chrono::Utc::now());
    meta.tenant = Some("starfleet".to_string());

    let envelope = qollective::envelope::Envelope::new(meta, log_entry);

    // Send to log agent
    agent.nats_client.publish("enterprise.logging.entry", envelope).await?;

    println!("📋 Command log entry sent to Enterprise Log Agent");

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

    println!("{}", PicardPersonality::startup());
    println!("{}", "Using StarTrek Enterprise agent patterns for command authority...".bright_blue().dimmed());

    // Create Enterprise Picard Agent using StarTrek patterns
    let agent = EnterpriseAgent::builder("Captain Jean-Luc Picard")
        .with_capabilities(vec![
            "command".to_string(),
            "decision-making".to_string(),
            "crew-coordination".to_string(),
            "diplomacy".to_string(),
            "leadership".to_string(),
            "crisis-management".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
        ])
        .with_function("Commanding Officer specializing in command decisions, crew coordination, and diplomatic resolution")
        .with_location("Bridge - Command Chair")
        .with_service_type("command_authority")
        .build()
        .await?;

    println!("{}", "🏛️ Captain Picard online. Command authority established using StarTrek patterns.".bright_blue().bold());
    println!("{}", "🔄 Intelligent reconnection monitoring enabled - command will auto-recover from server failures.".bright_blue().dimmed());

    // Create hybrid message handler
    let handler = PicardMessageHandler::new(agent.clone());

    // Subscribe to Q's challenges and crew responses with hybrid handling
    println!("{}", "📡 Starting command bridge message subscriptions...".bright_blue().dimmed());

    agent.subscribe_with_hybrid_handling("enterprise.bridge.challenge", PicardMessageHandler::new(agent.clone())).await?;
    agent.subscribe_with_hybrid_handling("enterprise.crew.response", handler).await?;

    println!("{}", "📋 Listening on: enterprise.bridge.challenge".bright_cyan());
    println!("{}", "📋 Listening on: enterprise.crew.response".bright_cyan());
    println!("{}", "✅ Command bridge established - supports both envelope and raw messages".bright_green().bold());
    println!("{}", "🖖 Ready to handle Q's challenges and coordinate crew responses. Make it so!".bright_blue().bold());

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.map_err(|e| qollective::error::QollectiveError::validation(format!("Signal handling error: {}", e)))?;

    // Graceful shutdown
    println!("\n{}", "🖖 Captain Picard signing off. End of watch.".bright_blue().bold());
    agent.shutdown().await?;

    Ok(())
}
