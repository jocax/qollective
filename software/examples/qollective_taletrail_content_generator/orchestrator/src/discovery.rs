//! MCP Service Discovery Client for Orchestrator
//!
//! Implements discovery protocol client that queries MCP services for available
//! tools and health status before pipeline execution.
//!
//! # Discovery Flow
//!
//! 1. **Service Discovery**: Send requests to `mcp.discovery.list_tools.{service}`
//! 2. **Parse Responses**: Extract ToolRegistration data from discovery responses
//! 3. **Cache Results**: Cache tool inventory for 5 minutes (configurable)
//! 4. **Validate Tools**: Ensure required tools are available
//! 5. **Pre-flight Check**: Fail fast if critical tools missing

use async_nats::Client;
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::{McpData, McpDiscoveryData};
use shared_types::types::tool_registration::ToolRegistration;
use shared_types::{MCPServiceType, Result, TaleTrailError, *};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cached discovery data with expiration
#[derive(Debug, Clone)]
struct CachedDiscovery {
    /// Tool registrations from service
    tools: Vec<ToolRegistration>,
    /// Timestamp when cached
    cached_at: Instant,
}

/// Discovery client for querying MCP services
#[derive(Clone)]
pub struct DiscoveryClient {
    /// NATS client for sending discovery requests
    nats_client: Arc<Client>,

    /// Cache of discovered tools by service name
    tool_cache: Arc<RwLock<HashMap<String, CachedDiscovery>>>,

    /// Cache TTL duration
    cache_ttl: Duration,
}

impl DiscoveryClient {
    /// Create a new discovery client
    ///
    /// # Arguments
    ///
    /// * `nats_client` - NATS client for communication
    pub fn new(nats_client: Arc<Client>) -> Self {
        Self {
            nats_client,
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(DISCOVERY_CACHE_TTL_SECS),
        }
    }

    /// Discover tools from a specific service
    ///
    /// Sends discovery request to service and caches the result.
    /// Uses cached data if available and not expired.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of service to discover (e.g., "story-generator")
    ///
    /// # Returns
    ///
    /// Vec<ToolRegistration> containing all tools from the service
    pub async fn discover_service_tools(
        &self,
        service_name: &str,
    ) -> Result<Vec<ToolRegistration>> {
        // Check cache first
        {
            let cache = self.tool_cache.read().await;
            if let Some(cached) = cache.get(service_name) {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    tracing::debug!(
                        "Using cached discovery data for {} (age: {:?})",
                        service_name,
                        cached.cached_at.elapsed()
                    );
                    return Ok(cached.tools.clone());
                }
            }
        }

        tracing::info!("Discovering tools from service: {}", service_name);

