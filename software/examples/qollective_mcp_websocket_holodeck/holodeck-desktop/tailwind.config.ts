// ABOUTME: Tailwind CSS configuration with Radix UI integration and Enterprise LCARS theme
// ABOUTME: Defines custom colors, animations, and design system for holodeck desktop app

import type { Config } from 'tailwindcss'

const config: Config = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "class",
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      // Enterprise LCARS Color Scheme
      colors: {
        // Primary Enterprise colors
        enterprise: {
          // LCARS Blue family
          blue: {
            50: '#e6f3ff',
            100: '#b3d9ff',
            200: '#80bfff',
            300: '#4da6ff',
            400: '#1a8cff',
            500: '#0073e6', // Primary LCARS blue
            600: '#0066cc',
            700: '#0059b3',
            800: '#004d99',
            900: '#004080',
          },
          // LCARS Orange/Amber family
          orange: {
            50: '#fff7ed',
            100: '#ffedd5',
            200: '#fed7aa',
            300: '#fdba74',
            400: '#fb923c',
            500: '#ff9500', // Primary LCARS orange
            600: '#ea580c',
            700: '#c2410c',
            800: '#9a3412',
            900: '#7c2d12',
            dark: '#cc7a00',
            light: '#ffab33',
          },
          amber: {
            50: '#fff8e6',
            100: '#ffecb3',
            200: '#ffe080',
            300: '#ffd54d',
            400: '#ffca1a',
            500: '#ffbf00', // Primary LCARS amber
            600: '#e6ac00',
            700: '#cc9900',
            800: '#b38600',
            900: '#997300',
          },
          // LCARS Green family (success/active)
          green: {
            50: '#f0fdf4',
            100: '#dcfce7',
            200: '#bbf7d0',
            300: '#86efac',
            400: '#4ade80',
            500: '#16a34a', // Primary LCARS green
            600: '#16a34a',
            700: '#15803d',
            800: '#166534',
            900: '#14532d',
            light: '#22c55e',
          },
          // LCARS Yellow family (warning/caution)
          yellow: {
            50: '#fefce8',
            100: '#fef3c7',
            200: '#fde68a',
            300: '#fcd34d',
            400: '#fbbf24',
            500: '#eab308', // Primary LCARS yellow
            600: '#d97706',
            700: '#b45309',
            800: '#92400e',
            900: '#78350f',
            light: '#fcd34d',
          },
          // LCARS Red family (alerts/warnings)
          red: {
            50: '#ffe6e6',
            100: '#ffb3b3',
            200: '#ff8080',
            300: '#ff4d4d',
            400: '#ff1a1a',
            500: '#e60000', // Primary LCARS red
            600: '#cc0000',
            700: '#b30000',
            800: '#990000',
            900: '#800000',
          },
          // LCARS Gray family
          gray: {
            50: '#f8f9fa',
            100: '#e9ecef',
            200: '#dee2e6',
            300: '#ced4da',
            400: '#adb5bd',
            500: '#6c757d', // Primary LCARS gray
            600: '#495057',
            700: '#343a40',
            800: '#212529',
            900: '#000000',
          },
        },
        // Semantic color mapping
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        xl: "calc(var(--radius) + 4px)",
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      fontFamily: {
        // LCARS-style monospace font
        mono: ["SF Mono", "Monaco", "Inconsolata", "Fira Code", "monospace"],
        // Clean sans-serif for readability
        sans: ["Inter", "SF Pro Display", "-apple-system", "BlinkMacSystemFont", "sans-serif"],
      },
      // Enterprise-style animations
      keyframes: {
        "fade-in": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        "fade-in-up": {
          "0%": {
            opacity: "0",
            transform: "translateY(10px)",
          },
          "100%": {
            opacity: "1",
            transform: "translateY(0)",
          },
        },
        "fade-in-scale": {
          "0%": {
            opacity: "0",
            transform: "scale(0.95)",
          },
          "100%": {
            opacity: "1",
            transform: "scale(1)",
          },
        },
        "slide-in-right": {
          "0%": {
            transform: "translateX(100%)",
          },
          "100%": {
            transform: "translateX(0)",
          },
        },
        "slide-in-left": {
          "0%": {
            transform: "translateX(-100%)",
          },
          "100%": {
            transform: "translateX(0)",
          },
        },
        "pulse-status": {
          "0%, 100%": {
            opacity: "1",
          },
          "50%": {
            opacity: "0.6",
          },
        },
        "spin-smooth": {
          "from": {
            transform: "rotate(0deg)",
          },
          "to": {
            transform: "rotate(360deg)",
          },
        },
        // LCARS panel entrance animations
        "panel-enter": {
          "0%": {
            opacity: "0",
            transform: "translateY(-20px) scale(0.95)",
          },
          "100%": {
            opacity: "1",
            transform: "translateY(0) scale(1)",
          },
        },
        // Enterprise-specific animations
        "enterprise-glow": {
          "0%, 100%": { 
            boxShadow: "0 0 5px rgba(255, 149, 0, 0.5)" 
          },
          "50%": { 
            boxShadow: "0 0 20px rgba(255, 149, 0, 0.8), 0 0 30px rgba(255, 149, 0, 0.6)" 
          },
        },
        "enterprise-scan": {
          "0%": { 
            transform: "translateX(-100%)", 
            opacity: "0" 
          },
          "50%": { opacity: "1" },
          "100%": { 
            transform: "translateX(100%)", 
            opacity: "0" 
          },
        },
        "enterprise-border-flow": {
          "0%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
          "100%": { backgroundPosition: "0% 50%" },
        },
        "lcars-alert-flash": {
          "0%, 100%": { 
            backgroundColor: "rgb(254, 242, 242)", 
            borderColor: "rgb(252, 165, 165)" 
          },
          "50%": { 
            backgroundColor: "rgb(254, 226, 226)", 
            borderColor: "rgb(248, 113, 113)" 
          },
        },
      },
      animation: {
        "fade-in": "fade-in 0.5s ease-out",
        "fade-in-up": "fade-in-up 0.5s ease-out",
        "fade-in-scale": "fade-in-scale 0.3s ease-out",
        "slide-in-right": "slide-in-right 0.3s ease-out",
        "slide-in-left": "slide-in-left 0.3s ease-out",
        "pulse-status": "pulse-status 2s ease-in-out infinite",
        "spin-smooth": "spin-smooth 2s linear infinite",
        "panel-enter": "panel-enter 0.6s ease-out",
        // Enterprise-specific animations
        "enterprise-glow": "enterprise-glow 2s ease-in-out infinite alternate",
        "enterprise-scan": "enterprise-scan 2s linear infinite",
        "enterprise-border-flow": "enterprise-border-flow 3s ease-in-out infinite",
        "lcars-alert": "lcars-alert-flash 1s ease-in-out infinite",
      },
      // LCARS-style spacing and sizing
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
        '128': '32rem',
      },
      height: {
        'screen-safe': 'calc(100vh - 2rem)',
      },
      minHeight: {
        'screen-safe': 'calc(100vh - 2rem)',
      },
      // Enterprise shadow system
      boxShadow: {
        'enterprise': '0 4px 12px rgba(30, 64, 175, 0.15), 0 2px 4px rgba(30, 64, 175, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.1)',
        'enterprise-lg': '0 8px 25px rgba(30, 64, 175, 0.2), 0 4px 10px rgba(30, 64, 175, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.1)',
        'enterprise-glow': '0 0 20px rgba(255, 149, 0, 0.3), 0 0 40px rgba(255, 149, 0, 0.1), 0 4px 12px rgba(30, 64, 175, 0.15)',
        'lcars-panel': '0 2px 8px rgba(0, 0, 0, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.1)',
        'lcars-inset': 'inset 0 2px 4px rgba(0, 0, 0, 0.1)',
      },
      // LCARS backdrop blur effects
      backdropBlur: {
        'lcars': '8px',
      },
    },
  },
  plugins: [],
} satisfies Config

export default config