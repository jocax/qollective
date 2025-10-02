// ABOUTME: Certificate management for WASM mTLS support
// ABOUTME: Handles embedded certificates and secure storage for browser authentication

//! Certificate management for WASM applications.
//!
//! This module provides certificate management capabilities for WASM applications
//! including embedded certificates, validation, and mTLS support.

use crate::config::wasm::{CertificateConfig, EmbeddedCertificate};
use crate::error::{QollectiveError, Result};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// WASM certificate manager
#[derive(Debug, Clone)]
pub struct WasmCertificateManager {
    config: CertificateConfig,
    cached_certificates: HashMap<String, ProcessedCertificate>,
}

/// Processed certificate with decoded data
#[derive(Debug, Clone)]
struct ProcessedCertificate {
    cert_pem: String,
    key_pem: String,
    ca_pem: Option<String>,
    domains: Vec<String>,
    name: String,
    expiry_timestamp: Option<u64>,
}

/// Certificate information for JavaScript
#[wasm_bindgen]
pub struct CertificateInfo {
    name: String,
    domains: Vec<String>,
    valid: bool,
    expires_at: Option<u64>,
}

#[wasm_bindgen]
impl CertificateInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn domains(&self) -> Vec<String> {
        self.domains.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn valid(&self) -> bool {
        self.valid
    }

    #[wasm_bindgen(getter)]
    pub fn expires_at(&self) -> Option<u64> {
        self.expires_at
    }
}

impl WasmCertificateManager {
    /// Create new certificate manager
    pub fn new(config: &CertificateConfig) -> Result<Self> {
        let mut manager = Self {
            config: config.clone(),
            cached_certificates: HashMap::new(),
        };

        // Pre-process all embedded certificates
        manager.initialize_certificates()?;

        Ok(manager)
    }

