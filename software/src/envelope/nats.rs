// ABOUTME: NATS-specific envelope functionality and subject pattern management
// ABOUTME: Provides shared subject pattern generation for HTTP-style tenant routing

//! NATS-specific envelope functionality.
//!
//! 🔴 CRITICAL: This module implements shared subject pattern (HTTP-style shared subjects) 🔴
//!
//! **TENANT ISOLATION STRATEGY:**
//! - Shared NATS subjects: `qollective.{service}.{operation}.{version}`
//! - NO tenant-scoped subjects (that would be tenant-scoped pattern - FORBIDDEN)
//! - Tenant context via envelope.meta.tenant only
//! - Application-level tenant filtering and routing

use crate::error::{QollectiveError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// NATS subject pattern for shared message routing (shared subject pattern).
///
/// **🚨 CRITICAL ARCHITECTURE DECISION: Shared Subject Pattern 🚨**
///
/// This implements HTTP-style shared subjects without tenant scoping:
/// - `qollective.{service}.{operation}.{version}`
/// - Tenant context passed via envelope metadata only
/// - Supports millions of tenants with bounded subject space
///
/// **FORBIDDEN:** Tenant-scoped subjects like `qollective.{tenant}.service.operation.version`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubjectPattern {
    service: String,
    operation: String,
    version: String,
}

impl SubjectPattern {
    /// Create a new shared subject pattern (shared subject architecture).
    ///
    /// **TENANT ISOLATION:** Tenants are isolated at the application level via
    /// envelope.meta.tenant, NOT via subject scoping.
    ///
    /// # Examples
    ///
    /// ```
    /// use qollective::envelope::nats::SubjectPattern;
    ///
    /// let pattern = SubjectPattern::new("user-service", "create", "v1");
    /// assert_eq!(pattern.to_string(), "qollective.user-service.create.v1");
    /// // All tenants use this same subject - tenant context in envelope!
    /// ```
    pub fn new(service: &str, operation: &str, version: &str) -> Self {
        Self {
            service: service.to_string(),
            operation: operation.to_string(),
            version: version.to_string(),
        }
    }

    /// Try to create a new subject pattern with validation.
    ///
    /// Returns an error if any component is invalid (empty or contains spaces).
    ///
    /// **TENANT ISOLATION:** This validates shared subject components only.
    /// Tenant context is handled at the application level.
    pub fn try_new(service: &str, operation: &str, version: &str) -> Result<Self> {
        let pattern = Self::new(service, operation, version);
        if pattern.is_valid() {
            Ok(pattern)
        } else {
            Err(QollectiveError::nats_subject(format!(
                "Invalid subject pattern components: service='{}', operation='{}', version='{}'",
                service, operation, version
            )))
        }
    }

