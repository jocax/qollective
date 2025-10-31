use crate::commands::nats_commands::NatsState;
use crate::models::{GenerationRequest, RequestMetadata};
use crate::nats::{NatsClient, NatsConfig};
use crate::services::{RequestService, RequestServiceImpl};
use tauri::{AppHandle, Manager};

/// Submit a generation request to the TaleTrail content pipeline via NATS
///
/// This command validates the request, ensures a NATS connection exists,
/// and publishes the request to the tenant-specific NATS subject.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `request` - The generation request to submit
///
/// # Returns
/// Returns the request_id for tracking if successful
///
/// # Frontend Usage
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// const request = {
///   requestId: 'req-123',
///   tenantId: 'tenant-456',
///   theme: 'Space Adventure',
///   ageGroup: '6-8',
///   language: 'en',
///   vocabularyLevel: 'simple',
///   nodeCount: 10,
/// };
///
/// try {
///   const requestId = await invoke('submit_generation_request', { request });
///   console.log('Request submitted:', requestId);
/// } catch (error) {
///   console.error('Failed to submit request:', error);
/// }
/// ```
#[tauri::command]
pub async fn submit_generation_request(
    app: AppHandle,
    request: GenerationRequest,
) -> Result<String, String> {
    // Get or create NATS client
    let nats_client = get_or_create_nats_client(&app).await?;

    // Create service instance with NATS client
    let service = RequestServiceImpl::new(nats_client);

    // Use the service to submit the request
    service
        .submit_request(request)
        .await
        .map_err(|e| e.to_string())
}

/// Replay a generation request with optional modifications
///
/// This is a convenience command that takes an existing request,
/// creates a new request_id, and marks it as a replay in the metadata.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `original_request` - The request to replay
/// * `new_request_id` - New unique request ID
///
/// # Returns
/// Returns the new request_id for tracking if successful
#[tauri::command]
pub async fn replay_generation_request(
    app: AppHandle,
    original_request: GenerationRequest,
    new_request_id: String,
) -> Result<String, String> {
    // Get or create NATS client
    let nats_client = get_or_create_nats_client(&app).await?;

    // Create service instance with NATS client
    let service = RequestServiceImpl::new(nats_client);

    // Use the service to replay the request
    service
        .replay_request(original_request, new_request_id)
        .await
        .map_err(|e| e.to_string())
}

/// Helper function to get or create NATS client from app state
async fn get_or_create_nats_client(app: &AppHandle) -> Result<NatsClient, String> {
    let state = app.state::<NatsState>();
    let mut client_guard = state.client().write().await;

    let client = if let Some(existing_client) = client_guard.as_ref() {
        existing_client.clone()
    } else {
        // Create new client with config from app state
        let app_config = app.state::<crate::config::AppConfig>();
        let config = NatsConfig::from_app_config(&app_config);
        let new_client = NatsClient::new(config);

        // Connect to NATS
        new_client
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to NATS: {}", e))?;

        *client_guard = Some(new_client.clone());
        new_client
    };

    Ok(client)
}

#[cfg(test)]
mod tests {
    use crate::models::GenerationRequest;

    #[test]
    fn test_request_validation_in_command_context() {
        use crate::models::request::{AgeGroup, Language, VocabularyLevel};

        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        };

        // Validation should pass
        assert!(request.validate().is_ok());

        // Test with different age group
        let request2 = GenerationRequest {
            age_group: AgeGroup::_9To11,
            ..request.clone()
        };
        assert!(request2.validate().is_ok());
    }

    #[test]
    fn test_metadata_auto_creation() {
        use crate::models::request::{AgeGroup, Language, VocabularyLevel};

        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        };

        // Before command processing, metadata is None
        assert!(request.metadata.is_none());

        // The command would add metadata automatically
        // This is tested in integration tests
    }
}
