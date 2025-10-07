//! Request mapper trait for gateway API transformation
//!
//! Abstracts gateway mapping logic to enable:
//! - External API (simple) → Internal API (rich) transformation
//! - Internal API → External API (DB-ready) reverse transformation
//! - Age-appropriate defaults application
//! - Mock-based testing of mapping rules
//!
//! # Example Usage
//!
//! ```rust,ignore
//! // Production config-based mapper
//! let mapper = ConfigBasedMapper::from_config(&gateway_config)?;
//! let internal = mapper.external_to_internal(external_req, tenant_id)?;
//! // Enrichment applied: node_count=16, vocabulary_level="basic", etc.
//!
//! // Testing with mock
//! let mut mock_mapper = MockRequestMapper::new();
//! mock_mapper
//!     .expect_external_to_internal()
//!     .returning(|ext, tid| Ok(enriched_request(ext, tid)));
//! ```

use uuid::Uuid;

use crate::errors::TaleTrailError;
use crate::generated::external_api::{
    ExternalGenerationRequestV1,
    ExternalGenerationResponseV1,
};
use crate::generated::internal_api::{
    GenerationRequest,
    GenerationResponse,
};
use crate::generated::gateway::GatewayMappingConfig;

/// Abstracts gateway API mapping logic
///
/// **Note**: This is a synchronous trait (no #[async_trait])
/// because mapping is pure transformation logic without I/O.
///
/// Implementations:
/// - `ConfigBasedMapper`: Production mapper using gateway config.toml
/// - `MockMapper`: Test mock with hardcoded mapping rules
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
pub trait RequestMapper: Send + Sync + std::fmt::Debug {
    /// Map external request to internal (with age-appropriate enrichment)
    ///
    /// Applies transformation:
    /// 1. Validate external request structure
    /// 2. Load age-appropriate defaults from config
    /// 3. Enrich with node_count, vocabulary_level, educational_goals, etc.
    /// 4. Add tenant_id from JWT claims
    ///
    /// # Arguments
    /// * `external` - Simple external request (theme, age_group, language)
    /// * `tenant_id` - Tenant ID extracted from JWT
    ///
    /// # Returns
    /// Rich internal `GenerationRequest` with all parameters
    ///
    /// # Errors
    /// - `TaleTrailError::ValidationError`: Invalid external request
    /// - `TaleTrailError::ConfigError`: Failed to load age defaults
    fn external_to_internal(
        &self,
        external: ExternalGenerationRequestV1,
        tenant_id: i64,
    ) -> Result<GenerationRequest, TaleTrailError>;

    /// Map internal response to external (DB-ready format)
    ///
    /// Applies reverse transformation:
    /// 1. Extract Trail and TrailStep data from DAG
    /// 2. Convert to Supabase-compatible format (TrailInsertData, TrailStepInsertData)
    /// 3. Wrap in ExternalGenerationResponseV1
    ///
    /// # Arguments
    /// * `internal` - Internal GenerationResponse with DAG
    /// * `job_id` - Job tracking UUID
    ///
    /// # Returns
    /// External response with DB-ready Trail and TrailStep structures
    ///
    /// # Errors
    /// - `TaleTrailError::MappingError`: Failed to convert DAG to Trail
    fn internal_to_external(
        &self,
        internal: GenerationResponse,
        job_id: Uuid,
    ) -> Result<ExternalGenerationResponseV1, TaleTrailError>;

    /// Get mapping configuration
    ///
    /// # Returns
    /// Reference to gateway mapping configuration
    fn get_config(&self) -> &GatewayMappingConfig;

    /// Validate external request before mapping
    ///
    /// Checks:
    /// - Required fields present (theme, age_group, language)
    /// - Valid age_group value ("3-5", "6-8", "9-12")
    /// - Valid language code ("en", "de")
    ///
    /// # Arguments
    /// * `external` - External request to validate
    ///
    /// # Returns
    /// `Ok(())` if valid
    ///
    /// # Errors
    /// - `TaleTrailError::ValidationError`: Invalid field values
    fn validate_external(&self, external: &ExternalGenerationRequestV1) -> Result<(), TaleTrailError>;
}
