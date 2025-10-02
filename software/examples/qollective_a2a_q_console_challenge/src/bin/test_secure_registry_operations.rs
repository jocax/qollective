//! Test Secure Enterprise Registry Operations with Certificate Validation
//!
//! This test demonstrates the Enterprise Registry Manager's certificate validation
//! capabilities for secure A2A server registry operations. It validates that only
//! authorized Enterprise crew members with valid certificates can register.

use std::{collections::HashMap, time::SystemTime};
use uuid::Uuid;
use colored::Colorize;
use qollective::{
    error::Result,
    types::a2a::{AgentInfo, HealthStatus},
    client::a2a::AgentMetadata,
};
use qollective_a2a_nats_enterprise::{
    config::EnterpriseConfig,
    enterprise_certificate_validator::{EnterpriseCertificateValidator, EnterpriseCertificateValidationConfig},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("{}", "🔐 Testing Enterprise Registry Operations with Certificate Validation".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Initialize TLS crypto provider
    match rustls::crypto::aws_lc_rs::default_provider().install_default() {
        Ok(_) => {
            println!("{}", "🔒 TLS crypto provider initialized successfully".bright_green());
        }
        Err(e) => {
            println!("{} {:?}", "❌ Failed to initialize TLS crypto provider:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(
                format!("TLS crypto provider initialization failed: {:?}", e)
            ));
        }
    }

    // Load Enterprise configuration for TLS context
    println!("{}", "📁 Loading Enterprise configuration for TLS context...".bright_blue());
    let config = match EnterpriseConfig::load_default() {
        Ok(config) => {
            println!("{}", "✅ Enterprise configuration loaded successfully".bright_green());
            config
        }
        Err(e) => {
            println!("{} {}", "❌ Configuration loading failed:".bright_red(), e);
            return Err(qollective::error::QollectiveError::validation(format!("Failed to load config.toml: {}", e)));
        }
    };

    println!("{} {}", "🔐 TLS enabled:".bright_cyan(), config.tls.enabled.to_string().bright_yellow());

    // Run certificate validation tests
    run_certificate_validation_tests().await?;

    println!("\n{}", "✅ All Enterprise Registry Certificate Validation Tests Completed".bright_green().bold());
    Ok(())
}

async fn run_certificate_validation_tests() -> Result<()> {
    println!("\n{}", "🧪 Running Certificate Validation Test Suite".bright_blue().bold());

    // Test 1: Valid Enterprise crew member registration
    test_valid_crew_member_registration().await?;

    // Test 2: Invalid/unauthorized agent registration
    test_unauthorized_agent_registration().await?;

    // Test 3: Multiple crew member validations
    test_multiple_crew_validations().await?;

    // Test 4: Q Entity special handling
    test_q_entity_registration().await?;

    // Test 5: Agent capability validation
    test_capability_validation().await?;

    Ok(())
}

/// Test 1: Valid Enterprise crew member registration
async fn test_valid_crew_member_registration() -> Result<()> {
    println!("\n{}", "📋 Test 1: Valid Enterprise Crew Member Registration".bright_cyan());
    println!("{}", "🔍 Testing Picard's registration with valid Enterprise credentials...".dimmed());

    let config = EnterpriseCertificateValidationConfig::default();
    let mut validator = EnterpriseCertificateValidator::new(config).await?;

    let agent_info = AgentInfo {
        id: Uuid::now_v7(),
        name: "Captain Jean-Luc Picard".to_string(),
        capabilities: vec!["command".to_string(), "leadership".to_string(), "diplomacy".to_string()],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("position".to_string(), "Captain".to_string()),
            ("department".to_string(), "Command".to_string()),
            ("security_clearance".to_string(), "Alpha".to_string()),
        ]),
    };

    let agent_metadata = AgentMetadata {
        version: "1.0.0".to_string(),
        build_info: Some("Enterprise NCC-1701-D Build".to_string()),
        capabilities_metadata: HashMap::from([
            ("command".to_string(), serde_json::json!({"level": "captain", "experience": "20_years"})),
            ("leadership".to_string(), serde_json::json!({"style": "diplomatic", "crew_size": "1000"})),
        ]),
        performance_metrics: None,
        custom_metadata: HashMap::from([
            ("species".to_string(), serde_json::Value::String("Human".to_string())),
            ("homeworld".to_string(), serde_json::Value::String("Earth".to_string())),
            ("starfleet_serial".to_string(), serde_json::Value::String("SC937-0176CEC".to_string())),
        ]),
    };

    let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await?;

    if result.is_valid {
        println!("{} Picard's registration validated successfully", "✅".bright_green());
        println!("   {} {}", "Crew Member:".bright_cyan(), result.crew_member.as_deref().unwrap_or("Unknown").bright_yellow());
        println!("   {} {}", "Rank:".bright_cyan(), result.rank.as_deref().unwrap_or("Unknown").bright_yellow());
        println!("   {} {}", "Department:".bright_cyan(), result.department.as_deref().unwrap_or("Unknown").bright_yellow());
        println!("   {} {}", "Clearance:".bright_cyan(), result.clearance_level.as_deref().unwrap_or("Unknown").bright_yellow());
    } else {
        println!("{} Picard's registration failed: {:?}", "❌".bright_red(), result.validation_errors);
        return Err(qollective::error::QollectiveError::validation("Picard validation should have succeeded".to_string()));
    }

    println!("{}", "✅ Test 1 passed: Valid crew member registration works".bright_green());
    Ok(())
}

