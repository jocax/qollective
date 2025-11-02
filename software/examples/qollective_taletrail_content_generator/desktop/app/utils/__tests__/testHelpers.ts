/**
 * Test Utility Helpers for TaleTrail Desktop Application
 *
 * Provides mocking utilities and fixtures for testing Tauri V2 + Nuxt 4 application
 * including Tauri invoke(), Pinia stores, NATS client responses, and common data fixtures.
 */

import type {
	CallToolResult,
	GroupedTemplates,
	HistoryEntry,
	McpResponseEnvelope,
	TemplateData,
	TemplateInfo
} from "@/types/mcp";
import type { TrailListItem } from "@/types/trails";
import type { PiniaPluginContext } from "pinia";
import { createPinia, setActivePinia } from "pinia";
import { vi } from "vitest";

// ============================================================================
// TAURI MOCKING
// ============================================================================

/**
 * Mock implementation of Tauri's invoke() function
 * Returns a vi.fn() that can be configured per test
 */
export function createMockInvoke() {
	return vi.fn<[string, any?], Promise<any>>();
}

/**
 * Setup Tauri invoke mock with default behavior
 * Can be overridden per test by calling mockInvoke.mockResolvedValue() or mockRejectedValue()
 */
export function setupTauriMock() {
	const mockInvoke = createMockInvoke();

	// Mock the @tauri-apps/api/core module
	vi.mock("@tauri-apps/api/core", () => ({
		invoke: mockInvoke
	}));

	return { mockInvoke };
}

/**
 * Create a mock response for specific Tauri commands
 * @param command - The Tauri command name
 * @param response - The response to return when command is invoked
 */
export function mockTauriCommand<T = any>(
	mockInvoke: ReturnType<typeof createMockInvoke>,
	command: string,
	response: T
) {
	mockInvoke.mockImplementation((cmd: string, args?: any) => {
		if (cmd === command) {
			return Promise.resolve(response);
		}
		return Promise.reject(new Error(`Unmocked command: ${cmd}`));
	});
}

/**
 * Create a mock that handles multiple Tauri commands
 * @param commandMap - Map of command names to responses
 */
export function mockMultipleTauriCommands(
	mockInvoke: ReturnType<typeof createMockInvoke>,
	commandMap: Record<string, any>
) {
	mockInvoke.mockImplementation((cmd: string, args?: any) => {
		if (cmd in commandMap) {
			const response = commandMap[cmd];
			// If response is a function, call it with args
			return Promise.resolve(typeof response === "function" ? response(args) : response);
		}
		return Promise.reject(new Error(`Unmocked command: ${cmd}`));
	});
}

// ============================================================================
// PINIA MOCKING
// ============================================================================

/**
 * Setup Pinia for testing
 * Creates a fresh Pinia instance and sets it as active
 */
export function setupPinia() {
	const pinia = createPinia();
	setActivePinia(pinia);
	return pinia;
}

/**
 * Create a mock Pinia store with common patterns
 * Useful for testing components that depend on stores
 */
export function createMockStore<T extends Record<string, any>>(
	initialState: T
) {
	const state = reactive(initialState);

	return {
		...state,
		$patch: vi.fn((patch: Partial<T>) => {
			Object.assign(state, patch);
		}),
		$reset: vi.fn(() => {
			Object.assign(state, initialState);
		}),
		$subscribe: vi.fn(),
		$onAction: vi.fn()
	};
}

// ============================================================================
// NATS CLIENT MOCKING
// ============================================================================

/**
 * Mock NATS connection response
 */
export function createMockNatsConnection(connected = true) {
	return {
		connected,
		subscribed: connected,
		url: "nats://localhost:5222"
	};
}

/**
 * Mock NATS message for monitoring
 */
export function createMockNatsMessage(
	subject: string,
	payload: any,
	timestamp?: string
) {
	return {
		subject,
		payload: JSON.stringify(payload),
		timestamp: timestamp || new Date().toISOString()
	};
}

// ============================================================================
// MCP TESTING FIXTURES
// ============================================================================

/**
 * Create a mock TemplateInfo
 */
export function createMockTemplateInfo(
	overrides?: Partial<TemplateInfo>
): TemplateInfo {
	return {
		file_name: "test_template.json",
		file_path: "/path/to/test_template.json",
		tool_name: "test_tool",
		description: "Test template for testing",
		...overrides
	};
}

/**
 * Create a mock TemplateData with complete envelope structure
 */
export function createMockTemplateData(
	overrides?: Partial<TemplateData>
): TemplateData {
	return {
		subject: "generation.requests",
		envelope: {
			meta: {
				request_id: "test-request-123",
				tenant: "test-tenant",
				tracing: {
					trace_id: "trace-123",
					operation_name: "test_operation"
				}
			},
			payload: {
				tool_call: {
					method: "tools/call",
					params: {
						name: "test_tool",
						arguments: {
							theme: "space adventure",
							age_group: "9-11",
							language: "en"
						}
					}
				}
			}
		},
		...overrides
	};
}

/**
 * Create mock grouped templates for all MCP servers
 */
