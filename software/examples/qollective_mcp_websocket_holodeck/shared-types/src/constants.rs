// ABOUTME: Centralized constants for all holodeck components - port management and configuration
// ABOUTME: CRITICAL - NO hardcoded values allowed anywhere except this file

use std::time::Duration;

/// Network configuration constants - ALL ports centrally managed
pub mod network {
    pub const HOLODECK_DESKTOP_PORT: u16 = 0; // N/A - desktop app
    pub const HOLODECK_COORDINATOR_PORT: u16 = 8447;
    pub const HOLODECK_STORYBOOK_PORT: u16 = 8080;
    pub const HOLODECK_DESIGNER_PORT: u16 = 8443;
    pub const HOLODECK_VALIDATOR_PORT: u16 = 8444;
    pub const HOLODECK_ENVIRONMENT_PORT: u16 = 8445;  
    pub const HOLODECK_SAFETY_PORT: u16 = 8446;
    pub const HOLODECK_CHARACTER_PORT: u16 = 8448;
    
    // Default hosts
    pub const DEFAULT_HOST: &str = "127.0.0.1";
    pub const DEFAULT_WEBSOCKET_HOST: &str = "localhost";
    
    // Protocol prefixes
    pub const MCP_PROTOCOL_PREFIX: &str = "mcp://";
    pub const WEBSOCKET_PROTOCOL_PREFIX: &str = "ws://";
    pub const WEBSOCKET_SECURE_PREFIX: &str = "wss://";
    pub const HTTP_PROTOCOL_PREFIX: &str = "http://";
    pub const HTTPS_PROTOCOL_PREFIX: &str = "https://";
    
    // URL construction helpers
    pub fn coordinator_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_COORDINATOR_PORT)
    }
    
    pub fn coordinator_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_COORDINATOR_PORT)
    }
    
    pub fn storybook_url() -> String {
        format!("{}{}:{}", HTTP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_STORYBOOK_PORT)
    }
    
    pub fn storybook_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_STORYBOOK_PORT)
    }
    
    pub fn designer_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_DESIGNER_PORT)
    }
    
    pub fn validator_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_VALIDATOR_PORT)
    }
    
    pub fn environment_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_ENVIRONMENT_PORT)
    }
    
    pub fn safety_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_SAFETY_PORT)
    }
    
    pub fn character_mcp_url() -> String {
        format!("{}{}:{}", MCP_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_CHARACTER_PORT)
    }
    
    pub fn character_websocket_url() -> String {
        format!("{}{}:{}/mcp", WEBSOCKET_PROTOCOL_PREFIX, DEFAULT_HOST, HOLODECK_CHARACTER_PORT)
    }
}

/// Timeout configuration constants
pub mod timeouts {
    use super::Duration;
    
    pub const DEFAULT_MCP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_WEBSOCKET_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(15);
    pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
    pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    pub const DEFAULT_RECONNECTION_DELAY: Duration = Duration::from_secs(5);
    pub const MAX_RECONNECTION_ATTEMPTS: u32 = 5;
}

/// MCP subject constants  
pub mod subjects {
    pub const HOLODECK_STORY_GENERATE: &str = "holodeck.story.generate";
    pub const HOLODECK_STORY_VALIDATE: &str = "holodeck.story.validate"; 
    pub const HOLODECK_CANON_CHECK: &str = "holodeck.canon.check";
    pub const HOLODECK_CHARACTER_VALIDATE: &str = "holodeck.character.validate";
    pub const HOLODECK_ENVIRONMENT_CREATE: &str = "holodeck.environment.create";
    pub const HOLODECK_ENVIRONMENT_UPDATE: &str = "holodeck.environment.update";
    pub const HOLODECK_ENVIRONMENT_GENERATE: &str = "holodeck.environment.generate";
    pub const HOLODECK_ENVIRONMENT_MANAGE: &str = "holodeck.environment.manage";
    pub const HOLODECK_ENVIRONMENT_VALIDATE: &str = "holodeck.environment.validate";
    pub const HOLODECK_SAFETY_CHECK: &str = "holodeck.safety.check";
    pub const HOLODECK_SAFETY_MONITOR: &str = "holodeck.safety.monitor";
    pub const HOLODECK_CHARACTER_INTERACT: &str = "holodeck.character.interact";
    pub const HOLODECK_CHARACTER_PROFILE: &str = "holodeck.character.profile";
    pub const HOLODECK_COORDINATOR_SESSION: &str = "holodeck.coordinator.session";
    pub const HOLODECK_COORDINATOR_HEALTH: &str = "holodeck.coordinator.health";
    pub const HOLODECK_COORDINATOR_DISCOVERY: &str = "holodeck.coordinator.discovery";
    pub const HOLODECK_COORDINATOR_VALIDATE: &str = "holodeck.coordinator.validate";
    pub const HOLODECK_SESSION_START: &str = "holodeck.session.start";
    pub const HOLODECK_SESSION_UPDATE: &str = "holodeck.session.update";
    pub const HOLODECK_SESSION_END: &str = "holodeck.session.end";
    pub const HOLODECK_HEALTH_CHECK: &str = "holodeck.health.check";
}

