/**
 * Test Data Loader
 *
 * Loads real trail data from response_test_epic_de_1.json for integration testing.
 * This ensures all component tests use actual generated story data rather than mocks.
 */

import type { Trail, TrailListItem, TrailStep } from "../../../types/trails";
import testDataRaw from "../../../../test-trails/response_test_epic_de_1.json";
import { reconstructDAG } from "../../dagReconstruction";

/**
 * Fix empty next_node_id fields in choices
 *
 * TEMPORARY WORKAROUND: The test data was generated before the orchestrator fix
 * that properly populates next_node_id from DAG edges. This function creates a
 * simple branching structure as a fallback for testing.
 *
 * The backend orchestrator now correctly populates next_node_id values from the
 * DAG edge analysis (see orchestrator/src/orchestrator.rs:1000-1055). Once new
 * test data is generated with the fixed orchestrator, this function can be removed.
 *
 * TODO: Remove this function once new test data is generated with proper next_node_id values
 *
 * @param trail_steps - Array of trail steps with potentially empty next_node_id
 * @returns Fixed trail steps with populated next_node_id fields
 */
function fixEmptyNextNodeIds(trail_steps: TrailStep[]): TrailStep[] {
	const fixedSteps = [...trail_steps];
	let nextNodeIndex = 1; // Start assigning from node index 1

	for (let i = 0; i < fixedSteps.length; i++) {
		const step = fixedSteps[i];
		const choices = step.content_reference.content.choices || [];

		for (let j = 0; j < choices.length; j++) {
			// If next_node_id is empty or missing, assign the next available node
			if (!choices[j].next_node_id || choices[j].next_node_id === "") {
				if (nextNodeIndex < fixedSteps.length) {
					choices[j].next_node_id = fixedSteps[nextNodeIndex].content_reference.temp_node_id;
					nextNodeIndex++;
				}
				// If we run out of nodes, point to last node (creates end nodes)
			}
		}
	}

	return fixedSteps;
}

/**
 * Load real test trail data from response_test_epic_de_1.json
 *
 * Parses the nested JSON structure from the envelope response.
 * Structure: envelope.payload.tool_response.content[0].text contains JSON string
 * with generation_response object.
 *
 * NOTE: The test data was generated before the orchestrator fix that populates
 * next_node_id from DAG edges. We fix this programmatically to enable testing.
 *
 * @returns Trail data with trail, trail_steps, execution_trace, and metadata
 */
export function loadTestTrail() {
	// Parse the nested JSON structure
	const content = JSON.parse(testDataRaw.payload.tool_response.content[0].text);
	const generationResponse = content.generation_response;

	// Fix empty next_node_id fields for testing
	const fixedSteps = fixEmptyNextNodeIds(generationResponse.trail_steps);

	return {
		trail: generationResponse.trail,
		trail_steps: fixedSteps,
		execution_trace: generationResponse.execution_trace,
		generation_metadata: generationResponse.generation_metadata
	};
}

/**
 * Load test trail with reconstructed DAG
 *
 * Reconstructs the DAG structure from trail_steps array using the
 * dagReconstruction utility. This provides the complete graph structure
 * needed for interactive navigation and visualization.
 *
 * @returns Trail data with reconstructed DAG attached
 */
export function loadTestTrailWithDAG() {
	const data = loadTestTrail();
	const dag = reconstructDAG(data.trail_steps, data.trail.metadata.start_node_id);

	return {
		...data,
		trail: {
			...data.trail,
			dag
		} as Trail
	};
}

/**
 * Convert to TrailListItem format for TrailCard
 *
 * Creates a TrailListItem (list view format) from the full trail data.
 * Used for testing components that display trail cards/lists.
 *
 * @returns TrailListItem with metadata for card display
 */
export function loadTestTrailListItem(): TrailListItem {
	const data = loadTestTrail();

	return {
		id: "test-trail-epic-de-1",
		file_path: "/test-trails/response_test_epic_de_1.json",
		title: data.trail.title,
		description: data.trail.description || "Interactive story for 15-17 age group",
		theme: data.trail.metadata.generation_params.theme,
		age_group: data.trail.metadata.generation_params.age_group,
		language: data.trail.metadata.generation_params.language,
		tags: ["nature", "mediterranean", "flora", "fauna", "education"],
		status: "completed",
		generated_at: data.generation_metadata.generated_at,
		node_count: data.trail_steps.length,
		tenantId: "tenant-1"
	};
}

/**
 * Get test trail statistics for assertions
 *
 * Extracts key statistics from the test trail for use in test assertions.
 *
 * @returns Object with trail statistics
 */
export function getTestTrailStats() {
	const data = loadTestTrailWithDAG();

	return {
		nodeCount: data.trail_steps.length,
		edgeCount: data.trail.dag?.edges.length || 0,
		convergencePointCount: data.trail.dag?.convergence_points?.length || 0,
		startNodeId: data.trail.metadata.start_node_id,
		title: data.trail.title,
		theme: data.trail.metadata.generation_params.theme,
		ageGroup: data.trail.metadata.generation_params.age_group,
		language: data.trail.metadata.generation_params.language
	};
}
