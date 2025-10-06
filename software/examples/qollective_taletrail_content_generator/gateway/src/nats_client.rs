//! NATS client with TLS support for Gateway

use shared_types::*;
use std::fs;

/// Load a certificate from PEM file
pub fn load_cert(path: &str) -> Result<rustls::pki_types::CertificateDer<'static>> {
    let cert_data = fs::read(path)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to read cert {}: {}", path, e)))?;

    let mut cursor = std::io::Cursor::new(cert_data);
    let certs = rustls_pemfile::certs(&mut cursor)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse cert: {}", e)))?;

    certs.into_iter().next()
        .ok_or_else(|| TaleTrailError::TlsCertificateError(format!("No certificate found in {}", path)))
}

/// Load a private key from PEM file
pub fn load_key(path: &str) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
    let key_data = fs::read(path)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to read key {}: {}", path, e)))?;

    let mut cursor = std::io::Cursor::new(key_data);
    rustls_pemfile::private_key(&mut cursor)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse key: {}", e)))?
        .ok_or_else(|| TaleTrailError::TlsCertificateError(format!("No private key found in {}", path)))
}

/// Connect to NATS with TLS using the provided configuration
///
/// Note: Crypto provider must be initialized by calling `qollective::ensure_crypto_provider()`
/// at application startup before calling this function.
pub async fn connect_nats_with_tls(
    url: &str,
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<async_nats::Client> {
    // Load TLS certificates
    let ca_cert = load_cert(ca_cert_path)?;
    let client_cert = load_cert(client_cert_path)?;
    let client_key = load_key(client_key_path)?;

    // Build TLS configuration
    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add(ca_cert).map_err(|e| {
        TaleTrailError::TlsCertificateError(format!("Failed to add CA cert: {:?}", e))
    })?;

    let client_auth_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(vec![client_cert], client_key)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to build TLS config: {}", e)))?;

    // Connect to NATS with TLS
    let nats_client = async_nats::ConnectOptions::new()
        .tls_client_config(client_auth_config)
        .connect(url)
        .await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to connect to NATS: {}", e)))?;

    Ok(nats_client)
}
