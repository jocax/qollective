// ABOUTME: Health indicator component with progress bar and status colors
// ABOUTME: Displays health percentage with Enterprise LCARS styling and animated progress

import React from 'react';
import { cva } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const healthIndicatorVariants = cva(
  "flex flex-col space-y-2",
  {
    variants: {
      variant: {
        default: "",
        enterprise: "",
        compact: "space-y-1",
      },
      size: {
        sm: "text-sm",
        default: "text-base",
        lg: "text-lg",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

interface HealthIndicatorProps {
  health: number; // 0-100
  label?: string;
  variant?: 'default' | 'enterprise' | 'compact';
  size?: 'sm' | 'default' | 'lg';
  showPercentage?: boolean;
  animated?: boolean;
  className?: string;
}

export const HealthIndicator: React.FC<HealthIndicatorProps> = ({
  health,
  label = "Health",
  variant = "default",
  size = "default",
  showPercentage = true,
  animated = true,
  className,
}) => {
  const getHealthColor = (healthValue: number) => {
    if (healthValue >= 90) return {
      bar: 'bg-green-500',
      text: 'text-green-600',
      bg: 'bg-green-50',
      border: 'border-green-200'
    };
    if (healthValue >= 70) return {
      bar: 'bg-yellow-500', 
      text: 'text-yellow-600',
      bg: 'bg-yellow-50',
      border: 'border-yellow-200'
    };
    if (healthValue >= 40) return {
      bar: 'bg-orange-500',
      text: 'text-orange-600', 
      bg: 'bg-orange-50',
      border: 'border-orange-200'
    };
    return {
      bar: 'bg-red-500',
      text: 'text-red-600',
      bg: 'bg-red-50', 
      border: 'border-red-200'
    };
  };

  const getHealthStatus = (healthValue: number) => {
    if (healthValue >= 90) return "OPTIMAL";
    if (healthValue >= 70) return "GOOD";
    if (healthValue >= 40) return "DEGRADED";
    return "CRITICAL";
  };

  const colors = getHealthColor(health);
  const status = getHealthStatus(health);
  const clampedHealth = Math.max(0, Math.min(100, health));

  return (
    <div className={cn(healthIndicatorVariants({ variant, size }), className)}>
      {/* Header with label and status */}
      <div className="flex items-center justify-between">
        <span className="text-enterprise-blue-600 font-mono text-xs uppercase">
          {label}
        </span>
        <div className="flex items-center space-x-2">
          {showPercentage && (
            <span className={cn("font-bold", colors.text)}>
              {clampedHealth.toFixed(1)}%
            </span>
          )}
          <span className={cn("text-xs font-mono uppercase", colors.text)}>
            {status}
          </span>
        </div>
      </div>

      {/* Progress Bar */}
      <div className={cn(
        "relative w-full h-3 rounded border",
        variant === 'enterprise' ? 'bg-enterprise-blue-100 border-enterprise-blue-200' : 'bg-gray-200 border-gray-300'
      )}>
        <div
          className={cn(
            "h-full rounded transition-all duration-500 ease-out",
            colors.bar,
            animated && "animate-pulse"
          )}
          style={{ width: `${clampedHealth}%` }}
        />
        
        {/* Health status indicator dot */}
        <div
          className={cn(
            "absolute top-1/2 -translate-y-1/2 w-2 h-2 rounded-full border-2 border-white transition-all duration-300",
            colors.bar
          )}
          style={{ left: `${Math.max(2, clampedHealth - 2)}%` }}
        />
      </div>

      {/* Additional status indicators for enterprise variant */}
      {variant === 'enterprise' && (
        <div className="flex justify-between text-xs">
          <span className="text-enterprise-blue-500 font-mono">0%</span>
          <span className="text-enterprise-blue-500 font-mono">25%</span>
          <span className="text-enterprise-blue-500 font-mono">50%</span>
          <span className="text-enterprise-blue-500 font-mono">75%</span>
          <span className="text-enterprise-blue-500 font-mono">100%</span>
        </div>
      )}
    </div>
  );
};