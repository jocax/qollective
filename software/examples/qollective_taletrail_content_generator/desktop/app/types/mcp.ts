/**
 * TypeScript type definitions for MCP Testing UI
 * These types mirror the Rust structures for MCP protocol interaction
 */

// ============================================================================
// Template Management Types
// ============================================================================

export interface GroupedTemplates {
	orchestrator: TemplateGroup
	story_generator: TemplateGroup
	quality_control: TemplateGroup
	constraint_enforcer: TemplateGroup
	prompt_helper: TemplateGroup
}

export interface TemplateGroup {
	server_name: string
	templates: TemplateInfo[]
}

export interface TemplateInfo {
	file_name: string
	file_path: string
	tool_name: string
	description: string
}

export interface TemplateData {
	subject: string // NATS subject for routing
	envelope: { // Complete Qollective envelope
		meta: {
			request_id?: string
			tenant?: string
			tracing?: {
				trace_id?: string
				operation_name?: string
				parent_span_id?: string
				span_id?: string
			}
		}
		payload: {
			tool_call?: {
				method: string
				params: {
					name: string
					arguments?: Record<string, any>
				}
			}
		}
	}
	schema?: ToolSchema // Optional, for backward compatibility
}

export interface ToolSchema {
	tool_name: string
	description: string
	input_schema: JsonSchema
}

export interface JsonSchema {
	type: string
	properties?: Record<string, SchemaProperty>
	required?: string[]
	additionalProperties?: boolean
}

export interface SchemaProperty {
	type: string
	description?: string
	enum?: any[]
	items?: JsonSchema
	properties?: Record<string, SchemaProperty>
	default?: any
	minimum?: number
	maximum?: number
	pattern?: string
}

// ============================================================================
// Request/Response Types
// ============================================================================

export interface CallToolResult {
	content: ContentItem[]
	isError?: boolean
}

export interface ContentItem {
	type: "text" | "image" | "resource"
	text?: string
	data?: string
	mimeType?: string
	uri?: string
}

// ============================================================================
// Qollective Envelope Types for MCP Responses
// ============================================================================

/**
 * Complete Qollective envelope wrapper for MCP responses
 * Matches the Rust McpResponseEnvelope structure
 */
export interface McpResponseEnvelope {
	meta: McpMeta
	payload: McpPayload
	error?: EnvelopeError
}

/**
 * Envelope metadata section
 * Contains tenant info, tracing, request IDs, and timestamps
 */
export interface McpMeta {
	timestamp?: string
	request_id?: string
	version?: string
	duration?: number
	tenant?: string
	on_behalf_of?: OnBehalfOfMeta
	security?: SecurityMeta
	debug?: DebugMeta
	performance?: PerformanceMeta
	monitoring?: MonitoringMeta
	tracing?: TracingMeta
	extensions?: Record<string, any>
}

/**
 * Distributed tracing metadata
 */
export interface TracingMeta {
	trace_id?: string
	parent_span_id?: string
	span_id?: string
	baggage?: Record<string, string>
	sampling_rate?: number
	sampled?: boolean
	trace_state?: string
	operation_name?: string
	span_kind?: string
	span_status?: string
	tags?: Record<string, string>
}

/**
 * On-behalf-of delegation metadata
 */
export interface OnBehalfOfMeta {
	originalUser: string
	delegatingUser: string
	delegatingTenant: string
}

/**
 * Security context metadata
 */
export interface SecurityMeta {
	user_id?: string
	session_id?: string
	auth_method?: string
	permissions?: string[]
	roles?: string[]
	scopes?: string[]
}

/**
 * Debug metadata
 */
export interface DebugMeta {
	source_file?: string
	source_line?: number
	source_function?: string
	stack_trace?: string[]
	variables?: Record<string, any>
}

/**
 * Performance metrics metadata
 */
export interface PerformanceMeta {
	cpu_time_ms?: number
	memory_bytes?: number
	network_bytes_sent?: number
	network_bytes_received?: number
	db_query_count?: number
	cache_hit_count?: number
	cache_miss_count?: number
}

/**
 * Monitoring metadata
 */
export interface MonitoringMeta {
	environment?: string
	service_name?: string
	service_version?: string
	host?: string
	region?: string
	availability_zone?: string
}

/**
 * MCP payload section of the envelope
 */
export interface McpPayload {
	tool_call?: any  // CallToolRequest
	tool_response?: CallToolResult
	tool_registration?: any
	discovery_data?: any
}

/**
 * Envelope error information
 */
export interface EnvelopeError {
	code: string
	message: string
	details?: Record<string, any>
}

export interface McpRequest {
	subject: string
	tool_name: string
	arguments: Record<string, any>
	tenant_id: number
}

export interface McpTemplateRequest {
	template_path: string
}

// ============================================================================
// History Management Types
// ============================================================================

export interface HistoryEntry {
	id: string
	timestamp: string
	server: string
	tool_name: string
	request: Record<string, any>
	response: CallToolResult
	duration_ms?: number
	success: boolean
	tenant_id: number
}

export interface HistoryQuery {
	server_filter?: string
	tool_filter?: string
	success_filter?: boolean
	limit?: number
	offset?: number
	tenant_id?: number
}

export interface HistoryPage {
	entries: HistoryEntry[]
	total: number
	page: number
	page_size: number
	has_more: boolean
}

// ============================================================================
// UI State Types
// ============================================================================

// Removed: EditorMode type (form mode has been removed from MCP Tester)
// export type EditorMode = "template" | "form";

export type ServerName
	= | "orchestrator"
		| "story-generator"
		| "quality-control"
		| "constraint-enforcer"
		| "prompt-helper";

export interface ServerOption {
	label: string
	value: ServerName
	subject: string
}

// ============================================================================
// Error Types
// ============================================================================

export interface McpError {
	code: string
	message: string
	details?: Record<string, any>
}
