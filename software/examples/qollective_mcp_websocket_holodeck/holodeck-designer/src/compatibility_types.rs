// ABOUTME: Compatibility layer for resolving schemars version conflicts between rmcp and rig-core
// ABOUTME: Provides separate struct types with conversion traits to handle JsonSchema trait coherence issues

use serde::{Deserialize, Serialize};

// For rmcp compatibility (schemars v1.0.4)
use schemars::JsonSchema as JsonSchemaV1;
// For rig-core compatibility (schemars v0.8.22) - manual implementations only
// (derive macro conflicts with schemars v1.0.4)

/// Story generation request for rmcp server endpoints (schemars v1.0.4)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchemaV1)]
pub struct RmcpStoryGenerationRequest {
    pub tenant: Option<String>,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
    pub theme: String,
    pub story_type: String,
    pub duration_minutes: Option<u32>,
    pub max_participants: Option<u32>,
    pub characters: Vec<String>,
    pub safety_level: Option<String>,
    pub participant_experience_level: Option<String>,
    pub environment_constraints: Option<String>,
    pub narrative_complexity: Option<String>,
}

/// Story generation request for rig-core LLM integration (schemars v0.8.22)
// Using manual JsonSchema implementation for schemars v0.8.22 compatibility
// (automatic derive conflicts with schemars v1.0.4 used by rmcp)
#[derive(Debug, Deserialize, Serialize)]
pub struct RigStoryGenerationRequest {
    pub tenant: Option<String>,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
    pub theme: String,
    pub story_type: String,
    pub duration_minutes: Option<u32>,
    pub max_participants: Option<u32>,
    pub characters: Vec<String>,
    pub safety_level: Option<String>,
    pub participant_experience_level: Option<String>,
    pub environment_constraints: Option<String>,
    pub narrative_complexity: Option<String>,
}

/// Story enhancement request for rmcp (schemars v1.0.4)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchemaV1)]
pub struct RmcpStoryEnhancementRequest {
    pub story_template: String,
    pub enhancement_areas: Vec<String>,
    pub target_improvements: Vec<String>,
    pub preserve_elements: Vec<String>,
}

/// Story enhancement request for rig-core (schemars v0.8.22)
// Using manual JsonSchema implementation for schemars v0.8.22 compatibility
// (automatic derive conflicts with schemars v1.0.4 used by rmcp)
#[derive(Debug, Deserialize, Serialize)]
pub struct RigStoryEnhancementRequest {
    pub story_template: String,
    pub enhancement_areas: Vec<String>,
    pub target_improvements: Vec<String>,
    pub preserve_elements: Vec<String>,
}

/// Story validation request for rmcp (schemars v1.0.4)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchemaV1)]
pub struct RmcpStoryValidationRequest {
    pub story_content: String,
    pub validation_criteria: Vec<String>,
}

/// Story validation request for rig-core (schemars v0.8.22)
// Using manual JsonSchema implementation for schemars v0.8.22 compatibility
// (automatic derive conflicts with schemars v1.0.4 used by rmcp)
#[derive(Debug, Deserialize, Serialize)]
pub struct RigStoryValidationRequest {
    pub story_content: String,
    pub validation_criteria: Vec<String>,
}

// ===== LLM RESPONSE TYPES FOR RIG-CORE COMPATIBILITY =====

/// LLM story generation response structure for rig-core (schemars v0.8.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigLlmStoryResponse {
    pub story_content: String,
    pub scenes: Vec<RigLlmScene>,
    pub story_graph: RigLlmStoryGraph,
}

/// Scene template from LLM for rig-core (schemars v0.8.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigLlmScene {
    pub id: String,
    pub name: String,
    pub description: String,
    pub environment_type: String,
    pub required_characters: Vec<String>,
    pub optional_characters: Vec<String>,
}

/// Story graph structure from LLM for rig-core (schemars v0.8.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigLlmStoryGraph {
    pub nodes: Vec<RigLlmGraphNode>,
    pub root_node_id: String,
    pub ending_node_ids: Vec<String>,
}

/// Graph node from LLM for rig-core (schemars v0.8.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigLlmGraphNode {
    pub id: String,
    pub scene_id: String,
    pub connections: Vec<RigLlmNodeConnection>,
    pub is_checkpoint: bool,
}

/// Node connection from LLM for rig-core (schemars v0.8.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigLlmNodeConnection {
    pub target_node_id: String,
    pub condition: String,
    pub description: String,
}

// ===== CONVERSION IMPLEMENTATIONS =====

