// ABOUTME: Core holodeck types and configurations for Star Trek holodeck experiences
// ABOUTME: Defines the main Holodeck struct and all related types for story management

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use schemars::JsonSchema;

use crate::characters::Character;
use crate::llm::LlmConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holodeck {
    pub id: Uuid,
    pub name: String,
    pub topic: String,
    pub story_type: HolodeckStoryType,
    pub participants: Vec<Character>,
    pub current_scene: Option<Scene>,
    pub configuration: HolodeckConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum HolodeckStoryType {
    Adventure,
    Mystery,
    Drama,
    Comedy,
    Historical,
    SciFi,
    Fantasy,
    Educational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub environment_id: Uuid,
    pub characters_present: Vec<Uuid>,
    pub props: Vec<SceneProp>,
    pub background_audio: Option<String>,
    pub lighting_config: LightingConfig,
    pub physics_settings: PhysicsSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneProp {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub position: Position3D,
    pub interactive: bool,
    pub physics_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingConfig {
    pub ambient_light: f32,
    pub directional_lights: Vec<DirectionalLight>,
    pub mood: LightingMood,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Position3D,
    pub intensity: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LightingMood {
    Bright,
    Dim,
    Dramatic,
    Cozy,
    Mysterious,
    Tense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSettings {
    pub gravity_enabled: bool,
    pub gravity_strength: f32,
    pub collision_detection: bool,
    pub real_time_physics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolodeckConfig {
    pub safety_level: SafetyLevel,
    pub max_participants: u32,
    pub duration_minutes: Option<u32>,
    pub auto_save_enabled: bool,
    pub voice_recognition: bool,
    pub haptic_feedback: bool,
    pub replicator_access: bool,
    pub transporter_integration: bool,
    pub environmental_controls: EnvironmentalControls,

    /// Configurable LLM provider settings for character AI and story generation
    pub llm_config: LlmConfig,
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum SafetyLevel {
    Training,    // No harm possible
    Standard,    // Standard safety protocols
    Reduced,     // Some risk allowed
    Disabled,    // All safeties off (rare)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum RiskLevel {
    None,        // No risk identified
    Low,         // Minimal risk
    Medium,      // Moderate risk
    High,        // Significant risk
    Critical,    // Severe risk requiring immediate attention
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum ContentType {
    Educational, // Educational content
    Historical,  // Historical recreations
    Adventure,   // Adventure scenarios
    Drama,       // Dramatic stories
    Comedy,      // Comedy programs
    SciFi,       // Science fiction
    Mystery,     // Mystery scenarios
    Fantasy,     // Fantasy worlds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalControls {
    pub temperature_celsius: f32,
    pub humidity_percent: f32,
    pub atmospheric_pressure: f32,
    pub oxygen_level: f32,
    pub wind_simulation: bool,
    pub weather_effects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolodeckSession {
    pub id: Uuid,
    pub holodeck_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub participants: Vec<Uuid>,
    pub session_log: Vec<SessionLogEntry>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: SessionEventType,
    pub participant_id: Option<Uuid>,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionEventType {
    SessionStart,
    SessionEnd,
    SceneChange,
    CharacterSpawn,
    CharacterDespawn,
    PropInteraction,
    DialogSpoken,
    SafetyProtocolTriggered,
    EnvironmentChange,
    UserCommand,
    SystemEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Aborted,
    SafetyHalt,
}

impl Holodeck {
    pub fn new(name: String, topic: String, story_type: HolodeckStoryType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name,
            topic,
            story_type,
            participants: Vec::new(),
            current_scene: None,
            configuration: HolodeckConfig::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_participant(&mut self, character: Character) {
        self.participants.push(character);
        self.updated_at = Utc::now();
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.current_scene = Some(scene);
        self.updated_at = Utc::now();
    }
}

impl Default for HolodeckConfig {
    fn default() -> Self {
        Self {
            safety_level: SafetyLevel::Standard,
            max_participants: 10,
            duration_minutes: None,
            auto_save_enabled: true,
            voice_recognition: true,
            haptic_feedback: true,
            replicator_access: false,
            transporter_integration: false,
            environmental_controls: EnvironmentalControls::default(),
            llm_config: LlmConfig::default(),
        }
    }
}

impl Default for EnvironmentalControls {
    fn default() -> Self {
        Self {
            temperature_celsius: 22.0,
            humidity_percent: 45.0,
            atmospheric_pressure: 101.325,
            oxygen_level: 21.0,
            wind_simulation: false,
            weather_effects: false,
        }
    }
}
