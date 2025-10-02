// ABOUTME: Space exploration shared data structures and types
// ABOUTME: Defines mission data, spacecraft status, telemetry, and tool definitions for demo

use serde::{Deserialize, Serialize};

/// Mission data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub status: MissionStatus,
    pub launch_date: String,
    pub destination: String,
    pub spacecraft: Vec<Spacecraft>,
    pub objectives: Vec<String>,
    pub current_phase: MissionPhase,
    pub duration_days: u32,
    pub crew_count: u32,
}

/// Mission status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionStatus {
    Planning,
    PreLaunch,
    Launch,
    Transit,
    Operational,
    Completed,
    Aborted,
}

/// Mission phase enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionPhase {
    Preparation,
    Launch,
    EarthOrbit,
    TranslunarInjection,
    LunarOrbit,
    LunarSurface,
    Return,
    Recovery,
}

/// Spacecraft data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacecraft {
    pub id: String,
    pub name: String,
    pub spacecraft_type: SpacecraftType,
    pub status: SpacecraftStatus,
    pub telemetry: SpacecraftTelemetry,
    pub systems: SystemsStatus,
    pub location: Location,
    pub mission_id: String,
}

/// Spacecraft type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpacecraftType {
    Command,
    Lunar,
    Rover,
    Probe,
    Relay,
}

/// Spacecraft operational status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpacecraftStatus {
    Nominal,
    Caution,
    Warning,
    Critical,
    Offline,
}

/// Real-time spacecraft telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacecraftTelemetry {
    pub timestamp: String,
    pub altitude_km: f64,
    pub velocity_mps: f64,
    pub fuel_percent: f32,
    pub power_watts: f32,
    pub temperature_celsius: f32,
    pub communication_strength: f32,
    pub battery_voltage: f32,
    pub solar_panel_output: f32,
}

/// Systems status overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemsStatus {
    pub propulsion: SystemHealth,
    pub power: SystemHealth,
    pub communication: SystemHealth,
    pub life_support: SystemHealth,
    pub navigation: SystemHealth,
    pub thermal: SystemHealth,
    pub science_instruments: SystemHealth,
}

/// Individual system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealth {
    Normal,
    Degraded,
    Failed,
    Maintenance,
}

/// Spacecraft location in space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub x_km: f64,
    pub y_km: f64,
    pub z_km: f64,
    pub reference_frame: String,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,
}

/// MCP tool definition for space operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub category: ToolCategory,
    pub spacecraft_compatibility: Vec<SpacecraftType>,
    pub execution_time_ms: u64,
}

/// Tool categories for space operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Navigation,
    Science,
    Maintenance,
    Communication,
    Emergency,
}

/// Space mission demo data generator
pub struct SpaceDataGenerator;

impl SpaceDataGenerator {
    /// Generate demo mission data
    pub fn generate_missions() -> Vec<Mission> {
        vec![
            Mission {
                id: "ARTEMIS-III".to_string(),
                name: "Artemis III Lunar Landing".to_string(),
                status: MissionStatus::Operational,
                launch_date: "2026-09-15T12:00:00Z".to_string(),
                destination: "Lunar South Pole".to_string(),
                spacecraft: Self::generate_artemis_spacecraft(),
                objectives: vec![
                    "Land first woman on Moon".to_string(),
                    "Establish lunar base camp".to_string(),
                    "Conduct scientific research".to_string(),
                    "Test Mars mission technologies".to_string(),
                ],
                current_phase: MissionPhase::LunarOrbit,
                duration_days: 30,
                crew_count: 4,
            },
            Mission {
                id: "MARS-PIONEER".to_string(),
                name: "Mars Pioneer Sample Return".to_string(),
                status: MissionStatus::Transit,
                launch_date: "2026-07-22T08:30:00Z".to_string(),
                destination: "Mars - Jezero Crater".to_string(),
                spacecraft: Self::generate_mars_spacecraft(),
                objectives: vec![
                    "Collect Martian soil samples".to_string(),
                    "Search for signs of past life".to_string(),
                    "Return samples to Earth".to_string(),
                    "Map subsurface water".to_string(),
                ],
                current_phase: MissionPhase::LunarOrbit,
                duration_days: 687,
                crew_count: 0,
            },
        ]
    }

