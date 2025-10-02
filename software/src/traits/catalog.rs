// ABOUTME: Server catalog trait for managing server capabilities and resources
// ABOUTME: Provides unified interface for exposing server capabilities across all protocols

//! Server catalog trait for unified capability management.
//!
//! This trait provides a standardized interface for servers to expose their
//! capabilities, tools, resources, and metadata. All server implementations
//! should implement this trait to enable consistent capability discovery.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Server capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerCapability {
    /// Capability name (e.g., "tools", "resources", "prompts")
    pub name: String,
    /// Capability version
    pub version: String,
    /// Whether capability is enabled
    pub enabled: bool,
    /// Capability metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool registration information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegisteredTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Tool input schema
    pub input_schema: Option<serde_json::Value>,
    /// Tool metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Resource registration information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegisteredResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource MIME type
    pub mime_type: Option<String>,
    /// Resource metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Server catalog trait for unified capability management.
///
/// This trait provides a standardized interface for servers to expose their
/// capabilities, tools, resources, and configuration. It enables unified
/// discovery and introspection across all transport protocols.
#[async_trait]
pub trait ServerCatalog {
    /// List all server capabilities
    ///
    /// Returns a list of capabilities supported by this server instance.
    /// Capabilities represent different functional areas like tools, resources,
    /// prompts, sampling, etc.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<ServerCapability>>` containing all available capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if capability enumeration fails.
    async fn list_capabilities(&self) -> Result<Vec<ServerCapability>>;

    /// List all registered tools
    ///
    /// Returns a list of tools that are registered and available for execution
    /// on this server instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<RegisteredTool>>` containing all registered tools.
    ///
    /// # Errors
    ///
    /// Returns an error if tool enumeration fails.
    async fn list_registered_tools(&self) -> Result<Vec<RegisteredTool>>;

    /// List all registered resources
    ///
    /// Returns a list of resources that are available for access
    /// on this server instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<RegisteredResource>>` containing all registered resources.
    ///
    /// # Errors
    ///
    /// Returns an error if resource enumeration fails.
    async fn list_registered_resources(&self) -> Result<Vec<RegisteredResource>>;

    /// Get server metadata
    ///
    /// Returns metadata about the server instance including version,
    /// name, configuration, and other server-specific information.
    ///
    /// # Returns
    ///
    /// Returns a `Result<HashMap<String, serde_json::Value>>` containing server metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata retrieval fails.
    async fn get_server_metadata(&self) -> Result<HashMap<String, serde_json::Value>>;

    /// Check if a specific capability is supported
    ///
    /// Checks whether the server supports a specific capability by name.
    ///
    /// # Arguments
    ///
    /// * `capability_name` - The name of the capability to check
    ///
    /// # Returns
    ///
    /// Returns a `Result<bool>` indicating whether the capability is supported.
    ///
    /// # Errors
    ///
    /// Returns an error if capability checking fails.
    async fn supports_capability(&self, capability_name: &str) -> Result<bool>;

    /// Get capability details
    ///
    /// Returns detailed information about a specific capability.
    ///
    /// # Arguments
    ///
    /// * `capability_name` - The name of the capability to get details for
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<ServerCapability>>` with capability details if found.
    ///
    /// # Errors
    ///
    /// Returns an error if capability lookup fails.
    async fn get_capability_details(
        &self,
        capability_name: &str,
    ) -> Result<Option<ServerCapability>>;
}
