// ABOUTME: Tauri window setup utilities to ensure proper window visibility and behavior
// ABOUTME: Handles window show/hide, focus, and desktop integration for holodeck app

export class TauriSetup {
  /**
   * Initialize Tauri window setup
   * Ensures the main window is visible and properly configured
   */
  static async initialize(): Promise<void> {
    try {
      if (!this.isTauri()) {
        console.log('Not running in Tauri context, skipping window setup');
        return;
      }

      // Import Tauri APIs dynamically to avoid issues in web context
      const { getCurrentWindow } = await import('@tauri-apps/api/webview');
      const window = getCurrentWindow();
      
      // Ensure window is visible
      await window.show();
      
      // Focus the window
      await window.setFocus();
      
      // Set minimum size if not already set
      await window.setMinSize({ width: 800, height: 600 });
      
      console.log('✅ Tauri window initialized successfully');
    } catch (error) {
      console.error('❌ Failed to initialize Tauri window:', error);
    }
  }

  /**
   * Show the main window (useful for splash screen transitions)
   */
  static async showWindow(): Promise<void> {
    try {
      if (!this.isTauri()) return;
      
      const { getCurrentWindow } = await import('@tauri-apps/api/webview');
      const window = getCurrentWindow();
      
      await window.show();
      await window.setFocus();
    } catch (error) {
      console.error('❌ Failed to show window:', error);
    }
  }

  /**
   * Check if running in Tauri context
   */
  static isTauri(): boolean {
    return typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;
  }

  /**
   * Get window label
   */
  static async getWindowLabel(): Promise<string> {
    try {
      if (!this.isTauri()) return 'web';
      
      const { getCurrentWindow } = await import('@tauri-apps/api/webview');
      const window = getCurrentWindow();
      
      return await window.label();
    } catch (error) {
      console.error('❌ Failed to get window label:', error);
      return 'main';
    }
  }
}

export default TauriSetup;