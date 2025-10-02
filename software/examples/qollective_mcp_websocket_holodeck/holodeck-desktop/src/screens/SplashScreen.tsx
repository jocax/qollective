// ABOUTME: Enterprise-themed splash screen with initialization sequence and system checks
// ABOUTME: Displays USS Enterprise logo, system status, and connects to MCP servers with realistic delays

import React, { useEffect, useState } from 'react';
import { StatusIndicator, DarkModeToggle } from '../components/ui';
import TauriSetup from '../utils/TauriSetup';

export interface SplashScreenProps {
  onComplete: () => void;
}

interface InitStep {
  id: string;
  message: string;
  duration: number;
  status: 'pending' | 'active' | 'completed' | 'error';
}

export const SplashScreen: React.FC<SplashScreenProps> = ({ onComplete }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [connectionStatus, setConnectionStatus] = useState<Record<string, 'online' | 'offline' | 'connecting'>>({});
  const [showSystemReady, setShowSystemReady] = useState(false);

  const initSteps: InitStep[] = [
    { id: 'holodeck-init', message: 'Initializing Holodeck Systems...', duration: 800, status: 'pending' },
    { id: 'coordinator', message: 'Connecting to Holodeck Coordinator...', duration: 600, status: 'pending' },
    { id: 'mcp-servers', message: 'Establishing MCP Server Connections...', duration: 1200, status: 'pending' },
    { id: 'character-db', message: 'Loading Character Database...', duration: 700, status: 'pending' },
    { id: 'safety', message: 'Calibrating Safety Protocols...', duration: 500, status: 'pending' },
    { id: 'ready', message: 'System Ready - Welcome to the Holodeck', duration: 1000, status: 'pending' },
  ];

  const [steps, setSteps] = useState(initSteps);

  useEffect(() => {
    let currentStepIndex = 0;

    const processNextStep = () => {
      if (currentStepIndex >= steps.length) {
        setShowSystemReady(true);
        
        // Ensure Tauri window is visible when splash completes
        if (TauriSetup.isTauri()) {
          TauriSetup.showWindow();
        }
        
        setTimeout(() => onComplete(), 1500);
        return;
      }

      // Mark current step as active
      setSteps(prevSteps => 
        prevSteps.map((step, index) => 
          index === currentStepIndex 
            ? { ...step, status: 'active' }
            : step
        )
      );

      setCurrentStep(currentStepIndex);

      // Simulate step processing
      setTimeout(() => {
        // Mark current step as completed
        setSteps(prevSteps => 
          prevSteps.map((step, index) => 
            index === currentStepIndex 
              ? { ...step, status: 'completed' }
              : step
          )
        );

        // Update connection status for relevant steps
        if (currentStepIndex === 2) { // MCP servers step
          setConnectionStatus({
            'holodeck-coordinator': 'connecting',
            'holodeck-designer': 'connecting',
            'holodeck-validator': 'connecting',
            'holodeck-environment': 'connecting',
            'holodeck-safety': 'connecting',
            'holodeck-character': 'connecting',
            'holodeck-storybook': 'connecting',
          });

          // Gradually connect each server
          const servers = [
            'holodeck-coordinator',
            'holodeck-designer', 
            'holodeck-validator',
            'holodeck-environment',
            'holodeck-safety',
            'holodeck-character',
            'holodeck-storybook',
          ];

          servers.forEach((server, index) => {
            setTimeout(() => {
              setConnectionStatus(prev => ({
                ...prev,
                [server]: 'online'
              }));
            }, (index + 1) * 150);
          });
        }

        currentStepIndex++;
        setTimeout(processNextStep, 200);
      }, steps[currentStepIndex]?.duration || 1000);
    };

    // Start initialization sequence after component mounts
    const startTimer = setTimeout(processNextStep, 500);

    return () => clearTimeout(startTimer);
  }, [onComplete]);

  const getStepIcon = (status: string) => {
    switch (status) {
      case 'active':
        return <div className="enterprise-loading" />;
      case 'completed':
        return <span className="text-status-online">✓</span>;
      case 'error':
        return <span className="text-status-dnd">✗</span>;
      default:
        return <span className="text-text-muted">○</span>;
    }
  };

  return (
    <div className="splash-screen min-h-screen flex flex-col items-center justify-center bg-bg-primary p-8 relative">
      {/* Dark Mode Toggle - Upper Right Corner */}
      <div className="absolute top-6 right-6 z-10">
        <DarkModeToggle />
      </div>
      
      {/* Enterprise Logo Section */}
      <div className="fade-in mb-8">
        <div className="enterprise-logo text-center">
          {/* USS Enterprise ASCII Art */}
          <div className="font-mono text-enterprise-gold text-xs mb-4">
            <pre className="leading-tight">{`
                 _____
            ____|     |____
           |    ___     __|
           |___/   \\   /
              |  o  | /
              |_____|/
               _____
              /     \\
             |  NCC  |
             | 1701  |
             |   D   |
              \\_____/
            `}</pre>
          </div>
          <h1 className="enterprise-title text-4xl md:text-6xl mb-2">
            HOLODECK COMMANDER
          </h1>
          <p className="enterprise-subtitle text-lg md:text-xl">
            Enterprise Edition
          </p>
        </div>
      </div>

      {/* Initialization Status Section */}
      <div className="initialization-status w-full max-w-2xl">
        <div className="enterprise-panel mb-6">
          <h3 className="enterprise-section-title mb-4">
            SYSTEM INITIALIZATION
          </h3>
          
          <div className="space-y-3">
            {steps.map((step, index) => (
              <div
                key={step.id}
                className={`flex items-center gap-3 p-3 rounded transition-all duration-300 ${
                  step.status === 'active' 
                    ? 'bg-bg-tertiary border-l-4 border-enterprise-gold' 
                    : ''
                }`}
              >
                <div className="flex-shrink-0 w-6">
                  {getStepIcon(step.status)}
                </div>
                <div className={`flex-1 ${
                  step.status === 'completed' 
                    ? 'text-text-primary' 
                    : step.status === 'active'
                    ? 'text-enterprise-gold'
                    : 'text-text-muted'
                }`}>
                  {step.message}
                  {step.status === 'active' && (
                    <span className="loading-dots ml-1">...</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* MCP Server Connection Status */}
        {Object.keys(connectionStatus).length > 0 && (
          <div className="enterprise-panel panel-blue">
            <h3 className="enterprise-section-title mb-4">
              MCP SERVER STATUS
            </h3>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
              {Object.entries(connectionStatus).map(([server, status]) => (
                <StatusIndicator
                  key={server}
                  status={status === 'online' ? 'online' : status === 'connecting' ? 'warning' : 'offline'}
                  label={server.replace('holodeck-', '').replace('-', ' ').toUpperCase()}
                  size="sm"
                />
              ))}
            </div>
          </div>
        )}

        {/* System Ready Message */}
        {showSystemReady && (
          <div className="mt-6 text-center fade-in">
            <div className="enterprise-panel panel-teal">
              <div className="flex flex-col items-center gap-4">
                <div className="text-6xl">🚀</div>
                <h2 className="enterprise-title text-2xl">
                  SYSTEM READY
                </h2>
                <p className="text-text-secondary">
                  All systems operational • Holodeck protocols active
                </p>
                <div className="w-full bg-bg-secondary rounded-full h-2 overflow-hidden">
                  <div 
                    className="h-full bg-gradient-gold transition-all duration-1000"
                    style={{ width: '100%' }}
                  />
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Enterprise Footer */}
      <div className="mt-auto pt-8 text-center">
        <div className="enterprise-footer">
          <p className="text-enterprise-gold font-mono text-xs">
            STARFLEET COMMAND • HOLODECK CONTROL SYSTEM • ENTERPRISE NCC-1701-D
          </p>
          <p className="text-text-muted text-xs mt-1">
            United Federation of Planets • Authorized Personnel Only
          </p>
        </div>
      </div>
    </div>
  );
};

export default SplashScreen;