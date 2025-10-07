use shared_types::generated::{ContentNode, DAG, Edge};
use shared_types::generated::models::{Choice, Content};
use shared_types::extensions::dag::*;
use uuid::Uuid;

#[test]
fn test_add_node() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node_id = Uuid::now_v7();
    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "Test node".to_string(),
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

    dag.add_node(node.clone());
    assert_eq!(dag.nodes.len(), 1);
    assert!(dag.nodes.contains_key(&node_id));
}

#[test]
fn test_add_edge() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let from_id = Uuid::now_v7();
    let to_id = Uuid::now_v7();

    // Add nodes first
    for id in &[from_id, to_id] {
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id: *id,
            text: "Test".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let node = ContentNode {
            id: *id,
            content,
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        };
        dag.add_node(node);
    }

    let choice_id = Uuid::now_v7();
    let edge = Edge {
        from_node_id: from_id,
        to_node_id: to_id,
        choice_id,
        weight: Some(1.0),
    };

    dag.add_edge(edge.clone()).expect("Failed to add edge");
    assert_eq!(dag.edges.len(), 1);
    assert_eq!(dag.nodes.get(&from_id).unwrap().outgoing_edges, 1);
    assert_eq!(dag.nodes.get(&to_id).unwrap().incoming_edges, 1);
}

#[test]
fn test_add_edge_missing_node() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let from_id = Uuid::now_v7();
    let to_id = Uuid::now_v7();
    let choice_id = Uuid::now_v7();

    let edge = Edge {
        from_node_id: from_id,
        to_node_id: to_id,
        choice_id,
        weight: Some(1.0),
    };

    let result = dag.add_edge(edge);
    assert!(result.is_err());
}

#[test]
fn test_validate_structure_valid() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node_id = Uuid::now_v7();
    dag.start_node_id = node_id;

    let content = Content {
        content_type: "interactive_story_node".to_string(),
        node_id,
        text: "Start".to_string(),
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
    dag.add_node(node);

    assert!(dag.validate_structure().is_ok());
}

#[test]
fn test_validate_structure_missing_start_node() {
    let dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let result = dag.validate_structure();
    assert!(result.is_err());
}

#[test]
fn test_find_paths() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1 = Uuid::now_v7();
    let node2 = Uuid::now_v7();
    let node3 = Uuid::now_v7();

    dag.start_node_id = node1;

    // Add nodes
    for id in &[node1, node2, node3] {
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id: *id,
            text: "Test".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let node = ContentNode {
            id: *id,
            content,
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        };
        dag.add_node(node);
    }

    // Add edges: node1 -> node2 -> node3
    let edge1 = Edge {
        from_node_id: node1,
        to_node_id: node2,
        choice_id: Uuid::now_v7(),
        weight: Some(1.0),
    };
    let edge2 = Edge {
        from_node_id: node2,
        to_node_id: node3,
        choice_id: Uuid::now_v7(),
        weight: Some(1.0),
    };

    dag.add_edge(edge1).unwrap();
    dag.add_edge(edge2).unwrap();

    let paths = dag.find_paths(node1, node3);
    assert!(!paths.is_empty());
}

#[test]
fn test_detect_convergence_points() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1 = Uuid::now_v7();
    let node2 = Uuid::now_v7();
    let node3 = Uuid::now_v7();
    let convergence = Uuid::now_v7();

    dag.start_node_id = node1;

    // Add nodes
    for id in &[node1, node2, node3, convergence] {
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id: *id,
            text: "Test".to_string(),
            choices: vec![],
            convergence_point: *id == convergence,
            next_nodes: vec![],
            educational_content: None,
        };
        let node = ContentNode {
            id: *id,
            content,
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        };
        dag.add_node(node);
    }

    // Create two paths that converge: node1->node2->convergence and node1->node3->convergence
    let edges = vec![
        Edge { from_node_id: node1, to_node_id: node2, choice_id: Uuid::now_v7(), weight: Some(1.0) },
        Edge { from_node_id: node1, to_node_id: node3, choice_id: Uuid::now_v7(), weight: Some(1.0) },
        Edge { from_node_id: node2, to_node_id: convergence, choice_id: Uuid::now_v7(), weight: Some(1.0) },
        Edge { from_node_id: node3, to_node_id: convergence, choice_id: Uuid::now_v7(), weight: Some(1.0) },
    ];

    for edge in edges {
        dag.add_edge(edge).unwrap();
    }

    let convergence_points = dag.detect_convergence_points();
    assert_eq!(convergence_points.len(), 1);
    assert!(convergence_points.contains(&convergence));
}

#[test]
fn test_is_reachable() {
    let mut dag = DAG {
        nodes: Default::default(),
        edges: vec![],
        start_node_id: Uuid::now_v7(),
        convergence_points: vec![],
    };

    let node1 = Uuid::now_v7();
    let node2 = Uuid::now_v7();
    let node3 = Uuid::now_v7();

    dag.start_node_id = node1;

    // Add nodes
    for id in &[node1, node2, node3] {
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id: *id,
            text: "Test".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let node = ContentNode {
            id: *id,
            content,
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        };
        dag.add_node(node);
    }

    // Add edges: node1 -> node2
    let edge = Edge {
        from_node_id: node1,
        to_node_id: node2,
        choice_id: Uuid::now_v7(),
        weight: Some(1.0),
    };
    dag.add_edge(edge).unwrap();

    assert!(dag.is_reachable(node1, node2));
    assert!(!dag.is_reachable(node1, node3)); // node3 is not reachable
}
