// ABOUTME: TLS Configuration Analysis - Q Console Challenge diagnostics
// ABOUTME: Comprehensive analysis of TLS path resolution and certificate loading issues

//! TLS Configuration Analysis Module
//! 
//! This module provides comprehensive analysis and testing of TLS configuration
//! issues in the Q Console Challenge example, focusing on certificate path
//! resolution and NATS TLS connection behavior.

use std::path::{Path, PathBuf};
use std::env;
use std::fs;

use qollective::{
    constants::network::tls_paths,
    config::tls::TlsConfig,
    error::{QollectiveError, Result},
};

use crate::config::{EnterpriseConfig, TlsExampleConfig};

/// TLS Analysis Report containing all discovered issues
#[derive(Debug, Clone)]
pub struct TlsAnalysisReport {
    /// Issues discovered during analysis
    pub issues: Vec<TlsIssue>,
    /// Current certificate path resolution results
    pub path_resolution: PathResolutionAnalysis,
    /// Configuration analysis results
    pub config_analysis: ConfigAnalysis,
    /// NATS connection analysis
    pub nats_analysis: NatsConnectionAnalysis,
}

/// Individual TLS issue discovered during analysis
#[derive(Debug, Clone)]
pub struct TlsIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Human-readable description
    pub description: String,
    /// Detailed explanation of the problem
    pub details: String,
    /// Suggested resolution
    pub resolution: String,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Critical, // Prevents functionality
    High,     // Major impact on functionality
    Medium,   // Moderate impact
    Low,      // Minor issue
}

/// Issue categories
#[derive(Debug, Clone, PartialEq)]
pub enum IssueCategory {
    PathResolution,
    CertificateAccess,
    Configuration,
    NatsConnection,
    SecurityModel,
}

/// Path resolution analysis results
#[derive(Debug, Clone)]
pub struct PathResolutionAnalysis {
    /// Current working directory
    pub current_dir: PathBuf,
    /// CARGO_MANIFEST_DIR value
    pub cargo_manifest_dir: Option<PathBuf>,
    /// Resolved TLS certificate base path
    pub resolved_base_path: PathBuf,
    /// Expected certificate paths
    pub expected_paths: CertificatePaths,
    /// Actual certificate file existence
    pub file_existence: CertificateExistence,
}

/// Certificate file paths
#[derive(Debug, Clone)]
pub struct CertificatePaths {
    pub ca_path: PathBuf,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

/// Certificate file existence status
#[derive(Debug, Clone)]
pub struct CertificateExistence {
    pub ca_exists: bool,
    pub cert_exists: bool,
    pub key_exists: bool,
}

/// Configuration analysis results
#[derive(Debug, Clone)]
pub struct ConfigAnalysis {
    /// Whether TLS is enabled in config
    pub tls_enabled: bool,
    /// Configured certificate paths from config.toml
    pub config_paths: CertificatePaths,
    /// Configuration conversion results
    pub framework_config: Result<TlsConfig>,
}

/// NATS connection analysis results
#[derive(Debug, Clone)]
pub struct NatsConnectionAnalysis {
    /// NATS URLs from configuration
    pub nats_urls: Vec<String>,
    /// Whether NATS is configured for TLS
    pub expects_tls: bool,
    /// Port analysis (TLS port vs non-TLS port)
    pub port_analysis: NatsPortAnalysis,
}

/// NATS port analysis
#[derive(Debug, Clone)]
pub struct NatsPortAnalysis {
    /// Configured ports
    pub configured_ports: Vec<u16>,
    /// Whether any ports are TLS ports (4443)
    pub has_tls_ports: bool,
    /// Whether any ports are non-TLS ports (4222)
    pub has_non_tls_ports: bool,
}

impl TlsAnalysisReport {
    /// Perform comprehensive TLS analysis
    pub fn analyze() -> Self {
        let mut issues = Vec::new();
        
        // Analyze path resolution
        let path_resolution = Self::analyze_path_resolution(&mut issues);
        
        // Analyze configuration
        let config_analysis = Self::analyze_configuration(&mut issues);
        
        // Analyze NATS connection
        let nats_analysis = Self::analyze_nats_connection(&mut issues);
        
        Self {
            issues,
            path_resolution,
            config_analysis,
            nats_analysis,
        }
    }
    
