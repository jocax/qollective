// ABOUTME: Enterprise-themed button component with multiple variants and states
// ABOUTME: Supports primary, secondary, danger variants with loading and disabled states

import React from 'react';

export interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary' | 'danger' | 'info';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  className?: string;
  type?: 'button' | 'submit' | 'reset';
}

export const Button: React.FC<ButtonProps> = ({
  children,
  onClick,
  variant = 'primary',
  size = 'md',
  disabled = false,
  loading = false,
  className = '',
  type = 'button',
}) => {
  const baseClasses = 'enterprise-button';
  const variantClasses = {
    primary: '',
    secondary: 'secondary',
    danger: 'danger',
    info: 'info',
  };
  
  const sizeClasses = {
    sm: 'text-xs px-3 py-1',
    md: 'text-sm px-4 py-2',
    lg: 'text-base px-6 py-3',
  };

  const classes = [
    baseClasses,
    variantClasses[variant],
    sizeClasses[size],
    className,
  ].filter(Boolean).join(' ');

  return (
    <button
      type={type}
      className={classes}
      onClick={onClick}
      disabled={disabled || loading}
      style={{ cursor: disabled || loading ? 'not-allowed' : 'pointer' }}
    >
      {loading ? (
        <span className="flex items-center gap-2">
          <div className="enterprise-loading" />
          {children}
        </span>
      ) : (
        children
      )}
    </button>
  );
};

export default Button;