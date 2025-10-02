// ABOUTME: Reusable StarTrek Enterprise agent patterns extracted from log-agent
// ABOUTME: Demonstrates how to build robust agents using Qollective without extending framework scope

//! StarTrek Enterprise Agent Framework
//!
//! This module contains reusable patterns extracted from the log-agent that can be
//! used by all Enterprise agents. It serves as an example of how to build robust
//! agents on top of Qollective without extending the framework's scope.
//!
//! **Reusable Patterns Included:**
//! - Persistent agent identity management
//! - Intelligent connection monitoring and reconnection
//! - Rate limit aware registration with delays
//! - Standardized NATS configuration with TLS
//! - Graceful shutdown and cleanup
//! - Enterprise-themed agent metadata and personality

use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime},
};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use futures::StreamExt;

use qollective::{
    constants::{network, subjects},
    client::a2a::A2AClient,
    client::nats::NatsClient,
    config::a2a::{AgentClientConfig, A2AClientConfig},
    types::a2a::{AgentInfo, HealthStatus},
    client::a2a::{AgentMetadata, AgentProviderInfo},
    envelope::{Envelope, Context},
    error::Result,
};

// ============================================================================
// HYBRID MESSAGE TYPES AND PATTERNS
// ============================================================================

/// Message type detection for hybrid envelope/raw message handling
#[derive(Debug, Clone)]
pub enum MessageType {
    /// Qollective envelope with metadata, tenant info, extensions
    Envelope(Envelope<serde_json::Value>),
    /// Raw NATS message payload for legacy integration
    Raw(Vec<u8>),
}

/// Trait for handling different message types in Enterprise agents
#[async_trait]
pub trait HybridMessageHandler {
    /// Handle Qollective envelope messages with rich metadata
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()>;

    /// Handle raw NATS messages for legacy integration
    async fn handle_raw_message(&self, payload: Vec<u8>) -> Result<()>;
}

// ============================================================================
// REUSABLE AGENT PATTERNS
// ============================================================================

/// Persistent agent identity management
///
/// Provides stable agent identity across restarts, crucial for production
/// deployments where rate limiting and registration state matter.
pub struct AgentIdentity {
    pub id: Uuid,
    id_file_path: String,
}

impl AgentIdentity {
    /// Create or load persistent agent identity
    pub fn new(agent_name: &str) -> Self {
        let id_filename = format!("{}.id", agent_name.to_lowercase().replace(' ', "-"));
        let id_file_path = format!("target/{}", id_filename);
        let id = Self::get_or_create_persistent_id(&id_file_path);

        Self { id, id_file_path }
    }

    /// Get or create persistent agent ID
    fn get_or_create_persistent_id(id_file_path: &str) -> Uuid {
        // Try to read existing agent ID
        if Path::new(id_file_path).exists() {
            if let Ok(id_str) = fs::read_to_string(id_file_path) {
                if let Ok(uuid) = Uuid::parse_str(id_str.trim()) {
                    tracing::info!("Using persistent agent ID: {} from {}", uuid, id_file_path);
                    return uuid;
                } else {
                    tracing::warn!("Invalid UUID in agent ID file {}, generating new one", id_file_path);
                }
            } else {
                tracing::warn!("Could not read agent ID file {}, generating new one", id_file_path);
            }
        }

        // Generate new agent ID and save it
        let new_id = Uuid::now_v7();
        if let Err(e) = fs::write(id_file_path, new_id.to_string()) {
            tracing::error!("Failed to save agent ID to file {}: {}", id_file_path, e);
        } else {
            tracing::info!("Generated and saved new persistent agent ID: {} to {}", new_id, id_file_path);
        }

        new_id
    }
}

/// Enterprise NATS configuration with centralized config.toml support
///
/// Provides standardized NATS connection logic using the centralized Enterprise configuration
/// with graceful fallback to non-TLS for development environments.
pub struct EnterpriseNatsConfig;