    /// Generate Artemis spacecraft
    fn generate_artemis_spacecraft() -> Vec<Spacecraft> {
        vec![
            Spacecraft {
                id: "ORION-CM".to_string(),
                name: "Orion Command Module".to_string(),
                spacecraft_type: SpacecraftType::Command,
                status: SpacecraftStatus::Nominal,
                telemetry: SpacecraftTelemetry {
                    timestamp: "2026-09-20T14:32:15Z".to_string(),
                    altitude_km: 384400.0,
                    velocity_mps: 1023.0,
                    fuel_percent: 78.5,
                    power_watts: 3200.0,
                    temperature_celsius: 22.1,
                    communication_strength: 0.95,
                    battery_voltage: 28.4,
                    solar_panel_output: 2800.0,
                },
                systems: SystemsStatus {
                    propulsion: SystemHealth::Normal,
                    power: SystemHealth::Normal,
                    communication: SystemHealth::Normal,
                    life_support: SystemHealth::Normal,
                    navigation: SystemHealth::Normal,
                    thermal: SystemHealth::Normal,
                    science_instruments: SystemHealth::Normal,
                },
                location: Location {
                    x_km: 0.0,
                    y_km: 384400.0,
                    z_km: 0.0,
                    reference_frame: "Earth-Moon".to_string(),
                    velocity_x: 1023.0,
                    velocity_y: 0.0,
                    velocity_z: 0.0,
                },
                mission_id: "ARTEMIS-III".to_string(),
            },
            Spacecraft {
                id: "STARSHIP-HLS".to_string(),
                name: "Starship Human Landing System".to_string(),
                spacecraft_type: SpacecraftType::Lunar,
                status: SpacecraftStatus::Nominal,
                telemetry: SpacecraftTelemetry {
                    timestamp: "2026-09-20T14:32:15Z".to_string(),
                    altitude_km: 100.0,
                    velocity_mps: 1633.0,
                    fuel_percent: 92.1,
                    power_watts: 4500.0,
                    temperature_celsius: 18.7,
                    communication_strength: 0.88,
                    battery_voltage: 48.2,
                    solar_panel_output: 4200.0,
                },
                systems: SystemsStatus {
                    propulsion: SystemHealth::Normal,
                    power: SystemHealth::Normal,
                    communication: SystemHealth::Normal,
                    life_support: SystemHealth::Normal,
                    navigation: SystemHealth::Normal,
                    thermal: SystemHealth::Normal,
                    science_instruments: SystemHealth::Normal,
                },
                location: Location {
                    x_km: 0.0,
                    y_km: 1837.4,
                    z_km: 100.0,
                    reference_frame: "Lunar".to_string(),
                    velocity_x: 1633.0,
                    velocity_y: 0.0,
                    velocity_z: 0.0,
                },
                mission_id: "ARTEMIS-III".to_string(),
            },
        ]
    }

    /// Generate Mars spacecraft
    fn generate_mars_spacecraft() -> Vec<Spacecraft> {
        vec![
            Spacecraft {
                id: "MARS-ROVER-07".to_string(),
                name: "Mars Pioneer Rover".to_string(),
                spacecraft_type: SpacecraftType::Rover,
                status: SpacecraftStatus::Nominal,
                telemetry: SpacecraftTelemetry {
                    timestamp: "2026-09-20T14:32:15Z".to_string(),
                    altitude_km: 0.0,
                    velocity_mps: 0.05,
                    fuel_percent: 0.0,
                    power_watts: 110.0,
                    temperature_celsius: -45.2,
                    communication_strength: 0.72,
                    battery_voltage: 28.8,
                    solar_panel_output: 95.0,
                },
                systems: SystemsStatus {
                    propulsion: SystemHealth::Normal,
                    power: SystemHealth::Normal,
                    communication: SystemHealth::Normal,
                    life_support: SystemHealth::Failed,
                    navigation: SystemHealth::Normal,
                    thermal: SystemHealth::Degraded,
                    science_instruments: SystemHealth::Normal,
                },
                location: Location {
                    x_km: 0.0,
                    y_km: 0.0,
                    z_km: 0.0,
                    reference_frame: "Mars Surface - Jezero Crater".to_string(),
                    velocity_x: 0.05,
                    velocity_y: 0.0,
                    velocity_z: 0.0,
                },
                mission_id: "MARS-PIONEER".to_string(),
            },
        ]
    }

