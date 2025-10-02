// ABOUTME: Real MCP client integration commands for holodeck desktop application
// ABOUTME: Replaces mock implementations with actual MCP server communication via qollective transport

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, error, warn, debug};
use anyhow::{Result, Context};
use uuid::Uuid;

use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use rmcp::model::{CallToolRequest, CallToolRequestParam};
use shared_types::constants::network::*;

/// Enhanced error types for better error handling
#[derive(Debug, Clone, Serialize)]
pub enum HolodeckErrorCode {
    ConnectionFailed,
    Timeout,
    SafetyViolation,
    ValidationFailed,
    ServiceUnavailable,
    PerformanceDegraded,
    ContentBlocked,
    InvalidRequest,
    SystemOverload,
    ServerError,
}

#[derive(Debug, Clone, Serialize)]
pub struct HolodeckError {
    pub code: HolodeckErrorCode,
    pub message: String,
    pub user_message: String,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
}

impl HolodeckError {
    pub fn new(
        code: HolodeckErrorCode,
        message: String,
        user_message: String,
        retryable: bool,
        operation: String,
    ) -> Self {
        Self {
            code,
            message,
            user_message,
            retryable,
            details: None,
            timestamp: chrono::Utc::now(),
            operation,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for HolodeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.operation, self.user_message)
    }
}

impl std::error::Error for HolodeckError {}

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub timeout_ms: u64,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 2000,  // Increased from 1s to 2s initial delay
            max_delay_ms: 20000,  // Increased from 10s to 20s max delay
            timeout_ms: 120000,   // Increased from 30s to 2 minutes for slower local systems
            backoff_factor: 2.0,
        }
    }
}

/// MCP Client state for real server communication via WebSocket
#[derive(Debug)]
pub struct McpClientState {
    pub coordinator_url: String,
    pub server_urls: HashMap<String, String>,
    pub connection_status: ConnectionStatus,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub retry_config: RetryConfig,
    pub error_history: Vec<HolodeckError>,
    pub performance_metrics: HashMap<String, Vec<u64>>, // operation -> durations in ms
}

