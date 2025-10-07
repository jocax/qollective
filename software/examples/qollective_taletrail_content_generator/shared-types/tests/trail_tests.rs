use shared_types::generated::{ContentNode, DAG, Edge};
use shared_types::generated::models::{Choice, Content, Trail, TrailStep, ContentReference};
use shared_types::generated::enums::TrailStatus;
use shared_types::extensions::trail::*;
use uuid::Uuid;
use std::collections::HashMap;

#[test]
fn test_dag_to_trail_conversion() {
    let mut dag = DAG {
        nodes: HashMap::new(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1_id = Uuid::now_v7();
    let node2_id = Uuid::now_v7();

    dag.start_node_id = node1_id;

    // Create nodes
    let choice = Choice {
        id: Uuid::now_v7(),
        text: "Go forward".to_string(),
        next_node_id: node2_id,
        metadata: None,
    };

    let content1 = Content {
        content_type: "interactive_story_node".to_string(),
        node_id: node1_id,
        text: "Start of story".to_string(),
        choices: vec![choice.clone()],
        convergence_point: false,
        next_nodes: vec![node2_id],
        educational_content: None,
    };

    let node1 = ContentNode {
        id: node1_id,
        content: content1,
        incoming_edges: 0,
        outgoing_edges: 1,
        generation_metadata: None,
    };

    let content2 = Content {
        content_type: "interactive_story_node".to_string(),
        node_id: node2_id,
        text: "End of story".to_string(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: None,
    };

    let node2 = ContentNode {
        id: node2_id,
        content: content2,
        incoming_edges: 1,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    dag.add_node(node1);
    dag.add_node(node2);

    let edge = Edge {
        from_node_id: node1_id,
        to_node_id: node2_id,
        choice_id: choice.id,
        weight: Some(1.0),
    };
    dag.add_edge(edge).unwrap();

    let (trail, trail_steps) = dag.to_trail_with_steps("Test Story".to_string());

    assert_eq!(trail.title, "Test Story");
    assert_eq!(trail.status, TrailStatus::Draft);
    assert_eq!(trail.category, "story");
    assert_eq!(trail_steps.len(), 2);
}

#[test]
fn test_trail_steps_sequential_order() {
    let mut dag = DAG {
        nodes: HashMap::new(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1_id = Uuid::now_v7();
    let node2_id = Uuid::now_v7();
    let node3_id = Uuid::now_v7();

    dag.start_node_id = node1_id;

    // Create linear chain: node1 -> node2 -> node3
    for (i, id) in [node1_id, node2_id, node3_id].iter().enumerate() {
        let next_id = match i {
            0 => Some(node2_id),
            1 => Some(node3_id),
            _ => None,
        };

        let choices = if let Some(next) = next_id {
            vec![Choice {
                id: Uuid::now_v7(),
                text: "Continue".to_string(),
                next_node_id: next,
                metadata: None,
            }]
        } else {
            vec![]
        };

        let next_nodes = if let Some(next) = next_id {
            vec![next]
        } else {
            vec![]
        };

        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id: *id,
            text: format!("Node {}", i + 1),
            choices,
            convergence_point: false,
            next_nodes,
            educational_content: None,
        };

        let node = ContentNode {
            id: *id,
            content,
            incoming_edges: if i == 0 { 0 } else { 1 },
            outgoing_edges: if i == 2 { 0 } else { 1 },
            generation_metadata: None,
        };

        dag.add_node(node);
    }

    // Add edges
    let edge1 = Edge {
        from_node_id: node1_id,
        to_node_id: node2_id,
        choice_id: Uuid::now_v7(),
        weight: Some(1.0),
    };
    let edge2 = Edge {
        from_node_id: node2_id,
        to_node_id: node3_id,
        choice_id: Uuid::now_v7(),
        weight: Some(1.0),
    };

    dag.add_edge(edge1).unwrap();
    dag.add_edge(edge2).unwrap();

    let (_, trail_steps) = dag.to_trail_with_steps("Linear Story".to_string());

    // Verify sequential ordering
    assert_eq!(trail_steps.len(), 3);
    for (i, step) in trail_steps.iter().enumerate() {
        assert_eq!(step.step_order, i as i32);
    }
}

#[test]
fn test_validate_trail_steps_valid() {
    let trail_steps = vec![
        TrailStep {
            step_order: 0,
            title: Some("Step 1".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "Content 1".to_string(),
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        },
        TrailStep {
            step_order: 1,
            title: Some("Step 2".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "Content 2".to_string(),
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        },
    ];

    assert!(validate_trail_steps(&trail_steps).is_ok());
}

#[test]
fn test_validate_trail_steps_invalid_order() {
    let trail_steps = vec![
        TrailStep {
            step_order: 0,
            title: Some("Step 1".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "Content 1".to_string(),
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        },
        TrailStep {
            step_order: 2, // Skip 1, should be invalid
            title: Some("Step 2".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "Content 2".to_string(),
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        },
    ];

    assert!(validate_trail_steps(&trail_steps).is_err());
}

#[test]
fn test_count_total_words() {
    let trail_steps = vec![
        TrailStep {
            step_order: 0,
            title: Some("Step 1".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "One two three".to_string(), // 3 words
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        },
        TrailStep {
            step_order: 1,
            title: Some("Step 2".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "Four five six seven".to_string(), // 4 words
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        },
    ];

    assert_eq!(count_total_words(&trail_steps), 7);
}
