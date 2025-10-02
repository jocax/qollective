// ABOUTME: Story configuration screen with character selection and story parameters
// ABOUTME: Allows users to configure scene count, language, story type, and select TNG crew members

import React, { useState } from 'react';
import { Button, Input, Select, Panel } from '../components/ui';
import type { PrepareStoryData, Character } from '../types/holodeck';
import { HolodeckStoryType } from '../types/holodeck';

export interface PrepareStoryScreenProps {
  initialTopic: string;
  onProceed: (data: PrepareStoryData) => void;
  onBack: () => void;
}

// Mock TNG Character Database with proper types
const AVAILABLE_CHARACTERS: Character[] = [
  {
    id: 'picard',
    name: 'Jean-Luc Picard',
    characterType: 'Captain' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['Starfleet' as any, 'Diplomacy' as any, 'History' as any, 'Archaeology' as any],
    relationships: [],
    currentMood: 'Calm' as any,
    status: 'Active' as any,
  },
  {
    id: 'riker',
    name: 'William Thomas Riker',
    characterType: 'FirstOfficer' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['Starfleet' as any, 'MilitaryTactics' as any, 'Navigation' as any],
    relationships: [],
    currentMood: 'Focused' as any,
    status: 'Active' as any,
  },
  {
    id: 'data',
    name: 'Data',
    characterType: 'Android' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['Technology' as any, 'Physics' as any, 'Engineering' as any, 'AlienCultures' as any],
    relationships: [],
    currentMood: 'Curious' as any,
    status: 'Active' as any,
  },
  {
    id: 'laforge',
    name: 'Geordi La Forge',
    characterType: 'ChiefEngineer' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['Engineering' as any, 'Technology' as any, 'Physics' as any],
    relationships: [],
    currentMood: 'Focused' as any,
    status: 'Active' as any,
  },
  {
    id: 'worf',
    name: 'Worf',
    characterType: 'ChiefOfSecurity' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['MilitaryTactics' as any, 'Starfleet' as any],
    relationships: [],
    currentMood: 'Serious' as any,
    status: 'Active' as any,
  },
  {
    id: 'troi',
    name: 'Deanna Troi',
    characterType: 'Counselor' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['Psychology' as any, 'Diplomacy' as any],
    relationships: [],
    currentMood: 'Contemplative' as any,
    status: 'Active' as any,
  },
  {
    id: 'crusher',
    name: 'Beverly Crusher',
    characterType: 'ChiefMedicalOfficer' as any,
    personality: {} as any,
    voiceConfig: {} as any,
    appearance: {} as any,
    knowledgeDomains: ['Medical' as any, 'Physics' as any],
    relationships: [],
    currentMood: 'Focused' as any,
    status: 'Active' as any,
  },
];

const STORY_TYPES = [
  { value: HolodeckStoryType.Adventure, label: 'Adventure' },
  { value: HolodeckStoryType.Mystery, label: 'Mystery' },
  { value: HolodeckStoryType.Drama, label: 'Drama' },
  { value: HolodeckStoryType.Educational, label: 'Educational' },
  { value: HolodeckStoryType.Historical, label: 'Historical' },
];

const LANGUAGES = [
  { value: 'English', label: 'English' },
  { value: 'Spanish', label: 'Español' },
  { value: 'French', label: 'Français' },
  { value: 'German', label: 'Deutsch' },
];

const WORDS_PER_SCENE = [
  { value: 'Short', label: 'Short (50-100 words)' },
  { value: 'Medium', label: 'Medium (100-200 words)' },
  { value: 'Long', label: 'Long (200-300 words)' },
];