impl EnterpriseNatsConfig {
    /// Load configuration from config.toml and create connection configs with fallback
    pub fn connection_configs() -> Vec<qollective::config::nats::NatsClientConfig> {
        // Try to load centralized configuration first
        match crate::config::EnterpriseConfig::load_default() {
            Ok(config) => {
                tracing::info!("🔧 Using centralized configuration from config.toml");
                vec![
                    // Primary configuration from config.toml
                    config.nats.to_framework_client_config(&config.tls),
                    // Non-TLS fallback configuration
                    qollective::config::nats::NatsClientConfig {
                        connection: qollective::config::nats::NatsConnectionConfig {
                            urls: vec![network::DEFAULT_NATS_URL.to_string()],
                            tls: qollective::config::tls::TlsConfig {
                                enabled: false,
                                ca_cert_path: None,
                                cert_path: None,
                                key_path: None,
                                verification_mode: qollective::config::tls::VerificationMode::Skip,
                            },
                            ..Default::default()
                        },
                        client_behavior: qollective::config::nats::NatsClientBehaviorConfig::default(),
                        discovery_cache_ttl_ms: 300000, // 5 minutes
                    },
                ]
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to load config.toml ({}), using hardcoded fallback", e);
                // Fallback to original hardcoded configurations
                vec![
                    // TLS configuration with mTLS
                    qollective::config::nats::NatsClientConfig {
                        connection: qollective::config::nats::NatsConnectionConfig {
                            urls: vec![network::DEFAULT_NATS_TLS_URL.to_string()],
                            tls: qollective::config::tls::TlsConfig {
                                enabled: true,
                                ca_cert_path: Some(network::tls_paths::default_ca_file().into()),
                                cert_path: Some(network::tls_paths::default_cert_file().into()),
                                key_path: Some(network::tls_paths::default_key_file().into()),
                                verification_mode: qollective::config::tls::VerificationMode::MutualTls,
                            },
                            ..Default::default()
                        },
                        client_behavior: qollective::config::nats::NatsClientBehaviorConfig::default(),
                        discovery_cache_ttl_ms: 300000, // 5 minutes
                    },
                    // Non-TLS fallback configuration
                    qollective::config::nats::NatsClientConfig {
                        connection: qollective::config::nats::NatsConnectionConfig {
                            urls: vec![network::DEFAULT_NATS_URL.to_string()],
                            tls: qollective::config::tls::TlsConfig {
                                enabled: false,
                                ca_cert_path: None,
                                cert_path: None,
                                key_path: None,
                                verification_mode: qollective::config::tls::VerificationMode::Skip,
                            },
                            ..Default::default()
                        },
                        client_behavior: qollective::config::nats::NatsClientBehaviorConfig::default(),
                        discovery_cache_ttl_ms: 300000, // 5 minutes
                    },
                ]
            }
        }
    }

    /// Attempt NATS connection with fallback logic
    pub async fn connect() -> Result<NatsClient> {
        let connection_configs = Self::connection_configs();

        for nats_client_config in connection_configs {
            let url = nats_client_config.connection.urls[0].clone();
            tracing::debug!("Attempting NATS connection to: {}", url);

            // Convert NatsClientConfig to NatsConfig for NatsClient::new()
            let nats_config = qollective::config::nats::NatsConfig {
                connection: nats_client_config.connection,
                client: nats_client_config.client_behavior,
                server: qollective::config::nats::NatsServerConfig::default(),
                discovery: qollective::config::nats::NatsDiscoveryConfig {
                    enabled: true,
                    ttl_ms: nats_client_config.discovery_cache_ttl_ms,
                    ..Default::default()
                },
            };

            match NatsClient::new(nats_config).await {
                Ok(client) => {
                    tracing::info!("Successfully connected to NATS: {}", url);
                    return Ok(client);
                }
                Err(e) => {
                    tracing::warn!("NATS connection failed to {}: {}", url, e);
                }
            }
        }

        Err(qollective::error::QollectiveError::nats_connection("Could not connect to any NATS server".to_string()))
    }
}

