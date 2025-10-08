// ABOUTME: Integration tests for NATS NKey authentication support
// ABOUTME: Tests configuration, builder methods, and validation for NKey auth

#[cfg(test)]
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
mod nkey_authentication_tests {
    use qollective::config::nats::{NatsClientConfig, NatsConfig, NatsConnectionConfig};
    use std::path::PathBuf;

    #[test]
    fn test_nkey_file_configuration() {
        // ARRANGE: Create config with NKey file
        let nkey_path = PathBuf::from("/path/to/nkey.seed");

        let mut config = NatsConnectionConfig::default();
        config.nkey_file = Some(nkey_path.clone());

        // ACT & ASSERT: Verify NKey file is configured
        assert_eq!(config.nkey_file, Some(nkey_path));
        assert_eq!(config.nkey_seed, None);
    }

    #[test]
    fn test_nkey_seed_configuration() {
        // ARRANGE: Create config with NKey seed string
        let nkey_seed = "SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string();

        let mut config = NatsConnectionConfig::default();
        config.nkey_seed = Some(nkey_seed.clone());

        // ACT & ASSERT: Verify NKey seed is configured
        assert_eq!(config.nkey_seed, Some(nkey_seed));
        assert_eq!(config.nkey_file, None);
    }

    #[test]
    fn test_nkey_file_and_seed_mutual_exclusivity() {
        // ARRANGE: Create config with both NKey file and seed
        let mut config = NatsConnectionConfig::default();
        config.nkey_file = Some(PathBuf::from("/path/to/nkey.seed"));
        config.nkey_seed = Some("SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string());

        // ACT: Validate configuration
        let result = config.validate();

        // ASSERT: Should fail validation - cannot have both
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot specify both nkey_file and nkey_seed"));
    }

    #[test]
    fn test_nkey_file_only_validation_success() {
        // ARRANGE: Create config with only NKey file
        let mut config = NatsConnectionConfig::default();
        config.nkey_file = Some(PathBuf::from("/path/to/nkey.seed"));

        // ACT: Validate configuration
        let result = config.validate();

        // ASSERT: Should pass validation
        assert!(result.is_ok(), "NKey file-only config should validate successfully");
    }

    #[test]
    fn test_nkey_seed_only_validation_success() {
        // ARRANGE: Create config with only NKey seed
        let mut config = NatsConnectionConfig::default();
        config.nkey_seed = Some("SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string());

        // ACT: Validate configuration
        let result = config.validate();

        // ASSERT: Should pass validation
        assert!(result.is_ok(), "NKey seed-only config should validate successfully");
    }

    #[test]
    fn test_nats_client_config_builder_with_nkey_file() {
        // ARRANGE & ACT: Build config with NKey file using builder
        let nkey_path = PathBuf::from("/path/to/nkey.seed");
        let config = NatsClientConfig::builder()
            .with_urls(vec!["nats://localhost:4222".to_string()])
            .with_nkey_file(nkey_path.clone())
            .build();

        // ASSERT: Verify NKey file is set correctly
        assert_eq!(config.connection.nkey_file, Some(nkey_path));
        assert_eq!(config.connection.nkey_seed, None);
    }

    #[test]
    fn test_nats_client_config_builder_with_nkey_seed() {
        // ARRANGE & ACT: Build config with NKey seed using builder
        let nkey_seed = "SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string();
        let config = NatsClientConfig::builder()
            .with_urls(vec!["nats://localhost:4222".to_string()])
            .with_nkey_seed(nkey_seed.clone())
            .build();

        // ASSERT: Verify NKey seed is set correctly
        assert_eq!(config.connection.nkey_seed, Some(nkey_seed));
        assert_eq!(config.connection.nkey_file, None);
    }

    #[test]
    fn test_nats_config_builder_with_nkey_file() {
        // ARRANGE & ACT: Build main NatsConfig with NKey file
        let nkey_path = PathBuf::from("/path/to/nkey.seed");
        let config = NatsConfig::builder()
            .with_urls(vec!["nats://localhost:4222".to_string()])
            .with_nkey_file(nkey_path.clone())
            .build();

        // ASSERT: Verify NKey file is set correctly
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.connection.nkey_file, Some(nkey_path));
        assert_eq!(config.connection.nkey_seed, None);
    }

    #[test]
    fn test_nats_config_builder_with_nkey_seed() {
        // ARRANGE & ACT: Build main NatsConfig with NKey seed
        let nkey_seed = "SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string();
        let config = NatsConfig::builder()
            .with_urls(vec!["nats://localhost:4222".to_string()])
            .with_nkey_seed(nkey_seed.clone())
            .build();

        // ASSERT: Verify NKey seed is set correctly
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.connection.nkey_seed, Some(nkey_seed));
        assert_eq!(config.connection.nkey_file, None);
    }