    /// Parse a shared subject string into a SubjectPattern (shared subject architecture).
    ///
    /// **IMPORTANT:** This only parses shared subjects. Tenant context
    /// must be extracted from the envelope, not the subject.
    ///
    /// # Examples
    ///
    /// ```
    /// use qollective::envelope::nats::SubjectPattern;
    ///
    /// let pattern = SubjectPattern::parse("qollective.user-service.create.v1").unwrap();
    /// assert_eq!(pattern.service(), "user-service");
    /// assert_eq!(pattern.operation(), "create");
    /// assert_eq!(pattern.version(), "v1");
    /// // Tenant context comes from envelope.meta.tenant, not subject!
    /// ```
    pub fn parse(subject: &str) -> Result<Self> {
        let parts: Vec<&str> = subject.split('.').collect();

        // Validate shared subject structure: qollective.service.operation.version
        if parts.len() != 4 || parts[0] != "qollective" {
            return Err(QollectiveError::nats_subject(format!(
                "Invalid shared subject format: '{}'. Expected: qollective.service.operation.version",
                subject
            )));
        }

        // Check for empty components
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                return Err(QollectiveError::nats_subject(format!(
                    "Empty component at position {} in subject: '{}'",
                    i, subject
                )));
            }
        }

        // Extract components (no tenant in subject for shared subject pattern)
        Ok(Self {
            service: parts[1].to_string(),
            operation: parts[2].to_string(),
            version: parts[3].to_string(),
        })
    }

    /// Create a builder for complex subject patterns.
    pub fn builder() -> SubjectPatternBuilder {
        SubjectPatternBuilder::new()
    }

    /// Validate that all components are non-empty and contain no spaces.
    ///
    /// **Shared Subject Validation:** Only validates shared subject components.
    /// Tenant validation happens at the application level.
    pub fn is_valid(&self) -> bool {
        !self.service.is_empty()
            && !self.operation.is_empty()
            && !self.version.is_empty()
            && !self.service.contains(' ')
            && !self.operation.contains(' ')
            && !self.version.contains(' ')
    }

    /// Get the service component.
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Get the operation component.
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Get the version component.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Generate a wildcard pattern for all services (shared subject pattern).
    ///
    /// **TENANT ISOLATION:** This creates shared wildcards. Applications
    /// must filter by tenant context from envelope metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use qollective::envelope::nats::SubjectPattern;
    ///
    /// let pattern = SubjectPattern::new("user-service", "create", "v1");
    /// assert_eq!(pattern.service_wildcard(), "qollective.*.create.v1");
    /// // Matches all services for this operation - tenant filtering in app layer
    /// ```
    pub fn service_wildcard(&self) -> String {
        format!("qollective.*.{}.{}", self.operation, self.version)
    }

    /// Generate a wildcard pattern for all operations (shared subject pattern).
    pub fn operation_wildcard(&self) -> String {
        format!("qollective.{}.*.{}", self.service, self.version)
    }

    /// Generate a wildcard pattern for all versions (shared subject pattern).
    pub fn version_wildcard(&self) -> String {
        format!("qollective.{}.{}.*", self.service, self.operation)
    }

    /// Generate a wildcard pattern for all Qollective messages.
    ///
    /// **TENANT ISOLATION:** This wildcard covers all tenants. Applications
    /// must filter by envelope.meta.tenant.
    pub fn all_wildcard(&self) -> String {
        "qollective.>".to_string()
    }

    /// Check if this pattern matches a given subject (exact match).
    ///
    /// **TENANT ISOLATION:** This only matches the shared subject.
    /// Tenant matching happens at the application level.
    ///
    /// # Examples
    ///
    /// ```
    /// use qollective::envelope::nats::SubjectPattern;
    ///
    /// let pattern = SubjectPattern::new("user-service", "create", "v1");
    /// assert!(pattern.matches("qollective.user-service.create.v1"));
    /// assert!(!pattern.matches("qollective.user-service.create.v2"));
    /// // Tenant context handled in application, not subject matching
    /// ```
    pub fn matches(&self, subject: &str) -> bool {
        subject == self.to_string()
    }
}

impl fmt::Display for SubjectPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Shared subject format: qollective.service.operation.version (NO tenant)
        write!(
            f,
            "qollective.{}.{}.{}",
            self.service, self.operation, self.version
        )
    }
}

/// Builder for creating subject patterns (shared subject pattern only).
///
/// **TENANT ISOLATION:** This builder creates shared subjects only.
/// Tenant context is managed at the application level.
#[derive(Debug, Default)]
pub struct SubjectPatternBuilder {
    service: Option<String>,
    operation: Option<String>,
    version: Option<String>,
}

