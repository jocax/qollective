// ABOUTME: Star Trek specific data structures for rich envelope content
// ABOUTME: Demonstrates how to use qollective envelopes with complex agent coordination data

//! Star Trek Agent Data Structures
//!
//! This module defines rich data structures that showcase how qollective envelopes
//! can transport sophisticated context between agents in a Star Trek scenario.
//! These structures will be packed into qollective Meta extensions.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};
use uuid::Uuid;

/// Threat level assessment for cosmic challenges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Minimal,
    Low,
    Moderate,
    High,
    Extreme,
    CosmicScale,
}

/// Ship condition status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShipCondition {
    Green,      // All normal
    Yellow,     // Heightened alert
    Red,        // General quarters
    Blue,       // Docked at starbase
    Gray,       // Stealth/silent running
}

/// Mission phase tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissionPhase {
    Exploration,
    Diplomacy,
    Scientific,
    Emergency,
    Combat,
    Rescue,
    Investigation,
}

/// Agent collaboration preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaborationStyle {
    Independent,    // Prefers working alone
    Consultative,   // Asks for input but decides alone
    Collaborative,  // Equal partnership
    Delegative,     // Assigns tasks to others
    Supervisory,    // Oversees team efforts
}

/// Q's test parameters - what Q is really testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QTestParameters {
    /// What aspect of humanity/crew is being tested
    pub test_focus: String,
    /// Q's expected crew response patterns
    pub expected_behaviors: Vec<String>,
    /// Q's amusement level with the proceedings
    pub amusement_level: f64,
    /// Whether this is part of a larger test series
    pub is_series_test: bool,
    /// Previous test results influence
    pub prior_test_influence: Option<String>,
}

/// Coordinates in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub sector: String,
}

impl Coordinates {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let sector = format!("Sector {}-{}-{}",
                           (x / 1000.0) as i32,
                           (y / 1000.0) as i32,
                           (z / 1000.0) as i32);
        Self { x, y, z, sector }
    }
}

/// Available ship resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// Power availability (0.0 to 1.0)
    pub power_level: f64,
    /// Available personnel count
    pub personnel_available: u32,
    /// Replicator capacity
    pub replicator_capacity: f64,
    /// Shuttle/auxiliary craft available
    pub auxiliary_craft: u32,
    /// Torpedo count
    pub torpedo_count: u32,
    /// Medical supplies level
    pub medical_supplies: f64,
    /// Emergency resources
    pub emergency_reserves: f64,
}

impl Default for ResourceManifest {
    fn default() -> Self {
        Self {
            power_level: 0.95,
            personnel_available: 1012,
            replicator_capacity: 0.87,
            auxiliary_craft: 3,
            torpedo_count: 24,
            medical_supplies: 0.92,
            emergency_reserves: 0.78,
        }
    }
}

/// Current diplomatic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticContext {
    /// Known species in vicinity and relationship status
    pub species_relations: HashMap<String, DiplomaticStatus>,
    /// Current treaty obligations
    pub active_treaties: Vec<String>,
    /// Diplomatic immunity status
    pub diplomatic_immunity: bool,
    /// First contact protocols active
    pub first_contact_protocols: bool,
}

/// Relationship status with species
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiplomaticStatus {
    Ally,
    Friendly,
    Neutral,
    Tense,
    Hostile,
    Unknown,
}

impl Default for DiplomaticContext {
    fn default() -> Self {
        let mut species_relations = HashMap::new();
        species_relations.insert("Klingon".to_string(), DiplomaticStatus::Ally);
        species_relations.insert("Romulan".to_string(), DiplomaticStatus::Tense);
        species_relations.insert("Cardassian".to_string(), DiplomaticStatus::Neutral);
        species_relations.insert("Ferengi".to_string(), DiplomaticStatus::Friendly);

        Self {
            species_relations,
            active_treaties: vec!["Khitomer Accords".to_string()],
            diplomatic_immunity: true,
            first_contact_protocols: false,
        }
    }
}

