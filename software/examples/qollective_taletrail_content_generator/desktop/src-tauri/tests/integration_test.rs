// NOTE: These tests were written for an older version of the schema and are currently disabled
// Most types tested here (TrailMetadata, GenerationParams, NodeContent, etc.) no longer exist
// in the current shared-types-generated schema. These tests need to be rewritten to match
// the current schema structure.

// The current schema uses:
// - GenerationRequest (with enums for age_group, language, vocabulary_level)
// - GenerationResponse (with Trail, TrailStep, GenerationMetadata, etc.)
// - DAG structure with ContentNode having incoming_edges/outgoing_edges fields
// - Edge with weight field
// - ServiceInvocation with different required fields (batch_id, node_id, started_at, etc.)

#[cfg(test)]
mod disabled_tests {
    // All tests in this module are temporarily disabled pending schema updates
    // TODO: Rewrite tests to match current shared-types-generated schema
}
