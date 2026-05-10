import { test, expect } from '@playwright/test';

const BASE_URL = process.env.PREVIEW_URL || 'http://localhost:8765';

test.describe('Frontend Smoke Tests', () => {
  test('landing page loads without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(`${BASE_URL}/landing.html`);
    await expect(page).toHaveTitle(/World Factory|Ion|Landing/i);
    expect(errors.filter(e => !e.includes('favicon'))).toHaveLength(0);
  });

  test('dashboard page loads without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(`${BASE_URL}/dashboard.html`);
    // Dashboard should load without crashing
    await page.waitForLoadState('domcontentloaded');
    expect(errors.filter(e => !e.includes('favicon'))).toHaveLength(0);
  });

  test('map page loads without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(`${BASE_URL}/map.html`);
    await page.waitForLoadState('domcontentloaded');
    expect(errors.filter(e => !e.includes('favicon'))).toHaveLength(0);
  });

  test('timeline page loads without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(`${BASE_URL}/timeline.html`);
    await page.waitForLoadState('domcontentloaded');
    expect(errors.filter(e => !e.includes('favicon'))).toHaveLength(0);
  });
});
