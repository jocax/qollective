// app/config/constants.ts
//
// Frontend constants for TaleTrail Desktop Application
// These constants mirror the backend configuration structure and provide type-safe values

// ============================================================================
// MCP CONFIGURATION
// ============================================================================

/**
 * Available MCP servers in the TaleTrail system
 */
export const MCP_SERVERS = [
	"orchestrator",
	"story-generator",
	"quality-control",
	"constraint-enforcer",
	"prompt-helper"
] as const;

export type McpServer = typeof MCP_SERVERS[number];

/**
 * Default MCP request timeout in milliseconds (3 minutes)
 */
export const MCP_DEFAULT_TIMEOUT_MS = 180000;

// ============================================================================
// MONITORING CONFIGURATION
// ============================================================================

/**
 * Event types emitted during story generation
 */
export const EVENT_TYPES = [
	"Started",
	"Progress",
	"Completed",
	"Failed",
	"ToolExecution"
] as const;

export type EventType = typeof EVENT_TYPES[number];

/**
 * Maximum number of events to buffer in memory
 */
export const MAX_EVENT_BUFFER_SIZE = 1000;

/**
 * Time range filters for event monitoring (in milliseconds)
 */
export const TIME_RANGES = {
	LAST_5MIN: 5 * 60 * 1000,
	LAST_1HR: 60 * 60 * 1000,
	LAST_24HR: 24 * 60 * 60 * 1000
} as const;

export type TimeRange = keyof typeof TIME_RANGES;

/**
 * Time range options for UI selector
 */
export const TIME_RANGE_OPTIONS = [
	{ value: "all", label: "All Time" },
	{ value: "last-5min", label: "Last 5 Minutes" },
	{ value: "last-1hr", label: "Last Hour" },
	{ value: "last-24hr", label: "Last 24 Hours" }
] as const;

// ============================================================================
// NETWORK CONFIGURATION (from environment or defaults)
// ============================================================================

/**
 * Network-related configuration with environment variable overrides
 */
export const NETWORK = {
	NATS_URL: import.meta.env.VITE_NATS_URL || "nats://localhost:5222",
	DEV_SERVER_PORT: Number.parseInt(import.meta.env.VITE_DEV_PORT || "3030"),
	HMR_PORT: Number.parseInt(import.meta.env.VITE_HMR_PORT || "3031")
} as const;

// ============================================================================
// PATH CONFIGURATION
// ============================================================================

/**
 * Path-related constants for display purposes only.
 * IMPORTANT: Actual path resolution is handled by the Rust backend.
 * These are relative paths or display values only - NO ABSOLUTE PATHS.
 */
export const PATHS = {
	/**
	 * Templates directory relative path (for display)
	 */
	TEMPLATES_DIR_RELATIVE: "templates",

	/**
	 * Default trails directory name (user can override)
	 */
	DEFAULT_TRAILS_DIR: "taletrail-data/trails",

	/**
	 * Default root directory for TaleTrail data
	 * Directory structure:
	 * [root]/
	 * ├── templates/           (MCP request templates organized by server)
	 * │   ├── orchestrator/
	 * │   ├── story-generator/
	 * │   ├── quality-control/
	 * │   ├── constraint-enforcer/
	 * │   └── prompt-helper/
	 * ├── execution/          (Request/response data organized by request ID)
	 * │   └── [request-id]/
	 * │       └── [mcp-server]/
	 * │           ├── request.json
	 * │           └── response.json
	 * └── trails/            (Generated story trails)
	 *     └── [story_files].json
	 */
	DEFAULT_ROOT_DIRECTORY: "taletrail-data"
} as const;

// ============================================================================
// UI CONFIGURATION
// ============================================================================

/**
 * UI timing and interaction constants
 */
export const UI = {
	/**
	 * Toast notification duration in milliseconds
	 */
	TOAST_DURATION_MS: 5000,

	/**
	 * Input debounce delay in milliseconds
	 */
	DEBOUNCE_DELAY_MS: 300,

	/**
	 * Auto-scroll threshold in pixels
	 */
	AUTO_SCROLL_THRESHOLD_PX: 100
} as const;

// ============================================================================
// NATS SUBJECTS (from backend constants)
// ============================================================================

/**
 * NATS subject patterns used by the system
 */
export const NATS_SUBJECTS = {
	GENERATION_REQUESTS: "generation.requests",
	GENERATION_EVENTS_PATTERN: "generation.events.>",
	GENERATION_EVENTS_PREFIX: "generation.events"
} as const;

// ============================================================================
// TYPE EXPORTS
// ============================================================================

/**
 * Re-export commonly used types for convenience
 */
export type {
	EventType as EventTypeEnum,
	McpServer as McpServerType,
	TimeRange as TimeRangeKey
};
