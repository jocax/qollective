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
	tool_name: string
	arguments: Record<string, any>
	schema?: ToolSchema
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

export interface McpRequest {
	subject: string
	tool_name: string
	arguments: Record<string, any>
	tenant_id: number
}

export interface McpTemplateRequest {
	template_path: string
	subject: string
	tenant_id: number
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
