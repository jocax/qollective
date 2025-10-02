// ABOUTME: React application entry point for holodeck desktop client
// ABOUTME: Mounts the main App component with Enterprise LCARS theme and Tauri setup

import { createRoot } from 'react-dom/client';
import StepByStepApp from './StepByStepApp';
import TauriSetup from './utils/TauriSetup';
import './styles/enterprise-theme.css';

console.log('🚀 Holodeck Desktop starting...');

// Initialize Tauri setup if running in Tauri context
if (TauriSetup.isTauri()) {
  TauriSetup.initialize().then(() => {
    console.log('✅ Holodeck Desktop - Tauri initialized');
  }).catch(error => {
    console.error('❌ Holodeck Desktop - Tauri initialization failed:', error);
  });
}

const container = document.getElementById('root');
if (!container) {
  console.error('Root element not found!');
  document.body.innerHTML = '<div style="color: red; font-family: monospace; padding: 20px;">Root element not found!</div>';
} else {
  const root = createRoot(container);
  console.log('React root created, rendering App...');
  
  try {
    root.render(<StepByStepApp />);
    console.log('✅ App rendered successfully');
  } catch (error) {
    console.error('❌ Error rendering App:', error);
    root.render(
      <div style={{ 
        color: '#ff9900', 
        backgroundColor: '#000', 
        fontFamily: 'monospace', 
        padding: '20px',
        minHeight: '100vh'
      }}>
        <h1>🚨 Holodeck System Error</h1>
        <p>Failed to load application: {String(error)}</p>
      </div>
    );
  }
}