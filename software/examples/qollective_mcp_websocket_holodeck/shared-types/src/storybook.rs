// ABOUTME: Storybook and history management types for tracking holodeck session progress
// ABOUTME: Manages played scenes, player decisions, session state, and story progression data

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::{GraphNode, DecisionOption, Consequence};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryBook {
    pub id: Uuid,
    pub template_id: Uuid,
    pub name: String,
    pub played_scenes: Vec<PlayedScene>,
    pub current_position: GraphNode,
    pub player_decisions: Vec<PlayerDecision>,
    pub story_flags: HashMap<String, serde_json::Value>,
    pub character_states: HashMap<Uuid, CharacterState>,
    pub session_metrics: SessionMetrics,
    pub created_at: DateTime<Utc>,
    pub last_saved: DateTime<Utc>,
    pub status: StoryBookStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoryBookStatus {
    InProgress,
    Completed,
    Paused,
    Abandoned,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayedScene {
    pub id: Uuid,
    pub scene_template_id: Uuid,
    pub scene_instance_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub participants: Vec<Uuid>,
    pub objectives_completed: Vec<ObjectiveCompletion>,
    pub interactions_performed: Vec<InteractionRecord>,
    pub dialogue_exchanges: Vec<DialogueExchange>,
    pub environmental_events: Vec<EnvironmentalEvent>,
    pub scene_outcome: Option<SceneOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveCompletion {
    pub objective_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub completion_method: CompletionMethod,
    pub success_rating: f32, // 0.0 - 1.0
    pub time_taken_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompletionMethod {
    PlayerAction,
    TeamWork,
    SkillCheck,
    ItemUsage,
    DialogueChoice,
    Environmental,
    TimeElapsed,
    Alternative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub timestamp: DateTime<Utc>,
    pub element_id: Uuid,
    pub interaction_type: String,
    pub participant_id: Uuid,
    pub result: InteractionOutcome,
    pub skill_check_result: Option<SkillCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOutcome {
    pub success: bool,
    pub description: String,
    pub consequences_triggered: Vec<Consequence>,
    pub story_flags_set: Vec<String>,
    pub items_gained: Vec<String>,
    pub relationships_affected: Vec<RelationshipChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCheckResult {
    pub skill_type: String,
    pub required_level: u8,
    pub rolled_value: u8,
    pub modifiers: Vec<SkillModifier>,
    pub final_result: u8,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillModifier {
    pub source: String,
    pub modifier_value: i8,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueExchange {
    pub timestamp: DateTime<Utc>,
    pub speaker_id: Uuid,
    pub speaker_name: String,
    pub content: String,
    pub dialogue_type: DialogueType,
    pub response_to: Option<Uuid>, // Reference to previous dialogue
    pub emotional_context: EmotionalContext,
    pub player_choices: Vec<DialogueChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DialogueType {
    CharacterSpeech,
    PlayerResponse,
    SystemNarration,
    EnvironmentalDescription,
    InternalThought,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalContext {
    pub speaker_mood: String,
    pub relationship_tension: f32, // -1.0 to 1.0
    pub scene_atmosphere: String,
    pub urgency_level: UrgencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UrgencyLevel {
    Relaxed,
    Normal,
    Elevated,
    Urgent,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub choice_text: String,
    pub chosen: bool,
    pub available: bool,
    pub requirements_met: bool,
    pub potential_consequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EnvironmentalEventType,
    pub description: String,
    pub affected_characters: Vec<Uuid>,
    pub triggered_by: Option<TriggerSource>,
    pub impact_rating: ImpactRating,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvironmentalEventType {
    LightingChange,
    SoundEffect,
    TemperatureChange,
    WeatherEvent,
    EmergencyAlert,
    SystemFailure,
    ObjectMovement,
    HapticFeedback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerSource {
    PlayerAction(Uuid),
    CharacterAction(Uuid),
    SystemEvent,
    TimeElapsed,
    StoryProgression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactRating {
    Minimal,
    Noticeable,
    Significant,
    Major,
    Dramatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneOutcome {
    pub success_rating: f32, // 0.0 - 1.0
    pub objectives_completed: u32,
    pub objectives_total: u32,
    pub key_decisions: Vec<String>,
    pub character_development: Vec<CharacterDevelopment>,
    pub story_branch_chosen: Option<String>,
    pub next_scene_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDevelopment {
    pub character_id: Uuid,
    pub development_type: DevelopmentType,
    pub description: String,
    pub impact_level: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DevelopmentType {
    SkillIncrease,
    RelationshipChange,
    PersonalityShift,
    KnowledgeGained,
    TrustBuilt,
    TrustLost,
    GoalAchievement,
    CharacterArcProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDecision {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub scene_id: Uuid,
    pub decision_point: String,
    pub options_presented: Vec<DecisionOption>,
    pub chosen_option: Uuid,
    pub decision_time_seconds: u32,
    pub confidence_level: f32, // 0.0 - 1.0
    pub consequences_realized: Vec<ConsequenceRealization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequenceRealization {
    pub consequence: Consequence,
    pub realized_at: DateTime<Utc>,
    pub intensity: f32, // 0.0 - 1.0
    pub player_awareness: bool, // Did player notice this consequence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub character_id: Uuid,
    pub current_mood: String,
    pub trust_levels: HashMap<Uuid, f32>, // Relationships with other characters
    pub skill_levels: HashMap<String, u8>,
    pub inventory: Vec<String>,
    pub knowledge_acquired: Vec<String>,
    pub personal_goals: Vec<PersonalGoal>,
    pub relationship_history: Vec<RelationshipEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalGoal {
    pub description: String,
    pub priority: GoalPriority,
    pub progress: f32, // 0.0 - 1.0
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub timestamp: DateTime<Utc>,
    pub other_character_id: Uuid,
    pub event_type: RelationshipEventType,
    pub impact: f32, // -1.0 to 1.0
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipEventType {
    FirstMeeting,
    PositiveInteraction,
    NegativeInteraction,
    Conflict,
    Cooperation,
    TrustBuilt,
    TrustBroken,
    RomanticInterest,
    FriendshipFormed,
    ProfessionalRespect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipChange {
    pub character_id: Uuid,
    pub relationship_type: String,
    pub old_value: f32,
    pub new_value: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_play_time_minutes: u32,
    pub scenes_completed: u32,
    pub objectives_achieved: u32,
    pub decisions_made: u32,
    pub skill_checks_passed: u32,
    pub skill_checks_failed: u32,
    pub character_interactions: u32,
    pub story_branches_explored: u32,
    pub items_collected: u32,
    pub player_engagement_score: f32, // 0.0 - 1.0
    pub difficulty_rating: f32, // Player's perceived difficulty
}

impl StoryBook {
    pub fn new(template_id: Uuid, name: String, starting_node: GraphNode) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            template_id,
            name,
            played_scenes: Vec::new(),
            current_position: starting_node,
            player_decisions: Vec::new(),
            story_flags: HashMap::new(),
            character_states: HashMap::new(),
            session_metrics: SessionMetrics::default(),
            created_at: now,
            last_saved: now,
            status: StoryBookStatus::InProgress,
        }
    }

    pub fn add_played_scene(&mut self, scene: PlayedScene) {
        self.played_scenes.push(scene);
        self.session_metrics.scenes_completed += 1;
        self.last_saved = Utc::now();
    }

    pub fn record_decision(&mut self, decision: PlayerDecision) {
        self.player_decisions.push(decision);
        self.session_metrics.decisions_made += 1;
        self.last_saved = Utc::now();
    }

    pub fn set_story_flag(&mut self, flag: String, value: serde_json::Value) {
        self.story_flags.insert(flag, value);
        self.last_saved = Utc::now();
    }

    pub fn get_story_flag(&self, flag: &str) -> Option<&serde_json::Value> {
        self.story_flags.get(flag)
    }

    pub fn update_character_state(&mut self, character_id: Uuid, state: CharacterState) {
        self.character_states.insert(character_id, state);
        self.last_saved = Utc::now();
    }

    pub fn calculate_completion_percentage(&self) -> f32 {
        if self.session_metrics.scenes_completed == 0 {
            return 0.0;
        }

        // This is a simplified calculation - real implementation would depend on story template
        let estimated_total_scenes = 10.0; // Would come from story template
        (self.session_metrics.scenes_completed as f32 / estimated_total_scenes).min(1.0)
    }

    pub fn get_current_scene_summary(&self) -> Option<String> {
        self.played_scenes.last().map(|scene| {
            format!(
                "Scene completed at {} with {} objectives completed and {} interactions performed",
                scene.end_time.unwrap_or(scene.start_time).format("%H:%M:%S"),
                scene.objectives_completed.len(),
                scene.interactions_performed.len()
            )
        })
    }
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            total_play_time_minutes: 0,
            scenes_completed: 0,
            objectives_achieved: 0,
            decisions_made: 0,
            skill_checks_passed: 0,
            skill_checks_failed: 0,
            character_interactions: 0,
            story_branches_explored: 0,
            items_collected: 0,
            player_engagement_score: 0.5,
            difficulty_rating: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryBookCollection {
    pub books: Vec<StoryBook>,
    pub templates_used: HashMap<Uuid, u32>, // template_id -> usage_count
    pub total_play_time: u32,
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub favorite_genres: Vec<String>,
    pub preferred_difficulty: String,
    pub favorite_characters: Vec<Uuid>,
    pub content_filters: Vec<String>,
    pub accessibility_options: AccessibilityOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityOptions {
    pub large_text: bool,
    pub high_contrast: bool,
    pub audio_descriptions: bool,
    pub subtitles: bool,
    pub reduced_motion: bool,
    pub voice_commands: bool,
}

impl StoryBookCollection {
    pub fn new() -> Self {
        Self {
            books: Vec::new(),
            templates_used: HashMap::new(),
            total_play_time: 0,
            user_preferences: UserPreferences::default(),
        }
    }

    pub fn add_story_book(&mut self, book: StoryBook) {
        // Update template usage count
        let template_id = book.template_id;
        *self.templates_used.entry(template_id).or_insert(0) += 1;

        // Update total play time
        self.total_play_time += book.session_metrics.total_play_time_minutes;

        self.books.push(book);
    }

    pub fn get_most_popular_templates(&self) -> Vec<(Uuid, u32)> {
        let mut template_popularity: Vec<_> = self.templates_used.iter()
            .map(|(&id, &count)| (id, count))
            .collect();
        template_popularity.sort_by(|a, b| b.1.cmp(&a.1));
        template_popularity
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            favorite_genres: vec!["Adventure".to_string(), "SciFi".to_string()],
            preferred_difficulty: "Intermediate".to_string(),
            favorite_characters: Vec::new(),
            content_filters: Vec::new(),
            accessibility_options: AccessibilityOptions::default(),
        }
    }
}

impl Default for AccessibilityOptions {
    fn default() -> Self {
        Self {
            large_text: false,
            high_contrast: false,
            audio_descriptions: false,
            subtitles: true,
            reduced_motion: false,
            voice_commands: false,
        }
    }
}
