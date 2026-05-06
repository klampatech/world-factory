import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  testMatch: '**/frontend-smoke-tests.spec.ts',
  
  timeout: 60000,
  retries: 0,
  workers: 1,
  
  reporter: [
    ['list'],
    ['html', { outputFolder: 'test-results/WOR-223-smoke-report' }],
  ],
  
  use: {
    baseURL: 'http://0.0.0.0:8787',
    trace: 'on-first-retry',
    screenshot: 'on',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
