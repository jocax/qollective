// ABOUTME: Story history and session management screen with replay functionality
// ABOUTME: Displays previous sessions, detailed progression, and resume/replay options

import React, { useState, useEffect } from 'react';
import { Button, Panel, StatusIndicator } from '../components/ui';
import { HolodeckMcpService } from '../services/HolodeckMcpService';
import type { StoryBook } from '../types/holodeck';
import { SessionStatus } from '../types/holodeck';

export interface StoryHistoryScreenProps {
  onResumeSession: (session: StoryBook) => void;
  onNewSession: () => void;
  onBackToMenu: () => void;
}

interface HistoryState {
  sessions: StoryBook[];
  selectedSession: StoryBook | null;
  isLoading: boolean;
  error: string | null;
}

export const StoryHistoryScreen: React.FC<StoryHistoryScreenProps> = ({
  onResumeSession,
  onNewSession,
  onBackToMenu,
}) => {
  const [historyState, setHistoryState] = useState<HistoryState>({
    sessions: [],
    selectedSession: null,
    isLoading: true,
    error: null,
  });

  const mcpService = HolodeckMcpService.getInstance();

  useEffect(() => {
    loadSessionHistory();
  }, []);

  const loadSessionHistory = async () => {
    try {
      setHistoryState(prev => ({ ...prev, isLoading: true, error: null }));
      const sessions = await mcpService.getSessionHistory();
      setHistoryState(prev => ({
        ...prev,
        sessions,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load session history:', error);
      setHistoryState(prev => ({
        ...prev,
        error: 'Failed to load session history',
        isLoading: false,
      }));
    }
  };

  const handleSessionSelect = (session: StoryBook) => {
    setHistoryState(prev => ({
      ...prev,
      selectedSession: prev.selectedSession?.id === session.id ? null : session,
    }));
  };

  const handleResumeSession = async (session: StoryBook) => {
    if (session.status === SessionStatus.Active || session.status === SessionStatus.Paused) {
      try {
        const resumedSession = await mcpService.resumeSession(session.id);
        onResumeSession(resumedSession);
      } catch (error) {
        console.error('Failed to resume session:', error);
        setHistoryState(prev => ({
          ...prev,
          error: 'Failed to resume session',
        }));
      }
    }
  };

  const getStatusColor = (status: SessionStatus): 'online' | 'warning' | 'error' | 'offline' => {
    switch (status) {
      case SessionStatus.Active:
      case SessionStatus.Completed:
        return 'online';
      case SessionStatus.Paused:
        return 'warning';
      case SessionStatus.Aborted:
      case SessionStatus.SafetyHalt:
        return 'error';
      default:
        return 'offline';
    }
  };

  const getStatusLabel = (status: SessionStatus): string => {
    switch (status) {
      case SessionStatus.Active:
        return 'ACTIVE';
      case SessionStatus.Paused:
        return 'PAUSED';
      case SessionStatus.Completed:
        return 'COMPLETED';
      case SessionStatus.Aborted:
        return 'ABORTED';
      case SessionStatus.SafetyHalt:
        return 'SAFETY HALT';
      default:
        return 'UNKNOWN';
    }
  };

  const formatDuration = (startTime: Date, endTime?: Date): string => {
    const end = endTime || new Date();
    const durationMs = end.getTime() - startTime.getTime();
    const minutes = Math.floor(durationMs / 60000);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    }
    return `${minutes}m`;
  };

  if (historyState.isLoading) {
    return (
      <div className="story-history-screen min-h-screen bg-bg-primary flex items-center justify-center">
        <Panel className="text-center">
          <div className="enterprise-loading mx-auto mb-4"></div>
          <h2 className="enterprise-title text-xl mb-2">Loading History</h2>
          <p className="text-text-secondary">Retrieving session records...</p>
        </Panel>
      </div>
    );
  }

  return (
    <div className="story-history-screen min-h-screen bg-bg-primary p-4">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-4">
            <Button variant="secondary" onClick={onBackToMenu}>
              ← Back
            </Button>
            <div>
              <h1 className="enterprise-title text-2xl">
                Mission Archive
              </h1>
              <p className="text-text-secondary">
                Review your holodeck adventures and continue where you left off
              </p>
            </div>
          </div>
          
          <Button variant="primary" onClick={onNewSession}>
            + New Mission
          </Button>
        </div>

        {/* Error Display */}
        {historyState.error && (
          <Panel variant="red" className="mb-6">
            <div className="flex items-center gap-2">
              <span className="text-status-dnd">⚠</span>
              <span>{historyState.error}</span>
            </div>
          </Panel>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Session List */}
          <div className="lg:col-span-2">
            <Panel title={`Session History (${historyState.sessions.length})`}>
              {historyState.sessions.length === 0 ? (
                <div className="text-center py-8">
                  <div className="text-6xl mb-4">📚</div>
                  <h3 className="text-lg font-semibold text-text-secondary mb-2">
                    No Sessions Found
                  </h3>
                  <p className="text-text-muted mb-4">
                    Start your first holodeck adventure to see it appear here
                  </p>
                  <Button onClick={onNewSession}>
                    Create First Session
                  </Button>
                </div>
              ) : (
                <div className="space-y-3">
                  {historyState.sessions.map(session => (
                    <div
                      key={session.id}
                      className={`enterprise-card cursor-pointer ${
                        historyState.selectedSession?.id === session.id ? 'selected' : ''
                      }`}
                      onClick={() => handleSessionSelect(session)}
                    >
                      <div className="flex items-start justify-between mb-3">
                        <div className="flex-1">
                          <h3 className="font-semibold text-enterprise-gold mb-1">
                            {session.sessionName}
                          </h3>
                          <p className="text-sm text-text-secondary">
                            Player: {session.playerName}
                          </p>
                        </div>
                        
                        <div className="flex flex-col items-end gap-1">
                          <StatusIndicator
                            status={getStatusColor(session.status)}
                            label={getStatusLabel(session.status)}
                            size="sm"
                          />
                          <span className="text-xs text-text-muted">
                            {session.lastPlayed.toLocaleDateString()}
                          </span>
                        </div>
                      </div>

                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs text-text-muted">
                        <div>
                          <span className="font-semibold">Scenes:</span> {session.playedScenes.length}
                        </div>
                        <div>
                          <span className="font-semibold">Decisions:</span> {session.playerDecisions.length}
                        </div>
                        <div>
                          <span className="font-semibold">Duration:</span> {formatDuration(session.startedAt, session.completedAt)}
                        </div>
                        <div>
                          <span className="font-semibold">Last Played:</span> {session.lastPlayed.toLocaleDateString()}
                        </div>
                      </div>

                      {session.status !== SessionStatus.Completed && (
                        <div className="mt-3 pt-3 border-t border-interactive-normal">
                          <Button
                            size="sm"
                            onClick={() => handleResumeSession(session)}
                            className="mr-2"
                          >
                            Resume Mission
                          </Button>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </Panel>
          </div>

          {/* Session Details */}
          <div>
            {historyState.selectedSession ? (
              <div className="space-y-4">
                <Panel variant="blue" title="Session Details">
                  <div className="space-y-3 text-sm">
                    <div>
                      <span className="font-semibold text-enterprise-blue">Player:</span>
                      <p className="text-text-primary">{historyState.selectedSession.playerName}</p>
                    </div>
                    
                    <div>
                      <span className="font-semibold text-enterprise-blue">Started:</span>
                      <p className="text-text-primary">
                        {historyState.selectedSession.startedAt.toLocaleString()}
                      </p>
                    </div>
                    
                    <div>
                      <span className="font-semibold text-enterprise-blue">Last Played:</span>
                      <p className="text-text-primary">
                        {historyState.selectedSession.lastPlayed.toLocaleString()}
                      </p>
                    </div>
                    
                    {historyState.selectedSession.completedAt && (
                      <div>
                        <span className="font-semibold text-enterprise-blue">Completed:</span>
                        <p className="text-text-primary">
                          {historyState.selectedSession.completedAt.toLocaleString()}
                        </p>
                      </div>
                    )}
                  </div>
                </Panel>

                <Panel variant="teal" title="Mission Statistics">
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span>Total Scenes:</span>
                      <span className="font-semibold">{historyState.selectedSession.playedScenes.length}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Decisions Made:</span>
                      <span className="font-semibold">{historyState.selectedSession.playerDecisions.length}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Avg. Response Time:</span>
                      <span className="font-semibold">
                        {Math.floor(historyState.selectedSession.sessionStatistics.averageResponseTimeMs / 1000)}s
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Characters Met:</span>
                      <span className="font-semibold">
                        {historyState.selectedSession.sessionStatistics.charactersInteractedWith.length}
                      </span>
                    </div>
                  </div>
                </Panel>

                {historyState.selectedSession.playedScenes.length > 0 && (
                  <Panel title="Scene History">
                    <div className="space-y-2 max-h-64 overflow-y-auto">
                      {historyState.selectedSession.playedScenes.map((scene, index) => (
                        <div key={scene.id} className="p-2 bg-bg-secondary rounded text-xs">
                          <div className="font-semibold text-enterprise-gold mb-1">
                            {scene.title}
                          </div>
                          <div className="text-text-muted mb-1">
                            {scene.description.substring(0, 100)}...
                          </div>
                          <div className="text-text-muted">
                            Completed: {scene.completedAt.toLocaleString()}
                          </div>
                        </div>
                      ))}
                    </div>
                  </Panel>
                )}

                {/* Session Actions */}
                <Panel>
                  <div className="space-y-2">
                    {historyState.selectedSession.status !== SessionStatus.Completed && (
                      <Button
                        className="w-full"
                        onClick={() => handleResumeSession(historyState.selectedSession!)}
                      >
                        Resume Mission
                      </Button>
                    )}
                    
                    <Button
                      variant="secondary"
                      size="sm"
                      className="w-full"
                      onClick={() => {/* TODO: Implement export */}}
                    >
                      Export Session
                    </Button>
                    
                    <Button
                      variant="danger"
                      size="sm"
                      className="w-full"
                      onClick={() => {/* TODO: Implement delete */}}
                    >
                      Delete Session
                    </Button>
                  </div>
                </Panel>
              </div>
            ) : (
              <Panel className="text-center">
                <div className="py-8">
                  <div className="text-4xl mb-4">📋</div>
                  <h3 className="text-lg font-semibold text-text-secondary mb-2">
                    Select a Session
                  </h3>
                  <p className="text-text-muted">
                    Click on a session to view detailed information and options
                  </p>
                </div>
              </Panel>
            )}
          </div>
        </div>

        {/* Summary Statistics */}
        {historyState.sessions.length > 0 && (
          <Panel variant="blue" className="mt-6">
            <h3 className="enterprise-section-title mb-4">
              MISSION ARCHIVE SUMMARY
            </h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
              <div>
                <div className="text-2xl font-bold text-enterprise-blue mb-1">
                  {historyState.sessions.length}
                </div>
                <div className="text-sm text-text-secondary">Total Sessions</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-enterprise-blue mb-1">
                  {historyState.sessions.filter(s => s.status === SessionStatus.Completed).length}
                </div>
                <div className="text-sm text-text-secondary">Completed</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-enterprise-blue mb-1">
                  {historyState.sessions.reduce((sum, s) => sum + s.playedScenes.length, 0)}
                </div>
                <div className="text-sm text-text-secondary">Total Scenes</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-enterprise-blue mb-1">
                  {historyState.sessions.reduce((sum, s) => sum + s.playerDecisions.length, 0)}
                </div>
                <div className="text-sm text-text-secondary">Total Decisions</div>
              </div>
            </div>
          </Panel>
        )}
      </div>
    </div>
  );
};

export default StoryHistoryScreen;