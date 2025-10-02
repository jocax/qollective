// ABOUTME: Enterprise Log Agent - Centralized logging service using StarTrek agent patterns
// ABOUTME: Demonstrates modern agent development with HybridAgent pattern for envelope and raw message support

//! Enterprise Log Agent - Centralized Logging Service
//!
//! This agent receives log entries from all crew members and displays them
//! with rich qollective envelope context in real-time. Built using the
//! StarTrek Enterprise agent patterns for maximum reliability and reusability.

use std::time::SystemTime;
use uuid::Uuid;
use colored::Colorize;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use qollective::{
    constants::subjects,
    types::a2a::RegistryEvent,
    envelope::{Envelope, Context},
    error::Result,
};

// Import StarTrek Enterprise agent patterns
use qollective_a2a_nats_enterprise::startrek_agent::{
    EnterpriseAgent,
    HybridMessageHandler,
};

/// Log Agent personality and display patterns
struct LogAgentPersonality;

impl LogAgentPersonality {
    fn startup() -> String {
        "📋 Enterprise Log Agent online. Monitoring all crew communications.".bright_blue().bold().to_string()
    }
    
    fn log_entry_header() -> String {
        "═══════════════════════════════════════════════════════════════".bright_yellow().to_string()
    }
    
    fn log_entry_footer() -> String {
        "═══════════════════════════════════════════════════════════════".bright_yellow().dimmed().to_string()
    }
    
    fn envelope_context_header() -> String {
        "📨 QOLLECTIVE ENVELOPE CONTEXT:".bright_cyan().bold().to_string()
    }
    
    fn personal_log_header() -> String {
        "📝 PERSONAL LOG ENTRY:".bright_green().bold().to_string()
    }
    
    fn ships_log_header() -> String {
        "🚀 SHIP'S LOG ENTRY:".bright_blue().bold().to_string()
    }
}

/// Generic log entry that can handle different types of logs
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

/// Hybrid message handler for log agent supporting both envelope and raw messages
pub struct LogAgentMessageHandler;

#[async_trait]
impl HybridMessageHandler for LogAgentMessageHandler {
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
        // Try to parse as LogEntry first
        if let Ok(log_entry) = serde_json::from_value::<LogEntry>(envelope.payload.clone()) {
            return handle_log_entry(log_entry, context).await;
        }
        
        // Try to parse as RegistryEvent
        if let Ok(registry_event) = serde_json::from_value::<RegistryEvent>(envelope.payload.clone()) {
            return handle_registry_event(registry_event, context).await;
        }
        
        // Generic envelope message handling
        tracing::info!("📨 Processing generic Qollective envelope message");
        
        // Display envelope context
        println!("\n{}", LogAgentPersonality::log_entry_header());
        println!("{}", LogAgentPersonality::envelope_context_header());
        
        if let Some(request_id) = &context.meta().request_id {
            println!("   📋 Request ID: {}", request_id.to_string().bright_green());
        }
        if let Some(timestamp) = &context.meta().timestamp {
            println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
        }
        if let Some(tenant) = &context.meta().tenant {
            println!("   🏢 Tenant: {}", tenant.bright_cyan());
        }
        
        println!("   📄 Generic Data: {}", envelope.payload.to_string().chars().take(200).collect::<String>().bright_white());
        println!("{}", LogAgentPersonality::log_entry_footer());
        
        Ok(())
    }
    
    async fn handle_raw_message(&self, payload: Vec<u8>) -> Result<()> {
        tracing::info!("📦 Processing raw NATS message");
        
        println!("\n{}", LogAgentPersonality::log_entry_header());
        println!("📦 RAW NATS MESSAGE:");
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
        
        println!("{}", LogAgentPersonality::log_entry_footer());
        
        Ok(())
    }
}

