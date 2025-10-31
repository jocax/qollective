use crate::nats::{NatsClient, NatsConfig, RequestTracker};
use tauri::{AppHandle, Emitter, Manager};
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::StreamExt;

/// State for managing NATS subscriptions
pub struct NatsState {
    client: Arc<RwLock<Option<NatsClient>>>,
}

impl NatsState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
        }
    }

    /// Get access to the client RwLock for internal use by commands
    pub(crate) fn client(&self) -> &Arc<RwLock<Option<NatsClient>>> {
        &self.client
    }
}

impl Default for NatsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize NATS connection for MCP requests
///
/// Creates and connects a NATS client using configuration from AppConfig.
/// Stores the client in NatsState for use by send_mcp_request command.
///
/// This should be called on app startup or when reconnection is needed.
#[tauri::command]
pub async fn connect_nats_for_mcp(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let state = app.state::<NatsState>();
    let mut client_guard = state.client().write().await;

    // If already connected, return early
    if client_guard.is_some() {
        return Ok(());
    }

    // Create NATS config from app config
    let app_config = app.state::<crate::config::AppConfig>();
    let nats_config = crate::nats::NatsConfig::from_app_config(&app_config);

    // Create new client
    let client = crate::nats::NatsClient::new(nats_config);

    // Connect to NATS
    client
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to NATS for MCP: {}", e))?;

    // Store in state
    *client_guard = Some(client);

    eprintln!("[TaleTrail] NATS connected for MCP requests");
    Ok(())
}

/// Subscribe to generation events from NATS
///
/// This command establishes a NATS subscription to monitor real-time generation events
/// from the TaleTrail content pipeline. Events are forwarded to the frontend via Tauri's
/// event system.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `tenant_id` - Optional tenant ID to filter events. If None, subscribes to all events.
///
/// # Frontend Event
/// Emits events with name "generation-event" containing GenerationEvent payloads
#[tauri::command]
pub async fn subscribe_to_generations(
    app: AppHandle,
    tenant_id: Option<String>,
) -> Result<(), String> {
    // Get or create NATS client
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
        new_client.connect().await.map_err(|e| e.to_string())?;

        *client_guard = Some(new_client.clone());
        new_client
    };

    // Drop the write lock before subscribing
    drop(client_guard);

    // Subscribe to events
    let subscriber = client
        .subscribe(tenant_id.clone())
        .await
        .map_err(|e| format!("Failed to subscribe: {}", e))?;

    // Store subscriber reference
    client.set_subscriber(subscriber).await;

    // Get the subscriber back for processing (we need to move it into the task)
    let client_guard_write = state.client().write().await;
    if let Some(ref client_ref) = *client_guard_write {
        // Create a new subscription for the background task
        let tenant_id_for_log = tenant_id.clone();
        let mut task_subscriber = client_ref
            .subscribe(tenant_id.clone())
            .await
            .map_err(|e| format!("Failed to create task subscriber: {}", e))?;

        drop(client_guard_write);

        // Log subscription details
        let subject_pattern = if let Some(ref tid) = tenant_id_for_log {
            format!("taletrail.generation.events.{}", tid)
        } else {
            "taletrail.generation.events.*".to_string()
        };
        eprintln!("[NATS DEBUG] Subscribed to subject pattern: {}", subject_pattern);
        eprintln!("[NATS DEBUG] Waiting for messages on subscription...");

        // Get request tracker from app state
        let tracker = app.state::<RequestTracker>();
        let tracker_clone = tracker.inner().clone();
        let app_handle = app.clone();

        // Spawn background task to process messages
        tauri::async_runtime::spawn(async move {
            let mut message_count = 0usize;
            eprintln!("[NATS DEBUG] Background task started, entering message loop");

            while let Some(message) = task_subscriber.next().await {
                message_count += 1;
                eprintln!("[NATS DEBUG] Received message #{} on subject: {}", message_count, message.subject);
                eprintln!("[NATS DEBUG] Payload size: {} bytes", message.payload.len());

                // Parse the event
                match NatsClient::parse_event(&message.payload) {
                    Ok(event) => {
                        eprintln!("[NATS DEBUG] Successfully parsed event:");
                        eprintln!("  - event_type: {}", event.event_type);
                        eprintln!("  - tenant_id: {}", event.tenant_id);
                        eprintln!("  - request_id: {}", event.request_id);
                        eprintln!("  - service_phase: {}", event.service_phase);
                        eprintln!("  - status: {}", event.status);
                        eprintln!("  - progress: {:?}", event.progress);
                        eprintln!("  - file_path: {:?}", event.file_path);
                        eprintln!("  - timestamp: {}", event.timestamp);

                        // Update request tracker from event
                        tracker_clone.update_from_event(&event).await;
                        eprintln!("[NATS DEBUG] Updated request tracker for request: {}", event.request_id);

                        // Emit event to frontend
                        if let Err(e) = app_handle.emit("generation-event", &event) {
                            eprintln!("[NATS ERROR] Failed to emit generation event to frontend: {}", e);
                        } else {
                            eprintln!("[NATS DEBUG] Successfully emitted event to frontend");
                        }
                    }
                    Err(e) => {
                        eprintln!("[NATS ERROR] Failed to parse NATS message: {}", e);
                        eprintln!("[NATS ERROR] Raw payload: {}", String::from_utf8_lossy(&message.payload));
                    }
                }
            }

            eprintln!("[NATS DEBUG] Message loop ended after {} messages", message_count);
        });

        Ok(())
    } else {
        Err("Failed to access NATS client".to_string())
    }
}