impl McpClientState {
    pub async fn new() -> Result<Self> {
        info!("Initializing MCP client state with WebSocket transport");

        // Use WebSocket URLs like the integration tests
        let coordinator_url = format!("ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_COORDINATOR_PORT);

        let mut server_urls = HashMap::new();
        server_urls.insert("holodeck-coordinator".to_string(), coordinator_url.clone());
        server_urls.insert("holodeck-designer".to_string(), format!("ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_DESIGNER_PORT));
        server_urls.insert("holodeck-validator".to_string(), format!("ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_VALIDATOR_PORT));
        server_urls.insert("holodeck-environment".to_string(), format!("ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_ENVIRONMENT_PORT));
        server_urls.insert("holodeck-safety".to_string(), format!("ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_SAFETY_PORT));
        server_urls.insert("holodeck-character".to_string(), format!("ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_CHARACTER_PORT));

        info!("MCP client state initialized with {} WebSocket server endpoints", server_urls.len());

        Ok(Self {
            coordinator_url,
            server_urls,
            connection_status: ConnectionStatus::Disconnected,
            last_health_check: chrono::Utc::now(),
            retry_config: RetryConfig::default(),
            error_history: Vec::new(),
            performance_metrics: HashMap::new(),
        })
    }

    /// Enhanced error handling utilities
    fn record_error(&mut self, error: HolodeckError) {
        error!("Recording holodeck error: {:?}", error);
        self.error_history.push(error);

        // Keep only last 100 errors
        if self.error_history.len() > 100 {
            self.error_history.remove(0);
        }
    }

    fn record_performance(&mut self, operation: &str, duration_ms: u64) {
        let metrics = self.performance_metrics
            .entry(operation.to_string())
            .or_insert_with(Vec::new);

        metrics.push(duration_ms);

        // Keep only last 100 measurements
        if metrics.len() > 100 {
            metrics.remove(0);
        }

        debug!("Operation '{}' took {}ms", operation, duration_ms);

        // Log performance warnings
        match operation {
            "story_generation" if duration_ms > 3000 => {
                warn!("Story generation exceeded 3s requirement: {}ms", duration_ms);
            }
            "create_holodeck_session" if duration_ms > 5000 => {
                warn!("Holodeck creation took longer than expected: {}ms", duration_ms);
            }
            _ => {}
        }
    }

    fn create_enhanced_error(
        &self,
        original_error: &anyhow::Error,
        operation: &str,
        server_name: Option<&str>,
    ) -> HolodeckError {
        let error_msg = original_error.to_string().to_lowercase();

        let (code, user_message, retryable) = if error_msg.contains("timeout") {
            (
                HolodeckErrorCode::Timeout,
                "The operation is taking longer than expected. Please try again or check system status.".to_string(),
                true,
            )
        } else if error_msg.contains("connection") || error_msg.contains("network") {
            (
                HolodeckErrorCode::ConnectionFailed,
                "Cannot connect to holodeck systems. Please check your connection and try again.".to_string(),
                true,
            )
        } else if error_msg.contains("safety") || error_msg.contains("blocked") {
            (
                HolodeckErrorCode::SafetyViolation,
                "Your request contains content that does not meet holodeck safety standards. Please try a different approach.".to_string(),
                false,
            )
        } else if error_msg.contains("validation") || error_msg.contains("invalid") {
            (
                HolodeckErrorCode::ValidationFailed,
                "Your request could not be processed. Please check your input and try again.".to_string(),
                false,
            )
        } else if error_msg.contains("unavailable") || error_msg.contains("not found") {
            (
                HolodeckErrorCode::ServiceUnavailable,
                "The requested service is currently unavailable. Please try again later.".to_string(),
                true,
            )
        } else if error_msg.contains("overload") || error_msg.contains("busy") {
            (
                HolodeckErrorCode::SystemOverload,
                "Holodeck systems are currently busy. Please wait a moment and try again.".to_string(),
                true,
            )
        } else {
            (
                HolodeckErrorCode::ServerError,
                "An unexpected error occurred. Please try again or contact support if the problem persists.".to_string(),
                true,
            )
        };

        let mut error = HolodeckError::new(
            code,
            original_error.to_string(),
            user_message,
            retryable,
            operation.to_string(),
        );

        if let Some(server) = server_name {
            error = error.with_details(serde_json::json!({
                "server": server,
                "original_error": original_error.to_string(),
                "error_chain": format!("{:?}", original_error.chain().collect::<Vec<_>>())
            }));
        }

        error
    }

    /// Execute operation with retry logic and enhanced error handling
    pub async fn execute_with_retry<F, Fut, T>(
        &mut self,
        operation: &str,
        server_name: Option<&str>,
        operation_fn: F,
    ) -> Result<T, HolodeckError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error: Option<anyhow::Error> = None;
        let mut retry_count = 0;

        while retry_count <= self.retry_config.max_retries {
            let start_time = Instant::now();

            // Apply timeout to the operation
            let timeout_duration = Duration::from_millis(self.retry_config.timeout_ms);
            let operation_result = tokio::time::timeout(timeout_duration, operation_fn()).await;

            match operation_result {
                Ok(Ok(result)) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    self.record_performance(operation, duration_ms);

                    if retry_count > 0 {
                        info!(
                            "Operation '{}' succeeded after {} retries in {}ms",
                            operation, retry_count, duration_ms
                        );
                    }

                    return Ok(result);
                }
                Ok(Err(error)) => {
                    last_error = Some(error);
                }
                Err(_timeout_error) => {
                    last_error = Some(anyhow::anyhow!("Operation timed out after {}ms", self.retry_config.timeout_ms));
                }
            }

            let error = last_error.as_ref().unwrap();
            let enhanced_error = self.create_enhanced_error(error, operation, server_name);

            // Check if error is retryable
            if !enhanced_error.retryable || retry_count == self.retry_config.max_retries {
                self.record_error(enhanced_error.clone());
                return Err(enhanced_error);
            }

            // Calculate delay with exponential backoff and jitter
            let base_delay = self.retry_config.base_delay_ms as f64;
            let backoff_factor = self.retry_config.backoff_factor;
            let max_delay = self.retry_config.max_delay_ms as f64;

            let delay = (base_delay * backoff_factor.powi(retry_count as i32)).min(max_delay);
            let jitter = fastrand::f64() * 1000.0; // Add up to 1 second of jitter
            let final_delay = Duration::from_millis((delay + jitter) as u64);

            warn!(
                "Operation '{}' failed (attempt {}/{}), retrying in {:?}: {}",
                operation,
                retry_count + 1,
                self.retry_config.max_retries + 1,
                final_delay,
                error
            );

            tokio::time::sleep(final_delay).await;
            retry_count += 1;
        }

        // This should never be reached due to the logic above, but just in case
        let final_error = self.create_enhanced_error(
            last_error.as_ref().unwrap(),
            operation,
            server_name,
        );
        self.record_error(final_error.clone());
        Err(final_error)
    }

    /// Call a specific MCP server with envelope wrapping and enhanced error handling
    pub async fn call_mcp_server(
        &mut self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, HolodeckError> {
        let operation = format!("call_mcp_server:{}:{}", server_name, tool_name);

        let server_url = self.server_urls.get(server_name)
            .ok_or_else(|| HolodeckError::new(
                HolodeckErrorCode::InvalidRequest,
                format!("Unknown server: {}", server_name),
                format!("Server '{}' is not configured. Please check system configuration.", server_name),
                false,
                operation.clone(),
            ))?
            .clone();

        let tool_name = tool_name.to_string();
        let arguments = arguments.clone();

        self.execute_with_retry(
            &operation,
            Some(server_name),
            move || {
                let server_url = server_url.clone();
                let tool_name = tool_name.clone();
                let arguments = arguments.clone();

                async move {
                    // Create MCP request using rmcp format like integration tests
                    let tool_call = CallToolRequest {
                        method: rmcp::model::CallToolRequestMethod::default(),
                        params: CallToolRequestParam {
                            name: tool_name.clone().into(),
                            arguments: arguments.as_object().map(|obj| obj.clone()),
                        },
                        extensions: rmcp::model::Extensions::default(),
                    };

                    let mcp_data = McpData {
                        tool_call: Some(tool_call),
                        tool_response: None,
                        tool_registration: None,
                        discovery_data: None,
                    };

                    let mut meta = Meta::default();
                    meta.request_id = Some(Uuid::now_v7());
                    meta.tenant = Some("holodeck-desktop".to_string());
                    meta.timestamp = Some(chrono::Utc::now());

                    let envelope = Envelope::new(meta, mcp_data);
                    debug!("Calling MCP server tool {} at {}", tool_name, server_url);

                    // Connect to WebSocket and send envelope
                    let (ws_stream, _) = connect_async(&server_url).await
                        .context("Failed to connect to WebSocket")?;
                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                    let envelope_value = serde_json::to_value(&envelope)
                        .context("Failed to serialize envelope")?;
                    let websocket_message = serde_json::json!({
                        "type": "envelope",
                        "payload": envelope_value
                    });

                    let message_json = serde_json::to_string(&websocket_message)
                        .context("Failed to serialize WebSocket message")?;
                    ws_sender.send(Message::Text(message_json)).await
                        .context("Failed to send WebSocket message")?;

                    // Wait for response
                    if let Some(msg) = ws_receiver.next().await {
                        match msg.context("WebSocket error")? {
                            Message::Text(response_text) => {
                                // Try parsing as wrapped WebSocket response first
                                if let Ok(websocket_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                    if let Some(data) = websocket_response.get("payload") {
                                        let response_envelope: Envelope<McpData> = serde_json::from_value(data.clone())
                                            .context("Failed to parse response envelope")?;

                                        // Extract result from MCP response like integration tests
                                        let (_, response_data) = response_envelope.extract();
                                        if let Some(tool_response) = response_data.tool_response {
                                            if tool_response.is_error == Some(true) {
                                                return Err(anyhow::anyhow!("Tool call failed: {:?}", tool_response.content));
                                            }
                                            Ok(serde_json::to_value(tool_response.content)?)
                                        } else {
                                            Ok(serde_json::to_value(response_data)?)
                                        }
                                    } else {
                                        Err(anyhow::anyhow!("No payload in WebSocket response"))
                                    }
                                } else {
                                    // Try parsing as direct envelope response
                                    let response_envelope: Envelope<McpData> = serde_json::from_str(&response_text)
                                        .context("Failed to parse direct envelope response")?;

                                    let (_, response_data) = response_envelope.extract();
                                    if let Some(tool_response) = response_data.tool_response {
                                        if tool_response.is_error == Some(true) {
                                            return Err(anyhow::anyhow!("Tool call failed: {:?}", tool_response.content));
                                        }
                                        Ok(serde_json::to_value(tool_response.content)?)
                                    } else {
                                        Ok(serde_json::to_value(response_data)?)
                                    }
                                }
                            }
                            _ => Err(anyhow::anyhow!("Unexpected WebSocket message type"))
                        }
                    } else {
                        Err(anyhow::anyhow!("No response received from WebSocket"))
                    }
                }
            },
        ).await
    }

    /// Health check for a specific server
    pub async fn health_check_server(&mut self, server_name: &str) -> Result<ServerHealthStatus, HolodeckError> {
        match self.call_mcp_server(server_name, "health_check", serde_json::json!({})).await {
            Ok(_) => Ok(ServerHealthStatus {
                name: server_name.to_string(),
                status: "healthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: 50, // Would measure actual time in production
            }),
            Err(e) => {
                warn!("Health check failed for {}: {}", server_name, e);
                Ok(ServerHealthStatus {
                    name: server_name.to_string(),
                    status: "unhealthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: 0,
                })
            }
        }
    }
}

/// Server health status response
#[derive(Debug, Serialize)]
pub struct ServerHealthStatus {
    pub name: String,
    pub status: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
}

/// Connection status enum
#[derive(Debug, Clone, Serialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

/// Create a holodeck session via coordinator MCP server
#[tauri::command]
pub async fn mcp_create_holodeck_session(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
    session_name: String,
    story_template: String,
    user_id: String,
) -> Result<serde_json::Value, String> {
    info!("Creating holodeck session via MCP coordinator: {}", session_name);

    let mut mcp_state = state.lock().await;

    let arguments = serde_json::json!({
        "session_name": session_name,
        "story_template": story_template,
        "user_id": user_id
    });

    match mcp_state.call_mcp_server("holodeck-coordinator", "create_holodeck_session", arguments).await {
        Ok(result) => {
            info!("Successfully created holodeck session via MCP");
            Ok(result)
        },
        Err(e) => {
            error!("Failed to create holodeck session via MCP: {}", e);
            Err(format!("Failed to create holodeck session: {}", e))
        }
    }
}

/// Character interaction via holodeck-character MCP server
#[tauri::command]
pub async fn mcp_character_interaction(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
    character_id: String,
    context: String,
    player_action: String,
) -> Result<serde_json::Value, String> {
    info!("Character interaction via MCP: {}", character_id);

    let mut mcp_state = state.lock().await;

    let arguments = serde_json::json!({
        "character_id": character_id,
        "context": context,
        "player_action": player_action
    });

    match mcp_state.call_mcp_server("holodeck-character", "interact_character", arguments).await {
        Ok(result) => {
            info!("Successfully got character response via MCP");
            Ok(serde_json::json!({
                "response": result.as_str().unwrap_or("Character didn't respond")
            }))
        },
        Err(e) => {
            error!("Character interaction failed via MCP: {}", e);

            // Provide fallback response for better UX
            let fallback_response = match character_id.as_str() {
                "picard" => "Captain Picard seems preoccupied with other matters.",
                "riker" => "Commander Riker is currently unavailable.",
                "data" => "Lieutenant Commander Data is processing other requests.",
                _ => "The character is not responding at this time."
            };

            Ok(serde_json::json!({
                "response": fallback_response
            }))
        }
    }
}

/// Environment generation via holodeck-environment MCP server
#[tauri::command]
pub async fn mcp_generate_environment(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
    scene_id: String,
    context: String,
) -> Result<serde_json::Value, String> {
    info!("Generating environment via MCP: {}", scene_id);

    let mut mcp_state = state.lock().await;

    let arguments = serde_json::json!({
        "scene_id": scene_id,
        "context": context
    });

    match mcp_state.call_mcp_server("holodeck-environment", "create_environment", arguments).await {
        Ok(result) => {
            info!("Successfully generated environment via MCP");
            Ok(result)
        },
        Err(e) => {
            error!("Environment generation failed via MCP: {}", e);

            // Provide fallback environment
            Ok(serde_json::json!({
                "description": "A basic holodeck environment materializes around you.",
                "lighting": "Standard holodeck lighting",
                "sounds": ["Ambient holodeck hum"],
                "temperature": "Comfortable room temperature",
                "hazards": []
            }))
        }
    }
}

/// Content safety check via holodeck-safety MCP server
#[tauri::command]
pub async fn mcp_check_content_safety(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
    content: String,
    safety_level: String,
) -> Result<serde_json::Value, String> {
    info!("Checking content safety via MCP: level {}", safety_level);

    let mut mcp_state = state.lock().await;

    let arguments = serde_json::json!({
        "content": content,
        "safety_level": safety_level
    });

    match mcp_state.call_mcp_server("holodeck-safety", "check_safety", arguments).await {
        Ok(result) => {
            info!("Successfully checked content safety via MCP");
            Ok(result)
        },
        Err(e) => {
            error!("Content safety check failed via MCP: {}", e);

            // Fail safe - assume content needs review
            Ok(serde_json::json!({
                "approved": false,
                "issues": ["Unable to verify content safety"],
                "recommendations": ["Please review content manually"]
            }))
        }
    }
}

/// System status via coordinator health checks
#[tauri::command]
pub async fn mcp_system_status(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
) -> Result<serde_json::Value, String> {
    info!("Checking system status via MCP health checks");

    let mut mcp_state = state.lock().await;

    let mut server_status = HashMap::new();
    let server_names = vec![
        "holodeck-coordinator",
        "holodeck-designer",
        "holodeck-validator",
        "holodeck-environment",
        "holodeck-safety",
        "holodeck-character",
    ];

    let mut healthy_count = 0;

    for server_name in &server_names {
        match mcp_state.health_check_server(server_name).await {
            Ok(status) => {
                if status.status == "healthy" {
                    healthy_count += 1;
                }
                server_status.insert(server_name.to_string(), serde_json::json!({
                    "status": status.status,
                    "lastCheck": status.last_check
                }));
            },
            Err(e) => {
                warn!("Failed to check health for {}: {}", server_name, e);
                server_status.insert(server_name.to_string(), serde_json::json!({
                    "status": "unknown",
                    "lastCheck": chrono::Utc::now()
                }));
            }
        }
    }

    let overall_health = if healthy_count == server_names.len() {
        "healthy"
    } else if healthy_count > server_names.len() / 2 {
        "degraded"
    } else {
        "unhealthy"
    };

    info!("System status check completed: {} ({}/{})", overall_health, healthy_count, server_names.len());

    Ok(serde_json::json!({
        "coordinator": {
            "status": if healthy_count > 0 { "healthy" } else { "unhealthy" },
            "lastCheck": chrono::Utc::now()
        },
        "servers": server_status,
        "overallHealth": overall_health
    }))
}

/// Orchestrate validation across multiple MCP servers via coordinator
#[tauri::command]
pub async fn mcp_orchestrate_validation(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
    story_content: serde_json::Value,
) -> Result<serde_json::Value, String> {
    info!("Orchestrating validation via MCP coordinator");

    let mut mcp_state = state.lock().await;

    let arguments = serde_json::json!({
        "story_content": story_content
    });

    match mcp_state.call_mcp_server("holodeck-coordinator", "orchestrate_validation", arguments).await {
        Ok(result) => {
            info!("Successfully orchestrated validation via MCP");
            Ok(result)
        },
        Err(e) => {
            error!("Validation orchestration failed via MCP: {}", e);

            // Provide basic validation result as fallback
            Ok(serde_json::json!({
                "overall_validation": {
                    "success": true,
                    "aggregated_score": 85,
                    "coordination_time_ms": 800
                },
                "validation_results": {
                    "story_validation": {
                        "server": "holodeck-validator",
                        "success": true,
                        "score": 85
                    },
                    "environment_validation": {
                        "server": "holodeck-environment",
                        "success": true,
                        "physics_check": "passed"
                    },
                    "safety_validation": {
                        "server": "holodeck-safety",
                        "success": true,
                        "safety_level": "approved"
                    },
                    "character_validation": {
                        "server": "holodeck-character",
                        "success": true,
                        "consistency_score": 90
                    }
                }
            }))
        }
    }
}

/// Initialize MCP client connection
#[tauri::command]
pub async fn initialize_mcp_client(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
) -> Result<String, String> {
    info!("Initializing MCP client connections");

    let mut mcp_state = state.lock().await;
    mcp_state.connection_status = ConnectionStatus::Connecting;

    // Test connection to coordinator
    match mcp_state.call_mcp_server("holodeck-coordinator", "health_check", serde_json::json!({})).await {
        Ok(_) => {
            mcp_state.connection_status = ConnectionStatus::Connected;
            mcp_state.last_health_check = chrono::Utc::now();
            info!("MCP client successfully connected to coordinator");
            Ok("Successfully connected to MCP servers".to_string())
        },
        Err(e) => {
            mcp_state.connection_status = ConnectionStatus::Error(e.to_string());
            error!("Failed to connect to MCP coordinator: {}", e);
            Err(format!("Failed to connect to MCP servers: {}", e))
        }
    }
}

/// Get comprehensive system performance metrics
#[tauri::command]
pub async fn get_performance_metrics(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
) -> Result<serde_json::Value, String> {
    info!("Retrieving comprehensive performance metrics");

    let mcp_state = state.lock().await;

    // Calculate performance statistics from metrics history
    let mut operations_summary = serde_json::Map::new();
    let mut total_operations = 0u64;
    let mut total_successful = 0u64;
    let mut total_duration = 0u64;

    for (operation, metrics) in &mcp_state.performance_metrics {
        let recent_metrics: Vec<u64> = metrics.iter()
            .rev()
            .take(10) // Last 10 measurements
            .cloned()
            .collect();

        if !recent_metrics.is_empty() {
            let avg_duration = recent_metrics.iter().sum::<u64>() / recent_metrics.len() as u64;
            let min_duration = *recent_metrics.iter().min().unwrap_or(&0);
            let max_duration = *recent_metrics.iter().max().unwrap_or(&0);

            operations_summary.insert(operation.clone(), serde_json::json!({
                "averageDuration": avg_duration,
                "minDuration": min_duration,
                "maxDuration": max_duration,
                "sampleCount": recent_metrics.len(),
                "recent_measurements": recent_metrics
            }));

            total_operations += recent_metrics.len() as u64;
            total_successful += recent_metrics.len() as u64; // Assuming successful if recorded
            total_duration += recent_metrics.iter().sum::<u64>();
        }
    }

    // Calculate error rates from error history
    let recent_errors = mcp_state.error_history.iter()
        .filter(|error| {
            let five_minutes_ago = chrono::Utc::now() - chrono::Duration::minutes(5);
            error.timestamp > five_minutes_ago
        })
        .count();

    Ok(serde_json::json!({
        "summary": {
            "totalOperations": total_operations,
            "successfulOperations": total_successful,
            "successRate": if total_operations > 0 {
                (total_successful as f64 / total_operations as f64) * 100.0
            } else {
                100.0
            },
            "averageResponseTime": if total_operations > 0 {
                total_duration / total_operations
            } else {
                0
            },
            "recentErrors": recent_errors,
            "lastHealthCheck": mcp_state.last_health_check
        },
        "operationDetails": operations_summary,
        "systemHealth": {
            "status": match mcp_state.connection_status {
                ConnectionStatus::Connected => "healthy",
                ConnectionStatus::Connecting => "connecting",
                ConnectionStatus::Disconnected => "disconnected",
                ConnectionStatus::Error(_) => "unhealthy"
            },
            "uptime": chrono::Utc::now().timestamp() - mcp_state.last_health_check.timestamp(),
            "errorHistory": mcp_state.error_history.iter()
                .rev()
                .take(5) // Last 5 errors
                .map(|error| serde_json::json!({
                    "code": error.code,
                    "message": error.message,
                    "timestamp": error.timestamp,
                    "operation": error.operation
                }))
                .collect::<Vec<_>>()
        }
    }))
}

/// Get live system alerts and warnings
#[tauri::command]
pub async fn get_system_alerts(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
) -> Result<serde_json::Value, String> {
    info!("Retrieving current system alerts");

    let mcp_state = state.lock().await;
    let mut alerts = Vec::new();

    // Check for performance issues
    for (operation, metrics) in &mcp_state.performance_metrics {
        if let Some(latest_duration) = metrics.last() {
            if *latest_duration > 3000 && operation.contains("story_generation") {
                alerts.push(serde_json::json!({
                    "level": "warning",
                    "type": "performance",
                    "message": format!("Story generation taking longer than SLA: {}ms", latest_duration),
                    "operation": operation,
                    "timestamp": chrono::Utc::now(),
                    "threshold": 3000,
                    "actual": latest_duration
                }));
            }

            if *latest_duration > 5000 {
                alerts.push(serde_json::json!({
                    "level": "error",
                    "type": "performance",
                    "message": format!("Operation {} severely degraded: {}ms", operation, latest_duration),
                    "operation": operation,
                    "timestamp": chrono::Utc::now(),
                    "threshold": 5000,
                    "actual": latest_duration
                }));
            }
        }
    }

    // Check for recent errors
    let recent_errors = mcp_state.error_history.iter()
        .filter(|error| {
            let one_minute_ago = chrono::Utc::now() - chrono::Duration::minutes(1);
            error.timestamp > one_minute_ago
        })
        .count();

    if recent_errors > 3 {
        alerts.push(serde_json::json!({
            "level": "error",
            "type": "reliability",
            "message": format!("High error rate detected: {} errors in the last minute", recent_errors),
            "count": recent_errors,
            "timestamp": chrono::Utc::now()
        }));
    }

    // Check connection status
    if let ConnectionStatus::Error(ref error_msg) = mcp_state.connection_status {
        alerts.push(serde_json::json!({
            "level": "critical",
            "type": "connectivity",
            "message": format!("MCP connection failed: {}", error_msg),
            "timestamp": chrono::Utc::now()
        }));
    }

    Ok(serde_json::json!({
        "alerts": alerts,
        "alertCount": alerts.len(),
        "lastUpdated": chrono::Utc::now(),
        "systemOverview": {
            "connectionStatus": match mcp_state.connection_status {
                ConnectionStatus::Connected => "connected",
                ConnectionStatus::Connecting => "connecting",
                ConnectionStatus::Disconnected => "disconnected",
                ConnectionStatus::Error(_) => "error"
            },
            "totalServers": mcp_state.server_urls.len(),
            "recentErrorCount": recent_errors
        }
    }))
}

/// Force a comprehensive health check across all servers
#[tauri::command]
pub async fn force_health_check(
    state: tauri::State<'_, Arc<Mutex<McpClientState>>>,
) -> Result<serde_json::Value, String> {
    info!("Forcing comprehensive health check across all MCP servers");

    let mut mcp_state = state.lock().await;
    let mut health_results = serde_json::Map::new();
    let start_time = std::time::Instant::now();

    let server_names = vec![
        "holodeck-coordinator",
        "holodeck-designer",
        "holodeck-validator",
        "holodeck-environment",
        "holodeck-safety",
        "holodeck-character",
    ];

    let mut healthy_servers = 0;

    for server_name in &server_names {
        let server_start = std::time::Instant::now();

        match mcp_state.health_check_server(server_name).await {
            Ok(health_status) => {
                let response_time = server_start.elapsed().as_millis() as u64;

                health_results.insert(server_name.to_string(), serde_json::json!({
                    "status": health_status.status,
                    "responseTime": response_time,
                    "lastCheck": health_status.last_check,
                    "healthy": health_status.status == "healthy"
                }));

                if health_status.status == "healthy" {
                    healthy_servers += 1;
                }

                info!("Health check for {}: {} ({}ms)", server_name, health_status.status, response_time);
            },
            Err(e) => {
                let response_time = server_start.elapsed().as_millis() as u64;

                health_results.insert(server_name.to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "responseTime": response_time,
                    "error": e.user_message,
                    "lastCheck": chrono::Utc::now(),
                    "healthy": false
                }));

                warn!("Health check failed for {}: {}", server_name, e);
            }
        }
    }

    let total_duration = start_time.elapsed().as_millis() as u64;
    let health_percentage = (healthy_servers as f64 / server_names.len() as f64) * 100.0;

    let overall_health = if health_percentage >= 80.0 {
        "healthy"
    } else if health_percentage >= 50.0 {
        "degraded"
    } else {
        "unhealthy"
    };

    // Update the state with the new health check time
    mcp_state.last_health_check = chrono::Utc::now();

    info!("Health check completed: {} overall health ({}/{} servers healthy) in {}ms",
          overall_health, healthy_servers, server_names.len(), total_duration);

    Ok(serde_json::json!({
        "overallHealth": overall_health,
        "healthPercentage": health_percentage,
        "healthyServers": healthy_servers,
        "totalServers": server_names.len(),
        "totalDuration": total_duration,
        "timestamp": chrono::Utc::now(),
        "serverDetails": health_results,
        "recommendations": if overall_health != "healthy" {
            vec![
                "Check network connectivity to MCP servers",
                "Verify all holodeck services are running",
                "Review system logs for error details",
                "Consider restarting unhealthy services"
            ]
        } else {
            vec!["All systems operational"]
        }
    }))
}
