use shared_types::generated::{ContentNode, DAG};
use shared_types::generated::models::{Choice, Content, EducationalContent};
use shared_types::extensions::content_node::*;
use uuid::Uuid;

#[test]
fn test_calculate_word_count() {
    let node_id = Uuid::now_v7();
    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "This is a test sentence with ten words in it.".to_string(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: None,
    };
    let node = ContentNode {
        id: node_id,
        content,
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    assert_eq!(node.calculate_word_count(), 10);
}

#[test]
fn test_calculate_word_count_empty() {
    let node_id = Uuid::now_v7();
    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "".to_string(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: None,
    };
    let node = ContentNode {
        id: node_id,
        content,
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    assert_eq!(node.calculate_word_count(), 0);
}

#[test]
fn test_validate_choices_valid() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1_id = Uuid::now_v7();
    let node2_id = Uuid::now_v7();

    let choice = Choice {
        id: Uuid::now_v7(),
        text: "Go left".to_string(),
        next_node_id: node2_id,
        metadata: None,
    };

    let content1 = Content {
        content_type: "interactive_story_node".to_string(),
        node_id: node1_id,
        text: "You are at a fork".to_string(),
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
        text: "You went left".to_string(),
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

    dag.add_node(node1.clone());
    dag.add_node(node2);

    assert!(node1.validate_choices(&dag).is_ok());
}

#[test]
fn test_validate_choices_missing_target() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1_id = Uuid::now_v7();
    let node2_id = Uuid::now_v7();

    let choice = Choice {
        id: Uuid::now_v7(),
        text: "Go left".to_string(),
        next_node_id: node2_id, // This node doesn't exist in DAG
        metadata: None,
    };

    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id: node1_id,
        text: "You are at a fork".to_string(),
        choices: vec![choice],
        convergence_point: false,
        next_nodes: vec![node2_id],
        educational_content: None,
    };

    let node = ContentNode {
        id: node1_id,
        content,
        incoming_edges: 0,
        outgoing_edges: 1,
        generation_metadata: None,
    };

    dag.add_node(node.clone());

    assert!(node.validate_choices(&dag).is_err());
}

#[test]
fn test_get_next_nodes() {
    let node_id = Uuid::now_v7();
    let next1 = Uuid::now_v7();
    let next2 = Uuid::now_v7();

    let choices = vec![
        Choice {
            id: Uuid::now_v7(),
            text: "Choice 1".to_string(),
            next_node_id: next1,
            metadata: None,
        },
        Choice {
            id: Uuid::now_v7(),
            text: "Choice 2".to_string(),
            next_node_id: next2,
            metadata: None,
        },
    ];

    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "Choose your path".to_string(),
        choices,
        convergence_point: false,
        next_nodes: vec![next1, next2],
        educational_content: None,
    };

    let node = ContentNode {
        id: node_id,
        content,
        incoming_edges: 0,
        outgoing_edges: 2,
        generation_metadata: None,
    };

    let next_nodes = node.get_next_nodes();
    assert_eq!(next_nodes.len(), 2);
    assert!(next_nodes.contains(&next1));
    assert!(next_nodes.contains(&next2));
}

#[test]
fn test_is_leaf_node() {
    let node_id = Uuid::now_v7();
    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "The end".to_string(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: None,
    };

    let leaf_node = ContentNode {
        id: node_id,
        content,
        incoming_edges: 1,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    assert!(leaf_node.is_leaf_node());

    // Non-leaf node
    let node_id2 = Uuid::now_v7();
    let next = Uuid::now_v7();
    let choice = Choice {
        id: Uuid::now_v7(),
        text: "Continue".to_string(),
        next_node_id: next,
        metadata: None,
    };

    let content2 = Content {
        content_type: "interactive_story_node".to_string(),
        node_id: node_id2,
        text: "Continue?".to_string(),
        choices: vec![choice],
        convergence_point: false,
        next_nodes: vec![next],
        educational_content: None,
    };

    let non_leaf = ContentNode {
        id: node_id2,
        content: content2,
        incoming_edges: 1,
        outgoing_edges: 1,
        generation_metadata: None,
    };

    assert!(!non_leaf.is_leaf_node());
}

#[test]
fn test_has_educational_content() {
    let node_id = Uuid::now_v7();

    // With educational content
    let edu_content = EducationalContent {
        topic: Some("Science".to_string()),
        learning_objective: Some("Learn about photosynthesis".to_string()),
        vocabulary_words: Some(vec!["chlorophyll".to_string(), "carbon dioxide".to_string()]),
        educational_facts: Some(vec!["Plants produce oxygen".to_string()]),
    };

    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "Learn about plants".to_string(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: Some(edu_content),
    };

    let node = ContentNode {
        id: node_id,
        content,
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    assert!(node.has_educational_content());

    // Without educational content
    let node_id2 = Uuid::now_v7();
    let content2 = Content {
        content_type: "interactive_story_node".to_string(),
        node_id: node_id2,
        text: "Just a story".to_string(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: None,
    };

    let node2 = ContentNode {
        id: node_id2,
        content: content2,
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    assert!(!node2.has_educational_content());
}