/// Component limits and constraints
pub mod limits {
    pub const MAX_STORY_SCENES: u32 = 10;
    pub const MIN_STORY_SCENES: u32 = 1;
    pub const MAX_CHARACTERS_PER_STORY: u32 = 5;
    pub const MIN_CHARACTERS_PER_STORY: u32 = 1;
    pub const MAX_SCENE_WORD_COUNT: u32 = 300;
    pub const MIN_SCENE_WORD_COUNT: u32 = 50;
    pub const MAX_CONCURRENT_SESSIONS: u32 = 100;
    pub const MAX_SESSION_DURATION_HOURS: u32 = 8;
    pub const MAX_STORY_TEMPLATE_SIZE_KB: u32 = 500;
    pub const MAX_CHARACTER_DATABASE_SIZE: u32 = 1000;
    pub const MAX_ENVIRONMENT_VARIANTS: u32 = 50;
}

/// Service version information
pub mod versions {
    pub const HOLODECK_VERSION: &str = "0.1.0";
    pub const API_VERSION: &str = "v1";
    pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26"; // Latest MCP protocol version
    pub const BUILD_INFO: &str = "Phase 5 - Full LLM Integration";
}

/// Development and testing constants
pub mod testing {
    pub const TEST_DATABASE_PATH: &str = ":memory:"; // SQLite in-memory for tests
    pub const TEST_CHARACTER_COUNT: u32 = 3;
    pub const TEST_STORY_SCENE_COUNT: u32 = 2;
    pub const TEST_SESSION_TIMEOUT_MS: u64 = 5000; // 5 seconds for tests
    pub const MOCK_DELAY_MS: u64 = 100; // Simulate processing time
}

/// Service names and identifiers
pub mod services {
    pub const COORDINATOR_SERVICE_NAME: &str = "holodeck-coordinator";
    pub const STORYBOOK_SERVICE_NAME: &str = "holodeck-storybook";
    pub const DESIGNER_SERVICE_NAME: &str = "holodeck-designer";
    pub const VALIDATOR_SERVICE_NAME: &str = "holodeck-validator";
    pub const ENVIRONMENT_SERVICE_NAME: &str = "holodeck-environment";
    pub const SAFETY_SERVICE_NAME: &str = "holodeck-safety";
    pub const CHARACTER_SERVICE_NAME: &str = "holodeck-character";
    pub const DESKTOP_SERVICE_NAME: &str = "holodeck-desktop";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_port_assignments() {
        // Verify all ports are unique and in expected range
        let ports = vec![
            network::HOLODECK_COORDINATOR_PORT,
            network::HOLODECK_STORYBOOK_PORT,
            network::HOLODECK_DESIGNER_PORT,
            network::HOLODECK_VALIDATOR_PORT,
            network::HOLODECK_ENVIRONMENT_PORT,
            network::HOLODECK_SAFETY_PORT,
            network::HOLODECK_CHARACTER_PORT,
        ];
        
        // Check uniqueness
        let mut unique_ports = ports.clone();
        unique_ports.sort();
        unique_ports.dedup();
        assert_eq!(ports.len(), unique_ports.len(), "All ports must be unique");
        
        // Check range (8080 and 8443-8448)
        for port in ports {
            assert!(port == 8080 || (port >= 8443 && port <= 8448), 
                   "Port {} is not in expected range", port);
        }
    }
    
    #[test]
    fn test_url_generation() {
        assert_eq!(network::coordinator_url(), "mcp://127.0.0.1:8447");
        assert_eq!(network::storybook_url(), "http://127.0.0.1:8080");
        assert_eq!(network::designer_mcp_url(), "mcp://127.0.0.1:8443");
    }
    
    #[test]
    fn test_timeout_values() {
        assert!(timeouts::DEFAULT_MCP_REQUEST_TIMEOUT.as_secs() > 0);
        assert!(timeouts::DEFAULT_STARTUP_TIMEOUT.as_secs() > timeouts::DEFAULT_MCP_REQUEST_TIMEOUT.as_secs());
    }
    
    #[test]
    fn test_limits_sanity() {
        assert!(limits::MIN_STORY_SCENES <= limits::MAX_STORY_SCENES);
        assert!(limits::MIN_CHARACTERS_PER_STORY <= limits::MAX_CHARACTERS_PER_STORY);
        assert!(limits::MIN_SCENE_WORD_COUNT <= limits::MAX_SCENE_WORD_COUNT);
    }
}