impl SubjectPatternBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the service component.
    pub fn service(mut self, service: &str) -> Self {
        self.service = Some(service.to_string());
        self
    }

    /// Set the operation component.
    pub fn operation(mut self, operation: &str) -> Self {
        self.operation = Some(operation.to_string());
        self
    }

    /// Set the version component.
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Build the shared subject pattern (shared subject architecture).
    ///
    /// **TENANT ISOLATION:** This creates shared subjects only.
    /// Tenant context is passed via envelope metadata.
    pub fn build(self) -> Result<SubjectPattern> {
        let service = self.service.ok_or_else(|| {
            QollectiveError::nats_subject("Service is required for subject pattern")
        })?;
        let operation = self.operation.ok_or_else(|| {
            QollectiveError::nats_subject("Operation is required for subject pattern")
        })?;
        let version = self.version.ok_or_else(|| {
            QollectiveError::nats_subject("Version is required for subject pattern")
        })?;

        let pattern = SubjectPattern {
            service,
            operation,
            version,
        };

        if pattern.is_valid() {
            Ok(pattern)
        } else {
            Err(QollectiveError::nats_subject(
                "Invalid subject pattern components",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_subject_pattern() {
        let pattern = SubjectPattern::new("user-service", "create", "v1");
        assert_eq!(pattern.to_string(), "qollective.user-service.create.v1");
        assert_eq!(pattern.service(), "user-service");
        assert_eq!(pattern.operation(), "create");
        assert_eq!(pattern.version(), "v1");
    }

    #[test]
    fn test_subject_pattern_validation() {
        assert!(SubjectPattern::new("service", "operation", "v1").is_valid());
        assert!(!SubjectPattern::new("", "operation", "v1").is_valid());
        assert!(!SubjectPattern::new("service", "", "v1").is_valid());
        assert!(!SubjectPattern::new("service", "operation", "").is_valid());
        assert!(!SubjectPattern::new("ser vice", "operation", "v1").is_valid());
    }

    #[test]
    fn test_shared_subject_parsing() {
        let pattern = SubjectPattern::parse("qollective.user-service.create.v1").unwrap();
        assert_eq!(pattern.service(), "user-service");
        assert_eq!(pattern.operation(), "create");
        assert_eq!(pattern.version(), "v1");
    }

    #[test]
    fn test_invalid_subject_parsing() {
        assert!(SubjectPattern::parse("user-service.create.v1").is_err());
        assert!(SubjectPattern::parse("qollective.user-service.create").is_err());
        assert!(SubjectPattern::parse("qollective..create.v1").is_err());
        assert!(SubjectPattern::parse("").is_err());

        // Tenant-scoped patterns should be rejected
        assert!(SubjectPattern::parse("qollective.tenant123.user-service.create.v1").is_err());
        assert!(SubjectPattern::parse("tenant123.qollective.user-service.create.v1").is_err());
    }

    #[test]
    fn test_shared_subject_wildcard_patterns() {
        let pattern = SubjectPattern::new("user-service", "create", "v1");
        assert_eq!(pattern.service_wildcard(), "qollective.*.create.v1");
        assert_eq!(pattern.operation_wildcard(), "qollective.user-service.*.v1");
        assert_eq!(
            pattern.version_wildcard(),
            "qollective.user-service.create.*"
        );
        assert_eq!(pattern.all_wildcard(), "qollective.>");
    }

    #[test]
    fn test_pattern_matching() {
        let pattern = SubjectPattern::new("user-service", "create", "v1");
        assert!(pattern.matches("qollective.user-service.create.v1"));
        assert!(!pattern.matches("qollective.user-service.create.v2"));
        assert!(!pattern.matches("qollective.other-service.create.v1"));
    }

    #[test]
    fn test_subject_pattern_builder() {
        let pattern = SubjectPattern::builder()
            .service("payment-service")
            .operation("process")
            .version("v3")
            .build()
            .unwrap();

        assert_eq!(pattern.to_string(), "qollective.payment-service.process.v3");
        assert_eq!(pattern.service(), "payment-service");
        assert_eq!(pattern.operation(), "process");
        assert_eq!(pattern.version(), "v3");
    }

    #[test]
    fn test_builder_validation() {
        let result = SubjectPattern::builder()
            .service("service")
            .operation("operation")
            .build();
        assert!(result.is_err()); // Missing version

        let result = SubjectPattern::builder()
            .service("service")
            .version("v1")
            .build();
        assert!(result.is_err()); // Missing operation
    }

    #[test]
    fn test_serialization() {
        let pattern = SubjectPattern::new("auth-service", "validate", "v1");

        // Serialize and deserialize
        let serialized = serde_json::to_string(&pattern).expect("Should serialize");
        let deserialized: SubjectPattern =
            serde_json::from_str(&serialized).expect("Should deserialize");

        // Should maintain equality
        assert_eq!(pattern, deserialized);
        assert_eq!(pattern.to_string(), deserialized.to_string());
    }
}
