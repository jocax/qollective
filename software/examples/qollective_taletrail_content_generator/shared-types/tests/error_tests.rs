use shared_types::errors::{TaleTrailError, Result};

#[test]
fn test_network_error() {
    let err = TaleTrailError::NetworkError("Connection refused".to_string());
    assert!(err.to_string().contains("Network error"));
    assert!(err.to_string().contains("Connection refused"));
}

#[test]
fn test_nats_error() {
    let err = TaleTrailError::NatsError("Failed to connect to NATS server".to_string());
    assert!(err.to_string().contains("NATS error"));
    assert!(err.to_string().contains("Failed to connect"));
}

#[test]
fn test_nats_tls_error() {
    let err = TaleTrailError::NatsTlsError("Invalid certificate".to_string());
    assert!(err.to_string().contains("NATS TLS error"));
    assert!(err.to_string().contains("Invalid certificate"));
}

#[test]
fn test_nats_nkey_error() {
    let err = TaleTrailError::NatsNKeyError("Invalid NKey format".to_string());
    assert!(err.to_string().contains("NATS NKey error"));
    assert!(err.to_string().contains("Invalid NKey format"));
}

#[test]
fn test_qollective_error() {
    let err = TaleTrailError::QollectiveError("Envelope validation failed".to_string());
    assert!(err.to_string().contains("Qollective error"));
    assert!(err.to_string().contains("Envelope validation"));
}

#[test]
fn test_tls_certificate_error() {
    let err = TaleTrailError::TlsCertificateError("Certificate expired".to_string());
    assert!(err.to_string().contains("TLS certificate error"));
    assert!(err.to_string().contains("Certificate expired"));
}

#[test]
fn test_config_error() {
    let err = TaleTrailError::ConfigError("Missing required field: nats_url".to_string());
    assert!(err.to_string().contains("Configuration error"));
    assert!(err.to_string().contains("Missing required field"));
}

#[test]
fn test_validation_error() {
    let err = TaleTrailError::ValidationError("Invalid age group".to_string());
    assert!(err.to_string().contains("Validation error"));
    assert!(err.to_string().contains("Invalid age group"));
}

#[test]
fn test_generation_error() {
    let err = TaleTrailError::GenerationError("Failed to generate DAG structure".to_string());
    assert!(err.to_string().contains("Generation error"));
    assert!(err.to_string().contains("Failed to generate DAG"));
}

#[test]
fn test_llm_error() {
    let err = TaleTrailError::LLMError("Model not found".to_string());
    assert!(err.to_string().contains("LLM error"));
    assert!(err.to_string().contains("Model not found"));
}

#[test]
fn test_timeout_error() {
    let err = TaleTrailError::TimeoutError;
    let msg = err.to_string();
    assert!(msg.contains("Operation timed out"));
}

#[test]
fn test_retry_exhausted_error() {
    let err = TaleTrailError::RetryExhausted;
    let msg = err.to_string();
    assert!(msg.contains("Retry attempts exhausted"));
}

#[test]
fn test_serialization_error() {
    let err = TaleTrailError::SerializationError("Invalid JSON format".to_string());
    assert!(err.to_string().contains("Serialization error"));
    assert!(err.to_string().contains("Invalid JSON"));
}

#[test]
fn test_invalid_request_error() {
    let err = TaleTrailError::InvalidRequest("Missing theme field".to_string());
    assert!(err.to_string().contains("Invalid request"));
    assert!(err.to_string().contains("Missing theme"));
}

