//! TaleTrail data types

pub mod tool_registration;

use serde::{Deserialize, Serialize};

// Re-export tool registration types
pub use tool_registration::{DiscoveryInfo, ServiceCapabilities, ToolRegistration};

/// MCP service types in the TaleTrail pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MCPServiceType {
    /// Prompt helper service for generating prompts
    PromptHelper,
    /// Story generator service for creating content
    StoryGenerator,
    /// Quality control service for validating content quality
    QualityControl,
    /// Constraint enforcer service for checking constraints
    ConstraintEnforcer,
}
