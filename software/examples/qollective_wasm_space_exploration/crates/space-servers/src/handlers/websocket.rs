// ABOUTME: Space exploration WebSocket server for real-time telemetry streaming
// ABOUTME: Provides live spacecraft telemetry updates and mission status changes

use qollective::{
    error::Result,
    server::websocket::{WebSocketServer, WebSocketServerConfig},
    server::common::ServerConfig,
    prelude::{ContextDataHandler, UnifiedEnvelopeReceiver},
    envelope::Context,
};
use space_shared::{SpaceDataGenerator, Spacecraft, SpacecraftTelemetry, Mission};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, debug, warn};
use async_trait::async_trait;
use tokio::time;

/// WebSocket server configuration for space telemetry
pub fn get_space_websocket_config() -> WebSocketServerConfig {
    WebSocketServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8444,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Request/Response types for WebSocket telemetry

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetrySubscribeRequest {
    pub spacecraft_id: String,
    pub update_interval_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetrySubscribeResponse {
    pub status: String,
    pub subscription_id: String,
    pub spacecraft_id: String,
    pub update_interval_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryUpdateNotification {
    pub spacecraft_id: String,
    pub telemetry: SpacecraftTelemetry,
    pub timestamp: String,
    pub sequence_number: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionStatusRequest {
    pub mission_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionStatusResponse {
    pub missions: Vec<Mission>,
    pub active_spacecraft_count: usize,
    pub total_missions: usize,
    pub status: String,
}

/// Telemetry subscription manager
pub struct TelemetryManager {
    pub spacecraft: Arc<RwLock<HashMap<String, Spacecraft>>>,
    pub subscriptions: Arc<RwLock<HashMap<String, TelemetrySubscription>>>,
    pub sequence_counter: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
pub struct TelemetrySubscription {
    pub subscription_id: String,
    pub spacecraft_id: String,
    pub update_interval_ms: u64,
    pub last_update: std::time::Instant,
}

/// Handler for telemetry subscriptions
pub struct TelemetrySubscribeHandler {
    pub manager: Arc<TelemetryManager>,
}

/// Handler for mission status updates
pub struct MissionStatusHandler {
    pub manager: Arc<TelemetryManager>,
}

impl TelemetryManager {
    pub fn new() -> Self {
        // Initialize with demo spacecraft data
        let missions = SpaceDataGenerator::generate_missions();
        let mut spacecraft = HashMap::new();
        
        for mission in missions {
            for craft in mission.spacecraft {
                spacecraft.insert(craft.id.clone(), craft);
            }
        }
        
        Self {
            spacecraft: Arc::new(RwLock::new(spacecraft)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            sequence_counter: Arc::new(RwLock::new(0)),
        }
    }
    
    pub fn generate_subscription_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        format!("sub_{}", timestamp)
    }
    
    pub fn get_next_sequence(&self) -> u64 {
        let mut counter = self.sequence_counter.write().unwrap();
        *counter += 1;
        *counter
    }
    
    /// Update spacecraft telemetry with realistic variations
    pub fn update_spacecraft_telemetry(&self, spacecraft_id: &str) -> Option<SpacecraftTelemetry> {
        let mut spacecraft_map = self.spacecraft.write().unwrap();
        if let Some(spacecraft) = spacecraft_map.get_mut(spacecraft_id) {
            SpaceDataGenerator::update_telemetry(&mut spacecraft.telemetry);
            Some(spacecraft.telemetry.clone())
        } else {
            None
        }
    }
}

/// Create and configure space WebSocket server
pub async fn create_space_websocket_server() -> Result<WebSocketServer> {
    info!("🛰️ Creating Space WebSocket server...");
    
    // Initialize telemetry manager
    let manager = Arc::new(TelemetryManager::new());
    let spacecraft_count = manager.spacecraft.read().unwrap().len();
    
    info!("📊 Loaded {} spacecraft for telemetry", spacecraft_count);
    
    // Create server with config
    let config = get_space_websocket_config();
    let mut server = WebSocketServer::new(config).await?;
    
    // Register handlers
    info!("📝 Registering WebSocket endpoints...");
    
    // Subscribe to telemetry stream
    let subscribe_handler = TelemetrySubscribeHandler {
        manager: Arc::clone(&manager),
    };
    server.receive_envelope_at("/telemetry/subscribe", subscribe_handler).await?;
    
    // Get mission status
    let status_handler = MissionStatusHandler {
        manager: Arc::clone(&manager),
    };
    server.receive_envelope_at("/mission/status", status_handler).await?;
    
    info!("✅ Registered WebSocket endpoints");
    
    // Start telemetry broadcast task
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        telemetry_broadcast_task(manager_clone).await;
    });
    
    // Display server information
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                   🛰️ SPACE EXPLORATION WEBSOCKET SERVER 🛰️                ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  📡 Endpoint: ws://127.0.0.1:8444                                          ║");
    println!("║                                                                              ║");
    println!("║  Available Endpoints:                                                        ║");
    println!("║    /telemetry/subscribe   - Subscribe to spacecraft telemetry               ║");
    println!("║    /mission/status        - Get real-time mission status                    ║");
    println!("║                                                                              ║");
    println!("║  Features:                                                                   ║");
    println!("║    • Real-time telemetry streaming                                          ║");
    println!("║    • Configurable update intervals                                          ║");
    println!("║    • Mission status monitoring                                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    Ok(server)
}

/// Background task for broadcasting telemetry updates
async fn telemetry_broadcast_task(manager: Arc<TelemetryManager>) {
    info!("🔄 Starting telemetry broadcast task...");
    
    let mut interval = time::interval(Duration::from_millis(1000)); // 1 second base interval
    
    loop {
        interval.tick().await;
        
        // Get current subscriptions
        let subscriptions = {
            let subs = manager.subscriptions.read().unwrap();
            subs.values().cloned().collect::<Vec<_>>()
        };
        
        for subscription in subscriptions {
            // Check if it's time to update this subscription
            let now = std::time::Instant::now();
            let interval_duration = Duration::from_millis(subscription.update_interval_ms);
            
            if now.duration_since(subscription.last_update) >= interval_duration {
                // Update telemetry for this spacecraft
                if let Some(telemetry) = manager.update_spacecraft_telemetry(&subscription.spacecraft_id) {
                    let notification = TelemetryUpdateNotification {
                        spacecraft_id: subscription.spacecraft_id.clone(),
                        telemetry,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        sequence_number: manager.get_next_sequence(),
                    };
                    
                    debug!(
                        "📡 Broadcasting telemetry update for spacecraft: {} (seq: {})",
                        notification.spacecraft_id,
                        notification.sequence_number
                    );
                    
                    // TODO: Broadcast to WebSocket clients
                    // For now, just log the update
                }
                
                // Update last update time
                let mut subs = manager.subscriptions.write().unwrap();
                if let Some(sub) = subs.get_mut(&subscription.subscription_id) {
                    sub.last_update = now;
                }
            }
        }
    }
}

// Handler implementations

#[async_trait]
impl ContextDataHandler<TelemetrySubscribeRequest, TelemetrySubscribeResponse> for TelemetrySubscribeHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        request: TelemetrySubscribeRequest,
    ) -> Result<TelemetrySubscribeResponse> {
        let spacecraft_id = &request.spacecraft_id;
        info!("🔌 WEBSOCKET REQUEST - Telemetry subscription for spacecraft: {}", spacecraft_id);
        info!("📊 Request details: {:?}", request);
        
        // Check if spacecraft exists
        let spacecraft_exists = {
            let spacecraft = self.manager.spacecraft.read().unwrap();
            spacecraft.contains_key(spacecraft_id)
        };
        
        if !spacecraft_exists {
            warn!("❌ Spacecraft not found: {}", spacecraft_id);
            return Ok(TelemetrySubscribeResponse {
                status: "error".to_string(),
                subscription_id: "".to_string(),
                spacecraft_id: spacecraft_id.clone(),
                update_interval_ms: 0,
            });
        }
        
        // Create subscription
        let subscription_id = self.manager.generate_subscription_id();
        let update_interval_ms = request.update_interval_ms.unwrap_or(5000); // Default 5 seconds
        
        let subscription = TelemetrySubscription {
            subscription_id: subscription_id.clone(),
            spacecraft_id: spacecraft_id.clone(),
            update_interval_ms,
            last_update: std::time::Instant::now(),
        };
        
        // Store subscription
        {
            let mut subscriptions = self.manager.subscriptions.write().unwrap();
            subscriptions.insert(subscription_id.clone(), subscription);
        }
        
        info!(
            "✅ WEBSOCKET RESPONSE - Created telemetry subscription: {} for spacecraft: {} (interval: {}ms)",
            subscription_id, spacecraft_id, update_interval_ms
        );
        info!("📤 Response details: {:?}", TelemetrySubscribeResponse {
            status: "success".to_string(),
            subscription_id: subscription_id.clone(),
            spacecraft_id: spacecraft_id.clone(),
            update_interval_ms,
        });
        
        Ok(TelemetrySubscribeResponse {
            status: "success".to_string(),
            subscription_id,
            spacecraft_id: spacecraft_id.clone(),
            update_interval_ms,
        })
    }
}

#[async_trait]
impl ContextDataHandler<MissionStatusRequest, MissionStatusResponse> for MissionStatusHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        request: MissionStatusRequest,
    ) -> Result<MissionStatusResponse> {
        info!("📊 WEBSOCKET REQUEST - Mission status request");
        info!("📋 Request details: {:?}", request);
        
        // Get all missions (in real implementation, this would come from a database)
        let missions = SpaceDataGenerator::generate_missions();
        
        // Filter by mission ID if specified
        let filtered_missions = if let Some(mission_id) = &request.mission_id {
            missions.into_iter()
                .filter(|m| &m.id == mission_id)
                .collect()
        } else {
            missions
        };
        
        // Count active spacecraft
        let active_spacecraft_count = {
            let spacecraft = self.manager.spacecraft.read().unwrap();
            spacecraft.len()
        };
        
        let response = MissionStatusResponse {
            total_missions: filtered_missions.len(),
            active_spacecraft_count,
            missions: filtered_missions,
            status: "success".to_string(),
        };
        
        info!(
            "📈 WEBSOCKET RESPONSE - Mission status: {} missions, {} active spacecraft",
            response.total_missions,
            response.active_spacecraft_count
        );
        info!("📤 Response details: {:?}", response);
        
        Ok(response)
    }
}