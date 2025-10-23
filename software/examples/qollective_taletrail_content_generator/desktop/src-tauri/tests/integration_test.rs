use nuxtor_lib::models::{
    GenerationResponse, Trail, TrailMetadata, GenerationParams,
    DAG, ContentNode, NodeContent, Choice, Edge, ServiceInvocation
};
use nuxtor_lib::error::AppError;
use std::collections::HashMap;

#[test]
fn test_generation_response_serialization() {
    // Create a sample generation response
    let generation_params = GenerationParams {
        age_group: "9-11".to_string(),
        theme: "Ocean Adventure".to_string(),
        language: "en".to_string(),
        node_count: 3,
    };

    let trail_metadata = TrailMetadata {
        generation_params,
        start_node_id: "0".to_string(),
    };

    let choice = Choice {
        id: "choice_0_0".to_string(),
        text: "Explore the coral reef".to_string(),
        next_node_id: "1".to_string(),
    };

    let node_content = NodeContent {
        text: "You discover a hidden underwater cave...".to_string(),
        choices: vec![choice],
    };

    let mut nodes = HashMap::new();
    nodes.insert(
        "0".to_string(),
        ContentNode {
            id: "0".to_string(),
            content: node_content,
            generation_metadata: None,
        },
    );

    let edges = vec![Edge {
        from_node_id: "0".to_string(),
        to_node_id: "1".to_string(),
        choice_id: "choice_0_0".to_string(),
    }];

    let dag = DAG {
        nodes,
        edges,
        convergence_points: vec![],
    };

    let trail = Trail {
        title: "Ocean Adventure".to_string(),
        description: "Interactive story for 9-11 age group".to_string(),
        metadata: trail_metadata,
        dag,
    };

    let response = GenerationResponse {
        status: "completed".to_string(),
        trail,
        service_invocations: vec![],
    };

    // Test serialization
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("Ocean Adventure"));
    assert!(json.contains("completed"));

    // Test deserialization
    let deserialized: GenerationResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.status, "completed");
    assert_eq!(deserialized.trail.title, "Ocean Adventure");
}

#[test]
fn test_service_invocation_structure() {
    let invocation = ServiceInvocation {
        service_name: "prompt-helper".to_string(),
        phase: "planning".to_string(),
        success: true,
        duration_ms: 234,
        error_message: None,
    };

    let json = serde_json::to_string(&invocation).unwrap();
    let deserialized: ServiceInvocation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.service_name, "prompt-helper");
    assert_eq!(deserialized.duration_ms, 234);
    assert!(deserialized.success);
}

#[test]
fn test_error_handling() {
    // Test AppError from IO error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_error: AppError = io_error.into();
    assert!(matches!(app_error, AppError::IoError(_)));

    // Test AppError from JSON error
    let invalid_json = "{invalid}";
    let json_result: Result<GenerationResponse, _> = serde_json::from_str(invalid_json);
    assert!(json_result.is_err());

    if let Err(err) = json_result {
        let app_error: AppError = err.into();
        assert!(matches!(app_error, AppError::JsonError(_)));
    }

    // Test validation error
    let validation_err = AppError::ValidationError("Invalid node count".to_string());
    assert!(validation_err.to_string().contains("Validation Error"));

    // Test not found error
    let not_found = AppError::NotFound("Trail not found".to_string());
    assert!(not_found.to_string().contains("Not Found"));
}

#[test]
fn test_dag_structure() {
    let mut nodes = HashMap::new();

    // Create three nodes
    for i in 0..3 {
        let node = ContentNode {
            id: i.to_string(),
            content: NodeContent {
                text: format!("Story node {}", i),
                choices: vec![],
            },
            generation_metadata: None,
        };
        nodes.insert(i.to_string(), node);
    }

    let dag = DAG {
        nodes,
        edges: vec![
            Edge {
                from_node_id: "0".to_string(),
                to_node_id: "1".to_string(),
                choice_id: "choice_0".to_string(),
            },
            Edge {
                from_node_id: "1".to_string(),
                to_node_id: "2".to_string(),
                choice_id: "choice_1".to_string(),
            },
        ],
        convergence_points: vec!["2".to_string()],
    };

    assert_eq!(dag.nodes.len(), 3);
    assert_eq!(dag.edges.len(), 2);
    assert_eq!(dag.convergence_points.len(), 1);
}