/// Intelligent connection monitoring and reconnection
///
/// Handles server failures gracefully with exponential backoff, health monitoring,
/// and automatic re-registration when servers recover. Includes rate limit awareness
/// to avoid hitting server-side rate limits on reconnection.
pub struct ConnectionMonitor {
    a2a_client: Arc<A2AClient>,
    agent_info: AgentInfo,
    health_check_interval: Duration,
}

impl ConnectionMonitor {
    /// Create new connection monitor
    pub fn new(a2a_client: Arc<A2AClient>, agent_info: AgentInfo) -> Self {
        Self {
            a2a_client,
            agent_info,
            health_check_interval: Duration::from_secs(30),
        }
    }

    /// Start intelligent connection monitoring task
    pub async fn start_monitoring(self) {
        tokio::spawn(async move {
            self.run_monitoring_loop().await;
        });
    }

    /// Main monitoring loop with intelligent reconnection
    async fn run_monitoring_loop(self) {
        let mut health_interval = tokio::time::interval(self.health_check_interval);
        let mut reconnection_needed = false;
        let mut consecutive_failures = 0u32;
        let mut last_successful_contact = SystemTime::now();

        // Consume the immediate first tick to avoid instant execution
        health_interval.tick().await;
        tracing::info!("Connection monitoring started with {:?} health checks", self.health_check_interval);

        loop {
            health_interval.tick().await;

            // Create health metadata
            let health_metadata = self.create_health_metadata(consecutive_failures);

            match self.a2a_client.publish_health_status(self.agent_info.id, HealthStatus::Healthy, Some(health_metadata)).await {
                Ok(_) => {
                    // Health update successful - server is reachable
                    if reconnection_needed {
                        // Server came back online - wait before attempting re-registration to avoid rate limits
                        tracing::info!("Server connectivity restored after {} failures, waiting 10 seconds before re-registration to avoid rate limits", consecutive_failures);
                        tokio::time::sleep(Duration::from_secs(10)).await;

                        let reconnection_metadata = self.create_reconnection_metadata(last_successful_contact);

                        tracing::info!("Attempting agent re-registration after rate limit avoidance delay");
                        match self.a2a_client.register_agent(self.agent_info.clone(), reconnection_metadata).await {
                            Ok(_) => {
                                tracing::info!("Agent successfully re-registered after server recovery");
                                reconnection_needed = false;
                                consecutive_failures = 0;
                                last_successful_contact = SystemTime::now();
                            }
                            Err(e) => {
                                tracing::warn!("Re-registration failed despite server being reachable: {}", e);
                            }
                        }
                    } else {
                        // Normal health update successful
                        consecutive_failures = 0;
                        last_successful_contact = SystemTime::now();
                        tracing::debug!("Agent health update successful");
                    }
                }
                Err(e) => {
                    // Health update failed - server may be down
                    consecutive_failures += 1;
                    tracing::warn!("Agent health update failed (attempt {}): {}", consecutive_failures, e);

                    if consecutive_failures >= 3 && !reconnection_needed {
                        // Server appears to be down after 3 consecutive failures
                        reconnection_needed = true;
                        tracing::error!("Server appears to be down after {} consecutive failures - will attempt re-registration when server recovers", consecutive_failures);
                    }

                    // Exponential backoff for health checks when server is down
                    if consecutive_failures > 5 {
                        let backoff_seconds = std::cmp::min(300, 30 * (2_u64.pow(consecutive_failures.min(5))));
                        tracing::info!("Extending health check interval to {} seconds due to server unavailability", backoff_seconds);
                        tokio::time::sleep(Duration::from_secs(backoff_seconds - 30)).await; // Subtract 30 since interval will add it
                    }
                }
            }
        }
    }

