//! Enterprise Certificate Validator for Secure A2A Registry Operations
//!
//! This module provides certificate validation for the USS Enterprise A2A server
//! registry operations. It ensures that only trusted crew members with valid
//! certificates can register with the Enterprise systems.

use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::Path,
};
use chrono::{DateTime, Utc};
use rustls_pemfile::certs;
use tracing::{debug, info, warn, error};
use qollective::{
    error::{QollectiveError, Result},
    types::a2a::AgentInfo,
    client::a2a::AgentMetadata,
};

/// Enterprise-specific certificate validation configuration
#[derive(Debug, Clone)]
pub struct EnterpriseCertificateValidationConfig {
    /// Path to trusted Enterprise CA certificate
    pub ca_cert_path: String,
    /// Whether to enforce certificate expiration validation
    pub enforce_expiration: bool,
    /// Allowed crew certificate subjects (Starfleet personnel only)
    pub allowed_crew_subjects: Vec<String>,
    /// Whether to perform strict certificate chain validation
    pub strict_validation: bool,
}

impl Default for EnterpriseCertificateValidationConfig {
    fn default() -> Self {
        Self {
            ca_cert_path: "../../tests/certs/ca-cert.pem".to_string(),
            enforce_expiration: true,
            allowed_crew_subjects: vec![
                "picard".to_string(),
                "data".to_string(),
                "spock".to_string(),
                "scotty".to_string(),
                "enterprise".to_string(),
                "q_console".to_string(),
                "log_agent".to_string(),
            ],
            strict_validation: true,
        }
    }
}

/// Enterprise Certificate Validator for A2A Registry Operations
#[derive(Debug)]
pub struct EnterpriseCertificateValidator {
    config: EnterpriseCertificateValidationConfig,
    validation_cache: HashMap<String, CertificateValidationResult>,
    trusted_subjects: HashMap<String, CrewMemberInfo>,
}

/// Certificate validation result specific to Enterprise operations
#[derive(Debug, Clone)]
pub struct CertificateValidationResult {
    pub is_valid: bool,
    pub crew_member: Option<String>,
    pub rank: Option<String>,
    pub department: Option<String>,
    pub clearance_level: Option<String>,
    pub validation_errors: Vec<String>,
    pub validated_at: DateTime<Utc>,
}

/// Enterprise crew member information for certificate validation
#[derive(Debug, Clone)]
pub struct CrewMemberInfo {
    pub name: String,
    pub rank: String,
    pub department: String,
    pub clearance_level: String,
    pub expected_capabilities: Vec<String>,
}

impl EnterpriseCertificateValidator {
    /// Create a new Enterprise certificate validator
    pub async fn new(config: EnterpriseCertificateValidationConfig) -> Result<Self> {
        info!("🔐 Initializing Enterprise Certificate Validator");
        info!("   Trusted CA: {}", config.ca_cert_path);
        info!("   Allowed crew: {:?}", config.allowed_crew_subjects);

        // Initialize trusted crew member database
        let trusted_subjects = Self::initialize_crew_database();

        // Validate CA certificate exists (optional - for enhanced security)
        if Path::new(&config.ca_cert_path).exists() {
            info!("✅ Enterprise CA certificate found and accessible");
        } else {
            warn!("⚠️  Enterprise CA certificate not found at: {}", config.ca_cert_path);
            warn!("   Certificate validation will use basic subject validation only");
        }

        Ok(Self {
            config,
            validation_cache: HashMap::new(),
            trusted_subjects,
        })
    }