#[test]
fn test_user_preferences_default() {
    use nuxtor_lib::models::{UserPreferences, ViewMode, Theme};

    let prefs = UserPreferences::default();
    assert_eq!(prefs.default_view_mode, ViewMode::Linear);
    assert_eq!(prefs.theme, Theme::System);
    assert!(prefs.auto_validate);
}

#[test]
fn test_bookmark_structure() {
    use nuxtor_lib::models::Bookmark;

    let bookmark = Bookmark {
        trail_id: "trail_123".to_string(),
        trail_title: "Test Trail".to_string(),
        file_path: "/path/to/trail.json".to_string(),
        timestamp: "2025-10-22T12:00:00Z".to_string(),
        user_note: "Great story!".to_string(),
        tenant_id: None,
    };

    let json = serde_json::to_string(&bookmark).unwrap();
    let deserialized: Bookmark = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.trail_id, "trail_123");
    assert_eq!(deserialized.user_note, "Great story!");
}

#[test]
fn test_load_real_test_trails() {
    use nuxtor_lib::utils::FileLoader;
    use std::path::PathBuf;

    // Get the path to test-trails directory
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.pop(); // Go up from src-tauri
    test_dir.push("test-trails");

    // Only run if directory exists (may not exist in CI)
    if !test_dir.exists() {
        println!("Test trails directory not found, skipping integration test");
        return;
    }

    let result = FileLoader::load_trails_from_directory(test_dir.to_str().unwrap());

    assert!(result.is_ok(), "Failed to load trails: {:?}", result);

    let trails = result.unwrap();

    // Should have at least the 3 test files we created
    assert!(trails.len() >= 3, "Expected at least 3 trails, got {}", trails.len());

    // Verify some expected data
    let titles: Vec<&str> = trails.iter().map(|t| t.title.as_str()).collect();
    assert!(titles.contains(&"The Enchanted Forest Quest"));
    assert!(titles.contains(&"The Case of the Missing Telescope"));
    assert!(titles.contains(&"Mission to Mars"));

    // Verify age groups
    let age_groups: Vec<&str> = trails.iter().map(|t| t.age_group.as_str()).collect();
    assert!(age_groups.contains(&"8-10"));
    assert!(age_groups.contains(&"10-12"));
    assert!(age_groups.contains(&"12-14"));

    // Verify statuses
    let statuses: Vec<&str> = trails.iter().map(|t| t.status.as_str()).collect();
    assert!(statuses.contains(&"completed"));
    assert!(statuses.contains(&"partial"));

    // Verify themes
    let themes: Vec<&str> = trails.iter().map(|t| t.theme.as_str()).collect();
    assert!(themes.contains(&"Fantasy Adventure"));
    assert!(themes.contains(&"Mystery Detective"));
    assert!(themes.contains(&"Space Exploration"));

    println!("Successfully loaded {} trails", trails.len());
    for trail in &trails {
        println!("  - {} ({}, {}, {} nodes)",
                 trail.title, trail.theme, trail.status, trail.node_count);
    }
}

#[test]
fn test_load_full_trail_real_file() {
    use nuxtor_lib::utils::FileLoader;
    use std::path::PathBuf;

    let mut test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file.pop();
    test_file.push("test-trails/response_test_adventure.json");

    // Only run if file exists
    if !test_file.exists() {
        println!("Test file not found, skipping integration test");
        return;
    }

    let result = FileLoader::load_trail_full(test_file.to_str().unwrap());

    assert!(result.is_ok(), "Failed to load trail: {:?}", result);

    let trail = result.unwrap();

    assert_eq!(trail.status, "completed");
    assert_eq!(trail.trail.title, "The Enchanted Forest Quest");
    assert_eq!(trail.trail.metadata.generation_params.age_group, "8-10");
    assert_eq!(trail.trail.metadata.generation_params.theme, "Fantasy Adventure");
    assert_eq!(trail.trail.metadata.generation_params.node_count, 12);

    // Verify DAG structure
    assert!(trail.trail.dag.nodes.contains_key("node_start"));
    assert!(trail.trail.dag.nodes.contains_key("node_mushrooms"));
    assert!(trail.trail.dag.nodes.contains_key("node_brook"));

    println!("Successfully loaded full trail: {}", trail.trail.title);
}