    /// Create health metadata for status updates
    fn create_health_metadata(&self, consecutive_failures: u32) -> AgentMetadata {
        AgentMetadata {
            version: "1.0.0".to_string(),
            build_info: Some(format!("{} - Qollective Enterprise Agent", self.agent_info.name)),
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::from([
                ("service_type".to_string(), serde_json::Value::String("enterprise_agent".to_string())),
                ("ship".to_string(), serde_json::Value::String("USS Enterprise NCC-1701-D".to_string())),
                ("last_health_check".to_string(), serde_json::Value::String(SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string())),
                ("consecutive_failures".to_string(), serde_json::Value::Number(serde_json::Number::from(consecutive_failures))),
            ]),
        }
    }

    /// Create reconnection metadata with downtime information
    fn create_reconnection_metadata(&self, last_successful_contact: SystemTime) -> AgentMetadata {
        AgentMetadata {
            version: "1.0.0".to_string(),
            build_info: Some(format!("{} - Qollective Enterprise Agent", self.agent_info.name)),
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::from([
                ("service_type".to_string(), serde_json::Value::String("enterprise_agent".to_string())),
                ("ship".to_string(), serde_json::Value::String("USS Enterprise NCC-1701-D".to_string())),
                ("reconnection_after_failure".to_string(), serde_json::Value::Bool(true)),
                ("downtime_seconds".to_string(), serde_json::Value::Number(serde_json::Number::from(
                    SystemTime::now().duration_since(last_successful_contact).unwrap_or_default().as_secs()
                ))),
            ]),
        }
    }
}

/// Enterprise agent metadata factory
///
/// Provides standardized metadata creation for Enterprise agents with
/// consistent ship information and versioning.
pub struct EnterpriseMetadata;

impl EnterpriseMetadata {
    /// Create standard Enterprise agent metadata
    pub fn create(service_type: &str, function_description: &str) -> AgentMetadata {
        AgentMetadata {
            version: "1.0.0".to_string(),
            build_info: Some("Qollective Enterprise Agent - Starfleet Computer Systems".to_string()),
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::from([
                ("service_type".to_string(), serde_json::Value::String(service_type.to_string())),
                ("ship".to_string(), serde_json::Value::String("USS Enterprise NCC-1701-D".to_string())),
                ("function".to_string(), serde_json::Value::String(function_description.to_string())),
                ("starfleet_division".to_string(), serde_json::Value::String("Computer Systems".to_string())),
                ("registry".to_string(), serde_json::Value::String("NCC-1701-D".to_string())),
                ("class".to_string(), serde_json::Value::String("Galaxy-class".to_string())),
            ]),
        }
    }

    /// Create standard Enterprise agent provider info
    pub fn create_provider() -> AgentProviderInfo {
        AgentProviderInfo {
            name: "Starfleet Computer Systems".to_string(),
            url: Some("https://starfleet.federation.gov/systems".to_string()),
            contact: Some("systems.admin@starfleet.gov".to_string()),
        }
    }
}

/// Enterprise agent builder
///
/// Provides a consistent way to create Enterprise agents with all the
/// standard patterns: persistent identity, connection monitoring, metadata, etc.
pub struct EnterpriseAgentBuilder {
    name: String,
    capabilities: Vec<String>,
    function_description: String,
    location: String,
    service_type: String,
}