/// Handle incoming log entry with qollective envelope context
async fn handle_log_entry(
    log_entry: LogEntry,
    context: Context,
) -> Result<()> {
    println!("\n{}", LogAgentPersonality::log_entry_header());
    
    // Display envelope context first
    println!("{}", LogAgentPersonality::envelope_context_header());
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }
    if let Some(tenant) = &context.meta().tenant {
        println!("   🏢 Tenant: {}", tenant.bright_cyan());
    }
    if let Some(version) = &context.meta().version {
        println!("   📦 Version: {}", version.bright_magenta());
    }
    
    // Display extension data (the rich context)
    if let Some(extensions) = context.extensions_ref() {
        println!("   🔧 Extensions: {} sections received", extensions.sections.len().to_string().bright_yellow());
        for (key, value) in &extensions.sections {
            println!("      └─ {}: {}", key.bright_white(), 
                    if key.contains("enterprise") || key.contains("crew") || key.contains("mission") {
                        format!("Enterprise data ({})", 
                               value.as_object().map(|o| o.len()).unwrap_or(0))
                    } else {
                        "Metadata".to_string()
                    }.bright_green());
            
            // Show some details for enterprise context
            if key == "enterprise_context" {
                if let Some(obj) = value.as_object() {
                    for (ek, ev) in obj {
                        println!("         🔹 {}: {}", ek.bright_cyan(), 
                               ev.as_str().unwrap_or("N/A").bright_white());
                    }
                }
            }
            
            // Show crew coordination details
            if key == "crew_coordination" {
                if let Some(obj) = value.as_object() {
                    for (ek, ev) in obj {
                        println!("         👥 {}: {}", ek.bright_cyan(), 
                               ev.as_str().unwrap_or("N/A").bright_white());
                    }
                }
            }
            
            // Show mission correlation
            if key == "mission_correlation" {
                if let Some(obj) = value.as_object() {
                    for (ek, ev) in obj {
                        println!("         🎯 {}: {}", ek.bright_cyan(), 
                               ev.as_str().unwrap_or("N/A").bright_white());
                    }
                }
            }
        }
    }
    
    println!();
    
    // Display the actual log entry
    match log_entry.log_type.as_str() {
        "personal" => println!("{}", LogAgentPersonality::personal_log_header()),
        "ships" => println!("{}", LogAgentPersonality::ships_log_header()),
        _ => println!("📄 {} LOG ENTRY:", log_entry.log_type.to_uppercase().bright_white().bold()),
    }
    
    println!("   👤 Crew Member: {} ({})", 
             log_entry.crew_member.bright_cyan().bold(), 
             log_entry.rank.bright_yellow());
    println!("   🆔 Correlation ID: {}", log_entry.correlation_id.to_string().bright_green());
    println!("   📋 Task: {} ({})", 
             log_entry.task_type.bright_magenta(), 
             log_entry.task_id.to_string().dimmed());
    println!("   📄 Description: {}", log_entry.description.bright_white());
    println!("   ✅ Outcome: {}", log_entry.outcome.bright_green());
    println!("   ⏱️  Duration: {}s", log_entry.duration_seconds.to_string().bright_blue());
    println!("   📊 Confidence: {:.1}%", log_entry.confidence_level * 100.0);
    println!("   😰 Stress Level: {:.1}%", log_entry.stress_level * 100.0);
    println!("   🏆 Performance: {}", log_entry.performance_rating.bright_yellow());
    
    if !log_entry.skills_demonstrated.is_empty() {
        println!("   🛠️  Skills: {}", log_entry.skills_demonstrated.join(", ").bright_cyan());
    }
    
    if !log_entry.challenges_faced.is_empty() {
        println!("   ⚠️  Challenges: {}", log_entry.challenges_faced.join(", ").bright_red());
    }
    
    if !log_entry.collaborated_with.is_empty() {
        println!("   🤝 Collaborated: {}", log_entry.collaborated_with.join(", ").bright_green());
    }
    
    if !log_entry.notes.is_empty() {
        println!("   💭 Notes: {}", log_entry.notes.bright_white());
    }
    
    println!("{}", LogAgentPersonality::log_entry_footer());
    
    Ok(())
}