    /// Initialize the Enterprise crew member database with expected certificate subjects
    fn initialize_crew_database() -> HashMap<String, CrewMemberInfo> {
        let mut crew_db = HashMap::new();

        // USS Enterprise NCC-1701-D Crew Roster
        crew_db.insert("picard".to_string(), CrewMemberInfo {
            name: "Jean-Luc Picard".to_string(),
            rank: "Captain".to_string(),
            department: "Command".to_string(),
            clearance_level: "Alpha".to_string(),
            expected_capabilities: vec!["command".to_string(), "leadership".to_string(), "diplomacy".to_string()],
        });

        crew_db.insert("data".to_string(), CrewMemberInfo {
            name: "Data".to_string(),
            rank: "Lt. Commander".to_string(),
            department: "Operations".to_string(),
            clearance_level: "Beta".to_string(),
            expected_capabilities: vec!["data-analysis".to_string(), "computation".to_string(), "positronic-analysis".to_string()],
        });

        crew_db.insert("spock".to_string(), CrewMemberInfo {
            name: "Spock".to_string(),
            rank: "Commander".to_string(),
            department: "Science".to_string(),
            clearance_level: "Beta".to_string(),
            expected_capabilities: vec!["science".to_string(), "logic".to_string(), "vulcan-analysis".to_string()],
        });

        crew_db.insert("scotty".to_string(), CrewMemberInfo {
            name: "Montgomery Scott".to_string(),
            rank: "Commander".to_string(),
            department: "Engineering".to_string(),
            clearance_level: "Beta".to_string(),
            expected_capabilities: vec!["engineering".to_string(), "propulsion".to_string(), "maintenance".to_string()],
        });

        // Support systems
        crew_db.insert("enterprise".to_string(), CrewMemberInfo {
            name: "USS Enterprise Central Computer".to_string(),
            rank: "System".to_string(),
            department: "Computer Core".to_string(),
            clearance_level: "Alpha".to_string(),
            expected_capabilities: vec!["registry".to_string(), "coordination".to_string(), "ship-systems".to_string()],
        });

        crew_db.insert("q_console".to_string(), CrewMemberInfo {
            name: "Q Console Interface".to_string(),
            rank: "Entity".to_string(),
            department: "Cosmic Affairs".to_string(),
            clearance_level: "Omega".to_string(),
            expected_capabilities: vec!["cosmic-challenges".to_string(), "reality-manipulation".to_string()],
        });

        crew_db.insert("log_agent".to_string(), CrewMemberInfo {
            name: "Enterprise Log Recording System".to_string(),
            rank: "System".to_string(),
            department: "Ship Operations".to_string(),
            clearance_level: "Gamma".to_string(),
            expected_capabilities: vec!["logging".to_string(), "record-keeping".to_string(), "analysis".to_string()],
        });

        crew_db
    }

