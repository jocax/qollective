// ABOUTME: Enterprise LCARS styled avatar component using Radix UI Avatar
// ABOUTME: Provides user avatars with holodeck command interface styling

import React from 'react';
import * as AvatarPrimitive from '@radix-ui/react-avatar';
import { cn } from '../../lib/utils';

const Avatar = React.forwardRef<
  React.ElementRef<typeof AvatarPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof AvatarPrimitive.Root>
>(({ className, ...props }, ref) => (
  <AvatarPrimitive.Root
    ref={ref}
    className={cn(
      'relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full border-2 border-enterprise-blue-300',
      className
    )}
    {...props}
  />
));
Avatar.displayName = AvatarPrimitive.Root.displayName;

const AvatarImage = React.forwardRef<
  React.ElementRef<typeof AvatarPrimitive.Image>,
  React.ComponentPropsWithoutRef<typeof AvatarPrimitive.Image>
>(({ className, ...props }, ref) => (
  <AvatarPrimitive.Image
    ref={ref}
    className={cn('aspect-square h-full w-full', className)}
    {...props}
  />
));
AvatarImage.displayName = AvatarPrimitive.Image.displayName;

const AvatarFallback = React.forwardRef<
  React.ElementRef<typeof AvatarPrimitive.Fallback>,
  React.ComponentPropsWithoutRef<typeof AvatarPrimitive.Fallback>
>(({ className, ...props }, ref) => (
  <AvatarPrimitive.Fallback
    ref={ref}
    className={cn(
      'flex h-full w-full items-center justify-center rounded-full bg-enterprise-blue text-white font-bold text-sm uppercase',
      className
    )}
    {...props}
  />
));
AvatarFallback.displayName = AvatarPrimitive.Fallback.displayName;

// Comprehensive Avatar component with src and fallback props
interface AvatarProps extends React.ComponentPropsWithoutRef<typeof AvatarPrimitive.Root> {
  src?: string;
  fallback?: string;
  alt?: string;
}

const ComprehensiveAvatar = React.forwardRef<
  React.ElementRef<typeof AvatarPrimitive.Root>,
  AvatarProps
>(({ className, src, fallback, alt, ...props }, ref) => (
  <AvatarPrimitive.Root
    ref={ref}
    className={cn(
      'relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full border-2 border-enterprise-blue-300',
      className
    )}
    {...props}
  >
    {src && <AvatarImage src={src} alt={alt} />}
    {fallback && <AvatarFallback>{fallback}</AvatarFallback>}
  </AvatarPrimitive.Root>
));
ComprehensiveAvatar.displayName = 'ComprehensiveAvatar';

export { Avatar as BaseAvatar, AvatarImage, AvatarFallback };
export { ComprehensiveAvatar as Avatar };
export default ComprehensiveAvatar;