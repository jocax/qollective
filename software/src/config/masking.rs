// ABOUTME: Field masking configuration and implementation for protecting sensitive data
// ABOUTME: Provides configurable masking rules for PII, tokens, and other sensitive fields

//! Field masking system for protecting sensitive data in logs and debugging output.
//!
//! This module provides a configurable field masking system that can protect PII
//! and sensitive data when serializing envelopes to logs, debug output, or other
//! non-secure contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for field masking behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskingConfig {
    /// Whether masking is enabled
    pub enabled: bool,
    /// The masking level to apply
    pub level: MaskingLevel,
    /// Custom masking rules that override defaults
    pub custom_rules: Vec<MaskingRule>,
    /// Whether to audit when sensitive fields are accessed
    pub audit_access: bool,
    /// Custom patterns for field path matching
    pub field_patterns: HashMap<String, MaskType>,
}

/// Predefined masking levels with increasing strictness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaskingLevel {
    /// No masking applied (development only)
    None,
    /// Mask only critical secrets (tokens, keys)
    Minimal,
    /// Mask PII + tokens + keys (recommended for production)
    Standard,
    /// Mask all potentially sensitive data (high security)
    Strict,
    /// Use only custom rules, ignore defaults
    Custom,
}

/// Individual masking rule for specific field paths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskingRule {
    /// Field path pattern (e.g., "security.userId", "jwt.claims.*")
    pub field_path: String,
    /// Type of masking to apply
    pub mask_type: MaskType,
    /// Optional condition for when to apply this rule
    pub condition: Option<String>,
    /// Priority for rule resolution (higher wins)
    pub priority: u8,
}

/// Types of masking that can be applied to sensitive fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaskType {
    /// Replace entire value with "***MASKED***"
    Full,
    /// Partially mask value (e.g., "user***@***domain.com")
    Partial,
    /// Replace with SHA256 hash for consistency across logs
    Hash,
    /// Replace with "[REDACTED]" for security auditing
    Redact,
    /// Show only first N characters
    Prefix(usize),
    /// Show only last N characters  
    Suffix(usize),
    /// Custom format string with placeholders
    Custom(String),
}

/// Field masker that applies masking rules to values
#[derive(Debug, Clone)]
pub struct FieldMasker {
    config: MaskingConfig,
    compiled_rules: Vec<CompiledRule>,
}

/// Internal compiled version of masking rule for performance
#[derive(Debug, Clone)]
struct CompiledRule {
    pattern: glob::Pattern,
    mask_type: MaskType,
    priority: u8,
}

/// Trait for types that can have their fields masked
pub trait Maskable {
    /// Create a new instance with sensitive fields masked
    fn mask_fields(&self, masker: &FieldMasker) -> Self;

    /// Generate a string representation with sensitive fields masked
    fn to_masked_string(&self, masker: &FieldMasker) -> String;
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: MaskingLevel::Standard,
            custom_rules: Vec::new(),
            audit_access: false,
            field_patterns: HashMap::new(),
        }
    }
}

impl MaskingConfig {
    /// Create a new masking configuration with specified level
    pub fn new(level: MaskingLevel) -> Self {
        Self {
            enabled: true,
            level,
            custom_rules: Vec::new(),
            audit_access: false,
            field_patterns: HashMap::new(),
        }
    }

    /// Disable masking entirely
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            level: MaskingLevel::None,
            custom_rules: Vec::new(),
            audit_access: false,
            field_patterns: HashMap::new(),
        }
    }

    /// Enable audit logging for sensitive field access
    pub fn with_audit(mut self) -> Self {
        self.audit_access = true;
        self
    }

    /// Add custom masking rule
    pub fn with_rule(mut self, rule: MaskingRule) -> Self {
        self.custom_rules.push(rule);
        self
    }

    /// Add multiple custom rules
    pub fn with_rules(mut self, rules: Vec<MaskingRule>) -> Self {
        self.custom_rules.extend(rules);
        self
    }
}

impl MaskingRule {
    /// Create a new masking rule
    pub fn new(field_path: impl Into<String>, mask_type: MaskType) -> Self {
        Self {
            field_path: field_path.into(),
            mask_type,
            condition: None,
            priority: 50, // Default priority
        }
    }

