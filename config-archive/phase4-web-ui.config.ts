import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for WOR-220: Phase 4 Web UI Tests
 * 
 * These tests cover:
 * - map-view: Canvas rendering, overlays, pan/zoom, interactions
 * - timeline: Event timeline display, filtering, eras
 * - app: Overall app initialization, routing, loading states
 * 
 * Usage:
 *   npx playwright test e2e/phase4-web-ui-tests.spec.ts
 *   npx playwright test e2e/phase4-web-ui-tests.spec.ts --headed
 *   npx playwright test e2e/phase4-web-ui-tests.spec.ts --project=chromium
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: 'phase4-web-ui-tests.spec.ts',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 1,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html'],
    ['list'],
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