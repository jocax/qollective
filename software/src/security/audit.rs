// ABOUTME: Security audit logging for JWT validation, token operations, and security events
// ABOUTME: Provides structured security event logging for compliance and monitoring requirements

//! Security Audit Logging Module
//!
//! This module provides comprehensive audit logging for security-related events including:
//! - JWT token validation attempts (success/failure)
//! - Token refresh operations
//! - Authentication failures
//! - Authorization decisions
//! - Configuration changes
//! - Suspicious activity detection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use thiserror::Error;

/// Security audit event types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SecurityEventType {
    JwtValidationSuccess,
    JwtValidationFailure,
    TokenRefresh,
    AuthenticationFailure,
    AuthorizationFailure,
    SuspiciousActivity,
    ConfigurationChange,
    PermissionDenied,
}

/// Security audit event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Security audit event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEvent {
    pub event_id: String,
    pub timestamp: SystemTime,
    pub event_type: SecurityEventType,
    pub severity: SecurityEventSeverity,
    pub subject: Option<String>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub result: SecurityEventResult,
    pub details: HashMap<String, serde_json::Value>,
    pub risk_score: Option<u8>,
}

/// Security event result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventResult {
    Success,
    Failure,
    Blocked,
}

/// Security audit logger trait for pluggable logging implementations
pub trait SecurityAuditLogger: Send + Sync {
    /// Log a security event
    fn log_event(&self, event: SecurityAuditEvent) -> Result<(), SecurityAuditError>;

    /// Log JWT validation attempt
    fn log_jwt_validation(
        &self,
        token_id: Option<&str>,
        subject: Option<&str>,
        result: SecurityEventResult,
        details: HashMap<String, serde_json::Value>,
    ) -> Result<(), SecurityAuditError>;

    /// Log authentication attempt
    fn log_authentication(
        &self,
        subject: &str,
        source_ip: Option<&str>,
        result: SecurityEventResult,
    ) -> Result<(), SecurityAuditError>;

    /// Log authorization decision
    fn log_authorization(
        &self,
        subject: &str,
        resource: &str,
        action: &str,
        result: SecurityEventResult,
    ) -> Result<(), SecurityAuditError>;
}

/// Security audit errors
#[derive(Debug, Error)]
pub enum SecurityAuditError {
    #[error("failed to write audit log: {0}")]
    WriteError(String),
    #[error("audit configuration error: {0}")]
    ConfigurationError(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// In-memory security audit logger for testing
pub struct InMemorySecurityAuditLogger {
    events: std::sync::Arc<std::sync::Mutex<Vec<SecurityAuditEvent>>>,
}

impl InMemorySecurityAuditLogger {
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get all logged events (for testing)
    pub fn get_events(&self) -> Vec<SecurityAuditEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Clear all logged events (for testing)
    pub fn clear_events(&self) {
        self.events.lock().unwrap().clear();
    }

    /// Count events by type
    pub fn count_events_by_type(&self, event_type: SecurityEventType) -> usize {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.event_type == event_type)
            .count()
    }
}

impl SecurityAuditLogger for InMemorySecurityAuditLogger {
    fn log_event(&self, event: SecurityAuditEvent) -> Result<(), SecurityAuditError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }

    fn log_jwt_validation(
        &self,
        token_id: Option<&str>,
        subject: Option<&str>,
        result: SecurityEventResult,
        details: HashMap<String, serde_json::Value>,
    ) -> Result<(), SecurityAuditError> {
        let event_type = match result {
            SecurityEventResult::Success => SecurityEventType::JwtValidationSuccess,
            _ => SecurityEventType::JwtValidationFailure,
        };

        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            timestamp: SystemTime::now(),
            event_type,
            severity: match result {
                SecurityEventResult::Success => SecurityEventSeverity::Info,
                SecurityEventResult::Failure => SecurityEventSeverity::Warning,
                SecurityEventResult::Blocked => SecurityEventSeverity::Error,
            },
            subject: subject.map(|s| s.to_string()),
            source_ip: None,
            user_agent: None,
            resource: token_id.map(|id| format!("jwt_token:{}", id)),
            action: "jwt_validation".to_string(),
            result,
            details,
            risk_score: None,
        };