    /// Set priority for rule resolution
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add condition for when rule applies
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

impl FieldMasker {
    /// Create a new field masker with given configuration
    pub fn new(config: MaskingConfig) -> Result<Self, MaskingError> {
        if !config.enabled {
            return Ok(Self {
                config,
                compiled_rules: Vec::new(),
            });
        }

        let mut rules = Self::build_default_rules(&config.level);
        rules.extend(Self::compile_custom_rules(&config.custom_rules)?);

        // Sort by priority (highest first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(Self {
            config,
            compiled_rules: rules,
        })
    }

    /// Check if masking is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Apply masking to a field value based on its path
    pub fn mask_value(&self, field_path: &str, value: &str) -> String {
        if !self.config.enabled {
            return value.to_string();
        }

        // Find first matching rule by priority
        for rule in &self.compiled_rules {
            if rule.pattern.matches(field_path) {
                if self.config.audit_access {
                    tracing::info!("Masking sensitive field: {}", field_path);
                }
                return self.apply_mask_type(&rule.mask_type, value);
            }
        }

        // No rule matched, return original value
        value.to_string()
    }

    /// Check if a field should be masked
    pub fn should_mask(&self, field_path: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.compiled_rules
            .iter()
            .any(|rule| rule.pattern.matches(field_path))
    }

    /// Get the masking level
    pub fn level(&self) -> &MaskingLevel {
        &self.config.level
    }

    // Private implementation methods
    fn build_default_rules(level: &MaskingLevel) -> Vec<CompiledRule> {
        match level {
            MaskingLevel::None => Vec::new(),
            MaskingLevel::Minimal => Self::minimal_rules(),
            MaskingLevel::Standard => Self::standard_rules(),
            MaskingLevel::Strict => Self::strict_rules(),
            MaskingLevel::Custom => Vec::new(), // Only use custom rules
        }
    }

    fn minimal_rules() -> Vec<CompiledRule> {
        vec![
            // Critical secrets only
            Self::compile_rule("jwt.token", MaskType::Redact, 100),
            Self::compile_rule("*.token", MaskType::Redact, 90),
            Self::compile_rule("*.apiKey", MaskType::Redact, 90),
            Self::compile_rule("*.password", MaskType::Full, 90),
        ]
    }

    fn standard_rules() -> Vec<CompiledRule> {
        let mut rules = Self::minimal_rules();

        // Add PII protection
        rules.extend(vec![
            // Security metadata
            Self::compile_rule("security.userId", MaskType::Hash, 80),
            Self::compile_rule("security.sessionId", MaskType::Hash, 80),
            Self::compile_rule("security.ipAddress", MaskType::Partial, 80),
            // JWT claims
            Self::compile_rule("jwt.claims.sub", MaskType::Hash, 80),
            Self::compile_rule("jwt.claims.email", MaskType::Partial, 80),
            Self::compile_rule("jwt.claims.phone", MaskType::Partial, 80),
            // Tenant information
            Self::compile_rule("onBehalfOf.originalUserId", MaskType::Hash, 80),
            Self::compile_rule("onBehalfOf.delegatedBy", MaskType::Hash, 80),
            // Common sensitive extension patterns
            Self::compile_rule("extensions.*.email", MaskType::Partial, 70),
            Self::compile_rule("extensions.*.userId", MaskType::Hash, 70),
            Self::compile_rule("extensions.*.sessionId", MaskType::Hash, 70),
        ]);

        rules
    }

    fn strict_rules() -> Vec<CompiledRule> {
        let mut rules = Self::standard_rules();

        // Add comprehensive masking
        rules.extend(vec![
            // All security metadata
            Self::compile_rule("security.*", MaskType::Hash, 60),
            // All JWT claims
            Self::compile_rule("jwt.claims.*", MaskType::Hash, 60),
            // All onBehalfOf data
            Self::compile_rule("onBehalfOf.*", MaskType::Redact, 60),
            // All extensions (except explicitly safe ones)
            Self::compile_rule("extensions.*", MaskType::Redact, 50),
            // Performance data that might contain sensitive info
            Self::compile_rule("performance.query", MaskType::Redact, 40),
            Self::compile_rule("performance.response", MaskType::Redact, 40),
            // Tracing baggage
            Self::compile_rule("tracing.baggage.*", MaskType::Hash, 40),
        ]);

        rules
    }

    fn compile_custom_rules(rules: &[MaskingRule]) -> Result<Vec<CompiledRule>, MaskingError> {
        rules
            .iter()
            .map(|rule| {
                let pattern = glob::Pattern::new(&rule.field_path).map_err(|e| {
                    MaskingError::InvalidPattern {
                        pattern: rule.field_path.clone(),
                        error: e.to_string(),
                    }
                })?;

                Ok(CompiledRule {
                    pattern,
                    mask_type: rule.mask_type.clone(),
                    priority: rule.priority,
                })
            })
            .collect()
    }

    fn compile_rule(pattern: &str, mask_type: MaskType, priority: u8) -> CompiledRule {
        CompiledRule {
            pattern: glob::Pattern::new(pattern).expect("Valid pattern"),
            mask_type,
            priority,
        }
    }

    fn apply_mask_type(&self, mask_type: &MaskType, value: &str) -> String {
        match mask_type {
            MaskType::Full => "***MASKED***".to_string(),
            MaskType::Redact => "[REDACTED]".to_string(),
            MaskType::Hash => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                format!("sha256:{:x}", hasher.finalize())
            }
            MaskType::Partial => {
                if value.contains('@') {
                    // Email masking
                    if let Some(at_pos) = value.find('@') {
                        let (local, domain) = value.split_at(at_pos);
                        if local.len() > 2 && domain.len() > 4 {
                            format!("{}***@***{}", &local[..2], &domain[domain.len() - 4..])
                        } else {
                            "***@***".to_string()
                        }
                    } else {
                        "***@***".to_string()
                    }
                } else if value.len() > 6 {
                    // General partial masking
                    format!("{}***{}", &value[..2], &value[value.len() - 2..])
                } else {
                    "***".to_string()
                }
            }
            MaskType::Prefix(count) => {
                if value.len() > *count {
                    format!("{}***", &value[..*count])
                } else {
                    "***".to_string()
                }
            }
            MaskType::Suffix(count) => {
                if value.len() > *count {
                    format!("***{}", &value[value.len() - count..])
                } else {
                    "***".to_string()
                }
            }
            MaskType::Custom(format) => {
                // Simple placeholder replacement for custom formats
                format
                    .replace("{masked}", "***")
                    .replace("{redacted}", "[REDACTED]")
                    .replace("{hash}", &self.apply_mask_type(&MaskType::Hash, value))
            }
        }
    }
}

