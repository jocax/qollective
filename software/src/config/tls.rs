// ABOUTME: Unified TLS configuration for all transport protocols in the framework
// ABOUTME: Provides centralized TLS setup, certificate management, and rustls configuration

//! Unified TLS configuration for all transport protocols.
//!
//! This module provides centralized TLS configuration that eliminates code duplication
//! across all transport protocols (REST, gRPC, WebSocket, NATS). It handles certificate
//! loading, path expansion, and rustls configuration with consistent behavior.
//!
//! # Quick Start
//!
//! ```rust
//! use qollective::config::tls::{TlsConfig, TlsConfigBuilder};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create TLS configuration
//! let tls_config = TlsConfigBuilder::new()
//!     .enabled(true)
//!     .cert_path("/path/to/cert.pem")
//!     .key_path("/path/to/key.pem")
//!     .verify_certificates(true)
//!     .build()?;
//!
//! // Create rustls client configuration
//! let client_config = tls_config.create_client_config().await?;
//!
//! // Create rustls server configuration
//! let server_config = tls_config.create_server_config().await?;
//! # Ok(())
//! # }\
//! ```

use crate::{
    constants::tls as tls_constants,
    error::{QollectiveError, Result},
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Certificate verification modes for TLS connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationMode {
    /// Use system CA store for verification (default)
    SystemCa,
    /// Use custom CA certificate for verification
    CustomCa,
    /// Skip certificate verification (development only)
    Skip,
    /// Enable mutual TLS with client certificate authentication
    MutualTls,
}

impl Default for VerificationMode {
    fn default() -> Self {
        Self::SystemCa
    }
}

/// Unified TLS configuration for all transport protocols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TlsConfig {
    /// Whether TLS is enabled
    pub enabled: bool,
    /// Path to certificate file
    pub cert_path: Option<PathBuf>,
    /// Path to private key file
    pub key_path: Option<PathBuf>,
    /// Path to CA certificate file
    pub ca_cert_path: Option<PathBuf>,
    /// Certificate verification mode
    pub verification_mode: VerificationMode,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_cert_path: None,
            verification_mode: VerificationMode::default(),
        }
    }
}

/// Builder for TLS configuration
#[derive(Debug, Default)]
pub struct TlsConfigBuilder {
    config: TlsConfig,
}

