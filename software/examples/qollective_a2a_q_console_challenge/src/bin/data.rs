// ABOUTME: Lt. Commander Data - Android Operations Officer using StarTrek agent patterns
// ABOUTME: Provides analytical and computational capabilities through hybrid agent communication patterns

//! Lieutenant Commander Data - Android Operations Officer
//!
//! The positronic android who provides analytical capabilities, tactical analysis,
//! and computational expertise to the Enterprise crew with curious precision.
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

/// Data's personality and speech patterns
struct DataPersonality;

impl DataPersonality {
    fn startup() -> String {
        "🤖 Lieutenant Commander Data reporting for duty. All analytical subroutines are functioning within normal parameters.".bright_cyan().bold().to_string()
    }

    fn task_received(task_type: &str) -> String {
        match task_type {
            "data-analysis" => "📊 Fascinating. Initiating comprehensive data analysis protocols.".bright_cyan().bold(),
            "tactical-analysis" => "⚔️ Beginning tactical assessment. Analyzing threat vectors and strategic options.".bright_red().bold(),
            "computation" => "🧮 Computational request received. Accessing positronic matrix processing capabilities.".bright_green().bold(),
            _ => "🔍 Intriguing problem detected. Commencing analytical evaluation.".bright_white().bold(),
        }.to_string()
    }

    fn analyzing() -> String {
        "🔄 Processing... accessing databases, running probability matrices, calculating outcomes...".bright_cyan().to_string()
    }

    fn pattern_detected(pattern: &str) -> String {
        format!("{} {}",
                "🎯 Pattern identified:".bright_cyan().bold(),
                pattern.bright_white())
    }

    fn recommendation(advice: &str) -> String {
        format!("{} {}",
                "💡 Recommendation:".bright_green().bold(),
                advice.bright_yellow())
    }

    fn probability(chance: f64) -> String {
        let emoji = if chance > 0.9 { "📈" } else if chance > 0.7 { "📊" } else if chance > 0.5 { "📉" } else { "❓" };
        format!("{} Probability of success: {:.2}%", emoji, chance * 100.0)
    }

    fn curiosity(observation: &str) -> String {
        format!("{} {}",
                "🤔 Curious observation:".bright_cyan().bold(),
                observation.bright_white())
    }

    fn computation_complete() -> String {
        "✅ Analysis complete. Results compiled and ready for transmission.".bright_green().bold().to_string()
    }
}

/// Task request from captain
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrewTaskRequest {
    pub challenge_id: String,
    pub challenge_type: String,
    pub description: String,
    pub required_capability: String,
    pub urgency: String,
    pub from_captain: bool,
}

/// Data's analytical response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalyticalResponse {
    pub challenge_id: String,
    pub analyst: String,
    pub analysis: String,
    pub recommendation: String,
    pub confidence: f64,
    pub probability_success: f64,
    pub alternative_options: Vec<String>,
    pub risk_factors: Vec<String>,
    pub data_quality: String,
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

/// Hybrid message handler for Data agent supporting both envelope and raw messages
pub struct DataMessageHandler {
    agent: EnterpriseAgent,
}

