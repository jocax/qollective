// ABOUTME: Main React application with complete Phase 4 holodeck UI navigation and state management
// ABOUTME: Orchestrates all 7 use case flows with Enterprise theme and mock data integration

import React, { useState } from 'react';
import { DarkModeProvider } from './contexts/DarkModeContext';
import SplashScreen from './screens/SplashScreen';
import WelcomeScreen from './screens/WelcomeScreen';
import PrepareStoryScreen from './screens/PrepareStoryScreen';
import PlaySceneScreen from './screens/PlaySceneScreen';
import StoryHistoryScreen from './screens/StoryHistoryScreen';
import LiveInformationScreen from './screens/LiveInformationScreen';
import { HolodeckMcpService } from './services/HolodeckMcpService';
import type { 
  WelcomeData, 
  PrepareStoryData, 
  StoryBook, 
  StoryTemplate 
} from './types/holodeck';

// Application state types
export type AppScreen = 
  | 'splash'
  | 'welcome' 
  | 'prepare-story'
  | 'loading-story'
  | 'play-scene'
  | 'story-history'
  | 'live-info'
  | 'main-menu';

interface ApplicationState {
  currentScreen: AppScreen;
  currentSession: StoryBook | null;
  welcomeData: WelcomeData | null;
  prepareData: PrepareStoryData | null;
  storyTemplate: StoryTemplate | null;
  isLoading: boolean;
  error: string | null;
}

