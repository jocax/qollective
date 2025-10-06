//! Orchestrator configuration

use serde::{Deserialize, Serialize};
use shared_types::*;

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// NATS server URL
    pub nats_url: String,

    /// LM Studio URL
    pub lm_studio_url: String,

    /// NATS TLS CA certificate path
    pub nats_tls_ca_cert: String,

    /// NATS TLS client certificate path
    pub nats_tls_client_cert: String,

    /// NATS TLS client key path
    pub nats_tls_client_key: String,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            nats_url: NATS_URL.clone(),
            lm_studio_url: LM_STUDIO_URL.clone(),
            nats_tls_ca_cert: NATS_TLS_CA_CERT_PATH.clone(),
            nats_tls_client_cert: NATS_TLS_CLIENT_CERT_PATH.clone(),
            nats_tls_client_key: NATS_TLS_CLIENT_KEY_PATH.clone(),
        }
    }
}
