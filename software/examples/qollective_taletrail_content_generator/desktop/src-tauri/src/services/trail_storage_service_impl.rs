use async_trait::async_trait;
use crate::error::{AppError, AppResult};
use crate::models::{GenerationResponse, TrailListItem};
use crate::services::traits::TrailStorageService;
use crate::utils::FileLoader;
use std::fs;
use std::path::Path;

/// Concrete implementation of TrailStorageService that handles trail file operations
pub struct TrailStorageServiceImpl;

impl TrailStorageServiceImpl {
    /// Create a new TrailStorageServiceImpl
    pub fn new() -> Self {
        Self
    }
}

impl Default for TrailStorageServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TrailStorageService for TrailStorageServiceImpl {
    async fn load_trail(&self, file_path: &str) -> AppResult<GenerationResponse> {
        FileLoader::load_trail_full(file_path)
            .map_err(|e| AppError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    async fn load_trails_from_directory(
        &self,
        directory: &str,
    ) -> AppResult<Vec<TrailListItem>> {
        FileLoader::load_trails_from_directory(directory)
            .map_err(|e| AppError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    async fn delete_trail(&self, file_path: &str) -> AppResult<()> {
        // Validate file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "File does not exist: {}",
                file_path
            )));
        }

        // Validate it's a file (not a directory)
        if !path.is_file() {
            return Err(AppError::ValidationError(format!(
                "Path is not a file: {}",
                file_path
            )));
        }

        // Delete file
        fs::remove_file(file_path).map_err(|e| AppError::IoError(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_file(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_load_trails_from_directory() {
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
                        "text": "{\"generation_response\":{\"request_id\":\"test-uuid-456\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Service Test Trail\",\"description\":\"Testing service\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"10-12\",\"theme\":\"Mystery\",\"language\":\"en\",\"node_count\":7},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        create_test_file(temp_dir.path(), "response_test.json", valid_content);

        let service = TrailStorageServiceImpl::new();
        let result = service
            .load_trails_from_directory(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let trails = result.unwrap();
        assert_eq!(trails.len(), 1);
        assert_eq!(trails[0].title, "Service Test Trail");
    }

    #[tokio::test]
    async fn test_load_trail_full() {
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
                        "text": "{\"generation_response\":{\"request_id\":\"test-uuid-789\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Full Trail Service Test\",\"description\":\"Testing full load via service\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"6-8\",\"theme\":\"Fantasy\",\"language\":\"en\",\"node_count\":3},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        let file_path = create_test_file(temp_dir.path(), "response_full.json", valid_content);

        let service = TrailStorageServiceImpl::new();
        let result = service.load_trail(file_path.to_str().unwrap()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(
            response.trail.as_ref().unwrap().title,
            "Full Trail Service Test"
        );
    }

    #[tokio::test]
    async fn test_delete_trail_success() {
        let temp_dir = tempfile::tempdir().unwrap();

        let test_content = r#"{"test": "data"}"#;
        let file_path = create_test_file(temp_dir.path(), "to_delete.json", test_content);

        // Verify file exists
        assert!(file_path.exists());

        let service = TrailStorageServiceImpl::new();
        let result = service.delete_trail(file_path.to_str().unwrap()).await;

        // Verify deletion was successful
        assert!(result.is_ok());

        // Verify file no longer exists
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_delete_trail_nonexistent_file() {
        let service = TrailStorageServiceImpl::new();
        let result = service
            .delete_trail("/tmp/qollective_nonexistent_12345.json")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => {
                assert!(msg.contains("File does not exist"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_trail_directory_error() {
        let temp_dir = tempfile::tempdir().unwrap();

        let service = TrailStorageServiceImpl::new();
        let result = service
            .delete_trail(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError(msg) => {
                assert!(msg.contains("Path is not a file"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
