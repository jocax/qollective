// ABOUTME: Step-by-step component loading to identify problematic imports
// ABOUTME: Tests each major component individually to isolate black screen cause

import React, { useState } from 'react';

// Step 1: Test DarkModeProvider import
const TestStep1: React.FC<{ onNext: () => void; onError: (error: string) => void }> = ({ onNext, onError }) => {
  React.useEffect(() => {
    const testImport = async () => {
      try {
        console.log('Testing DarkModeProvider import...');
        const { DarkModeProvider } = await import('./contexts/DarkModeContext');
        console.log('✅ DarkModeProvider imported successfully');
        setTimeout(onNext, 1000);
      } catch (error) {
        console.error('❌ DarkModeProvider import failed:', error);
        onError(`DarkModeProvider import failed: ${error}`);
      }
    };
    testImport();
  }, [onNext, onError]);

  return (
    <div style={{ 
      backgroundColor: '#1e40af', 
      color: '#ff9900', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>🔍 STEP 1: Testing DarkModeProvider Import</h1>
      <p>Loading DarkModeProvider...</p>
      <div style={{ fontSize: '24px', animation: 'pulse 1s infinite' }}>⏳</div>
    </div>
  );
};

// Step 2: Test SplashScreen import
const TestStep2: React.FC<{ onNext: () => void; onError: (error: string) => void }> = ({ onNext, onError }) => {
  React.useEffect(() => {
    const testImport = async () => {
      try {
        console.log('Testing SplashScreen import...');
        const SplashScreen = (await import('./screens/SplashScreen')).default;
        console.log('✅ SplashScreen imported successfully');
        setTimeout(onNext, 1000);
      } catch (error) {
        console.error('❌ SplashScreen import failed:', error);
        onError(`SplashScreen import failed: ${error}`);
      }
    };
    testImport();
  }, [onNext, onError]);

  return (
    <div style={{ 
      backgroundColor: '#1e40af', 
      color: '#ff9900', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>🔍 STEP 2: Testing SplashScreen Import</h1>
      <p>✅ DarkModeProvider OK</p>
      <p>Loading SplashScreen...</p>
      <div style={{ fontSize: '24px', animation: 'pulse 1s infinite' }}>⏳</div>
    </div>
  );
};

// Step 3: Test HolodeckMcpService import
const TestStep3: React.FC<{ onNext: () => void; onError: (error: string) => void }> = ({ onNext, onError }) => {
  React.useEffect(() => {
    const testImport = async () => {
      try {
        console.log('Testing HolodeckMcpService import...');
        const { HolodeckMcpService } = await import('./services/HolodeckMcpService');
        console.log('✅ HolodeckMcpService imported successfully');
        const service = HolodeckMcpService.getInstance();
        console.log('✅ HolodeckMcpService instance created');
        setTimeout(onNext, 1000);
      } catch (error) {
        console.error('❌ HolodeckMcpService import failed:', error);
        onError(`HolodeckMcpService import failed: ${error}`);
      }
    };
    testImport();
  }, [onNext, onError]);

  return (
    <div style={{ 
      backgroundColor: '#1e40af', 
      color: '#ff9900', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>🔍 STEP 3: Testing HolodeckMcpService Import</h1>
      <p>✅ DarkModeProvider OK</p>
      <p>✅ SplashScreen OK</p>
      <p>Loading HolodeckMcpService...</p>
      <div style={{ fontSize: '24px', animation: 'pulse 1s infinite' }}>⏳</div>
    </div>
  );
};

// Step 4: Test UI Components import
const TestStep4: React.FC<{ onNext: () => void; onError: (error: string) => void }> = ({ onNext, onError }) => {
  React.useEffect(() => {
    const testImport = async () => {
      try {
        console.log('Testing UI Components import...');
        const uiComponents = await import('./components/ui');
        console.log('✅ UI Components imported successfully');
        setTimeout(onNext, 1000);
      } catch (error) {
        console.error('❌ UI Components import failed:', error);
        onError(`UI Components import failed: ${error}`);
      }
    };
    testImport();
  }, [onNext, onError]);

  return (
    <div style={{ 
      backgroundColor: '#1e40af', 
      color: '#ff9900', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>🔍 STEP 4: Testing UI Components Import</h1>
      <p>✅ DarkModeProvider OK</p>
      <p>✅ SplashScreen OK</p>
      <p>✅ HolodeckMcpService OK</p>
      <p>Loading UI Components...</p>
      <div style={{ fontSize: '24px', animation: 'pulse 1s infinite' }}>⏳</div>
    </div>
  );
};

// Success screen that loads the full app
const TestSuccess: React.FC<{ onLoadApp: () => void }> = ({ onLoadApp }) => {
  const [loading, setLoading] = React.useState(false);
  const [AppComponent, setAppComponent] = React.useState<React.ComponentType | null>(null);

  const handleLoadApp = async () => {
    setLoading(true);
    try {
      console.log('Loading full holodeck App...');
      const AppModule = await import('./App');
      const App = AppModule.default;
      console.log('✅ Full holodeck App loaded successfully');
      setAppComponent(() => App);
    } catch (error) {
      console.error('❌ Failed to load full App:', error);
      alert(`Failed to load full App: ${error}`);
      setLoading(false);
    }
  };

  // If App is loaded, render it
  if (AppComponent) {
    return <AppComponent />;
  }

  return (
    <div style={{ 
      backgroundColor: '#16a34a', 
      color: '#fff', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>🎉 ALL IMPORTS SUCCESSFUL!</h1>
      <p>✅ DarkModeProvider OK</p>
      <p>✅ SplashScreen OK</p>
      <p>✅ HolodeckMcpService OK</p>
      <p>✅ UI Components OK</p>
      <br />
      <p>All components loaded successfully. The main App should work now.</p>
      <p style={{ fontSize: '14px', marginBottom: '20px' }}>
        Note: MCP servers are not running, so you'll see connection errors in the backend logs. 
        This is expected for testing the frontend.
      </p>
      {loading ? (
        <div style={{ fontSize: '18px' }}>
          🔄 Loading Full Holodeck Application...
        </div>
      ) : (
        <button onClick={handleLoadApp} style={{
          backgroundColor: '#fff',
          color: '#16a34a',
          padding: '15px 30px',
          border: 'none',
          fontSize: '16px',
          fontWeight: 'bold',
          borderRadius: '5px',
          cursor: 'pointer'
        }}>
          Load Full Holodeck App
        </button>
      )}
    </div>
  );
};

// Error screen
const TestError: React.FC<{ error: string; onRestart: () => void }> = ({ error, onRestart }) => {
  return (
    <div style={{ 
      backgroundColor: '#dc2626', 
      color: '#fff', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>❌ IMPORT ERROR DETECTED</h1>
      <div style={{ 
        backgroundColor: '#7f1d1d', 
        padding: '15px', 
        borderRadius: '5px',
        marginBottom: '20px',
        fontSize: '14px'
      }}>
        <pre style={{ whiteSpace: 'pre-wrap', margin: 0 }}>
          {error}
        </pre>
      </div>
      <p>This is the component causing the black screen issue.</p>
      <button onClick={onRestart} style={{
        backgroundColor: '#fff',
        color: '#dc2626',
        padding: '15px 30px',
        border: 'none',
        fontSize: '16px',
        fontWeight: 'bold',
        borderRadius: '5px',
        cursor: 'pointer'
      }}>
        Restart Test
      </button>
    </div>
  );
};

const StepByStepApp: React.FC = () => {
  const [currentStep, setCurrentStep] = useState(1);
  const [error, setError] = useState<string | null>(null);

  if (error) {
    return <TestError error={error} onRestart={() => { setError(null); setCurrentStep(1); }} />;
  }

  const handleLoadApp = async () => {
    try {
      console.log('Loading full App...');
      const App = (await import('./App')).default;
      // This would need to be handled by the parent component
      console.log('✅ Full App loaded successfully');
    } catch (error) {
      console.error('❌ Full App load failed:', error);
      setError(`Full App load failed: ${error}`);
    }
  };

  switch (currentStep) {
    case 1:
      return <TestStep1 onNext={() => setCurrentStep(2)} onError={setError} />;
    case 2:
      return <TestStep2 onNext={() => setCurrentStep(3)} onError={setError} />;
    case 3:
      return <TestStep3 onNext={() => setCurrentStep(4)} onError={setError} />;
    case 4:
      return <TestStep4 onNext={() => setCurrentStep(5)} onError={setError} />;
    case 5:
      return <TestSuccess onLoadApp={handleLoadApp} />;
    default:
      return <TestStep1 onNext={() => setCurrentStep(2)} onError={setError} />;
  }
};

export default StepByStepApp;