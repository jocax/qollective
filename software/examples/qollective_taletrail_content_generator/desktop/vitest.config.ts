import { defineVitestConfig } from '@nuxt/test-utils/config'

export default defineVitestConfig({
  test: {
    environment: 'nuxt',
    globals: true,
    setupFiles: ['./vitest.setup.ts'],

    // Add environmentOptions for Vitest 4+ compatibility
    environmentOptions: {
      nuxt: {
        domEnvironment: 'happy-dom'
      }
    },

    // Optimize test execution with forks pool
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: false
      }
    },

    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'dist/',
        '.nuxt/',
        'src-tauri/',
        '**/*.spec.ts',
        '**/__tests__/**'
      ]
    }
  }
})
