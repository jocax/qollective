// ABOUTME: React Error Boundary component with Enterprise LCARS styling for graceful error handling
// ABOUTME: Catches JavaScript errors anywhere in the child component tree and displays fallback UI

import { Component, ReactNode, ErrorInfo } from 'react';
import { Panel, Button, StatusIndicator } from './';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
  errorId?: string;
  retryCount: number;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  enableRetry?: boolean;
  maxRetries?: number;
  showDetails?: boolean;
  className?: string;
}

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {

  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      retryCount: 0,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    // Update state to show the fallback UI
    return {
      hasError: true,
      error,
      errorId: `error-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    
    this.setState({
      error,
      errorInfo,
    });

    // Call custom error handler
    this.props.onError?.(error, errorInfo);

    // Send error to monitoring service (if available)
    this.reportError(error, errorInfo);
  }

  private reportError = async (error: Error, errorInfo: ErrorInfo) => {
    try {
      // This would typically send to a logging service
      const errorReport = {
        message: error.message,
        stack: error.stack,
        componentStack: errorInfo.componentStack,
        errorId: this.state.errorId,
        timestamp: new Date().toISOString(),
        userAgent: navigator.userAgent,
        url: window.location.href,
      };

      console.error('Error Report:', errorReport);
      
      // In a real application, you might send this to a service like Sentry
      // await sendErrorReport(errorReport);
    } catch (reportingError) {
      console.error('Failed to report error:', reportingError);
    }
  };

  private handleRetry = () => {
    const { maxRetries = 3 } = this.props;
    const { retryCount } = this.state;

    if (retryCount >= maxRetries) {
      return;
    }

    this.setState({
      hasError: false,
      error: undefined,
      errorInfo: undefined,
      retryCount: retryCount + 1,
    });
  };

  private handleReload = () => {
    window.location.reload();
  };

  private handleCopyError = () => {
    const { error, errorInfo, errorId } = this.state;
    const errorText = [
      `Error ID: ${errorId}`,
      `Message: ${error?.message}`,
      `Stack: ${error?.stack}`,
      `Component Stack: ${errorInfo?.componentStack}`,
      `Timestamp: ${new Date().toISOString()}`,
    ].join('\n\n');

    navigator.clipboard.writeText(errorText).then(() => {
      // Show success feedback (you might want to add a toast notification here)
      console.log('Error details copied to clipboard');
    }).catch(() => {
      console.error('Failed to copy error details');
    });
  };

  render() {
    const { hasError, error, errorInfo, errorId, retryCount } = this.state;
    const { 
      children, 
      fallback, 
      enableRetry = true, 
      maxRetries = 3, 
      showDetails = true, 
      className 
    } = this.props;

    if (hasError) {
      if (fallback) {
        return fallback;
      }

      return (
        <div className={`min-h-96 flex items-center justify-center p-6 ${className || ''}`}>
          <Panel
            variant="enterprise"
            size="sm"
            className="max-w-4xl w-full"
          >
            <div className="space-y-6">
              {/* Error Header */}
              <div className="flex items-center space-x-3">
                <StatusIndicator
                  status="enterprise-error"
                  size="default"
                  animated={true}
                />
                <div>
                  <h2 className="text-2xl font-bold text-enterprise-red-800 mb-1">
                    SYSTEM ERROR DETECTED
                  </h2>
                  <div className="text-sm text-enterprise-red-600 font-mono">
                    ERROR ID: {errorId}
                  </div>
                </div>
              </div>

              {/* Error Message */}
              <div className="p-4 bg-red-50 border-l-4 border-red-400 rounded">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <div className="text-red-400 text-xl">⚠️</div>
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-bold text-red-800">
                      Application Error
                    </h3>
                    <div className="mt-2 text-sm text-red-700">
                      <p>
                        An unexpected error occurred while rendering this component. 
                        The application has been safely contained to prevent further issues.
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              {/* Error Details (if enabled) */}
              {showDetails && error && (
                <div className="space-y-3">
                  <h4 className="text-enterprise-blue-700 font-bold uppercase tracking-wider text-sm">
                    Technical Details
                  </h4>
                  
                  <div className="p-3 bg-enterprise-blue-50 border border-enterprise-blue-200 rounded font-mono text-sm">
                    <div className="mb-2">
                      <span className="font-bold text-enterprise-blue-700">Message:</span>
                      <div className="text-enterprise-red-700 mt-1">{error.message}</div>
                    </div>
                    
                    {error.stack && (
                      <div className="mb-2">
                        <span className="font-bold text-enterprise-blue-700">Stack Trace:</span>
                        <pre className="text-xs text-enterprise-blue-800 mt-1 overflow-x-auto whitespace-pre-wrap">
                          {error.stack}
                        </pre>
                      </div>
                    )}
                    
                    {errorInfo?.componentStack && (
                      <div>
                        <span className="font-bold text-enterprise-blue-700">Component Stack:</span>
                        <pre className="text-xs text-enterprise-blue-800 mt-1 overflow-x-auto whitespace-pre-wrap">
                          {errorInfo.componentStack}
                        </pre>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Recovery Actions */}
              <div className="space-y-4">
                <h4 className="text-enterprise-blue-700 font-bold uppercase tracking-wider text-sm">
                  Recovery Options
                </h4>
                
                <div className="flex flex-wrap gap-3">
                  {enableRetry && retryCount < maxRetries && (
                    <Button
                      variant="lcars"
                      onClick={this.handleRetry}
                      className="flex items-center space-x-2"
                    >
                      <span>🔄</span>
                      <span>RETRY COMPONENT ({maxRetries - retryCount} attempts left)</span>
                    </Button>
                  )}
                  
                  <Button
                    variant="command"
                    onClick={this.handleReload}
                    className="flex items-center space-x-2"
                  >
                    <span>🔄</span>
                    <span>RELOAD APPLICATION</span>
                  </Button>
                  
                  <Button
                    variant="ghost"
                    onClick={this.handleCopyError}
                    className="flex items-center space-x-2 text-enterprise-blue-600"
                  >
                    <span>📋</span>
                    <span>COPY ERROR DETAILS</span>
                  </Button>
                  
                  <Button
                    variant="ghost"
                    onClick={() => window.history.back()}
                    className="flex items-center space-x-2 text-enterprise-blue-600"
                  >
                    <span>⬅️</span>
                    <span>GO BACK</span>
                  </Button>
                </div>

                {retryCount >= maxRetries && (
                  <div className="p-3 bg-enterprise-orange-50 border border-enterprise-orange-200 rounded">
                    <div className="flex items-center space-x-2">
                      <StatusIndicator status="enterprise-warning" size="sm" />
                      <span className="text-enterprise-orange-700 text-sm font-bold">
                        Maximum retry attempts reached. Please reload the application or contact support.
                      </span>
                    </div>
                  </div>
                )}
              </div>

              {/* Support Information */}
              <div className="pt-4 border-t border-enterprise-blue-200">
                <div className="text-sm text-enterprise-blue-600">
                  <p className="mb-2">
                    <span className="font-bold">Need help?</span> If this error persists, please contact the system administrator.
                  </p>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
                    <div>
                      <span className="font-bold">Error ID:</span> {errorId}
                    </div>
                    <div>
                      <span className="font-bold">Timestamp:</span> {new Date().toLocaleString()}
                    </div>
                    <div>
                      <span className="font-bold">Component:</span> {error?.stack?.split('\n')[1]?.trim() || 'Unknown'}
                    </div>
                    <div>
                      <span className="font-bold">Retry Count:</span> {retryCount}/{maxRetries}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Panel>
        </div>
      );
    }

    return children;
  }
}

export default ErrorBoundary;