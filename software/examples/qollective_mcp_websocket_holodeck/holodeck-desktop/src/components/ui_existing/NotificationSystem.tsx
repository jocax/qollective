// ABOUTME: Toast notification system with Enterprise LCARS styling
// ABOUTME: Manages queued notifications with auto-dismiss, sound alerts, and persistent notifications

import React, { useState, useEffect, useCallback, createContext, useContext, ReactNode } from 'react';
import { Button, StatusIndicator } from './';

interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  duration?: number; // milliseconds, 0 for persistent
  persistent?: boolean;
  sound?: boolean;
  actions?: Array<{
    label: string;
    action: () => void;
    variant?: 'primary' | 'secondary';
  }>;
  metadata?: Record<string, any>;
  timestamp: Date;
}

interface NotificationContextType {
  notifications: Notification[];
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp'>) => string;
  removeNotification: (id: string) => void;
  clearAll: () => void;
  clearByType: (type: Notification['type']) => void;
}

const NotificationContext = createContext<NotificationContextType | null>(null);

export const useNotifications = () => {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error('useNotifications must be used within a NotificationProvider');
  }
  return context;
};

interface NotificationProviderProps {
  children: ReactNode;
  maxNotifications?: number;
  defaultDuration?: number;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left';
  soundEnabled?: boolean;
  className?: string;
}

