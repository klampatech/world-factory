import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for World Factory Frontend Smoke Tests (WOR-130)
 * Tests against static web server on port 8765
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/frontend-smoke-tests.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 1,
  workers: 1,
  reporter: [
    ['list'],
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