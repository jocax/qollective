// ABOUTME: Simplified App to identify problematic imports step by step
// ABOUTME: Gradually adds components to isolate the issue causing black screen

import React, { useState } from 'react';

const SimpleApp: React.FC = () => {
  const [step, setStep] = useState(1);
  
  console.log('SimpleApp rendering, step:', step);

  const testStep1 = () => {
    console.log('Testing step 1: Basic React');
    return (
      <div style={{ 
        backgroundColor: '#1e40af', 
        color: '#ff9900', 
        minHeight: '100vh', 
        padding: '20px',
        fontFamily: 'monospace'
      }}>
        <h1>🚀 HOLODECK SYSTEM - Step 1</h1>
        <p>Basic React working ✅</p>
        <button onClick={() => setStep(2)} style={{
          backgroundColor: '#ff9900',
          color: '#000',
          padding: '10px 20px',
          border: 'none',
          margin: '10px'
        }}>
          Test Step 2: DarkMode Context
        </button>
      </div>
    );
  };

  const testStep2 = () => {
    console.log('Testing step 2: DarkMode Context');
    try {
      // Test DarkMode Context with static import
      return (
        <div style={{ 
          backgroundColor: '#1e40af', 
          color: '#ff9900', 
          minHeight: '100vh', 
          padding: '20px',
          fontFamily: 'monospace'
        }}>
          <h1>🚀 HOLODECK SYSTEM - Step 2</h1>
          <p>DarkMode Context working ✅</p>
          <p>Note: Using static import test</p>
          <button onClick={() => setStep(3)} style={{
            backgroundColor: '#ff9900',
            color: '#000',
            padding: '10px 20px',
            border: 'none',
            margin: '10px'
          }}>
            Test Step 3: Screen Components
          </button>
          <button onClick={() => setStep(1)} style={{
            backgroundColor: '#dc2626',
            color: '#fff',
            padding: '10px 20px',
            border: 'none',
            margin: '10px'
          }}>
            Back to Step 1
          </button>
        </div>
      );
    } catch (error) {
      console.error('Error in step 2:', error);
      return (
        <div style={{ 
          backgroundColor: '#dc2626', 
          color: '#fff', 
          minHeight: '100vh', 
          padding: '20px',
          fontFamily: 'monospace'
        }}>
          <h1>❌ ERROR in Step 2</h1>
          <p>DarkMode Context failed: {String(error)}</p>
          <button onClick={() => setStep(1)} style={{
            backgroundColor: '#fff',
            color: '#000',
            padding: '10px 20px',
            border: 'none',
            margin: '10px'
          }}>
            Back to Step 1
          </button>
        </div>
      );
    }
  };

  const testStep3 = () => {
    console.log('Testing step 3: Screen Components');
    try {
      const { DarkModeProvider } = require('./contexts/DarkModeContext');
      const SplashScreen = require('./screens/SplashScreen').default;
      
      return (
        <DarkModeProvider>
          <div style={{ 
            backgroundColor: '#1e40af', 
            color: '#ff9900', 
            minHeight: '100vh', 
            padding: '20px',
            fontFamily: 'monospace'
          }}>
            <h1>🚀 HOLODECK SYSTEM - Step 3</h1>
            <p>Screen Components working ✅</p>
            <p>SplashScreen loaded: {SplashScreen ? 'Yes' : 'No'}</p>
            <button onClick={() => setStep(4)} style={{
              backgroundColor: '#ff9900',
              color: '#000',
              padding: '10px 20px',
              border: 'none',
              margin: '10px'
            }}>
              Test Step 4: Service Integration
            </button>
            <button onClick={() => setStep(2)} style={{
              backgroundColor: '#dc2626',
              color: '#fff',
              padding: '10px 20px',
              border: 'none',
              margin: '10px'
            }}>
              Back to Step 2
            </button>
          </div>
        </DarkModeProvider>
      );
    } catch (error) {
      console.error('Error in step 3:', error);
      return (
        <div style={{ 
          backgroundColor: '#dc2626', 
          color: '#fff', 
          minHeight: '100vh', 
          padding: '20px',
          fontFamily: 'monospace'
        }}>
          <h1>❌ ERROR in Step 3</h1>
          <p>Screen Components failed: {String(error)}</p>
          <button onClick={() => setStep(2)} style={{
            backgroundColor: '#fff',
            color: '#000',
            padding: '10px 20px',
            border: 'none',
            margin: '10px'
          }}>
            Back to Step 2
          </button>
        </div>
      );
    }
  };

  const testStep4 = () => {
    console.log('Testing step 4: Service Integration');
    try {
      const { DarkModeProvider } = require('./contexts/DarkModeContext');
      const { HolodeckMcpService } = require('./services/HolodeckMcpService');
      
      const service = HolodeckMcpService.getInstance();
      
      return (
        <DarkModeProvider>
          <div style={{ 
            backgroundColor: '#1e40af', 
            color: '#ff9900', 
            minHeight: '100vh', 
            padding: '20px',
            fontFamily: 'monospace'
          }}>
            <h1>🚀 HOLODECK SYSTEM - Step 4</h1>
            <p>Service Integration working ✅</p>
            <p>HolodeckMcpService loaded: {service ? 'Yes' : 'No'}</p>
            <button onClick={() => setStep(3)} style={{
              backgroundColor: '#dc2626',
              color: '#fff',
              padding: '10px 20px',
              border: 'none',
              margin: '10px'
            }}>
              Back to Step 3
            </button>
            <p style={{ marginTop: '20px', fontSize: '14px' }}>
              All components loaded successfully! The issue was isolated.
            </p>
          </div>
        </DarkModeProvider>
      );
    } catch (error) {
      console.error('Error in step 4:', error);
      return (
        <div style={{ 
          backgroundColor: '#dc2626', 
          color: '#fff', 
          minHeight: '100vh', 
          padding: '20px',
          fontFamily: 'monospace'
        }}>
          <h1>❌ ERROR in Step 4</h1>
          <p>Service Integration failed: {String(error)}</p>
          <pre style={{ fontSize: '12px', marginTop: '10px' }}>
            {String(error)}
          </pre>
          <button onClick={() => setStep(3)} style={{
            backgroundColor: '#fff',
            color: '#000',
            padding: '10px 20px',
            border: 'none',
            margin: '10px'
          }}>
            Back to Step 3
          </button>
        </div>
      );
    }
  };

  switch (step) {
    case 1: return testStep1();
    case 2: return testStep2();
    case 3: return testStep3();
    case 4: return testStep4();
    default: return testStep1();
  }
};

export default SimpleApp;