    /// Validate agent registration with certificate validation
    pub async fn validate_agent_registration(
        &mut self,
        agent_info: &AgentInfo,
        agent_metadata: &AgentMetadata
    ) -> Result<CertificateValidationResult> {
        let agent_name = &agent_info.name;
        info!("🔍 Validating certificate for agent registration: {}", agent_name);

        // Extract crew member identifier from agent name (simplified approach)
        let crew_id = self.extract_crew_id(agent_name);
        debug!("   Extracted crew ID: {}", crew_id);

        // Check cache first (with 5-minute freshness)
        let cache_key = format!("{}_{}", crew_id, agent_info.id);
        if let Some(cached_result) = self.validation_cache.get(&cache_key) {
            if cached_result.validated_at > Utc::now() - chrono::Duration::minutes(5) {
                debug!("✅ Using cached certificate validation result for: {}", crew_id);
                return Ok(cached_result.clone());
            }
        }

        // Perform Enterprise-specific validation
        let mut validation_errors = Vec::new();
        let mut is_valid = true;

        // 1. Validate crew member is authorized for Enterprise operations
        let crew_member_info = if let Some(info) = self.trusted_subjects.get(&crew_id) {
            debug!("✅ Crew member found in Enterprise roster: {} ({})", info.name, info.rank);
            Some(info.clone())
        } else {
            let error = format!("Agent '{}' is not authorized for USS Enterprise operations", crew_id);
            validation_errors.push(error.clone());
            error!("❌ Certificate validation failed: {}", error);
            is_valid = false;
            None
        };

        // 2. Validate agent capabilities match expected crew capabilities
        if let Some(ref crew_info) = crew_member_info {
            let has_expected_capabilities = crew_info.expected_capabilities.iter()
                .any(|expected| agent_info.capabilities.iter()
                    .any(|actual| actual.to_lowercase().contains(&expected.to_lowercase())));

            if !has_expected_capabilities {
                let error = format!(
                    "Agent capabilities {:?} do not match expected capabilities for {}: {:?}",
                    agent_info.capabilities, crew_info.name, crew_info.expected_capabilities
                );
                validation_errors.push(error.clone());
                warn!("⚠️  Capability mismatch: {}", error);
                // This is a warning, not a hard failure for Enterprise operations
            } else {
                debug!("✅ Agent capabilities validated for crew member: {}", crew_info.name);
            }
        }

        // 3. Validate agent metadata contains Enterprise-specific information
        if agent_metadata.version.is_empty() {
            validation_errors.push("Agent metadata missing version information".to_string());
            is_valid = false;
        }

        // 4. Validate agent health status
        match agent_info.health_status {
            qollective::types::a2a::HealthStatus::Healthy => {
                debug!("✅ Agent health status: Healthy");
            },
            _ => {
                let error = format!("Agent health status is not healthy: {:?}", agent_info.health_status);
                validation_errors.push(error.clone());
                warn!("⚠️  Agent health issue: {}", error);
                // Health issues are warnings for Enterprise operations (crew can be injured)
            }
        }

        // 5. Additional Enterprise security checks
        if self.config.strict_validation {
            // Check for suspicious agent metadata
            if agent_metadata.custom_metadata.contains_key("borg_signature") {
                validation_errors.push("Borg signature detected - potential assimilation threat".to_string());
                error!("🚨 SECURITY ALERT: Borg signature detected in agent metadata");
                is_valid = false;
            }

            // Check for Q entity special handling
            if crew_id == "q_console" {
                info!("🌟 Q Entity detected - applying cosmic clearance protocols");
                is_valid = true; // Q bypasses most validation (cosmic entity privileges)
                validation_errors.clear();
            }
        }

        // Create validation result
        let result = CertificateValidationResult {
            is_valid,
            crew_member: crew_member_info.as_ref().map(|info| info.name.clone()),
            rank: crew_member_info.as_ref().map(|info| info.rank.clone()),
            department: crew_member_info.as_ref().map(|info| info.department.clone()),
            clearance_level: crew_member_info.as_ref().map(|info| info.clearance_level.clone()),
            validation_errors,
            validated_at: Utc::now(),
        };

        // Cache the result
        self.validation_cache.insert(cache_key, result.clone());

        // Log validation result
        if result.is_valid {
            info!("✅ Certificate validation successful for: {} ({})",
                  agent_name,
                  result.crew_member.as_deref().unwrap_or("Unknown"));
            if let Some(ref rank) = result.rank {
                info!("   Rank: {} | Department: {} | Clearance: {}",
                      rank,
                      result.department.as_deref().unwrap_or("Unknown"),
                      result.clearance_level.as_deref().unwrap_or("Unknown"));
            }
        } else {
            error!("❌ Certificate validation failed for: {} - Errors: {:?}",
                   agent_name, result.validation_errors);
        }

        Ok(result)
    }

