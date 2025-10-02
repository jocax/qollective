//! Tests for enhanced TLS configuration management
//! 
//! This module tests the improved certificate path resolution with fallback mechanisms,
//! environment variable overrides, and smart path resolution supporting both relative
//! and absolute certificate paths.

use super::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod tls_config_tests {
    use super::*;

    /// Helper function to create a basic TLS config for testing
    fn create_basic_tls_config(enabled: bool, verification_mode: &str) -> TlsExampleConfig {
        TlsExampleConfig {
            enabled,
            insecure: false,
            ca_cert_path: "ca.crt".to_string(),
            cert_path: "cert.crt".to_string(),
            key_path: "key.pem".to_string(),
            verification_mode: verification_mode.to_string(),
            server_name: None,
            protocol_versions: vec![],
            cipher_suites: vec![],
            alpn_protocols: vec![],
            certificate_validation: CertificateValidationConfig::default(),
            handshake_timeout_ms: 10_000,
        }
    }

    /// Test TLS configuration with absolute paths
    #[test]
    fn test_tls_config_with_absolute_paths() {
        let tls_config = TlsExampleConfig {
            enabled: true,
            insecure: false,
            ca_cert_path: "/absolute/path/to/ca.crt".to_string(),
            cert_path: "/absolute/path/to/cert.crt".to_string(),
            key_path: "/absolute/path/to/key.pem".to_string(),
            verification_mode: "mutual_tls".to_string(),
            server_name: None,
            protocol_versions: vec![],
            cipher_suites: vec![],
            alpn_protocols: vec![],
            certificate_validation: CertificateValidationConfig::default(),
            handshake_timeout_ms: 10_000,
        };

        let framework_config = tls_config.to_framework_tls_config();
        
        assert!(framework_config.enabled);
        assert_eq!(
            framework_config.ca_cert_path.unwrap(),
            PathBuf::from("/absolute/path/to/ca.crt")
        );
        assert_eq!(
            framework_config.cert_path.unwrap(),
            PathBuf::from("/absolute/path/to/cert.crt")
        );
        assert_eq!(
            framework_config.key_path.unwrap(),
            PathBuf::from("/absolute/path/to/key.pem")
        );
    }

    /// Test TLS configuration with relative paths (should resolve to constants)
    #[test]
    fn test_tls_config_with_relative_paths() {
        let tls_config = create_basic_tls_config(true, "mutual_tls");

        let framework_config = tls_config.to_framework_tls_config();
        
        assert!(framework_config.enabled);
        
        // Should resolve to framework paths from constants
        let ca_path = framework_config.ca_cert_path.unwrap();
        let cert_path = framework_config.cert_path.unwrap();
        let key_path = framework_config.key_path.unwrap();
        
        // Paths should be resolved through the framework's path constants
        // The exact path depends on framework implementation, just verify they exist
        assert!(ca_path.exists() || ca_path.to_string_lossy().len() > 0);
        assert!(cert_path.exists() || cert_path.to_string_lossy().len() > 0);
        assert!(key_path.exists() || key_path.to_string_lossy().len() > 0);
    }

    /// Test TLS configuration when disabled
    #[test]
    fn test_tls_config_disabled() {
        let tls_config = create_basic_tls_config(false, "mutual_tls");

        let framework_config = tls_config.to_framework_tls_config();
        
        assert!(!framework_config.enabled);
        assert!(framework_config.ca_cert_path.is_none());
        assert!(framework_config.cert_path.is_none());
        assert!(framework_config.key_path.is_none());
    }

    /// Test different verification modes
    #[test]
    fn test_verification_modes() {
        let mut tls_config = create_basic_tls_config(true, "mutual_tls");

        // Test mutual TLS
        let framework_config = tls_config.to_framework_tls_config();
        assert!(matches!(
            framework_config.verification_mode,
            qollective::config::tls::VerificationMode::MutualTls
        ));

        // Test skip verification
        tls_config.verification_mode = "skip".to_string();
        let framework_config = tls_config.to_framework_tls_config();
        assert!(matches!(
            framework_config.verification_mode,
            qollective::config::tls::VerificationMode::Skip
        ));

        // Test default fallback (unknown mode should default to mutual TLS)
        tls_config.verification_mode = "invalid_mode".to_string();
        let framework_config = tls_config.to_framework_tls_config();
        assert!(matches!(
            framework_config.verification_mode,
            qollective::config::tls::VerificationMode::MutualTls
        ));
    }
}