impl DataMessageHandler {
    pub fn new(agent: EnterpriseAgent) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl HybridMessageHandler for DataMessageHandler {
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
        // Try to parse as CosmicChallenge first
        if let Ok(challenge) = serde_json::from_value::<CosmicChallenge>(envelope.payload.clone()) {
            return handle_cosmic_challenge(&self.agent, challenge, context).await;
        }

        // Try to parse as CrewTaskRequest
        if let Ok(task_request) = serde_json::from_value::<CrewTaskRequest>(envelope.payload.clone()) {
            return handle_analytical_task(&self.agent, task_request, context).await;
        }

        // Generic envelope message handling
        tracing::info!("📨 Data: Processing generic Qollective envelope message");

        // Display envelope context
        println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("{}", "📨 OPERATIONS STATION - ENVELOPE RECEIVED:".bright_cyan().bold());

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
        tracing::info!("📦 Data: Processing raw NATS message");

        println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("📦 OPERATIONS STATION - RAW TRANSMISSION:");
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

/// Handle cosmic challenges from Q with positronic analysis
async fn handle_cosmic_challenge(
    agent: &EnterpriseAgent,
    challenge: CosmicChallenge,
    context: Context,
) -> Result<()> {
    println!("\\n{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());
    println!("{}", "🎭 Q'S COSMIC CHALLENGE - POSITRONIC ANALYSIS! 🎭".bright_magenta().bold());
    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Q's Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Challenge Time: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", DataPersonality::task_received(&challenge.challenge_type));
    println!("🎯 Challenge: {}", challenge.description.bright_white().bold());
    println!("⚡ Threat Level: {:?}", challenge.threat_level);
    println!("⏰ Estimated Duration: {} seconds", challenge.estimated_duration.as_secs().to_string().bright_yellow());

    if !challenge.affected_sectors.is_empty() {
        println!("🚨 Affected Sectors: {}", challenge.affected_sectors.join(", ").bright_red());
    }

    println!("\\n{}", DataPersonality::analyzing());
    sleep(Duration::from_millis(2000)).await; // Data processes very quickly but thoroughly

    // Transform CosmicChallenge to CrewTaskRequest for analysis
    let task_request = CrewTaskRequest {
        challenge_id: challenge.challenge_id.clone(),
        challenge_type: challenge.challenge_type.clone(),
        description: challenge.description.clone(),
        required_capability: "data-analysis".to_string(),
        urgency: match challenge.threat_level {
            ThreatLevel::Minimal => "Low".to_string(),
            ThreatLevel::Low => "Moderate".to_string(),
            ThreatLevel::Moderate => "High".to_string(),
            ThreatLevel::High => "Critical".to_string(),
            ThreatLevel::Extreme => "Emergency".to_string(),
            ThreatLevel::CosmicScale => "Q-Level Emergency".to_string(),
        },
        from_captain: true,
    };

    // Perform analytical assessment
    let response = perform_analytical_assessment(&task_request).await;

    println!("\\n{}", DataPersonality::pattern_detected(&response.analysis));
    println!("{}", DataPersonality::recommendation(&response.recommendation));
    println!("{}", DataPersonality::probability(response.probability_success));

    // Data often makes curious observations
    if response.confidence > 0.95 {
        println!("{}", DataPersonality::curiosity("This problem exhibits patterns remarkably similar to scenarios in my experience database."));
    } else if response.confidence < 0.6 {
        println!("{}", DataPersonality::curiosity("Insufficient data available. Additional sensor readings would improve analytical accuracy."));
    }

    println!("{}", DataPersonality::computation_complete());

    // Send response to bridge
    if let Err(e) = send_analytical_response(agent, &response, context.clone()).await {
        println!("⚠️ Failed to send analytical response: {}", e.to_string().dimmed());
    }

    // Send log entry to Enterprise Log Agent
    if let Err(e) = send_analytical_log_entry(agent, &challenge, &response, context).await {
        println!("⚠️ Failed to log analytical response: {}", e.to_string().dimmed());
    }

    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().dimmed());

    Ok(())
}

/// Handle regular analytical tasks from crew
async fn handle_analytical_task(
    agent: &EnterpriseAgent,
    task: CrewTaskRequest,
    context: Context,
) -> Result<()> {
    println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
    println!("{}", "📊 ANALYTICAL REQUEST:".bright_cyan().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", DataPersonality::task_received(&task.required_capability));
    println!("   🎯 Task: {}", task.description.bright_white());
    println!("   🛠️ Capability: {}", task.required_capability.bright_magenta());
    println!("   ⚡ Urgency: {}", task.urgency.bright_red());
    println!("   👤 From Captain: {}", if task.from_captain { "Yes".bright_green() } else { "No".bright_yellow() });

    println!("\\n{}", DataPersonality::analyzing());
    sleep(Duration::from_millis(2000)).await; // Data processes very quickly but thoroughly

    // Perform analytical assessment
    let response = perform_analytical_assessment(&task).await;

    println!("\\n{}", DataPersonality::pattern_detected(&response.analysis));
    println!("{}", DataPersonality::recommendation(&response.recommendation));
    println!("{}", DataPersonality::probability(response.probability_success));

    // Data often makes curious observations
    if response.confidence > 0.95 {
        println!("{}", DataPersonality::curiosity("This problem exhibits patterns remarkably similar to scenarios in my experience database."));
    } else if response.confidence < 0.6 {
        println!("{}", DataPersonality::curiosity("Insufficient data available. Additional sensor readings would improve analytical accuracy."));
    }

    println!("{}", DataPersonality::computation_complete());

    // Send response
    if let Err(e) = send_analytical_response(agent, &response, context).await {
        println!("⚠️ Failed to send analytical response: {}", e.to_string().dimmed());
    }

    println!("{}", "═══════════════════════════════════════════════════════════════".bright_yellow().dimmed());

    Ok(())
}

/// Perform Data's characteristic analytical assessment
async fn perform_analytical_assessment(task: &CrewTaskRequest) -> AnalyticalResponse {
    let (analysis, recommendation, confidence, probability, alternatives, risks, data_quality) = match task.challenge_type.as_str() {
        "unknown-contact" => (
            "Vessel configuration does not match any known starship designs in Federation, Klingon, Romulan, or Cardassian databases. Energy signature suggests technology 2.3 centuries more advanced than current Federation standard.".to_string(),
            "Recommend peaceful first contact protocols with enhanced defensive measures. Suggest communication on multiple subspace frequencies to establish dialogue.".to_string(),
            0.87,
            0.73,
            vec![
                "Withdraw to safe distance for extended observation".to_string(),
                "Attempt to communicate via universal translator".to_string(),
                "Send unmanned probe for closer inspection".to_string(),
            ],
            vec![
                "Unknown technological capabilities".to_string(),
                "Potentially hostile intentions".to_string(),
                "Prime Directive considerations".to_string(),
            ],
            "Moderate - sensor data is clear but lacks historical context".to_string(),
        ),
        "warp-core-anomaly" => (
            "Quantum fluctuation patterns in dilithium crystal matrix indicate 73.2% probability of cascade failure within 4.7 hours if current degradation rate continues. Antimatter containment field remains stable.".to_string(),
            "Immediate warp drive shutdown required. Replace primary dilithium crystals and recalibrate matter/antimatter injectors. Probability of successful repair: 94.3%.".to_string(),
            0.94,
            0.94,
            vec![
                "Emergency ejection of warp core if repair fails".to_string(),
                "Switch to auxiliary warp drive system".to_string(),
                "Impulse power only with extended repair schedule".to_string(),
            ],
            vec![
                "Warp core breach (low probability if action taken immediately)".to_string(),
                "Extended mission delay".to_string(),
                "Reduced defensive capabilities during repair".to_string(),
            ],
            "High - internal sensors provide excellent data fidelity".to_string(),
        ),
        "temporal-anomaly" => (
            "Temporal displacement readings indicate localized chronoton particle concentrations 847% above normal. Ship's chronometer variance suggests we are experiencing temporal flux at a rate of 1.3 seconds per minute.".to_string(),
            "Activate temporal stabilizers immediately. Increase structural integrity field to compensate for temporal stress. Avoid any course corrections until temporal field normalizes.".to_string(),
            0.76,
            0.82,
            vec![
                "Emergency temporal jump to escape anomaly".to_string(),
                "Deploy temporal beacon for Starfleet recovery".to_string(),
                "Use deflector array to create temporal barrier".to_string(),
            ],
            vec![
                "Temporal displacement to unknown time period".to_string(),
                "Potential paradox creation".to_string(),
                "Ship systems aging at accelerated rate".to_string(),
            ],
            "Moderate - temporal phenomena inherently difficult to measure accurately".to_string(),
        ),
        "diplomatic-crisis" => (
            "Analysis of cultural databases reveals 12 distinct species with conflicting territorial claims based on different interpretations of galactic law. 67% probability this is a orchestrated test of Federation diplomatic capabilities.".to_string(),
            "Suggest multi-party conference aboard Enterprise. Utilize Federation diplomatic protocols Article 12, Section 4. Present neutral arbitration based on galactic law precedents.".to_string(),
            0.85,
            0.78,
            vec![
                "Request Starfleet diplomatic corps intervention".to_string(),
                "Propose neutral meeting location on nearby planet".to_string(),
                "Suggest sequential bilateral meetings with each species".to_string(),
            ],
            vec![
                "Diplomatic failure leading to military conflict".to_string(),
                "Federation credibility damage".to_string(),
                "Prime Directive violations if forced to choose sides".to_string(),
            ],
            "Good - diplomatic database records are comprehensive".to_string(),
        ),
        "data-cascade" => (
            "Incoming data stream exhibits characteristics of self-modifying code with exponential replication patterns. 89.4% probability this is an artificial intelligence attempting to communicate, not malicious code.".to_string(),
            "Isolate data stream to secondary computer core with restricted access. Establish controlled communication protocols. Monitor for consciousness indicators while maintaining security barriers.".to_string(),
            0.89,
            0.85,
            vec![
                "Complete system isolation and data purge".to_string(),
                "Transfer data to isolated holodeck for safe interaction".to_string(),
                "Create communication buffer using positronic matrix".to_string(),
            ],
            vec![
                "System corruption or takeover".to_string(),
                "Potential hostile AI infiltration".to_string(),
                "Unknown technological contamination".to_string(),
            ],
            "High - pattern analysis yields clear structural indicators".to_string(),
        ),
        _ => (
            "Analyzing available data parameters. Current information sufficient for preliminary assessment but additional sensor data would improve analytical precision.".to_string(),
            "Recommend standard exploratory protocols with enhanced monitoring. Suggest gathering additional data before proceeding with major decisions.".to_string(),
            0.70,
            0.75,
            vec![
                "Extended observation period".to_string(),
                "Deploy additional sensor platforms".to_string(),
                "Consult Starfleet database for similar incidents".to_string(),
            ],
            vec![
                "Insufficient data for optimal decision-making".to_string(),
                "Unknown variables may affect outcome".to_string(),
            ],
            "Moderate - requires additional sensor input for comprehensive analysis".to_string(),
        ),
    };

    AnalyticalResponse {
        challenge_id: task.challenge_id.clone(),
        analyst: "Data".to_string(),
        analysis,
        recommendation,
        confidence,
        probability_success: probability,
        alternative_options: alternatives,
        risk_factors: risks,
        data_quality,
        timestamp: SystemTime::now(),
    }
}

/// Send analytical response to bridge
async fn send_analytical_response(
    agent: &EnterpriseAgent,
    response: &AnalyticalResponse,
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

    println!("📡 Analytical response sent to bridge");

    Ok(())
}

/// Send analytical log entry to Enterprise Log Agent
async fn send_analytical_log_entry(
    agent: &EnterpriseAgent,
    challenge: &CosmicChallenge,
    response: &AnalyticalResponse,
    _context: Context,
) -> Result<()> {
    let log_entry = LogEntry {
        log_type: "operations".to_string(),
        crew_member: "Lt. Commander Data".to_string(),
        rank: "Lieutenant Commander".to_string(),
        timestamp: SystemTime::now(),
        correlation_id: Uuid::now_v7(),
        task_id: Uuid::now_v7(),
        task_type: format!("Analytical Challenge: {}", challenge.challenge_type),
        description: challenge.description.clone(),
        outcome: response.recommendation.clone(),
        duration_seconds: 2000, // Data's characteristic analysis time
        confidence_level: response.confidence,
        stress_level: match challenge.threat_level {
            ThreatLevel::Minimal => 0.1,
            ThreatLevel::Low => 0.15,
            ThreatLevel::Moderate => 0.25,
            ThreatLevel::High => 0.35,
            ThreatLevel::Extreme => 0.45,
            ThreatLevel::CosmicScale => 0.55,
        },
        skills_demonstrated: vec!["data-analysis".to_string(), "computation".to_string(), "tactical-analysis".to_string(), "pattern-recognition".to_string()],
        challenges_faced: vec![format!("Analytical Challenge: {}", challenge.challenge_type)],
        collaborated_with: vec!["Bridge crew".to_string()],
        performance_rating: "Analytical".to_string(),
        notes: format!("Positronic analysis: {}. Probability of success: {:.1}%. Data quality: {}.", response.analysis, response.probability_success * 100.0, response.data_quality),
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

    println!("📋 Analytical log entry sent to Enterprise Log Agent");

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

    println!("{}", DataPersonality::startup());
    println!("{}", "Using StarTrek Enterprise agent patterns for positronic analysis...".bright_cyan().dimmed());

    // Create Enterprise Data Agent using StarTrek patterns
    let agent = EnterpriseAgent::builder("Lt. Commander Data")
        .with_capabilities(vec![
            "data-analysis".to_string(),
            "computation".to_string(),
            "tactical-analysis".to_string(),
            "pattern-recognition".to_string(),
            "probability-calculation".to_string(),
            "database-access".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
            "hybrid-message-handling".to_string(),
        ])
        .with_function("Operations Officer specializing in data analysis, computational processing, and positronic matrix capabilities")
        .with_location("Operations Station - Main Bridge")
        .with_service_type("analytical_operations")
        .build()
        .await?;

    println!("{}", "🤖 Lt. Commander Data online. Positronic matrix operational using StarTrek patterns.".bright_cyan().bold());
    println!("{}", "🔄 Intelligent reconnection monitoring enabled - operations will auto-recover from server failures.".bright_cyan().dimmed());

    // Create hybrid message handler
    let handler = DataMessageHandler::new(agent.clone());

    // Subscribe to analytical requests and challenges with hybrid handling
    println!("{}", "📡 Starting analytical operations message subscriptions...".bright_cyan().dimmed());

    agent.subscribe_with_hybrid_handling("enterprise.bridge.challenge", DataMessageHandler::new(agent.clone())).await?;
    agent.subscribe_with_hybrid_handling("enterprise.operations.task", handler).await?;

    println!("{}", "📋 Listening on: enterprise.bridge.challenge".bright_cyan());
    println!("{}", "📋 Listening on: enterprise.operations.task".bright_cyan());
    println!("{}", "✅ Operations station operational - supports both envelope and raw messages".bright_green().bold());
    println!("{}", "📊 Ready to provide positronic analysis and computational support to all challenges!".bright_cyan().bold());

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.map_err(|e| qollective::error::QollectiveError::validation(format!("Signal handling error: {}", e)))?;

    // Graceful shutdown
    println!("\\n{}", "🖖 Data signing off. All analytical subroutines disengaged.".bright_cyan().bold());
    agent.shutdown().await?;

    Ok(())
}