export const NotificationProvider: React.FC<NotificationProviderProps> = ({
  children,
  maxNotifications = 5,
  defaultDuration = 5000,
  position = 'top-right',
  soundEnabled = true,
  className,
}) => {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  // Generate unique ID for notifications
  const generateId = useCallback(() => {
    return `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }, []);

  // Play notification sound
  const playNotificationSound = useCallback((type: Notification['type']) => {
    if (!soundEnabled) return;
    
    try {
      // Create different tones for different notification types
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();
      
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);
      
      // Different frequencies for different types
      const frequencies = {
        info: 440,     // A4
        success: 523,  // C5
        warning: 659,  // E5
        error: 330,    // E4 (lower, more serious)
      };
      
      oscillator.frequency.setValueAtTime(frequencies[type], audioContext.currentTime);
      oscillator.type = 'sine';
      
      gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.1);
      
      oscillator.start(audioContext.currentTime);
      oscillator.stop(audioContext.currentTime + 0.1);
    } catch (error) {
      console.error('Failed to play notification sound:', error);
    }
  }, [soundEnabled]);

  // Add notification
  const addNotification = useCallback((notificationData: Omit<Notification, 'id' | 'timestamp'>) => {
    const id = generateId();
    const notification: Notification = {
      ...notificationData,
      id,
      timestamp: new Date(),
      duration: notificationData.duration ?? defaultDuration,
    };

    setNotifications(prev => {
      const updated = [notification, ...prev];
      
      // Limit number of notifications
      if (updated.length > maxNotifications) {
        return updated.slice(0, maxNotifications);
      }
      
      return updated;
    });

    // Play sound notification
    if (notification.sound !== false) {
      playNotificationSound(notification.type);
    }

    // Auto-dismiss if not persistent
    if (!notification.persistent && notification.duration && notification.duration > 0) {
      setTimeout(() => {
        removeNotification(id);
      }, notification.duration);
    }

    return id;
  }, [generateId, defaultDuration, maxNotifications, playNotificationSound]);

  // Remove notification
  const removeNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(notification => notification.id !== id));
  }, []);

  // Clear all notifications
  const clearAll = useCallback(() => {
    setNotifications([]);
  }, []);

  // Clear by type
  const clearByType = useCallback((type: Notification['type']) => {
    setNotifications(prev => prev.filter(notification => notification.type !== type));
  }, []);

  const contextValue: NotificationContextType = {
    notifications,
    addNotification,
    removeNotification,
    clearAll,
    clearByType,
  };

  // Position classes
  const positionClasses = {
    'top-right': 'top-4 right-4',
    'top-left': 'top-4 left-4',
    'bottom-right': 'bottom-4 right-4',
    'bottom-left': 'bottom-4 left-4',
  };

  return (
    <NotificationContext.Provider value={contextValue}>
      {children}
      
      {/* Notification Container */}
      <div 
        className={`fixed ${positionClasses[position]} z-50 space-y-2 max-w-sm w-full pointer-events-none ${className || ''}`}
      >
        {notifications.map((notification) => (
          <NotificationToast
            key={notification.id}
            notification={notification}
            onClose={() => removeNotification(notification.id)}
          />
        ))}
      </div>
    </NotificationContext.Provider>
  );
};

interface NotificationToastProps {
  notification: Notification;
  onClose: () => void;
}

const NotificationToast: React.FC<NotificationToastProps> = ({
  notification,
  onClose,
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);

  // Animation timing
  useEffect(() => {
    // Slide in
    setTimeout(() => setIsVisible(true), 50);
  }, []);

  const handleClose = useCallback(() => {
    setIsRemoving(true);
    setTimeout(() => onClose(), 150); // Match animation duration
  }, [onClose]);

  const getStatusIndicatorType = (type: Notification['type']) => {
    switch (type) {
      case 'success': return 'enterprise-active';
      case 'warning': return 'enterprise-warning';
      case 'error': return 'enterprise-error';
      default: return 'enterprise-inactive';
    }
  };

  // Removed unused getTypeIcon function

  const getBorderColor = (type: Notification['type']) => {
    switch (type) {
      case 'success': return 'border-l-green-500';
      case 'warning': return 'border-l-yellow-500';
      case 'error': return 'border-l-red-500';
      default: return 'border-l-blue-500';
    }
  };

  return (
    <div
      className={`pointer-events-auto transform transition-all duration-150 ease-in-out ${
        isVisible && !isRemoving
          ? 'translate-x-0 opacity-100 scale-100'
          : 'translate-x-full opacity-0 scale-95'
      }`}
    >
      <div className={`bg-enterprise-blue-50 border-2 border-enterprise-blue-200 ${getBorderColor(notification.type)} border-l-4 rounded shadow-lg max-w-sm`}>
        {/* Header */}
        <div className="flex items-start justify-between p-3 pb-2">
          <div className="flex items-center space-x-2 flex-1">
            <StatusIndicator
              status={getStatusIndicatorType(notification.type)}
              size="sm"
            />
            <div className="flex-1 min-w-0">
              <h4 className="font-bold text-enterprise-blue-800 text-sm truncate">
                {notification.title}
              </h4>
              <div className="flex items-center space-x-2 text-xs text-enterprise-blue-600">
                <span className="font-mono uppercase">{notification.type}</span>
                <span>•</span>
                <span>{notification.timestamp.toLocaleTimeString()}</span>
              </div>
            </div>
          </div>
          
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClose}
            className="text-enterprise-blue-600 hover:text-enterprise-blue-800 p-1 min-w-0"
          >
            ×
          </Button>
        </div>

        {/* Message */}
        <div className="px-3 pb-2">
          <p className="text-sm text-enterprise-blue-800 leading-relaxed">
            {notification.message}
          </p>
        </div>

        {/* Actions */}
        {notification.actions && notification.actions.length > 0 && (
          <div className="px-3 pb-3">
            <div className="flex space-x-2">
              {notification.actions.map((action, index) => (
                <Button
                  key={index}
                  variant={action.variant === 'primary' ? 'lcars' : 'ghost'}
                  size="sm"
                  onClick={() => {
                    action.action();
                    handleClose();
                  }}
                  className="text-xs"
                >
                  {action.label}
                </Button>
              ))}
            </div>
          </div>
        )}

        {/* Progress Bar for Timed Notifications */}
        {!notification.persistent && notification.duration && notification.duration > 0 && (
          <div className="relative">
            <div className="h-1 bg-enterprise-blue-200 rounded-b">
              <div
                className="h-full bg-enterprise-orange rounded-b transition-all linear"
                style={{
                  animation: `shrink ${notification.duration}ms linear`,
                  width: '100%',
                }}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

// Convenience functions for common notification types
export const useNotificationHelpers = () => {
  const { addNotification } = useNotifications();
  
  return {
    showSuccess: (title: string, message: string, options?: Partial<Omit<Notification, 'id' | 'timestamp' | 'type' | 'title' | 'message'>>) =>
      addNotification({ type: 'success', title, message, ...options }),
      
    showError: (title: string, message: string, options?: Partial<Omit<Notification, 'id' | 'timestamp' | 'type' | 'title' | 'message'>>) =>
      addNotification({ type: 'error', title, message, persistent: true, ...options }),
      
    showWarning: (title: string, message: string, options?: Partial<Omit<Notification, 'id' | 'timestamp' | 'type' | 'title' | 'message'>>) =>
      addNotification({ type: 'warning', title, message, ...options }),
      
    showInfo: (title: string, message: string, options?: Partial<Omit<Notification, 'id' | 'timestamp' | 'type' | 'title' | 'message'>>) =>
      addNotification({ type: 'info', title, message, ...options }),
  };
};

// Add keyframes for progress bar animation
const style = document.createElement('style');
style.textContent = `
  @keyframes shrink {
    from { width: 100%; }
    to { width: 0%; }
  }
`;
document.head.appendChild(style);

export default NotificationProvider;