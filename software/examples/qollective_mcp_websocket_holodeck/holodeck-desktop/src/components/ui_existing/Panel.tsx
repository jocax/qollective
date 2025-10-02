// ABOUTME: Enterprise LCARS styled panel component for holodeck interface sections
// ABOUTME: Provides consistent panel layouts with Star Trek Enterprise command styling

import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const panelVariants = cva(
  'relative overflow-hidden',
  {
    variants: {
      variant: {
        default: 'bg-white border border-gray-200 rounded-lg shadow-sm',
        enterprise: 'bg-gradient-to-br from-enterprise-blue-50 to-enterprise-blue-100 border-2 border-enterprise-blue-200 shadow-enterprise',
        command: 'bg-enterprise-blue text-white border border-enterprise-blue-light',
        status: 'bg-gray-900 border border-gray-700 text-green-400',
        warning: 'bg-yellow-50 border border-yellow-200 text-yellow-800',
        danger: 'bg-red-50 border border-red-200 text-red-800',
      },
      size: {
        sm: 'p-3',
        default: 'p-4',
        lg: 'p-6',
        xl: 'p-8',
      },
      corner: {
        default: 'rounded-lg',
        none: 'rounded-none',
        enterprise: 'rounded-none', // LCARS panels are typically angular
      },
    },
    defaultVariants: {
      variant: 'enterprise',
      size: 'default',
      corner: 'enterprise',
    },
  }
);

interface PanelProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof panelVariants> {
  title?: string;
  subtitle?: string;
  headerAction?: React.ReactNode;
  showBorder?: boolean;
}

const Panel = React.forwardRef<HTMLDivElement, PanelProps>(
  ({ 
    className, 
    variant, 
    size, 
    corner, 
    title, 
    subtitle, 
    headerAction, 
    showBorder = true,
    children, 
    ...props 
  }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(panelVariants({ variant, size, corner, className }))}
        {...props}
      >
        {/* Enterprise LCARS accent border */}
        {showBorder && variant === 'enterprise' && (
          <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-enterprise-orange via-enterprise-orange-light to-enterprise-blue"></div>
        )}
        
        {/* Panel Header */}
        {(title || subtitle || headerAction) && (
          <div className={cn(
            'flex items-center justify-between mb-4 pb-3',
            variant === 'enterprise' && 'border-b border-enterprise-blue-200'
          )}>
            <div>
              {title && (
                <h3 className={cn(
                  'font-bold',
                  variant === 'enterprise' && 'text-enterprise-blue uppercase tracking-wide text-lg',
                  variant === 'command' && 'text-white font-mono uppercase tracking-widest',
                  variant === 'status' && 'text-green-400 font-mono uppercase'
                )}>
                  {title}
                </h3>
              )}
              {subtitle && (
                <p className={cn(
                  'text-sm mt-1',
                  variant === 'enterprise' && 'text-enterprise-blue-600',
                  variant === 'command' && 'text-enterprise-blue-200',
                  variant === 'status' && 'text-green-300'
                )}>
                  {subtitle}
                </p>
              )}
            </div>
            {headerAction && (
              <div className="flex items-center space-x-2">
                {headerAction}
              </div>
            )}
          </div>
        )}
        
        {/* Panel Content */}
        <div className={cn(
          variant === 'enterprise' && 'text-enterprise-blue-700',
          variant === 'command' && 'text-white',
          variant === 'status' && 'text-green-300'
        )}>
          {children}
        </div>
        
        {/* Enterprise LCARS corner accent */}
        {variant === 'enterprise' && (
          <>
            <div className="absolute top-2 left-2 w-3 h-3 border-l-2 border-t-2 border-enterprise-orange"></div>
            <div className="absolute bottom-2 right-2 w-3 h-3 border-r-2 border-b-2 border-enterprise-blue"></div>
          </>
        )}
      </div>
    );
  }
);

Panel.displayName = 'Panel';

export { Panel, panelVariants };