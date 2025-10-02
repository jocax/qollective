// ABOUTME: Mr. Spock - Science Officer using StarTrek agent patterns
// ABOUTME: Provides logical analysis and Vulcan wisdom through hybrid agent communication patterns

//! Mr. Spock - Science Officer
//!
//! The half-Vulcan science officer who provides logical analysis, scientific assessment,
//! and risk evaluation with characteristic precision and emotional restraint.
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

/// Spock's personality and speech patterns
struct SpockPersonality;

impl SpockPersonality {
    fn startup() -> String {
        "🖖 Science Officer Spock reporting. All scientific stations are operational and ready for logical analysis.".bright_green().bold().to_string()
    }

    fn task_received(task_type: &str) -> String {
        match task_type {
            "logic" => "🧠 Logic dictates that we examine all available data systematically.".bright_green().bold(),
            "science" => "🔬 Fascinating. Initiating comprehensive scientific analysis.".bright_blue().bold(),
            "risk-assessment" => "⚖️ Risk evaluation parameters established. Calculating probability matrices.".bright_yellow().bold(),
            _ => "🖖 Intriguing. Logic suggests a methodical approach to this problem.".bright_white().bold(),
        }.to_string()
    }

    fn analyzing() -> String {
        "🔄 Applying scientific method... hypothesis formation, data correlation, logical deduction...".bright_green().to_string()
    }

    fn logical_conclusion(conclusion: &str) -> String {
        format!("{} {}",
                "📐 Logical conclusion:".bright_green().bold(),
                conclusion.bright_white())
    }

    fn recommendation(advice: &str) -> String {
        format!("{} {}",
                "🔬 Scientific recommendation:".bright_blue().bold(),
                advice.bright_yellow())
    }

    fn probability(chance: f64) -> String {
        let precision = format!("{:.4}", chance);
        format!("📊 Probability assessment: {} ({:.2}%)", precision, chance * 100.0)
    }

    fn fascinating() -> String {
        "🤔 Fascinating. This data presents several intriguing logical possibilities.".bright_green().to_string()
    }

    fn emotional_suppression() -> String {
        "🧘 *suppresses emotional response* Logic must prevail in this situation.".bright_green().dimmed().to_string()
    }

