// ABOUTME: Story template types and graph structures for holodeck narrative generation
// ABOUTME: Defines story templates, scenes, decision trees and narrative flow patterns for Star Trek experiences

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::Position3D;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryTemplate {
    pub id: Uuid,
    pub name: String,
    pub topic: String,
    pub genre: StoryGenre,
    pub scenes: Vec<SceneTemplate>,
    pub story_graph: StoryGraph,
    pub metadata: StoryMetadata,
    pub estimated_duration_minutes: u32,
    pub difficulty_level: DifficultyLevel,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoryGenre {
    Adventure,
    Mystery,
    Drama,
    Comedy,
    Historical,
    SciFi,
    Fantasy,
    Educational,
    Diplomatic,
    Exploration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGraph {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub root_node_id: Uuid,
    pub ending_node_ids: Vec<Uuid>,
    pub branching_points: Vec<BranchingPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub connections: Vec<NodeConnection>,
    pub is_checkpoint: bool,
    pub prerequisites: Vec<Prerequisite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub target_node_id: Uuid,
    pub condition: TransitionCondition,
    pub weight: f32, // Preference/probability weight
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransitionCondition {
    PlayerChoice(String),
    SkillCheck { skill: SkillType, difficulty: u8 },
    ItemRequired(String),
    CharacterPresent(Uuid),
    TimeElapsed(u32), // minutes
    EventTriggered(String),
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillType {
    Diplomacy,
    Engineering,
    Medical,
    Navigation,
    Tactics,
    Physics,
    Leadership,
    Investigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchingPoint {
    pub node_id: Uuid,
    pub decision_type: DecisionType,
    pub options: Vec<DecisionOption>,
    pub timeout_seconds: Option<u32>,
    pub default_option: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionType {
    MultipleChoice,
    YesNo,
    TextInput,
    ActionSelection,
    CharacterInteraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: Uuid,
    pub text: String,
    pub target_node_id: Uuid,
    pub consequences: Vec<Consequence>,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consequence {
    pub consequence_type: ConsequenceType,
    pub description: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsequenceType {
    CharacterMoodChange { character_id: Uuid, new_mood: String },
    EnvironmentChange(String),
    ItemGained(String),
    ItemLost(String),
    RelationshipChange { character_id: Uuid, change: i8 },
    StoryFlagSet(String),
    HealthChange(i8),
    TimePenalty(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Minor,
    Moderate,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub requirement_type: RequirementType,
    pub description: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementType {
    SkillLevel { skill: SkillType, min_level: u8 },
    ItemPossessed(String),
    CharacterTrust { character_id: Uuid, min_trust: f32 },
    PreviousChoice(String),
    StoryFlag(String),
    TimeRemaining(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub environment_type: EnvironmentType,
    pub required_characters: Vec<CharacterRole>,
    pub optional_characters: Vec<CharacterRole>,
    pub scene_objectives: Vec<SceneObjective>,
    pub dialogue_templates: Vec<DialogueTemplate>,
    pub environmental_cues: Vec<EnvironmentalCue>,
    pub interactive_elements: Vec<InteractiveElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvironmentType {
    EnterpriseDbridge,
    EnterpriseDtenForward,
    EnterpriseDengineering,
    EnterpriseDsickbay,
    EnterpriseDreadyRoom,
    EnterpriseDholoDeck,
    Starbase,
    AlienPlanet,
    SpaceStation,
    Shuttlecraft,
    AlienShip,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRole {
    pub role_name: String,
    pub character_type: String, // Can map to actual Character
    pub importance: RoleImportance,
    pub starting_position: Option<Position3D>,
    pub role_objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoleImportance {
    Essential,
    Important,
    Supporting,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObjective {
    pub id: Uuid,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub success_conditions: Vec<SuccessCondition>,
    pub hints: Vec<String>,
    pub time_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObjectiveType {
    GatherInformation,
    SolveProblem,
    NavigateSocial,
    TechnicalChallenge,
    CombatEncounter,
    Exploration,
    Negotiation,
    Investigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCondition {
    pub condition_type: ConditionType,
    pub target_value: serde_json::Value,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionType {
    VariableEquals(String),
    VariableGreaterThan(String),
    ItemCollected(String),
    CharacterSpokenTo(Uuid),
    LocationReached(String),
    TimeElapsed(u32),
    EventTriggered(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTemplate {
    pub id: Uuid,
    pub speaker_role: String,
    pub content: String,
    pub context_triggers: Vec<ContextTrigger>,
    pub response_options: Vec<ResponseOption>,
    pub emotional_tone: EmotionalTone,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextTrigger {
    PlayerApproaches,
    ObjectiveActive(Uuid),
    StoryFlagSet(String),
    CharacterMoodIs(String),
    TimeOfScene(u32), // seconds into scene
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOption {
    pub text: String,
    pub leads_to_dialogue: Option<Uuid>,
    pub sets_story_flag: Option<String>,
    pub character_reaction: CharacterReaction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CharacterReaction {
    Positive,
    Negative,
    Neutral,
    Surprised,
    Annoyed,
    Pleased,
    Concerned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmotionalTone {
    Neutral,
    Formal,
    Friendly,
    Urgent,
    Mysterious,
    Concerned,
    Excited,
    Serious,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalCue {
    pub cue_type: CueType,
    pub trigger_condition: String,
    pub description: String,
    pub sensory_details: SensoryDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CueType {
    Visual,
    Audio,
    Haptic,
    Environmental, // Temperature, wind, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryDetails {
    pub visual: Option<String>,
    pub audio: Option<String>,
    pub haptic: Option<String>,
    pub environmental: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub id: Uuid,
    pub name: String,
    pub element_type: ElementType,
    pub position: Position3D,
    pub interactions: Vec<Interaction>,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    ControlPanel,
    ComputerTerminal,
    Door,
    Container,
    Weapon,
    MedicalDevice,
    CommunicationDevice,
    Tool,
    Decoration,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub action_name: String,
    pub description: String,
    pub results: Vec<InteractionResult>,
    pub skill_check: Option<SkillCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCheck {
    pub skill: SkillType,
    pub difficulty: u8, // 1-10
    pub success_description: String,
    pub failure_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResult {
    pub result_type: InteractionResultType,
    pub description: String,
    pub story_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InteractionResultType {
    InformationGained,
    ItemObtained,
    PassageOpened,
    SystemActivated,
    CharacterSummoned,
    EnvironmentChanged,
    ObjectiveUpdated,
    StoryProgressed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryMetadata {
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub target_audience: TargetAudience,
    pub content_rating: ContentRating,
    pub learning_objectives: Vec<String>,
    pub cultural_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum TargetAudience {
    Children,
    Teens,
    Adults,
    Families,
    StarfleetCadets,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentRating {
    Everyone,
    Teen,
    Mature,
    Educational,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    pub prerequisite_type: PrerequisiteType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrerequisiteType {
    SceneCompleted(Uuid),
    ObjectiveAchieved(Uuid),
    SkillLevelReached { skill: SkillType, level: u8 },
    ItemObtained(String),
    CharacterRelationship { character_id: Uuid, min_trust: f32 },
}

impl StoryTemplate {
    pub fn new(name: String, topic: String, genre: StoryGenre) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            topic,
            genre,
            scenes: Vec::new(),
            story_graph: StoryGraph {
                nodes: HashMap::new(),
                root_node_id: Uuid::now_v7(),
                ending_node_ids: Vec::new(),
                branching_points: Vec::new(),
            },
            metadata: StoryMetadata {
                author: "Holodeck System".to_string(),
                version: "1.0".to_string(),
                tags: Vec::new(),
                target_audience: TargetAudience::General,
                content_rating: ContentRating::Everyone,
                learning_objectives: Vec::new(),
                cultural_notes: Vec::new(),
            },
            estimated_duration_minutes: 30,
            difficulty_level: DifficultyLevel::Beginner,
            created_at: Utc::now(),
        }
    }

    pub fn add_scene(&mut self, scene: SceneTemplate) {
        self.scenes.push(scene);
    }

    pub fn create_linear_graph(&mut self) -> Result<(), String> {
        if self.scenes.is_empty() {
            return Err("Cannot create graph with no scenes".to_string());
        }

        // Create nodes for each scene
        let mut node_ids = Vec::new();
        for scene in &self.scenes {
            let node_id = Uuid::now_v7();
            let node = GraphNode {
                id: node_id,
                scene_id: scene.id,
                connections: Vec::new(),
                is_checkpoint: false,
                prerequisites: Vec::new(),
            };
            self.story_graph.nodes.insert(node_id, node);
            node_ids.push(node_id);
        }

        // Connect nodes linearly
        for i in 0..node_ids.len() - 1 {
            let current_id = node_ids[i];
            let next_id = node_ids[i + 1];

            if let Some(node) = self.story_graph.nodes.get_mut(&current_id) {
                node.connections.push(NodeConnection {
                    target_node_id: next_id,
                    condition: TransitionCondition::Always,
                    weight: 1.0,
                    description: "Continue to next scene".to_string(),
                });
            }
        }

        // Set root and ending nodes
        self.story_graph.root_node_id = node_ids[0];
        self.story_graph.ending_node_ids = vec![node_ids[node_ids.len() - 1]];

        Ok(())
    }
}
