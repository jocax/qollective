import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useMcpTesterStore } from "../mcpTester";
import type { GroupedTemplates, TemplateInfo, TemplateData, McpResponseEnvelope } from "@/types/mcp";

describe("useMcpTesterStore - Cross-Cutting Concerns", () => {
	beforeEach(() => {
		// Create a fresh pinia instance before each test
		setActivePinia(createPinia());
	});

	describe("State Persistence Across Navigation", () => {
		it("should maintain selected template when store is accessed multiple times", () => {
			// Simulate accessing store from different components/pages
			const store1 = useMcpTesterStore();
			const store2 = useMcpTesterStore();

			const mockTemplate: TemplateInfo = {
				name: "test-template",
				path: "/path/to/template.json",
				server: "orchestrator",
				category: "test"
			};

			// Set template in one instance
			store1.selectTemplate(mockTemplate);

			// Verify it's accessible from another instance (same store)
			expect(store2.selectedTemplate).toEqual(mockTemplate);
			expect(store2.selectedTemplate).toBe(store1.selectedTemplate);
		});

		it("should preserve request JSON across store access", () => {
			const store1 = useMcpTesterStore();
			const store2 = useMcpTesterStore();

			const mockRequestJson = JSON.stringify({
				theme: "Space Adventure",
				age_group: "9-11",
				language: "en"
			}, null, 2);

			store1.updateRequestJson(mockRequestJson);

			expect(store2.requestJson).toBe(mockRequestJson);
			expect(store2.requestParams).toEqual(JSON.parse(mockRequestJson));
		});

		it("should persist server selection across navigation", () => {
			const store1 = useMcpTesterStore();
			const store2 = useMcpTesterStore();

			store1.setServer("prompt-helper");

			expect(store2.selectedServer).toBe("prompt-helper");
		});
	});

	describe("Store Actions Update State Correctly", () => {
		it("should update template content and sync request params", () => {
			const store = useMcpTesterStore();

			const mockTemplateData: TemplateData = {
				subject: "mcp.orchestrator.request",
				envelope: {
					meta: {
						request_id: "test-123",
						tenant: "1"
					},
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "orchestrate_generation",
								arguments: {
									theme: "Dragons",
									age_group: "6-8",
									language: "en"
								}
							}
						}
					}
				}
			};

			store.setTemplateContent(mockTemplateData);

			expect(store.templateContent).toEqual(mockTemplateData);
			expect(store.requestParams).toEqual({
				theme: "Dragons",
				age_group: "6-8",
				language: "en"
			});
			expect(store.requestJson).toBeTruthy();
			expect(JSON.parse(store.requestJson)).toEqual(store.requestParams);
		});

		it("should update response and clear error on success", () => {
			const store = useMcpTesterStore();

			// Set an error first
			store.setError("Previous error");
			expect(store.error).toBe("Previous error");
			expect(store.hasError).toBe(true);

			const mockResponse: McpResponseEnvelope = {
				meta: {
					request_id: "test-456",
					tenant: "1"
				},
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Success" }],
						isError: false
					}
				}
			};

			store.setResponse(mockResponse);

			expect(store.currentResponse).toEqual(mockResponse);
			expect(store.error).toBeNull();
			expect(store.hasError).toBe(false);
			expect(store.hasResponse).toBe(true);
		});

		it("should update error and clear response on failure", () => {
			const store = useMcpTesterStore();

			// Set a response first
			const mockResponse: McpResponseEnvelope = {
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Previous success" }],
						isError: false
					}
				}
			};
			store.setResponse(mockResponse);

			expect(store.hasResponse).toBe(true);

			// Now set error
			store.setError("Connection failed");

			expect(store.error).toBe("Connection failed");
			expect(store.currentResponse).toBeNull();
			expect(store.hasError).toBe(true);
			expect(store.hasResponse).toBe(false);
		});

		it("should sync requestParams with requestJson bidirectionally", () => {
			const store = useMcpTesterStore();

			const params = {
				theme: "Ocean",
				age_group: "12-15",
				language: "de"
			};

			// Update params -> should update JSON
			store.updateRequestParams(params);
			expect(store.requestJson).toBe(JSON.stringify(params, null, 2));

			// Update JSON -> should update params
			const newJson = JSON.stringify({ theme: "Mountains" }, null, 2);
			store.updateRequestJson(newJson);
			expect(store.requestParams).toEqual({ theme: "Mountains" });
		});

		it("should handle invalid JSON in updateRequestJson gracefully", () => {
			const store = useMcpTesterStore();

			const validParams = { theme: "Valid" };
			store.updateRequestParams(validParams);

			const invalidJson = "{ invalid json }";
			store.updateRequestJson(invalidJson);

			// requestJson should update, but requestParams should remain unchanged
			expect(store.requestJson).toBe(invalidJson);
			expect(store.requestParams).toEqual(validParams);
		});
	});

	describe("Computed Properties Reflect State Changes", () => {
		it("canSend should be true when template and valid JSON exist", () => {
			const store = useMcpTesterStore();

			expect(store.canSend).toBe(false);

			const mockTemplateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: { key: "value" }
							}
						}
					}
				}
			};

			store.setTemplateContent(mockTemplateData);
			expect(store.canSend).toBe(true);
		});

		it("canSend should be false when JSON is invalid", () => {
			const store = useMcpTesterStore();

			const mockTemplateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: {}
							}
						}
					}
				}
			};

			store.setTemplateContent(mockTemplateData);
			expect(store.canSend).toBe(true);

			// Break the JSON
			store.updateRequestJson("{ invalid }");
			expect(store.canSend).toBe(false);
		});

		it("canSend should be false when loading", () => {
			const store = useMcpTesterStore();

			const mockTemplateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: { key: "value" }
							}
						}
					}
				}
			};

			store.setTemplateContent(mockTemplateData);
			expect(store.canSend).toBe(true);

			store.setLoading(true);
			expect(store.canSend).toBe(false);
		});

		it("hasTemplate, hasResponse, hasError should reflect current state", () => {
			const store = useMcpTesterStore();

			expect(store.hasTemplate).toBe(false);
			expect(store.hasResponse).toBe(false);
			expect(store.hasError).toBe(false);

			const mockTemplateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: {}
							}
						}
					}
				}
			};

			store.setTemplateContent(mockTemplateData);
			expect(store.hasTemplate).toBe(true);

			const mockResponse: McpResponseEnvelope = {
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Result" }],
						isError: false
					}
				}
			};

			store.setCurrentResponse(mockResponse);
			expect(store.hasResponse).toBe(true);

			store.setError("Test error");
			expect(store.hasError).toBe(true);
			expect(store.hasResponse).toBe(false); // Error clears response
		});
	});

	describe("Store Clearing and Reset", () => {
		it("should clear all state when clearState is called", () => {
			const store = useMcpTesterStore();

			// Populate store
			store.selectTemplate({
				name: "test",
				path: "/test",
				server: "orchestrator",
				category: "test"
			});
			store.updateRequestJson('{"test": "data"}');
			store.setError("Test error");
			store.setLoading(true);

			// Clear state
			store.clearState();

			expect(store.selectedTemplate).toBeNull();
			expect(store.templateContent).toBeNull();
			expect(store.requestParams).toEqual({});
			expect(store.requestJson).toBe("");
			expect(store.currentRequest).toBeNull();
			expect(store.currentResponse).toBeNull();
			expect(store.error).toBeNull();
			expect(store.isLoading).toBe(false);
		});

		it("should preserve server selection when reset is called", () => {
			const store = useMcpTesterStore();

			store.setServer("prompt-helper");
			store.updateRequestJson('{"test": "data"}');

			store.reset();

			expect(store.selectedServer).toBe("prompt-helper"); // Preserved
			expect(store.requestJson).toBe(""); // Cleared
		});

		it("should clear template when clearTemplate is called", () => {
			const store = useMcpTesterStore();

			const mockTemplateData: TemplateData = {
				subject: "mcp.test.request",
				envelope: {
					meta: { request_id: "test", tenant: "1" },
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "test_tool",
								arguments: { key: "value" }
							}
						}
					}
				}
			};

			store.selectTemplate({
				name: "test",
				path: "/test",
				server: "orchestrator",
				category: "test"
			});
			store.setTemplateContent(mockTemplateData);

			expect(store.hasTemplate).toBe(true);

			store.clearTemplate();

			expect(store.selectedTemplate).toBeNull();
			expect(store.templateContent).toBeNull();
			expect(store.templateSchema).toBeNull();
			expect(store.requestParams).toEqual({});
			expect(store.requestJson).toBe("");
			expect(store.hasTemplate).toBe(false);
		});
	});

	describe("NATS Envelope Structure Preservation", () => {
		it("should preserve envelope metadata in template content", () => {
			const store = useMcpTesterStore();

			const mockTemplateData: TemplateData = {
				subject: "mcp.orchestrator.request",
				envelope: {
					meta: {
						request_id: "unique-request-123",
						tenant: "42",
						tracing: {
							trace_id: "trace-abc-123",
							span_id: "span-xyz-456"
						},
						timestamp: "2025-11-02T10:00:00Z"
					},
					payload: {
						tool_call: {
							method: "tools/call",
							params: {
								name: "orchestrate_generation",
								arguments: {
									theme: "Test Theme"
								}
							}
						}
					}
				}
			};

			store.setTemplateContent(mockTemplateData);

			// Verify envelope structure is preserved
			expect(store.templateContent?.envelope.meta.request_id).toBe("unique-request-123");
			expect(store.templateContent?.envelope.meta.tenant).toBe("42");
			expect(store.templateContent?.envelope.meta.tracing?.trace_id).toBe("trace-abc-123");
			expect(store.templateContent?.envelope.meta.tracing?.span_id).toBe("span-xyz-456");
			expect(store.templateContent?.envelope.meta.timestamp).toBe("2025-11-02T10:00:00Z");
			expect(store.templateContent?.subject).toBe("mcp.orchestrator.request");
		});

		it("should preserve response envelope structure", () => {
			const store = useMcpTesterStore();

			const mockResponse: McpResponseEnvelope = {
				meta: {
					request_id: "req-789",
					tenant: "10",
					tracing: {
						trace_id: "response-trace-abc",
						span_id: "response-span-xyz"
					},
					timestamp: "2025-11-02T11:00:00Z",
					correlation_id: "corr-123"
				},
				payload: {
					tool_response: {
						content: [
							{
								type: "text",
								text: "Generated content"
							}
						],
						isError: false
					}
				}
			};

			store.setResponse(mockResponse);

			// Verify full envelope is preserved
			expect(store.currentResponse?.meta.request_id).toBe("req-789");
			expect(store.currentResponse?.meta.tenant).toBe("10");
			expect(store.currentResponse?.meta.tracing?.trace_id).toBe("response-trace-abc");
			expect(store.currentResponse?.meta.correlation_id).toBe("corr-123");
			expect(store.currentResponse?.payload.tool_response?.isError).toBe(false);
		});
	});
});