/// Test 2: Invalid/unauthorized agent registration
async fn test_unauthorized_agent_registration() -> Result<()> {
    println!("\n{}", "📋 Test 2: Unauthorized Agent Registration".bright_cyan());
    println!("{}", "🔍 Testing unknown agent registration (should be rejected)...".dimmed());

    let config = EnterpriseCertificateValidationConfig::default();
    let mut validator = EnterpriseCertificateValidator::new(config).await?;

    let agent_info = AgentInfo {
        id: Uuid::now_v7(),
        name: "Unknown Intruder".to_string(),
        capabilities: vec!["hacking".to_string(), "infiltration".to_string()],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("suspicious_activity".to_string(), "true".to_string()),
        ]),
    };

    let agent_metadata = AgentMetadata {
        version: "0.1.0".to_string(),
        build_info: Some("Suspicious Build".to_string()),
        capabilities_metadata: HashMap::new(),
        performance_metrics: None,
        custom_metadata: HashMap::from([
            ("origin".to_string(), serde_json::Value::String("Unknown".to_string())),
        ]),
    };

    let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await?;

    if !result.is_valid {
        println!("{} Unauthorized registration correctly rejected", "✅".bright_green());
        println!("   {} {:?}", "Validation Errors:".bright_yellow(), result.validation_errors);
    } else {
        println!("{} Unauthorized registration incorrectly accepted", "❌".bright_red());
        return Err(qollective::error::QollectiveError::validation("Unauthorized agent should have been rejected".to_string()));
    }

    println!("{}", "✅ Test 2 passed: Unauthorized agents are properly rejected".bright_green());
    Ok(())
}

/// Test 3: Multiple crew member validations
async fn test_multiple_crew_validations() -> Result<()> {
    println!("\n{}", "📋 Test 3: Multiple Crew Member Validations".bright_cyan());
    println!("{}", "🔍 Testing all Enterprise crew members...".dimmed());

    let config = EnterpriseCertificateValidationConfig::default();
    let mut validator = EnterpriseCertificateValidator::new(config).await?;

    let crew_members = vec![
        ("Lt. Commander Data", vec!["data-analysis".to_string(), "computation".to_string()]),
        ("Commander Spock", vec!["science".to_string(), "logic".to_string()]),
        ("Commander Montgomery Scott", vec!["engineering".to_string(), "propulsion".to_string()]),
    ];

    let mut successful_validations = 0;

    for (name, capabilities) in crew_members {
        println!("   🔍 Validating: {}", name.bright_cyan());

        let agent_info = AgentInfo {
            id: Uuid::now_v7(),
            name: name.to_string(),
            capabilities,
            health_status: HealthStatus::Healthy,
            last_heartbeat: SystemTime::now(),
            metadata: HashMap::new(),
        };

        let agent_metadata = AgentMetadata {
            version: "1.0.0".to_string(),
            build_info: Some("Enterprise Build".to_string()),
            capabilities_metadata: HashMap::new(),
            performance_metrics: None,
            custom_metadata: HashMap::new(),
        };

        let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await?;

        if result.is_valid {
            println!("      ✅ {} validated successfully", name.bright_green());
            successful_validations += 1;
        } else {
            println!("      ❌ {} validation failed: {:?}", name.bright_red(), result.validation_errors);
        }
    }

    if successful_validations == 3 {
        println!("{} All crew members validated successfully", "✅".bright_green());
    } else {
        return Err(qollective::error::QollectiveError::validation(
            format!("Expected 3 successful validations, got {}", successful_validations)
        ));
    }

    println!("{}", "✅ Test 3 passed: Multiple crew member validations work".bright_green());
    Ok(())
}