    /// Generate available space tools for MCP
    pub fn generate_space_tools() -> Vec<SpaceTool> {
        vec![
            SpaceTool {
                name: "drill_sample".to_string(),
                description: "Drill and collect geological samples from surface".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "depth_cm": {"type": "number", "minimum": 1, "maximum": 200},
                        "sample_type": {"type": "string", "enum": ["rock", "soil", "ice"]},
                        "location": {"type": "string"}
                    },
                    "required": ["depth_cm", "sample_type"]
                }),
                category: ToolCategory::Science,
                spacecraft_compatibility: vec![SpacecraftType::Rover],
                execution_time_ms: 45000,
            },
            SpaceTool {
                name: "navigate_to_waypoint".to_string(),
                description: "Navigate spacecraft to specified coordinates".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "latitude": {"type": "number", "minimum": -90, "maximum": 90},
                        "longitude": {"type": "number", "minimum": -180, "maximum": 180},
                        "altitude_m": {"type": "number", "minimum": 0},
                        "speed_mps": {"type": "number", "minimum": 0.01, "maximum": 2.0}
                    },
                    "required": ["latitude", "longitude"]
                }),
                category: ToolCategory::Navigation,
                spacecraft_compatibility: vec![SpacecraftType::Rover, SpacecraftType::Probe],
                execution_time_ms: 30000,
            },
            SpaceTool {
                name: "capture_panorama".to_string(),
                description: "Capture 360-degree panoramic images of surroundings".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "resolution": {"type": "string", "enum": ["low", "medium", "high", "ultra"]},
                        "filters": {"type": "array", "items": {"type": "string"}},
                        "stitching": {"type": "boolean"}
                    }
                }),
                category: ToolCategory::Science,
                spacecraft_compatibility: vec![SpacecraftType::Rover, SpacecraftType::Lunar, SpacecraftType::Probe],
                execution_time_ms: 25000,
            },
            SpaceTool {
                name: "emergency_contact_earth".to_string(),
                description: "Establish emergency communication with mission control".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "priority": {"type": "string", "enum": ["low", "medium", "high", "critical"]},
                        "message": {"type": "string", "maxLength": 500},
                        "require_acknowledgment": {"type": "boolean"}
                    },
                    "required": ["priority", "message"]
                }),
                category: ToolCategory::Emergency,
                spacecraft_compatibility: vec![SpacecraftType::Command, SpacecraftType::Lunar, SpacecraftType::Rover, SpacecraftType::Probe],
                execution_time_ms: 5000,
            },
            SpaceTool {
                name: "system_diagnostic".to_string(),
                description: "Run comprehensive diagnostic on spacecraft systems".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "systems": {"type": "array", "items": {"type": "string"}},
                        "deep_scan": {"type": "boolean"},
                        "generate_report": {"type": "boolean"}
                    }
                }),
                category: ToolCategory::Maintenance,
                spacecraft_compatibility: vec![SpacecraftType::Command, SpacecraftType::Lunar, SpacecraftType::Rover, SpacecraftType::Probe],
                execution_time_ms: 60000,
            },
        ]
    }

    /// Update telemetry with realistic variations
    pub fn update_telemetry(telemetry: &mut SpacecraftTelemetry) {
        // Time-based oscillation simulation
        let time_factor = chrono::Utc::now().timestamp() as f32 * 0.001;
        
        // Add realistic variations to telemetry
        telemetry.fuel_percent = (telemetry.fuel_percent - 0.01).max(0.0);
        telemetry.power_watts += time_factor.sin() * 50.0;
        telemetry.temperature_celsius += time_factor.cos() * 2.0;
        telemetry.communication_strength = 0.7 + 0.25 * (time_factor * 0.1).sin();
        telemetry.solar_panel_output = telemetry.power_watts * 0.85;
        telemetry.timestamp = chrono::Utc::now().to_rfc3339();
    }
}