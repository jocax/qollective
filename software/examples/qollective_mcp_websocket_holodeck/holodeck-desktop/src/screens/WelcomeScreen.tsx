// ABOUTME: Welcome screen for user registration and story topic selection
// ABOUTME: Allows name input and choice between custom topics or predefined Star Trek scenarios

import React, { useState } from 'react';
import { Button, Input, Select, Panel } from '../components/ui';
import type { WelcomeData } from '../types/holodeck';

export interface WelcomeScreenProps {
  onProceed: (data: WelcomeData) => void;
}

const DEFAULT_TOPICS = [
  {
    value: "diplomatic-mission",
    label: "Diplomatic mission to negotiate peace treaty"
  },
  {
    value: "archaeological-expedition", 
    label: "Archaeological expedition to discover ancient artifacts"
  },
  {
    value: "space-exploration",
    label: "Space exploration mission to establish first contact"
  },
  {
    value: "medieval-intrigue",
    label: "Medieval court intrigue and alliance building"
  },
  {
    value: "temporal-investigation",
    label: "Scientific investigation of temporal anomalies"
  },
  {
    value: "detective-mystery",
    label: "Detective mystery in 1940s San Francisco"
  },
  {
    value: "survival-adventure",
    label: "Survival adventure on an uncharted planet"
  }
];

