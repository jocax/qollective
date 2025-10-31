/**
 * Unit tests for DAG Reconstruction Utility
 *
 * Tests validation logic and edge case handling for reconstructing DAG
 * structures from trail_steps arrays.
 */

import type { DAG, TrailStep } from "~/types/trails";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { reconstructDAG, validateDAG } from "../dagReconstruction";

/**
 * Helper function to create a trail step with content
 */
function createTrailStep(
	nodeId: string,
	text: string,
	choices: Array<{ id: string, text: string, next_node_id: string }>,
	stepOrder: number,
	metadata?: Record<string, any>
): TrailStep {
	return {
		step_order: stepOrder,
		is_required: true,
		metadata: metadata || {},
		content_reference: {
			temp_node_id: nodeId,
			content: {
				type: "interactive_story_node",
				text,
				choices,
				educational_content: {
					topic: "Test Topic",
					vocabulary_words: ["test", "example"],
					learning_objectives: ["Learn testing"]
				}
			}
		}
	};
}

describe("dAG Reconstruction - Valid Scenarios", () => {
	describe("test 1: Valid Linear Story", () => {
		let trailSteps: TrailStep[];
		let reconstructedDAG: DAG;

		beforeEach(() => {
			// Create a simple 3-node linear story
			trailSteps = [
				createTrailStep("node1", "Beginning of the story", [
					{ id: "choice1", text: "Continue", next_node_id: "node2" }
				], 1),
				createTrailStep("node2", "Middle of the story", [
					{ id: "choice2", text: "Continue", next_node_id: "node3" }
				], 2),
				createTrailStep("node3", "End of the story", [], 3)
			];

			reconstructedDAG = reconstructDAG(trailSteps, "node1");
		});

		it("should reconstruct all nodes correctly", () => {
			expect(Object.keys(reconstructedDAG.nodes)).toHaveLength(3);
			expect(reconstructedDAG.nodes.node1).toBeDefined();
			expect(reconstructedDAG.nodes.node2).toBeDefined();
			expect(reconstructedDAG.nodes.node3).toBeDefined();
		});

		it("should create correct edges for linear progression", () => {
			expect(reconstructedDAG.edges).toHaveLength(2);
			expect(reconstructedDAG.edges).toContainEqual({
				from_node_id: "node1",
				to_node_id: "node2",
				choice_id: "choice1"
			});
			expect(reconstructedDAG.edges).toContainEqual({
				from_node_id: "node2",
				to_node_id: "node3",
				choice_id: "choice2"
			});
		});

		it("should set correct start_node_id", () => {
			expect(reconstructedDAG.start_node_id).toBe("node1");
		});

		it("should preserve node content", () => {
			expect(reconstructedDAG.nodes.node1.content.text).toBe("Beginning of the story");
			expect(reconstructedDAG.nodes.node2.content.text).toBe("Middle of the story");
			expect(reconstructedDAG.nodes.node3.content.text).toBe("End of the story");
		});

		it("should preserve choices in nodes", () => {
			expect(reconstructedDAG.nodes.node1.content.choices).toHaveLength(1);
			expect(reconstructedDAG.nodes.node1.content.choices![0].next_node_id).toBe("node2");
			expect(reconstructedDAG.nodes.node2.content.choices).toHaveLength(1);
			expect(reconstructedDAG.nodes.node3.content.choices).toHaveLength(0);
		});
	});

	describe("test 2: Valid Branching Story", () => {
		let trailSteps: TrailStep[];
		let reconstructedDAG: DAG;

		beforeEach(() => {
			// Create a branching story with multiple choices
			trailSteps = [
				createTrailStep("node1", "You are at a crossroads", [
					{ id: "choice1", text: "Go left", next_node_id: "node2" },
					{ id: "choice2", text: "Go right", next_node_id: "node3" },
					{ id: "choice3", text: "Go straight", next_node_id: "node4" }
				], 1),
				createTrailStep("node2", "You went left", [], 2),
				createTrailStep("node3", "You went right", [], 3),
				createTrailStep("node4", "You went straight", [], 4)
			];

			reconstructedDAG = reconstructDAG(trailSteps, "node1");
		});

		it("should create all edges for branching choices", () => {
			expect(reconstructedDAG.edges).toHaveLength(3);
			expect(reconstructedDAG.edges).toContainEqual({
				from_node_id: "node1",
				to_node_id: "node2",
				choice_id: "choice1"
			});
			expect(reconstructedDAG.edges).toContainEqual({
				from_node_id: "node1",
				to_node_id: "node3",
				choice_id: "choice2"
			});
			expect(reconstructedDAG.edges).toContainEqual({
				from_node_id: "node1",
				to_node_id: "node4",
				choice_id: "choice3"
			});
		});

		it("should verify all edge targets exist as nodes", () => {
			reconstructedDAG.edges.forEach((edge) => {
				expect(reconstructedDAG.nodes[edge.to_node_id]).toBeDefined();
				expect(reconstructedDAG.nodes[edge.from_node_id]).toBeDefined();
			});
		});

		it("should preserve multiple choices in the branching node", () => {
			expect(reconstructedDAG.nodes.node1.content.choices).toHaveLength(3);
			const choiceIds = reconstructedDAG.nodes.node1.content.choices!.map((c) => c.id);
			expect(choiceIds).toContain("choice1");
			expect(choiceIds).toContain("choice2");
			expect(choiceIds).toContain("choice3");
		});

		it("should map choices to correct target nodes", () => {
			const node1 = reconstructedDAG.nodes.node1;
			const choice1 = node1.content.choices!.find((c) => c.id === "choice1");
			const choice2 = node1.content.choices!.find((c) => c.id === "choice2");
			const choice3 = node1.content.choices!.find((c) => c.id === "choice3");

			expect(choice1?.next_node_id).toBe("node2");
			expect(choice2?.next_node_id).toBe("node3");
			expect(choice3?.next_node_id).toBe("node4");
		});
	});

	describe("test 4: Convergence Points", () => {
		let trailSteps: TrailStep[];
		let reconstructedDAG: DAG;

		beforeEach(() => {
			// Create a story with convergence point
			trailSteps = [
				createTrailStep("node1", "Start", [
					{ id: "choice1", text: "Path A", next_node_id: "node2" },
					{ id: "choice2", text: "Path B", next_node_id: "node3" }
				], 1),
				createTrailStep("node2", "Path A content", [
					{ id: "choice3", text: "Continue", next_node_id: "node4" }
				], 2, { outgoing_edges: 1 }),
				createTrailStep("node3", "Path B content", [
					{ id: "choice4", text: "Continue", next_node_id: "node4" }
				], 3, { outgoing_edges: 1 }),
				createTrailStep("node4", "Convergence point", [], 4, {
					convergence_point: true,
					incoming_edges: 2
				})
			];

			reconstructedDAG = reconstructDAG(trailSteps, "node1");
		});

		it("should identify convergence points", () => {
			expect(reconstructedDAG.convergence_points).toContain("node4");
		});

		it("should create multiple incoming edges to convergence point", () => {
			const incomingEdges = reconstructedDAG.edges.filter((e) => e.to_node_id === "node4");
			expect(incomingEdges).toHaveLength(2);
			expect(incomingEdges.map((e) => e.from_node_id)).toContain("node2");
			expect(incomingEdges.map((e) => e.from_node_id)).toContain("node3");
		});

		it("should preserve incoming_edges count in metadata", () => {
			expect(reconstructedDAG.nodes.node4.incoming_edges).toBe(2);
		});

		it("should preserve convergence_point metadata", () => {
			expect(reconstructedDAG.convergence_points).toBeDefined();
			expect(reconstructedDAG.convergence_points!.length).toBeGreaterThan(0);
		});
	});
});

