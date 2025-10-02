// ABOUTME: Header component with Enterprise LCARS styling and coordinator status
// ABOUTME: Displays application title and real-time connection status indicators

import React from 'react';
import { useCoordinatorStatus } from '../hooks/useTauri';

interface HeaderProps {
  appInfo?: {
    name: string;
    version: string;
  };
}

const Header: React.FC<HeaderProps> = ({ appInfo }) => {
  const { status } = useCoordinatorStatus();

  const getStatusClass = () => {
    if (!status) return 'status-connecting';
    return status.connected ? 'status-connected' : 'status-disconnected';
  };

  const getStatusText = () => {
    if (!status) return 'INITIALIZING';
    return status.connected ? 'ONLINE' : 'OFFLINE';
  };

  return (
    <header className="header-bar">
      <div className="header-title">
        {appInfo?.name || 'Holodeck Desktop'}
        <span className="enterprise-badge">NCC-1701-D</span>
      </div>
      
      <div className="header-status">
        <span className={`status-indicator ${getStatusClass()}`}></span>
        <span>COORDINATOR: {getStatusText()}</span>
        {appInfo?.version && (
          <span style={{ marginLeft: '20px', fontSize: '14px', opacity: 0.8 }}>
            v{appInfo.version}
          </span>
        )}
      </div>
    </header>
  );
};

export default Header;