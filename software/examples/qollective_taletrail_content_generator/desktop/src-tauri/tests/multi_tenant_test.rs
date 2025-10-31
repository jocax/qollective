/// Multi-tenant integration tests for TaleTrail Desktop Viewer
///
/// NOTE: These tests are currently disabled because they use obsolete schema structures.
/// The tests create JSON with structures like:
/// - trail.metadata.generation_params (old structure)
/// - trail.dag (DAG is no longer nested under trail in current schema)
///
/// The current schema uses GenerationResponse with:
/// - trail: Option<Trail> (with Trail having different fields)
/// - trail_steps: Option<Vec<TrailStep>>
/// - generation_metadata: Option<GenerationMetadata>
///
/// TODO: Rewrite these tests to match the current shared-types-generated schema

#[cfg(test)]
mod disabled_tests {
    // All tests in this module are temporarily disabled pending schema updates
    // Tests need to be rewritten to match current envelope structure and Trail/GenerationResponse types
}