describe("dAG Reconstruction - Error Handling", () => {
	describe("test 3: Missing next_node_id (Error Case)", () => {
		let trailSteps: TrailStep[];
		let consoleWarnSpy: any;

		beforeEach(() => {
			// Spy on console.warn to capture validation warnings
			consoleWarnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

			// Create a story with a choice missing next_node_id
			trailSteps = [
				createTrailStep("node1", "Start", [
					{ id: "choice1", text: "Good choice", next_node_id: "node2" },
					{ id: "choice2", text: "Bad choice", next_node_id: "" } // Empty next_node_id
				], 1),
				createTrailStep("node2", "Valid destination", [], 2)
			];
		});

		afterEach(() => {
			consoleWarnSpy.mockRestore();
		});

		it("should log warning for missing next_node_id", () => {
			reconstructDAG(trailSteps, "node1");

			expect(consoleWarnSpy).toHaveBeenCalled();
			const warnCalls = consoleWarnSpy.mock.calls;
			const validationWarning = warnCalls.find((call: any[]) =>
				call[0].includes("Invalid choice")
			);
			expect(validationWarning).toBeDefined();
		});

		it("should still create the node (graceful degradation)", () => {
			const dag = reconstructDAG(trailSteps, "node1");

			expect(dag.nodes.node1).toBeDefined();
			expect(dag.nodes.node2).toBeDefined();
		});

		it("should only create edge for valid choice", () => {
			const dag = reconstructDAG(trailSteps, "node1");

			// Should only have one edge (for the valid choice)
			expect(dag.edges).toHaveLength(1);
			expect(dag.edges[0]).toEqual({
				from_node_id: "node1",
				to_node_id: "node2",
				choice_id: "choice1"
			});
		});

		it("should preserve both choices in node content", () => {
			const dag = reconstructDAG(trailSteps, "node1");

			// Both choices should still be in the node content
			expect(dag.nodes.node1.content.choices).toHaveLength(2);
		});

		it("should handle choice with missing choice.id", () => {
			const invalidSteps = [
				{
					step_order: 1,
					is_required: true,
					metadata: {},
					content_reference: {
						temp_node_id: "node1",
						content: {
							type: "interactive_story_node",
							text: "Test",
							choices: [
								{ id: "", text: "No ID", next_node_id: "node2" } // Missing id
							]
						}
					}
				} as TrailStep
			];

			const dag = reconstructDAG(invalidSteps, "node1");

			expect(consoleWarnSpy).toHaveBeenCalled();
			expect(dag.edges).toHaveLength(0); // No edge created
		});
	});

	describe("empty and Invalid Input Handling", () => {
		it("should throw error for empty trail_steps", () => {
			expect(() => reconstructDAG([], "node1")).toThrow(
				"Cannot reconstruct DAG: trail_steps array is empty"
			);
		});

		it("should throw error for missing start_node_id", () => {
			const steps = [createTrailStep("node1", "Test", [], 1)];
			expect(() => reconstructDAG(steps, "")).toThrow(
				"Cannot reconstruct DAG: start_node_id is missing"
			);
		});

		it("should handle steps with missing content_reference", () => {
			const consoleWarnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

			const invalidSteps = [
				{
					step_order: 1,
					is_required: true,
					metadata: {},
					content_reference: null as any
				} as TrailStep,
				createTrailStep("node1", "Valid", [], 2)
			];

			const dag = reconstructDAG(invalidSteps, "node1");

			expect(consoleWarnSpy).toHaveBeenCalled();
			expect(Object.keys(dag.nodes)).toHaveLength(1); // Only valid node
			expect(dag.nodes.node1).toBeDefined();

			consoleWarnSpy.mockRestore();
		});
	});
});