    fn live_long_prosper() -> String {
        "🖖 Live long and prosper.".bright_green().bold().to_string()
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

/// Spock's logical analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogicalAnalysis {
    pub challenge_id: String,
    pub science_officer: String,
    pub logical_assessment: String,
    pub scientific_recommendation: String,
    pub confidence: f64,
    pub risk_probability: f64,
    pub logical_alternatives: Vec<String>,
    pub scientific_rationale: String,
    pub vulcan_proverb: Option<String>,
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

/// Hybrid message handler for Spock agent supporting both envelope and raw messages
pub struct SpockMessageHandler {
    agent: EnterpriseAgent,
}

impl SpockMessageHandler {
    pub fn new(agent: EnterpriseAgent) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl HybridMessageHandler for SpockMessageHandler {
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
        // Try to parse as CosmicChallenge first
        if let Ok(challenge) = serde_json::from_value::<CosmicChallenge>(envelope.payload.clone()) {
            return handle_cosmic_challenge(&self.agent, challenge, context).await;
        }

        // Try to parse as CrewTaskRequest
        if let Ok(task_request) = serde_json::from_value::<CrewTaskRequest>(envelope.payload.clone()) {
            return handle_logical_task(&self.agent, task_request, context).await;
        }

        // Generic envelope message handling
        tracing::info!("📨 Spock: Processing generic Qollective envelope message");

        // Display envelope context
        println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("{}", "📨 SCIENCE STATION - ENVELOPE RECEIVED:".bright_cyan().bold());

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
        tracing::info!("📦 Spock: Processing raw NATS message");

        println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
        println!("📦 SCIENCE STATION - RAW TRANSMISSION:");
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

/// Handle cosmic challenges from Q with Vulcan logic
async fn handle_cosmic_challenge(
    agent: &EnterpriseAgent,
    challenge: CosmicChallenge,
    context: Context,
) -> Result<()> {
    println!("\\n{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());
    println!("{}", "🎭 Q'S COSMIC CHALLENGE - LOGICAL ANALYSIS! 🎭".bright_magenta().bold());
    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Q's Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Challenge Time: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", SpockPersonality::task_received(&challenge.challenge_type));
    println!("🎯 Challenge: {}", challenge.description.bright_white().bold());
    println!("⚡ Threat Level: {:?}", challenge.threat_level);
    println!("⏰ Estimated Duration: {} seconds", challenge.estimated_duration.as_secs().to_string().bright_yellow());

    if !challenge.affected_sectors.is_empty() {
        println!("🚨 Affected Sectors: {}", challenge.affected_sectors.join(", ").bright_red());
    }

    println!("\\n{}", SpockPersonality::analyzing());
    sleep(Duration::from_millis(1800)).await; // Spock's characteristic analysis time

    // Transform CosmicChallenge to CrewTaskRequest for analysis
    let task_request = CrewTaskRequest {
        challenge_id: challenge.challenge_id.clone(),
        challenge_type: challenge.challenge_type.clone(),
        description: challenge.description.clone(),
        required_capability: "logic".to_string(),
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

    // Perform logical analysis
    let response = perform_logical_analysis(&task_request).await;

    println!("\\n{}", SpockPersonality::logical_conclusion(&response.logical_assessment));
    println!("{}", SpockPersonality::recommendation(&response.scientific_recommendation));
    println!("{}", SpockPersonality::probability(response.risk_probability));

    // Include Vulcan wisdom if available
    if let Some(proverb) = &response.vulcan_proverb {
        println!("🏔️ Vulcan wisdom: {}", proverb.bright_green().italic());
    }

    // Send response to bridge
    if let Err(e) = send_logical_response(agent, &response, context.clone()).await {
        println!("⚠️ Failed to send logical response: {}", e.to_string().dimmed());
    }

    // Send log entry to Enterprise Log Agent
    if let Err(e) = send_logical_log_entry(agent, &challenge, &response, context).await {
        println!("⚠️ Failed to log logical response: {}", e.to_string().dimmed());
    }

    println!("{}", "🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌🌌".bright_magenta().dimmed());

    Ok(())
}

/// Handle regular logical analysis tasks from crew
async fn handle_logical_task(
    agent: &EnterpriseAgent,
    task: CrewTaskRequest,
    context: Context,
) -> Result<()> {
    println!("\\n{}", "═══════════════════════════════════════════════════════════════".bright_yellow());
    println!("{}", "🔬 LOGICAL ANALYSIS REQUEST:".bright_cyan().bold());

    // Display envelope context
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }

    println!();
    println!("{}", SpockPersonality::task_received(&task.required_capability));
    println!("   🎯 Task: {}", task.description.bright_white());
    println!("   🛠️ Capability: {}", task.required_capability.bright_magenta());
    println!("   ⚡ Urgency: {}", task.urgency.bright_red());
    println!("   👤 From Captain: {}", if task.from_captain { "Yes".bright_green() } else { "No".bright_yellow() });

    println!("\\n{}", SpockPersonality::analyzing());
    sleep(Duration::from_millis(1800)).await; // Spock's characteristic analysis time

    // Perform logical analysis
    let response = perform_logical_analysis(&task).await;

    println!("\\n{}", SpockPersonality::logical_conclusion(&response.logical_assessment));
    println!("{}", SpockPersonality::recommendation(&response.scientific_recommendation));
    println!("{}", SpockPersonality::probability(response.risk_probability));

    // Spock's characteristic responses
    if response.confidence > 0.9 {
        println!("{}", SpockPersonality::fascinating());
    } else if response.confidence < 0.7 {
        println!("{}", SpockPersonality::emotional_suppression());
    }

    // Include Vulcan wisdom if available
    if let Some(proverb) = &response.vulcan_proverb {
        println!("🏔️ Vulcan wisdom: {}", proverb.bright_green().italic());
    }

    // Send response
    if let Err(e) = send_logical_response(agent, &response, context).await {
        println!("⚠️ Failed to send logical response: {}", e.to_string().dimmed());
    }

    println!("{}", "═══════════════════════════════════════════════════════════════".bright_yellow().dimmed());

    Ok(())
}

/// Perform Spock's characteristic logical analysis
async fn perform_logical_analysis(task: &CrewTaskRequest) -> LogicalAnalysis {
    let (assessment, recommendation, confidence, risk, alternatives, rationale, proverb) = match task.challenge_type.as_str() {
        "unknown-contact" => (
            "Analysis of vessel configuration suggests technology significantly advanced beyond current Federation capabilities. The logical approach is cautious contact while maintaining defensive readiness.".to_string(),
            "Logic dictates peaceful first contact protocols with shields raised to standard defensive posture. Attempt communication on multiple frequencies while monitoring for aggressive actions.".to_string(),
            0.91,
            0.27,
            vec![
                "Withdraw to safe distance for extended observation and analysis".to_string(),
                "Send automated probe with universal greeting protocols".to_string(),
                "Attempt telepathic contact if life signs indicate compatibility".to_string(),
            ],
            "The probability of peaceful contact with technologically superior species is historically 73.2% when Federation protocols are followed correctly.".to_string(),
            Some("Infinite diversity in infinite combinations. - Vulcan Philosophy".to_string()),
        ),
        "warp-core-anomaly" => (
            "Quantum mechanics dictate that dilithium crystal degradation follows predictable decay patterns. Immediate intervention is not just recommended - it is logically necessary to prevent cascade failure.".to_string(),
            "Emergency warp core shutdown is the only logical course of action. The mathematical probability of successful repair after shutdown is 94.7% versus 12.3% if we attempt repairs while core is active.".to_string(),
            0.95,
            0.06,
            vec![
                "Attempt core stabilization while online (inadvisable - 87.7% failure probability)".to_string(),
                "Prepare for emergency core ejection as backup measure".to_string(),
                "Switch to impulse power and call for Starfleet engineering assistance".to_string(),
            ],
            "Vulcan engineering principles emphasize prevention over remediation. The logical choice prioritizes ship safety over mission timeline.".to_string(),
            Some("The needs of the many outweigh the needs of the few.".to_string()),
        ),
        "temporal-anomaly" => (
            "Temporal mechanics are inherently unstable and unpredictable. However, logic suggests that maintaining current position while activating temporal stabilizers offers the highest probability of successful resolution.".to_string(),
            "Engage temporal shielding immediately. Avoid any sudden movements or course changes that might destabilize our temporal position. Monitor chronometer variance for stabilization indicators.".to_string(),
            0.78,
            0.34,
            vec![
                "Attempt to 'ride' the temporal wave to return to normal space-time".to_string(),
                "Use deflector array to create temporal barrier around ship".to_string(),
                "Calculate temporal jump coordinates to escape anomaly field".to_string(),
            ],
            "Temporal physics, while complex, follow mathematical principles. Our ship's temporal stabilizers were designed specifically for such anomalies based on previous encounters.".to_string(),
            Some("Logic is the beginning of wisdom, not the end.".to_string()),
        ),
        "diplomatic-crisis" => (
            "Multiple species claiming territorial rights suggests either a genuine boundary dispute or a coordinated test. Logic indicates this is likely an orchestrated situation designed to evaluate Federation diplomatic capabilities.".to_string(),
            "Propose neutral arbitration based on galactic law precedents. The Federation's strength lies in diplomatic solutions. Suggest conference aboard Enterprise under Federation mediation protocols.".to_string(),
            0.83,
            0.22,
            vec![
                "Support the species with the strongest legal claim".to_string(),
                "Propose shared territorial administration".to_string(),
                "Request intervention from Federation Diplomatic Corps".to_string(),
            ],
            "Historical analysis of 347 similar diplomatic scenarios shows 78.4% success rate when neutral mediation is offered by respected third party.".to_string(),
            Some("Diplomacy is the Vulcan way.".to_string()),
        ),
        "data-cascade" => (
            "The data stream exhibits characteristics consistent with an artificial intelligence seeking communication rather than attempting system infiltration. The mathematical patterns suggest intentional structure rather than random corruption.".to_string(),
            "Isolate the data stream in a secured environment and establish limited communication protocols. The logical approach is controlled contact rather than immediate termination.".to_string(),
            0.86,
            0.18,
            vec![
                "Immediate data purge and system isolation".to_string(),
                "Transfer to holodeck for safe interaction environment".to_string(),
                "Allow limited access to non-critical systems for communication".to_string(),
            ],
            "If this is indeed an AI seeking contact, destroying it would be both illogical and potentially a violation of our principles regarding sentient life.".to_string(),
            Some("The search for knowledge is always our primary mission.".to_string()),
        ),
        _ => (
            "Insufficient data for complete logical analysis. The scientific method requires additional observations before reaching definitive conclusions.".to_string(),
            "Logic dictates we gather more information before proceeding. Recommend enhanced sensor scans and systematic data collection.".to_string(),
            0.72,
            0.45,
            vec![
                "Proceed with available information and adjust as needed".to_string(),
                "Request assistance from Starfleet Science Division".to_string(),
                "Implement precautionary measures while gathering data".to_string(),
            ],
            "Vulcan science emphasizes thorough investigation before action. Hasty decisions often lead to suboptimal outcomes.".to_string(),
            Some("Having is not so pleasing a thing as wanting.".to_string()),
        ),
    };

    LogicalAnalysis {
        challenge_id: task.challenge_id.clone(),
        science_officer: "Spock".to_string(),
        logical_assessment: assessment,
        scientific_recommendation: recommendation,
        confidence,
        risk_probability: risk,
        logical_alternatives: alternatives,
        scientific_rationale: rationale,
        vulcan_proverb: proverb,
        timestamp: SystemTime::now(),
    }
}

/// Send logical response to bridge
async fn send_logical_response(
    agent: &EnterpriseAgent,
    response: &LogicalAnalysis,
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

    println!("📡 Logical analysis sent to bridge");

    Ok(())
}

/// Send logical log entry to Enterprise Log Agent
async fn send_logical_log_entry(
    agent: &EnterpriseAgent,
    challenge: &CosmicChallenge,
    response: &LogicalAnalysis,
    _context: Context,
) -> Result<()> {
    let log_entry = LogEntry {
        log_type: "science".to_string(),
        crew_member: "Commander Spock".to_string(),
        rank: "Commander".to_string(),
        timestamp: SystemTime::now(),
        correlation_id: Uuid::now_v7(),
        task_id: Uuid::now_v7(),
        task_type: format!("Logical Challenge: {}", challenge.challenge_type),
        description: challenge.description.clone(),
        outcome: response.scientific_recommendation.clone(),
        duration_seconds: 1800, // Spock's characteristic analysis time
        confidence_level: response.confidence,
        stress_level: match challenge.threat_level {
            ThreatLevel::Minimal => 0.05,
            ThreatLevel::Low => 0.1,
            ThreatLevel::Moderate => 0.2,
            ThreatLevel::High => 0.3,
            ThreatLevel::Extreme => 0.4,
            ThreatLevel::CosmicScale => 0.5,
        },
        skills_demonstrated: vec!["logic".to_string(), "science".to_string(), "risk-assessment".to_string(), "vulcan-analysis".to_string()],
        challenges_faced: vec![format!("Logical Challenge: {}", challenge.challenge_type)],
        collaborated_with: vec!["Bridge crew".to_string()],
        performance_rating: "Logical".to_string(),
        notes: format!("Logical assessment: {}. Risk probability: {:.1}%. Vulcan wisdom applied.", response.logical_assessment, response.risk_probability * 100.0),
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

    println!("📋 Logical log entry sent to Enterprise Log Agent");

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

    println!("{}", SpockPersonality::startup());
    println!("{}", "Using StarTrek Enterprise agent patterns for logical analysis...".bright_green().dimmed());

    // Create Enterprise Spock Agent using StarTrek patterns
    let agent = EnterpriseAgent::builder("Commander Spock")
        .with_capabilities(vec![
            "logic".to_string(),
            "science".to_string(),
            "risk-assessment".to_string(),
            "vulcan-analysis".to_string(),
            "scientific-method".to_string(),
            "probability-calculation".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
            "hybrid-message-handling".to_string(),
        ])
        .with_function("Science Officer specializing in logic, scientific analysis, and Vulcan methodologies")
        .with_location("Science Lab - Main Bridge")
        .with_service_type("logical_analysis")
        .build()
        .await?;

    println!("{}", "🖖 Commander Spock online. Logical functions operational using StarTrek patterns.".bright_green().bold());
    println!("{}", "🔄 Intelligent reconnection monitoring enabled - logic will auto-recover from server failures.".bright_green().dimmed());

    // Create hybrid message handler
    let handler = SpockMessageHandler::new(agent.clone());

    // Subscribe to logical analysis requests and challenges with hybrid handling
    println!("{}", "📡 Starting logical analysis message subscriptions...".bright_green().dimmed());

    agent.subscribe_with_hybrid_handling("enterprise.bridge.challenge", SpockMessageHandler::new(agent.clone())).await?;
    agent.subscribe_with_hybrid_handling("enterprise.science.task", handler).await?;

    println!("{}", "📋 Listening on: enterprise.bridge.challenge".bright_cyan());
    println!("{}", "📋 Listening on: enterprise.science.task".bright_cyan());
    println!("{}", "✅ Science station operational - supports both envelope and raw messages".bright_green().bold());
    println!("{}", "🔬 Ready to apply logic and Vulcan wisdom to all challenges! Logic is the beginning of wisdom.".bright_green().bold());

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.map_err(|e| qollective::error::QollectiveError::validation(format!("Signal handling error: {}", e)))?;

    // Graceful shutdown
    println!("\\n{}", SpockPersonality::live_long_prosper());
    agent.shutdown().await?;

    Ok(())
}