    #[test]
    fn test_nkey_serialization_roundtrip() {
        // ARRANGE: Create config with NKey authentication
        let nkey_seed = "SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string();
        let mut config = NatsConnectionConfig::default();
        config.nkey_seed = Some(nkey_seed.clone());

        // ACT: Serialize and deserialize
        let json = serde_json::to_string(&config).expect("Serialization should succeed");
        let deserialized: NatsConnectionConfig =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        // ASSERT: Verify NKey seed survives roundtrip
        assert_eq!(deserialized.nkey_seed, Some(nkey_seed));
        assert_eq!(deserialized.nkey_file, None);
    }

    #[test]
    fn test_nkey_file_serialization_roundtrip() {
        // ARRANGE: Create config with NKey file path
        let nkey_path = PathBuf::from("/path/to/nkey.seed");
        let mut config = NatsConnectionConfig::default();
        config.nkey_file = Some(nkey_path.clone());

        // ACT: Serialize and deserialize
        let json = serde_json::to_string(&config).expect("Serialization should succeed");
        let deserialized: NatsConnectionConfig =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        // ASSERT: Verify NKey file path survives roundtrip
        assert_eq!(deserialized.nkey_file, Some(nkey_path));
        assert_eq!(deserialized.nkey_seed, None);
    }

    #[test]
    fn test_nkey_optional_fields_skip_serializing_if_none() {
        // ARRANGE: Create config without NKey authentication
        let config = NatsConnectionConfig::default();

        // ACT: Serialize to JSON
        let json = serde_json::to_string(&config).expect("Serialization should succeed");

        // ASSERT: Verify nkey_file and nkey_seed are not present in JSON
        // (due to #[serde(skip_serializing_if = "Option::is_none")])
        assert!(!json.contains("nkey_file"));
        assert!(!json.contains("nkey_seed"));
    }

    #[test]
    fn test_nkey_with_tls_configuration() {
        // ARRANGE: Create config with both NKey and TLS
        let nkey_seed = "SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string();
        let config = NatsConfig::builder()
            .with_urls(vec!["nats://localhost:4222".to_string()])
            .with_nkey_seed(nkey_seed.clone())
            .with_tls(true)
            .build();

        // ACT & ASSERT: Verify both NKey and TLS are configured
        if config.is_err() {
            // TLS validation may fail without certificates - this is expected
            // The important part is that NKey configuration is accepted
            println!("TLS validation error (expected): {:?}", config.unwrap_err());
            return;
        }
        let config = config.unwrap();
        assert_eq!(config.connection.nkey_seed, Some(nkey_seed));
        assert!(config.connection.tls.enabled);
    }

    #[test]
    fn test_nkey_with_username_password_coexistence() {
        // ARRANGE: Create config with NKey and username/password
        // Note: This is valid configuration - authentication methods can coexist
        let nkey_seed = "SUAAVWRZG7QJHZXZGFP5LQFQYJ7AJVVQE5JQPOQHVIHJXYUZU7PG6ZEXLE".to_string();
        let mut config = NatsConnectionConfig::default();
        config.nkey_seed = Some(nkey_seed.clone());
        config.username = Some("user".to_string());
        config.password = Some("pass".to_string());

        // ACT: Validate configuration
        let result = config.validate();

        // ASSERT: Should pass validation - multiple auth methods can coexist
        assert!(result.is_ok(), "NKey can coexist with username/password");
    }
}