        self.log_event(event)
    }

    fn log_authentication(
        &self,
        subject: &str,
        source_ip: Option<&str>,
        result: SecurityEventResult,
    ) -> Result<(), SecurityAuditError> {
        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::AuthenticationFailure,
            severity: match result {
                SecurityEventResult::Success => SecurityEventSeverity::Info,
                _ => SecurityEventSeverity::Warning,
            },
            subject: Some(subject.to_string()),
            source_ip: source_ip.map(|ip| ip.to_string()),
            user_agent: None,
            resource: None,
            action: "authentication".to_string(),
            result,
            details: HashMap::new(),
            risk_score: None,
        };

        self.log_event(event)
    }

    fn log_authorization(
        &self,
        subject: &str,
        resource: &str,
        action: &str,
        result: SecurityEventResult,
    ) -> Result<(), SecurityAuditError> {
        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::AuthorizationFailure,
            severity: match result {
                SecurityEventResult::Success => SecurityEventSeverity::Info,
                _ => SecurityEventSeverity::Warning,
            },
            subject: Some(subject.to_string()),
            source_ip: None,
            user_agent: None,
            resource: Some(resource.to_string()),
            action: action.to_string(),
            result,
            details: HashMap::new(),
            risk_score: None,
        };

        self.log_event(event)
    }
}

/// File-based security audit logger
pub struct FileSecurityAuditLogger {
    log_file_path: String,
}

impl FileSecurityAuditLogger {
    pub fn new(log_file_path: String) -> Self {
        Self { log_file_path }
    }
}

impl SecurityAuditLogger for FileSecurityAuditLogger {
    fn log_event(&self, event: SecurityAuditEvent) -> Result<(), SecurityAuditError> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let json_event = serde_json::to_string(&event)
            .map_err(|e| SecurityAuditError::SerializationError(e.to_string()))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .map_err(|e| SecurityAuditError::WriteError(e.to_string()))?;

        writeln!(file, "{}", json_event)
            .map_err(|e| SecurityAuditError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn log_jwt_validation(
        &self,
        token_id: Option<&str>,
        subject: Option<&str>,
        result: SecurityEventResult,
        details: HashMap<String, serde_json::Value>,
    ) -> Result<(), SecurityAuditError> {
        let event_type = match result {
            SecurityEventResult::Success => SecurityEventType::JwtValidationSuccess,
            _ => SecurityEventType::JwtValidationFailure,
        };

        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            timestamp: SystemTime::now(),
            event_type,
            severity: match result {
                SecurityEventResult::Success => SecurityEventSeverity::Info,
                SecurityEventResult::Failure => SecurityEventSeverity::Warning,
                SecurityEventResult::Blocked => SecurityEventSeverity::Error,
            },
            subject: subject.map(|s| s.to_string()),
            source_ip: None,
            user_agent: None,
            resource: token_id.map(|id| format!("jwt_token:{}", id)),
            action: "jwt_validation".to_string(),
            result,
            details,
            risk_score: None,
        };

        self.log_event(event)
    }

    fn log_authentication(
        &self,
        subject: &str,
        source_ip: Option<&str>,
        result: SecurityEventResult,
    ) -> Result<(), SecurityAuditError> {
        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::AuthenticationFailure,
            severity: match result {
                SecurityEventResult::Success => SecurityEventSeverity::Info,
                _ => SecurityEventSeverity::Warning,
            },
            subject: Some(subject.to_string()),
            source_ip: source_ip.map(|ip| ip.to_string()),
            user_agent: None,
            resource: None,
            action: "authentication".to_string(),
            result,
            details: HashMap::new(),
            risk_score: None,
        };

