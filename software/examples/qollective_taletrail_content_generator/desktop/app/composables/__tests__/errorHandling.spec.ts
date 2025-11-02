import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useMcpTesterStore } from "../../stores/mcpTester";

/**
 * Cross-Cutting Error Handling Tests
 *
 * Tests error handling behaviors across the application including:
 * - Error message display
 * - Failed request handling
 * - Response viewer error states
 * - Toast notifications for file operations
 */
describe("Error Handling - Cross-Cutting Concerns", () => {
	beforeEach(() => {
		setActivePinia(createPinia());
	});

	describe("Error Messages Display with Clear Text", () => {
		it("should set clear error message in store", () => {
			const store = useMcpTesterStore();

			const errorMessage = "Failed to connect to NATS server at nats://localhost:4222";
			store.setError(errorMessage);

			expect(store.error).toBe(errorMessage);
			expect(store.hasError).toBe(true);
		});

		it("should clear error message when null is provided", () => {
			const store = useMcpTesterStore();

			store.setError("Previous error");
			expect(store.hasError).toBe(true);

			store.setError(null);

			expect(store.error).toBeNull();
			expect(store.hasError).toBe(false);
		});

		it("should clear response when error is set", () => {
			const store = useMcpTesterStore();

			// Set a successful response first
			store.setResponse({
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Success" }],
						isError: false
					}
				}
			});

			expect(store.hasResponse).toBe(true);

			// Set error - should clear response
			store.setError("Network timeout");

			expect(store.hasResponse).toBe(false);
			expect(store.currentResponse).toBeNull();
		});
	});

	describe("Failed Requests Show Error in Response Viewer", () => {
		it("should preserve error state for display in response viewer", () => {
			const store = useMcpTesterStore();

			const errorDetails = "Tool 'orchestrate_generation' failed: Invalid argument 'theme'";
			store.setError(errorDetails);

			// Simulate response viewer reading error state
			expect(store.error).toBe(errorDetails);
			expect(store.hasError).toBe(true);
			expect(store.hasResponse).toBe(false);
		});

		it("should handle error responses with isError flag", () => {
			const store = useMcpTesterStore();

			const errorResponse = {
				meta: { request_id: "error-req", tenant: "1" },
				payload: {
					tool_response: {
						content: [
							{
								type: "text",
								text: "Tool execution failed: Missing required parameter"
							}
						],
						isError: true
					}
				}
			};

			store.setResponse(errorResponse);

			expect(store.hasResponse).toBe(true);
			expect(store.currentResponse?.payload.tool_response?.isError).toBe(true);
		});
	});

	describe("Loading States During Request Execution", () => {
		it("should set loading state during request", () => {
			const store = useMcpTesterStore();

			expect(store.isLoading).toBe(false);
			expect(store.isLoadingResponse).toBe(false);

			store.setLoading(true);
			store.setLoadingResponse(true);

			expect(store.isLoading).toBe(true);
			expect(store.isLoadingResponse).toBe(true);
		});

		it("should clear loading state after request completes", () => {
			const store = useMcpTesterStore();

			store.setLoading(true);
			store.setLoadingResponse(true);

			// Simulate request completion
			store.setResponse({
				meta: { request_id: "test", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Result" }],
						isError: false
					}
				}
			});

			store.setLoading(false);
			store.setLoadingResponse(false);

			expect(store.isLoading).toBe(false);
			expect(store.isLoadingResponse).toBe(false);
		});

		it("should clear loading state after request fails", () => {
			const store = useMcpTesterStore();

			store.setLoading(true);
			store.setLoadingResponse(true);

			// Simulate request failure
			store.setError("Request timeout after 30000ms");

			store.setLoading(false);
			store.setLoadingResponse(false);

			expect(store.isLoading).toBe(false);
			expect(store.isLoadingResponse).toBe(false);
			expect(store.hasError).toBe(true);
		});
	});

	describe("Error Recovery and State Reset", () => {
		it("should allow clearing error and starting fresh", () => {
			const store = useMcpTesterStore();

			// Simulate failed state
			store.setError("Connection failed");
			store.setLoading(true);

			expect(store.hasError).toBe(true);

			// Clear error and reset loading
			store.setError(null);
			store.setLoading(false);

			expect(store.hasError).toBe(false);
			expect(store.isLoading).toBe(false);
		});

		it("should clear error when new successful response arrives", () => {
			const store = useMcpTesterStore();

			store.setError("Previous request failed");
			expect(store.hasError).toBe(true);

			// New successful response should clear error
			store.setResponse({
				meta: { request_id: "new-req", tenant: "1" },
				payload: {
					tool_response: {
						content: [{ type: "text", text: "Success" }],
						isError: false
					}
				}
			});

			expect(store.hasError).toBe(false);
			expect(store.error).toBeNull();
		});

		it("should support full state reset after error", () => {
			const store = useMcpTesterStore();

			// Populate error state
			store.setError("Critical error");
			store.setLoading(true);
			store.updateRequestJson('{"invalid"}');

			// Reset entire state
			store.clearState();

			expect(store.error).toBeNull();
			expect(store.isLoading).toBe(false);
			expect(store.requestJson).toBe("");
			expect(store.hasError).toBe(false);
		});
	});

	describe("Error Message Quality", () => {
		it("should support detailed error messages with context", () => {
			const store = useMcpTesterStore();

			const detailedError = {
				type: "NATS_CONNECTION_ERROR",
				message: "Failed to connect to NATS server",
				details: "Connection refused at nats://localhost:4222",
				suggestion: "Ensure NATS server is running and accessible"
			};

			const errorString = JSON.stringify(detailedError);
			store.setError(errorString);

			expect(store.error).toBe(errorString);

			// Error can be parsed back for structured display
			const parsed = JSON.parse(store.error!);
			expect(parsed.type).toBe("NATS_CONNECTION_ERROR");
			expect(parsed.suggestion).toBeTruthy();
		});

		it("should handle simple string error messages", () => {
			const store = useMcpTesterStore();

			const simpleError = "Network timeout";
			store.setError(simpleError);

			expect(store.error).toBe(simpleError);
			expect(typeof store.error).toBe("string");
		});
	});

	describe("Concurrent Error States", () => {
		it("should not lose error state when response is cleared", () => {
			const store = useMcpTesterStore();

			store.setError("Request failed");
			expect(store.hasError).toBe(true);

			store.clearResponse();

			// Error should still be present
			expect(store.hasError).toBe(true);
			expect(store.error).toBe("Request failed");
		});

		it("should allow error to exist alongside loading state", () => {
			const store = useMcpTesterStore();

			// Edge case: showing previous error while new request is loading
			store.setError("Previous request failed");
			store.setLoadingResponse(true);

			expect(store.hasError).toBe(true);
			expect(store.isLoadingResponse).toBe(true);

			// This state is valid - UI can show "Retrying..." with previous error visible
		});
	});
});