impl TlsConfigBuilder {
    /// Create a new TLS configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable TLS
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set certificate file path
    pub fn cert_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.cert_path = Some(TlsConfig::expand_path(&path.as_ref().to_string_lossy()));
        self
    }

    /// Set private key file path
    pub fn key_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.key_path = Some(TlsConfig::expand_path(&path.as_ref().to_string_lossy()));
        self
    }

    /// Set CA certificate file path
    pub fn ca_cert_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.ca_cert_path = Some(TlsConfig::expand_path(&path.as_ref().to_string_lossy()));
        self
    }

    /// Set certificate verification mode
    pub fn verification_mode(mut self, mode: VerificationMode) -> Self {
        self.config.verification_mode = mode;
        self
    }

    /// Set verification mode to system CA (default)
    pub fn verify_certificates(mut self, verify: bool) -> Self {
        self.config.verification_mode = if verify {
            VerificationMode::SystemCa
        } else {
            VerificationMode::Skip
        };
        self
    }

    /// Build the TLS configuration
    pub fn build(self) -> Result<TlsConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl TlsConfig {
    /// Create a new TLS configuration builder
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::new()
    }

    /// Create TLS configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Check if TLS is enabled
        if let Ok(enabled_str) = env::var(tls_constants::env_vars::QOLLECTIVE_TLS_ENABLED) {
            config.enabled = enabled_str.to_lowercase() == "true";
        }

        // Get certificate path
        if let Ok(cert_path) = env::var(tls_constants::env_vars::QOLLECTIVE_TLS_CERT_PATH) {
            config.cert_path = Some(Self::expand_path(&cert_path));
        }

        // Get key path
        if let Ok(key_path) = env::var(tls_constants::env_vars::QOLLECTIVE_TLS_KEY_PATH) {
            config.key_path = Some(Self::expand_path(&key_path));
        }

        // Get CA certificate path
        if let Ok(ca_path) = env::var(tls_constants::env_vars::QOLLECTIVE_TLS_CA_PATH) {
            config.ca_cert_path = Some(Self::expand_path(&ca_path));
        }

        // Get verification mode
        if let Ok(mode_str) = env::var(tls_constants::env_vars::QOLLECTIVE_TLS_VERIFY_MODE) {
            config.verification_mode = match mode_str.to_lowercase().as_str() {
                "system_ca" | "systemca" => VerificationMode::SystemCa,
                "custom_ca" | "customca" => VerificationMode::CustomCa,
                "skip" => VerificationMode::Skip,
                "mutual_tls" | "mutualtls" => VerificationMode::MutualTls,
                _ => {
                    return Err(QollectiveError::config(format!(
                        "Invalid verification mode: {}",
                        mode_str
                    )))
                }
            };
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate TLS configuration
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Check that required paths are provided
        if self.cert_path.is_none() {
            return Err(QollectiveError::config(
                "Certificate path is required when TLS is enabled",
            ));
        }

        if self.key_path.is_none() {
            return Err(QollectiveError::config(
                "Private key path is required when TLS is enabled",
            ));
        }

        // Validate custom CA mode
        if self.verification_mode == VerificationMode::CustomCa && self.ca_cert_path.is_none() {
            return Err(QollectiveError::config(
                "CA certificate path is required for CustomCa verification mode",
            ));
        }

        Ok(())
    }

    /// Create rustls client configuration
    #[cfg(feature = "tls")]
    pub async fn create_client_config(&self) -> Result<Arc<rustls::ClientConfig>> {
        if !self.enabled {
            return Err(QollectiveError::config(
                "Cannot create client config when TLS is disabled",
            ));
        }

        // Ensure crypto provider is initialized
        crate::crypto::ensure_crypto_provider()?;

        let config_builder = rustls::ClientConfig::builder();

        // Configure verification based on verification mode
        let mut config = match self.verification_mode {
            VerificationMode::SystemCa => {
                // Use system CA store for verification
                config_builder
                    .with_root_certificates(rustls::RootCertStore::from_iter(
                        webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
                    ))
                    .with_no_client_auth()
            }
            VerificationMode::CustomCa => {
                // Use custom CA certificate
                let ca_path = self.ca_cert_path.as_ref().ok_or_else(|| {
                    QollectiveError::config("CA certificate path is required for CustomCa mode")
                })?;

                let ca_cert = Self::load_ca_certificate(ca_path).await?;
                let mut root_cert_store = rustls::RootCertStore::empty();
                root_cert_store.add(ca_cert).map_err(|e| {
                    QollectiveError::tls(format!("Failed to add CA certificate: {}", e))
                })?;

                config_builder
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth()
            }
            VerificationMode::Skip => {
                // Skip certificate verification (development only)
                config_builder
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerification))
                    .with_no_client_auth()
            }
            VerificationMode::MutualTls => {
                // Enable mutual TLS with client certificate
                let (cert_chain, private_key) = self.load_certificate_chain().await?;

                let root_cert_store = if let Some(ca_path) = &self.ca_cert_path {
                    let ca_cert = Self::load_ca_certificate(ca_path).await?;
                    let mut store = rustls::RootCertStore::empty();
                    store.add(ca_cert).map_err(|e| {
                        QollectiveError::tls(format!("Failed to add CA certificate: {}", e))
                    })?;
                    store
                } else {
                    rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned())
                };

                config_builder
                    .with_root_certificates(root_cert_store)
                    .with_client_auth_cert(cert_chain, private_key)
                    .map_err(|e| {
                        QollectiveError::tls(format!(
                            "Failed to configure client certificate: {}",
                            e
                        ))
                    })?
            }
        };

        // For Skip mode, disable SNI to avoid "Illegal SNI extension" errors with IP addresses
        if self.verification_mode == VerificationMode::Skip {
            config.enable_sni = false;
        }

        Ok(Arc::new(config))
    }

    /// Create rustls server configuration
    #[cfg(feature = "tls")]
    pub async fn create_server_config(&self) -> Result<Arc<rustls::ServerConfig>> {
        if !self.enabled {
            return Err(QollectiveError::config(
                "Cannot create server config when TLS is disabled",
            ));
        }

        // Ensure crypto provider is initialized
        crate::crypto::ensure_crypto_provider()?;

        let (cert_chain, private_key) = self.load_certificate_chain().await?;

        let config = match self.verification_mode {
            VerificationMode::SystemCa | VerificationMode::CustomCa | VerificationMode::Skip => {
                // Server configuration without client certificate requirement
                rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert_chain, private_key)
                    .map_err(|e| {
                        QollectiveError::tls(format!("Failed to create server config: {}", e))
                    })?
            }
            VerificationMode::MutualTls => {
                // Server configuration with client certificate requirement
                let client_cert_verifier = if let Some(ca_path) = &self.ca_cert_path {
                    let ca_cert = Self::load_ca_certificate(ca_path).await?;
                    let mut client_cert_store = rustls::RootCertStore::empty();
                    client_cert_store.add(ca_cert).map_err(|e| {
                        QollectiveError::tls(format!("Failed to add client CA certificate: {}", e))
                    })?;

                    rustls::server::WebPkiClientVerifier::builder(Arc::new(client_cert_store))
                        .build()
                        .map_err(|e| {
                            QollectiveError::tls(format!("Failed to create client verifier: {}", e))
                        })?
                } else {
                    return Err(QollectiveError::config(
                        "CA certificate path is required for mutual TLS",
                    ));
                };

                rustls::ServerConfig::builder()
                    .with_client_cert_verifier(client_cert_verifier)
                    .with_single_cert(cert_chain, private_key)
                    .map_err(|e| {
                        QollectiveError::tls(format!("Failed to create mTLS server config: {}", e))
                    })?
            }
        };

        Ok(Arc::new(config))
    }

    /// Load certificate chain and private key
    #[cfg(feature = "tls")]
    async fn load_certificate_chain(
        &self,
    ) -> Result<(
        Vec<rustls::pki_types::CertificateDer<'static>>,
        rustls::pki_types::PrivateKeyDer<'static>,
    )> {
        let cert_path = self
            .cert_path
            .as_ref()
            .ok_or_else(|| QollectiveError::config("Certificate path is required"))?;

        let key_path = self
            .key_path
            .as_ref()
            .ok_or_else(|| QollectiveError::config("Private key path is required"))?;

        let (cert_chain, private_key) = tokio::task::spawn_blocking({
            let cert_path = cert_path.clone();
            let key_path = key_path.clone();
            move || -> Result<(Vec<rustls::pki_types::CertificateDer<'static>>, rustls::pki_types::PrivateKeyDer<'static>)> {
                use rustls_pemfile::{certs, private_key};
                use std::io::BufReader;

                // Load certificate file
                let cert_file = std::fs::File::open(&cert_path)
                    .map_err(|e| QollectiveError::tls(format!("Failed to open certificate file {:?}: {}", cert_path, e)))?;
                let mut cert_reader = BufReader::new(cert_file);

                // Parse certificates
                let cert_chain: Vec<rustls::pki_types::CertificateDer<'static>> = certs(&mut cert_reader)
                    .collect::<std::result::Result<Vec<_>, std::io::Error>>()
                    .map_err(|e| QollectiveError::tls(format!("Failed to parse certificate file: {}", e)))?;

                if cert_chain.is_empty() {
                    return Err(QollectiveError::tls("No certificates found in certificate file"));
                }

                // Load private key file
                let key_file = std::fs::File::open(&key_path)
                    .map_err(|e| QollectiveError::tls(format!("Failed to open private key file {:?}: {}", key_path, e)))?;
                let mut key_reader = BufReader::new(key_file);

                // Parse private key
                let private_key = private_key(&mut key_reader)
                    .map_err(|e| QollectiveError::tls(format!("Failed to parse private key file: {}", e)))?
                    .ok_or_else(|| QollectiveError::tls("No private key found in key file"))?;

                Ok((cert_chain, private_key))
            }
        })
        .await
        .map_err(|e| QollectiveError::tls(format!("Failed to load certificates: {}", e)))??;

        Ok((cert_chain, private_key))
    }

    /// Load CA certificate
    #[cfg(feature = "tls")]
    async fn load_ca_certificate(
        ca_path: &Path,
    ) -> Result<rustls::pki_types::CertificateDer<'static>> {
        let ca_path = ca_path.to_path_buf();
        let ca_cert = tokio::task::spawn_blocking(
            move || -> Result<rustls::pki_types::CertificateDer<'static>> {
                use rustls_pemfile::certs;
                use std::io::BufReader;

                let ca_file = std::fs::File::open(&ca_path).map_err(|e| {
                    QollectiveError::tls(format!(
                        "Failed to open CA certificate file {:?}: {}",
                        ca_path, e
                    ))
                })?;
                let mut ca_reader = BufReader::new(ca_file);

                let mut ca_certs: Vec<rustls::pki_types::CertificateDer<'static>> =
                    certs(&mut ca_reader)
                        .collect::<std::result::Result<Vec<_>, std::io::Error>>()
                        .map_err(|e| {
                            QollectiveError::tls(format!(
                                "Failed to parse CA certificate file: {}",
                                e
                            ))
                        })?;

                if ca_certs.is_empty() {
                    return Err(QollectiveError::tls(
                        "No CA certificates found in CA certificate file",
                    ));
                }

                Ok(ca_certs.remove(0))
            },
        )
        .await
        .map_err(|e| QollectiveError::tls(format!("Failed to load CA certificate: {}", e)))??;

        Ok(ca_cert)
    }

    /// Expand path with environment variables and home directory
    fn expand_path(path: &str) -> PathBuf {
        let mut expanded = path.to_string();

        // Expand home directory
        if expanded.starts_with('~') {
            if let Some(home) = env::var_os("HOME") {
                expanded = expanded.replacen('~', &home.to_string_lossy(), 1);
            }
        }

        // Simple environment variable expansion for ${VAR} format
        while let Some(start) = expanded.find("${") {
            if let Some(end) = expanded[start..].find('}') {
                let var_name = &expanded[start + 2..start + end];
                let replacement =
                    env::var(var_name).unwrap_or_else(|_| format!("${{{}}}", var_name));
                expanded.replace_range(start..start + end + 1, &replacement);
            } else {
                break;
            }
        }

        PathBuf::from(expanded)
    }
}