#[test]
fn test_result_type_alias_ok() {
    let result: Result<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_result_type_alias_err() {
    let result: Result<i32> = Err(TaleTrailError::ValidationError("Test error".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Validation error"));
}

#[test]
fn test_error_conversion_in_functions() {
    fn validate_age_group(age_group: &str) -> Result<i32> {
        match age_group {
            "6-8" => Ok(6),
            "9-11" => Ok(9),
            _ => Err(TaleTrailError::ValidationError(
                format!("Invalid age group: {}", age_group)
            )),
        }
    }

    assert_eq!(validate_age_group("6-8").unwrap(), 6);
    assert!(validate_age_group("invalid").is_err());
}

#[test]
fn test_error_propagation() {
    fn inner_function() -> Result<String> {
        Err(TaleTrailError::NetworkError("Connection failed".to_string()))
    }

    fn outer_function() -> Result<String> {
        inner_function()?;
        Ok("Success".to_string())
    }

    let result = outer_function();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Network error"));
}

#[test]
fn test_error_chaining() {
    fn step1() -> Result<i32> {
        Err(TaleTrailError::ConfigError("Invalid config".to_string()))
    }

    fn step2() -> Result<i32> {
        step1().map_err(|e| {
            TaleTrailError::GenerationError(format!("Generation failed due to: {}", e))
        })
    }

    let result = step2();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Generation failed"));
    assert!(error_msg.contains("Invalid config"));
}

#[test]
fn test_error_debug_format() {
    let err = TaleTrailError::LLMError("Model timeout".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("LLMError"));
    assert!(debug_str.contains("Model timeout"));
}

#[test]
fn test_multiple_error_types() {
    let errors = vec![
        TaleTrailError::NetworkError("Network 1".to_string()),
        TaleTrailError::NatsError("NATS 1".to_string()),
        TaleTrailError::ValidationError("Validation 1".to_string()),
    ];

    assert_eq!(errors.len(), 3);
    assert!(errors[0].to_string().contains("Network"));
    assert!(errors[1].to_string().contains("NATS"));
    assert!(errors[2].to_string().contains("Validation"));
}

#[test]
fn test_error_matching() {
    fn handle_error(err: TaleTrailError) -> String {
        match err {
            TaleTrailError::TimeoutError => "Retry operation".to_string(),
            TaleTrailError::RetryExhausted => "Give up".to_string(),
            TaleTrailError::NetworkError(_) => "Check connection".to_string(),
            _ => "Unknown error".to_string(),
        }
    }

    assert_eq!(handle_error(TaleTrailError::TimeoutError), "Retry operation");
    assert_eq!(handle_error(TaleTrailError::RetryExhausted), "Give up");
    assert_eq!(handle_error(TaleTrailError::NetworkError("Test".to_string())), "Check connection");
    assert_eq!(handle_error(TaleTrailError::LLMError("Test".to_string())), "Unknown error");
}

#[test]
fn test_contextual_error_messages() {
    let context = "story-generator service";
    let err = TaleTrailError::NatsError(format!("[{}] Failed to subscribe to subject", context));
    let msg = err.to_string();
    assert!(msg.contains("story-generator service"));
    assert!(msg.contains("Failed to subscribe"));
}

#[test]
fn test_error_with_suggestions() {
    fn create_config_error_with_suggestion(field: &str) -> TaleTrailError {
        TaleTrailError::ConfigError(
            format!(
                "Missing required field '{}'. Please add it to config.toml or set environment variable {}",
                field,
                field.to_uppercase()
            )
        )
    }

    let err = create_config_error_with_suggestion("nats_url");
    let msg = err.to_string();
    assert!(msg.contains("Missing required field 'nats_url'"));
    assert!(msg.contains("NATS_URL"));
    assert!(msg.contains("config.toml"));
}

#[test]
fn test_error_suggestion_method() {
    let nats_err = TaleTrailError::NatsError("Connection failed".to_string());
    assert!(nats_err.suggestion().is_some());
    assert!(nats_err.suggestion().unwrap().contains("docker ps"));

    let tls_err = TaleTrailError::NatsTlsError("Invalid cert".to_string());
    assert!(tls_err.suggestion().is_some());
    assert!(tls_err.suggestion().unwrap().contains("setup-tls.sh"));

    let config_err = TaleTrailError::ConfigError("Missing field".to_string());
    assert!(config_err.suggestion().is_some());
    assert!(config_err.suggestion().unwrap().contains("config.toml"));

    let timeout_err = TaleTrailError::TimeoutError;
    assert!(timeout_err.suggestion().is_some());
    assert!(timeout_err.suggestion().unwrap().contains("timeout"));

    let network_err = TaleTrailError::NetworkError("Connection failed".to_string());
    assert!(network_err.suggestion().is_none());
}

#[test]
fn test_error_category_method() {
    assert_eq!(TaleTrailError::NetworkError("Test".to_string()).category(), "network");
    assert_eq!(TaleTrailError::NatsError("Test".to_string()).category(), "network");
    assert_eq!(TaleTrailError::NatsTlsError("Test".to_string()).category(), "network");
    assert_eq!(TaleTrailError::ConfigError("Test".to_string()).category(), "configuration");
    assert_eq!(TaleTrailError::ValidationError("Test".to_string()).category(), "validation");
    assert_eq!(TaleTrailError::InvalidRequest("Test".to_string()).category(), "validation");
    assert_eq!(TaleTrailError::GenerationError("Test".to_string()).category(), "generation");
    assert_eq!(TaleTrailError::LLMError("Test".to_string()).category(), "generation");
    assert_eq!(TaleTrailError::QollectiveError("Test".to_string()).category(), "framework");
    assert_eq!(TaleTrailError::TlsCertificateError("Test".to_string()).category(), "security");
    assert_eq!(TaleTrailError::TimeoutError.category(), "reliability");
    assert_eq!(TaleTrailError::RetryExhausted.category(), "reliability");
    assert_eq!(TaleTrailError::SerializationError("Test".to_string()).category(), "serialization");
}

#[test]
fn test_error_is_retryable_method() {
    // Retryable errors
    assert!(TaleTrailError::NetworkError("Test".to_string()).is_retryable());
    assert!(TaleTrailError::NatsError("Test".to_string()).is_retryable());
    assert!(TaleTrailError::TimeoutError.is_retryable());
    assert!(TaleTrailError::LLMError("Test".to_string()).is_retryable());

    // Non-retryable errors
    assert!(!TaleTrailError::ValidationError("Test".to_string()).is_retryable());
    assert!(!TaleTrailError::ConfigError("Test".to_string()).is_retryable());
    assert!(!TaleTrailError::InvalidRequest("Test".to_string()).is_retryable());
    assert!(!TaleTrailError::RetryExhausted.is_retryable());
    assert!(!TaleTrailError::SerializationError("Test".to_string()).is_retryable());
}

#[test]
fn test_error_retry_logic() {
    fn should_retry(err: &TaleTrailError, attempt: u32, max_attempts: u32) -> bool {
        err.is_retryable() && attempt < max_attempts
    }

    let retryable_err = TaleTrailError::NetworkError("Connection lost".to_string());
    assert!(should_retry(&retryable_err, 1, 3));
    assert!(should_retry(&retryable_err, 2, 3));
    assert!(!should_retry(&retryable_err, 3, 3));

    let non_retryable_err = TaleTrailError::ValidationError("Invalid input".to_string());
    assert!(!should_retry(&non_retryable_err, 0, 3));
}

#[test]
fn test_error_clone() {
    let original = TaleTrailError::GenerationError("DAG creation failed".to_string());
    let cloned = original.clone();

    assert_eq!(original.to_string(), cloned.to_string());
    assert_eq!(original.category(), cloned.category());
    assert_eq!(original.is_retryable(), cloned.is_retryable());
}
