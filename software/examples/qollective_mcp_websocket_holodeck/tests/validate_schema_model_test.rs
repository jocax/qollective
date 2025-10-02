// ABOUTME: Comprehensive validation tests ensuring JSON schemas match Rust shared types perfectly
// ABOUTME: Critical for data integrity across all MCP servers and desktop client components

use std::fs;
use std::path::Path;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use jsonschema::{Validator};

use shared_types::*;

/// Local schema resolver that inlines cross-schema references
struct LocalSchemaResolver {
    schema_documents: HashMap<String, Value>,
}

impl LocalSchemaResolver {
    fn new(schema_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut schema_documents = HashMap::new();

        // Load all schema files
        let schema_files = [
            ("holodeck.json", "holodeck.json"),
            ("character.json", "character.json"),
            ("story_template.json", "story_template.json"),
            ("story_book.json", "story_book.json"),
        ];

        for (filename, key) in &schema_files {
            let content = fs::read_to_string(schema_dir.join(filename))?;
            let schema_value: Value = serde_json::from_str(&content)?;
            schema_documents.insert(key.to_string(), schema_value);
        }

        Ok(Self { schema_documents })
    }

    fn compile_schema(&self, schema_name: &str) -> Result<Validator, Box<dyn std::error::Error>> {
        let mut schema_value = self.schema_documents.get(schema_name)
            .ok_or_else(|| format!("Schema not found: {}", schema_name))?
            .clone();

        // Inline cross-schema references
        self.inline_external_refs(&mut schema_value)?;

        let compiled = jsonschema::validator_for(&schema_value)
            .map_err(|e| format!("Failed to compile schema {}: {}", schema_name, e))?;
        Ok(compiled)
    }

    fn inline_external_refs(&self, value: &mut Value) -> Result<(), Box<dyn std::error::Error>> {
        match value {
            Value::Object(obj) => {
                if let Some(ref_value) = obj.get("$ref") {
                    if let Value::String(ref_str) = ref_value {
                        if ref_str.ends_with(".json#") {
                            // This is a cross-schema reference like "character.json#"
                            let schema_name = ref_str.replace("#", "");
                            if let Some(referenced_schema) = self.schema_documents.get(&schema_name) {
                                // Replace this object with the referenced schema
                                *value = referenced_schema.clone();
                                // Continue processing the inlined schema
                                self.inline_external_refs(value)?;
                            } else {
                                return Err(format!("Referenced schema not found: {}", schema_name).into());
                            }
                        }
                    }
                } else {
                    // Recursively process all object values
                    for (_, v) in obj.iter_mut() {
                        self.inline_external_refs(v)?;
                    }
                }
            },
            Value::Array(arr) => {
                // Recursively process all array elements
                for item in arr.iter_mut() {
                    self.inline_external_refs(item)?;
                }
            },
            _ => {} // Primitive values don't need processing
        }
        Ok(())
    }
}

/// Test infrastructure for schema validation
struct SchemaValidator {
    holodeck_schema: Validator,
    character_schema: Validator,
    story_template_schema: Validator,
    story_book_schema: Validator,
}

impl SchemaValidator {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let schema_dir = Path::new("schemas");
        let resolver = LocalSchemaResolver::new(schema_dir)?;

        let holodeck_schema = resolver.compile_schema("holodeck.json")
            .map_err(|e| format!("Failed to compile holodeck schema: {}", e))?;
        let character_schema = resolver.compile_schema("character.json")
            .map_err(|e| format!("Failed to compile character schema: {}", e))?;
        let story_template_schema = resolver.compile_schema("story_template.json")
            .map_err(|e| format!("Failed to compile story_template schema: {}", e))?;
        let story_book_schema = resolver.compile_schema("story_book.json")
            .map_err(|e| format!("Failed to compile story_book schema: {}", e))?;

        Ok(Self {
            holodeck_schema,
            character_schema,
            story_template_schema,
            story_book_schema,
        })
    }

    fn validate_against_schema(&self, schema: &Validator, data: &Value, type_name: &str) -> Result<(), String> {
        match schema.validate(data) {
            Ok(()) => Ok(()),
            Err(error) => {
                let error_description = format!("Schema validation failed for {}: {}", type_name, error);
                println!("{}", error_description);
                Err(error_description)
            }
        }
    }
}

