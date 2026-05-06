import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Screenshot Tests (WOR-434)
 * Captures UI screenshots for visual verification
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/screenshot-tests.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0, // Screenshots need fresh runs
  workers: 1,
  reporter: [
    ['list'],
    ['html', { open: 'never' }], // Optional: generate HTML report
  ],
  
  use: {
    baseURL: 'http://localhost:8765',
    // Always take screenshots on test completion
    screenshot: 'only-on-failure', // Change to 'always' for full capture
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'chromium-headed',
      use: { ...devices['Desktop Chrome'], headless: false },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  timeout: 30000,
});