impl From<RmcpStoryGenerationRequest> for RigStoryGenerationRequest {
    fn from(rmcp_req: RmcpStoryGenerationRequest) -> Self {
        Self {
            tenant: rmcp_req.tenant,
            user_id: rmcp_req.user_id,
            request_id: rmcp_req.request_id,
            theme: rmcp_req.theme,
            story_type: rmcp_req.story_type,
            duration_minutes: rmcp_req.duration_minutes,
            max_participants: rmcp_req.max_participants,
            characters: rmcp_req.characters,
            safety_level: rmcp_req.safety_level,
            participant_experience_level: rmcp_req.participant_experience_level,
            environment_constraints: rmcp_req.environment_constraints,
            narrative_complexity: rmcp_req.narrative_complexity,
        }
    }
}

impl From<RigStoryGenerationRequest> for RmcpStoryGenerationRequest {
    fn from(rig_req: RigStoryGenerationRequest) -> Self {
        Self {
            tenant: rig_req.tenant,
            user_id: rig_req.user_id,
            request_id: rig_req.request_id,
            theme: rig_req.theme,
            story_type: rig_req.story_type,
            duration_minutes: rig_req.duration_minutes,
            max_participants: rig_req.max_participants,
            characters: rig_req.characters,
            safety_level: rig_req.safety_level,
            participant_experience_level: rig_req.participant_experience_level,
            environment_constraints: rig_req.environment_constraints,
            narrative_complexity: rig_req.narrative_complexity,
        }
    }
}

impl From<RmcpStoryEnhancementRequest> for RigStoryEnhancementRequest {
    fn from(rmcp_req: RmcpStoryEnhancementRequest) -> Self {
        Self {
            story_template: rmcp_req.story_template,
            enhancement_areas: rmcp_req.enhancement_areas,
            target_improvements: rmcp_req.target_improvements,
            preserve_elements: rmcp_req.preserve_elements,
        }
    }
}

impl From<RigStoryEnhancementRequest> for RmcpStoryEnhancementRequest {
    fn from(rig_req: RigStoryEnhancementRequest) -> Self {
        Self {
            story_template: rig_req.story_template,
            enhancement_areas: rig_req.enhancement_areas,
            target_improvements: rig_req.target_improvements,
            preserve_elements: rig_req.preserve_elements,
        }
    }
}

impl From<RmcpStoryValidationRequest> for RigStoryValidationRequest {
    fn from(rmcp_req: RmcpStoryValidationRequest) -> Self {
        Self {
            story_content: rmcp_req.story_content,
            validation_criteria: rmcp_req.validation_criteria,
        }
    }
}

impl From<RigStoryValidationRequest> for RmcpStoryValidationRequest {
    fn from(rig_req: RigStoryValidationRequest) -> Self {
        Self {
            story_content: rig_req.story_content,
            validation_criteria: rig_req.validation_criteria,
        }
    }
}

// ===== LLM RESPONSE TYPE CONVERSIONS =====

// Import the shared types for conversion
use shared_types::{LlmStoryResponse, LlmScene, LlmStoryGraph, LlmGraphNode, LlmNodeConnection};

impl From<LlmStoryResponse> for RigLlmStoryResponse {
    fn from(shared_response: LlmStoryResponse) -> Self {
        Self {
            story_content: shared_response.story_content,
            scenes: shared_response.scenes.into_iter().map(|scene| scene.into()).collect(),
            story_graph: shared_response.story_graph.into(),
        }
    }
}

impl From<RigLlmStoryResponse> for LlmStoryResponse {
    fn from(rig_response: RigLlmStoryResponse) -> Self {
        Self {
            story_content: rig_response.story_content,
            scenes: rig_response.scenes.into_iter().map(|scene| scene.into()).collect(),
            story_graph: rig_response.story_graph.into(),
        }
    }
}

impl From<LlmScene> for RigLlmScene {
    fn from(shared_scene: LlmScene) -> Self {
        Self {
            id: shared_scene.id,
            name: shared_scene.name,
            description: shared_scene.description,
            environment_type: shared_scene.environment_type,
            required_characters: shared_scene.required_characters,
            optional_characters: shared_scene.optional_characters,
        }
    }
}

impl From<RigLlmScene> for LlmScene {
    fn from(rig_scene: RigLlmScene) -> Self {
        Self {
            id: rig_scene.id,
            name: rig_scene.name,
            description: rig_scene.description,
            environment_type: rig_scene.environment_type,
            required_characters: rig_scene.required_characters,
            optional_characters: rig_scene.optional_characters,
        }
    }
}

impl From<LlmStoryGraph> for RigLlmStoryGraph {
    fn from(shared_graph: LlmStoryGraph) -> Self {
        Self {
            nodes: shared_graph.nodes.into_iter().map(|node| node.into()).collect(),
            root_node_id: shared_graph.root_node_id,
            ending_node_ids: shared_graph.ending_node_ids,
        }
    }
}