/// Test utilities for generating valid test data
mod test_data {
    use super::*;

    pub fn create_test_character() -> Character {
        Character {
            id: Uuid::now_v7(),
            name: "Jean-Luc Picard".to_string(),
            character_type: CharacterType::Captain,
            personality: PersonalityTraits {
                logical: 0.8,
                emotional: 0.6,
                authoritative: 0.9,
                diplomatic: 0.95,
                curious: 0.7,
                cautious: 0.8,
                humor_tendency: 0.3,
                leadership_style: LeadershipStyle::Commanding,
                communication_style: CommunicationStyle::Diplomatic,
            },
            voice_config: VoiceConfig {
                voice_actor: "Patrick Stewart".to_string(),
                speech_patterns: vec![SpeechPattern {
                    pattern_type: SpeechPatternType::MilitaryProtocol,
                    frequency: 0.8,
                    examples: vec!["Make it so.".to_string(), "Engage.".to_string()],
                }],
                common_phrases: vec!["Make it so".to_string(), "Number One".to_string()],
                accent: Some("British".to_string()),
                tone_modulation: ToneModulation {
                    base_pitch: 0.3,
                    emotional_range: 0.7,
                    formality_level: 0.9,
                    speech_speed: 0.5,
                },
            },
            appearance: CharacterAppearance {
                species: Species::Human,
                height_cm: 178.0,
                uniform_color: UniformColor::Command,
                distinctive_features: vec!["Bald head".to_string(), "French accent".to_string()],
                holographic_fidelity: FidelityLevel::Photorealistic,
            },
            knowledge_domains: vec![KnowledgeDomain::Starfleet, KnowledgeDomain::Diplomacy, KnowledgeDomain::History],
            relationships: vec![],
            current_mood: Mood::Focused,
            position: Some(Position3D { x: 0.0, y: 0.0, z: 0.0 }),
            status: CharacterStatus::Active,
        }
    }

