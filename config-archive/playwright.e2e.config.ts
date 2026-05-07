import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for World Factory E2E tests
 * Uses the already-running server on port 8765
 * 
 * Usage:
 *   npx playwright test --config=playwright.e2e.config.ts e2e/frontend-smoke-tests.spec.ts
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
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