    /// Analyze TLS certificate path resolution
    fn analyze_path_resolution(issues: &mut Vec<TlsIssue>) -> PathResolutionAnalysis {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").ok().map(PathBuf::from);
        
        // Get resolved base path using framework logic
        let resolved_base_path_str = tls_paths::resolve_tls_cert_base_path();  
        let resolved_base_path = PathBuf::from(&resolved_base_path_str);
        
        // Build expected certificate paths using framework functions
        let expected_paths = CertificatePaths {
            ca_path: PathBuf::from(tls_paths::ca_file_path(&resolved_base_path_str)),
            cert_path: PathBuf::from(tls_paths::cert_file_path(&resolved_base_path_str)),
            key_path: PathBuf::from(tls_paths::key_file_path(&resolved_base_path_str)),
        };
        
        // Check file existence
        let file_existence = CertificateExistence {
            ca_exists: expected_paths.ca_path.exists(),
            cert_exists: expected_paths.cert_path.exists(),
            key_exists: expected_paths.key_path.exists(),
        };
        
        // Check for issues
        if !file_existence.ca_exists {
            issues.push(TlsIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::CertificateAccess,
                description: "CA certificate file not found".to_string(),
                details: format!(
                    "Framework resolved CA path to '{}' but file does not exist",
                    expected_paths.ca_path.display()
                ),
                resolution: "Ensure CA certificate exists at the resolved path or update QOLLECTIVE_TLS_CERT_BASE_PATH environment variable".to_string(),
            });
        }
        
        if !file_existence.cert_exists {
            issues.push(TlsIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::CertificateAccess,
                description: "Client certificate file not found".to_string(),
                details: format!(
                    "Framework resolved client cert path to '{}' but file does not exist",
                    expected_paths.cert_path.display()
                ),
                resolution: "Ensure client certificate exists at the resolved path or update QOLLECTIVE_TLS_CERT_BASE_PATH environment variable".to_string(),
            });
        }
        
