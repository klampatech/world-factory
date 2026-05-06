import { defineConfig, devices } from '@playwright/test';

/**
 * Smoke Test Configuration for WOR-89
 * Tests the complete e2e app with frontend and backend
 * 
 * Frontend is already running on port 8765, so no webServer needed.
 */

export default defineConfig({
  testDir: './e2e',
  testMatch: '**/frontend-smoke-tests.spec.ts',
  
  timeout: 30000,
  retries: 0,
  workers: 1,
  
  reporter: [
    ['list'],
    ['html', { outputFolder: 'test-results/smoke-test-report' }],
  ],
  
  use: {
    baseURL: 'http://localhost:8765',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