impl EnterpriseAgentBuilder {
    /// Start building a new Enterprise agent
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            capabilities: Vec::new(),
            function_description: String::new(),
            location: "USS Enterprise NCC-1701-D".to_string(),
            service_type: "enterprise_agent".to_string(),
        }
    }

    /// Add capabilities to the agent
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set function description
    pub fn with_function(mut self, description: &str) -> Self {
        self.function_description = description.to_string();
        self
    }

    /// Set agent location
    pub fn with_location(mut self, location: &str) -> Self {
        self.location = location.to_string();
        self
    }

    /// Set service type for metadata
    pub fn with_service_type(mut self, service_type: &str) -> Self {
        self.service_type = service_type.to_string();
        self
    }

    /// Build the complete enterprise agent setup
    pub async fn build(self) -> Result<EnterpriseAgent> {
        // Create persistent identity
        let identity = AgentIdentity::new(&self.name);

        // Create agent info
        let agent_info = AgentInfo {
            id: identity.id,
            name: self.name.clone(),
            capabilities: self.capabilities,
            health_status: HealthStatus::Healthy,
            last_heartbeat: SystemTime::now(),
            metadata: HashMap::from([
                ("location".to_string(), self.location),
                ("function".to_string(), self.function_description.clone()),
                ("species".to_string(), "AI System".to_string()),
                ("vessel".to_string(), "USS Enterprise NCC-1701-D".to_string()),
            ]),
        };

        // Connect to NATS
        let nats_client = EnterpriseNatsConfig::connect().await?;

        // Create A2A client configuration
        let working_nats_config = EnterpriseNatsConfig::connection_configs()[0].clone();
        let a2a_client_config = A2AClientConfig {
            client: AgentClientConfig {
                agent_id: agent_info.id.to_string(),
                agent_name: agent_info.name.clone(),
                capabilities: agent_info.capabilities.clone(),
                nats_url: working_nats_config.connection.urls[0].clone(),
                ..Default::default()
            },
            transport: Default::default(),
            nats_client: working_nats_config,
            discovery_cache_ttl_ms: 300000, // 5 minutes
        };

        // Create A2A client
        let a2a_client = Arc::new(A2AClient::new(a2a_client_config).await?);

        // Perform initial registration
        let initial_metadata = EnterpriseMetadata::create(&self.service_type, &self.function_description);

        tracing::info!("Attempting initial Enterprise agent registration: {}", self.name);
        match a2a_client.register_agent(agent_info.clone(), initial_metadata).await {
            Ok(_) => {
                tracing::info!("Enterprise agent registration successful: {}", self.name);
            },
            Err(e) => {
                tracing::warn!("Initial Enterprise agent registration failed: {}", e);
            },
        }

        // Start connection monitoring
        let connection_monitor = ConnectionMonitor::new(a2a_client.clone(), agent_info.clone());
        connection_monitor.start_monitoring().await;

        Ok(EnterpriseAgent {
            identity,
            agent_info,
            nats_client,
            a2a_client,
        })
    }
}

/// Complete Enterprise agent with all reusable patterns
///
/// This struct contains all the components needed for a robust Enterprise agent:
/// persistent identity, NATS connection, A2A client, and connection monitoring.
pub struct EnterpriseAgent {
    pub identity: AgentIdentity,
    pub agent_info: AgentInfo,
    pub nats_client: NatsClient,
    pub a2a_client: Arc<A2AClient>,
}

impl EnterpriseAgent {
    /// Create a new Enterprise agent builder
    pub fn builder(name: &str) -> EnterpriseAgentBuilder {
        EnterpriseAgentBuilder::new(name)
    }

    /// Handle incoming message with automatic type detection (HybridAgent pattern)
    ///
    /// This is the core HybridAgent functionality that automatically detects whether
    /// an incoming message is a Qollective envelope or raw NATS payload and routes
    /// it to the appropriate handler. This enables Enterprise agents to seamlessly
    /// work with both modern envelope-based workflows and legacy raw message systems.
    pub async fn handle_message<T>(&self, raw_payload: Vec<u8>, handler: &T) -> Result<()>
    where
        T: HybridMessageHandler,
    {
        match self.detect_message_type(&raw_payload)? {
            MessageType::Envelope(envelope) => {
                tracing::debug!("Processing Qollective envelope message");
                let context = Context::new(envelope.meta.clone());
                handler.handle_envelope_message(envelope, context).await
            }
            MessageType::Raw(data) => {
                tracing::debug!("Processing raw NATS message");
                handler.handle_raw_message(data).await
            }
        }
    }

