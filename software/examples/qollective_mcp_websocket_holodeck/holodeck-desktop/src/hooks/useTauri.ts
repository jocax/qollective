// ABOUTME: React hooks for communicating with Tauri Rust backend
// ABOUTME: Provides typed interfaces for all holodeck MCP server interactions

import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';
import type { 
  CoordinatorStatusResponse, 
  SystemHealthResponse, 
  ServerInfo,
  AppInfoResponse,
  ValidationResult 
} from '../types/api';

export const useTauri = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleInvoke = async <T>(command: string, args?: any): Promise<T> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<T>(command, args);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  return {
    isLoading,
    error,
    clearError: () => setError(null),
    
    // Coordinator Operations
    connectCoordinator: () => 
      handleInvoke<string>('connect_coordinator'),
    
    getCoordinatorStatus: () => 
      handleInvoke<CoordinatorStatusResponse>('get_coordinator_status'),
    
    // Session Management
    createHolodeckSession: (sessionName: string, storyTemplate: string, userId: string) =>
      handleInvoke<string>('create_holodeck_session', { 
        session_name: sessionName, 
        story_template: storyTemplate,
        user_id: userId 
      }),
    
    // System Health
    getSystemHealth: () =>
      handleInvoke<SystemHealthResponse>('get_system_health'),
    
    discoverServers: () =>
      handleInvoke<ServerInfo[]>('discover_servers'),
    
    // Validation
    orchestrateValidation: (storyContent: any) =>
      handleInvoke<ValidationResult>('orchestrate_validation', { story_content: storyContent }),
    
    // App Info
    getAppInfo: () =>
      handleInvoke<AppInfoResponse>('get_app_info'),
  };
};

export const useCoordinatorStatus = () => {
  const [status, setStatus] = useState<CoordinatorStatusResponse | null>(null);
  const { getCoordinatorStatus } = useTauri();

  const refreshStatus = async () => {
    try {
      const result = await getCoordinatorStatus();
      setStatus(result);
    } catch (err) {
      console.error('Failed to get coordinator status:', err);
    }
  };

  useEffect(() => {
    refreshStatus();
    const interval = setInterval(refreshStatus, 5000); // Poll every 5 seconds
    return () => clearInterval(interval);
  }, []);

  return { status, refreshStatus };
};

export const useSystemHealth = () => {
  const [health, setHealth] = useState<SystemHealthResponse | null>(null);
  const { getSystemHealth } = useTauri();

  const refreshHealth = async () => {
    try {
      const result = await getSystemHealth();
      setHealth(result);
    } catch (err) {
      console.error('Failed to get system health:', err);
    }
  };

  useEffect(() => {
    refreshHealth();
    const interval = setInterval(refreshHealth, 10000); // Poll every 10 seconds
    return () => clearInterval(interval);
  }, []);

  return { health, refreshHealth };
};