/// Custom certificate verifier that accepts all certificates (for development/testing)
#[cfg(feature = "tls")]
#[derive(Debug)]
pub struct NoVerification;

#[cfg(feature = "tls")]
impl rustls::client::danger::ServerCertVerifier for NoVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

// Preset configurations for different environments
impl TlsConfig {
    /// Create production-ready TLS configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            cert_path: Some(PathBuf::from("/etc/ssl/certs/qollective.crt")),
            key_path: Some(PathBuf::from("/etc/ssl/private/qollective.key")),
            ca_cert_path: None,
            verification_mode: VerificationMode::SystemCa,
        }
    }

    /// Create development TLS configuration with self-signed certificates
    pub fn development() -> Self {
        Self {
            enabled: true,
            cert_path: Some(PathBuf::from("./certs/dev-cert.pem")),
            key_path: Some(PathBuf::from("./certs/dev-key.pem")),
            ca_cert_path: None,
            verification_mode: VerificationMode::Skip,
        }
    }

    /// Create high-performance TLS configuration
    pub fn high_performance() -> Self {
        Self {
            enabled: true,
            cert_path: Some(PathBuf::from("/etc/ssl/certs/qollective.crt")),
            key_path: Some(PathBuf::from("/etc/ssl/private/qollective.key")),
            ca_cert_path: None,
            verification_mode: VerificationMode::SystemCa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(!config.enabled);
        assert!(config.cert_path.is_none());
        assert!(config.key_path.is_none());
        assert!(config.ca_cert_path.is_none());
        assert_eq!(config.verification_mode, VerificationMode::SystemCa);
    }

    #[test]
    fn test_tls_config_builder() {
        let config = TlsConfig::builder()
            .enabled(true)
            .cert_path("/path/to/cert.pem")
            .key_path("/path/to/key.pem")
            .verification_mode(VerificationMode::Skip)
            .build()
            .unwrap();

        assert!(config.enabled);
        assert_eq!(config.cert_path, Some(PathBuf::from("/path/to/cert.pem")));
        assert_eq!(config.key_path, Some(PathBuf::from("/path/to/key.pem")));
        assert_eq!(config.verification_mode, VerificationMode::Skip);
    }

    #[test]
    fn test_tls_config_from_env() {
        env::set_var("QOLLECTIVE_TLS_ENABLED", "true");
        env::set_var("QOLLECTIVE_TLS_CERT_PATH", "/path/to/cert.pem");
        env::set_var("QOLLECTIVE_TLS_KEY_PATH", "/path/to/key.pem");
        env::set_var("QOLLECTIVE_TLS_VERIFY_MODE", "skip");

        let config = TlsConfig::from_env().unwrap();
        assert!(config.enabled);
        assert_eq!(config.cert_path, Some(PathBuf::from("/path/to/cert.pem")));
        assert_eq!(config.key_path, Some(PathBuf::from("/path/to/key.pem")));
        assert_eq!(config.verification_mode, VerificationMode::Skip);

        // Clean up
        env::remove_var("QOLLECTIVE_TLS_ENABLED");
        env::remove_var("QOLLECTIVE_TLS_CERT_PATH");
        env::remove_var("QOLLECTIVE_TLS_KEY_PATH");
        env::remove_var("QOLLECTIVE_TLS_VERIFY_MODE");
    }

    #[test]
    fn test_tls_config_validation() {
        // Test disabled TLS - should pass validation
        let config = TlsConfig::default();
        assert!(config.validate().is_ok());

        // Test enabled TLS without cert path - should fail
        let config = TlsConfig::builder().enabled(true).build();
        assert!(config.is_err());

        // Test custom CA mode without CA path - should fail
        let config = TlsConfig::builder()
            .enabled(true)
            .cert_path("/path/to/cert.pem")
            .key_path("/path/to/key.pem")
            .verification_mode(VerificationMode::CustomCa)
            .build();
        assert!(config.is_err());

        // Test valid configuration
        let config = TlsConfig::builder()
            .enabled(true)
            .cert_path("/path/to/cert.pem")
            .key_path("/path/to/key.pem")
            .verification_mode(VerificationMode::Skip)
            .build()
            .unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_expand_path_basic() {
        let path = TlsConfig::expand_path("/absolute/path/to/cert.pem");
        assert_eq!(path, PathBuf::from("/absolute/path/to/cert.pem"));
    }

    #[test]
    fn test_expand_path_home() {
        env::set_var("HOME", "/home/user");
        let path = TlsConfig::expand_path("~/cert.pem");
        assert_eq!(path, PathBuf::from("/home/user/cert.pem"));
        env::remove_var("HOME");
    }

    #[test]
    fn test_expand_path_env_var() {
        env::set_var("CERT_DIR", "/etc/ssl/certs");
        let path = TlsConfig::expand_path("${CERT_DIR}/cert.pem");
        assert_eq!(path, PathBuf::from("/etc/ssl/certs/cert.pem"));
        env::remove_var("CERT_DIR");
    }

    #[test]
    fn test_verification_mode_serialization() {
        let mode = VerificationMode::SystemCa;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: VerificationMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mode, deserialized);
    }

    #[test]
    fn test_tls_config_presets() {
        let prod_config = TlsConfig::production();
        assert!(prod_config.enabled);
        assert_eq!(prod_config.verification_mode, VerificationMode::SystemCa);

        let dev_config = TlsConfig::development();
        assert!(dev_config.enabled);
        assert_eq!(dev_config.verification_mode, VerificationMode::Skip);

        let perf_config = TlsConfig::high_performance();
        assert!(perf_config.enabled);
        assert_eq!(perf_config.verification_mode, VerificationMode::SystemCa);
    }
}
