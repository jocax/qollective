// ABOUTME: Enterprise LCARS styled status indicator for agent and system monitoring
// ABOUTME: Provides real-time status visualization with holodeck command interface styling

import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const statusVariants = cva(
  'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium uppercase tracking-wide',
  {
    variants: {
      status: {
        active: 'bg-green-100 text-green-800 border border-green-300',
        inactive: 'bg-gray-100 text-gray-800 border border-gray-300',
        connecting: 'bg-yellow-100 text-yellow-800 border border-yellow-300',
        error: 'bg-red-100 text-red-800 border border-red-300',
        warning: 'bg-orange-100 text-orange-800 border border-orange-300',
        // Enterprise LCARS variants
        'enterprise-active': 'bg-enterprise-green text-white border border-enterprise-green-light shadow-lg',
        'enterprise-inactive': 'bg-enterprise-gray text-white border border-enterprise-gray-light',
        'enterprise-connecting': 'bg-enterprise-orange text-enterprise-black border border-enterprise-orange-light animate-pulse',
        'enterprise-error': 'bg-enterprise-red text-white border border-enterprise-red-light',
        'enterprise-warning': 'bg-enterprise-yellow text-enterprise-black border border-enterprise-yellow-light',
      },
      size: {
        sm: 'px-2 py-0.5 text-xs',
        default: 'px-2.5 py-0.5 text-xs',
        lg: 'px-3 py-1 text-sm',
      },
    },
    defaultVariants: {
      status: 'enterprise-active',
      size: 'default',
    },
  }
);

interface StatusIndicatorProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof statusVariants> {
  label?: string;
  showDot?: boolean;
  animated?: boolean;
}

const StatusIndicator = React.forwardRef<HTMLSpanElement, StatusIndicatorProps>(
  ({ className, status, size, label, showDot = true, animated = false, ...props }, ref) => {
    const isConnecting = status === 'connecting' || status === 'enterprise-connecting';
    const shouldAnimate = animated || isConnecting;

    return (
      <span
        ref={ref}
        className={cn(
          statusVariants({ status, size }),
          shouldAnimate && 'animate-pulse',
          className
        )}
        {...props}
      >
        {showDot && (
          <span
            className={cn(
              'w-2 h-2 rounded-full mr-2',
              {
                'bg-green-500': status === 'active' || status === 'enterprise-active',
                'bg-gray-400': status === 'inactive' || status === 'enterprise-inactive',
                'bg-yellow-500': status === 'connecting' || status === 'enterprise-connecting',
                'bg-red-500': status === 'error' || status === 'enterprise-error',
                'bg-orange-500': status === 'warning' || status === 'enterprise-warning',
              }
            )}
          />
        )}
        {label || status?.replace('enterprise-', '').replace('-', ' ').toUpperCase()}
      </span>
    );
  }
);

StatusIndicator.displayName = 'StatusIndicator';

// Health indicator for agents with numeric health values
interface HealthIndicatorProps extends React.HTMLAttributes<HTMLDivElement> {
  health: number;
  label?: string;
  showPercentage?: boolean;
  variant?: 'default' | 'enterprise';
}

const HealthIndicator = React.forwardRef<HTMLDivElement, HealthIndicatorProps>(
  ({ className, health, label, showPercentage = true, variant = 'enterprise', ...props }, ref) => {
    const getHealthStatus = (health: number) => {
      if (health >= 90) return variant === 'enterprise' ? 'enterprise-active' : 'active';
      if (health >= 70) return variant === 'enterprise' ? 'enterprise-warning' : 'warning';
      if (health >= 40) return variant === 'enterprise' ? 'enterprise-error' : 'error';
      return variant === 'enterprise' ? 'enterprise-inactive' : 'inactive';
    };

    const getHealthColor = (health: number) => {
      if (health >= 90) return 'text-green-600';
      if (health >= 70) return 'text-yellow-600';
      if (health >= 40) return 'text-orange-600';
      return 'text-red-600';
    };

    return (
      <div ref={ref} className={cn('flex items-center space-x-2', className)} {...props}>
        <StatusIndicator status={getHealthStatus(health)} showDot={true} size="sm" />
        <div className="flex items-center space-x-1">
          {label && (
            <span className={cn(
              'text-sm font-medium',
              variant === 'enterprise' ? 'text-enterprise-blue' : 'text-gray-700'
            )}>
              {label}:
            </span>
          )}
          <span className={cn(
            'text-sm font-bold',
            variant === 'enterprise' ? getHealthColor(health) : getHealthColor(health)
          )}>
            {showPercentage ? `${health.toFixed(1)}%` : health.toFixed(1)}
          </span>
        </div>
      </div>
    );
  }
);

HealthIndicator.displayName = 'HealthIndicator';

export { StatusIndicator, HealthIndicator, statusVariants };