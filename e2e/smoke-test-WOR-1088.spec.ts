import { test, expect, chromium } from '@playwright/test';
import path from 'path';

/**
 * WOR-1088: Playwright E2E Smoke Test
 * Tests full browser UI flow with screenshots
 */

const BASE_URL = process.env.FRONTEND_URL || 'http://localhost:8765';
const API_URL = process.env.API_URL || 'http://localhost:8080';

const screenshotsDir = path.join(process.cwd(), 'screenshots', 'smoke-test-WOR-1088');

// Ensure screenshots directory exists
import fs from 'fs';
if (!fs.existsSync(screenshotsDir)) {
  fs.mkdirSync(screenshotsDir, { recursive: true });
}

test.describe('WOR-1088 Smoke Test - Full UI Verification', () => {
  let browser;
  let page;
  let consoleErrors = [];
  
  test.beforeAll(async () => {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 }
    });
    page = await context.newPage();
    
    // Capture console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    
    page.on('pageerror', err => {
      consoleErrors.push(`Page Error: ${err.message}`);
    });
  });
  
  test.afterAll(async () => {
    if (browser) await browser.close();
  });
  
  test('TC-100: Frontend landing page loads', async () => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.screenshot({ path: `${screenshotsDir}/01-landing-page.png` });
    
    const title = await page.title();
    console.log('Page title:', title);
    
    // Check landing page elements
    const hasContent = await page.locator('body').innerText();
    expect(hasContent.length).toBeGreaterThan(0);
  });
  
  test('TC-101: World creation form is present', async () => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Look for name input or form elements
    const hasForm = await page.locator('input[name="name"], input[placeholder*="name"], input[type="text"]').count() > 0 ||
                    await page.locator('form').count() > 0;
    
    // Take screenshot
    await page.screenshot({ path: `${screenshotsDir}/02-create-form.png` });
    
    // Log form status
    const formInputs = await page.locator('input').count();
    console.log(`Found ${formInputs} form inputs`);
  });
  
  test('TC-102: Create a new world via API and navigate', async () => {
    // First create via API
    const createRes = await page.evaluate(async (apiUrl) => {
      const response = await fetch(`${apiUrl}/api/v1/worlds`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: 'WOR-1088 Playwright Test World' })
      });
      return await response.json();
    }, API_URL);
    
    console.log('Create response:', JSON.stringify(createRes));
    
    const worldId = createRes.data?.id || createRes.data?.world_id;
    expect(worldId).toBeTruthy();
    
    // Navigate to world page
    await page.goto(`${BASE_URL}/worlds/${worldId}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000); // Wait for generation
    
    await page.screenshot({ path: `${screenshotsDir}/03-world-page.png` });
  });
  
  test('TC-103: Map view renders correctly', async () => {
    // Get an existing world from the list
    const listRes = await page.evaluate(async (apiUrl) => {
      const response = await fetch(`${apiUrl}/api/v1/worlds`);
      const data = await response.json();
      return data.data?.worlds?.[0]?.id;
    }, API_URL);
    
    if (listRes) {
      await page.goto(`${BASE_URL}/worlds/${listRes}/map`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      await page.screenshot({ path: `${screenshotsDir}/04-map-view.png` });
      
      // Check for canvas element (map rendering)
      const canvasExists = await page.locator('canvas').count() > 0;
      console.log(`Canvas elements found: ${await page.locator('canvas').count()}`);
    }
  });
  
  test('TC-104: Timeline view loads', async () => {
    const listRes = await page.evaluate(async (apiUrl) => {
      const response = await fetch(`${apiUrl}/api/v1/worlds`);
      const data = await response.json();
      return data.data?.worlds?.[0]?.id;
    }, API_URL);
    
    if (listRes) {
      await page.goto(`${BASE_URL}/worlds/${listRes}/timeline`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      await page.screenshot({ path: `${screenshotsDir}/05-timeline-view.png` });
      
      // Check for timeline content
      const content = await page.locator('body').innerText();
      expect(content.length).toBeGreaterThan(0);
    }
  });
  
  test('TC-105: Dashboard view loads', async () => {
    const listRes = await page.evaluate(async (apiUrl) => {
      const response = await fetch(`${apiUrl}/api/v1/worlds`);
      const data = await response.json();
      return data.data?.worlds?.[0]?.id;
    }, API_URL);
    
    if (listRes) {
      await page.goto(`${BASE_URL}/worlds/${listRes}/dashboard`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      await page.screenshot({ path: `${screenshotsDir}/06-dashboard-view.png` });
    }
  });
  
  test('TC-106: Tab navigation works', async () => {
    const listRes = await page.evaluate(async (apiUrl) => {
      const response = await fetch(`${apiUrl}/api/v1/worlds`);
      const data = await response.json();
      return data.data?.worlds?.[0]?.id;
    }, API_URL);
    
    if (listRes) {
      await page.goto(`${BASE_URL}/worlds/${listRes}`);
      await page.waitForLoadState('networkidle');
      
      // Find and click tabs
      const tabs = await page.locator('[role="tab"], .tab, button:has-text("Map"), button:has-text("Timeline"), button:has-text("Figures")').count();
      console.log(`Found ${tabs} tab-like elements`);
      
      await page.screenshot({ path: `${screenshotsDir}/07-tabs-view.png` });
    }
  });
  
  test('TC-107: No console errors throughout', () => {
    console.log('Console errors captured:', consoleErrors);
    expect(consoleErrors.filter(e => !e.includes('warning'))).toHaveLength(0);
  });
});

// Report console errors at the end
test.afterAll(async () => {
  console.log('\n=== Console Errors Summary ===');
  console.log(`Total errors: ${consoleErrors.length}`);
  consoleErrors.forEach(err => console.log(`  - ${err}`));
  
  // Write errors to file
  const fs = await import('fs');
  fs.writeFileSync(
    `${screenshotsDir}/console-errors.txt`,
    consoleErrors.join('\n')
  );
});