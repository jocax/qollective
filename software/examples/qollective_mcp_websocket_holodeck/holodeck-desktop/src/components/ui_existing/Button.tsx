// ABOUTME: Enterprise LCARS styled button component using Radix UI primitives
// ABOUTME: Provides consistent button styling with holodeck command interface aesthetics

import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const buttonVariants = cva(
  'inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
  {
    variants: {
      variant: {
        default: 'bg-enterprise-orange text-enterprise-black hover:bg-enterprise-orange-dark',
        destructive: 'bg-enterprise-red text-white hover:bg-enterprise-red/90',
        outline: 'border border-enterprise-blue text-enterprise-blue hover:bg-enterprise-blue hover:text-white',
        secondary: 'bg-enterprise-blue text-white hover:bg-enterprise-blue-dark',
        ghost: 'hover:bg-enterprise-blue/10 text-enterprise-blue',
        link: 'text-enterprise-blue underline-offset-4 hover:underline',
        lcars: 'bg-gradient-to-r from-enterprise-orange to-enterprise-orange-dark text-enterprise-black font-bold uppercase tracking-wide rounded-none border-l-4 border-enterprise-orange-dark shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200',
        command: 'bg-enterprise-blue text-white font-mono uppercase tracking-widest border border-enterprise-blue-light hover:bg-enterprise-blue-light hover:border-enterprise-orange transition-all duration-150',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
        icon: 'h-10 w-10',
        lcars: 'h-12 px-6 py-3',
        command: 'h-8 px-4 text-xs',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);

Button.displayName = 'Button';

export { Button, buttonVariants };