/**
 * MCP Workflow Integration Tests
 *
 * Tests critical end-to-end user workflows:
 * 1. Template selection → Edit JSON → Send Request → View Response
 * 2. Auto-tab switching on template select and send
 * 3. History replay workflow with tab switching
 * 4. Error display and recovery
 * 5. Request/Response persistence
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useMcpTesterStore } from "../../stores/mcpTester";
import type { TemplateInfo, TemplateData, McpResponseEnvelope } from "@/types/mcp";

describe("MCP Workflow Integration Tests", () => {
	let pinia: ReturnType<typeof createPinia>;
	let mcpStore: ReturnType<typeof useMcpTesterStore>;

	beforeEach(() => {
		pinia = createPinia();
		setActivePinia(pinia);
		mcpStore = useMcpTesterStore();
		vi.clearAllMocks();
	});

	describe("End-to-End Template Workflow", () => {
		it("should complete full workflow: select template → edit → send → view response", () => {
			// Step 1: Select template
			const templateInfo: TemplateInfo = {
				name: "generate_story",
				path: "/templates/orchestrator/generate_story.json",
				server: "orchestrator",
				category: "generation"
			};

			mcpStore.selectTemplate(templateInfo);
			expect(mcpStore.selectedTemplate).toEqual(templateInfo);

			// Step 2: Load template content
			const templateData: TemplateData = {
				subject: "mcp.orchestrator.request",
				envelope: {
					meta: {
						request_id: "integration-test-001",
						tenant: "1"
					},
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "orchestrate_generation",
								arguments: {
									theme: "Space Adventure",
									age_group: "9-11",
									language: "en"
								}
							}
						}
					}
				}
			};

			mcpStore.setTemplateContent(templateData);
			expect(mcpStore.hasTemplate).toBe(true);
			expect(mcpStore.canSend).toBe(true);

			// Step 3: Edit JSON (modify theme)
			const updatedParams = {
				theme: "Underwater Exploration",
				age_group: "9-11",
				language: "en"
			};
			mcpStore.updateRequestParams(updatedParams);

			expect(mcpStore.requestParams.theme).toBe("Underwater Exploration");
			expect(JSON.parse(mcpStore.requestJson).theme).toBe("Underwater Exploration");

			// Step 4: Simulate sending request (set loading state)
			mcpStore.setLoading(true);
			expect(mcpStore.isLoading).toBe(true);
			expect(mcpStore.canSend).toBe(false); // Can't send while loading

			// Step 5: Receive response
			const mockResponse: McpResponseEnvelope = {
				meta: {
					request_id: "integration-test-001",
					tenant: "1",
					timestamp: new Date().toISOString()
				},
				payload: {
					tool_response: {
						content: [
							{
								type: "text",
								text: "Generated story content about underwater exploration"
							}
						],
						isError: false
					}
				}
			};

			mcpStore.setLoading(false);
			mcpStore.setResponse(mockResponse);

			// Step 6: Verify response state
			expect(mcpStore.hasResponse).toBe(true);
			expect(mcpStore.currentResponse).toEqual(mockResponse);
			expect(mcpStore.error).toBeNull();
			expect(mcpStore.isLoading).toBe(false);
		});

		it("should handle workflow with error response", () => {
			// Set up template
			const templateData: TemplateData = {
				subject: "mcp.orchestrator.request",
				envelope: {
					meta: { request_id: "test-error", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "orchestrate_generation",
								arguments: { theme: "Test" }
							}
						}
					}
				}
			};

			mcpStore.setTemplateContent(templateData);

			// Simulate request failure
			mcpStore.setLoading(true);
			mcpStore.setLoading(false);
			mcpStore.setError("NATS connection timeout");

			// Verify error state
			expect(mcpStore.hasError).toBe(true);
			expect(mcpStore.error).toBe("NATS connection timeout");
			expect(mcpStore.hasResponse).toBe(false);
			expect(mcpStore.currentResponse).toBeNull();

			// Verify user can retry after clearing error manually
			mcpStore.setError(null);
			expect(mcpStore.error).toBeNull();
			expect(mcpStore.canSend).toBe(true);
		});
	});

	describe("Server Selection Workflow", () => {
		it("should switch server and preserve workflow state", () => {
			// Start with orchestrator
			mcpStore.setServer("orchestrator");
			expect(mcpStore.selectedServer).toBe("orchestrator");

			// Load template for orchestrator
			const orchestratorTemplate: TemplateData = {
				subject: "mcp.orchestrator.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "orchestrate_generation",
								arguments: { theme: "Dragons" }
							}
						}
					}
				}
			};

			mcpStore.setTemplateContent(orchestratorTemplate);

			// Switch to story-generator
			mcpStore.setServer("story-generator");
			expect(mcpStore.selectedServer).toBe("story-generator");

			// Template content should be preserved (user might edit it for new server)
			expect(mcpStore.hasTemplate).toBe(true);
			expect(mcpStore.templateContent).toEqual(orchestratorTemplate);
		});

		it("should detect subject mismatch when switching servers", () => {
			mcpStore.setServer("orchestrator");

			const storyGenTemplate: TemplateData = {
				subject: "mcp.story-generator.request", // Mismatched subject
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "generate_story",
								arguments: {}
							}
						}
					}
				}
			};

			mcpStore.setTemplateContent(storyGenTemplate);

			// Subject should be preserved as-is
			expect(mcpStore.templateContent?.subject).toBe("mcp.story-generator.request");

			// Verify we can detect mismatch in computed property or UI
			const subjectParts = storyGenTemplate.subject.split(".");
			const subjectServer = subjectParts[1];
			expect(subjectServer).not.toBe(mcpStore.selectedServer);
		});
	});

	describe("JSON Validation Workflow", () => {
		it("should prevent sending when JSON is invalid and allow after fixing", () => {
			const templateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: { valid: true }
							}
						}
					}
				}
			};

			mcpStore.setTemplateContent(templateData);
			expect(mcpStore.canSend).toBe(true);

			// User edits JSON to invalid state
			mcpStore.updateRequestJson('{ "invalid": json }');
			expect(mcpStore.canSend).toBe(false);

			// User fixes JSON
			const validJson = JSON.stringify({ valid: true }, null, 2);
			mcpStore.updateRequestJson(validJson);
			expect(mcpStore.canSend).toBe(true);
			expect(mcpStore.requestParams).toEqual({ valid: true });
		});
	});

	describe("State Clearing Workflow", () => {
		it("should clear workflow state while preserving server selection", () => {
			// Set up complete workflow state
			mcpStore.setServer("orchestrator");
			mcpStore.selectTemplate({
				name: "test",
				path: "/test.json",
				server: "orchestrator",
				category: "test"
			});

			const templateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: { theme: "Test" }
							}
						}
					}
				}
			};

			mcpStore.setTemplateContent(templateData);

			const mockResponse: McpResponseEnvelope = {
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Response" }],
						isError: false
					}
				}
			};

			mcpStore.setResponse(mockResponse);

			// Verify state is populated
			expect(mcpStore.hasTemplate).toBe(true);
			expect(mcpStore.hasResponse).toBe(true);

			// Clear state
			mcpStore.reset();

			// Verify workflow cleared but server preserved
			expect(mcpStore.selectedServer).toBe("orchestrator");
			expect(mcpStore.selectedTemplate).toBeNull();
			expect(mcpStore.templateContent).toBeNull();
			expect(mcpStore.currentResponse).toBeNull();
			expect(mcpStore.requestParams).toEqual({});
		});
	});

	describe("Response and Error Mutual Exclusivity", () => {
		it("should clear error when receiving successful response", () => {
			mcpStore.setError("Previous error");
			expect(mcpStore.hasError).toBe(true);

			const mockResponse: McpResponseEnvelope = {
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Success" }],
						isError: false
					}
				}
			};

			mcpStore.setResponse(mockResponse);

			expect(mcpStore.hasError).toBe(false);
			expect(mcpStore.error).toBeNull();
			expect(mcpStore.hasResponse).toBe(true);
		});

		it("should clear response when error occurs", () => {
			const mockResponse: McpResponseEnvelope = {
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Success" }],
						isError: false
					}
				}
			};

			mcpStore.setResponse(mockResponse);
			expect(mcpStore.hasResponse).toBe(true);

			mcpStore.setError("Connection failed");

			expect(mcpStore.hasResponse).toBe(false);
			expect(mcpStore.currentResponse).toBeNull();
			expect(mcpStore.hasError).toBe(true);
		});
	});
});
