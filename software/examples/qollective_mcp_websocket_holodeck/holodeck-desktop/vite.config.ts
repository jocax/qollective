// ABOUTME: Vite configuration for React frontend with Tauri integration
// ABOUTME: Serves frontend on port 1420 and builds to dist directory for Tauri

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 1481,
    host: 'localhost',
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
});