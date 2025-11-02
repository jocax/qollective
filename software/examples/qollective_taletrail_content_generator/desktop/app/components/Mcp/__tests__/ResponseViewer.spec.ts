/**
 * ResponseViewer Component Tests
 *
 * Tests critical behaviors:
 * 1. Loading state displays correctly
 * 2. Error state displays with message
 * 3. Success state shows formatted response
 * 4. Copy and download actions work
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import ResponseViewer from "../ResponseViewer.vue";
import { createMockMcpResponseEnvelope } from "../../../utils/__tests__/testHelpers";

// Mock clipboard API
Object.defineProperty(navigator, 'clipboard', {
	value: {
		writeText: vi.fn().mockResolvedValue(undefined)
	},
	writable: true,
	configurable: true
});

describe("ResponseViewer", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	// Test 1: Loading state displays spinner
	it("should display loading state when waiting for response", async () => {
		const wrapper = mount(ResponseViewer, {
			props: {
				response: null,
				loading: true,
				error: null
			}
		});

		// Verify loading indicator is visible
		expect(wrapper.text()).toContain("Waiting for response");
		expect(wrapper.find(".animate-spin").exists()).toBe(true);
	});

	// Test 2: Error state displays error message
	it("should display error state with message", async () => {
		const errorMessage = "Request timeout: No response from MCP server after 30s";

		const wrapper = mount(ResponseViewer, {
			props: {
				response: null,
				loading: false,
				error: errorMessage
			}
		});

		// Verify error is displayed
		expect(wrapper.text()).toContain("Request Failed");
		expect(wrapper.text()).toContain(errorMessage);

		// Verify error styling is present (background color indicates error state)
		expect(wrapper.html()).toContain('bg-error');
	});

	// Test 3: Success state shows formatted response
	it("should display response in formatted JSON", async () => {
		const mockResponse = createMockMcpResponseEnvelope({
			meta: {
				request_id: "req-123",
				duration: 1500,
				tenant: "test-tenant"
			},
			payload: {
				tool_response: {
					content: [
						{
							type: "text",
							text: "Generated story content here"
						}
					]
				}
			}
		});

		const wrapper = mount(ResponseViewer, {
			props: {
				response: mockResponse,
				loading: false,
				error: null
			}
		});

		// Verify response is displayed in textarea
		const textarea = wrapper.find("textarea");
		expect(textarea.exists()).toBe(true);

		const textareaValue = textarea.element.value;
		expect(textareaValue).toContain('"request_id": "req-123"');
		expect(textareaValue).toContain('"duration": 1500');
		expect(textareaValue).toContain("Generated story content here");

		// Verify textarea is readonly
		expect(textarea.attributes("readonly")).toBeDefined();
	});

	// Test 4: Empty state shows when no response
	it("should display empty state when no response", async () => {
		const wrapper = mount(ResponseViewer, {
			props: {
				response: null,
				loading: false,
				error: null
			}
		});

		// Verify empty state message
		expect(wrapper.text()).toContain("No response yet");
		expect(wrapper.text()).toContain("Send a request to see the response here");
	});

	// Test 5: Copy button copies response to clipboard
	it("should copy response to clipboard when copy button clicked", async () => {
		const mockResponse = createMockMcpResponseEnvelope({
			meta: {
				request_id: "req-123"
			}
		});

		const wrapper = mount(ResponseViewer, {
			props: {
				response: mockResponse,
				loading: false,
				error: null
			}
		});

		// Find and click copy button
		const copyButton = wrapper.findAll("button").find(btn =>
			btn.attributes("title")?.includes("Copy")
		);
		expect(copyButton).toBeTruthy();

		await copyButton!.trigger("click");
		await wrapper.vm.$nextTick();

		// Verify clipboard writeText was called
		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			expect.stringContaining('"request_id": "req-123"')
		);
	});

	// Test 6: Download button creates downloadable file
	it("should download response as JSON file when download button clicked", async () => {
		const mockResponse = createMockMcpResponseEnvelope();

		// Mock URL and DOM methods
		const createObjectURLSpy = vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock-url");
		const revokeObjectURLSpy = vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});
		const createElementSpy = vi.spyOn(document, "createElement");
		const appendChildSpy = vi.spyOn(document.body, "appendChild").mockImplementation(() => null as any);
		const removeChildSpy = vi.spyOn(document.body, "removeChild").mockImplementation(() => null as any);

		const wrapper = mount(ResponseViewer, {
			props: {
				response: mockResponse,
				loading: false,
				error: null
			}
		});

		// Find and click download button
		const downloadButton = wrapper.findAll("button").find(btn =>
			btn.attributes("title")?.includes("Download")
		);
		expect(downloadButton).toBeTruthy();

		await downloadButton!.trigger("click");
		await wrapper.vm.$nextTick();

		// Verify download flow was triggered
		expect(createObjectURLSpy).toHaveBeenCalled();
		expect(createElementSpy).toHaveBeenCalledWith("a");
		expect(appendChildSpy).toHaveBeenCalled();
		expect(removeChildSpy).toHaveBeenCalled();
		expect(revokeObjectURLSpy).toHaveBeenCalled();

		// Cleanup
		createObjectURLSpy.mockRestore();
		revokeObjectURLSpy.mockRestore();
		createElementSpy.mockRestore();
		appendChildSpy.mockRestore();
		removeChildSpy.mockRestore();
	});

	// Test 7: Response envelope structure is preserved
	it("should preserve complete envelope structure in response", async () => {
		const mockResponse = createMockMcpResponseEnvelope({
			meta: {
				request_id: "req-123",
				tenant: "test-tenant",
				tracing: {
					trace_id: "trace-456",
					span_id: "span-789"
				}
			},
			payload: {
				tool_response: {
					content: [{ type: "text", text: "Content" }]
				}
			}
		});

		const wrapper = mount(ResponseViewer, {
			props: {
				response: mockResponse,
				loading: false,
				error: null
			}
		});

		const textarea = wrapper.find("textarea");
		const textareaValue = textarea.element.value;

		// Verify envelope structure is complete
		expect(textareaValue).toContain('"meta"');
		expect(textareaValue).toContain('"payload"');
		expect(textareaValue).toContain('"tracing"');
		expect(textareaValue).toContain('"trace_id": "trace-456"');
		expect(textareaValue).toContain('"span_id": "span-789"');
		expect(textareaValue).toContain('"tool_response"');
	});
});
