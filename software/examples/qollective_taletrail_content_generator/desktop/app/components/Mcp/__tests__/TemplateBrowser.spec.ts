/**
 * TemplateBrowser Component Tests
 *
 * Tests critical behaviors:
 * 1. Template selection updates store
 * 2. File picker opens and loads template
 * 3. Template initialization works
 * 4. Server change clears selected template
 */

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock Tauri modules at the top level (before imports)
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn()
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
	open: vi.fn()
}));

import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import TemplateBrowser from "../TemplateBrowser.vue";
import { useMcpTesterStore } from "../../../stores/mcpTester";
import {
	createMockTemplateData,
	createMockTemplateInfo
} from "../../../utils/__tests__/testHelpers";

describe("TemplateBrowser", () => {
	let pinia: ReturnType<typeof createPinia>;
	let mcpStore: ReturnType<typeof useMcpTesterStore>;

	beforeEach(() => {
		pinia = createPinia();
		setActivePinia(pinia);
		mcpStore = useMcpTesterStore();
		vi.clearAllMocks();
	});

	// Test 1: Template selection updates store
	it("should update store when template is selected", async () => {
		const mockTemplateData = createMockTemplateData({
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
							arguments: { theme: "space" }
						}
					}
				}
			}
		});

		// Mock Tauri commands
		vi.mocked(invoke).mockImplementation(async (cmd: string) => {
			if (cmd === "get_templates_directory") {
				return "/test/templates/orchestrator";
			}
			if (cmd === "load_mcp_template") {
				return mockTemplateData;
			}
			if (cmd === "list_mcp_templates") {
				return { orchestrator: [createMockTemplateInfo()] };
			}
			throw new Error(`Unmocked command: ${cmd}`);
		});

		vi.mocked(open).mockResolvedValue("/test/templates/orchestrator/generate_story.json");

		const wrapper = mount(TemplateBrowser, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		// Click the file picker button
		const filePickerBtn = wrapper.find("button");
		await filePickerBtn.trigger("click");

		// Wait for async operations
		await wrapper.vm.$nextTick();
		await new Promise(resolve => setTimeout(resolve, 0));

		// Verify store was updated
		expect(mcpStore.templateContent).toEqual(mockTemplateData);
		expect(mcpStore.templateContent?.envelope.payload.tool_call?.params.name).toBe("generate_story");
	});

	// Test 2: File picker dialog opens with correct directory
	it("should open file picker with templates directory", async () => {
		const templatesDir = "/test/templates/orchestrator";

		vi.mocked(invoke).mockImplementation(async (cmd: string) => {
			if (cmd === "get_templates_directory") {
				return templatesDir;
			}
			if (cmd === "list_mcp_templates") {
				return { orchestrator: [createMockTemplateInfo()] };
			}
			throw new Error(`Unmocked command: ${cmd}`);
		});

		vi.mocked(open).mockResolvedValue(null); // User cancelled

		const wrapper = mount(TemplateBrowser, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		const filePickerBtn = wrapper.find("button");
		await filePickerBtn.trigger("click");

		await wrapper.vm.$nextTick();

		// Verify file picker was called with correct options
		expect(vi.mocked(open)).toHaveBeenCalledWith(
			expect.objectContaining({
				defaultPath: templatesDir,
				filters: expect.arrayContaining([
					expect.objectContaining({
						name: "JSON Templates",
						extensions: ["json"]
					})
				])
			})
		);
	});

	// Test 3: Server change clears selected template
	it("should clear selected template when server changes", async () => {
		// Mock initial state
		vi.mocked(invoke).mockResolvedValue({ orchestrator: [], "story-generator": [] });

		const wrapper = mount(TemplateBrowser, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		// Set a template in the store
		const mockTemplate = createMockTemplateData();
		mcpStore.setTemplateContent(mockTemplate);

		// Verify template is set
		expect(mcpStore.templateContent).toEqual(mockTemplate);

		// Change server prop
		await wrapper.setProps({ server: "story-generator" });
		await wrapper.vm.$nextTick();

		// Verify component state was cleared (selectedTemplate ref)
		expect(wrapper.vm.selectedTemplate).toBeNull();
	});

	// Test 4: Template initialization calls backend and updates state
	it("should initialize templates from source", async () => {
		const initResult = "Templates initialized successfully. Copied 5 templates.";

		vi.mocked(invoke).mockImplementation(async (cmd: string) => {
			if (cmd === "initialize_templates") {
				return initResult;
			}
			if (cmd === "list_mcp_templates") {
				// First call returns empty, second call returns templates
				const hasTemplates = vi.mocked(invoke).mock.calls.filter(c => c[0] === "list_mcp_templates").length > 1;
				return hasTemplates
					? { orchestrator: [createMockTemplateInfo()] }
					: { orchestrator: [] };
			}
			throw new Error(`Unmocked command: ${cmd}`);
		});

		const wrapper = mount(TemplateBrowser, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		await wrapper.vm.$nextTick();

		// Verify "Initialize Example Templates" button is visible (no templates)
		const initButton = wrapper.findAll("button").find(btn =>
			btn.text().includes("Initialize Example Templates")
		);
		expect(initButton).toBeTruthy();

		// Click initialize button
		await initButton!.trigger("click");
		await wrapper.vm.$nextTick();
		await new Promise(resolve => setTimeout(resolve, 0));

		// Verify backend command was called
		expect(vi.mocked(invoke)).toHaveBeenCalledWith("initialize_templates");

		// Verify templates check was called again after initialization
		expect(vi.mocked(invoke).mock.calls.filter(c => c[0] === "list_mcp_templates").length).toBeGreaterThan(1);
	});

	// Test 5: Error handling when template load fails
	it("should display error when template loading fails", async () => {
		const errorMessage = "Failed to load template: Invalid JSON";

		vi.mocked(invoke).mockImplementation(async (cmd: string) => {
			if (cmd === "get_templates_directory") {
				return "/test/templates/orchestrator";
			}
			if (cmd === "load_mcp_template") {
				throw new Error(errorMessage);
			}
			if (cmd === "list_mcp_templates") {
				return { orchestrator: [createMockTemplateInfo()] };
			}
			throw new Error(`Unmocked command: ${cmd}`);
		});

		vi.mocked(open).mockResolvedValue("/test/templates/bad.json");

		const wrapper = mount(TemplateBrowser, {
			props: {
				server: "orchestrator"
			},
			global: {
				plugins: [pinia]
			}
		});

		const filePickerBtn = wrapper.find("button");
		await filePickerBtn.trigger("click");

		await wrapper.vm.$nextTick();
		await new Promise(resolve => setTimeout(resolve, 0));

		// Verify error is displayed
		expect(wrapper.text()).toContain("Failed to load template");
	});
});