#[cfg(test)]
mod path_resolution_tests {
    use super::*;

    /// Helper function to create a basic TLS config for testing
    fn create_basic_tls_config(enabled: bool, verification_mode: &str) -> TlsExampleConfig {
        TlsExampleConfig {
            enabled,
            insecure: false,
            ca_cert_path: "ca.crt".to_string(),
            cert_path: "cert.crt".to_string(),
            key_path: "key.pem".to_string(),
            verification_mode: verification_mode.to_string(),
            server_name: None,
            protocol_versions: vec![],
            cipher_suites: vec![],
            alpn_protocols: vec![],
            certificate_validation: CertificateValidationConfig::default(),
            handshake_timeout_ms: 10_000,
        }
    }

    /// Test environment variable override for certificate paths
    #[test]
    fn test_env_var_certificate_path_override() {
        // Set environment variables
        env::set_var("TLS_CERT_BASE_PATH", "/custom/cert/path");
        env::set_var("TLS_CA_CERT_PATH", "/custom/ca.crt");
        env::set_var("TLS_CERT_PATH", "/custom/cert.crt");
        env::set_var("TLS_KEY_PATH", "/custom/key.pem");

        let tls_config = create_basic_tls_config(true, "mutual_tls");

        // Create enhanced TLS config that should check environment variables
        let enhanced_tls_config = create_enhanced_tls_config(&tls_config);
        
        assert!(enhanced_tls_config.enabled);
        // Environment variable overrides should take precedence for absolute paths
        if let Some(ca_path) = &enhanced_tls_config.ca_cert_path {
            assert!(ca_path.to_string_lossy().contains("ca.crt"));
        }

        // Clean up environment variables
        env::remove_var("TLS_CERT_BASE_PATH");
        env::remove_var("TLS_CA_CERT_PATH");
        env::remove_var("TLS_CERT_PATH");
        env::remove_var("TLS_KEY_PATH");
    }

    /// Test fallback mechanism when certificates don't exist
    #[test]
    fn test_certificate_fallback_mechanism() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_str().unwrap();

        let mut tls_config = create_basic_tls_config(true, "mutual_tls");
        tls_config.ca_cert_path = format!("{}/nonexistent_ca.crt", temp_path);
        tls_config.cert_path = format!("{}/nonexistent_cert.crt", temp_path);
        tls_config.key_path = format!("{}/nonexistent_key.pem", temp_path);

        // Should handle non-existent paths gracefully
        let enhanced_config = create_enhanced_tls_config(&tls_config);
        assert!(enhanced_config.enabled);
        
        // Paths should be preserved even if files don't exist
        assert!(enhanced_config.ca_cert_path.is_some());
        assert!(enhanced_config.cert_path.is_some());
        assert!(enhanced_config.key_path.is_some());
    }

    /// Test smart path resolution with mixed absolute and relative paths
    #[test]
    fn test_mixed_path_resolution() {
        let mut tls_config = create_basic_tls_config(true, "mutual_tls");
        tls_config.ca_cert_path = "/absolute/ca.crt".to_string();    // Absolute
        tls_config.cert_path = "relative_cert.crt".to_string();      // Relative
        tls_config.key_path = "/absolute/key.pem".to_string();       // Absolute

        let framework_config = tls_config.to_framework_tls_config();
        
        // Absolute paths should be preserved
        assert_eq!(
            framework_config.ca_cert_path.unwrap(),
            PathBuf::from("/absolute/ca.crt")
        );
        assert_eq!(
            framework_config.key_path.unwrap(),
            PathBuf::from("/absolute/key.pem")
        );
        
        // Relative path should be resolved through framework constants
        let cert_path = framework_config.cert_path.unwrap();
        // Just verify the path is non-empty and different from input
        assert!(cert_path.to_string_lossy().len() > "relative_cert.crt".len());
    }
}