/// Unsubscribe from generation events
///
/// This command cleanly closes the NATS subscription and releases resources.
///
/// # Arguments
/// * `app` - Tauri application handle
#[tauri::command]
pub async fn unsubscribe_from_generations(app: AppHandle) -> Result<(), String> {
    let state = app.state::<NatsState>();
    let client_guard = state.client().read().await;

    if let Some(client) = client_guard.as_ref() {
        client
            .unsubscribe()
            .await
            .map_err(|e| format!("Failed to unsubscribe: {}", e))?;
    }

    Ok(())
}

/// Connection status information
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    /// Whether the client is connected
    pub connected: bool,
    /// NATS server URL
    pub url: String,
    /// Number of active tracked requests
    pub active_requests: usize,
}

/// Get NATS connection status
///
/// Returns detailed connection status including server URL and active request count.
///
/// # Arguments
/// * `app` - Tauri application handle
#[tauri::command]
pub async fn nats_connection_status(app: AppHandle) -> Result<ConnectionStatus, String> {
    let state = app.state::<NatsState>();
    let client_guard = state.client().read().await;

    let app_config = app.state::<crate::config::AppConfig>();
    let tracker = app.state::<RequestTracker>();

    let connected = if let Some(client) = client_guard.as_ref() {
        client.is_connected().await
    } else {
        false
    };

    Ok(ConnectionStatus {
        connected,
        url: app_config.nats.url.clone(),
        active_requests: tracker.count().await,
    })
}

/// Disconnect from NATS server
///
/// Cleanly disconnects from the NATS server and releases all resources.
///
/// # Arguments
/// * `app` - Tauri application handle
#[tauri::command]
pub async fn disconnect_nats(app: AppHandle) -> Result<(), String> {
    let state = app.state::<NatsState>();
    let mut client_guard = state.client().write().await;

    if let Some(client) = client_guard.take() {
        client
            .disconnect()
            .await
            .map_err(|e| format!("Failed to disconnect: {}", e))?;
    }

    Ok(())
}

/// Get all active generation requests being tracked
///
/// Returns a list of currently tracked generation requests with their metadata,
/// including request_id, tenant_id, start_time, current_phase, and progress.
///
/// # Arguments
/// * `app` - Tauri application handle
///
/// # Returns
/// A list of active request tracking information
#[tauri::command]
pub async fn get_active_requests(
    app: AppHandle,
) -> Result<Vec<crate::nats::TrackedRequest>, String> {
    let tracker = app.state::<RequestTracker>();
    Ok(tracker.get_active_requests().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_state_creation() {
        let _state = NatsState::new();
        // State is created successfully
        assert!(true);
    }

    #[test]
    fn test_nats_state_default() {
        let _state = NatsState::default();
        // Default state is created successfully
        assert!(true);
    }
}