export const PrepareStoryScreen: React.FC<PrepareStoryScreenProps> = ({
  initialTopic,
  onProceed,
  onBack,
}) => {
  const [topic, setTopic] = useState(initialTopic);
  const [sceneCount, setSceneCount] = useState('5');
  const [language, setLanguage] = useState('English');
  const [storyType, setStoryType] = useState<HolodeckStoryType>(HolodeckStoryType.Adventure);
  const [wordsPerScene, setWordsPerScene] = useState('Medium');
  const [selectedCharacters, setSelectedCharacters] = useState<Character[]>([]);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!topic.trim()) {
      newErrors.topic = 'Topic is required';
    }

    const sceneNum = parseInt(sceneCount);
    if (isNaN(sceneNum) || sceneNum < 1 || sceneNum > 10) {
      newErrors.sceneCount = 'Scene count must be between 1 and 10';
    }

    if (selectedCharacters.length === 0) {
      newErrors.characters = 'Please select at least one character';
    } else if (selectedCharacters.length > 5) {
      newErrors.characters = 'Maximum 5 characters allowed';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = () => {
    if (!validateForm()) {
      return;
    }

    onProceed({
      topic: topic.trim(),
      sceneCount: parseInt(sceneCount),
      language,
      storyType,
      wordsPerScene,
      selectedCharacters,
    });
  };

  const handleCharacterToggle = (character: Character) => {
    setSelectedCharacters(prev => {
      const isSelected = prev.some(c => c.id === character.id);
      if (isSelected) {
        return prev.filter(c => c.id !== character.id);
      } else if (prev.length < 5) {
        return [...prev, character];
      }
      return prev;
    });
    
    if (errors.characters) {
      setErrors(prev => ({ ...prev, characters: '' }));
    }
  };

  const isCharacterSelected = (character: Character): boolean => {
    return selectedCharacters.some(c => c.id === character.id);
  };

  return (
    <div className="prepare-story-screen min-h-screen bg-bg-primary p-8">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-6 fade-in">
          <h1 className="enterprise-title text-3xl md:text-4xl mb-4">
            Configure Your Mission
          </h1>
          <p className="enterprise-subtitle text-lg">
            Set mission parameters and select your crew
          </p>
        </div>

        {/* Navigation Buttons - Moved to top */}
        <div className="flex flex-col sm:flex-row gap-4 mb-8 max-w-lg mx-auto">
          <Button
            variant="secondary"
            className="flex-1"
            onClick={onBack}
          >
            ← Back to Topics
          </Button>
          <Button
            variant="primary"
            className="flex-1"
            onClick={handleSubmit}
            disabled={selectedCharacters.length === 0 || !topic.trim()}
          >
            Generate Story →
          </Button>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Story Configuration */}
          <div className="space-y-6">
            <Panel title="Mission Parameters">
              <div className="space-y-4">
                <Input
                  label="Mission Topic"
                  value={topic}
                  onChange={(e) => setTopic(e.target.value)}
                  placeholder="Enter your story topic..."
                  error={errors.topic}
                  maxLength={200}
                />

                <div className="grid grid-cols-2 gap-4">
                  <Input
                    label="Scene Count"
                    type="number"
                    value={sceneCount}
                    onChange={(e) => setSceneCount(e.target.value)}
                    placeholder="1-10"
                    error={errors.sceneCount}
                  />

                  <Select
                    label="Language"
                    value={language}
                    onChange={setLanguage}
                    options={LANGUAGES}
                  />
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <Select
                    label="Story Type"
                    value={storyType}
                    onChange={(value) => setStoryType(value as HolodeckStoryType)}
                    options={STORY_TYPES}
                  />

                  <Select
                    label="Scene Length"
                    value={wordsPerScene}
                    onChange={setWordsPerScene}
                    options={WORDS_PER_SCENE}
                  />
                </div>
              </div>
            </Panel>

            {/* Selected Characters Summary */}
            <Panel variant="blue" title={`Selected Crew (${selectedCharacters.length}/5)`}>
              {selectedCharacters.length === 0 ? (
                <p className="text-text-muted text-center py-4">
                  No crew members selected
                </p>
              ) : (
                <div className="space-y-2">
                  {selectedCharacters.map(character => (
                    <div 
                      key={character.id}
                      className="flex items-center justify-between p-2 bg-bg-secondary rounded"
                    >
                      <div>
                        <div className="font-semibold text-enterprise-gold">
                          {character.name}
                        </div>
                        <div className="text-xs text-text-secondary">
                          {character.characterType}
                        </div>
                      </div>
                      <Button
                        variant="danger"
                        size="sm"
                        onClick={() => handleCharacterToggle(character)}
                      >
                        Remove
                      </Button>
                    </div>
                  ))}
                </div>
              )}
              {errors.characters && (
                <p className="text-enterprise-red text-xs mt-2">{errors.characters}</p>
              )}
            </Panel>
          </div>

          {/* Character Selection */}
          <div>
            <Panel title="Select Your Crew">
              <div className="character-grid">
                {AVAILABLE_CHARACTERS.map(character => {
                  const selected = isCharacterSelected(character);
                  const disabled = !selected && selectedCharacters.length >= 5;
                  
                  return (
                    <div
                      key={character.id}
                      className={`character-card ${selected ? 'selected' : ''} ${disabled ? 'opacity-50' : ''}`}
                      onClick={() => !disabled && handleCharacterToggle(character)}
                    >
                      <div className="character-name">
                        {character.name}
                      </div>
                      <div className="character-details">
                        <div className="font-semibold text-enterprise-blue">
                          {character.characterType}
                        </div>
                        <div className="text-xs mb-2">
                          Status: {character.status}
                        </div>
                        <div className="text-xs">
                          Mood: {character.currentMood}
                        </div>
                        <div className="mt-2 text-xs">
                          <strong>Specialties:</strong>
                          <div className="flex flex-wrap gap-1 mt-1">
                            {character.knowledgeDomains.slice(0, 3).map(domain => (
                              <span 
                                key={domain}
                                className="bg-bg-tertiary px-2 py-1 rounded text-xs"
                              >
                                {domain}
                              </span>
                            ))}
                          </div>
                        </div>
                      </div>
                      {selected && (
                        <div className="absolute top-2 right-2 text-status-online text-lg">
                          ✓
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </Panel>
          </div>
        </div>


        {/* Information Panel */}
        <Panel variant="teal" className="mt-8">
          <h3 className="enterprise-section-title mb-3">
            MISSION BRIEFING
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
            <div>
              <h4 className="text-enterprise-teal font-semibold mb-2">Story Generation</h4>
              <p className="text-text-secondary text-xs">
                Your selected parameters will be used to generate a unique Star Trek adventure with branching storylines and character interactions.
              </p>
            </div>
            <div>
              <h4 className="text-enterprise-teal font-semibold mb-2">Crew Selection</h4>
              <p className="text-text-secondary text-xs">
                Choose 1-5 crew members who will participate in your adventure. Each character brings unique skills and perspectives to the story.
              </p>
            </div>
            <div>
              <h4 className="text-enterprise-teal font-semibold mb-2">Safety Protocols</h4>
              <p className="text-text-secondary text-xs">
                All holodeck experiences include safety monitoring and age-appropriate content filtering based on your preferences.
              </p>
            </div>
          </div>
        </Panel>
      </div>
    </div>
  );
};

export default PrepareStoryScreen;