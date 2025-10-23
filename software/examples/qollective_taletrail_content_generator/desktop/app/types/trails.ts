/**
 * TypeScript type definitions for TaleTrail Desktop Viewer
 * These types mirror the Rust structures for type safety across the language boundary
 */

export interface TrailListItem {
  id: string
  file_path: string
  title: string
  description: string
  theme: string
  age_group: string
  language: string
  tags: string[]
  status: string
  generated_at: string
  node_count: number
  tenantId?: string  // Multi-tenant support
}

export interface GenerationResponse {
  status: string
  trail: Trail
  trail_steps?: TrailStep[]  // Backend populates this instead of dag
  service_invocations?: ServiceInvocation[]
}

export interface Trail {
  title: string
  description: string
  metadata: TrailMetadata
  dag?: DAG  // Optional - reconstructed from trail_steps if missing
}

export interface TrailMetadata {
  generation_params: GenerationParams
  start_node_id: string
}

export interface GenerationParams {
  age_group: string
  theme: string
  language: string
  node_count: number
}

export interface DAG {
  nodes: Record<string, ContentNode>
  edges: Edge[]
  start_node_id: string
  convergence_points?: string[]
}

export interface ContentNode {
  id: string
  content: NodeContent
  incoming_edges?: number
  outgoing_edges?: number
  generation_metadata?: GenerationMetadata
}

export interface NodeContent {
  text: string
  choices?: Choice[]
}

export interface Choice {
  id: string
  text: string
  next_node_id: string
}

export interface Edge {
  from_node_id: string
  to_node_id: string
  choice_id: string
}

export interface GenerationMetadata {
  llm_model?: string
  timestamp?: string
}

export interface ServiceInvocation {
  service_name: string
  phase: string
  success: boolean
  duration_ms: number
  error_message?: string
}

/**
 * TrailStep - Database-normalized format from backend
 * Stores content references and metadata for each step in the trail
 */
export interface TrailStep {
  step_order: number
  title?: string
  description?: string
  is_required: boolean
  metadata: Record<string, any>
  content_reference: ContentReference
}

export interface ContentReference {
  temp_node_id: string
  content: Content
}

export interface Content {
  type: string  // "interactive_story_node"
  node_id?: string
  text: string
  choices: Choice[]
  convergence_point?: boolean
  next_nodes?: string[]
  educational_content?: EducationalContent
}

export interface EducationalContent {
  topic?: string
  vocabulary_words?: string[]
  learning_objectives?: string[]
}

export interface UserPreferences {
  default_view_mode: ViewMode
  theme: Theme
  directory_path: string
  auto_validate: boolean
}

export enum ViewMode {
  Linear = 'Linear',
  Interactive = 'Interactive',
  DAG = 'DAG',
  ExecutionTrace = 'ExecutionTrace'
}

export enum Theme {
  Light = 'Light',
  Dark = 'Dark',
  System = 'System'
}

export interface Bookmark {
  trail_id: string
  timestamp: string
  user_note: string
  tenantId?: string  // Multi-tenant support
}

/**
 * Full envelope response structure from NATS/MCP
 */
export interface EnvelopeResponse {
  meta: {
    timestamp: string
    request_id: string
    version: string
    tenant: string
  }
  payload: {
    tool_response: {
      content: Array<{
        type: string
        text: string
      }>
    }
  }
}

/**
 * NATS Live Generation Event
 * Event emitted during real-time trail generation
 */
export interface GenerationEvent {
  eventType: string
  tenantId: string
  requestId: string
  timestamp: string
  servicePhase: string
  status: string
  progress?: number
  errorMessage?: string
  filePath?: string  // Rust serde camelCase conversion from file_path
}

/**
 * Extended generation request with phase tracking
 */
export interface GenerationRequest {
  requestId: string
  tenantId: string
  startTime: string
  phases: Map<string, PhaseProgress>
  status: string
  errorMessage?: string
}

/**
 * Phase progress tracking for each service phase
 */
export interface PhaseProgress {
  phase: string
  status: string
  progress: number
  startTime?: string
  endTime?: string
  errorMessage?: string
}

/**
 * Connection status for NATS live monitoring
 */
export interface NatsConnectionStatus {
  connected: boolean
  subscribed: boolean
  tenantId?: string
}

/**
 * Generation Request Types for Request Replay System
 */

export type AgeGroup = '6-8' | '9-11' | '12-14' | '15-17' | '+18'
export type VocabularyLevel = 'basic' | 'intermediate' | 'advanced'
export type Language = 'de' | 'en'
export type StoryStructure = 'guided' | 'adventure' | 'epic' | 'choose_your_path'

export interface StoryStructureOption {
  value: StoryStructure
  label: string
  description: string
  node_count: number
}

export interface RequestConstraints {
  maxChoicesPerNode?: number  // 2-10
  minStoryLength?: number  // 100-10000
  forbiddenTopics?: string[]
  requiredTopics?: string[]
}

export interface RequestMetadata {
  submittedAt: string
  submittedBy?: string
  originalRequestId?: string
}

export interface SubmitGenerationRequest {
  request_id: string
  tenant_id: string
  theme: string
  age_group: AgeGroup
  language: Language
  vocabulary_level?: VocabularyLevel

  // EITHER story_structure (preset) OR node_count (custom)
  story_structure?: StoryStructure
  node_count?: number

  educational_focus?: string[]
  constraints?: RequestConstraints
  metadata?: RequestMetadata
}

/**
 * Language option for UI selector
 */
export interface LanguageOption {
  code: string  // ISO 639-1
  name: string
}

/**
 * Tenant statistics for analytics
 */
export interface TenantStatistics {
  tenantId: string
  trailCount: number
  successRate: number
  averageNodeCount: number
}