export const WelcomeScreen: React.FC<WelcomeScreenProps> = ({ onProceed }) => {
  const [playerName, setPlayerName] = useState('');
  const [storyTopic, setStoryTopic] = useState('');
  const [useDefaultTopic, setUseDefaultTopic] = useState(false);
  const [selectedDefaultTopic, setSelectedDefaultTopic] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!playerName.trim()) {
      newErrors.playerName = 'Please enter your name';
    } else if (playerName.trim().length < 2) {
      newErrors.playerName = 'Name must be at least 2 characters';
    } else if (playerName.trim().length > 50) {
      newErrors.playerName = 'Name must be less than 50 characters';
    }

    const topic = useDefaultTopic ? selectedDefaultTopic : storyTopic;
    if (!topic.trim()) {
      newErrors.topic = useDefaultTopic 
        ? 'Please select a story topic'
        : 'Please enter a story topic';
    } else if (!useDefaultTopic && topic.trim().length > 200) {
      newErrors.topic = 'Topic must be less than 200 characters';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = () => {
    if (!validateForm()) {
      return;
    }

    const finalTopic = useDefaultTopic 
      ? DEFAULT_TOPICS.find(t => t.value === selectedDefaultTopic)?.label || selectedDefaultTopic
      : storyTopic.trim();

    onProceed({
      playerName: playerName.trim(),
      storyTopic: finalTopic,
    });
  };

  const handlePlayerNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setPlayerName(e.target.value);
    if (errors.playerName) {
      setErrors(prev => ({ ...prev, playerName: '' }));
    }
  };

  const handleStoryTopicChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setStoryTopic(e.target.value);
    if (errors.topic) {
      setErrors(prev => ({ ...prev, topic: '' }));
    }
  };

  const handleDefaultTopicChange = (value: string) => {
    setSelectedDefaultTopic(value);
    if (errors.topic) {
      setErrors(prev => ({ ...prev, topic: '' }));
    }
  };

  const handleTopicModeChange = (useDefault: boolean) => {
    setUseDefaultTopic(useDefault);
    setErrors(prev => ({ ...prev, topic: '' }));
  };

  return (
    <div className="welcome-screen min-h-screen flex flex-col items-center justify-center bg-bg-primary p-8">
      <div className="w-full max-w-2xl">
        {/* Header */}
        <div className="text-center mb-8 fade-in">
          <h1 className="enterprise-title text-3xl md:text-4xl mb-4">
            Welcome to the Holodeck
          </h1>
          <p className="enterprise-subtitle text-lg">
            Prepare for your next adventure
          </p>
          <div className="mt-4 text-text-muted">
            Configure your holodeck experience and select your preferred story scenario
          </div>
        </div>

        {/* Main Form */}
        <Panel className="slide-in-right">
          <div className="space-y-6">
            {/* Player Name Section */}
            <div>
              <h3 className="enterprise-section-title mb-4">
                OFFICER IDENTIFICATION
              </h3>
              <Input
                id="player-name"
                label="Your Name"
                value={playerName}
                onChange={handlePlayerNameChange}
                placeholder="Enter your name..."
                maxLength={50}
                required
                error={errors.playerName}
              />
              <p className="text-xs text-text-muted mt-1">
                This name will be used throughout your holodeck experience
              </p>
            </div>

            {/* Story Topic Section */}
            <div>
              <h3 className="enterprise-section-title mb-4">
                MISSION PARAMETERS
              </h3>
              
              {/* Topic Mode Selection */}
              <div className="mb-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div 
                    className={`enterprise-card ${!useDefaultTopic ? 'selected' : ''}`}
                    onClick={() => handleTopicModeChange(false)}
                  >
                    <h4 className="font-semibold text-enterprise-gold mb-2">
                      Custom Mission
                    </h4>
                    <p className="text-sm text-text-secondary">
                      Create your own unique holodeck scenario
                    </p>
                  </div>
                  
                  <div 
                    className={`enterprise-card ${useDefaultTopic ? 'selected' : ''}`}
                    onClick={() => handleTopicModeChange(true)}
                  >
                    <h4 className="font-semibold text-enterprise-gold mb-2">
                      Starfleet Scenarios
                    </h4>
                    <p className="text-sm text-text-secondary">
                      Choose from pre-configured mission templates
                    </p>
                  </div>
                </div>
              </div>

              {/* Custom Topic Input */}
              {!useDefaultTopic && (
                <div className="fade-in">
                  <Input
                    id="story-topic"
                    label="Mission Description"
                    value={storyTopic}
                    onChange={handleStoryTopicChange}
                    placeholder="Enter your holodeck story topic..."
                    maxLength={200}
                    required
                    error={errors.topic}
                  />
                  <p className="text-xs text-text-muted mt-1">
                    Describe the type of adventure or scenario you'd like to experience
                  </p>
                </div>
              )}

              {/* Default Topic Selection */}
              {useDefaultTopic && (
                <div className="fade-in">
                  <Select
                    id="default-topic"
                    label="Select Mission Template"
                    value={selectedDefaultTopic}
                    onChange={handleDefaultTopicChange}
                    options={DEFAULT_TOPICS}
                    placeholder="Choose a story template..."
                    required
                    error={errors.topic}
                  />
                  <p className="text-xs text-text-muted mt-1">
                    Pre-designed scenarios crafted for optimal holodeck experiences
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Action Buttons - Moved to top */}
          <div className="flex flex-col sm:flex-row gap-4 mb-6 pb-6 border-b border-interactive-normal">
            <Button
              variant="secondary"
              className="flex-1"
              onClick={() => window.location.reload()}
            >
              Reset Form
            </Button>
            <Button
              variant="primary"
              className="flex-1"
              onClick={handleSubmit}
              disabled={!playerName.trim() || (!storyTopic.trim() && !selectedDefaultTopic)}
            >
              Begin Adventure
            </Button>
          </div>
        </Panel>

        {/* Tips Panel */}
        <Panel variant="blue" className="mt-6 fade-in">
          <h3 className="enterprise-section-title mb-3">
            MISSION BRIEFING
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div>
              <h4 className="text-enterprise-blue font-semibold mb-2">Custom Missions</h4>
              <ul className="text-text-secondary space-y-1 text-xs">
                <li>• Create unique storylines and scenarios</li>
                <li>• AI will generate appropriate characters and settings</li>
                <li>• Unlimited creative possibilities</li>
              </ul>
            </div>
            <div>
              <h4 className="text-enterprise-blue font-semibold mb-2">Starfleet Scenarios</h4>
              <ul className="text-text-secondary space-y-1 text-xs">
                <li>• Carefully crafted Star Trek experiences</li>
                <li>• Balanced difficulty and engagement</li>
                <li>• Authentic TNG crew interactions</li>
              </ul>
            </div>
          </div>
        </Panel>

        {/* Footer */}
        <div className="text-center mt-6">
          <p className="text-xs text-text-muted">
            All holodeck experiences are monitored by safety protocols
          </p>
        </div>
      </div>
    </div>
  );
};

export default WelcomeScreen;