impl From<RigLlmStoryGraph> for LlmStoryGraph {
    fn from(rig_graph: RigLlmStoryGraph) -> Self {
        Self {
            nodes: rig_graph.nodes.into_iter().map(|node| node.into()).collect(),
            root_node_id: rig_graph.root_node_id,
            ending_node_ids: rig_graph.ending_node_ids,
        }
    }
}

impl From<LlmGraphNode> for RigLlmGraphNode {
    fn from(shared_node: LlmGraphNode) -> Self {
        Self {
            id: shared_node.id,
            scene_id: shared_node.scene_id,
            connections: shared_node.connections.into_iter().map(|conn| conn.into()).collect(),
            is_checkpoint: shared_node.is_checkpoint,
        }
    }
}

impl From<RigLlmGraphNode> for LlmGraphNode {
    fn from(rig_node: RigLlmGraphNode) -> Self {
        Self {
            id: rig_node.id,
            scene_id: rig_node.scene_id,
            connections: rig_node.connections.into_iter().map(|conn| conn.into()).collect(),
            is_checkpoint: rig_node.is_checkpoint,
        }
    }
}

impl From<LlmNodeConnection> for RigLlmNodeConnection {
    fn from(shared_conn: LlmNodeConnection) -> Self {
        Self {
            target_node_id: shared_conn.target_node_id,
            condition: shared_conn.condition,
            description: shared_conn.description,
        }
    }
}

impl From<RigLlmNodeConnection> for LlmNodeConnection {
    fn from(rig_conn: RigLlmNodeConnection) -> Self {
        Self {
            target_node_id: rig_conn.target_node_id,
            condition: rig_conn.condition,
            description: rig_conn.description,
        }
    }
}

// ===== MANUAL JSONSCHEMA IMPLEMENTATIONS FOR RIG-CORE COMPATIBILITY =====

impl schemars_v08::JsonSchema for RigLlmStoryResponse {
    fn schema_name() -> String {
        "RigLlmStoryResponse".to_string()
    }

    fn json_schema(gen: &mut schemars_v08::gen::SchemaGenerator) -> schemars_v08::schema::Schema {
        use schemars_v08::schema::*;
        
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("LLM Story Response".to_string()),
                description: Some("Complete story response from LLM with content, scenes, and story graph".to_string()),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = std::collections::BTreeMap::new();
                    props.insert("story_content".to_string(), gen.subschema_for::<String>());
                    props.insert("scenes".to_string(), gen.subschema_for::<Vec<RigLlmScene>>());
                    props.insert("story_graph".to_string(), gen.subschema_for::<RigLlmStoryGraph>());
                    props
                },
                required: ["story_content".to_string(), "scenes".to_string(), "story_graph".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        }.into()
    }
}

impl schemars_v08::JsonSchema for RigLlmScene {
    fn schema_name() -> String {
        "RigLlmScene".to_string()
    }

    fn json_schema(gen: &mut schemars_v08::gen::SchemaGenerator) -> schemars_v08::schema::Schema {
        use schemars_v08::schema::*;
        
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("LLM Scene".to_string()),
                description: Some("Individual scene within a holodeck story".to_string()),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = std::collections::BTreeMap::new();
                    props.insert("id".to_string(), gen.subschema_for::<String>());
                    props.insert("name".to_string(), gen.subschema_for::<String>());
                    props.insert("description".to_string(), gen.subschema_for::<String>());
                    props.insert("environment_type".to_string(), gen.subschema_for::<String>());
                    props.insert("required_characters".to_string(), gen.subschema_for::<Vec<String>>());
                    props.insert("optional_characters".to_string(), gen.subschema_for::<Vec<String>>());
                    props
                },
                required: ["id".to_string(), "name".to_string(), "description".to_string(), 
                          "environment_type".to_string(), "required_characters".to_string(), 
                          "optional_characters".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        }.into()
    }
}

impl schemars_v08::JsonSchema for RigLlmStoryGraph {
    fn schema_name() -> String {
        "RigLlmStoryGraph".to_string()
    }

    fn json_schema(gen: &mut schemars_v08::gen::SchemaGenerator) -> schemars_v08::schema::Schema {
        use schemars_v08::schema::*;
        
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("LLM Story Graph".to_string()),
                description: Some("Story flow graph with nodes and connections".to_string()),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = std::collections::BTreeMap::new();
                    props.insert("nodes".to_string(), gen.subschema_for::<Vec<RigLlmGraphNode>>());
                    props.insert("root_node_id".to_string(), gen.subschema_for::<String>());
                    props.insert("ending_node_ids".to_string(), gen.subschema_for::<Vec<String>>());
                    props
                },
                required: ["nodes".to_string(), "root_node_id".to_string(), "ending_node_ids".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        }.into()
    }
}