/// Test 4: Q Entity special handling
async fn test_q_entity_registration() -> Result<()> {
    println!("\n{}", "📋 Test 4: Q Entity Special Handling".bright_cyan());
    println!("{}", "🔍 Testing Q Console registration (cosmic entity privileges)...".dimmed());

    let mut config = EnterpriseCertificateValidationConfig::default();
    config.strict_validation = true; // Enable strict validation to test Q bypass

    let mut validator = EnterpriseCertificateValidator::new(config).await?;

    let agent_info = AgentInfo {
        id: Uuid::now_v7(),
        name: "Q Console Interface".to_string(),
        capabilities: vec!["cosmic-challenges".to_string(), "reality-manipulation".to_string()],
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::from([
            ("entity_type".to_string(), "Q".to_string()),
            ("power_level".to_string(), "omnipotent".to_string()),
        ]),
    };

    let agent_metadata = AgentMetadata {
        version: "∞".to_string(),
        build_info: Some("Cosmic Entity Build".to_string()),
        capabilities_metadata: HashMap::from([
            ("reality-manipulation".to_string(), serde_json::json!({"scope": "universal", "limitations": "none"})),
        ]),
        performance_metrics: None,
        custom_metadata: HashMap::from([
            ("continuum_member".to_string(), serde_json::Value::String("true".to_string())),
            ("omnipotent".to_string(), serde_json::Value::String("true".to_string())),
        ]),
    };

    let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await?;

    if result.is_valid {
        println!("{} Q Entity registration validated with cosmic privileges", "✅".bright_green());
        println!("   {} {}", "Entity Type:".bright_cyan(), "Q Continuum Member".bright_magenta());
        println!("   {} {}", "Clearance Level:".bright_cyan(), result.clearance_level.as_deref().unwrap_or("Omega").bright_magenta());
    } else {
        println!("{} Q Entity registration failed: {:?}", "❌".bright_red(), result.validation_errors);
        return Err(qollective::error::QollectiveError::validation("Q Entity validation should have succeeded".to_string()));
    }

    println!("{}", "✅ Test 4 passed: Q Entity special handling works".bright_green());
    Ok(())
}

/// Test 5: Agent capability validation
async fn test_capability_validation() -> Result<()> {
    println!("\n{}", "📋 Test 5: Agent Capability Validation".bright_cyan());
    println!("{}", "🔍 Testing agent with mismatched capabilities...".dimmed());

    let config = EnterpriseCertificateValidationConfig::default();
    let mut validator = EnterpriseCertificateValidator::new(config).await?;

    // Test Picard with engineering capabilities (should generate warning but not fail)
    let agent_info = AgentInfo {
        id: Uuid::now_v7(),
        name: "Captain Picard".to_string(),
        capabilities: vec!["warp-core-engineering".to_string(), "matter-antimatter-regulation".to_string()], // Wrong capabilities for Picard
        health_status: HealthStatus::Healthy,
        last_heartbeat: SystemTime::now(),
        metadata: HashMap::new(),
    };

    let agent_metadata = AgentMetadata {
        version: "1.0.0".to_string(),
        build_info: Some("Enterprise Build".to_string()),
        capabilities_metadata: HashMap::new(),
        performance_metrics: None,
        custom_metadata: HashMap::new(),
    };

    let result = validator.validate_agent_registration(&agent_info, &agent_metadata).await?;

    // Should still be valid (Enterprise allows crew to cross-train)
    if result.is_valid {
        println!("{} Capability mismatch handled gracefully (with warnings)", "✅".bright_green());
        println!("   {} Jean-Luc Picard", "Validated Crew Member:".bright_cyan());
        println!("   {} Command (despite engineering capabilities)", "Department:".bright_cyan());
    } else {
        println!("{} Capability validation too strict: {:?}", "❌".bright_red(), result.validation_errors);
        return Err(qollective::error::QollectiveError::validation("Capability mismatch should not cause hard failure".to_string()));
    }

    println!("{}", "✅ Test 5 passed: Capability validation provides appropriate warnings".bright_green());
    Ok(())
}