    pub fn create_test_holodeck() -> Holodeck {
        Holodeck {
            id: Uuid::now_v7(),
            name: "Enterprise Bridge Training".to_string(),
            topic: "Command bridge operations training scenario".to_string(),
            story_type: HolodeckStoryType::Educational,
            participants: vec![create_test_character()],
            current_scene: None,
            configuration: HolodeckConfig {
                safety_level: SafetyLevel::Training,
                max_participants: 4,
                duration_minutes: Some(60),
                auto_save_enabled: true,
                voice_recognition: true,
                haptic_feedback: true,
                replicator_access: false,
                transporter_integration: false,
                environmental_controls: EnvironmentalControls {
                    temperature_celsius: 22.0,
                    humidity_percent: 45.0,
                    atmospheric_pressure: 101.3,
                    oxygen_level: 21.0,
                    wind_simulation: false,
                    weather_effects: false,
                },
                llm_config: LlmConfig::default(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_test_story_template() -> StoryTemplate {
        StoryTemplate {
            id: Uuid::now_v7(),
            name: "Diplomatic Mission to Risa".to_string(),
            topic: "Negotiate peace treaty on pleasure planet".to_string(),
            genre: StoryGenre::Diplomatic,
            scenes: vec![],
            story_graph: StoryGraph {
                nodes: std::collections::HashMap::new(),
                root_node_id: Uuid::now_v7(),
                ending_node_ids: vec![Uuid::now_v7()],
                branching_points: vec![],
            },
            metadata: StoryMetadata {
                author: "Holodeck Designer AI".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["diplomacy".to_string(), "risa".to_string()],
                target_audience: TargetAudience::Adults,
                content_rating: ContentRating::Everyone,
                learning_objectives: vec!["Diplomatic negotiation skills".to_string()],
                cultural_notes: vec!["Risian hospitality customs".to_string()],
            },
            estimated_duration_minutes: 45,
            difficulty_level: DifficultyLevel::Intermediate,
            created_at: Utc::now(),
        }
    }

    pub fn create_test_story_book() -> StoryBook {
        StoryBook {
            id: Uuid::now_v7(),
            template_id: Uuid::now_v7(),
            name: "Captain's Diplomatic Mission Log".to_string(),
            played_scenes: vec![],
            current_position: GraphNode {
                id: Uuid::now_v7(),
                scene_id: Uuid::now_v7(),
                connections: vec![],
                is_checkpoint: true,
                prerequisites: vec![],
            },
            player_decisions: vec![],
            story_flags: std::collections::HashMap::new(),
            character_states: std::collections::HashMap::new(),
            session_metrics: SessionMetrics {
                total_play_time_minutes: 0,
                scenes_completed: 0,
                objectives_achieved: 0,
                decisions_made: 0,
                skill_checks_passed: 0,
                skill_checks_failed: 0,
                character_interactions: 0,
                story_branches_explored: 0,
                items_collected: 0,
                player_engagement_score: 0.0,
                difficulty_rating: 0.5,
            },
            created_at: Utc::now(),
            last_saved: Utc::now(),
            status: StoryBookStatus::InProgress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_loading() {
        let validator = SchemaValidator::new();
        assert!(validator.is_ok(), "Failed to load schemas: {:?}", validator.err());
    }

    #[test]
    fn test_character_schema_alignment() {
        let validator = SchemaValidator::new().expect("Failed to create validator");
        let test_character = test_data::create_test_character();

        // Test Rust -> JSON -> Schema validation
        let json_value = serde_json::to_value(&test_character)
            .expect("Failed to serialize Character to JSON");

        validator.validate_against_schema(&validator.character_schema, &json_value, "Character")
            .expect("Character schema validation failed");

        // Test JSON -> Rust roundtrip
        let deserialized: Character = serde_json::from_value(json_value)
            .expect("Failed to deserialize Character from JSON");

        assert_eq!(test_character.name, deserialized.name);
        assert_eq!(test_character.character_type, deserialized.character_type);
        assert_eq!(test_character.personality.logical, deserialized.personality.logical);
    }

    #[test]
    fn test_holodeck_schema_alignment() {
        let validator = SchemaValidator::new().expect("Failed to create validator");
        let test_holodeck = test_data::create_test_holodeck();

        // Test Rust -> JSON -> Schema validation
        let json_value = serde_json::to_value(&test_holodeck)
            .expect("Failed to serialize Holodeck to JSON");

        validator.validate_against_schema(&validator.holodeck_schema, &json_value, "Holodeck")
            .expect("Holodeck schema validation failed");

        // Test JSON -> Rust roundtrip
        let deserialized: Holodeck = serde_json::from_value(json_value)
            .expect("Failed to deserialize Holodeck from JSON");

        assert_eq!(test_holodeck.name, deserialized.name);
        assert_eq!(test_holodeck.story_type, deserialized.story_type);
        assert_eq!(test_holodeck.configuration.safety_level, deserialized.configuration.safety_level);
    }

    #[test]
    fn test_story_template_schema_alignment() {
        let validator = SchemaValidator::new().expect("Failed to create validator");
        let test_template = test_data::create_test_story_template();

        // Test Rust -> JSON -> Schema validation
        let json_value = serde_json::to_value(&test_template)
            .expect("Failed to serialize StoryTemplate to JSON");

        validator.validate_against_schema(&validator.story_template_schema, &json_value, "StoryTemplate")
            .expect("StoryTemplate schema validation failed");

        // Test JSON -> Rust roundtrip
        let deserialized: StoryTemplate = serde_json::from_value(json_value)
            .expect("Failed to deserialize StoryTemplate from JSON");

        assert_eq!(test_template.name, deserialized.name);
        assert_eq!(test_template.genre, deserialized.genre);
        assert_eq!(test_template.difficulty_level, deserialized.difficulty_level);
    }

    #[test]
    fn test_story_book_schema_alignment() {
        let validator = SchemaValidator::new().expect("Failed to create validator");
        let test_book = test_data::create_test_story_book();

        // Test Rust -> JSON -> Schema validation
        let json_value = serde_json::to_value(&test_book)
            .expect("Failed to serialize StoryBook to JSON");

        validator.validate_against_schema(&validator.story_book_schema, &json_value, "StoryBook")
            .expect("StoryBook schema validation failed");

        // Test JSON -> Rust roundtrip
        let deserialized: StoryBook = serde_json::from_value(json_value)
            .expect("Failed to deserialize StoryBook from JSON");

        assert_eq!(test_book.name, deserialized.name);
        assert_eq!(test_book.status, deserialized.status);
        assert_eq!(test_book.session_metrics.total_play_time_minutes, deserialized.session_metrics.total_play_time_minutes);
    }

    #[test]
    fn test_enum_variants_comprehensive() {
        // Test all major enum variants are correctly represented in schemas and Rust

        // Character enums
        let character_types = vec![
            CharacterType::Captain,
            CharacterType::FirstOfficer,
            CharacterType::Android,
            CharacterType::ChiefEngineer,
        ];

        for char_type in character_types {
            let json_val = serde_json::to_value(&char_type).expect("Failed to serialize CharacterType");
            let back: CharacterType = serde_json::from_value(json_val).expect("Failed to deserialize CharacterType");
            assert_eq!(char_type, back);
        }

        // Story type enums
        let story_types = vec![
            HolodeckStoryType::Adventure,
            HolodeckStoryType::Mystery,
            HolodeckStoryType::Educational,
        ];

        for story_type in story_types {
            let json_val = serde_json::to_value(&story_type).expect("Failed to serialize HolodeckStoryType");
            let back: HolodeckStoryType = serde_json::from_value(json_val).expect("Failed to deserialize HolodeckStoryType");
            assert_eq!(story_type, back);
        }

        // Safety level enums
        let safety_levels = vec![
            SafetyLevel::Training,
            SafetyLevel::Standard,
            SafetyLevel::Reduced,
            SafetyLevel::Disabled,
        ];

        for safety_level in safety_levels {
            let json_val = serde_json::to_value(&safety_level).expect("Failed to serialize SafetyLevel");
            let back: SafetyLevel = serde_json::from_value(json_val).expect("Failed to deserialize SafetyLevel");
            assert_eq!(safety_level, back);
        }
    }

    #[test]
    fn test_uuid_format_consistency() {
        let test_uuid = Uuid::now_v7();
        let json_val = serde_json::to_value(&test_uuid).expect("Failed to serialize UUID");

        // Ensure UUID is serialized as string
        assert!(json_val.is_string());

        // Ensure it can be deserialized back
        let back: Uuid = serde_json::from_value(json_val).expect("Failed to deserialize UUID");
        assert_eq!(test_uuid, back);
    }

    #[test]
    fn test_datetime_format_consistency() {
        let test_datetime = Utc::now();
        let json_val = serde_json::to_value(&test_datetime).expect("Failed to serialize DateTime");

        // Ensure DateTime is serialized as string in ISO format
        assert!(json_val.is_string());

        // Ensure it can be deserialized back
        let back: DateTime<Utc> = serde_json::from_value(json_val).expect("Failed to deserialize DateTime");

        // Allow for minor precision differences in serialization
        let diff = (test_datetime.timestamp_millis() - back.timestamp_millis()).abs();
        assert!(diff < 1000, "DateTime roundtrip exceeded 1 second tolerance");
    }

    #[test]
    fn test_schema_rejects_invalid_data() {
        let validator = SchemaValidator::new().expect("Failed to create validator");

        // Test invalid Character data
        let invalid_character = json!({
            "id": "not-a-uuid",
            "name": "",
            "character_type": "InvalidType"
        });

        let result = validator.validate_against_schema(&validator.character_schema, &invalid_character, "InvalidCharacter");
        assert!(result.is_err(), "Schema should reject invalid character data");

        // Test invalid Holodeck data
        let invalid_holodeck = json!({
            "id": "not-a-uuid",
            "name": "",
            "story_type": "InvalidStoryType",
            "participants": "should-be-array"
        });

        let result = validator.validate_against_schema(&validator.holodeck_schema, &invalid_holodeck, "InvalidHolodeck");
        assert!(result.is_err(), "Schema should reject invalid holodeck data");
    }

    #[test]
    fn test_complex_nested_structure() {
        let validator = SchemaValidator::new().expect("Failed to create validator");

        // Create a more complex holodeck with nested structures
        let mut complex_holodeck = test_data::create_test_holodeck();
        complex_holodeck.current_scene = Some(Scene {
            id: Uuid::now_v7(),
            name: "Enterprise Bridge".to_string(),
            description: "The bridge of the USS Enterprise NCC-1701-D".to_string(),
            environment_id: Uuid::now_v7(),
            characters_present: vec![Uuid::now_v7()],
            props: vec![SceneProp {
                id: Uuid::now_v7(),
                name: "Captain's Chair".to_string(),
                description: "The iconic captain's chair".to_string(),
                position: Position3D { x: 0.0, y: 1.0, z: 2.0 },
                interactive: true,
                physics_enabled: true,
            }],
            background_audio: Some("Bridge ambient sounds".to_string()),
            lighting_config: LightingConfig {
                ambient_light: 0.7,
                directional_lights: vec![DirectionalLight {
                    direction: Position3D { x: 1.0, y: -1.0, z: 0.0 },
                    intensity: 0.8,
                    color: Color { r: 255, g: 255, b: 255, a: 255 },
                }],
                mood: LightingMood::Bright,
            },
            physics_settings: PhysicsSettings {
                gravity_enabled: true,
                gravity_strength: 9.81,
                collision_detection: true,
                real_time_physics: true,
            },
        });

        let json_value = serde_json::to_value(&complex_holodeck)
            .expect("Failed to serialize complex Holodeck");

        validator.validate_against_schema(&validator.holodeck_schema, &json_value, "ComplexHolodeck")
            .expect("Complex holodeck validation failed");

        // Ensure roundtrip works
        let deserialized: Holodeck = serde_json::from_value(json_value)
            .expect("Failed to deserialize complex Holodeck");

        assert!(deserialized.current_scene.is_some());
        let scene = deserialized.current_scene.unwrap();
        assert_eq!(scene.name, "Enterprise Bridge");
        assert_eq!(scene.props.len(), 1);
        assert_eq!(scene.props[0].name, "Captain's Chair");
    }

    #[test]
    fn test_edge_case_values() {
        let validator = SchemaValidator::new().expect("Failed to create validator");

        // Test boundary values for numeric fields
        let mut test_character = test_data::create_test_character();
        test_character.personality.logical = 0.0; // Minimum value
        test_character.personality.emotional = 1.0; // Maximum value
        test_character.appearance.height_cm = 30.0; // Minimum height

        let json_value = serde_json::to_value(&test_character)
            .expect("Failed to serialize edge case Character");

        validator.validate_against_schema(&validator.character_schema, &json_value, "EdgeCaseCharacter")
            .expect("Edge case character validation failed");

        // Test maximum values for holodeck config
        let mut test_holodeck = test_data::create_test_holodeck();
        test_holodeck.configuration.max_participants = 20; // Maximum allowed
        test_holodeck.configuration.duration_minutes = Some(480); // Maximum 8 hours
        test_holodeck.configuration.environmental_controls.temperature_celsius = 100.0; // Maximum temp

        let json_value = serde_json::to_value(&test_holodeck)
            .expect("Failed to serialize edge case Holodeck");

        validator.validate_against_schema(&validator.holodeck_schema, &json_value, "EdgeCaseHolodeck")
            .expect("Edge case holodeck validation failed");
    }

    #[test]
    fn test_optional_fields_handling() {
        let validator = SchemaValidator::new().expect("Failed to create validator");

        // Test character with minimal optional fields
        let mut minimal_character = test_data::create_test_character();
        minimal_character.position = None;
        minimal_character.voice_config.accent = None;

        let json_value = serde_json::to_value(&minimal_character)
            .expect("Failed to serialize minimal Character");

        validator.validate_against_schema(&validator.character_schema, &json_value, "MinimalCharacter")
            .expect("Minimal character validation failed");

        // Test holodeck with minimal config
        let mut minimal_holodeck = test_data::create_test_holodeck();
        minimal_holodeck.current_scene = None;
        minimal_holodeck.configuration.duration_minutes = None;

        let json_value = serde_json::to_value(&minimal_holodeck)
            .expect("Failed to serialize minimal Holodeck");

        validator.validate_against_schema(&validator.holodeck_schema, &json_value, "MinimalHolodeck")
            .expect("Minimal holodeck validation failed");
    }
}