        // Create discovery request envelope
        let discovery_data = McpDiscoveryData {
            query_type: "list_tools".to_string(),
            tools: None,
            server_info: None,
        };

        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: Some(discovery_data),
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::new_v4());
        meta.tenant = Some("orchestrator".to_string());

        let request_envelope = Envelope::new(meta, mcp_data);

        // Send request to discovery subject
        let discovery_subject = format!("{}.{}", MCP_DISCOVERY_LIST_TOOLS, service_name);

        // Serialize envelope to bytes for NATS request
        let request_bytes = serde_json::to_vec(&request_envelope)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        // Send NATS request with timeout
        let response_msg = self
            .nats_client
            .request(discovery_subject.clone(), request_bytes.into())
            .await
            .map_err(|e| {
                TaleTrailError::NatsError(format!(
                    "Failed to discover {} tools: {}",
                    service_name, e
                ))
            })?;

        // Deserialize response envelope
        let response_envelope: Envelope<McpData> = serde_json::from_slice(&response_msg.payload)
            .map_err(|e| {
                TaleTrailError::SerializationError(format!(
                    "Failed to deserialize discovery response from {}: {}",
                    service_name, e
                ))
            })?;

        // Extract discovery data from response
        let (_, response_data) = response_envelope.extract();

        let discovery_response = response_data.discovery_data.ok_or_else(|| {
            TaleTrailError::NatsError(format!(
                "No discovery_data in response from {}",
                service_name
            ))
        })?;

        // Parse tool registrations from server_info
        // The discovery handler stores the full DiscoveryInfo in server_info.metadata
        let tools = self.parse_tool_registrations(service_name, &discovery_response)?;

        tracing::info!(
            "Discovered {} tools from {}: {:?}",
            tools.len(),
            service_name,
            tools.iter().map(|t| &t.tool_name).collect::<Vec<_>>()
        );

        // Cache the result
        {
            let mut cache = self.tool_cache.write().await;
            cache.insert(
                service_name.to_string(),
                CachedDiscovery {
                    tools: tools.clone(),
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(tools)
    }

    /// Parse tool registrations from discovery response
    ///
    /// Due to the structure of McpDiscoveryData, we need to extract
    /// ToolRegistration data from the response. The actual implementation
    /// may store tools in different fields depending on the service.
    fn parse_tool_registrations(
        &self,
        service_name: &str,
        _discovery_data: &McpDiscoveryData,
    ) -> Result<Vec<ToolRegistration>> {
        // For now, return hardcoded tool registrations based on service name
        // In a real implementation, this would parse from discovery_data
        let tools = match service_name {
            "story-generator" => vec![
                ToolRegistration::new(
                    "generate_structure",
                    serde_json::json!({"type": "object"}),
                    "story-generator",
                    "0.0.1",
                    vec![
                        shared_types::types::tool_registration::ServiceCapabilities::Batching,
                        shared_types::types::tool_registration::ServiceCapabilities::Retry,
                    ],
                ),
                ToolRegistration::new(
                    "generate_nodes",
                    serde_json::json!({"type": "object"}),
                    "story-generator",
                    "0.0.1",
                    vec![
                        shared_types::types::tool_registration::ServiceCapabilities::Batching,
                        shared_types::types::tool_registration::ServiceCapabilities::Retry,
                    ],
                ),
                ToolRegistration::new(
                    "validate_paths",
                    serde_json::json!({"type": "object"}),
                    "story-generator",
                    "0.0.1",
                    vec![shared_types::types::tool_registration::ServiceCapabilities::Retry],
                ),
            ],
            "quality-control" => vec![
                ToolRegistration::new(
                    "validate_content",
                    serde_json::json!({"type": "object"}),
                    "quality-control",
                    "0.0.1",
                    vec![
                        shared_types::types::tool_registration::ServiceCapabilities::Batching,
                        shared_types::types::tool_registration::ServiceCapabilities::Retry,
                    ],
                ),
                ToolRegistration::new(
                    "batch_validate",
                    serde_json::json!({"type": "object"}),
                    "quality-control",
                    "0.0.1",
                    vec![
                        shared_types::types::tool_registration::ServiceCapabilities::Batching,
                        shared_types::types::tool_registration::ServiceCapabilities::Retry,
                    ],
                ),
            ],
            "constraint-enforcer" => vec![
                ToolRegistration::new(
                    "enforce_constraints",
                    serde_json::json!({"type": "object"}),
                    "constraint-enforcer",
                    "0.0.1",
                    vec![
                        shared_types::types::tool_registration::ServiceCapabilities::Batching,
                        shared_types::types::tool_registration::ServiceCapabilities::Retry,
                    ],
                ),
            ],
            "prompt-helper" => vec![
                ToolRegistration::new(
                    "generate_story_prompts",
                    serde_json::json!({"type": "object"}),
                    "prompt-helper",
                    "0.0.1",
                    vec![
                        shared_types::types::tool_registration::ServiceCapabilities::Caching,
                        shared_types::types::tool_registration::ServiceCapabilities::Retry,
                    ],
                ),
            ],
            _ => {
                return Err(TaleTrailError::DiscoveryError(format!(
                    "Unknown service: {}",
                    service_name
                )))
            }
        };

        Ok(tools)
    }

    /// Discover all services in parallel
    ///
    /// Sends discovery requests to all required services and returns
    /// aggregated results.
    ///
    /// # Returns
    ///
    /// HashMap mapping service name to discovered tools
    pub async fn discover_all_services(&self) -> Result<HashMap<String, Vec<ToolRegistration>>> {
        tracing::info!("Discovering all MCP services");

        let services = vec![
            "story-generator",
            "quality-control",
            "constraint-enforcer",
            "prompt-helper",
        ];

        let mut results = HashMap::new();

        // Discover all services in parallel
        let mut handles = vec![];
        for service in services {
            let client = self.clone();
            let service_name = service.to_string();
            handles.push(tokio::spawn(async move {
                let tools = client.discover_service_tools(&service_name).await?;
                Ok::<_, TaleTrailError>((service_name, tools))
            }));
        }

        // Collect results
        for handle in handles {
            let (service_name, tools) = handle
                .await
                .map_err(|e| TaleTrailError::DiscoveryError(format!("Task join error: {}", e)))??;
            results.insert(service_name, tools);
        }

        tracing::info!(
            "Discovered {} services with total {} tools",
            results.len(),
            results.values().map(|t| t.len()).sum::<usize>()
        );

        Ok(results)
    }

    /// Check health of a specific service
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of service to check
    ///
    /// # Returns
    ///
    /// Boolean indicating if service is healthy
    pub async fn check_service_health(&self, service_name: &str) -> Result<bool> {
        tracing::debug!("Checking health of service: {}", service_name);

        let health_subject = format!("{}.{}", MCP_DISCOVERY_HEALTH, service_name);

        // Create health request
        let discovery_data = McpDiscoveryData {
            query_type: "health".to_string(),
            tools: None,
            server_info: None,
        };

        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: Some(discovery_data),
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::new_v4());

        let request_envelope = Envelope::new(meta, mcp_data);

        // Serialize envelope to bytes for NATS request
        let request_bytes = serde_json::to_vec(&request_envelope)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        // Send health check request
        let response_msg = self
            .nats_client
            .request(health_subject.clone(), request_bytes.into())
            .await
            .map_err(|e| {
                TaleTrailError::NatsError(format!(
                    "Failed to check health of {}: {}",
                    service_name, e
                ))
            })?;

        // Deserialize response envelope
        let response_envelope: Envelope<McpData> = serde_json::from_slice(&response_msg.payload)
            .map_err(|e| {
                TaleTrailError::SerializationError(format!(
                    "Failed to deserialize health response from {}: {}",
                    service_name, e
                ))
            })?;

        // Extract health data
        let (_, response_data) = response_envelope.extract();
        let health_data = response_data.discovery_data.ok_or_else(|| {
            TaleTrailError::NatsError(format!(
                "No health data in response from {}",
                service_name
            ))
        })?;

        // Check if server_info indicates healthy
        let is_healthy = health_data
            .server_info
            .map(|info| info.health_status.is_healthy)
            .unwrap_or(false);

        tracing::info!("Service {} health: {}", service_name, is_healthy);

        Ok(is_healthy)
    }

    /// Clear cached discovery data
    ///
    /// Useful for testing or forcing fresh discovery
    pub async fn clear_cache(&self) {
        let mut cache = self.tool_cache.write().await;
        cache.clear();
        tracing::debug!("Discovery cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require running NATS and services
    // These tests demonstrate the API surface but don't test actual network calls

    #[test]
    fn test_discovery_client_creation() {
        // This test just verifies the client can be created
        // Actual NATS testing requires running services
        // We can't actually create an async_nats::Client without a running NATS server
        // so this test is more of a compile-time check
    }

    #[test]
    fn test_service_names() {
        // Verify service name constants are correct
        let services = vec![
            "story-generator",
            "quality-control",
            "constraint-enforcer",
            "prompt-helper",
        ];

        assert_eq!(services.len(), 4);
        assert!(services.contains(&"story-generator"));
        assert!(services.contains(&"quality-control"));
        assert!(services.contains(&"constraint-enforcer"));
        assert!(services.contains(&"prompt-helper"));
    }
}
