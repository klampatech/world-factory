import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for World Factory E2E tests.
 * Explicitly scoped to e2e/ directory to avoid picking up vitest test files in tests/
 */
export default defineConfig({
  testDir: './e2e',
  
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report' }],
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
