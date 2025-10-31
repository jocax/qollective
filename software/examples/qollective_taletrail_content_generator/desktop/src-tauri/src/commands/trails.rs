use crate::models::{GenerationResponse, TrailListItem};
use crate::services::{TrailStorageService, TrailStorageServiceImpl};

/// Load trail metadata from a directory
///
/// Scans the directory recursively for response_*.json files and returns
/// a list of trail metadata items
#[tauri::command]
pub async fn load_trails_from_directory(directory: String) -> Result<Vec<TrailListItem>, String> {
    let service = TrailStorageServiceImpl::new();
    service
        .load_trails_from_directory(&directory)
        .await
        .map_err(|e| e.to_string())
}

/// Load full trail data from a file
///
/// Reads and parses the complete GenerationResponse from a trail file
#[tauri::command]
pub async fn load_trail_full(file_path: String) -> Result<GenerationResponse, String> {
    let service = TrailStorageServiceImpl::new();
    service
        .load_trail(&file_path)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a trail file from the filesystem
///
/// Removes the specified trail JSON file
#[tauri::command]
pub async fn delete_trail(file_path: String) -> Result<(), String> {
    let service = TrailStorageServiceImpl::new();
    service
        .delete_trail(&file_path)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_file(dir: &std::path::Path, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_load_trails_command() {
        let temp_dir = tempfile::tempdir().unwrap();

        let valid_content = r#"{
            "meta": {
                "request_id": "test-uuid-456",
                "timestamp": "2025-10-22T13:00:00Z",
                "tenant": "1",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-uuid-456\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Command Test Trail\",\"description\":\"Testing commands\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"10-12\",\"theme\":\"Mystery\",\"language\":\"en\",\"node_count\":7},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        create_test_file(temp_dir.path(), "response_test_cmd.json", valid_content);

        let result = load_trails_from_directory(temp_dir.path().to_str().unwrap().to_string()).await;

        assert!(result.is_ok());
        let trails = result.unwrap();
        assert_eq!(trails.len(), 1);
        assert_eq!(trails[0].title, "Command Test Trail");
    }

    #[tokio::test]
    async fn test_load_trail_full_command() {
        let temp_dir = tempfile::tempdir().unwrap();

        let valid_content = r#"{
            "meta": {
                "request_id": "test-uuid-789",
                "timestamp": "2025-10-22T14:00:00Z",
                "tenant": "1",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-uuid-789\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Full Trail Test\",\"description\":\"Testing full load\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"6-8\",\"theme\":\"Fantasy\",\"language\":\"en\",\"node_count\":3},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        let file_path = create_test_file(temp_dir.path(), "response_full_test.json", valid_content);

        let result = load_trail_full(file_path.to_str().unwrap().to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.trail.as_ref().unwrap().title, "Full Trail Test");
        use crate::models::GenerationStatus;
        assert_eq!(response.status, GenerationStatus::Completed);
    }

    #[tokio::test]
    async fn test_delete_trail_success() {
        let temp_dir = tempfile::tempdir().unwrap();

        let test_content = r#"{
            "meta": {
                "request_id": "test-delete-uuid",
                "timestamp": "2025-10-22T15:00:00Z",
                "tenant": "1",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-delete-uuid\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Delete Test Trail\",\"description\":\"To be deleted\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"10-12\",\"theme\":\"Adventure\",\"language\":\"en\",\"node_count\":5},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        let file_path = create_test_file(temp_dir.path(), "response_to_delete.json", test_content);

        // Verify file exists before deletion
        assert!(file_path.exists());

        // Delete the file
        let result = delete_trail(file_path.to_str().unwrap().to_string()).await;

        // Verify deletion was successful
        assert!(result.is_ok());

        // Verify file no longer exists
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_delete_trail_non_existent_file() {
        let non_existent_path = "/tmp/qollective_test_non_existent_file_12345.json";

        let result = delete_trail(non_existent_path.to_string()).await;

        // Should fail with appropriate error message
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("File does not exist"));
        assert!(error.contains(non_existent_path));
    }

    #[tokio::test]
    async fn test_delete_trail_directory_error() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = delete_trail(temp_dir.path().to_str().unwrap().to_string()).await;

        // Should fail because path is a directory, not a file
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Path is not a file"));
    }
}
