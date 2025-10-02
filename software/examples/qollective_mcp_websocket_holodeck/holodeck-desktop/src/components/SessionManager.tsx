// ABOUTME: Holodeck session management interface with story template selection
// ABOUTME: Creates and manages holodeck sessions with authentic TNG workflows

import React, { useState } from 'react';
import { useTauri } from '../hooks/useTauri';

const SessionManager: React.FC = () => {
  const [sessionName, setSessionName] = useState('');
  const [storyTemplate, setStoryTemplate] = useState('Dixon Hill Mystery');
  const [userId, setUserId] = useState('picard-001');
  const [result, setResult] = useState<string | null>(null);
  
  const { createHolodeckSession, orchestrateValidation, isLoading, error } = useTauri();

  const storyTemplates = [
    'Dixon Hill Mystery',
    'Sherlock Holmes - Baker Street',
    'Wild West Adventure',
    'Medieval Castle Siege',
    'Enterprise Recreation',
    'Risa Vacation Paradise',
    'Klingon Battle Simulation',
    'Deep Space Exploration'
  ];

  const handleCreateSession = async () => {
    if (!sessionName.trim()) {
      setResult('Please enter a session name');
      return;
    }

    try {
      const response = await createHolodeckSession(sessionName, storyTemplate, userId);
      setResult(response);
      setSessionName(''); // Clear form on success
    } catch (err) {
      console.error('Failed to create session:', err);
    }
  };

  const handleValidateStory = async () => {
    const storyContent = {
      template: storyTemplate,
      name: sessionName,
      user_preferences: {
        difficulty: 'standard',
        safety_level: 'enabled',
        duration: 60
      }
    };

    try {
      const validation = await orchestrateValidation(storyContent);
      const score = validation.overall_validation.aggregated_score;
      const success = validation.overall_validation.success;
      
      setResult(
        `Validation ${success ? 'PASSED' : 'FAILED'} - Score: ${score}/100\n` +
        `Story: ${validation.validation_results.story_validation.success ? '✓' : '✗'} ` +
        `Environment: ${validation.validation_results.environment_validation.success ? '✓' : '✗'} ` +
        `Safety: ${validation.validation_results.safety_validation.success ? '✓' : '✗'} ` +
        `Character: ${validation.validation_results.character_validation.success ? '✓' : '✗'}`
      );
    } catch (err) {
      console.error('Failed to validate story:', err);
    }
  };

  return (
    <div className="lcars-panel">
      <h3>HOLODECK SESSION MANAGER</h3>
      
      <div style={{ marginBottom: '20px' }}>
        <div style={{ marginBottom: '12px' }}>
          <label style={{ display: 'block', marginBottom: '4px', color: 'var(--lcars-orange)' }}>
            SESSION NAME:
          </label>
          <input
            type="text"
            value={sessionName}
            onChange={(e) => setSessionName(e.target.value)}
            placeholder="Enter session name..."
            className="lcars-input"
          />
        </div>

        <div style={{ marginBottom: '12px' }}>
          <label style={{ display: 'block', marginBottom: '4px', color: 'var(--lcars-orange)' }}>
            STORY TEMPLATE:
          </label>
          <select 
            value={storyTemplate}
            onChange={(e) => setStoryTemplate(e.target.value)}
            className="lcars-input"
          >
            {storyTemplates.map(template => (
              <option key={template} value={template}>{template}</option>
            ))}
          </select>
        </div>

        <div style={{ marginBottom: '12px' }}>
          <label style={{ display: 'block', marginBottom: '4px', color: 'var(--lcars-orange)' }}>
            USER ID:
          </label>
          <select 
            value={userId}
            onChange={(e) => setUserId(e.target.value)}
            className="lcars-input"
          >
            <option value="picard-001">Captain Jean-Luc Picard</option>
            <option value="riker-001">Commander William Riker</option>
            <option value="data-001">Lt. Commander Data</option>
            <option value="laforge-001">Lt. Commander Geordi La Forge</option>
            <option value="worf-001">Lt. Worf</option>
            <option value="troi-001">Counselor Deanna Troi</option>
            <option value="beverly-001">Dr. Beverly Crusher</option>
          </select>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '12px', marginBottom: '20px' }}>
        <button 
          className="lcars-button" 
          onClick={handleValidateStory}
          disabled={isLoading}
        >
          {isLoading ? 'VALIDATING...' : 'VALIDATE STORY'}
        </button>
        
        <button 
          className="lcars-button secondary" 
          onClick={handleCreateSession}
          disabled={isLoading}
        >
          {isLoading ? 'CREATING...' : 'CREATE SESSION'}
        </button>
      </div>

      {error && (
        <div style={{ 
          background: 'var(--lcars-red)', 
          color: 'var(--lcars-black)',
          padding: '8px 12px',
          borderRadius: 'var(--border-radius-sm)',
          marginBottom: '12px',
          fontWeight: 'bold'
        }}>
          ERROR: {error}
        </div>
      )}

      {result && (
        <div style={{ 
          background: 'rgba(255, 153, 0, 0.1)',
          border: '1px solid var(--lcars-orange)',
          padding: '12px',
          borderRadius: 'var(--border-radius-sm)',
          fontFamily: 'var(--font-mono)',
          fontSize: '14px',
          whiteSpace: 'pre-line'
        }}>
          {result}
        </div>
      )}
    </div>
  );
};

export default SessionManager;