import { defineConfig, devices } from '@playwright/test';

/**
 * Smoke Test Configuration for WOR-89
 * Complete E2E test of frontend + backend
 * 
 * Frontend runs on port 8765, Backend on port 8080
 */

export default defineConfig({
  testDir: './e2e',
  testMatch: 'frontend-smoke-tests.spec.ts',
  fullyParallel: false,
  retries: 0,
  workers: 1,
  reporter: [
    ['list'],
    ['html', { outputFolder: 'test-results/WOR-89-smoke-test-report' }],
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