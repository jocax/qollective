/**
 * NATS Message Interface
 * Represents a message received from NATS monitoring
 */
export interface NatsMessage {
	timestamp: string // ISO 8601 timestamp
	subject: string // Full NATS subject
	endpoint: string // e.g., "orchestrator", "story-generator"
	message_type: string // "Request", "Response", "Event", "Unknown"
	payload: string // JSON string
	request_id?: string // Optional extracted request ID
}

/**
 * NATS Monitor Status
 */
export interface NatsMonitorStatus {
	connected: boolean
	message: string
}

/**
 * Endpoint filter options
 */
export type EndpointFilter
	= | "all"
		| "orchestrator"
		| "story-generator"
		| "quality-control"
		| "constraint-enforcer"
		| "prompt-helper";

/**
 * Monitoring Diagnostics Interface
 * Tracks message flow and connection health
 */
export interface MonitoringDiagnostics {
	received: number // Total messages received from backend
	emitted: number // Total messages successfully displayed
	failures: number // Number of emission failures
	lastMessage?: string // ISO timestamp of last message received
	connected: string // ISO timestamp when connection was established
}

/**
 * Monitoring Error Interface
 * Represents errors that occur during monitoring
 */
export interface MonitoringError {
	message: string // Error message
	timestamp: string // ISO timestamp when error occurred
	severity: "warning" | "error" // Error severity level
}