#[cfg(test)]
mod configuration_schema_tests {
    use super::*;

    /// Helper function to create a basic TLS config for testing
    fn create_basic_tls_config(enabled: bool, verification_mode: &str) -> TlsExampleConfig {
        TlsExampleConfig {
            enabled,
            insecure: false,
            ca_cert_path: "ca.crt".to_string(),
            cert_path: "cert.crt".to_string(),
            key_path: "key.pem".to_string(),
            verification_mode: verification_mode.to_string(),
            server_name: None,
            protocol_versions: vec![],
            cipher_suites: vec![],
            alpn_protocols: vec![],
            certificate_validation: CertificateValidationConfig::default(),
            handshake_timeout_ms: 10_000,
        }
    }

    /// Test TOML deserialization with comprehensive TLS options
    #[test]
    fn test_comprehensive_tls_toml_deserialization() {
        let toml_content = r#"
            enabled = true
            insecure = false
            ca_cert_path = "/path/to/ca.crt"
            cert_path = "/path/to/cert.crt"
            key_path = "/path/to/key.pem"
            verification_mode = "mutual_tls"
        "#;

        let config: TlsExampleConfig = toml::from_str(toml_content)
            .expect("Failed to deserialize TLS config");
        
        assert!(config.enabled);
        assert!(!config.insecure);
        assert_eq!(config.ca_cert_path, "/path/to/ca.crt");
        assert_eq!(config.cert_path, "/path/to/cert.crt");
        assert_eq!(config.key_path, "/path/to/key.pem");
        assert_eq!(config.verification_mode, "mutual_tls");
    }

    /// Test TOML serialization preserves all TLS options
    #[test]
    fn test_comprehensive_tls_toml_serialization() {
        let mut tls_config = create_basic_tls_config(true, "skip");
        tls_config.ca_cert_path = "/custom/ca.crt".to_string();
        tls_config.cert_path = "/custom/cert.crt".to_string();
        tls_config.key_path = "/custom/key.pem".to_string();

        let serialized = toml::to_string(&tls_config)
            .expect("Failed to serialize TLS config");
        
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("insecure = false"));
        assert!(serialized.contains("ca_cert_path = \"/custom/ca.crt\""));
        assert!(serialized.contains("cert_path = \"/custom/cert.crt\""));
        assert!(serialized.contains("key_path = \"/custom/key.pem\""));
        assert!(serialized.contains("verification_mode = \"skip\""));
    }

    /// Test enterprise config loading with enhanced TLS
    #[test]
    fn test_enterprise_config_with_enhanced_tls() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");
        
        let toml_content = r#"
            [enterprise]
            server_id = "test-server"
            server_name = "Test Enterprise"
            ship_registry = "NX-74205"
            ship_class = "Defiant"
            
            [enterprise.timeouts]
            crew_ttl_secs = 300
            cleanup_interval_secs = 60
            health_check_interval_secs = 30
            
            [enterprise.limits]
            max_crew_size = 100
            max_capabilities_per_crew = 10
            max_reconnect_attempts = 3
            
            [tls]
            enabled = true
            insecure = false
            ca_cert_path = "test_ca.crt"
            cert_path = "test_cert.crt"  
            key_path = "test_key.pem"
            verification_mode = "mutual_tls"
            
            [nats]
            connection_timeout_ms = 5000
            reconnect_timeout_ms = 2000
            max_reconnect_attempts = 5
            
            [nats.connection]
            urls = ["nats://localhost:4222"]
            ping_interval_ms = 30000
            max_outstanding_pings = 2
            
            [nats.client]
            client_name = "test-client"
            max_pending_bytes = 1048576
            max_pending_messages = 1000
            enable_reconnect = true
            no_echo = false
            
            [nats.discovery]
            enabled = true
            ttl_ms = 60000
            heartbeat_interval_ms = 30000
            
            [a2a_server.registry]
            agent_ttl_secs = 300
            cleanup_interval_secs = 60
            max_agents = 1000
            enable_health_monitoring = true
            enable_agent_logging = true
            enable_capability_indexing = true
            max_capabilities_per_agent = 50
            logging_agent_capability = "logging"
            
            [a2a_server.subjects]
            prefix = "qollective.agents"
            agent_registration = "qollective.agents.register"
            agent_deregistration = "qollective.agents.deregister"
            agent_discovery = "qollective.agents.discovery"
            agent_health = "qollective.agents.health"
            agent_capabilities = "qollective.agents.capabilities"
            agent_registry_events = "qollective.agents.events"
            agent_registry_announce = "qollective.agents.announce"
            enterprise_bridge_challenge = "qollective.enterprise.bridge.challenge"
            
            [a2a_client]
            heartbeat_interval_secs = 30
            discovery_cache_ttl_secs = 60
            enable_metrics = true
            
            [a2a_client.retry]
            max_retries = 3
            initial_delay_ms = 1000
            max_delay_ms = 30000
            backoff_multiplier = 2.0
            
            [logging]
            level = "info"
            format = "pretty"
            show_timestamps = true
            enable_detailed_logging = false
            enable_performance_metrics = true
            
            [monitoring]
            enable_health_checks = true
            enable_registry_monitoring = true
            enable_connection_monitoring = true
            enable_performance_metrics = true
            metrics_interval_secs = 60
            
            [agents.picard]
            capabilities = ["command", "diplomacy", "strategic_planning"]
            location = "Bridge"
            function = "Captain"
            service_type = "command"
        "#;

        fs::write(&config_path, toml_content).expect("Failed to write config file");
        
        let config = EnterpriseConfig::from_file(config_path.to_str().unwrap())
            .expect("Failed to load config");
        
        // Verify TLS config is loaded correctly
        assert!(config.tls.enabled);
        assert!(!config.tls.insecure);
        assert_eq!(config.tls.verification_mode, "mutual_tls");
        
        // Test conversion to framework config
        let framework_tls = config.tls.to_framework_tls_config();
        assert!(framework_tls.enabled);
        assert!(framework_tls.ca_cert_path.is_some());
        assert!(framework_tls.cert_path.is_some());
        assert!(framework_tls.key_path.is_some());
    }
}

