// ABOUTME: Common MCP server metadata and types to avoid duplication across servers
// ABOUTME: Provides standardized server information structures for health checks and monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Common server metadata used across all MCP servers for health checks and monitoring
/// This avoids duplicating ServerInfo structures in each server component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub service_name: String,
    pub version: String,
    pub port: u16,
    pub started_at: DateTime<Utc>,
}

impl ServerMetadata {
    /// Create new server metadata with current timestamp
    pub fn new(service_name: String, version: String, port: u16) -> Self {
        Self {
            service_name,
            version,
            port,
            started_at: Utc::now(),
        }
    }

    /// Calculate server uptime in seconds
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }
}

/// Health status response structure used by all MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub version: String,
    pub port: u16,
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: i64,
    pub timestamp: DateTime<Utc>,
    pub build_info: String,
    pub mcp_protocol_version: String,
}

impl From<&ServerMetadata> for HealthStatus {
    fn from(metadata: &ServerMetadata) -> Self {
        let now = Utc::now();
        Self {
            status: "healthy".to_string(),
            service: metadata.service_name.clone(),
            version: metadata.version.clone(),
            port: metadata.port,
            started_at: metadata.started_at,
            uptime_seconds: (now - metadata.started_at).num_seconds(),
            timestamp: now,
            build_info: crate::constants::versions::BUILD_INFO.to_string(),
            mcp_protocol_version: crate::constants::versions::MCP_PROTOCOL_VERSION.to_string(),
        }
    }
}