/// Handle incoming registry events (agent registration/deregistration)
async fn handle_registry_event(
    event: RegistryEvent,
    context: Context,
) -> Result<()> {
    println!("\n{}", LogAgentPersonality::log_entry_header());
    
    // Display envelope context first
    println!("{}", LogAgentPersonality::envelope_context_header());
    if let Some(request_id) = &context.meta().request_id {
        println!("   📋 Request ID: {}", request_id.to_string().bright_green());
    }
    if let Some(timestamp) = &context.meta().timestamp {
        println!("   ⏰ Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_blue());
    }
    
    println!();
    
    // Display the registry event
    let header = match event.event_type.as_str() {
        "agent_registered" => "🟢 AGENT REGISTRATION EVENT:".bright_green().bold(),
        "agent_deregistered" => "🔴 AGENT DEREGISTRATION EVENT:".bright_red().bold(),
        _ => format!("📄 {} EVENT:", event.event_type.to_uppercase()).bright_white().bold(),
    };
    
    println!("{}", header);
    println!("   👤 Agent: {}", event.agent_name.bright_cyan().bold());
    println!("   🆔 Agent ID: {}", event.agent_id.to_string().bright_green());
    println!("   🛠️  Capabilities: {}", event.capabilities.join(", ").bright_yellow());
    
    let action = match event.event_type.as_str() {
        "agent_registered" => "joined the Enterprise registry",
        "agent_deregistered" => "left the Enterprise registry",
        _ => "performed an action in the registry",
    };
    
    println!("   📝 Action: {}", action.bright_white());
    
    println!("{}", LogAgentPersonality::log_entry_footer());
    
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
    
    println!("{}", LogAgentPersonality::startup());
    println!("{}", "Using StarTrek Enterprise agent patterns...".bright_blue().dimmed());
    
    // Create Enterprise Log Agent using StarTrek patterns
    let agent = EnterpriseAgent::builder("Enterprise Log Agent")
        .with_capabilities(vec![
            "logging".to_string(),
            "log-aggregation".to_string(),
            "real-time-display".to_string(),
            "context-analysis".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
            "hybrid-message-handling".to_string(),
        ])
        .with_function("Centralized logging service for all Enterprise crew activities with hybrid message support")
        .with_location("Computer Core - Logging Bay")
        .with_service_type("hybrid_logging")
        .build()
        .await?;
    
    println!("{}", "📋 Enterprise Log Agent online. Successfully registered using StarTrek patterns.".bright_blue().bold());
    println!("{}", "🔄 Intelligent reconnection monitoring enabled - agent will auto-recover from server failures.".bright_blue().dimmed());
    
    // Create hybrid message handler
    let handler = LogAgentMessageHandler;
    
    // Subscribe to log entries and registry events with hybrid handling
    println!("{}", "📡 Starting hybrid message subscriptions...".bright_blue().dimmed());
    
    agent.subscribe_with_hybrid_handling("enterprise.logging.entry", LogAgentMessageHandler).await?;
    agent.subscribe_with_hybrid_handling(subjects::AGENT_REGISTRY_EVENTS, handler).await?;
    
    println!("{}", "📋 Listening on: enterprise.logging.entry".bright_cyan());
    println!("{}", format!("📋 Listening on: {}", subjects::AGENT_REGISTRY_EVENTS).bright_cyan());
    println!("{}", "✅ Hybrid logging agent started - supports both envelope and raw messages".bright_green().bold());
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.map_err(|e| qollective::error::QollectiveError::validation(format!("Signal handling error: {}", e)))?;
    
    // Graceful shutdown
    println!("\n{}", "📋 Log Agent signing off. End of logging watch.".bright_blue().bold());
    agent.shutdown().await?;
    
    Ok(())
}