        if !file_existence.key_exists {
            issues.push(TlsIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::CertificateAccess,
                description: "Client private key file not found".to_string(),
                details: format!(
                    "Framework resolved client key path to '{}' but file does not exist",
                    expected_paths.key_path.display()
                ),
                resolution: "Ensure client private key exists at the resolved path or update QOLLECTIVE_TLS_CERT_BASE_PATH environment variable".to_string(),
            });
        }
        
        // Check for path resolution issues
        if cargo_manifest_dir.is_none() {
            issues.push(TlsIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::PathResolution,
                description: "CARGO_MANIFEST_DIR not set".to_string(),
                details: "Framework path resolution relies on CARGO_MANIFEST_DIR but it's not available in current context".to_string(),
                resolution: "Set QOLLECTIVE_TLS_CERT_BASE_PATH environment variable to absolute path containing certificates".to_string(),
            });
        }
        
        // Check if we're falling back to legacy path
        if resolved_base_path_str.contains("/Users/ms/development/docker/nats/certs/server") {
            issues.push(TlsIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::PathResolution,
                description: "Using legacy hardcoded certificate path".to_string(),
                details: "Framework fell back to legacy absolute path which may not exist on all systems".to_string(),
                resolution: "Set QOLLECTIVE_TLS_CERT_BASE_PATH to point to actual certificate location".to_string(),
            });
        }
        
        PathResolutionAnalysis {
            current_dir,
            cargo_manifest_dir,
            resolved_base_path,
            expected_paths,
            file_existence,
        }
    }
    
    /// Analyze configuration setup
    fn analyze_configuration(issues: &mut Vec<TlsIssue>) -> ConfigAnalysis {
        // Load the enterprise configuration
        let config_result = EnterpriseConfig::load_default();
        
        let (tls_enabled, config_paths, framework_config) = match config_result {
            Ok(config) => {
                let tls_enabled = config.tls.enabled;
                let config_paths = CertificatePaths {
                    ca_path: PathBuf::from(&config.tls.ca_cert_path),
                    cert_path: PathBuf::from(&config.tls.cert_path),
                    key_path: PathBuf::from(&config.tls.key_path),
                };
                
                // Try to convert to framework config
                let framework_config = Ok(config.tls.to_framework_tls_config());
                
                (tls_enabled, config_paths, framework_config)
            }
            Err(e) => {
                issues.push(TlsIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Configuration,
                    description: "Failed to load configuration".to_string(),
                    details: format!("Could not load config.toml: {}", e),
                    resolution: "Ensure config.toml exists and is valid TOML format".to_string(),
                });
                
                (false, CertificatePaths {
                    ca_path: PathBuf::new(),
                    cert_path: PathBuf::new(),
                    key_path: PathBuf::new(),
                }, Err(QollectiveError::validation("Config load failed".to_string())))
            }
        };
        
        // Check for configuration issues
        if !tls_enabled {
            issues.push(TlsIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Configuration,
                description: "TLS is disabled in configuration".to_string(),
                details: "config.toml has tls.enabled = false but NATS URL suggests TLS connection".to_string(),
                resolution: "Set tls.enabled = true in config.toml".to_string(),
            });
        }
        
        // Check if config paths are relative (which will trigger smart resolution)
        if tls_enabled {
            if !config_paths.ca_path.is_absolute() {
                issues.push(TlsIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Configuration,
                    description: "Certificate paths in config are relative".to_string(),
                    details: "Relative paths trigger smart path resolution which may fail if CARGO_MANIFEST_DIR is not set".to_string(),
                    resolution: "Use absolute paths in config.toml or ensure QOLLECTIVE_TLS_CERT_BASE_PATH is set".to_string(),
                });
            }
        }
        
        ConfigAnalysis {
            tls_enabled,
            config_paths,
            framework_config,
        }
    }
    
    /// Analyze NATS connection configuration
    fn analyze_nats_connection(issues: &mut Vec<TlsIssue>) -> NatsConnectionAnalysis {
        let config_result = EnterpriseConfig::load_default();
        
        let (nats_urls, expects_tls, port_analysis) = match config_result {
            Ok(config) => {
                let nats_urls = config.nats.connection.urls.clone();
                let expects_tls = config.tls.enabled;
                
                // Analyze ports
                let mut configured_ports = Vec::new();
                let mut has_tls_ports = false;
                let mut has_non_tls_ports = false;
                
                for url in &nats_urls {
                    if let Some(port) = Self::extract_port_from_url(url) {
                        configured_ports.push(port);
                        if port == 4443 {
                            has_tls_ports = true;
                        } else if port == 4222 {
                            has_non_tls_ports = true;
                        }
                    }
                }
                
                let port_analysis = NatsPortAnalysis {
                    configured_ports,
                    has_tls_ports,
                    has_non_tls_ports,
                };
                
                (nats_urls, expects_tls, port_analysis)
            }
            Err(_) => {
                (vec![], false, NatsPortAnalysis {
                    configured_ports: vec![],
                    has_tls_ports: false,
                    has_non_tls_ports: false,
                })
            }
        };
        
        // Check for NATS/TLS configuration mismatches
        if expects_tls && !port_analysis.has_tls_ports {
            issues.push(TlsIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::NatsConnection,
                description: "TLS enabled but NATS not configured for TLS port".to_string(),
                details: "Configuration enables TLS but NATS URLs don't use TLS port 4443".to_string(),
                resolution: "Change NATS URL to use port 4443 for TLS connection".to_string(),
            });
        }
        
        if !expects_tls && port_analysis.has_tls_ports {
            issues.push(TlsIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::NatsConnection,
                description: "TLS disabled but NATS configured for TLS port".to_string(),
                details: "Configuration disables TLS but NATS URLs use TLS port 4443".to_string(),
                resolution: "Either enable TLS in config or change NATS URL to use port 4222".to_string(),
            });
        }
        
        if port_analysis.has_tls_ports && port_analysis.has_non_tls_ports {
            issues.push(TlsIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::NatsConnection,
                description: "Mixed TLS and non-TLS NATS URLs".to_string(),
                details: "Configuration contains both TLS (4443) and non-TLS (4222) NATS URLs".to_string(),
                resolution: "Use consistent TLS configuration across all NATS URLs".to_string(),
            });
        }
        
        NatsConnectionAnalysis {
            nats_urls,
            expects_tls,
            port_analysis,
        }
    }
    
    /// Extract port number from NATS URL
    fn extract_port_from_url(url: &str) -> Option<u16> {
        if let Some(port_start) = url.rfind(':') {
            let port_str = &url[port_start + 1..];
            port_str.parse().ok()
        } else {
            None
        }
    }
    
    /// Generate a comprehensive analysis report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# TLS Configuration Analysis Report\n");
        report.push_str("## Q Console Challenge - USS Enterprise NCC-1701-D\n\n");
        
        // Summary
        let critical_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
        let high_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::High).count();
        let medium_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Medium).count();
        let low_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Low).count();
        
        report.push_str(&format!("**Issues Summary:** {} Critical, {} High, {} Medium, {} Low\n\n", 
                                critical_count, high_count, medium_count, low_count));
        
        // Path Resolution Analysis
        report.push_str("## Path Resolution Analysis\n\n");
        report.push_str(&format!("- **Current Directory:** `{}`\n", self.path_resolution.current_dir.display()));
        report.push_str(&format!("- **CARGO_MANIFEST_DIR:** `{}`\n", 
                                self.path_resolution.cargo_manifest_dir.as_ref()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|| "Not Set".to_string())));
        report.push_str(&format!("- **Resolved Base Path:** `{}`\n", self.path_resolution.resolved_base_path.display()));
        report.push_str("\n### Expected Certificate Paths:\n");
        report.push_str(&format!("- **CA Certificate:** `{}` (exists: {})\n", 
                                self.path_resolution.expected_paths.ca_path.display(),
                                if self.path_resolution.file_existence.ca_exists { "✅" } else { "❌" }));
        report.push_str(&format!("- **Client Certificate:** `{}` (exists: {})\n", 
                                self.path_resolution.expected_paths.cert_path.display(),
                                if self.path_resolution.file_existence.cert_exists { "✅" } else { "❌" }));
        report.push_str(&format!("- **Client Key:** `{}` (exists: {})\n", 
                                self.path_resolution.expected_paths.key_path.display(),
                                if self.path_resolution.file_existence.key_exists { "✅" } else { "❌" }));
        
        // Configuration Analysis
        report.push_str("\n## Configuration Analysis\n\n");
        report.push_str(&format!("- **TLS Enabled:** {}\n", if self.config_analysis.tls_enabled { "✅" } else { "❌" }));
        report.push_str(&format!("- **Framework Config Conversion:** {}\n", 
                                if self.config_analysis.framework_config.is_ok() { "✅" } else { "❌" }));
        
        // NATS Analysis
        report.push_str("\n## NATS Connection Analysis\n\n");
        report.push_str(&format!("- **NATS URLs:** {:?}\n", self.nats_analysis.nats_urls));
        report.push_str(&format!("- **Expects TLS:** {}\n", if self.nats_analysis.expects_tls { "✅" } else { "❌" }));
        report.push_str(&format!("- **Configured Ports:** {:?}\n", self.nats_analysis.port_analysis.configured_ports));
        report.push_str(&format!("- **Has TLS Ports (4443):** {}\n", if self.nats_analysis.port_analysis.has_tls_ports { "✅" } else { "❌" }));
        report.push_str(&format!("- **Has Non-TLS Ports (4222):** {}\n", if self.nats_analysis.port_analysis.has_non_tls_ports { "✅" } else { "❌" }));
        
        // Issues
        if !self.issues.is_empty() {
            report.push_str("\n## Discovered Issues\n\n");
            
            for (i, issue) in self.issues.iter().enumerate() {
                let severity_icon = match issue.severity {
                    IssueSeverity::Critical => "🚨",
                    IssueSeverity::High => "⚠️",
                    IssueSeverity::Medium => "⚡",
                    IssueSeverity::Low => "ℹ️",
                };
                
                report.push_str(&format!("### {} Issue #{}: {} {}\n\n", 
                                        severity_icon, i + 1, issue.description, severity_icon));
                report.push_str(&format!("**Category:** {:?}\n", issue.category));
                report.push_str(&format!("**Details:** {}\n", issue.details));
                report.push_str(&format!("**Resolution:** {}\n\n", issue.resolution));
            }
        }
        
        // Recommendations
        report.push_str("\n## Recommendations\n\n");
        if critical_count > 0 || high_count > 0 {
            report.push_str("### Immediate Actions Required:\n\n");
            if critical_count > 0 {
                report.push_str("1. **Fix Critical Issues:** Certificate files are missing or inaccessible\n");
                report.push_str("2. **Set Environment Variable:** `export QOLLECTIVE_TLS_CERT_BASE_PATH=/Users/ms/development/qollective/software/tests/certs`\n");
            }
            if high_count > 0 {
                report.push_str("3. **Configuration Mismatch:** Ensure TLS configuration matches NATS connection settings\n");
            }
        }
        
        report.push_str("\n### Long-term Improvements:\n\n");
        report.push_str("1. **Use Absolute Paths:** Update config.toml to use absolute certificate paths\n");
        report.push_str("2. **Environment Variables:** Set QOLLECTIVE_TLS_CERT_BASE_PATH in deployment environment\n");
        report.push_str("3. **Validation:** Add certificate file validation during startup\n");
        report.push_str("4. **Documentation:** Document TLS setup requirements clearly\n");
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_analysis_runs_without_panic() {
        let report = TlsAnalysisReport::analyze();
        
        // Should always have some analysis data
        assert!(!report.path_resolution.current_dir.as_os_str().is_empty());
        assert!(!report.path_resolution.resolved_base_path.as_os_str().is_empty());
    }
    
    #[test]
    fn test_path_resolution_identifies_missing_certificates() {
        let report = TlsAnalysisReport::analyze();
        
        // If certificates don't exist, should have critical issues
        if !report.path_resolution.file_existence.ca_exists ||
           !report.path_resolution.file_existence.cert_exists ||
           !report.path_resolution.file_existence.key_exists {
            let critical_issues: Vec<_> = report.issues.iter()
                .filter(|issue| issue.severity == IssueSeverity::Critical && 
                               issue.category == IssueCategory::CertificateAccess)
                .collect();
            assert!(!critical_issues.is_empty(), "Should identify missing certificate files as critical issues");
        }
    }
    
    #[test]
    fn test_nats_port_analysis() {
        let report = TlsAnalysisReport::analyze();
        
        // Should analyze NATS ports
        assert!(!report.nats_analysis.nats_urls.is_empty() || report.issues.iter().any(|i| i.category == IssueCategory::Configuration));
    }
    
    #[test]
    fn test_report_generation() {
        let report = TlsAnalysisReport::analyze();
        let report_text = report.generate_report();
        
        assert!(report_text.contains("TLS Configuration Analysis Report"));
        assert!(report_text.contains("Path Resolution Analysis"));
        assert!(report_text.contains("Configuration Analysis"));
        assert!(report_text.contains("NATS Connection Analysis"));
    }
    
    #[test]
    fn test_port_extraction() {
        assert_eq!(TlsAnalysisReport::extract_port_from_url("nats://localhost:4443"), Some(4443));
        assert_eq!(TlsAnalysisReport::extract_port_from_url("nats://localhost:4222"), Some(4222));
        assert_eq!(TlsAnalysisReport::extract_port_from_url("nats://localhost"), None);
    }
}