/// Errors that can occur during masking operations
#[derive(Debug, Clone, PartialEq)]
pub enum MaskingError {
    /// Invalid field path pattern
    InvalidPattern { pattern: String, error: String },
    /// Configuration validation error
    InvalidConfig { message: String },
}

impl std::fmt::Display for MaskingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaskingError::InvalidPattern { pattern, error } => {
                write!(f, "Invalid field pattern '{}': {}", pattern, error)
            }
            MaskingError::InvalidConfig { message } => {
                write!(f, "Invalid masking configuration: {}", message)
            }
        }
    }
}

impl std::error::Error for MaskingError {}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests will all fail initially - this is intentional for TDD!

    #[test]
    fn test_masking_config_default() {
        let config = MaskingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.level, MaskingLevel::Standard);
        assert!(config.custom_rules.is_empty());
        assert!(!config.audit_access);
    }

    #[test]
    fn test_masking_config_disabled() {
        let config = MaskingConfig::disabled();
        assert!(!config.enabled);
        assert_eq!(config.level, MaskingLevel::None);
    }

    #[test]
    fn test_masking_config_with_audit() {
        let config = MaskingConfig::default().with_audit();
        assert!(config.audit_access);
    }

    #[test]
    fn test_masking_config_with_rule() {
        let rule = MaskingRule::new("test.field", MaskType::Full);
        let config = MaskingConfig::default().with_rule(rule.clone());
        assert_eq!(config.custom_rules.len(), 1);
        assert_eq!(config.custom_rules[0], rule);
    }

    #[test]
    fn test_masking_rule_creation() {
        let rule = MaskingRule::new("security.userId", MaskType::Hash);
        assert_eq!(rule.field_path, "security.userId");
        assert_eq!(rule.mask_type, MaskType::Hash);
        assert_eq!(rule.priority, 50);
        assert!(rule.condition.is_none());
    }

    #[test]
    fn test_masking_rule_with_priority() {
        let rule = MaskingRule::new("test.field", MaskType::Full).with_priority(100);
        assert_eq!(rule.priority, 100);
    }

    #[test]
    fn test_masking_rule_with_condition() {
        let rule =
            MaskingRule::new("test.field", MaskType::Full).with_condition("environment=production");
        assert_eq!(rule.condition, Some("environment=production".to_string()));
    }

    #[test]
    fn test_field_masker_disabled() {
        let config = MaskingConfig::disabled();
        let masker = FieldMasker::new(config).unwrap();
        assert!(!masker.is_enabled());

        let result = masker.mask_value("security.userId", "user123");
        assert_eq!(result, "user123"); // Should not be masked when disabled
    }

    #[test]
    fn test_field_masker_minimal_level() {
        let config = MaskingConfig::new(MaskingLevel::Minimal);
        let masker = FieldMasker::new(config).unwrap();
        assert!(masker.is_enabled());
        assert_eq!(masker.level(), &MaskingLevel::Minimal);

        // Should mask tokens but not PII
        assert_eq!(masker.mask_value("jwt.token", "eyJ0eXAi"), "[REDACTED]");
        assert_eq!(masker.mask_value("security.userId", "user123"), "user123");
    }

    #[test]
    fn test_field_masker_standard_level() {
        let config = MaskingConfig::new(MaskingLevel::Standard);
        let masker = FieldMasker::new(config).unwrap();

        // Should mask both tokens and PII
        assert_eq!(masker.mask_value("jwt.token", "eyJ0eXAi"), "[REDACTED]");
        assert!(masker
            .mask_value("security.userId", "user123")
            .starts_with("sha256:"));
        assert_eq!(
            masker.mask_value("security.ipAddress", "192.168.1.100"),
            "19***00"
        );
    }

    #[test]
    fn test_field_masker_strict_level() {
        let config = MaskingConfig::new(MaskingLevel::Strict);
        let masker = FieldMasker::new(config).unwrap();

        // Should mask everything aggressively
        assert_eq!(masker.mask_value("jwt.token", "eyJ0eXAi"), "[REDACTED]");
        assert!(masker
            .mask_value("security.userId", "user123")
            .starts_with("sha256:"));
        assert_eq!(
            masker.mask_value("extensions.customField", "value"),
            "[REDACTED]"
        );
    }

    #[test]
    fn test_mask_type_full() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Full, "sensitive_data");
        assert_eq!(result, "***MASKED***");
    }

    #[test]
    fn test_mask_type_redact() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Redact, "secret_token");
        assert_eq!(result, "[REDACTED]");
    }

    #[test]
    fn test_mask_type_hash() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Hash, "user123");
        assert!(result.starts_with("sha256:"));
        assert_eq!(result.len(), 71); // "sha256:" + 64 chars

        // Same input should produce same hash
        let result2 = masker.apply_mask_type(&MaskType::Hash, "user123");
        assert_eq!(result, result2);
    }

    #[test]
    fn test_mask_type_partial_email() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Partial, "user@example.com");
        assert_eq!(result, "us***@***.com");
    }

    #[test]
    fn test_mask_type_partial_general() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Partial, "sensitive123");
        assert_eq!(result, "se***23");
    }

    #[test]
    fn test_mask_type_prefix() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Prefix(3), "abcdefgh");
        assert_eq!(result, "abc***");
    }

    #[test]
    fn test_mask_type_suffix() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let result = masker.apply_mask_type(&MaskType::Suffix(3), "abcdefgh");
        assert_eq!(result, "***fgh");
    }

    #[test]
    fn test_mask_type_custom_format() {
        let config = MaskingConfig::new(MaskingLevel::Custom);
        let masker = FieldMasker::new(config).unwrap();
        let format = "Value: {masked}".to_string();
        let result = masker.apply_mask_type(&MaskType::Custom(format), "secret");
        assert_eq!(result, "Value: ***");
    }

    #[test]
    fn test_should_mask_patterns() {
        let config = MaskingConfig::new(MaskingLevel::Standard);
        let masker = FieldMasker::new(config).unwrap();

        // Should match standard patterns
        assert!(masker.should_mask("security.userId"));
        assert!(masker.should_mask("jwt.token"));
        assert!(masker.should_mask("jwt.claims.email"));
        assert!(masker.should_mask("extensions.custom.email"));

        // Should not match non-sensitive patterns
        assert!(!masker.should_mask("data.publicField"));
        assert!(!masker.should_mask("meta.timestamp"));
        assert!(!masker.should_mask("performance.duration"));
    }

    #[test]
    fn test_custom_rules_override_defaults() {
        let custom_rule = MaskingRule::new("security.userId", MaskType::Full).with_priority(200); // Higher priority than defaults

        let config = MaskingConfig::new(MaskingLevel::Standard).with_rule(custom_rule);

        let masker = FieldMasker::new(config).unwrap();

        // Custom rule should override default (hash -> full)
        assert_eq!(
            masker.mask_value("security.userId", "user123"),
            "***MASKED***"
        );
    }

    #[test]
    fn test_rule_priority_resolution() {
        let high_priority = MaskingRule::new("test.*", MaskType::Full).with_priority(100);
        let low_priority = MaskingRule::new("test.field", MaskType::Hash).with_priority(50);

        let config =
            MaskingConfig::new(MaskingLevel::Custom).with_rules(vec![low_priority, high_priority]); // Order shouldn't matter

        let masker = FieldMasker::new(config).unwrap();

        // Higher priority rule should win
        assert_eq!(masker.mask_value("test.field", "value"), "***MASKED***");
    }

    #[test]
    fn test_invalid_pattern_error() {
        let invalid_rule = MaskingRule::new("[invalid_glob_pattern", MaskType::Full);
        let config = MaskingConfig::new(MaskingLevel::Custom).with_rule(invalid_rule);

        let result = FieldMasker::new(config);
        assert!(result.is_err());

        match result.unwrap_err() {
            MaskingError::InvalidPattern { pattern, .. } => {
                assert_eq!(pattern, "[invalid_glob_pattern");
            }
            _ => panic!("Expected InvalidPattern error"),
        }
    }

    #[test]
    fn test_audit_logging() {
        let config = MaskingConfig::new(MaskingLevel::Standard).with_audit();

        let masker = FieldMasker::new(config).unwrap();

        // This test will need to capture tracing output to verify audit logging
        // For now, just verify the masking still works
        let result = masker.mask_value("security.userId", "user123");
        assert!(result.starts_with("sha256:"));
    }

    #[test]
    fn test_nested_field_patterns() {
        let config = MaskingConfig::new(MaskingLevel::Standard);
        let masker = FieldMasker::new(config).unwrap();

        // Test deep nesting patterns
        assert!(masker.should_mask("extensions.auth.userId"));
        assert!(masker.should_mask("extensions.billing.email"));
        assert!(masker.should_mask("jwt.claims.sub"));
        assert!(masker.should_mask("onBehalfOf.originalUserId"));
    }

    #[test]
    fn test_wildcard_patterns() {
        let config = MaskingConfig::new(MaskingLevel::Strict);
        let masker = FieldMasker::new(config).unwrap();

        // Test wildcard matching
        assert!(masker.should_mask("security.anything"));
        assert!(masker.should_mask("jwt.claims.whatever"));
        assert!(masker.should_mask("extensions.any.thing"));
    }

    #[test]
    fn test_performance_minimal_overhead() {
        let config = MaskingConfig::disabled();
        let masker = FieldMasker::new(config).unwrap();

        // Disabled masker should have minimal overhead
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            masker.mask_value("security.userId", "user123");
        }
        let duration = start.elapsed();

        // Should complete very quickly when disabled
        assert!(duration.as_millis() < 10);
    }
}