describe("dAG Validation", () => {
	describe("validateDAG function", () => {
		it("should validate a correct linear DAG", () => {
			const steps = [
				createTrailStep("node1", "Start", [
					{ id: "c1", text: "Next", next_node_id: "node2" }
				], 1),
				createTrailStep("node2", "End", [], 2)
			];

			const dag = reconstructDAG(steps, "node1");
			const validation = validateDAG(dag);

			expect(validation.valid).toBe(true);
			expect(validation.stats.nodeCount).toBe(2);
			expect(validation.stats.edgeCount).toBe(1);
		});

		it("should detect missing start node", () => {
			const steps = [
				createTrailStep("node1", "Test", [], 1)
			];

			const dag = reconstructDAG(steps, "node1");
			dag.start_node_id = "nonexistent";

			const validation = validateDAG(dag);

			expect(validation.valid).toBe(false);
			expect(validation.warnings).toContain("Start node \"nonexistent\" not found in DAG nodes");
		});

		it("should detect orphan nodes", () => {
			const steps = [
				createTrailStep("node1", "Start", [], 1),
				createTrailStep("node2", "Orphan", [], 2)
			];

			const dag = reconstructDAG(steps, "node1");
			const validation = validateDAG(dag);

			expect(validation.stats.orphanNodes).toBe(1);
			expect(validation.warnings.some((w) => w.includes("orphan"))).toBe(true);
		});

		it("should count dead end nodes", () => {
			const steps = [
				createTrailStep("node1", "Start", [
					{ id: "c1", text: "Go", next_node_id: "node2" }
				], 1),
				createTrailStep("node2", "Dead end", [], 2)
			];

			const dag = reconstructDAG(steps, "node1");
			const validation = validateDAG(dag);

			expect(validation.stats.deadEndNodes).toBe(1); // node2 has no outgoing edges
		});
	});
});

describe("educational Content Preservation", () => {
	it("should preserve educational_content in generation_metadata", () => {
		const steps = [
			createTrailStep("node1", "Educational content", [], 1)
		];

		const dag = reconstructDAG(steps, "node1");

		expect(dag.nodes.node1.generation_metadata).toBeDefined();
		expect(dag.nodes.node1.generation_metadata?.topic).toBe("Test Topic");
		expect(dag.nodes.node1.generation_metadata?.vocabulary_words).toContain("test");
		expect(dag.nodes.node1.generation_metadata?.learning_objectives).toHaveLength(1);
	});

	it("should handle nodes without educational content", () => {
		const stepWithoutEducation: TrailStep = {
			step_order: 1,
			is_required: true,
			metadata: {},
			content_reference: {
				temp_node_id: "node1",
				content: {
					type: "interactive_story_node",
					text: "No education",
					choices: []
				}
			}
		};

		const dag = reconstructDAG([stepWithoutEducation], "node1");

		expect(dag.nodes.node1).toBeDefined();
		expect(dag.nodes.node1.generation_metadata).toBeUndefined();
	});
});

/**
 * Integration test scenarios to manually verify:
 *
 * 1. Load a trail generated by the backend and verify it reconstructs correctly
 * 2. Verify validation warnings appear in console for malformed data
 * 3. Test with real trail_steps from backend service invocations
 * 4. Verify convergence points are correctly identified in complex stories
 * 5. Test with epic story structures (50+ nodes)
 * 6. Verify graceful degradation with partially invalid data
 */