export function createMockGroupedTemplates(
	templatesPerServer = 3
): GroupedTemplates {
	const createTemplates = (serverName: string, count: number): TemplateInfo[] => {
		return Array.from({ length: count }, (_, i) => ({
			file_name: `${serverName}_template_${i + 1}.json`,
			file_path: `/templates/${serverName}/${serverName}_template_${i + 1}.json`,
			tool_name: `${serverName}_tool_${i + 1}`,
			description: `Test template ${i + 1} for ${serverName}`
		}));
	};

	return {
		orchestrator: {
			server_name: "orchestrator",
			templates: createTemplates("orchestrator", templatesPerServer)
		},
		story_generator: {
			server_name: "story_generator",
			templates: createTemplates("story_generator", templatesPerServer)
		},
		quality_control: {
			server_name: "quality_control",
			templates: createTemplates("quality_control", templatesPerServer)
		},
		constraint_enforcer: {
			server_name: "constraint_enforcer",
			templates: createTemplates("constraint_enforcer", templatesPerServer)
		},
		prompt_helper: {
			server_name: "prompt_helper",
			templates: createTemplates("prompt_helper", templatesPerServer)
		}
	};
}

/**
 * Create a mock CallToolResult (MCP tool response)
 */
export function createMockCallToolResult(
	overrides?: Partial<CallToolResult>
): CallToolResult {
	return {
		content: [
			{
				type: "text",
				text: "Test tool response content"
			}
		],
		isError: false,
		...overrides
	};
}

/**
 * Create a mock McpResponseEnvelope (complete Qollective envelope)
 */
export function createMockMcpResponseEnvelope(
	overrides?: Partial<McpResponseEnvelope>
): McpResponseEnvelope {
	return {
		meta: {
			timestamp: new Date().toISOString(),
			request_id: "test-request-123",
			version: "1.0.0",
			duration: 1500,
			tenant: "test-tenant",
			tracing: {
				trace_id: "trace-123",
				span_id: "span-456",
				operation_name: "test_operation"
			}
		},
		payload: {
			tool_response: {
				content: [
					{
						type: "text",
						text: "Mock response content"
					}
				]
			}
		},
		...overrides
	};
}

/**
 * Create a mock HistoryEntry
 */
export function createMockHistoryEntry(
	overrides?: Partial<HistoryEntry>
): HistoryEntry {
	return {
		id: "history-entry-123",
		timestamp: new Date().toISOString(),
		server: "orchestrator",
		tool_name: "generate_story",
		request: {
			theme: "space adventure",
			age_group: "9-11",
			language: "en"
		},
		response: createMockCallToolResult(),
		duration_ms: 1500,
		success: true,
		tenant_id: 1,
		...overrides
	};
}

// ============================================================================
// TRAIL VIEWER FIXTURES
// ============================================================================

/**
 * Create a mock TrailListItem
 */
export function createMockTrailListItem(
	overrides?: Partial<TrailListItem>
): TrailListItem {
	return {
		id: "trail-123",
		file_path: "/trails/test_trail.json",
		title: "Test Adventure Trail",
		description: "A test story trail for testing",
		theme: "space adventure",
		age_group: "9-11",
		language: "en",
		tags: ["test", "adventure", "space"],
		status: "completed",
		generated_at: new Date().toISOString(),
		node_count: 15,
		tenantId: "test-tenant",
		...overrides
	};
}

/**
 * Create multiple mock trail items
 */
export function createMockTrailList(count = 10): TrailListItem[] {
	return Array.from({ length: count }, (_, i) => {
		const themes = ["space adventure", "medieval quest", "underwater exploration", "jungle expedition"];
		const ageGroups = ["6-8", "9-11", "12-14", "15-17"];
		const languages = ["en", "de"];
		const statuses = ["completed", "in_progress", "failed"];

		return createMockTrailListItem({
			id: `trail-${i + 1}`,
			title: `Test Trail ${i + 1}`,
			theme: themes[i % themes.length],
			age_group: ageGroups[i % ageGroups.length],
			language: languages[i % languages.length],
			status: statuses[i % statuses.length],
			node_count: 10 + i
		});
	});
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Wait for Vue's nextTick and any pending promises
 * Useful for testing async component updates
 */
export async function flushPromises() {
	return new Promise((resolve) => {
		setTimeout(resolve, 0);
	});
}

/**
 * Create a mock error response for Tauri commands
 */
export function createMockTauriError(message: string, code?: string) {
	return {
		code: code || "TAURI_ERROR",
		message,
		details: {}
	};
}

/**
 * Wait for a specific amount of time
 * Useful for testing debounced or throttled functions
 */
export function wait(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Create a mock file dialog response
 */
export function createMockFileDialogResponse(selected = true, path?: string) {
	if (!selected) {
		return null;
	}
	return path || "/test/directory/path";
}

/**
 * Assert that a mock function was called with specific arguments
 * More readable than toHaveBeenCalledWith for complex objects
 */
export function expectCalledWith<T extends (...args: any[]) => any>(
	fn: T,
	...expectedArgs: Parameters<T>
) {
	const calls = vi.mocked(fn).mock.calls;
	const matchingCall = calls.find((call) => {
		return expectedArgs.every((arg, index) => {
			if (typeof arg === "object" && arg !== null) {
				return JSON.stringify(call[index]) === JSON.stringify(arg);
			}
			return call[index] === arg;
		});
	});

	return {
		toBeCalled: () => expect(matchingCall).toBeDefined(),
		toBeCalledTimes: (times: number) => {
			const matchingCalls = calls.filter((call) =>
				expectedArgs.every((arg, index) => call[index] === arg)
			);
			expect(matchingCalls.length).toBe(times);
		}
	};
}