/// Enhanced cosmic challenge with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmicChallenge {
    /// Basic challenge identification
    pub challenge_id: String,
    pub challenge_type: String,
    pub description: String,
    pub urgency: String,
    pub requires_capabilities: Vec<String>,
    pub timestamp: SystemTime,

    /// Enhanced context
    pub threat_level: ThreatLevel,
    pub affected_sectors: Vec<String>,
    pub estimated_duration: Duration,
    pub resource_requirements: ResourceManifest,
    pub previous_encounters: Vec<String>,
    pub similar_challenges: Vec<String>,
    pub q_test_parameters: Option<QTestParameters>,
}

/// Current Enterprise ship status and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseContext {
    /// Ship location and status
    pub current_location: Coordinates,
    pub ship_condition: ShipCondition,
    pub heading: f64,
    pub warp_factor: f64,

    /// Crew and resources
    pub crew_morale: f64,
    pub available_resources: ResourceManifest,
    pub current_mission_phase: MissionPhase,
    pub diplomatic_status: DiplomaticContext,

    /// Environmental factors
    pub local_hazards: Vec<String>,
    pub sensor_contacts: Vec<String>,
    pub subspace_conditions: f64,

    /// Mission context
    pub mission_priority: u8, // 1-10 scale
    pub mission_time_remaining: Option<Duration>,
    pub backup_plans_available: u8,
}

impl Default for EnterpriseContext {
    fn default() -> Self {
        Self {
            current_location: Coordinates::new(15234.7, 23891.2, 8734.1),
            ship_condition: ShipCondition::Green,
            heading: 127.5,
            warp_factor: 0.0, // At impulse
            crew_morale: 0.87,
            available_resources: ResourceManifest::default(),
            current_mission_phase: MissionPhase::Exploration,
            diplomatic_status: DiplomaticContext::default(),
            local_hazards: vec![],
            sensor_contacts: vec!["Science probe Echo-7".to_string()],
            subspace_conditions: 0.94,
            mission_priority: 5,
            mission_time_remaining: Some(Duration::from_secs(86400 * 7)), // 7 days
            backup_plans_available: 3,
        }
    }
}

/// Performance metrics for agent evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Success rate for tasks (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Confidence accuracy (how often confidence matches actual success)
    pub confidence_accuracy: f64,
    /// Innovation factor (novel solutions provided)
    pub innovation_factor: f64,
    /// Collaboration effectiveness
    pub collaboration_score: f64,
    /// Recent performance trend (improving/declining)
    pub performance_trend: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            success_rate: 0.85,
            avg_response_time: Duration::from_secs(45),
            confidence_accuracy: 0.78,
            innovation_factor: 0.65,
            collaboration_score: 0.82,
            performance_trend: 0.05, // Slightly improving
        }
    }
}

/// Trust matrix between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustMatrix {
    /// Trust levels with other agents (0.0 to 1.0)
    pub agent_trust: HashMap<String, f64>,
    /// Reliability assessments
    pub reliability_scores: HashMap<String, f64>,
    /// Collaboration preferences
    pub preferred_partners: Vec<String>,
    /// Agents to avoid for certain task types
    pub task_specific_preferences: HashMap<String, Vec<String>>,
}

impl Default for TrustMatrix {
    fn default() -> Self {
        let mut agent_trust = HashMap::new();
        agent_trust.insert("Captain Jean-Luc Picard".to_string(), 0.95);
        agent_trust.insert("Montgomery Scott".to_string(), 0.88);
        agent_trust.insert("Data".to_string(), 0.92);
        agent_trust.insert("Spock".to_string(), 0.89);

        let mut reliability_scores = HashMap::new();
        reliability_scores.insert("Captain Jean-Luc Picard".to_string(), 0.97);
        reliability_scores.insert("Montgomery Scott".to_string(), 0.94);
        reliability_scores.insert("Data".to_string(), 0.99);
        reliability_scores.insert("Spock".to_string(), 0.96);

        Self {
            agent_trust,
            reliability_scores,
            preferred_partners: vec!["Data".to_string(), "Spock".to_string()],
            task_specific_preferences: HashMap::new(),
        }
    }
}

/// Individual interaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionHistory {
    pub interaction_id: Uuid,
    pub other_agent: String,
    pub task_type: String,
    pub success: bool,
    pub confidence_provided: f64,
    pub actual_outcome_quality: f64,
    pub response_time: Duration,
    pub timestamp: SystemTime,
    pub notes: Option<String>,
}

