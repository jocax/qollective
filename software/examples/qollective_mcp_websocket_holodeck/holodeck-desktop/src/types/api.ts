// ABOUTME: TypeScript types for Tauri API communication with Rust backend
// ABOUTME: Defines interfaces for holodeck server status, session data, and MCP interactions

export interface CoordinatorStatusResponse {
  connected: boolean;
  url: string;
  last_ping: string | null;
  error: string | null;
}

export interface ServerInfo {
  name: string;
  url: string;
  status: string;
  capabilities: string[];
  last_health_check: string;
}

export interface SystemHealthResponse {
  overall_health: string;
  connected_servers: number;
  total_servers: number;
  server_details: { [key: string]: ServerInfo };
}

export interface AppInfoResponse {
  name: string;
  version: string;
  coordinator_url: string;
  available_servers: string[];
  theme: string;
}

export interface ValidationResult {
  overall_validation: {
    success: boolean;
    aggregated_score: number;
    coordination_time_ms: number;
  };
  validation_results: {
    story_validation: {
      server: string;
      success: boolean;
      score: number;
    };
    environment_validation: {
      server: string;
      success: boolean;
      physics_check: string;
    };
    safety_validation: {
      server: string;
      success: boolean;
      safety_level: string;
    };
    character_validation: {
      server: string;
      success: boolean;
      consistency_score: number;
    };
  };
}

// Holodeck Session Types
export interface HolodeckSession {
  id: string;
  name: string;
  story_template: string;
  participants: string[];
  status: 'active' | 'paused' | 'completed' | 'aborted';
  start_time: string;
  end_time?: string;
}