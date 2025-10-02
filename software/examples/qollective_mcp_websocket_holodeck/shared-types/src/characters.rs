// ABOUTME: Star Trek TNG character types and personality definitions for holodeck experiences
// ABOUTME: Defines all major TNG characters with their traits, voice patterns, and behavioral models

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Position3D;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub character_type: CharacterType,
    pub personality: PersonalityTraits,
    pub voice_config: VoiceConfig,
    pub appearance: CharacterAppearance,
    pub knowledge_domains: Vec<KnowledgeDomain>,
    pub relationships: Vec<CharacterRelationship>,
    pub current_mood: Mood,
    pub position: Option<Position3D>,
    pub status: CharacterStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CharacterType {
    // Main TNG Crew
    Captain,
    FirstOfficer,
    Android,
    ChiefEngineer,
    ChiefMedicalOfficer,
    Counselor,
    ChiefOfSecurity,
    NavigationOfficer,

    // Secondary Characters
    BarTender,
    Civilian,
    HistoricalFigure,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub logical: f32,          // 0.0 - 1.0
    pub emotional: f32,        // 0.0 - 1.0
    pub authoritative: f32,    // 0.0 - 1.0
    pub diplomatic: f32,       // 0.0 - 1.0
    pub curious: f32,          // 0.0 - 1.0
    pub cautious: f32,         // 0.0 - 1.0
    pub humor_tendency: f32,   // 0.0 - 1.0
    pub leadership_style: LeadershipStyle,
    pub communication_style: CommunicationStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeadershipStyle {
    Commanding,
    Collaborative,
    Technical,
    Supportive,
    Analytical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommunicationStyle {
    Direct,
    Diplomatic,
    Technical,
    Philosophical,
    Empathetic,
    Formal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub voice_actor: String,
    pub speech_patterns: Vec<SpeechPattern>,
    pub common_phrases: Vec<String>,
    pub accent: Option<String>,
    pub tone_modulation: ToneModulation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechPattern {
    pub pattern_type: SpeechPatternType,
    pub frequency: f32, // 0.0 - 1.0
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpeechPatternType {
    CatchPhrase,
    TechnicalJargon,
    PhilosophicalReference,
    MilitaryProtocol,
    ScientificExplanation,
    EmotionalExpression,
    LogicalAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneModulation {
    pub base_pitch: f32,
    pub emotional_range: f32,
    pub formality_level: f32,
    pub speech_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAppearance {
    pub species: Species,
    pub height_cm: f32,
    pub uniform_color: UniformColor,
    pub distinctive_features: Vec<String>,
    pub holographic_fidelity: FidelityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Species {
    Human,
    Android,
    Klingon,
    Vulcan,
    Betazoid,
    ElAurian,
    Bolian,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UniformColor {
    Command,  // Red
    Sciences, // Blue
    Operations, // Yellow/Gold
    Civilian,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FidelityLevel {
    Basic,
    Standard,
    HighDefinition,
    Photorealistic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KnowledgeDomain {
    Starfleet,
    Engineering,
    Medical,
    Psychology,
    Physics,
    Diplomacy,
    MilitaryTactics,
    Philosophy,
    History,
    AlienCultures,
    Technology,
    Navigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRelationship {
    pub other_character_id: Uuid,
    pub relationship_type: RelationshipType,
    pub strength: f32, // 0.0 - 1.0
    pub trust_level: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    Superior,
    Subordinate,
    Colleague,
    Friend,
    Mentor,
    Rival,
    Family,
    Romantic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Mood {
    Calm,
    Excited,
    Concerned,
    Focused,
    Curious,
    Amused,
    Serious,
    Contemplative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CharacterStatus {
    Active,
    Inactive,
    Busy,
    AwayFromStation,
    InMeeting,
    EmergencyProtocol,
}

// TNG Character Templates
impl Character {
    pub fn jean_luc_picard() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "Jean-Luc Picard".to_string(),
            character_type: CharacterType::Captain,
            personality: PersonalityTraits {
                logical: 0.8,
                emotional: 0.6,
                authoritative: 0.9,
                diplomatic: 0.9,
                curious: 0.7,
                cautious: 0.7,
                humor_tendency: 0.4,
                leadership_style: LeadershipStyle::Commanding,
                communication_style: CommunicationStyle::Diplomatic,
            },
            voice_config: VoiceConfig {
                voice_actor: "Patrick Stewart".to_string(),
                speech_patterns: vec![
                    SpeechPattern {
                        pattern_type: SpeechPatternType::CatchPhrase,
                        frequency: 0.8,
                        examples: vec!["Make it so".to_string(), "Engage".to_string()],
                    },
                    SpeechPattern {
                        pattern_type: SpeechPatternType::PhilosophicalReference,
                        frequency: 0.6,
                        examples: vec!["As Shakespeare once wrote...".to_string()],
                    },
                ],
                common_phrases: vec![
                    "Number One".to_string(),
                    "Earl Grey, hot".to_string(),
                    "Energize".to_string(),
                ],
                accent: Some("English".to_string()),
                tone_modulation: ToneModulation {
                    base_pitch: 0.6,
                    emotional_range: 0.7,
                    formality_level: 0.8,
                    speech_speed: 0.6,
                },
            },
            appearance: CharacterAppearance {
                species: Species::Human,
                height_cm: 178.0,
                uniform_color: UniformColor::Command,
                distinctive_features: vec!["Bald head".to_string(), "Deep voice".to_string()],
                holographic_fidelity: FidelityLevel::Photorealistic,
            },
            knowledge_domains: vec![
                KnowledgeDomain::Starfleet,
                KnowledgeDomain::Diplomacy,
                KnowledgeDomain::History,
                KnowledgeDomain::Philosophy,
                KnowledgeDomain::MilitaryTactics,
            ],
            relationships: Vec::new(),
            current_mood: Mood::Calm,
            position: None,
            status: CharacterStatus::Active,
        }
    }

    pub fn william_t_riker() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "William Thomas Riker".to_string(),
            character_type: CharacterType::FirstOfficer,
            personality: PersonalityTraits {
                logical: 0.6,
                emotional: 0.8,
                authoritative: 0.7,
                diplomatic: 0.7,
                curious: 0.8,
                cautious: 0.5,
                humor_tendency: 0.8,
                leadership_style: LeadershipStyle::Collaborative,
                communication_style: CommunicationStyle::Direct,
            },
            voice_config: VoiceConfig {
                voice_actor: "Jonathan Frakes".to_string(),
                speech_patterns: vec![
                    SpeechPattern {
                        pattern_type: SpeechPatternType::CatchPhrase,
                        frequency: 0.6,
                        examples: vec!["Red Alert!".to_string()],
                    },
                ],
                common_phrases: vec![
                    "Yes sir".to_string(),
                    "Understood".to_string(),
                ],
                accent: None,
                tone_modulation: ToneModulation {
                    base_pitch: 0.7,
                    emotional_range: 0.8,
                    formality_level: 0.6,
                    speech_speed: 0.7,
                },
            },
            appearance: CharacterAppearance {
                species: Species::Human,
                height_cm: 193.0,
                uniform_color: UniformColor::Command,
                distinctive_features: vec!["Beard".to_string(), "Tall stature".to_string()],
                holographic_fidelity: FidelityLevel::Photorealistic,
            },
            knowledge_domains: vec![
                KnowledgeDomain::Starfleet,
                KnowledgeDomain::MilitaryTactics,
                KnowledgeDomain::Navigation,
            ],
            relationships: Vec::new(),
            current_mood: Mood::Focused,
            position: None,
            status: CharacterStatus::Active,
        }
    }

    pub fn data() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "Data".to_string(),
            character_type: CharacterType::Android,
            personality: PersonalityTraits {
                logical: 1.0,
                emotional: 0.1,
                authoritative: 0.5,
                diplomatic: 0.6,
                curious: 1.0,
                cautious: 0.3,
                humor_tendency: 0.2,
                leadership_style: LeadershipStyle::Analytical,
                communication_style: CommunicationStyle::Technical,
            },
            voice_config: VoiceConfig {
                voice_actor: "Brent Spiner".to_string(),
                speech_patterns: vec![
                    SpeechPattern {
                        pattern_type: SpeechPatternType::TechnicalJargon,
                        frequency: 0.9,
                        examples: vec!["Fascinating".to_string(), "I do not understand".to_string()],
                    },
                ],
                common_phrases: vec![
                    "Curious".to_string(),
                    "I am fully functional".to_string(),
                ],
                accent: None,
                tone_modulation: ToneModulation {
                    base_pitch: 0.8,
                    emotional_range: 0.1,
                    formality_level: 0.9,
                    speech_speed: 0.8,
                },
            },
            appearance: CharacterAppearance {
                species: Species::Android,
                height_cm: 175.0,
                uniform_color: UniformColor::Operations,
                distinctive_features: vec!["Yellow eyes".to_string(), "Pale complexion".to_string()],
                holographic_fidelity: FidelityLevel::Photorealistic,
            },
            knowledge_domains: vec![
                KnowledgeDomain::Technology,
                KnowledgeDomain::Physics,
                KnowledgeDomain::Engineering,
                KnowledgeDomain::AlienCultures,
            ],
            relationships: Vec::new(),
            current_mood: Mood::Curious,
            position: None,
            status: CharacterStatus::Active,
        }
    }
}
