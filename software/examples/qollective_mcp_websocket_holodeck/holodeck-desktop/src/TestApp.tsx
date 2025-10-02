// Simple test app to debug the black screen issue

import React from 'react';

const TestApp: React.FC = () => {
  console.log('TestApp rendering...');
  
  return (
    <div style={{ 
      backgroundColor: '#1e40af', 
      color: '#ff9900', 
      minHeight: '100vh', 
      padding: '20px',
      fontFamily: 'monospace'
    }}>
      <h1>🚀 Holodeck Test Application</h1>
      <p>If you can see this, React is working!</p>
      <p>Current time: {new Date().toLocaleString()}</p>
      <button onClick={() => alert('Button clicked!')}>
        Test Button
      </button>
    </div>
  );
};

export default TestApp;