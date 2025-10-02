// ABOUTME: System status panel showing holodeck server health and capabilities
// ABOUTME: Real-time monitoring of all MCP servers with Enterprise LCARS styling

import React from 'react';
import { useSystemHealth } from '../hooks/useTauri';

const SystemStatus: React.FC = () => {
  const { health, refreshHealth } = useSystemHealth();

  if (!health) {
    return (
      <div className="lcars-panel">
        <h3>SYSTEM STATUS</h3>
        <div className="loading-spinner"></div>
      </div>
    );
  }

  const getHealthColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy': return 'var(--lcars-green)';
      case 'warning': return 'var(--lcars-yellow)';
      case 'error': return 'var(--lcars-red)';
      default: return 'var(--lcars-orange)';
    }
  };

  return (
    <div className="lcars-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
        <h3>SYSTEM STATUS</h3>
        <button className="lcars-button" onClick={refreshHealth}>
          REFRESH
        </button>
      </div>
      
      <div style={{ marginBottom: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
          <span 
            className="status-indicator"
            style={{ backgroundColor: getHealthColor(health.overall_health) }}
          ></span>
          <strong>Overall Health: {health.overall_health.toUpperCase()}</strong>
        </div>
        <p>Connected Servers: {health.connected_servers} / {health.total_servers}</p>
      </div>

      <table className="data-table">
        <thead>
          <tr>
            <th>Server</th>
            <th>Status</th>
            <th>Capabilities</th>
            <th>Last Check</th>
          </tr>
        </thead>
        <tbody>
          {Object.values(health.server_details).map((server) => (
            <tr key={server.name}>
              <td style={{ fontWeight: 'bold' }}>{server.name}</td>
              <td>
                <span 
                  className="status-indicator"
                  style={{ backgroundColor: getHealthColor(server.status) }}
                ></span>
                {server.status.toUpperCase()}
              </td>
              <td>
                {server.capabilities.map(cap => (
                  <span 
                    key={cap}
                    style={{ 
                      background: 'var(--lcars-blue)', 
                      color: 'var(--lcars-black)',
                      padding: '2px 6px',
                      marginRight: '4px',
                      borderRadius: '4px',
                      fontSize: '11px'
                    }}
                  >
                    {cap}
                  </span>
                ))}
              </td>
              <td style={{ fontSize: '12px', fontFamily: 'var(--font-mono)' }}>
                {new Date(server.last_health_check).toLocaleTimeString()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default SystemStatus;