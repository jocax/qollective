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
