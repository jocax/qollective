/// Tauri commands for NATS monitoring
///
/// Provides frontend access to real-time NATS message monitoring.

use crate::config::AppConfig;
use crate::nats::monitoring;

/// Start NATS monitoring
///
/// Connects to NATS using configuration and subscribes to wildcard subjects.
/// Emits messages to frontend via "nats-message" event.
///
/// This is called automatically on app startup, but can also be called manually
/// to reconnect after disconnection.
#[tauri::command]
pub async fn start_nats_monitoring(
    app_handle: tauri::AppHandle,
    config: tauri::State<'_, AppConfig>,
) -> Result<(), String> {
    let nats_url = config.nats.url.clone();
    let ca_cert_path = config.ca_cert_path();
    let nkey_path = config.nkey_path();

    monitoring::start_monitoring(nats_url, app_handle, ca_cert_path, nkey_path)
        .await
        .map_err(|e| format!("Failed to start NATS monitoring: {}", e))
}

/// Stop NATS monitoring
///
/// Disconnects from NATS and stops message emission.
#[tauri::command]
pub async fn stop_nats_monitoring() -> Result<(), String> {
    monitoring::stop_monitoring()
        .await
        .map_err(|e| format!("Failed to stop NATS monitoring: {}", e))
}

/// Get NATS monitoring connection status
///
/// Returns true if connected and monitoring, false otherwise.
#[tauri::command]
pub async fn get_monitoring_status() -> Result<bool, String> {
    Ok(monitoring::get_connection_status().await)
}
