import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for World Factory E2E Smoke Tests (WOR-141)
 * Tests against static web server on port 8765
 * Backend API on port 8080
 */
export default defineConfig({
  testDir: '.',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 1,
  workers: 1,
  reporter: [
    ['list'],
    ['html', { open: 'never', outputFolder: '../playwright-report/WOR-141' }],
  ],
  
  use: {
    baseURL: 'http://localhost:8765',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
