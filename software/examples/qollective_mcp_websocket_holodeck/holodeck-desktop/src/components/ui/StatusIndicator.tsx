// ABOUTME: Enterprise-themed status indicator with colored dots and labels
// ABOUTME: Shows system status with animated dots and descriptive text

import React from 'react';

export interface StatusIndicatorProps {
  status: 'online' | 'warning' | 'error' | 'offline';
  label: string;
  animated?: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const StatusIndicator: React.FC<StatusIndicatorProps> = ({
  status,
  label,
  animated = true,
  size = 'md',
  className = '',
}) => {
  const statusClasses = {
    online: 'online',
    warning: 'warning',
    error: 'error',
    offline: '',
  };

  const sizeClasses = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-base',
  };

  const dotSize = {
    sm: 'w-2 h-2',
    md: 'w-2 h-2',
    lg: 'w-3 h-3',
  };

  const containerClasses = [
    'status-indicator',
    sizeClasses[size],
    className,
  ].filter(Boolean).join(' ');

  const dotClasses = [
    'status-dot',
    statusClasses[status],
    dotSize[size],
    !animated ? 'animate-none' : '',
  ].filter(Boolean).join(' ');

  return (
    <div className={containerClasses}>
      <div className={dotClasses} />
      <span>{label}</span>
    </div>
  );
};

export default StatusIndicator;