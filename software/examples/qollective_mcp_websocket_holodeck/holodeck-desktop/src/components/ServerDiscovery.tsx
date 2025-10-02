// ABOUTME: MCP server discovery and connection management interface
// ABOUTME: Displays available holodeck servers with connection capabilities

import React, { useState, useEffect } from 'react';
import { useTauri } from '../hooks/useTauri';
import type { ServerInfo } from '../types/api';

const ServerDiscovery: React.FC = () => {
  const [servers, setServers] = useState<ServerInfo[]>([]);
  const { discoverServers, connectCoordinator, isLoading } = useTauri();

  const discoverAndSetServers = async () => {
    try {
      const discovered = await discoverServers();
      setServers(discovered);
    } catch (err) {
      console.error('Failed to discover servers:', err);
    }
  };

  const handleConnectCoordinator = async () => {
    try {
      await connectCoordinator();
      // Refresh server list after connecting
      await discoverAndSetServers();
    } catch (err) {
      console.error('Failed to connect to coordinator:', err);
    }
  };

  useEffect(() => {
    discoverAndSetServers();
  }, []);

  const getServerTypeIcon = (name: string) => {
    if (name.includes('coordinator')) return '🎯';
    if (name.includes('validator')) return '✓';
    if (name.includes('environment')) return '🌍';
    if (name.includes('safety')) return '🛡️';
    if (name.includes('character')) return '👤';
    if (name.includes('storybook')) return '📚';
    if (name.includes('designer')) return '🎨';
    return '🔧';
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy': return 'var(--lcars-green)';
      case 'warning': return 'var(--lcars-yellow)';
      case 'error': case 'offline': return 'var(--lcars-red)';
      default: return 'var(--lcars-orange)';
    }
  };

  return (
    <div className="lcars-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
        <h3>SERVER DISCOVERY</h3>
        <div>
          <button 
            className="lcars-button secondary" 
            onClick={handleConnectCoordinator}
            disabled={isLoading}
          >
            {isLoading ? 'CONNECTING...' : 'CONNECT'}
          </button>
          <button 
            className="lcars-button" 
            onClick={discoverAndSetServers}
            disabled={isLoading}
          >
            {isLoading ? 'SCANNING...' : 'SCAN'}
          </button>
        </div>
      </div>

      {servers.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '20px' }}>
          <div className="loading-spinner"></div>
          <p>Scanning for available servers...</p>
        </div>
      ) : (
        <div>
          <p style={{ marginBottom: '16px', color: 'var(--lcars-green)' }}>
            DISCOVERED {servers.length} SERVERS
          </p>
          
          <div style={{ display: 'grid', gap: '12px' }}>
            {servers.map((server) => (
              <div 
                key={server.name}
                style={{
                  background: 'rgba(255, 153, 0, 0.05)',
                  border: '1px solid var(--lcars-orange)',
                  borderRadius: 'var(--border-radius-sm)',
                  padding: '12px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between'
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <span style={{ fontSize: '20px' }}>{getServerTypeIcon(server.name)}</span>
                  <div>
                    <div style={{ fontWeight: 'bold', color: 'var(--lcars-orange)' }}>
                      {server.name.toUpperCase()}
                    </div>
                    <div style={{ fontSize: '12px', fontFamily: 'var(--font-mono)', opacity: 0.8 }}>
                      {server.url}
                    </div>
                    <div style={{ display: 'flex', gap: '4px', marginTop: '4px' }}>
                      {server.capabilities.slice(0, 3).map(cap => (
                        <span 
                          key={cap}
                          style={{
                            background: 'var(--lcars-blue)',
                            color: 'var(--lcars-black)',
                            padding: '1px 4px',
                            borderRadius: '3px',
                            fontSize: '10px',
                            fontWeight: 'bold'
                          }}
                        >
                          {cap}
                        </span>
                      ))}
                      {server.capabilities.length > 3 && (
                        <span style={{ fontSize: '10px', opacity: 0.6 }}>
                          +{server.capabilities.length - 3} more
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span 
                    className="status-indicator"
                    style={{ backgroundColor: getStatusColor(server.status) }}
                  ></span>
                  <span style={{ fontSize: '12px', fontWeight: 'bold' }}>
                    {server.status.toUpperCase()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ServerDiscovery;