    /// Detect message type with robust envelope validation
    ///
    /// Uses multiple validation criteria to distinguish between Qollective envelopes
    /// and raw messages:
    /// 1. JSON parsing success
    /// 2. Presence of envelope structure (meta field)
    /// 3. Valid Qollective envelope version
    /// 4. Reasonable envelope field validation
    pub fn detect_message_type(&self, payload: &[u8]) -> Result<MessageType> {
        // First, try to parse as JSON
        match serde_json::from_slice::<serde_json::Value>(payload) {
            Ok(json_value) => {
                // Check if it has the structure of a Qollective envelope
                if let Some(obj) = json_value.as_object() {
                    if obj.contains_key("meta") && obj.contains_key("payload") {
                        // Try to parse as Qollective envelope
                        match serde_json::from_slice::<Envelope<serde_json::Value>>(payload) {
                            Ok(envelope) => {
                                // Additional validation for Qollective envelope
                                if self.is_valid_qollective_envelope(&envelope) {
                                    tracing::debug!("Detected Qollective envelope message");
                                    return Ok(MessageType::Envelope(envelope));
                                }
                            }
                            Err(e) => {
                                tracing::debug!("JSON has envelope structure but failed envelope parsing: {}", e);
                            }
                        }
                    }
                }

                // Valid JSON but not a Qollective envelope
                tracing::debug!("Detected JSON message but not Qollective envelope, treating as raw");
                Ok(MessageType::Raw(payload.to_vec()))
            }
            Err(_) => {
                // Not valid JSON, definitely raw message
                tracing::debug!("Detected non-JSON raw message");
                Ok(MessageType::Raw(payload.to_vec()))
            }
        }
    }

    /// Validate that an envelope is a proper Qollective envelope
    ///
    /// Performs additional checks beyond basic JSON structure to ensure
    /// this is genuinely a Qollective envelope with valid metadata.
    fn is_valid_qollective_envelope(&self, envelope: &Envelope<serde_json::Value>) -> bool {
        // Check for essential envelope characteristics

        // 1. Must have a version (Qollective envelopes always have versions)
        if envelope.meta.version.is_none() {
            return false;
        }

        // 2. Check for reasonable timestamp (optional but common)
        if let Some(timestamp) = &envelope.meta.timestamp {
            // Basic sanity check - timestamp shouldn't be too far in future
            let current_time = chrono::Utc::now();
            let time_diff = timestamp.signed_duration_since(current_time);
            // Reject timestamps more than 1 hour in the future (clock skew tolerance)
            if time_diff.num_seconds() > 3600 {
                return false;
            }
        }

        // 3. If tenant is specified, it should be non-empty
        if let Some(tenant) = &envelope.meta.tenant {
            if tenant.trim().is_empty() {
                return false;
            }
        }

        // 4. Version should follow semantic versioning pattern (basic check)
        if let Some(version) = &envelope.meta.version {
            if version.trim().is_empty() {
                return false;
            }
        }

        // Passed all validation checks
        true
    }

    /// Subscribe to NATS subject with hybrid message handling
    ///
    /// Convenience method that sets up NATS subscription and automatically
    /// processes all incoming messages through the HybridAgent pattern.
    pub async fn subscribe_with_hybrid_handling<T>(
        &self,
        subject: &str,
        handler: T
    ) -> Result<()>
    where
        T: HybridMessageHandler + Send + Sync + 'static,
    {
        let subject_string = subject.to_string();
        tracing::info!("Starting hybrid message subscription on subject: {}", subject_string);

        let mut subscription = self.nats_client.subscribe(subject, None).await?;
        let handler = Arc::new(handler);

        // Process messages with hybrid detection
        tokio::spawn({
            let agent_clone = self.clone();
            let subject_clone = subject_string.clone();
            async move {
                while let Some(message) = subscription.next().await {
                    let handler_clone = handler.clone();
                    let agent_clone = agent_clone.clone();
                    let subject_clone = subject_clone.clone();

                    // Process each message in a separate task for concurrency
                    tokio::spawn(async move {
                        if let Err(e) = agent_clone.handle_message(message.payload.to_vec(), handler_clone.as_ref()).await {
                            tracing::error!("Failed to process hybrid message on {}: {}", subject_clone, e);
                        }
                    });
                }
            }
        });

        tracing::info!("Hybrid message handler started for subject: {}", subject_string);
        Ok(())
    }

