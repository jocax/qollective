/**
 * RequestEditor Component Tests
 *
 * Tests critical behaviors:
 * 1. JSON validation enables/disables send button
 * 2. Template content loads into editor
 * 3. JSON editing updates store
 * 4. Subject mismatch warning appears correctly
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import RequestEditor from "../RequestEditor.vue";
import { useMcpTesterStore } from "../../../stores/mcpTester";
import { createMockTemplateData } from "../../../utils/__tests__/testHelpers";

describe("RequestEditor", () => {
	let pinia: ReturnType<typeof createPinia>;
	let mcpStore: ReturnType<typeof useMcpTesterStore>;

	beforeEach(() => {
		pinia = createPinia();
		setActivePinia(pinia);
		mcpStore = useMcpTesterStore();
		vi.clearAllMocks();
	});

	// Test 1: canSend computed property reflects JSON validity
	it("should have canSend false when template content has invalid JSON", async () => {
		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		// Load valid template first
		const mockTemplate = createMockTemplateData();
		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Verify canSend is true with valid JSON
		expect(mcpStore.canSend).toBe(true);

		// Now update the store's requestJson to be invalid
		mcpStore.updateRequestJson('{ "invalid": json }');
		await wrapper.vm.$nextTick();

		// Verify canSend is false
		expect(mcpStore.canSend).toBe(false);
	});

	// Test 2: Send button enabled when JSON is valid
	it("should enable send button when JSON is valid", async () => {
		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		// Load template with valid JSON
		const mockTemplate = createMockTemplateData();
		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Verify canSend computed is true
		expect(mcpStore.canSend).toBe(true);

		// Verify send button is enabled
		const sendButton = wrapper.findAll("button").find(btn => btn.text().includes("Send Request"));
		expect(sendButton?.element.hasAttribute("disabled")).toBe(false);
	});

	// Test 3: Template content loads into editor
	it("should load template content into JSON editor", async () => {
		const mockTemplate = createMockTemplateData({
			subject: "mcp.orchestrator.request",
			envelope: {
				meta: {
					request_id: "test-123",
					tenant: "test-tenant"
				},
				payload: {
					tool_call: {
						method: "tools/call",
						params: {
							name: "generate_story",
							arguments: { theme: "space adventure" }
						}
					}
				}
			}
		});

		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		// Set template in store
		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Verify textarea contains formatted JSON
		const textarea = wrapper.find("textarea");
		const textareaValue = textarea.element.value;

		expect(textareaValue).toContain('"subject"');
		expect(textareaValue).toContain('"envelope"');
		expect(textareaValue).toContain('"generate_story"');
		expect(textareaValue).toContain('"space adventure"');
	});

	// Test 4: Store's setTemplateContent updates the displayed JSON
	it("should display updated JSON when store template content changes", async () => {
		const mockTemplate = createMockTemplateData();

		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Verify initial content
		const textarea = wrapper.find("textarea");
		expect(textarea.element.value).toContain('"theme": "space adventure"');

		// Update via store
		const newTemplate = {
			...mockTemplate,
			envelope: {
				...mockTemplate.envelope,
				payload: {
					tool_call: {
						method: "tools/call",
						params: {
							name: "generate_story",
							arguments: { theme: "underwater exploration" }
						}
					}
				}
			}
		};

		mcpStore.setTemplateContent(newTemplate);
		await wrapper.vm.$nextTick();

		// Verify textarea updated
		expect(textarea.element.value).toContain('"theme": "underwater exploration"');
	});

	// Test 5: Subject mismatch warning appears correctly
	it("should show warning when subject doesn't match current server", async () => {
		const mockTemplate = createMockTemplateData({
			subject: "mcp.story-generator.request" // Different from current server
		});

		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator" // Current server is orchestrator
			},
			global: {
				plugins: [pinia]
			}
		});

		// Set server in store to match prop
		mcpStore.setServer("orchestrator");

		// Load template with mismatched subject
		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Verify warning is shown
		expect(wrapper.text()).toContain("Subject Mismatch");
		expect(wrapper.text()).toContain("story-generator");
		expect(wrapper.text()).toContain("orchestrator");
	});

	// Test 6: No warning when subject matches current server
	it("should not show warning when subject matches current server", async () => {
		const mockTemplate = createMockTemplateData({
			subject: "mcp.orchestrator.request" // Matches current server
		});

		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		mcpStore.setServer("orchestrator");
		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Verify no warning is shown
		expect(wrapper.text()).not.toContain("Subject Mismatch");
	});

	// Test 7: Send event emits correct data
	it("should emit send event with template and timeout", async () => {
		const mockTemplate = createMockTemplateData();

		const wrapper = mount(RequestEditor, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		mcpStore.setTemplateContent(mockTemplate);
		await wrapper.vm.$nextTick();

		// Click send button
		const sendButton = wrapper.findAll("button").find(btn => btn.text().includes("Send Request"));
		await sendButton!.trigger("click");
		await wrapper.vm.$nextTick();

		// Verify send event was emitted
		const sendEvents = wrapper.emitted("send");
		expect(sendEvents).toBeTruthy();
		expect(sendEvents![0][0]).toHaveProperty("template");
		expect(sendEvents![0][0]).toHaveProperty("timeout");
	});
});
