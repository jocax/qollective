/**
 * TypeScript type definitions for Search/Execution History functionality
 */

/**
 * Execution directory structure
 * Represents a request-id directory with associated MCP server files
 */
export interface ExecutionDirectory {
	requestId: string // The request-id (directory name)
	servers: string[] // MCP servers that have files (e.g., ["orchestrator", "story-generator"])
	timestamp: string // For display (extracted from request-id or file timestamp)
}

/**
 * Execution file type
 */
export type ExecutionFileType = "request" | "response";

/**
 * Execution file content
 */
export interface ExecutionFile {
	requestId: string
	server: string // MCP server name
	fileType: ExecutionFileType
	content: string // JSON string content
}