/// Agent context including relationships and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Who requested this task
    pub requester_id: String,
    pub requester_rank: Option<String>,

    /// Interaction history
    pub previous_interactions: Vec<InteractionHistory>,
    pub total_interactions_count: u32,

    /// Trust and performance tracking
    pub trust_metrics: TrustMatrix,
    pub performance_ratings: HashMap<String, PerformanceMetrics>,

    /// Collaboration preferences
    pub collaboration_style: CollaborationStyle,
    pub preferred_communication_style: String,

    /// Current agent state
    pub current_workload: f64, // 0.0 to 1.0
    pub stress_level: f64,     // 0.0 to 1.0
    pub expertise_confidence: HashMap<String, f64>, // Confidence in different skill areas
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            requester_id: "unknown".to_string(),
            requester_rank: None,
            previous_interactions: Vec::new(),
            total_interactions_count: 0,
            trust_metrics: TrustMatrix::default(),
            performance_ratings: HashMap::new(),
            collaboration_style: CollaborationStyle::Collaborative,
            preferred_communication_style: "Direct and professional".to_string(),
            current_workload: 0.25,
            stress_level: 0.15,
            expertise_confidence: HashMap::new(),
        }
    }
}

/// Decision point in the request flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub agent_name: String,
    pub decision_type: String,
    pub options_considered: Vec<String>,
    pub chosen_option: String,
    pub reasoning: String,
    pub confidence: f64,
    pub timestamp: SystemTime,
    pub factors_considered: Vec<String>,
}

/// Agent interaction in the request chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInteraction {
    pub agent_name: String,
    pub interaction_type: String, // "request", "response", "consultation", "delegation"
    pub message_size_bytes: u32,
    pub processing_time: Duration,
    pub timestamp: SystemTime,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Timing metrics for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    pub total_request_duration: Duration,
    pub agent_processing_times: HashMap<String, Duration>,
    pub network_latencies: HashMap<String, Duration>,
    pub bottleneck_analysis: Vec<String>,
    pub efficiency_score: f64,
}

/// Complete request tracing through the agent network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTrace {
    /// Correlation tracking
    pub correlation_id: Uuid,
    pub parent_request_id: Option<Uuid>,
    pub child_request_ids: Vec<Uuid>,

    /// Request flow
    pub request_chain: Vec<AgentInteraction>,
    pub decision_path: Vec<DecisionPoint>,

    /// Performance analysis
    pub timing_data: TimingMetrics,
    pub aggregated_confidence: f64,
    pub final_outcome_quality: Option<f64>,

    /// Context preservation
    pub context_evolution: Vec<String>, // How context changed through the flow
    pub resource_utilization: HashMap<String, f64>,

    /// Learning opportunities
    pub improvement_suggestions: Vec<String>,
    pub lessons_learned: Vec<String>,
}

impl RequestTrace {
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::now_v7(),
            parent_request_id: None,
            child_request_ids: Vec::new(),
            request_chain: Vec::new(),
            decision_path: Vec::new(),
            timing_data: TimingMetrics {
                total_request_duration: Duration::from_secs(0),
                agent_processing_times: HashMap::new(),
                network_latencies: HashMap::new(),
                bottleneck_analysis: Vec::new(),
                efficiency_score: 0.0,
            },
            aggregated_confidence: 0.0,
            final_outcome_quality: None,
            context_evolution: Vec::new(),
            resource_utilization: HashMap::new(),
            improvement_suggestions: Vec::new(),
            lessons_learned: Vec::new(),
        }
    }

    pub fn add_interaction(&mut self, interaction: AgentInteraction) {
        self.request_chain.push(interaction);
    }

    pub fn add_decision(&mut self, decision: DecisionPoint) {
        self.decision_path.push(decision);
    }

    pub fn calculate_efficiency(&mut self) {
        let total_time = self.timing_data.total_request_duration.as_secs_f64();
        let processing_time: f64 = self.timing_data.agent_processing_times
            .values()
            .map(|d| d.as_secs_f64())
            .sum();

        if total_time > 0.0 {
            self.timing_data.efficiency_score = processing_time / total_time;
        }
    }
}

impl Default for RequestTrace {
    fn default() -> Self {
        Self::new()
    }
}
