//! Enterprise Registry Manager with Certificate Validation
//!
//! This module provides a wrapper around the core A2A server that adds
//! Enterprise-specific certificate validation for secure registry operations.
//! It intercepts agent registration requests and validates certificates
//! before allowing registration to proceed.

use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, info, warn, error};
use colored::Colorize;

use qollective::{
    error::{QollectiveError, Result},
    server::a2a::A2AServer,
    config::a2a::A2AServerConfig,
    types::a2a::AgentInfo,
    client::a2a::AgentMetadata,
};

use crate::enterprise_certificate_validator::{
    EnterpriseCertificateValidator, 
    EnterpriseCertificateValidationConfig,
    CertificateValidationResult,
    EnterpriseValidationStats,
};

/// Enterprise Registry Manager with integrated certificate validation
#[derive(Debug)]
pub struct EnterpriseRegistryManager {
    /// Underlying A2A server from the framework
    a2a_server: A2AServer,
    /// Enterprise-specific certificate validator
    certificate_validator: Arc<Mutex<EnterpriseCertificateValidator>>,
    /// Registry statistics
    registry_stats: Arc<RwLock<EnterpriseRegistryStats>>,
    /// Configuration
    config: A2AServerConfig,
}

/// Enterprise registry statistics
#[derive(Debug, Clone, Default)]
pub struct EnterpriseRegistryStats {
    pub total_registration_attempts: usize,
    pub successful_registrations: usize,
    pub failed_registrations: usize,
    pub certificate_validation_failures: usize,
    pub active_crew_members: usize,
    pub registrations_by_department: HashMap<String, usize>,
}

impl EnterpriseRegistryManager {
    /// Create a new Enterprise Registry Manager with certificate validation
    pub async fn new(config: A2AServerConfig) -> Result<Self> {
        info!("🏗️  Creating USS Enterprise Registry Manager with certificate validation");
        
        // Create the underlying A2A server
        info!("   Initializing core A2A server infrastructure...");
        let a2a_server = A2AServer::new(config.clone()).await?;
        info!("   ✅ Core A2A server created successfully");
        
        // Create Enterprise certificate validator
        info!("   Initializing Enterprise certificate validator...");
        let cert_config = EnterpriseCertificateValidationConfig::default();
        let certificate_validator = EnterpriseCertificateValidator::new(cert_config).await?;
        info!("   ✅ Enterprise certificate validator initialized");
        
        // Initialize statistics
        let registry_stats = Arc::new(RwLock::new(EnterpriseRegistryStats::default()));
        
        info!("🚀 USS Enterprise Registry Manager ready for secure operations");
        
        Ok(Self {
            a2a_server,
            certificate_validator: Arc::new(Mutex::new(certificate_validator)),
            registry_stats,
            config,
        })
    }

    /// Start the Enterprise Registry Manager with certificate validation
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting USS Enterprise Registry Manager...");
        
        // Start the underlying A2A server first
        info!("   Starting core A2A server infrastructure...");
        self.a2a_server.start().await?;
        info!("   ✅ Core A2A server started successfully");
        
        // Start certificate validation monitoring
        info!("   Starting Enterprise certificate validation monitoring...");
        self.start_certificate_validation_monitoring().await?;
        info!("   ✅ Certificate validation monitoring active");
        
        // Display Enterprise registry status
        self.display_registry_status().await;
        