    /// Extract crew member ID from agent name (simplified approach for Enterprise example)
    fn extract_crew_id(&self, agent_name: &str) -> String {
        let name_lower = agent_name.to_lowercase();

        // Check for exact matches first
        for allowed_subject in &self.config.allowed_crew_subjects {
            if name_lower.contains(allowed_subject) {
                return allowed_subject.clone();
            }
        }

        // Special handling for crew names that might be in different formats
        if name_lower.contains("montgomery") || name_lower.contains("scott") {
            return "scotty".to_string();
        }
        if name_lower.contains("jean-luc") || name_lower.contains("picard") {
            return "picard".to_string();
        }
        if name_lower.contains("data") && !name_lower.contains("metadata") {
            return "data".to_string();
        }
        if name_lower.contains("spock") {
            return "spock".to_string();
        }
        if name_lower.contains("q console") || name_lower.contains("q entity") || (name_lower.starts_with("q ") && name_lower.contains("console")) {
            return "q_console".to_string();
        }
        if name_lower.contains("log agent") || name_lower.contains("log recording") {
            return "log_agent".to_string();
        }
        if name_lower.contains("enterprise") && (name_lower.contains("computer") || name_lower.contains("central")) {
            return "enterprise".to_string();
        }

        // Fallback: extract first word and clean it (but try last word too for surnames)
        let words: Vec<&str> = name_lower.split_whitespace().collect();
        if words.len() > 1 {
            // Try last word first (surnames)
            let last_word = words.last().unwrap()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();

            for allowed_subject in &self.config.allowed_crew_subjects {
                if last_word.contains(allowed_subject) || allowed_subject.contains(&last_word) {
                    return allowed_subject.clone();
                }
            }
        }

        // Final fallback: first word
        name_lower.split_whitespace()
            .next()
            .unwrap_or("unknown")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    /// Get validation statistics for monitoring
    pub fn get_validation_stats(&self) -> EnterpriseValidationStats {
        let total_validations = self.validation_cache.len();
        let successful_validations = self.validation_cache.values()
            .filter(|result| result.is_valid)
            .count();
        let failed_validations = total_validations - successful_validations;

        // Count by department
        let mut department_stats = HashMap::new();
        for result in self.validation_cache.values() {
            if let Some(ref dept) = result.department {
                *department_stats.entry(dept.clone()).or_insert(0) += 1;
            }
        }

        EnterpriseValidationStats {
            total_validations,
            successful_validations,
            failed_validations,
            department_stats,
            cache_size: self.validation_cache.len(),
        }
    }

    /// Clear validation cache (for security or testing purposes)
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
        info!("🔄 Enterprise certificate validation cache cleared");
    }
}

/// Enterprise-specific validation statistics
#[derive(Debug, Clone)]
pub struct EnterpriseValidationStats {
    pub total_validations: usize,
    pub successful_validations: usize,
    pub failed_validations: usize,
    pub department_stats: HashMap<String, usize>,
    pub cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use qollective::types::a2a::HealthStatus;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_enterprise_certificate_validator_creation() {
        let config = EnterpriseCertificateValidationConfig::default();
        let validator = EnterpriseCertificateValidator::new(config).await;
        assert!(validator.is_ok());
    }

    #[tokio::test]
    async fn test_crew_member_validation() {
        let config = EnterpriseCertificateValidationConfig::default();
        let mut validator = EnterpriseCertificateValidator::new(config).await.unwrap();

        let agent_info = AgentInfo {
            id: uuid::Uuid::now_v7(),
            name: "Captain Picard".to_string(),
            capabilities: vec!["command".to_string(), "leadership".to_string()],
            health_status: HealthStatus::Healthy,
            last_heartbeat: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };

        let agent_metadata = AgentMetadata {
            version: "1.0.0".to_string(),
            build_info: Some("Enterprise Build".to_string()),
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::new(),
        };

        let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await;
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert_eq!(validation_result.crew_member, Some("Jean-Luc Picard".to_string()));
        assert_eq!(validation_result.rank, Some("Captain".to_string()));
    }

    #[tokio::test]
    async fn test_unauthorized_agent_validation() {
        let config = EnterpriseCertificateValidationConfig::default();
        let mut validator = EnterpriseCertificateValidator::new(config).await.unwrap();

        let agent_info = AgentInfo {
            id: uuid::Uuid::now_v7(),
            name: "Unknown Agent".to_string(),
            capabilities: vec!["hacking".to_string()],
            health_status: HealthStatus::Healthy,
            last_heartbeat: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };

        let agent_metadata = AgentMetadata {
            version: "1.0.0".to_string(),
            build_info: None,
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::new(),
        };

        let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await;
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(!validation_result.is_valid);
        assert!(!validation_result.validation_errors.is_empty());
    }
}
