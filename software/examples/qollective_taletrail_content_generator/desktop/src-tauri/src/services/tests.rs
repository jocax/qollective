#[cfg(test)]
mod request_service_tests {
    use crate::error::AppResult;
    use crate::models::request::{AgeGroup, GenerationRequest, Language, VocabularyLevel};
    use crate::services::traits::RequestService;
    use mockall::predicate::*;
    use mockall::mock;

    fn init_crypto_provider() {
        // Initialize rustls crypto provider for tests
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }

    // Mock implementation for testing
    mock! {
        pub NatsClient {
            pub async fn connect(&self) -> AppResult<()>;
            pub async fn is_connected(&self) -> bool;
            pub async fn publish_request(&self, request: &GenerationRequest) -> AppResult<()>;
        }

        impl Clone for NatsClient {
            fn clone(&self) -> Self;
        }
    }

    fn create_valid_request() -> GenerationRequest {
        GenerationRequest {
            request_id: "req-test-mock-123".to_string(),
            tenant_id: "1".to_string(),
            theme: "Space Adventure Mock".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        }
    }

    #[tokio::test]
    async fn test_request_service_with_mock_validates_empty_theme() {
        init_crypto_provider();
        // This test demonstrates that validation works even with mocked NATS client
        use crate::nats::{NatsClient, NatsConfig};
        use crate::services::RequestServiceImpl;
        use crate::config::AppConfig;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        let mut invalid_request = create_valid_request();
        invalid_request.theme = "".to_string();

        let result = service.submit_request(invalid_request).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Validation"));
    }

    #[tokio::test]
    async fn test_request_service_validates_empty_request_id() {
        init_crypto_provider();
        use crate::nats::{NatsClient, NatsConfig};
        use crate::services::RequestServiceImpl;
        use crate::config::AppConfig;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        let mut invalid_request = create_valid_request();
        invalid_request.request_id = "".to_string();

        let result = service.submit_request(invalid_request).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Validation"));
    }

    #[tokio::test]
    async fn test_request_service_validates_node_count_too_low() {
        init_crypto_provider();
        use crate::nats::{NatsClient, NatsConfig};
        use crate::services::RequestServiceImpl;
        use crate::config::AppConfig;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        let mut invalid_request = create_valid_request();
        invalid_request.node_count = 0; // Below minimum

        let result = service.submit_request(invalid_request).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("node_count"));
    }

    #[tokio::test]
    async fn test_replay_request_validation_failure() {
        init_crypto_provider();
        use crate::nats::{NatsClient, NatsConfig};
        use crate::services::RequestServiceImpl;
        use crate::config::AppConfig;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        let mut original_request = create_valid_request();
        // Make invalid
        original_request.theme = "".to_string();
        let new_id = "req-replay-789".to_string();

        // Should fail validation
        let result = service.replay_request(original_request, new_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Validation"));
    }
}

#[cfg(test)]
mod trail_storage_service_tests {
    use crate::services::TrailStorageServiceImpl;
    use crate::services::traits::TrailStorageService;
    use std::fs;

    fn create_test_file(dir: &std::path::Path, filename: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_trail_storage_service_loads_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let valid_content = r#"{
            "meta": {
                "request_id": "test-service-uuid-123",
                "timestamp": "2025-10-22T13:00:00Z",
                "tenant": "1",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-service-uuid-123\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Service Integration Test\",\"description\":\"Testing service layer\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"10-12\",\"theme\":\"Mystery\",\"language\":\"en\",\"node_count\":7},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        create_test_file(temp_dir.path(), "response_service_test.json", valid_content);

        let service = TrailStorageServiceImpl::new();
        let result = service.load_trails_from_directory(temp_dir.path().to_str().unwrap()).await;

        assert!(result.is_ok());
        let trails = result.unwrap();
        assert_eq!(trails.len(), 1);
        assert_eq!(trails[0].title, "Service Integration Test");
        assert_eq!(trails[0].theme, "Mystery");
        assert_eq!(trails[0].node_count, 7);
    }

    #[tokio::test]
    async fn test_trail_storage_service_load_full_trail() {
        let temp_dir = tempfile::tempdir().unwrap();

        let valid_content = r#"{
            "meta": {
                "request_id": "test-full-uuid-456",
                "timestamp": "2025-10-22T14:00:00Z",
                "tenant": "2",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-full-uuid-456\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Full Trail Service Test\",\"description\":\"Complete trail data\",\"is_public\":true,\"status\":\"PUBLISHED\",\"tags\":[\"test\",\"integration\"],\"metadata\":{\"generation_params\":{\"age_group\":\"12-14\",\"theme\":\"Sci-Fi\",\"language\":\"de\",\"node_count\":15},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        let file_path = create_test_file(temp_dir.path(), "response_full_trail.json", valid_content);

        let service = TrailStorageServiceImpl::new();
        let result = service.load_trail(file_path.to_str().unwrap()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.trail.as_ref().unwrap().title, "Full Trail Service Test");
        assert_eq!(response.trail.as_ref().unwrap().is_public, true);
        assert_eq!(response.trail.as_ref().unwrap().tags, Some(vec!["test".to_string(), "integration".to_string()]));
    }

    #[tokio::test]
    async fn test_trail_storage_service_delete_trail() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = create_test_file(temp_dir.path(), "to_delete.json", r#"{"test":"data"}"#);

        assert!(file_path.exists());

        let service = TrailStorageServiceImpl::new();
        let result = service.delete_trail(file_path.to_str().unwrap()).await;

        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_trail_storage_service_delete_nonexistent_file() {
        let service = TrailStorageServiceImpl::new();
        let result = service.delete_trail("/tmp/nonexistent_qollective_test_file_999999.json").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not Found"));
    }

    #[tokio::test]
    async fn test_trail_storage_service_delete_directory_fails() {
        let temp_dir = tempfile::tempdir().unwrap();

        let service = TrailStorageServiceImpl::new();
        let result = service.delete_trail(temp_dir.path().to_str().unwrap()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Validation"));
    }

    #[tokio::test]
    async fn test_trail_storage_service_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let service = TrailStorageServiceImpl::new();
        let result = service.load_trails_from_directory(temp_dir.path().to_str().unwrap()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
