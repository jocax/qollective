// ABOUTME: Space exploration REST API handlers following integration test patterns  
// ABOUTME: Provides mission data, spacecraft status, and system information via REST endpoints

use qollective::{
    error::Result,
    server::rest::{RestServer, RestServerConfig},
    server::common::ServerConfig,
    prelude::{ContextDataHandler, UnifiedEnvelopeReceiver},
    envelope::Context,
};
use space_shared::{SpaceDataGenerator, Mission};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tracing::{info, debug, warn};
use async_trait::async_trait;

/// Simple configuration following test patterns
pub fn get_space_rest_config() -> RestServerConfig {
    RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8443,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Request/Response types for REST endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionsListRequest {
    pub mission_filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionsListResponse {
    pub missions: Vec<Mission>,
    pub total_count: usize,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionByIdRequest {
    pub mission_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionByIdResponse {
    pub mission: Option<Mission>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatusRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub server_status: String,
    pub uptime_seconds: u64,
    pub active_missions: usize,
    pub active_spacecraft: usize,
    pub server_version: String,
}

/// Handler for missions list
pub struct MissionsHandler {
    pub missions: Arc<RwLock<HashMap<String, Mission>>>,
}

/// Handler for mission by ID
pub struct MissionByIdHandler {
    pub missions: Arc<RwLock<HashMap<String, Mission>>>,
}

/// Handler for system status
pub struct SystemStatusHandler;

/// Create and configure space REST server
pub async fn create_space_rest_server() -> Result<RestServer> {
    info!("🚀 Creating Space REST server...");
    
    // Initialize demo data
    let missions_data = SpaceDataGenerator::generate_missions();
    let mut missions = HashMap::new();
    
    for mission in missions_data {
        missions.insert(mission.id.clone(), mission);
    }
    let missions_arc = Arc::new(RwLock::new(missions));
    
    info!("📊 Loaded {} missions", missions_arc.read().unwrap().len());
    
    // Create server with config
    let config = get_space_rest_config();
    let mut server = RestServer::new(config).await?;
    
    // Register handlers following test pattern
    info!("📝 Registering REST endpoints...");
    
    // GET /missions - List all missions
    let missions_handler = MissionsHandler {
        missions: Arc::clone(&missions_arc),
    };
    server.receive_envelope_at("/missions", missions_handler).await?;
    
    // GET /mission/{id} - Get mission by ID  
    let mission_by_id_handler = MissionByIdHandler {
        missions: Arc::clone(&missions_arc),
    };
    server.receive_envelope_at("/mission", mission_by_id_handler).await?;
    
    // GET /status - Server status
    let status_handler = SystemStatusHandler;
    server.receive_envelope_at("/status", status_handler).await?;
    
    info!("✅ Registered {} REST endpoints", server.route_count());
    
    // Display server information
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    🚀 SPACE EXPLORATION REST SERVER 🚀                     ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  📡 Endpoint: http://127.0.0.1:8443                                        ║");
    println!("║                                                                              ║");
    println!("║  Available Endpoints:                                                        ║");
    println!("║    POST /missions          - List all space missions                        ║");
    println!("║    POST /mission           - Get specific mission details                   ║");
    println!("║    POST /status            - Server status and statistics                   ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    Ok(server)
}

// Handler implementations following async_trait pattern

#[async_trait]
impl ContextDataHandler<MissionsListRequest, MissionsListResponse> for MissionsHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        _request: MissionsListRequest,
    ) -> Result<MissionsListResponse> {
        info!("📋 REST REQUEST - Missions list request");
        info!("📊 Request details: {:?}", _request);
        
        let missions = self.missions.read().unwrap();
        let missions_vec: Vec<Mission> = missions.values().cloned().collect();
        let total_count = missions_vec.len();
        
        let response = MissionsListResponse {
            missions: missions_vec,
            total_count,
            status: "success".to_string(),
        };
        
        info!("📊 REST RESPONSE - Returning {} missions", total_count);
        info!("📤 Response details: {:?}", response);
        Ok(response)
    }
}

#[async_trait]
impl ContextDataHandler<MissionByIdRequest, MissionByIdResponse> for MissionByIdHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        request: MissionByIdRequest,
    ) -> Result<MissionByIdResponse> {
        let mission_id = &request.mission_id;
        debug!("🔍 Handling mission by ID request: {}", mission_id);
        
        let missions = self.missions.read().unwrap();
        let mission = missions.get(mission_id).cloned();
        
        let response = MissionByIdResponse {
            mission: mission.clone(),
            status: if mission.is_some() { "success" } else { "not_found" }.to_string(),
        };
        
        if mission.is_some() {
            info!("✅ Found mission: {}", mission_id);
        } else {
            warn!("❌ Mission not found: {}", mission_id);
        }
        
        Ok(response)
    }
}

#[async_trait]
impl ContextDataHandler<SystemStatusRequest, SystemStatusResponse> for SystemStatusHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        _request: SystemStatusRequest,
    ) -> Result<SystemStatusResponse> {
        debug!("⚡ Handling system status request");
        
        let response = SystemStatusResponse {
            server_status: "operational".to_string(),
            uptime_seconds: 3600, // Mock uptime
            active_missions: 2,
            active_spacecraft: 3,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        info!("📈 System status: operational");
        Ok(response)
    }
}