import type { EventType, McpServer, TimeRange } from "../constants";
import { describe, expect, it } from "vitest";
import {
	EVENT_TYPES,
	MAX_EVENT_BUFFER_SIZE,
	MCP_DEFAULT_TIMEOUT_MS,
	MCP_SERVERS,
	NATS_SUBJECTS,
	NETWORK,
	PATHS,
	TIME_RANGES,
	UI
} from "../constants";

describe("constants", () => {
	describe("mCP Configuration", () => {
		it("should export MCP_SERVERS array", () => {
			expect(MCP_SERVERS).toBeDefined();
			expect(Array.isArray(MCP_SERVERS)).toBe(true);
			expect(MCP_SERVERS).toContain("orchestrator");
			expect(MCP_SERVERS).toContain("story-generator");
			expect(MCP_SERVERS).toContain("quality-control");
			expect(MCP_SERVERS).toContain("constraint-enforcer");
			expect(MCP_SERVERS).toContain("prompt-helper");
		});

		it("should export MCP_DEFAULT_TIMEOUT_MS", () => {
			expect(MCP_DEFAULT_TIMEOUT_MS).toBe(180000);
		});

		it("should support McpServer type", () => {
			const server: McpServer = "orchestrator";
			expect(MCP_SERVERS).toContain(server);
		});
	});

	describe("monitoring Configuration", () => {
		it("should export EVENT_TYPES array", () => {
			expect(EVENT_TYPES).toBeDefined();
			expect(Array.isArray(EVENT_TYPES)).toBe(true);
			expect(EVENT_TYPES).toContain("Started");
			expect(EVENT_TYPES).toContain("Progress");
			expect(EVENT_TYPES).toContain("Completed");
			expect(EVENT_TYPES).toContain("Failed");
			expect(EVENT_TYPES).toContain("ToolExecution");
		});

		it("should export MAX_EVENT_BUFFER_SIZE", () => {
			expect(MAX_EVENT_BUFFER_SIZE).toBe(1000);
		});

		it("should export TIME_RANGES object", () => {
			expect(TIME_RANGES).toBeDefined();
			expect(TIME_RANGES.LAST_5MIN).toBe(5 * 60 * 1000);
			expect(TIME_RANGES.LAST_1HR).toBe(60 * 60 * 1000);
			expect(TIME_RANGES.LAST_24HR).toBe(24 * 60 * 60 * 1000);
		});

		it("should support EventType type", () => {
			const eventType: EventType = "Started";
			expect(EVENT_TYPES).toContain(eventType);
		});

		it("should support TimeRange type", () => {
			const timeRange: TimeRange = "LAST_5MIN";
			expect(TIME_RANGES[timeRange]).toBeDefined();
		});
	});

	describe("network Configuration", () => {
		it("should export NETWORK object with defaults", () => {
			expect(NETWORK).toBeDefined();
			expect(NETWORK.NATS_URL).toBeDefined();
			expect(NETWORK.DEV_SERVER_PORT).toBe(3030);
			expect(NETWORK.HMR_PORT).toBe(3031);
		});
	});

	describe("path Configuration", () => {
		it("should export PATHS object", () => {
			expect(PATHS).toBeDefined();
			expect(PATHS.TEMPLATES_DIR_RELATIVE).toBe("templates");
			expect(PATHS.DEFAULT_TRAILS_DIR).toBe("taletrail-data/trails");
		});

		it("should not contain absolute paths", () => {
			Object.values(PATHS).forEach((path) => {
				expect(path).not.toMatch(/^\//); // Should not start with /
				expect(path).not.toMatch(/^[A-Z]:/); // Should not be Windows absolute path
			});
		});
	});

	describe("uI Configuration", () => {
		it("should export UI object", () => {
			expect(UI).toBeDefined();
			expect(UI.TOAST_DURATION_MS).toBe(5000);
			expect(UI.DEBOUNCE_DELAY_MS).toBe(300);
			expect(UI.AUTO_SCROLL_THRESHOLD_PX).toBe(100);
		});
	});

	describe("nATS Subjects", () => {
		it("should export NATS_SUBJECTS object", () => {
			expect(NATS_SUBJECTS).toBeDefined();
			expect(NATS_SUBJECTS.GENERATION_REQUESTS).toBe("generation.requests");
			expect(NATS_SUBJECTS.GENERATION_EVENTS_PATTERN).toBe("generation.events.>");
			expect(NATS_SUBJECTS.GENERATION_EVENTS_PREFIX).toBe("generation.events");
		});
	});

	describe("type Safety", () => {
		it("should enforce McpServer type safety", () => {
			const validServer: McpServer = "orchestrator";
			expect(MCP_SERVERS).toContain(validServer);

			// TypeScript will prevent this at compile time:
			// const invalidServer: McpServer = "invalid"; // Error
		});

		it("should enforce EventType type safety", () => {
			const validEvent: EventType = "Started";
			expect(EVENT_TYPES).toContain(validEvent);

			// TypeScript will prevent this at compile time:
			// const invalidEvent: EventType = "InvalidEvent"; // Error
		});

		it("should enforce TimeRange type safety", () => {
			const validRange: TimeRange = "LAST_5MIN";
			expect(TIME_RANGES[validRange]).toBeDefined();

			// TypeScript will prevent this at compile time:
			// const invalidRange: TimeRange = "INVALID_RANGE"; // Error
		});
	});

	describe("immutability", () => {
		it("should be immutable (as const)", () => {
			// These should be readonly due to 'as const'
			expect(Object.isFrozen(MCP_SERVERS)).toBe(false); // Array itself not frozen
			expect(MCP_SERVERS.length).toBe(5);

			// But TypeScript will prevent mutations at compile time:
			// MCP_SERVERS[0] = "different"; // Error: readonly
			// MCP_SERVERS.push("new"); // Error: readonly
		});
	});
});