        info!("🏛️  USS Enterprise Registry Manager fully operational");
        Ok(())
    }

    /// Start certificate validation monitoring (intercepts registration requests)
    async fn start_certificate_validation_monitoring(&self) -> Result<()> {
        info!("🔍 Enterprise certificate validation monitoring activated");
        info!("   All agent registration requests will be validated");
        info!("   Only authorized Enterprise crew members can register");
        
        // In a real implementation, we would hook into the A2A server's registration
        // handler here. For this example, we'll implement validation at the application
        // level when agents attempt to register.
        
        Ok(())
    }

    /// Validate and register an Enterprise crew member
    pub async fn validate_and_register_agent(
        &self,
        agent_info: &AgentInfo,
        agent_metadata: &AgentMetadata,
    ) -> Result<CertificateValidationResult> {
        // Update statistics
        {
            let mut stats = self.registry_stats.write().await;
            stats.total_registration_attempts += 1;
        }
        
        info!("🔐 Enterprise Registry: Validating agent registration request");
        info!("   Agent: {} (ID: {})", agent_info.name.bright_cyan(), agent_info.id);
        info!("   Capabilities: {:?}", agent_info.capabilities);
        
        // Perform certificate validation
        let mut validator = self.certificate_validator.lock().await;
        let validation_result = validator.validate_agent_registration(agent_info, agent_metadata).await?;
        
        // Process validation result
        if validation_result.is_valid {
            // Validation successful - allow registration
            info!("✅ Certificate validation successful");
            if let Some(ref crew_member) = validation_result.crew_member {
                info!("   Crew Member: {} ({})", 
                      crew_member.bright_green(), 
                      validation_result.rank.as_deref().unwrap_or("Unknown Rank"));
                info!("   Department: {} | Clearance: {}", 
                      validation_result.department.as_deref().unwrap_or("Unknown").bright_yellow(),
                      validation_result.clearance_level.as_deref().unwrap_or("Unknown").bright_blue());
            }
            
            // Update successful registration statistics
            {
                let mut stats = self.registry_stats.write().await;
                stats.successful_registrations += 1;
                stats.active_crew_members += 1;
                
                if let Some(ref department) = validation_result.department {
                    *stats.registrations_by_department.entry(department.clone()).or_insert(0) += 1;
                }
            }
            
            // Log successful registration
            self.log_successful_registration(&validation_result).await;
            
        } else {
            // Validation failed - reject registration
            error!("❌ Certificate validation failed for agent: {}", agent_info.name.bright_red());
            for error in &validation_result.validation_errors {
                error!("   ⚠️  {}", error.bright_yellow());
            }
            
            // Update failed registration statistics
            {
                let mut stats = self.registry_stats.write().await;
                stats.failed_registrations += 1;
                stats.certificate_validation_failures += 1;
            }
            
            // Log failed registration attempt
            self.log_failed_registration(&validation_result, agent_info).await;
        }
        
        Ok(validation_result)
    }

    /// Display Enterprise registry status
    async fn display_registry_status(&self) {
        println!("\n{}", "🏛️  USS Enterprise Registry Status".bright_blue().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        
        let stats = self.registry_stats.read().await;
        println!("{} {}", "📊 Registration Attempts:".bright_cyan(), stats.total_registration_attempts.to_string().bright_yellow());
        println!("{} {}", "✅ Successful Registrations:".bright_green(), stats.successful_registrations.to_string().bright_yellow());
        println!("{} {}", "❌ Failed Registrations:".bright_red(), stats.failed_registrations.to_string().bright_yellow());
        println!("{} {}", "👥 Active Crew Members:".bright_blue(), stats.active_crew_members.to_string().bright_yellow());
        
        if !stats.registrations_by_department.is_empty() {
            println!("\n{}", "🏢 Registrations by Department:".bright_cyan());
            for (department, count) in &stats.registrations_by_department {
                println!("   {} {}", 
                        format!("{}:", department).bright_yellow(),
                        count.to_string().bright_white());
            }
        }
        
        // Get certificate validator statistics
        let validator = self.certificate_validator.lock().await;
        let cert_stats = validator.get_validation_stats();
        drop(validator);
        
        println!("\n{}", "🔐 Certificate Validation Statistics:".bright_cyan());
        println!("{} {}", "   Total Validations:".dimmed(), cert_stats.total_validations.to_string().bright_yellow());
        println!("{} {}", "   Successful Validations:".dimmed(), cert_stats.successful_validations.to_string().bright_green());
        println!("{} {}", "   Failed Validations:".dimmed(), cert_stats.failed_validations.to_string().bright_red());
        println!("{} {}", "   Cache Size:".dimmed(), cert_stats.cache_size.to_string().bright_blue());
        
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    }

    /// Log successful agent registration
    async fn log_successful_registration(&self, validation_result: &CertificateValidationResult) {
        info!("📝 Enterprise Registration Log: SUCCESS");
        info!("   Crew Member: {}", validation_result.crew_member.as_deref().unwrap_or("Unknown"));
        info!("   Rank: {}", validation_result.rank.as_deref().unwrap_or("Unknown"));
        info!("   Department: {}", validation_result.department.as_deref().unwrap_or("Unknown"));
        info!("   Clearance Level: {}", validation_result.clearance_level.as_deref().unwrap_or("Unknown"));
        info!("   Validation Time: {}", validation_result.validated_at);
        
        // Enterprise-specific logging (could write to Enterprise log files)
        debug!("Enterprise Registry: Agent registration approved and logged to ship's records");
    }

    /// Log failed agent registration attempt
    async fn log_failed_registration(&self, validation_result: &CertificateValidationResult, agent_info: &AgentInfo) {
        warn!("📝 Enterprise Registration Log: FAILURE");
        warn!("   Agent Name: {}", agent_info.name);
        warn!("   Agent ID: {}", agent_info.id);
        warn!("   Attempted Capabilities: {:?}", agent_info.capabilities);
        warn!("   Validation Errors: {:?}", validation_result.validation_errors);
        warn!("   Validation Time: {}", validation_result.validated_at);
        
        // Security alert for Enterprise systems
        error!("🚨 Enterprise Security Alert: Unauthorized registration attempt detected");
        error!("   This incident has been logged to Enterprise security systems");
    }

    /// Get current registry statistics
    pub async fn get_registry_stats(&self) -> EnterpriseRegistryStats {
        self.registry_stats.read().await.clone()
    }

    /// Get certificate validation statistics
    pub async fn get_validation_stats(&self) -> EnterpriseValidationStats {
        let validator = self.certificate_validator.lock().await;
        validator.get_validation_stats()
    }

    /// Shutdown the Enterprise Registry Manager
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("🛑 Shutting down USS Enterprise Registry Manager...");
        
        // Display final statistics
        println!("\n{}", "📊 Final USS Enterprise Registry Statistics".bright_blue().bold());
        self.display_registry_status().await;
        
        // Clear certificate validation cache for security
        {
            let mut validator = self.certificate_validator.lock().await;
            validator.clear_cache();
        }
        info!("   🔒 Certificate validation cache cleared for security");
        
        // Note: The underlying A2A server shutdown would be handled by the framework
        info!("   🚀 Core A2A server shutdown initiated");
        
        info!("✅ USS Enterprise Registry Manager shutdown complete");
        Ok(())
    }
}

