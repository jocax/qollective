// ABOUTME: Montgomery "Scotty" Scott - Chief Engineer using StarTrek agent patterns
// ABOUTME: Handles engineering tasks and diagnostics through hybrid agent communication patterns

//! Chief Engineer Montgomery "Scotty" Scott - Engineering Deck
//!
//! The miracle worker who keeps the Enterprise running and responds to
//! engineering challenges with Scottish flair and technical expertise.
//! Built using the StarTrek Enterprise agent patterns for maximum reliability.

use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use uuid::Uuid;
use colored::Colorize;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use qollective::{
    envelope::{Envelope, Context},
    error::Result,
};

// Import StarTrek Enterprise agent patterns
use qollective_a2a_nats_enterprise::startrek_agent::{
    EnterpriseAgent,
    HybridMessageHandler,
};

// Star Trek specific modules
use qollective_a2a_nats_enterprise::startrek_types::{CosmicChallenge, ThreatLevel};

/// Scotty's personality and speech patterns
struct ScottyPersonality;

impl ScottyPersonality {
    fn startup() -> String {
        "🔧 Montgomery Scott reporting for duty! Engineering systems are purring like a kitten, Captain!".bright_yellow().bold().to_string()
    }

    fn task_received(task_type: &str) -> String {
        match task_type {
            "warp-core-anomaly" => "🚨 Och! The warp core's acting up again! Don't ye worry, I'll sort her out!".bright_red().bold(),
            "systems-diagnostics" => "🔍 Running full diagnostics, Captain! Every circuit, every relay!".bright_green().bold(),
            "power-management" => "⚡ Power systems under my watchful eye! She'll give ye all she's got!".bright_yellow().bold(),
            "engineering" => "🛠️ Engineering challenge accepted! Time to work some miracles!".bright_cyan().bold(),
            _ => "🔧 Aye, I'll take a look at that technical problem right away!".bright_white().bold(),
        }.to_string()
    }

    fn analyzing() -> String {
        "🔍 Running diagnostics... checking plasma flow, dilithium matrix, EPS conduits...".bright_yellow().to_string()
    }

    fn problem_found(issue: &str) -> String {
        format!("{} {}",
                "⚠️ Found the culprit:".bright_red().bold(),
                issue.bright_white())
    }

    fn solution(fix: &str) -> String {
        format!("{} {}",
                "🔧 My recommendation:".bright_green().bold(),
                fix.bright_yellow())
    }

    fn confidence(level: f64) -> String {
        let emoji = if level > 0.9 { "💯" } else if level > 0.7 { "👍" } else { "🤔" };
        format!("{} I'm {:.0}% certain this'll work, Captain!", emoji, level * 100.0)
    }

    fn miracle_needed() -> String {
        "✨ Ye need a miracle? Well, ye've come to the right engineer!".bright_yellow().bold().to_string()
    }

    fn emergency_response() -> String {
        "🚨 Emergency response protocols engaged! Routing power from all non-essential systems!".bright_red().bold().to_string()
    }
}

/// Task request from captain or other crew
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrewTaskRequest {
    pub challenge_id: String,
    pub challenge_type: String,
    pub description: String,
    pub required_capability: String,
    pub urgency: String,
    pub from_captain: bool,
}

/// Engineering response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EngineeringResponse {
    pub challenge_id: String,
    pub engineer: String,
    pub analysis: String,
    pub recommendation: String,
    pub confidence: f64,
    pub estimated_time: Duration,
    pub resources_needed: Vec<String>,
    pub risk_assessment: String,
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

/// Hybrid message handler for Scotty agent supporting both envelope and raw messages
pub struct ScottyMessageHandler {
    agent: EnterpriseAgent,
}