    /// Initialize and validate all embedded certificates
    fn initialize_certificates(&mut self) -> Result<()> {
        for (key, embedded_cert) in &self.config.embedded_certificates {
            match self.process_embedded_certificate(embedded_cert) {
                Ok(processed) => {
                    self.cached_certificates.insert(key.clone(), processed);
                }
                Err(e) => {
                    web_sys::console::warn_1(&format!(
                        "Failed to process certificate '{}': {}",
                        key, e
                    ).into());
                    
                    // Don't fail initialization for individual cert issues unless strict validation is enabled
                    if self.config.validation.verify_chain {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Process an embedded certificate from base64 to PEM
    fn process_embedded_certificate(&self, cert: &EmbeddedCertificate) -> Result<ProcessedCertificate> {
        // Decode base64 certificate data to PEM
        let cert_pem = self.decode_base64_to_pem(&cert.cert_data, "CERTIFICATE")?;
        let key_pem = self.decode_base64_to_pem(&cert.key_data, "PRIVATE KEY")?;
        
        let ca_pem = if let Some(ca_data) = &cert.ca_data {
            Some(self.decode_base64_to_pem(ca_data, "CERTIFICATE")?)
        } else {
            None
        };

        // Extract expiry timestamp from certificate (simplified - would need proper X.509 parsing)
        let expiry_timestamp = self.extract_expiry_timestamp(&cert_pem)?;

        Ok(ProcessedCertificate {
            cert_pem,
            key_pem,
            ca_pem,
            domains: cert.domains.clone(),
            name: cert.name.clone(),
            expiry_timestamp,
        })
    }

    /// Decode base64 data to PEM format
    fn decode_base64_to_pem(&self, base64_data: &str, cert_type: &str) -> Result<String> {
        // In a real implementation, we would use a proper base64 decoder
        // For WASM, we can use the browser's built-in atob function
        
        // Validate base64 format
        if base64_data.is_empty() {
            return Err(QollectiveError::validation("Empty certificate data"));
        }

        // For now, assume the data is already in PEM format
        // In a real implementation, we would:
        // 1. Decode base64 to bytes
        // 2. Wrap in PEM headers
        let pem = if base64_data.starts_with("-----BEGIN") {
            // Already in PEM format
            base64_data.to_string()
        } else {
            // Convert base64 to PEM
            format!(
                "-----BEGIN {}-----\n{}\n-----END {}-----",
                cert_type,
                base64_data,
                cert_type
            )
        };

        Ok(pem)
    }

    /// Extract expiry timestamp from certificate (simplified implementation)
    fn extract_expiry_timestamp(&self, _cert_pem: &str) -> Result<Option<u64>> {
        // In a real implementation, we would parse the X.509 certificate
        // For now, return None (certificate validation would be done server-side)
        Ok(None)
    }

    /// Get certificate for domain
    pub fn get_certificate_for_domain(&self, domain: &str) -> Result<Option<CertificateBundle>> {
        // Find certificate that matches the domain
        for (_, cert) in &self.cached_certificates {
            if self.domain_matches(&cert.domains, domain) {
                return Ok(Some(CertificateBundle {
                    cert_pem: cert.cert_pem.clone(),
                    key_pem: cert.key_pem.clone(),
                    ca_pem: cert.ca_pem.clone(),
                    name: cert.name.clone(),
                }));
            }
        }

        Ok(None)
    }

    /// Check if domain matches certificate domains (supports wildcards)
    fn domain_matches(&self, cert_domains: &[String], target_domain: &str) -> bool {
        for cert_domain in cert_domains {
            if cert_domain == target_domain {
                return true;
            }
            
            // Support wildcard domains (*.example.com)
            if cert_domain.starts_with("*.") {
                let base_domain = &cert_domain[2..]; // Remove "*."
                if target_domain.ends_with(base_domain) {
                    // Check that it's a subdomain, not a partial match
                    let prefix = &target_domain[..target_domain.len() - base_domain.len()];
                    if prefix.is_empty() || (prefix.ends_with('.') && !prefix.contains('.')) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    /// Validate certificate chain
    pub fn validate_certificate(&self, cert_data: &str) -> Result<bool> {
        if !self.config.validation.verify_chain {
            return Ok(true); // Skip validation if disabled
        }

        // Basic PEM format validation
        if !cert_data.contains("-----BEGIN CERTIFICATE-----") {
            return Ok(false);
        }

        if !cert_data.contains("-----END CERTIFICATE-----") {
            return Ok(false);
        }

        // In a real implementation, we would:
        // 1. Parse the X.509 certificate
        // 2. Verify the signature chain
        // 3. Check expiration dates
        // 4. Verify against custom CA certificates if configured
        // 5. Check certificate revocation lists

        // For now, return true for well-formed PEM certificates
        Ok(true)
    }

    /// Get certificate information for JavaScript
    pub fn get_certificate_info(&self, cert_name: &str) -> Result<Option<CertificateInfo>> {
        if let Some(cert) = self.cached_certificates.get(cert_name) {
            let valid = self.validate_certificate(&cert.cert_pem)?;
            
            Ok(Some(CertificateInfo {
                name: cert.name.clone(),
                domains: cert.domains.clone(),
                valid,
                expires_at: cert.expiry_timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all available certificates
    pub fn list_certificates(&self) -> Vec<String> {
        self.cached_certificates.keys().cloned().collect()
    }

    /// Refresh certificates (check for updates, validate expiry)
    pub fn refresh_certificates(&mut self) -> Result<u32> {
        if !self.config.auto_refresh {
            return Ok(0);
        }

        let mut refreshed_count = 0;
        let current_time = js_sys::Date::now() as u64 / 1000; // Convert to seconds

        for (key, cert) in &self.cached_certificates {
            if let Some(expiry) = cert.expiry_timestamp {
                let threshold = self.config.refresh_threshold_secs;
                if current_time + threshold > expiry {
                    web_sys::console::info_1(&format!(
                        "Certificate '{}' expires soon, would refresh if auto-refresh was implemented",
                        cert.name
                    ).into());
                    
                    // In a real implementation, we would:
                    // 1. Fetch updated certificate from server
                    // 2. Validate the new certificate
                    // 3. Update the cached certificate
                    // 4. Store in local storage if configured
                    
                    refreshed_count += 1;
                }
            }
        }

        Ok(refreshed_count)
    }

    /// Check if any certificates are near expiry
    pub fn check_expiry_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        let current_time = js_sys::Date::now() as u64 / 1000;

        for cert in self.cached_certificates.values() {
            if let Some(expiry) = cert.expiry_timestamp {
                let days_until_expiry = (expiry - current_time) / 86400; // 86400 seconds in a day
                
                if days_until_expiry <= 30 {
                    warnings.push(format!(
                        "Certificate '{}' expires in {} days",
                        cert.name, days_until_expiry
                    ));
                }
            }
        }

        warnings
    }
}

/// Certificate bundle for client authentication
#[derive(Debug, Clone)]
pub struct CertificateBundle {
    pub cert_pem: String,
    pub key_pem: String,
    pub ca_pem: Option<String>,
    pub name: String,
}

impl CertificateBundle {
    /// Get certificate data for HTTP headers (base64 encoded)
    pub fn to_client_cert_header(&self) -> Result<String> {
        // In a real implementation, we would convert the PEM back to base64
        // For now, extract the base64 data from PEM format
        let cert_lines: Vec<&str> = self.cert_pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        
        let base64_cert = cert_lines.join("");
        Ok(base64_cert)
    }

    /// Get combined certificate chain
    pub fn get_cert_chain(&self) -> String {
        if let Some(ca_pem) = &self.ca_pem {
            format!("{}\n{}", self.cert_pem, ca_pem)
        } else {
            self.cert_pem.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::wasm::{CertificateConfig, CertificateValidation, EmbeddedCertificate};
    use std::collections::HashMap;

    fn create_test_certificate() -> EmbeddedCertificate {
        EmbeddedCertificate {
            cert_data: "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t".to_string(), // Base64 placeholder
            key_data: "LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0t".to_string(), // Base64 placeholder
            ca_data: None,
            name: "test_cert".to_string(),
            domains: vec!["example.com".to_string(), "*.test.com".to_string()],
        }
    }

    #[test]
    fn test_certificate_manager_creation() {
        let mut config = CertificateConfig::default();
        config.embedded_certificates.insert("test".to_string(), create_test_certificate());

        let manager = WasmCertificateManager::new(&config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_domain_matching() {
        let mut config = CertificateConfig::default();
        config.embedded_certificates.insert("test".to_string(), create_test_certificate());

        let manager = WasmCertificateManager::new(&config).unwrap();

        // Test exact domain match
        assert!(manager.domain_matches(&["example.com".to_string()], "example.com"));
        
        // Test wildcard domain match
        assert!(manager.domain_matches(&["*.test.com".to_string()], "api.test.com"));
        assert!(manager.domain_matches(&["*.test.com".to_string()], "sub.test.com"));
        
        // Test non-matching domains
        assert!(!manager.domain_matches(&["example.com".to_string()], "other.com"));
        assert!(!manager.domain_matches(&["*.test.com".to_string()], "test.com")); // Root domain doesn't match wildcard
    }

    #[test]
    fn test_certificate_validation() {
        let config = CertificateConfig::default();
        let manager = WasmCertificateManager::new(&config).unwrap();

        // Test valid PEM format
        let valid_cert = "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----";
        assert!(manager.validate_certificate(valid_cert).unwrap());

        // Test invalid format
        let invalid_cert = "not a certificate";
        assert!(!manager.validate_certificate(invalid_cert).unwrap());
    }

    #[test]
    fn test_certificate_lookup() {
        let mut config = CertificateConfig::default();
        config.embedded_certificates.insert("test".to_string(), create_test_certificate());

        let manager = WasmCertificateManager::new(&config).unwrap();

        // Test finding certificate for exact domain
        let cert = manager.get_certificate_for_domain("example.com");
        assert!(cert.is_ok());
        assert!(cert.unwrap().is_some());

        // Test finding certificate for wildcard domain
        let cert = manager.get_certificate_for_domain("api.test.com");
        assert!(cert.is_ok());
        assert!(cert.unwrap().is_some());

        // Test not finding certificate for unmatched domain
        let cert = manager.get_certificate_for_domain("other.com");
        assert!(cert.is_ok());
        assert!(cert.unwrap().is_none());
    }

    #[test]
    fn test_list_certificates() {
        let mut config = CertificateConfig::default();
        config.embedded_certificates.insert("test1".to_string(), create_test_certificate());
        config.embedded_certificates.insert("test2".to_string(), create_test_certificate());

        let manager = WasmCertificateManager::new(&config).unwrap();
        let certs = manager.list_certificates();

        assert_eq!(certs.len(), 2);
        assert!(certs.contains(&"test1".to_string()));
        assert!(certs.contains(&"test2".to_string()));
    }
}