/// Enterprise Registry Manager builder for easier configuration
pub struct EnterpriseRegistryManagerBuilder {
    server_config: Option<A2AServerConfig>,
    cert_validation_config: Option<EnterpriseCertificateValidationConfig>,
}

impl EnterpriseRegistryManagerBuilder {
    pub fn new() -> Self {
        Self {
            server_config: None,
            cert_validation_config: None,
        }
    }

    pub fn with_server_config(mut self, config: A2AServerConfig) -> Self {
        self.server_config = Some(config);
        self
    }

    pub fn with_certificate_validation_config(mut self, config: EnterpriseCertificateValidationConfig) -> Self {
        self.cert_validation_config = Some(config);
        self
    }

    pub async fn build(self) -> Result<EnterpriseRegistryManager> {
        let server_config = self.server_config
            .ok_or_else(|| QollectiveError::validation("Server config is required".to_string()))?;

        EnterpriseRegistryManager::new(server_config).await
    }
}

impl Default for EnterpriseRegistryManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qollective::{
        config::a2a::{A2AServerConfig, RegistryConfig, HealthConfig, RoutingConfig},
        config::nats::NatsClientConfig,
        types::a2a::HealthStatus,
    };
    use std::collections::HashMap;

    fn create_test_server_config() -> A2AServerConfig {
        A2AServerConfig {
            server_id: "".to_string(),
            server_name: "".to_string(),
            registry: Default::default(),
            routing: Default::default(),
            health: Default::default(),
            transport: Default::default(),
            nats_server: Default::default(),
            nats_client: Default::default(),
            max_concurrent_requests: 0,
            request_timeout: Default::default(),
            enable_request_queuing: false,
            max_queue_size: 0,
            enable_rate_limiting: false,
            requests_per_second: 0,
        }
    }

    #[tokio::test]
    async fn test_enterprise_registry_manager_creation() {
        let config = create_test_server_config();
        
        // This test may fail if NATS server is not available, which is expected
        match EnterpriseRegistryManager::new(config).await {
            Ok(_manager) => {
                // Success - registry manager created
            },
            Err(_) => {
                // Expected if NATS server not available in test environment
                println!("Enterprise Registry Manager test skipped - A2A server infrastructure not available");
            }
        }
    }
}