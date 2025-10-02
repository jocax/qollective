// ABOUTME: Enterprise-themed dark mode toggle button for upper right corner
// ABOUTME: Animated toggle with sun/moon icons and LCARS styling

import React from 'react';
import { useDarkMode } from '../../contexts/DarkModeContext';

interface DarkModeToggleProps {
  className?: string;
}

export const DarkModeToggle: React.FC<DarkModeToggleProps> = ({ className = '' }) => {
  const { isDarkMode, toggleDarkMode } = useDarkMode();

  return (
    <button
      onClick={toggleDarkMode}
      className={`
        relative inline-flex items-center justify-center
        w-12 h-6 rounded-full border-2 transition-all duration-300 ease-in-out
        ${isDarkMode 
          ? 'bg-enterprise-blue-800 border-enterprise-blue-600 shadow-enterprise-glow' 
          : 'bg-enterprise-orange-500 border-enterprise-orange-400 shadow-enterprise'
        }
        hover:scale-105 active:scale-95 focus:outline-none focus:ring-2 focus:ring-enterprise-orange-400
        ${className}
      `}
      title={isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
      aria-label={isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
    >
      {/* Toggle slider */}
      <div
        className={`
          absolute w-4 h-4 rounded-full transition-all duration-300 ease-in-out
          flex items-center justify-center text-xs shadow-lg
          ${isDarkMode
            ? 'translate-x-3 bg-enterprise-blue-100 text-enterprise-blue-800'
            : '-translate-x-3 bg-enterprise-orange-100 text-enterprise-orange-800'
          }
        `}
      >
        {isDarkMode ? (
          // Moon icon for dark mode
          <svg 
            width="10" 
            height="10" 
            viewBox="0 0 24 24" 
            fill="currentColor"
            className="animate-pulse-slow"
          >
            <path d="M20.958 15.325c-2.551 4.414-8.186 5.933-12.6 3.382S2.425 10.521 4.976 6.107c.899-1.556 2.178-2.836 3.734-3.734-3.077 2.065-4.81 5.799-4.351 9.683.459 3.884 3.072 7.228 6.717 8.596 3.645 1.368 7.695.648 10.4-1.853 1.18-.885 2.151-2.025 2.832-3.322-.456.059-.913.06-1.35.048z"/>
          </svg>
        ) : (
          // Sun icon for light mode
          <svg 
            width="10" 
            height="10" 
            viewBox="0 0 24 24" 
            fill="currentColor"
            className="animate-enterprise-glow"
          >
            <path d="M12 2.25a.75.75 0 01.75.75v2.25a.75.75 0 01-1.5 0V3a.75.75 0 01.75-.75zM7.5 12a4.5 4.5 0 119 0 4.5 4.5 0 01-9 0zM18.894 6.166a.75.75 0 00-1.06-1.06l-1.591 1.59a.75.75 0 101.06 1.061l1.591-1.59zM21.75 12a.75.75 0 01-.75.75h-2.25a.75.75 0 010-1.5H21a.75.75 0 01.75.75zM17.834 18.894a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 10-1.061 1.06l1.59 1.591zM12 18a.75.75 0 01.75.75V21a.75.75 0 01-1.5 0v-2.25A.75.75 0 0112 18zM7.758 17.303a.75.75 0 00-1.061-1.06l-1.591 1.59a.75.75 0 001.06 1.061l1.591-1.59zM6 12a.75.75 0 01-.75.75H3a.75.75 0 010-1.5h2.25A.75.75 0 016 12zM6.697 7.757a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 00-1.061 1.06l1.59 1.591z"/>
          </svg>
        )}
      </div>

      {/* Background gradient effect */}
      <div 
        className={`
          absolute inset-0 rounded-full opacity-30 transition-opacity duration-300
          ${isDarkMode 
            ? 'bg-gradient-to-r from-enterprise-blue-700 to-enterprise-blue-900' 
            : 'bg-gradient-to-r from-enterprise-orange-400 to-enterprise-orange-600'
          }
        `}
      />
    </button>
  );
};

export default DarkModeToggle;