impl schemars_v08::JsonSchema for RigLlmGraphNode {
    fn schema_name() -> String {
        "RigLlmGraphNode".to_string()
    }

    fn json_schema(gen: &mut schemars_v08::gen::SchemaGenerator) -> schemars_v08::schema::Schema {
        use schemars_v08::schema::*;
        
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("LLM Graph Node".to_string()),
                description: Some("Individual node in the story graph".to_string()),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = std::collections::BTreeMap::new();
                    props.insert("id".to_string(), gen.subschema_for::<String>());
                    props.insert("scene_id".to_string(), gen.subschema_for::<String>());
                    props.insert("connections".to_string(), gen.subschema_for::<Vec<RigLlmNodeConnection>>());
                    props.insert("is_checkpoint".to_string(), gen.subschema_for::<bool>());
                    props
                },
                required: ["id".to_string(), "scene_id".to_string(), "connections".to_string(), "is_checkpoint".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        }.into()
    }
}

impl schemars_v08::JsonSchema for RigLlmNodeConnection {
    fn schema_name() -> String {
        "RigLlmNodeConnection".to_string()
    }

    fn json_schema(gen: &mut schemars_v08::gen::SchemaGenerator) -> schemars_v08::schema::Schema {
        use schemars_v08::schema::*;
        
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("LLM Node Connection".to_string()),
                description: Some("Connection between story graph nodes".to_string()),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: {
                    let mut props = std::collections::BTreeMap::new();
                    props.insert("target_node_id".to_string(), gen.subschema_for::<String>());
                    props.insert("condition".to_string(), gen.subschema_for::<String>());
                    props.insert("description".to_string(), gen.subschema_for::<String>());
                    props
                },
                required: ["target_node_id".to_string(), "condition".to_string(), "description".to_string()].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        }.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_generation_request_conversion() {
        let rmcp_req = RmcpStoryGenerationRequest {
            tenant: Some("test_tenant".to_string()),
            user_id: Some("user123".to_string()),
            request_id: Some("req456".to_string()),
            theme: "Adventure".to_string(),
            story_type: "Action".to_string(),
            duration_minutes: Some(45),
            max_participants: Some(4),
            characters: vec!["Kirk".to_string(), "Spock".to_string()],
            safety_level: Some("safe".to_string()),
            participant_experience_level: Some("intermediate".to_string()),
            environment_constraints: Some("starship".to_string()),
            narrative_complexity: Some("medium".to_string()),
        };

        // Test conversion to rig format
        let rig_req: RigStoryGenerationRequest = rmcp_req.clone().into();
        assert_eq!(rig_req.theme, "Adventure");
        assert_eq!(rig_req.characters.len(), 2);
        
        // Test conversion back to rmcp format
        let rmcp_req_converted: RmcpStoryGenerationRequest = rig_req.into();
        assert_eq!(rmcp_req_converted.theme, rmcp_req.theme);
        assert_eq!(rmcp_req_converted.characters, rmcp_req.characters);
    }

    #[test]
    fn test_story_enhancement_request_conversion() {
        let rmcp_req = RmcpStoryEnhancementRequest {
            story_template: "A great story".to_string(),
            enhancement_areas: vec!["dialogue".to_string(), "plot".to_string()],
            target_improvements: vec!["more tension".to_string()],
            preserve_elements: vec!["characters".to_string()],
        };

        let rig_req: RigStoryEnhancementRequest = rmcp_req.clone().into();
        assert_eq!(rig_req.story_template, "A great story");
        assert_eq!(rig_req.enhancement_areas.len(), 2);
        
        let rmcp_req_converted: RmcpStoryEnhancementRequest = rig_req.into();
        assert_eq!(rmcp_req_converted.story_template, rmcp_req.story_template);
    }

    #[test]
    fn test_story_validation_request_conversion() {
        let rmcp_req = RmcpStoryValidationRequest {
            story_content: "Story content here".to_string(),
            validation_criteria: vec!["canon compliance".to_string(), "plot coherence".to_string()],
        };

        let rig_req: RigStoryValidationRequest = rmcp_req.clone().into();
        assert_eq!(rig_req.story_content, "Story content here");
        assert_eq!(rig_req.validation_criteria.len(), 2);
        
        let rmcp_req_converted: RmcpStoryValidationRequest = rig_req.into();
        assert_eq!(rmcp_req_converted.story_content, rmcp_req.story_content);
    }
}