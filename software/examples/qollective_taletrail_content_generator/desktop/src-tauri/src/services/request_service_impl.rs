use async_trait::async_trait;
use crate::error::AppResult;
use crate::models::{GenerationRequest, RequestMetadata};
use crate::services::traits::RequestService;
use crate::nats::NatsClient;

/// Concrete implementation of RequestService that handles generation request operations
pub struct RequestServiceImpl {
    nats_client: NatsClient,
}

impl RequestServiceImpl {
    /// Create a new RequestServiceImpl with the provided NATS client
    pub fn new(nats_client: NatsClient) -> Self {
        Self { nats_client }
    }
}

#[async_trait]
impl RequestService for RequestServiceImpl {
    async fn submit_request(&self, mut request: GenerationRequest) -> AppResult<String> {
        // Ensure metadata exists with current timestamp
        if request.metadata.is_none() {
            request.metadata = Some(RequestMetadata::new());
        }

        // Validate the request
        request.validate()?;

        // Ensure NATS client is connected
        if !self.nats_client.is_connected().await {
            self.nats_client.connect().await?;
        }

        // Publish the request to NATS
        self.nats_client.publish_request(&request).await?;

        // Return the request_id for tracking
        Ok(request.request_id.clone())
    }

    async fn replay_request(
        &self,
        mut original_request: GenerationRequest,
        new_request_id: String,
    ) -> AppResult<String> {
        // Store the original request ID
        let original_id = original_request.request_id.clone();

        // Update request with new ID
        original_request.request_id = new_request_id.clone();

        // Create new metadata marking this as a replay
        let metadata = if let Some(mut existing_metadata) = original_request.metadata {
            existing_metadata.submitted_at = chrono::Utc::now().to_rfc3339();
            existing_metadata.original_request_id = Some(original_id);
            existing_metadata
        } else {
            RequestMetadata::new().with_original(original_id)
        };

        original_request.metadata = Some(metadata);

        // Submit the modified request
        self.submit_request(original_request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::{AgeGroup, Language, VocabularyLevel};

    fn init_crypto_provider() {
        // Initialize rustls crypto provider for tests
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }

    fn create_valid_request() -> GenerationRequest {
        GenerationRequest {
            request_id: "req-test-123".to_string(),
            tenant_id: "1".to_string(),
            theme: "Space Adventure".to_string(),
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
    async fn test_submit_request_validates_request() {
        init_crypto_provider();
        use crate::config::AppConfig;
        use crate::nats::NatsConfig;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        let mut invalid_request = create_valid_request();
        invalid_request.theme = "".to_string(); // Invalid empty theme

        let result = service.submit_request(invalid_request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Validation Error"));
    }

    #[tokio::test]
    async fn test_submit_request_validates_node_count_bounds() {
        init_crypto_provider();
        use crate::config::AppConfig;
        use crate::nats::NatsConfig;
        use crate::constants::validation;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        // Test node_count too high
        let mut request = create_valid_request();
        request.node_count = validation::MAX_NODE_COUNT + 1;

        let result = service.submit_request(request).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("node_count"));
    }

    #[tokio::test]
    async fn test_replay_request_validation() {
        init_crypto_provider();
        use crate::config::AppConfig;
        use crate::nats::NatsConfig;

        let app_config = AppConfig::create_test_app_config();
        let config = NatsConfig::from_app_config(&app_config);
        let nats_client = NatsClient::new(config);
        let service = RequestServiceImpl::new(nats_client);

        let mut original_request = create_valid_request();
        // Make request invalid
        original_request.theme = "".to_string();
        let new_id = "req-replay-456".to_string();

        // Should fail validation before attempting to connect
        let result = service.replay_request(original_request, new_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Validation"));
    }
}