impl ScottyMessageHandler {
    pub fn new(agent: EnterpriseAgent) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl HybridMessageHandler for ScottyMessageHandler {
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
        // Try to parse as CosmicChallenge first
        if let Ok(challenge) = serde_json::from_value::<CosmicChallenge>(envelope.payload.clone()) {
            return handle_cosmic_challenge(&self.agent, challenge, context).await;
        }

        // Try to parse as CrewTaskRequest
        if let Ok(task_request) = serde_json::from_value::<CrewTaskRequest>(envelope.payload.clone()) {
            return handle_engineering_task(&self.agent, task_request, context).await;
        }

        // Generic envelope message handling
        tracing::info!("📨 Scotty: Processing generic Qollective envelope message");

        // Display envelope context
        println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("{}", "📨 ENGINEERING DECK - ENVELOPE RECEIVED:".bright_cyan().bold());

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
        tracing::info!("📦 Scotty: Processing raw NATS message");

        println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("📦 ENGINEERING DECK - RAW TRANSMISSION:");
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

/// Handle cosmic challenges from Q with engineering expertise
async fn handle_cosmic_challenge(
    agent: &EnterpriseAgent,
    challenge: CosmicChallenge,
    context: Context,
) -> Result<()> {
    println!("\\n{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());
    println!("{}", "🎭 Q'S COSMIC CHALLENGE - ENGINEERING RESPONSE! 🎭".bright_magenta().bold());
    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Q's Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Challenge Time: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", ScottyPersonality::task_received(&challenge.challenge_type));
    println!("🎯 Challenge: {}", challenge.description.bright_white().bold());
    println!("⚡ Threat Level: {:?}", challenge.threat_level);
    println!("⏰ Estimated Duration: {} seconds", challenge.estimated_duration.as_secs().to_string().bright_yellow());

    if !challenge.affected_sectors.is_empty() {
        println!("🚨 Affected Sectors: {}", challenge.affected_sectors.join(", ").bright_red());
    }

    println!("\\n{}", ScottyPersonality::analyzing());
    sleep(Duration::from_millis(2000)).await; // Realistic analysis time

    // Engineering analysis based on challenge type
    let (analysis, recommendation, confidence, resources) = analyze_engineering_challenge(&challenge).await;

    println!("\\n{}", ScottyPersonality::problem_found(&analysis));
    println!("{}", ScottyPersonality::solution(&recommendation));
    println!("{}", ScottyPersonality::confidence(confidence));

    if !resources.is_empty() {
        println!("🛠️ Resources needed: {}", resources.join(", ").bright_cyan());
    }

    // Create engineering response
    let response = EngineeringResponse {
        challenge_id: challenge.challenge_id.clone(),
        engineer: "Montgomery Scott".to_string(),
        analysis,
        recommendation: recommendation.clone(),
        confidence,
        estimated_time: Duration::from_secs(1800), // 30 minutes for most engineering tasks
        resources_needed: resources,
        risk_assessment: assess_risk(&challenge.threat_level),
        timestamp: SystemTime::now(),
    };

    // Send response to bridge
    if let Err(e) = send_engineering_response(agent, &response, context.clone()).await {
        println!("⚠️ Failed to send engineering response: {}", e.to_string().dimmed());
    }

    // Send log entry to Enterprise Log Agent
    if let Err(e) = send_engineering_log_entry(agent, &challenge, &response, context).await {
        println!("⚠️ Failed to log engineering response: {}", e.to_string().dimmed());
    }

    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().dimmed());

    Ok(())
}

/// Handle regular engineering tasks from crew
async fn handle_engineering_task(
    agent: &EnterpriseAgent,
    task: CrewTaskRequest,
    context: Context,
) -> Result<()> {
    println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
    println!("{}", "🛠️ ENGINEERING TASK REQUEST:".bright_cyan().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", ScottyPersonality::task_received(&task.challenge_type));
    println!("   🎯 Task: {}", task.description.bright_white());
    println!("   🛠️ Capability: {}", task.required_capability.bright_magenta());
    println!("   ⚡ Urgency: {}", task.urgency.bright_red());
    println!("   👤 From Captain: {}", if task.from_captain { "Yes".bright_green() } else { "No".bright_yellow() });

    println!("\\n{}", ScottyPersonality::analyzing());
    sleep(Duration::from_millis(1500)).await; // Analysis time

    // Perform engineering analysis
    let (analysis, recommendation, confidence) = perform_engineering_analysis(&task).await;

    println!("\\n{}", ScottyPersonality::problem_found(&analysis));
    println!("{}", ScottyPersonality::solution(&recommendation));
    println!("{}", ScottyPersonality::confidence(confidence));

    // Create engineering response
    let response = EngineeringResponse {
        challenge_id: task.challenge_id.clone(),
        engineer: "Montgomery Scott".to_string(),
        analysis,
        recommendation: recommendation.clone(),
        confidence,
        estimated_time: Duration::from_secs(900), // 15 minutes for regular tasks
        resources_needed: get_required_resources(&task.required_capability),
        risk_assessment: "Low to moderate risk with proper procedures".to_string(),
        timestamp: SystemTime::now(),
    };

    // Send response
    if let Err(e) = send_crew_response(agent, &response, context).await {
        println!("⚠️ Failed to send engineering response: {}", e.to_string().dimmed());
    }

    println!("{}", "═══════════════════════════════════════════════════════════════".bright_yellow().dimmed());

    Ok(())
}

/// Analyze engineering challenges with Scottish engineering wisdom
async fn analyze_engineering_challenge(challenge: &CosmicChallenge) -> (String, String, f64, Vec<String>) {
    match challenge.challenge_type.as_str() {
        "warp-core-anomaly" => (
            "Quantum flux destabilization in the dilithium matrix causing antimatter containment fluctuations".to_string(),
            "Realign the magnetic constrictors and recalibrate the matter/antimatter injection assembly. We'll need to shut down the core for 20 minutes".to_string(),
            0.92,
            vec!["Dilithium crystals".to_string(), "Magnetic constrictors".to_string(), "Emergency power cells".to_string()]
        ),
        "power-management" => (
            "Power grid overload due to increased demand from multiple ship systems".to_string(),
            "Reroute power through auxiliary conduits and implement load balancing protocols. Non-essential systems to standby".to_string(),
            0.88,
            vec!["EPS conduits".to_string(), "Power regulators".to_string()]
        ),
        "systems-diagnostics" => (
            "Cascading system failures detected across multiple subsystems".to_string(),
            "Run level-5 diagnostics on all primary systems and replace any faulty isolinear chips. Should take about 45 minutes".to_string(),
            0.85,
            vec!["Isolinear chips".to_string(), "Diagnostic tools".to_string(), "Tricorder".to_string()]
        ),
        "temporal-anomaly" => (
            "Temporal distortions affecting ship's chronometer and structural integrity field".to_string(),
            "Modulate the deflector array to create a temporal shielding effect. Dangerous work, but doable".to_string(),
            0.70,
            vec!["Deflector array".to_string(), "Temporal sensors".to_string(), "Structural integrity field generators".to_string()]
        ),
        _ => (
            "Complex technical anomaly requiring detailed analysis".to_string(),
            "Implement standard engineering protocols and monitor all systems carefully. When in doubt, apply more power!".to_string(),
            0.75,
            vec!["Standard engineering kit".to_string(), "Monitoring equipment".to_string()]
        )
    }
}

/// Perform standard engineering task analysis
async fn perform_engineering_analysis(task: &CrewTaskRequest) -> (String, String, f64) {
    match task.required_capability.as_str() {
        "engineering" => (
            "Standard engineering procedures applicable with minor modifications".to_string(),
            "Apply proper safety protocols and use calibrated instruments. Should be straightforward".to_string(),
            0.90
        ),
        "systems-diagnostics" => (
            "Requires comprehensive system scan to identify root cause".to_string(),
            "Run full diagnostic sweep and replace any failing components immediately".to_string(),
            0.87
        ),
        "power-management" => (
            "Power distribution optimization needed for maximum efficiency".to_string(),
            "Balance load across all power grids and optimize energy flow patterns".to_string(),
            0.85
        ),
        _ => (
            "Technical challenge requiring engineering expertise".to_string(),
            "Apply standard engineering principles and monitor results closely".to_string(),
            0.80
        )
    }
}

/// Assess risk based on threat level
fn assess_risk(threat_level: &ThreatLevel) -> String {
    match threat_level {
        ThreatLevel::Minimal => "Minimal risk - standard procedures sufficient".to_string(),
        ThreatLevel::Low => "Low risk - exercise normal caution".to_string(),
        ThreatLevel::Moderate => "Moderate risk - enhanced safety protocols recommended".to_string(),
        ThreatLevel::High => "High risk - maximum safety precautions required".to_string(),
        ThreatLevel::Extreme => "Extreme risk - emergency protocols and backup systems essential".to_string(),
        ThreatLevel::CosmicScale => "Cosmic scale risk - all safety measures and contingency plans active".to_string(),
    }
}

/// Get required resources for capability
fn get_required_resources(capability: &str) -> Vec<String> {
    match capability {
        "engineering" => vec!["Engineering kit".to_string(), "Tricorder".to_string()],
        "systems-diagnostics" => vec!["Diagnostic tools".to_string(), "Replacement components".to_string()],
        "power-management" => vec!["Power regulators".to_string(), "EPS conduits".to_string()],
        "warp-drive-maintenance" => vec!["Dilithium crystals".to_string(), "Warp coils".to_string()],
        _ => vec!["Standard tools".to_string()],
    }
}

/// Send engineering response to bridge
async fn send_engineering_response(
    agent: &EnterpriseAgent,
    response: &EngineeringResponse,
    _context: Context,
) -> Result<()> {
    // Create envelope with enterprise context
    let mut meta = qollective::envelope::Meta::default();
    meta.version = Some("1.0".to_string());
    meta.request_id = Some(Uuid::now_v7());
    meta.timestamp = Some(chrono::Utc::now());
    meta.tenant = Some("starfleet".to_string());

    let envelope = qollective::envelope::Envelope::new(meta, response);

    // Send to bridge
    agent.nats_client.publish("enterprise.crew.response", envelope).await?;

    println!("📡 Engineering response sent to bridge");

    Ok(())
}

/// Send response to crew member
async fn send_crew_response(
    agent: &EnterpriseAgent,
    response: &EngineeringResponse,
    _context: Context,
) -> Result<()> {
    // Create envelope with enterprise context
    let mut meta = qollective::envelope::Meta::default();
    meta.version = Some("1.0".to_string());
    meta.request_id = Some(Uuid::now_v7());
    meta.timestamp = Some(chrono::Utc::now());
    meta.tenant = Some("starfleet".to_string());

    let envelope = qollective::envelope::Envelope::new(meta, response);

    // Send to crew response channel
    agent.nats_client.publish("enterprise.crew.response", envelope).await?;

    println!("📡 Engineering response sent to crew");

    Ok(())
}

/// Send engineering log entry to Enterprise Log Agent
async fn send_engineering_log_entry(
    agent: &EnterpriseAgent,
    challenge: &CosmicChallenge,
    response: &EngineeringResponse,
    _context: Context,
) -> Result<()> {
    let log_entry = LogEntry {
        log_type: "engineering".to_string(),
        crew_member: "Montgomery Scott".to_string(),
        rank: "Lieutenant Commander".to_string(),
        timestamp: SystemTime::now(),
        correlation_id: Uuid::now_v7(),
        task_id: Uuid::now_v7(),
        task_type: format!("Engineering Challenge: {}", challenge.challenge_type),
        description: challenge.description.clone(),
        outcome: response.recommendation.clone(),
        duration_seconds: response.estimated_time.as_secs(),
        confidence_level: response.confidence,
        stress_level: match challenge.threat_level {
            ThreatLevel::Minimal => 0.1,
            ThreatLevel::Low => 0.2,
            ThreatLevel::Moderate => 0.4,
            ThreatLevel::High => 0.6,
            ThreatLevel::Extreme => 0.8,
            ThreatLevel::CosmicScale => 0.9,
        },
        skills_demonstrated: vec!["engineering".to_string(), "problem-solving".to_string(), "technical-analysis".to_string()],
        challenges_faced: vec![format!("Engineering Challenge: {}", challenge.challenge_type)],
        collaborated_with: vec!["Bridge crew".to_string()],
        performance_rating: "Excellent".to_string(),
        notes: format!("Engineering assessment: {}. Risk: {}", response.analysis, response.risk_assessment),
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

    println!("📋 Engineering log entry sent to Enterprise Log Agent");

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

    println!("{}", ScottyPersonality::startup());
    println!("{}", "Using StarTrek Enterprise agent patterns for engineering excellence...".bright_yellow().dimmed());

    // Create Enterprise Scotty Agent using StarTrek patterns
    let agent = EnterpriseAgent::builder("Montgomery Scott")
        .with_capabilities(vec![
            "engineering".to_string(),
            "systems-diagnostics".to_string(),
            "power-management".to_string(),
            "warp-drive-maintenance".to_string(),
            "technical-analysis".to_string(),
            "problem-solving".to_string(),
            "emergency-repair".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
            "hybrid-message-handling".to_string(),
        ])
        .with_function("Chief Engineer specializing in warp drive maintenance, power systems, and emergency repairs")
        .with_location("Engineering Deck - Main Engineering")
        .with_service_type("engineering_systems")
        .build()
        .await?;

    println!("{}", "🔧 Montgomery Scott online. Engineering systems nominal using StarTrek patterns.".bright_yellow().bold());
    println!("{}", "🔄 Intelligent reconnection monitoring enabled - engineering will auto-recover from server failures.".bright_yellow().dimmed());

    // Create hybrid message handler
    let handler = ScottyMessageHandler::new(agent.clone());

    // Subscribe to engineering tasks and challenges with hybrid handling
    println!("{}", "📡 Starting engineering message subscriptions...".bright_yellow().dimmed());

    agent.subscribe_with_hybrid_handling("enterprise.bridge.challenge", ScottyMessageHandler::new(agent.clone())).await?;
    agent.subscribe_with_hybrid_handling("enterprise.engineering.task", handler).await?;

    println!("{}", "📋 Listening on: enterprise.bridge.challenge".bright_cyan());
    println!("{}", "📋 Listening on: enterprise.engineering.task".bright_cyan());
    println!("{}", "✅ Engineering deck operational - supports both envelope and raw messages".bright_green().bold());
    println!("{}", "🛠️ Ready to handle all engineering challenges and miracles! She'll give ye all she's got!".bright_yellow().bold());

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.map_err(|e| qollective::error::QollectiveError::validation(format!("Signal handling error: {}", e)))?;

    // Graceful shutdown
    println!("\\n{}", "🖖 Scotty signing off. Engineering deck secured.".bright_yellow().bold());
    agent.shutdown().await?;

    Ok(())
}