        self.log_event(event)
    }

    fn log_authorization(
        &self,
        subject: &str,
        resource: &str,
        action: &str,
        result: SecurityEventResult,
    ) -> Result<(), SecurityAuditError> {
        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::now_v7().to_string(),
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::AuthorizationFailure,
            severity: match result {
                SecurityEventResult::Success => SecurityEventSeverity::Info,
                _ => SecurityEventSeverity::Warning,
            },
            subject: Some(subject.to_string()),
            source_ip: None,
            user_agent: None,
            resource: Some(resource.to_string()),
            action: action.to_string(),
            result,
            details: HashMap::new(),
            risk_score: None,
        };

        self.log_event(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_audit_logger_creation() {
        let logger = InMemorySecurityAuditLogger::new();
        assert_eq!(logger.get_events().len(), 0);
    }

    #[test]
    fn test_jwt_validation_success_logging() {
        let logger = InMemorySecurityAuditLogger::new();
        let mut details = HashMap::new();
        details.insert(
            "algorithm".to_string(),
            serde_json::Value::String("HS256".to_string()),
        );

        let result = logger.log_jwt_validation(
            Some("token123"),
            Some("user123"),
            SecurityEventResult::Success,
            details,
        );

        assert!(result.is_ok());
        assert_eq!(logger.get_events().len(), 1);

        let event = &logger.get_events()[0];
        assert!(matches!(
            event.event_type,
            SecurityEventType::JwtValidationSuccess
        ));
        assert_eq!(event.subject, Some("user123".to_string()));
        assert_eq!(event.resource, Some("jwt_token:token123".to_string()));
        assert!(matches!(event.result, SecurityEventResult::Success));
    }

    #[test]
    fn test_jwt_validation_failure_logging() {
        let logger = InMemorySecurityAuditLogger::new();
        let details = HashMap::new();

        let result = logger.log_jwt_validation(
            Some("invalid_token"),
            None,
            SecurityEventResult::Failure,
            details,
        );

        assert!(result.is_ok());
        assert_eq!(logger.get_events().len(), 1);

        let event = &logger.get_events()[0];
        assert!(matches!(
            event.event_type,
            SecurityEventType::JwtValidationFailure
        ));
        assert_eq!(event.subject, None);
        assert!(matches!(event.result, SecurityEventResult::Failure));
        assert!(matches!(event.severity, SecurityEventSeverity::Warning));
    }

    #[test]
    fn test_authentication_logging() {
        let logger = InMemorySecurityAuditLogger::new();

        let result =
            logger.log_authentication("user123", Some("192.168.1.1"), SecurityEventResult::Failure);

        assert!(result.is_ok());
        assert_eq!(logger.get_events().len(), 1);

        let event = &logger.get_events()[0];
        assert!(matches!(
            event.event_type,
            SecurityEventType::AuthenticationFailure
        ));
        assert_eq!(event.subject, Some("user123".to_string()));
        assert_eq!(event.source_ip, Some("192.168.1.1".to_string()));
        assert!(matches!(event.result, SecurityEventResult::Failure));
    }

    #[test]
    fn test_authorization_logging() {
        let logger = InMemorySecurityAuditLogger::new();

        let result = logger.log_authorization(
            "user123",
            "/api/admin",
            "read",
            SecurityEventResult::Blocked,
        );

        assert!(result.is_ok());
        assert_eq!(logger.get_events().len(), 1);

        let event = &logger.get_events()[0];
        assert!(matches!(
            event.event_type,
            SecurityEventType::AuthorizationFailure
        ));
        assert_eq!(event.subject, Some("user123".to_string()));
        assert_eq!(event.resource, Some("/api/admin".to_string()));
        assert_eq!(event.action, "read");
        assert!(matches!(event.result, SecurityEventResult::Blocked));
    }

    #[test]
    fn test_event_counting_by_type() {
        let logger = InMemorySecurityAuditLogger::new();

        // Log multiple events of different types
        logger
            .log_jwt_validation(
                Some("token1"),
                Some("user1"),
                SecurityEventResult::Success,
                HashMap::new(),
            )
            .unwrap();
        logger
            .log_jwt_validation(
                Some("token2"),
                None,
                SecurityEventResult::Failure,
                HashMap::new(),
            )
            .unwrap();
        logger
            .log_authentication("user1", None, SecurityEventResult::Success)
            .unwrap();

        assert_eq!(
            logger.count_events_by_type(SecurityEventType::JwtValidationSuccess),
            1
        );
        assert_eq!(
            logger.count_events_by_type(SecurityEventType::JwtValidationFailure),
            1
        );
        assert_eq!(
            logger.count_events_by_type(SecurityEventType::AuthenticationFailure),
            1
        );
        assert_eq!(
            logger.count_events_by_type(SecurityEventType::AuthorizationFailure),
            0
        );
    }

    #[test]
    fn test_event_clearing() {
        let logger = InMemorySecurityAuditLogger::new();

        logger
            .log_jwt_validation(
                Some("token1"),
                Some("user1"),
                SecurityEventResult::Success,
                HashMap::new(),
            )
            .unwrap();
        assert_eq!(logger.get_events().len(), 1);

        logger.clear_events();
        assert_eq!(logger.get_events().len(), 0);
    }

    #[test]
    fn test_event_serialization() {
        let event = SecurityAuditEvent {
            event_id: "test-id".to_string(),
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::JwtValidationSuccess,
            severity: SecurityEventSeverity::Info,
            subject: Some("user123".to_string()),
            source_ip: Some("192.168.1.1".to_string()),
            user_agent: Some("TestAgent/1.0".to_string()),
            resource: Some("jwt_token:token123".to_string()),
            action: "jwt_validation".to_string(),
            result: SecurityEventResult::Success,
            details: HashMap::new(),
            risk_score: Some(10),
        };

        let json = serde_json::to_string(&event);
        assert!(json.is_ok());

        let deserialized: Result<SecurityAuditEvent, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