    /// Graceful shutdown with proper deregistration
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Initiating graceful shutdown for agent: {}", self.agent_info.name);

        self.a2a_client.deregister_agent(self.agent_info.id).await?;

        tracing::info!("Agent shutdown complete: {}", self.agent_info.name);
        Ok(())
    }
}

impl Clone for EnterpriseAgent {
    fn clone(&self) -> Self {
        Self {
            identity: AgentIdentity {
                id: self.identity.id,
                id_file_path: self.identity.id_file_path.clone(),
            },
            agent_info: self.agent_info.clone(),
            nats_client: self.nats_client.clone(),
            a2a_client: self.a2a_client.clone(),
        }
    }
}

// ============================================================================
// EXAMPLE USAGE PATTERNS
// ============================================================================

/// Example hybrid message handler for logging agent
pub struct LoggingMessageHandler;

#[async_trait]
impl HybridMessageHandler for LoggingMessageHandler {
    async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
        tracing::info!("📨 Processing Qollective envelope message");

        // Extract envelope metadata
        if let Some(request_id) = &context.meta().request_id {
            tracing::debug!("   📋 Request ID: {}", request_id);
        }
        if let Some(tenant) = &context.meta().tenant {
            tracing::debug!("   🏢 Tenant: {}", tenant);
        }
        if let Some(timestamp) = &context.meta().timestamp {
            tracing::debug!("   ⏰ Timestamp: {:?}", timestamp);
        }

        // Process envelope extensions for rich context
        if let Some(extensions) = context.extensions_ref() {
            tracing::debug!("   🔧 Extensions: {} sections", extensions.sections.len());
            for (key, _value) in &extensions.sections {
                tracing::debug!("      └─ Extension: {}", key);
            }
        }

        // Process the actual data payload
        tracing::info!("   📄 Data payload size: {} bytes",
                      serde_json::to_string(&envelope.payload).map(|s| s.len()).unwrap_or(0));

        Ok(())
    }

    async fn handle_raw_message(&self, payload: Vec<u8>) -> Result<()> {
        tracing::info!("📦 Processing raw NATS message");
        tracing::debug!("   📄 Raw payload size: {} bytes", payload.len());

        // Try to parse as JSON for better logging
        match serde_json::from_slice::<serde_json::Value>(&payload) {
            Ok(json_value) => {
                tracing::debug!("   📋 Raw JSON content: {}",
                              serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| "Invalid JSON".to_string()));
            }
            Err(_) => {
                // Handle binary or non-JSON data
                let text = String::from_utf8_lossy(&payload);
                tracing::debug!("   📋 Raw text content: {}", text.chars().take(200).collect::<String>());
            }
        }

        Ok(())
    }
}

/// Example of how to create a hybrid logging agent
pub async fn create_example_hybrid_logging_agent() -> Result<()> {
    // Create the Enterprise agent with hybrid capabilities
    let agent = EnterpriseAgent::builder("Enterprise Hybrid Logging Agent")
        .with_capabilities(vec![
            "logging".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
            "hybrid-message-handling".to_string(),
        ])
        .with_function("Hybrid logging service supporting both Qollective envelopes and raw NATS messages")
        .with_location("Computer Core - Hybrid Logging Bay")
        .with_service_type("hybrid_logging")
        .build()
        .await?;

    // Create message handler
    let handler = LoggingMessageHandler;

    // Subscribe to multiple subjects with hybrid handling
    agent.subscribe_with_hybrid_handling("enterprise.logging.entry", handler).await?;
    agent.subscribe_with_hybrid_handling(subjects::AGENT_REGISTRY_EVENTS, LoggingMessageHandler).await?;

    tracing::info!("✅ Hybrid logging agent started - supports both envelope and raw messages");

    // Keep the agent running
    tokio::signal::ctrl_c().await.map_err(|e| qollective::error::QollectiveError::validation(format!("Signal handling error: {}", e)))?;

    // Graceful shutdown
    agent.shutdown().await?;

    Ok(())
}