// Main Menu component
const MainMenuScreen: React.FC<{
  onNavigate: (screen: AppScreen) => void;
  welcomeData: WelcomeData | null;
}> = ({ onNavigate, welcomeData }) => {
  return (
    <div className="main-menu-screen min-h-screen bg-bg-primary p-8">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8 fade-in">
          <h1 className="enterprise-title text-4xl md:text-5xl mb-4">
            HOLODECK COMMAND CENTER
          </h1>
          <p className="enterprise-subtitle text-lg">
            Enterprise Edition • NCC-1701-D
          </p>
          {welcomeData && (
            <p className="text-text-secondary mt-2">
              Welcome back, {welcomeData.playerName}
            </p>
          )}
        </div>

        {/* Main Menu Options */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          <div 
            className="enterprise-card cursor-pointer"
            onClick={() => onNavigate('welcome')}
          >
            <div className="text-6xl mb-4">🚀</div>
            <h3 className="text-xl font-semibold text-enterprise-gold mb-2">
              New Mission
            </h3>
            <p className="text-text-secondary">
              Start a new holodeck adventure with custom characters and scenarios
            </p>
          </div>

          <div 
            className="enterprise-card cursor-pointer"
            onClick={() => onNavigate('story-history')}
          >
            <div className="text-6xl mb-4">📚</div>
            <h3 className="text-xl font-semibold text-enterprise-gold mb-2">
              Mission Archive
            </h3>
            <p className="text-text-secondary">
              Review your past adventures and continue where you left off
            </p>
          </div>

          <div 
            className="enterprise-card cursor-pointer"
            onClick={() => onNavigate('live-info')}
          >
            <div className="text-6xl mb-4">📊</div>
            <h3 className="text-xl font-semibold text-enterprise-gold mb-2">
              System Monitor
            </h3>
            <p className="text-text-secondary">
              Real-time holodeck system status and performance metrics
            </p>
          </div>

          <div className="enterprise-card opacity-50 cursor-not-allowed">
            <div className="text-6xl mb-4">⚙️</div>
            <h3 className="text-xl font-semibold text-text-secondary mb-2">
              Configuration
            </h3>
            <p className="text-text-muted">
              System settings and holodeck parameters (Coming Soon)
            </p>
          </div>
        </div>

        {/* Enterprise Footer */}
        <div className="text-center">
          <div className="enterprise-footer">
            <p className="text-enterprise-gold font-mono text-sm">
              STARFLEET COMMAND • HOLODECK CONTROL SYSTEM • ENTERPRISE NCC-1701-D
            </p>
            <p className="text-text-muted text-xs mt-1">
              United Federation of Planets • Authorized Personnel Only
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

// Loading screen component
const LoadingStoryScreen: React.FC<{
  prepareData: PrepareStoryData;
  onComplete: (template: StoryTemplate) => void;
  onError: (error: string) => void;
}> = ({ prepareData, onComplete, onError }) => {
  const [progress, setProgress] = React.useState(0);
  const [currentStep, setCurrentStep] = React.useState('');
  const [isComplete, setIsComplete] = React.useState(false);
  const [storyTemplate, setStoryTemplate] = React.useState<StoryTemplate | null>(null);

  React.useEffect(() => {
    const generateStory = async () => {
      const mcpService = HolodeckMcpService.getInstance();
      
      const steps = [
        'Analyzing mission parameters...',
        'Generating story framework...',
        'Creating character interactions...',
        'Designing scene progressions...',
        'Validating safety protocols...',
        'Finalizing holodeck program...'
      ];

      try {
        // Progress through each step with proper progress bar updates
        for (let i = 0; i < steps.length; i++) {
          setCurrentStep(steps[i]!);
          setProgress(((i + 1) / (steps.length + 1)) * 100); // +1 for the final generation step
          await new Promise(resolve => setTimeout(resolve, 800 + Math.random() * 1200));
        }

        // Final step: actual story generation
        setCurrentStep('Generating holodeck program...');
        setProgress(90);
        
        const template = await mcpService.generateStoryTemplate(prepareData);
        
        // Complete with 100% progress
        setProgress(100);
        setCurrentStep('Story generation complete!');
        setStoryTemplate(template);
        setIsComplete(true);
        
      } catch (error) {
        onError('Failed to generate story: ' + (error as Error).message);
      }
    };

    generateStory();
  }, [prepareData, onError]);

  const handleBeginAdventure = () => {
    if (storyTemplate) {
      onComplete(storyTemplate);
    }
  };

  return (
    <div className="loading-story-screen min-h-screen bg-bg-primary flex items-center justify-center">
      <div className="text-center max-w-lg">
        {!isComplete ? (
          <div className="enterprise-loading mx-auto mb-6 text-6xl"></div>
        ) : (
          <div className="text-6xl mb-6 text-enterprise-gold">✓</div>
        )}
        
        <h2 className="enterprise-title text-2xl mb-4">
          {isComplete ? 'Adventure Ready!' : 'Generating Your Adventure'}
        </h2>
        
        <p className="text-text-secondary mb-6">
          {currentStep}
        </p>
        
        <div className="w-full bg-bg-secondary rounded-full h-3 overflow-hidden mb-4">
          <div 
            className="h-full bg-gradient-gold transition-all duration-500"
            style={{ width: `${progress}%` }}
          />
        </div>
        
        <p className="text-sm text-text-muted mb-6">
          Topic: {prepareData.topic} • {prepareData.sceneCount} scenes • {prepareData.selectedCharacters.length} characters
        </p>
        
        {isComplete && (
          <div className="space-y-4">
            <div className="enterprise-panel p-4 mb-4">
              <p className="text-text-secondary text-sm mb-2">
                Your holodeck program has been successfully generated and validated.
              </p>
              <p className="text-enterprise-gold text-xs">
                Ready to begin your {prepareData.topic.toLowerCase()} adventure.
              </p>
            </div>
            
            <button
              onClick={handleBeginAdventure}
              className="enterprise-button primary text-lg px-8 py-3"
              style={{ cursor: 'pointer' }}
            >
              🚀 Begin Adventure
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

const App: React.FC = () => {
  const [appState, setAppState] = useState<ApplicationState>({
    currentScreen: 'splash',
    currentSession: null,
    welcomeData: null,
    prepareData: null,
    storyTemplate: null,
    isLoading: false,
    error: null,
  });

  const mcpService = HolodeckMcpService.getInstance();

  // Navigation handler
  const navigateToScreen = (screen: AppScreen) => {
    setAppState(prev => ({
      ...prev,
      currentScreen: screen,
      error: null,
    }));
  };

  // Handler for splash screen completion
  const handleSplashComplete = () => {
    navigateToScreen('main-menu');
  };

  // Handler for welcome screen data
  const handleWelcomeData = async (data: WelcomeData) => {
    setAppState(prev => ({
      ...prev,
      welcomeData: data,
      currentScreen: 'prepare-story',
    }));
  };

  // Handler for story preparation data
  const handlePrepareStoryData = async (data: PrepareStoryData) => {
    setAppState(prev => ({
      ...prev,
      prepareData: data,
      currentScreen: 'loading-story',
    }));
  };

  // Handler for story template generation
  const handleStoryTemplateGenerated = async (template: StoryTemplate) => {
    console.log('🎯 handleStoryTemplateGenerated called with template:', template);
    
    try {
      console.log('📝 Step 1: Creating holodeck session...');
      // Create holodeck session
      const holodeck = await mcpService.createHolodeck(appState.welcomeData!);
      console.log('✅ Step 1 complete: Holodeck created:', holodeck);
      
      console.log('📝 Step 2: Creating story session...');
      // Create story session
      const session = await mcpService.createStorySession(template, appState.welcomeData!.playerName);
      console.log('✅ Step 2 complete: Story session created:', session);
      
      console.log('📝 Step 3: Transitioning to play-scene screen...');
      setAppState(prev => ({
        ...prev,
        storyTemplate: template,
        currentSession: session,
        currentScreen: 'play-scene',
      }));
      console.log('✅ Step 3 complete: Transitioned to play-scene');
    } catch (error) {
      console.error('❌ handleStoryTemplateGenerated failed:', error);
      setAppState(prev => ({
        ...prev,
        error: 'Failed to create story session: ' + (error as Error).message,
        currentScreen: 'main-menu',
      }));
    }
  };

  // Handler for story generation error
  const handleStoryGenerationError = (error: string) => {
    setAppState(prev => ({
      ...prev,
      error,
      currentScreen: 'main-menu',
    }));
  };

  // Handler for session completion
  const handleSessionComplete = () => {
    navigateToScreen('story-history');
  };

  // Handler for session resume
  const handleResumeSession = (session: StoryBook) => {
    setAppState(prev => ({
      ...prev,
      currentSession: session,
      currentScreen: 'play-scene',
    }));
  };

  // Handler for back to menu
  const handleBackToMenu = () => {
    navigateToScreen('main-menu');
  };

  // Error handler - Removed auto-dismiss timer
  React.useEffect(() => {
    if (appState.error) {
      console.error('Application error:', appState.error);
    }
  }, [appState.error]);

  // Handler to manually dismiss error
  const dismissError = () => {
    setAppState(prev => ({
      ...prev,
      error: null,
    }));
  };

  // Render the current screen
  const renderCurrentScreen = () => {
    switch (appState.currentScreen) {
      case 'splash':
        return <SplashScreen onComplete={handleSplashComplete} />;

      case 'main-menu':
        return (
          <MainMenuScreen 
            onNavigate={navigateToScreen}
            welcomeData={appState.welcomeData}
          />
        );

      case 'welcome':
        return (
          <WelcomeScreen 
            onProceed={handleWelcomeData}
          />
        );

      case 'prepare-story':
        return (
          <PrepareStoryScreen 
            initialTopic={appState.welcomeData?.storyTopic || ''}
            onProceed={handlePrepareStoryData}
            onBack={() => navigateToScreen('welcome')}
          />
        );

      case 'loading-story':
        return appState.prepareData ? (
          <LoadingStoryScreen 
            prepareData={appState.prepareData}
            onComplete={handleStoryTemplateGenerated}
            onError={handleStoryGenerationError}
          />
        ) : null;

      case 'play-scene':
        return appState.currentSession ? (
          <PlaySceneScreen 
            session={appState.currentSession}
            onSessionComplete={handleSessionComplete}
            onBackToMenu={handleBackToMenu}
          />
        ) : null;

      case 'story-history':
        return (
          <StoryHistoryScreen 
            onResumeSession={handleResumeSession}
            onNewSession={() => navigateToScreen('welcome')}
            onBackToMenu={handleBackToMenu}
          />
        );

      case 'live-info':
        return (
          <LiveInformationScreen 
            onBackToMenu={handleBackToMenu}
          />
        );

      default:
        return <MainMenuScreen onNavigate={navigateToScreen} welcomeData={appState.welcomeData} />;
    }
  };

  return (
    <DarkModeProvider>
      <div className="app">
        {appState.error && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4">
            <div className="enterprise-panel panel-red max-w-2xl w-full">
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center gap-3">
                  <span className="text-status-dnd text-2xl">⚠</span>
                  <div>
                    <div className="font-bold text-enterprise-red text-lg mb-1">VALIDATION ERROR</div>
                    <div className="text-xs text-text-muted uppercase tracking-wide">
                      HOLODECK SYSTEM ALERT
                    </div>
                  </div>
                </div>
                <button
                  onClick={dismissError}
                  className="text-text-secondary hover:text-text-primary text-xl font-bold min-w-0 p-2 hover:bg-bg-secondary rounded"
                  style={{ cursor: 'pointer' }}
                  title="Close error message"
                >
                  ×
                </button>
              </div>
              
              <div className="bg-bg-secondary p-4 rounded border-l-4 border-l-enterprise-red mb-4">
                <div className="text-text-primary leading-relaxed">
                  {appState.error}
                </div>
              </div>
              
              <div className="flex justify-between items-center text-xs text-text-muted">
                <div>
                  Please review your input parameters and try again
                </div>
                <button
                  onClick={dismissError}
                  className="enterprise-button secondary text-xs px-4 py-2"
                  style={{ cursor: 'pointer' }}
                >
                  ACKNOWLEDGE
                </button>
              </div>
            </div>
          </div>
        )}
        
        {renderCurrentScreen()}
      </div>
    </DarkModeProvider>
  );
};

export default App;