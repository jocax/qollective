// ABOUTME: Enterprise-themed panel component with color variants and title support
// ABOUTME: Provides consistent styling for content sections with optional headers

import React from 'react';

export interface PanelProps {
  children: React.ReactNode;
  title?: string;
  variant?: 'default' | 'blue' | 'red' | 'teal';
  className?: string;
}

export const Panel: React.FC<PanelProps> = ({
  children,
  title,
  variant = 'default',
  className = '',
}) => {
  const variantClasses = {
    default: 'enterprise-panel',
    blue: 'enterprise-panel panel-blue',
    red: 'enterprise-panel panel-red',
    teal: 'enterprise-panel panel-teal',
  };

  const panelClasses = [
    variantClasses[variant],
    className,
  ].filter(Boolean).join(' ');

  return (
    <div className={panelClasses}>
      {title && (
        <h3 className="enterprise-section-title">
          {title}
        </h3>
      )}
      {children}
    </div>
  );
};

export default Panel;