/// Example of how to create a specialized monitoring agent using the reusable patterns
pub async fn create_example_monitoring_agent() -> Result<EnterpriseAgent> {
    EnterpriseAgent::builder("Enterprise Monitoring Agent")
        .with_capabilities(vec![
            "monitoring".to_string(),
            "metrics-collection".to_string(),
            "health-checking".to_string(),
            "alerting".to_string(),
            "envelope-processing".to_string(),
        ])
        .with_function("Real-time monitoring and alerting for all Enterprise systems")
        .with_location("Computer Core - Monitoring Station")
        .with_service_type("monitoring")
        .build()
        .await
}

/// Example of how to create a processing agent using the reusable patterns
pub async fn create_example_processing_agent() -> Result<EnterpriseAgent> {
    EnterpriseAgent::builder("Enterprise Data Processing Agent")
        .with_capabilities(vec![
            "data-processing".to_string(),
            "analytics".to_string(),
            "batch-processing".to_string(),
            "real-time-processing".to_string(),
            "envelope-processing".to_string(),
            "raw-message-processing".to_string(),
        ])
        .with_function("Advanced data processing and analytics for Enterprise operations")
        .with_location("Computer Core - Processing Bay")
        .with_service_type("processing")
        .build()
        .await
}

/// Complete example of hybrid agent usage
pub async fn demonstrate_hybrid_agent_patterns() -> Result<()> {
    tracing::info!("🚀 Demonstrating StarTrek Enterprise Hybrid Agent patterns");

    // 1. Create a hybrid agent
    let agent = create_example_monitoring_agent().await?;

    // 2. Define a custom message handler
    struct MonitoringHandler;

    #[async_trait]
    impl HybridMessageHandler for MonitoringHandler {
        async fn handle_envelope_message(&self, envelope: Envelope<serde_json::Value>, context: Context) -> Result<()> {
            tracing::info!("📊 Monitoring: Processing envelope with tenant isolation");

            // Example: Extract tenant for multi-tenant monitoring
            if let Some(tenant) = context.meta().tenant.as_ref() {
                tracing::info!("   🏢 Monitoring tenant: {}", tenant);
                // Route to tenant-specific monitoring logic
            }

            // Example: Process monitoring data from envelope
            if let Some(extensions) = context.extensions_ref() {
                if let Some(monitoring_data) = extensions.sections.get("monitoring") {
                    tracing::info!("   📈 Processing monitoring data: {}", monitoring_data);
                }
            }

            Ok(())
        }

        async fn handle_raw_message(&self, payload: Vec<u8>) -> Result<()> {
            tracing::info!("📊 Monitoring: Processing raw metrics data");

            // Example: Parse legacy monitoring format
            let metrics_text = String::from_utf8_lossy(&payload);
            if metrics_text.starts_with("METRICS:") {
                tracing::info!("   📈 Legacy metrics detected: {}", metrics_text.chars().take(100).collect::<String>());
                // Process legacy metrics format
            }

            Ok(())
        }
    }

    // 3. Subscribe to subjects with hybrid handling
    let monitoring_handler = MonitoringHandler;
    agent.subscribe_with_hybrid_handling("enterprise.monitoring.metrics", monitoring_handler).await?;

    tracing::info!("✅ Hybrid monitoring agent demonstration complete");

    Ok(())
}
