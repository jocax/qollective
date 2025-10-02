// ABOUTME: Enterprise LCARS styled progress component using Radix UI Progress
// ABOUTME: Provides progress bars with holodeck command interface styling

import React from 'react';
import * as ProgressPrimitive from '@radix-ui/react-progress';
import { cn } from '../../lib/utils';

const Progress = React.forwardRef<
  React.ElementRef<typeof ProgressPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof ProgressPrimitive.Root> & {
    variant?: 'default' | 'enterprise' | 'status';
    indicatorClassName?: string;
  }
>(({ className, value, variant = 'default', indicatorClassName, ...props }, ref) => (
  <ProgressPrimitive.Root
    ref={ref}
    className={cn(
      'relative h-4 w-full overflow-hidden rounded-full',
      {
        'bg-secondary': variant === 'default',
        'bg-enterprise-blue-900 border border-enterprise-blue-300': variant === 'enterprise',
        'bg-gray-800 border border-gray-600': variant === 'status',
      },
      className
    )}
    {...props}
  >
    <ProgressPrimitive.Indicator
      className={cn(
        'h-full w-full flex-1 transition-all',
        {
          'bg-primary': variant === 'default' && !indicatorClassName,
          'bg-gradient-to-r from-enterprise-orange to-enterprise-orange-light': variant === 'enterprise' && !indicatorClassName,
          'bg-gradient-to-r from-green-500 to-green-400': variant === 'status' && (value || 0) >= 70 && !indicatorClassName,
          'bg-gradient-to-r from-yellow-500 to-yellow-400': variant === 'status' && (value || 0) >= 40 && (value || 0) < 70 && !indicatorClassName,
          'bg-gradient-to-r from-red-500 to-red-400': variant === 'status' && (value || 0) < 40 && !indicatorClassName,
        },
        indicatorClassName
      )}
      style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
    />
  </ProgressPrimitive.Root>
));
Progress.displayName = ProgressPrimitive.Root.displayName;

export { Progress };