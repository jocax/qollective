//! Constants for the Qollective A2A Enterprise example
//! 
//! Following the CONSTANTS FIRST principle, all hardcoded values are defined here
//! before use in the application code.

/// Environment variable constants for TLS configuration override
pub mod tls_env {
    /// Base path for TLS certificates directory
    pub const TLS_CERT_BASE_PATH: &str = "TLS_CERT_BASE_PATH";
    
    /// CA certificate file path override
    pub const TLS_CA_CERT_PATH: &str = "TLS_CA_CERT_PATH";
    
    /// Client certificate file path override  
    pub const TLS_CERT_PATH: &str = "TLS_CERT_PATH";
    
    /// Private key file path override
    pub const TLS_KEY_PATH: &str = "TLS_KEY_PATH";
    
    /// TLS verification mode override (mutual_tls, skip)
    pub const TLS_VERIFICATION_MODE: &str = "TLS_VERIFICATION_MODE";
    
    /// TLS insecure mode override (true, false)
    pub const TLS_INSECURE: &str = "TLS_INSECURE";
}

/// Configuration file constants
pub mod config {
    /// Default configuration file name
    pub const DEFAULT_CONFIG_FILE: &str = "config.toml";
    
    /// Environment variable for configuration file path override
    pub const CONFIG_FILE_PATH: &str = "CONFIG_FILE_PATH";
}

/// Enterprise application constants
pub mod enterprise {
    /// Default server ID if not specified in config
    pub const DEFAULT_SERVER_ID: &str = "enterprise-nx-74205";
    
    /// Default server name if not specified in config
    pub const DEFAULT_SERVER_NAME: &str = "USS Defiant";
    
    /// Default ship registry if not specified in config
    pub const DEFAULT_SHIP_REGISTRY: &str = "NX-74205";
    
    /// Default ship class if not specified in config
    pub const DEFAULT_SHIP_CLASS: &str = "Defiant";
}

/// NATS connection constants
pub mod nats {
    /// Default NATS server URL
    pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";
    
    /// Default NATS client name prefix
    pub const DEFAULT_CLIENT_NAME_PREFIX: &str = "qollective-enterprise";
    
    /// Default connection timeout in milliseconds
    pub const DEFAULT_CONNECTION_TIMEOUT_MS: u64 = 5000;
    
    /// Default reconnect timeout in milliseconds
    pub const DEFAULT_RECONNECT_TIMEOUT_MS: u64 = 2000;
    
    /// Default maximum reconnect attempts
    pub const DEFAULT_MAX_RECONNECT_ATTEMPTS: u32 = 5;
}

/// A2A (Agent-to-Agent) communication constants
pub mod a2a {
    /// Default subject prefix for agent communication
    pub const DEFAULT_SUBJECT_PREFIX: &str = "qollective.agents";
    
    /// Default agent TTL in seconds
    pub const DEFAULT_AGENT_TTL_SECS: u64 = 300;
    
    /// Default cleanup interval in seconds
    pub const DEFAULT_CLEANUP_INTERVAL_SECS: u64 = 60;
    
    /// Default maximum number of agents
    pub const DEFAULT_MAX_AGENTS: usize = 1000;
    
    /// Default heartbeat interval in seconds
    pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;
    
    /// Default discovery cache TTL in seconds
    pub const DEFAULT_DISCOVERY_CACHE_TTL_SECS: u64 = 60;
}

/// Logging and monitoring constants
pub mod logging {
    /// Default log level
    pub const DEFAULT_LOG_LEVEL: &str = "info";
    
    /// Default log format
    pub const DEFAULT_LOG_FORMAT: &str = "pretty";
    
    /// Default metrics interval in seconds
    pub const DEFAULT_METRICS_INTERVAL_SECS: u64 = 60;
}

/// Agent capability constants
pub mod capabilities {
    /// Logging agent capability name
    pub const LOGGING_CAPABILITY: &str = "logging";
    
    /// Command capability for senior officers
    pub const COMMAND_CAPABILITY: &str = "command";
    
    /// Engineering capability
    pub const ENGINEERING_CAPABILITY: &str = "engineering";
    
    /// Science capability
    pub const SCIENCE_CAPABILITY: &str = "science";
    
    /// Security capability
    pub const SECURITY_CAPABILITY: &str = "security";
    
    /// Diplomatic capability
    pub const DIPLOMACY_CAPABILITY: &str = "diplomacy";
    
    /// Medical capability
    pub const MEDICAL_CAPABILITY: &str = "medical";
}

/// Star Trek Enterprise crew member constants
pub mod crew {
    /// Captain Jean-Luc Picard
    pub const PICARD_AGENT_ID: &str = "picard";
    pub const PICARD_AGENT_NAME: &str = "Captain Jean-Luc Picard";
    pub const PICARD_LOCATION: &str = "Bridge";
    pub const PICARD_FUNCTION: &str = "Captain";
    
    /// Chief Engineer Montgomery Scott
    pub const SCOTTY_AGENT_ID: &str = "scotty";
    pub const SCOTTY_AGENT_NAME: &str = "Chief Engineer Montgomery Scott";
    pub const SCOTTY_LOCATION: &str = "Engineering";
    pub const SCOTTY_FUNCTION: &str = "Chief Engineer";
    
    /// Lieutenant Commander Data
    pub const DATA_AGENT_ID: &str = "data";
    pub const DATA_AGENT_NAME: &str = "Lieutenant Commander Data";
    pub const DATA_LOCATION: &str = "Bridge";
    pub const DATA_FUNCTION: &str = "Operations Officer";
    
    /// Science Officer Spock
    pub const SPOCK_AGENT_ID: &str = "spock";
    pub const SPOCK_AGENT_NAME: &str = "Science Officer Spock";
    pub const SPOCK_LOCATION: &str = "Science Lab";
    pub const SPOCK_FUNCTION: &str = "Science Officer";
}

/// Q Console constants (omnipotent being interface)
pub mod q_console {
    /// Q Console agent ID
    pub const Q_AGENT_ID: &str = "q_console";
    
    /// Q Console agent name
    pub const Q_AGENT_NAME: &str = "Q Console";
    
    /// Q Console location (Q Continuum)
    pub const Q_LOCATION: &str = "Q Continuum";
    
    /// Q Console function
    pub const Q_FUNCTION: &str = "Omnipotent Observer";
    
    /// Q Console service type
    pub const Q_SERVICE_TYPE: &str = "omnipotent";
}