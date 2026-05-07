import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for World Factory E2E tests
 * 
 * Usage:
 *   npx playwright test           # Run all tests
 *   npx playwright test --ui     # Run with UI
 *   npx playwright test --headed # Run in headed mode
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html'],
    ['list'],
  ],
  
  use: {
    baseURL: 'http://0.0.0.0:8787',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] },
    },
  ],

  // webServer: {
  //   command: 'python3 -m http.server 8787',
  //   url: 'http://localhost:8787',
  //   reuseExistingServer: true,
  //   timeout: 120 * 1000,
  // },
});