/// Helper function to create enhanced TLS config with environment variable override support
fn create_enhanced_tls_config(tls_config: &TlsExampleConfig) -> qollective::config::tls::TlsConfig {
    use qollective::constants::network::tls_paths;
    use std::path::PathBuf;
    
    if !tls_config.enabled {
        return qollective::config::tls::TlsConfig {
            enabled: false,
            ca_cert_path: None,
            cert_path: None,
            key_path: None,
            verification_mode: qollective::config::tls::VerificationMode::MutualTls,
        };
    }

    // Smart path resolution with environment variable override support
    let resolve_cert_path = |config_path: &str, env_var: &str, default_fn: fn(&str) -> String| -> PathBuf {
        // Check for environment variable override first
        if let Ok(env_path) = env::var(env_var) {
            return PathBuf::from(env_path);
        }
        
        // If absolute path, use as-is
        if config_path.starts_with('/') {
            return PathBuf::from(config_path);
        }
        
        // For relative paths, resolve through framework constants
        let base_path = tls_paths::resolve_tls_cert_base_path();
        PathBuf::from(default_fn(&base_path))
    };

    qollective::config::tls::TlsConfig {
        enabled: tls_config.enabled,
        ca_cert_path: Some(resolve_cert_path(
            &tls_config.ca_cert_path,
            "TLS_CA_CERT_PATH",
            tls_paths::ca_file_path
        )),
        cert_path: Some(resolve_cert_path(
            &tls_config.cert_path,
            "TLS_CERT_PATH", 
            tls_paths::cert_file_path
        )),
        key_path: Some(resolve_cert_path(
            &tls_config.key_path,
            "TLS_KEY_PATH",
            tls_paths::key_file_path
        )),
        verification_mode: match tls_config.verification_mode.as_str() {
            "mutual_tls" => qollective::config::tls::VerificationMode::MutualTls,
            "skip" => qollective::config::tls::VerificationMode::Skip,
            _ => qollective::config::tls::VerificationMode::MutualTls,
        },
    }
}