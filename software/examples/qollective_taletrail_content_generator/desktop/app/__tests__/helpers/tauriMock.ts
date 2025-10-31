import type {
	GenerationEvent,
	GenerationResponse,
	NatsConnectionStatus,
	TrackedRequest,
	TrailListItem
} from "~/types/trails";
import { vi } from "vitest";

/**
 * Create a generic mock invoke function
 */
export function createMockInvoke() {
	return vi.fn();
}

/**
 * Mock successful submit_generation_request command
 */
export function mockSuccessfulSubmit(requestId: string = "req-12345") {
	return vi.fn().mockResolvedValue(requestId);
}

/**
 * Mock failed submit_generation_request command
 */
export function mockFailedSubmit(error: string = "Validation failed") {
	return vi.fn().mockRejectedValue(new Error(error));
}

/**
 * Mock successful load_trail_full command
 */
export function mockTrailLoad(trail: GenerationResponse) {
	return vi.fn().mockResolvedValue(trail);
}

/**
 * Mock failed load_trail_full command
 */
export function mockTrailLoadError(error: string = "Trail not found") {
	return vi.fn().mockRejectedValue(new Error(error));
}

/**
 * Mock NATS connection status
 */
export function mockConnectionStatus(status: Partial<NatsConnectionStatus> = {}) {
	return vi.fn().mockResolvedValue({
		connected: true,
		subscribed: false,
		tenantId: undefined,
		...status
	});
}

/**
 * Mock load_trails_from_directory command
 */
export function mockLoadTrails(trails: TrailListItem[] = []) {
	return vi.fn().mockResolvedValue(trails);
}

/**
 * Mock replay_generation_request command
 */
export function mockReplayRequest(requestId: string = "req-67890") {
	return vi.fn().mockResolvedValue(requestId);
}

/**
 * Mock subscribe_to_generations command
 */
export function mockSubscribe() {
	return vi.fn().mockResolvedValue(undefined);
}

/**
 * Mock unsubscribe_from_generations command
 */
export function mockUnsubscribe() {
	return vi.fn().mockResolvedValue(undefined);
}

/**
 * Mock disconnect_nats command
 */
export function mockDisconnect() {
	return vi.fn().mockResolvedValue(undefined);
}

/**
 * Mock nats_connection_status command
 */
export function mockNatsStatus(status: Partial<NatsConnectionStatus> = {}) {
	return vi.fn().mockResolvedValue({
		connected: false,
		subscribed: false,
		tenantId: undefined,
		...status
	});
}

/**
 * Mock get_active_requests command
 */
export function mockActiveRequests(requests: TrackedRequest[] = []) {
	return vi.fn().mockResolvedValue(requests);
}

/**
 * Create a mock invoke function that handles multiple commands
 */
export function createMockInvokeWithCommands(commandHandlers: Record<string, any>) {
	return vi.fn((command: string, args?: any) => {
		const handler = commandHandlers[command];
		if (handler) {
			return typeof handler === "function" ? handler(args) : Promise.resolve(handler);
		}
		return Promise.reject(new Error(`Unknown command: ${command}`));
	});
}

/**
 * Create a mock Tauri event listener
 */
export function createMockEventListener() {
	const listeners: Map<string, Set<(payload: any) => void>> = new Map();

	const listen = vi.fn((event: string, handler: (payload: any) => void) => {
		if (!listeners.has(event)) {
			listeners.set(event, new Set());
		}
		listeners.get(event)!.add(handler);

		// Return unlisten function
		return Promise.resolve(() => {
			listeners.get(event)?.delete(handler);
		});
	});

	const emit = (event: string, payload: any) => {
		listeners.get(event)?.forEach((handler) => {
			handler({ payload });
		});
	};

	const clear = () => {
		listeners.clear();
	};

	return { listen, emit, clear };
}

/**
 * Factory for creating test trail data
 */
export function createTestTrail(overrides: Partial<TrailListItem> = {}): TrailListItem {
	return {
		id: "trail-123",
		file_path: "/test/trails/trail-123.json",
		title: "Test Adventure",
		description: "A test adventure for unit testing",
		theme: "space",
		age_group: "9-11",
		language: "en",
		tags: ["test", "adventure"],
		status: "completed",
		generated_at: "2025-01-15T10:30:00Z",
		node_count: 10,
		tenantId: "tenant-test",
		...overrides
	};
}

/**
 * Factory for creating test generation response
 */
export function createTestGenerationResponse(overrides: Partial<GenerationResponse> = {}): GenerationResponse {
	return {
		status: "completed",
		trail: {
			title: "Test Trail",
			description: "Test description",
			metadata: {
				generation_params: {
					age_group: "9-11",
					theme: "space",
					language: "en",
					node_count: 10
				},
				start_node_id: "node-001"
			},
			dag: {
				nodes: {
					"node-001": {
						id: "node-001",
						content: {
							text: "You wake up on a spaceship.",
							choices: [
								{ id: "choice-1", text: "Explore the bridge", next_node_id: "node-002" }
							]
						}
					},
					"node-002": {
						id: "node-002",
						content: {
							text: "The bridge is empty.",
							choices: []
						}
					}
				},
				edges: [
					{ from_node_id: "node-001", to_node_id: "node-002", choice_id: "choice-1" }
				],
				start_node_id: "node-001",
				convergence_points: []
			}
		},
		trail_steps: [],
		service_invocations: [
			{
				service_name: "story-generator",
				phase: "generation",
				success: true,
				duration_ms: 1234
			}
		],
		...overrides
	};
}

/**
 * Factory for creating test generation events
 */
export function createTestGenerationEvent(overrides: Partial<GenerationEvent> = {}): GenerationEvent {
	return {
		eventType: "Progress",
		tenantId: "tenant-test",
		requestId: "req-123",
		timestamp: new Date().toISOString(),
		servicePhase: "story-generator",
		status: "in_progress",
		progress: 0.5,
		...overrides
	};
}

/**
 * Factory for creating test tracked request
 */
export function createTestTrackedRequest(overrides: Partial<TrackedRequest> = {}): TrackedRequest {
	return {
		requestId: "req-123",
		tenantId: "tenant-test",
		startTime: new Date().toISOString(),
		currentPhase: "story-generator",
		progress: 0.5,
		lastUpdate: new Date().toISOString(),
		component: "desktop",
		status: "in_progress",
		...overrides
	};
}
