// ABOUTME: Interactive holodeck scene play experience with real-time character dialogue
// ABOUTME: Displays scenes, character interactions, player decisions, and story progression

import React, { useState, useEffect } from 'react';
import { Button, Panel, StatusIndicator } from '../components/ui';
import { HolodeckMcpService } from '../services/HolodeckMcpService';
import type { StoryBook } from '../types/holodeck';

export interface PlaySceneScreenProps {
  session: StoryBook;
  onSessionComplete: () => void;
  onBackToMenu: () => void;
}

interface CurrentScene {
  id: string;
  title: string;
  description: string;
  environmentContext: string;
  characterDialogue: string[];
  availableDecisions: {
    id: string;
    text: string;
    consequence: string;
  }[];
}

interface SceneState {
  currentScene: CurrentScene | null;
  isLoading: boolean;
  isLastScene: boolean;
  decisionMade: string | null;
  showConsequence: boolean;
  sceneHistory: {
    title: string;
    decision: string;
    timestamp: Date;
  }[];
}

export const PlaySceneScreen: React.FC<PlaySceneScreenProps> = ({
  session,
  onSessionComplete,
  onBackToMenu,
}) => {
  const [sceneState, setSceneState] = useState<SceneState>({
    currentScene: null,
    isLoading: true,
    isLastScene: false,
    decisionMade: null,
    showConsequence: false,
    sceneHistory: [],
  });

  const mcpService = HolodeckMcpService.getInstance();

  useEffect(() => {
    loadNextScene();
  }, []);

  const loadNextScene = async (previousDecision?: string) => {
    setSceneState(prev => ({ ...prev, isLoading: true, showConsequence: false }));

    try {
      const result = await mcpService.getNextScene(session.id, previousDecision);
      
      setSceneState(prev => ({
        ...prev,
        currentScene: result.scene,
        isLastScene: result.isLastScene,
        isLoading: false,
        decisionMade: null,
      }));
    } catch (error) {
      console.error('Failed to load next scene:', error);
      setSceneState(prev => ({ ...prev, isLoading: false }));
    }
  };

  const handleDecision = async (decision: { id: string; text: string; consequence: string }) => {
    if (!sceneState.currentScene) return;

    // Record the decision
    await mcpService.recordDecision(session.id, sceneState.currentScene.id, decision.id);

    // Mark the scene as completed
    await mcpService.completeScene(session.id, {
      id: sceneState.currentScene.id,
      title: sceneState.currentScene.title,
      description: sceneState.currentScene.description,
      characterInteractions: sceneState.currentScene.characterDialogue,
      playerDecisions: [decision.text],
      completedAt: new Date(),
    });

    // Update scene history
    setSceneState(prev => ({
      ...prev,
      decisionMade: decision.id,
      showConsequence: true,
      sceneHistory: [
        ...prev.sceneHistory,
        {
          title: prev.currentScene?.title || 'Unknown Scene',
          decision: decision.text,
          timestamp: new Date(),
        }
      ]
    }));

    // Show consequence message
    setTimeout(() => {
      if (sceneState.isLastScene) {
        completeSession();
      } else {
        loadNextScene(decision.id);
      }
    }, 3000);
  };

  const completeSession = async () => {
    await mcpService.completeSession(session.id);
    onSessionComplete();
  };

  const renderDialogue = (dialogue: string[]) => {
    return dialogue.map((line, index) => {
      const [speaker = 'Unknown', ...messageParts] = line.split(':');
      const message = messageParts.join(':').trim();
      
      const getCharacterColor = (speaker: string) => {
        if (speaker.includes('Picard')) return 'text-enterprise-gold';
        if (speaker.includes('Riker')) return 'text-enterprise-blue';
        if (speaker.includes('Data')) return 'text-enterprise-teal';
        return 'text-text-primary';
      };

      return (
        <div key={index} className="mb-3 p-3 bg-bg-secondary rounded border-l-4 border-enterprise-gold">
          <div className={`font-semibold ${getCharacterColor(speaker)} mb-1`}>
            {speaker}
          </div>
          <div className="text-text-primary text-sm leading-relaxed">
            {message}
          </div>
        </div>
      );
    });
  };

  if (sceneState.isLoading) {
    return (
      <div className="play-scene-screen min-h-screen bg-bg-primary flex items-center justify-center">
        <Panel className="text-center">
          <div className="enterprise-loading mx-auto mb-4"></div>
          <h2 className="enterprise-title text-xl mb-2">Loading Scene</h2>
          <p className="text-text-secondary">Preparing holodeck environment...</p>
        </Panel>
      </div>
    );
  }

  return (
    <div className="play-scene-screen min-h-screen bg-bg-primary p-4">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-4">
            <Button variant="secondary" onClick={onBackToMenu}>
              ← Menu
            </Button>
            <div>
              <h1 className="text-xl font-bold text-enterprise-gold">
                {session.sessionName}
              </h1>
              <p className="text-sm text-text-secondary">
                Scene {session.playedScenes.length + 1} • {session.playerName}
              </p>
            </div>
          </div>
          
          {/* Session Progress */}
          <div className="flex items-center gap-4">
            <StatusIndicator status="online" label="Session Active" />
            <div className="text-right">
              <div className="text-sm text-text-secondary">
                Scenes: {session.playedScenes.length}
              </div>
              <div className="text-sm text-text-secondary">
                Decisions: {session.playerDecisions.length}
              </div>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 xl:grid-cols-4 gap-6">
          {/* Main Scene Area */}
          <div className="xl:col-span-3 space-y-6">
            {sceneState.currentScene && (
              <>
                {/* Scene Title */}
                <div className="text-center">
                  <h2 className="scene-title">
                    {sceneState.currentScene.title}
                  </h2>
                </div>

                {/* Environment Context */}
                <Panel variant="blue" title="Environment">
                  <p className="text-text-primary leading-relaxed">
                    {sceneState.currentScene.environmentContext}
                  </p>
                </Panel>

                {/* Scene Description */}
                <div className="scene-container">
                  <div className="scene-description">
                    {sceneState.currentScene.description}
                  </div>
                </div>

                {/* Character Dialogue */}
                {sceneState.currentScene.characterDialogue.length > 0 && (
                  <Panel title="Character Interactions">
                    <div className="space-y-3">
                      {renderDialogue(sceneState.currentScene.characterDialogue)}
                    </div>
                  </Panel>
                )}

                {/* Decision Display */}
                {!sceneState.showConsequence && !sceneState.isLastScene && (
                  <Panel variant="teal" title="Your Decision">
                    <p className="text-text-secondary mb-4">
                      What do you choose to do?
                    </p>
                    <div className="decision-options">
                      {sceneState.currentScene.availableDecisions.map(decision => (
                        <button
                          key={decision.id}
                          className="decision-button"
                          onClick={() => handleDecision(decision)}
                          disabled={!!sceneState.decisionMade}
                        >
                          <div className="font-semibold text-enterprise-teal mb-2">
                            {decision.text}
                          </div>
                          <div className="text-text-muted text-sm">
                            {decision.consequence}
                          </div>
                        </button>
                      ))}
                    </div>
                  </Panel>
                )}

                {/* Consequence Display */}
                {sceneState.showConsequence && (
                  <Panel variant="red" title="Consequence">
                    <div className="text-center">
                      <div className="enterprise-loading mx-auto mb-4"></div>
                      <p className="text-text-primary">
                        Processing your decision...
                      </p>
                      {sceneState.isLastScene ? (
                        <p className="text-enterprise-gold mt-2">
                          Preparing mission summary...
                        </p>
                      ) : (
                        <p className="text-text-secondary mt-2">
                          Loading next scene...
                        </p>
                      )}
                    </div>
                  </Panel>
                )}

                {/* Mission Complete */}
                {sceneState.isLastScene && !sceneState.showConsequence && (
                  <Panel className="text-center">
                    <div className="text-6xl mb-4">🎉</div>
                    <h2 className="enterprise-title text-2xl mb-4">
                      Mission Complete
                    </h2>
                    <p className="text-text-secondary mb-6">
                      Your holodeck adventure has concluded. Review your choices and their outcomes.
                    </p>
                    <div className="flex gap-4 justify-center">
                      <Button variant="secondary" onClick={onBackToMenu}>
                        Return to Menu
                      </Button>
                      <Button variant="primary" onClick={onSessionComplete}>
                        View Session Summary
                      </Button>
                    </div>
                  </Panel>
                )}
              </>
            )}
          </div>

          {/* Sidebar - Scene History & Progress */}
          <div className="space-y-6">
            {/* Current Status */}
            <Panel title="Mission Status">
              <div className="space-y-2">
                <StatusIndicator status="online" label="Holodeck Active" size="sm" />
                <StatusIndicator status="online" label="Safety Protocols" size="sm" />
                <StatusIndicator status="online" label="Character AI" size="sm" />
                <StatusIndicator status="warning" label="Story Generation" size="sm" />
              </div>
            </Panel>

            {/* Progress Tracking */}
            <Panel variant="blue" title="Progress">
              <div className="space-y-3">
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span>Scenes Completed</span>
                    <span>{session.playedScenes.length}</span>
                  </div>
                  <div className="progress-bar">
                    <div 
                      className="progress-fill"
                      style={{ width: `${Math.min(100, (session.playedScenes.length / 5) * 100)}%` }}
                    />
                  </div>
                </div>
                
                <div className="text-xs text-text-muted">
                  <div>Decisions Made: {session.playerDecisions.length}</div>
                  <div>Session Time: {Math.floor((Date.now() - session.startedAt.getTime()) / 60000)}m</div>
                </div>
              </div>
            </Panel>

            {/* Scene History */}
            {sceneState.sceneHistory.length > 0 && (
              <Panel variant="teal" title="Decision History">
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {sceneState.sceneHistory.map((entry, index) => (
                    <div key={index} className="p-2 bg-bg-secondary rounded text-xs">
                      <div className="font-semibold text-enterprise-teal mb-1">
                        {entry.title}
                      </div>
                      <div className="text-text-secondary mb-1">
                        {entry.decision}
                      </div>
                      <div className="text-text-muted">
                        {entry.timestamp.toLocaleTimeString()}
                      </div>
                    </div>
                  ))}
                </div>
              </Panel>
            )}

            {/* Quick Actions */}
            <Panel title="Quick Actions">
              <div className="space-y-2">
                <Button 
                  variant="secondary" 
                  size="sm" 
                  className="w-full"
                  onClick={() => {/* TODO: Implement save */}}
                >
                  Save Progress
                </Button>
                <Button 
                  variant="secondary" 
                  size="sm" 
                  className="w-full"
                  onClick={() => {/* TODO: Implement pause */}}
                >
                  Pause Session
                </Button>
                <Button 
                  variant="danger" 
                  size="sm" 
                  className="w-full"
                  onClick={onBackToMenu}
                >
                  Exit Mission
                </Button>
              </div>
            </Panel>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PlaySceneScreen;