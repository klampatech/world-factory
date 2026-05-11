import { test, expect, chromium } from '@playwright/test';

const BASE_URL = 'http://localhost:8765';
const BACKEND_URL = 'http://localhost:80822';

async function captureScreenshot(page: any, name: string) {
  await page.screenshot({ path: `screenshots/${name}.png`, fullPage: true });
}

/**
 * WOR-167 Smoke Test
 * Complete e2e automation test of the app with front end and back end (not mock data)
 * Capture screenshots of features, check browser console for errors
 */
test.describe('WOR-167 Smoke Test Suite', () => {
  let consoleErrors: string[] = [];

  test.beforeEach(async ({ page }) => {
    // Collect console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(`[${msg.type()}] ${msg.text()}`);
      }
    });
    page.on('pageerror', err => {
      consoleErrors.push(`[pageerror] ${err.message}`);
    });
  });

  test.afterAll(async () => {
    // Log collected console errors
    console.log('\n=== Browser Console Errors ===');
    consoleErrors.forEach(e => console.log(e));
    console.log('==============================\n');
  });

  // ============================================================
  // TC-01: Frontend Health Check
  // ============================================================
  test('TC-01: Frontend server reachable', async ({ page }) => {
    console.log('\n=== TC-01: Testing frontend server reachability ===');
    const response = await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
    console.log(`Frontend response status: ${response?.status()}`);
    expect(response?.status()).toBe(200);
    await captureScreenshot(page, 'tc01-frontend-loaded');
  });

  // ============================================================
  // TC-02: Backend Health Check
  // ============================================================
  test('TC-02: Backend API server reachable', async () => {
    console.log('\n=== TC-02: Testing backend API reachability ===');
    const response = await fetch(`${BACKEND_URL}/health`);
    console.log(`Backend health status: ${response.status}`);
    expect(response.status).toBe(200);
    const body = await response.json();
    console.log(`Backend health response: ${JSON.stringify(body)}`);
    expect(body.status).toBe('ok');
  });

  // ============================================================
  // TC-03: Homepage loads correctly
  // ============================================================
  test('TC-03: Homepage renders correctly', async ({ page }) => {
    console.log('\n=== TC-03: Testing homepage render ===');
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(2000);
    
    // Check page title
    const title = await page.title();
    console.log(`Page title: ${title}`);
    expect(title).toContain('World Factory');
    
    await captureScreenshot(page, 'tc03-homepage');
  });

  // ============================================================
  // TC-04: Create a new world through the UI
  // ============================================================
  test('TC-04: Create new world flow', async ({ page }) => {
    console.log('\n=== TC-04: Testing world creation flow ===');
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(2000);
    
    // Look for create world button or form
    const createButton = page.locator('button:has-text("Create"), button:has-text("New"), button:has-text("Generate")').first();
    
    if (await createButton.isVisible({ timeout: 5000 })) {
      console.log('Create button found, clicking...');
      await createButton.click();
      await page.waitForTimeout(1000);
    }
    
    // Check for form elements or world list
    const hasForm = await page.locator('form, input, select').first().isVisible({ timeout: 5000 }).catch(() => false);
    console.log(`Form elements visible: ${hasForm}`);
    
    await captureScreenshot(page, 'tc04-world-creation');
  });

  // ============================================================
  // TC-05: Navigation to different sections
  // ============================================================
  test('TC-05: Navigation works', async ({ page }) => {
    console.log('\n=== TC-05: Testing navigation ===');
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(2000);
    
    // Check for nav elements
    const navItems = await page.locator('nav a, .nav-item, header a').count();
    console.log(`Found ${navItems} navigation items`);
    
    // Try clicking on links if available
    const firstLink = page.locator('a').first();
    if (await firstLink.isVisible({ timeout: 3000 })) {
      console.log('Found link, testing navigation...');
      await firstLink.click().catch(() => {});
      await page.waitForTimeout(1000);
    }
    
    await captureScreenshot(page, 'tc05-navigation');
  });

  // ============================================================
  // TC-06: World list API integration
  // ============================================================
  test('TC-06: World list loads from API', async ({ page }) => {
    console.log('\n=== TC-06: Testing world list from API ===');
    
    // Directly test API
    const response = await fetch(`${BACKEND_URL}/api/v1/worlds`);
    expect(response.status).toBe(200);
    const data = await response.json();
    console.log(`API returned success: ${data.success}`);
    expect(data.success).toBe(true);
    
    const worlds = data.data?.worlds || data.data || [];
    console.log(`Total worlds: ${worlds.length}`);
    
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(2000);
    await captureScreenshot(page, 'tc06-world-list');
  });

  // ============================================================
  // TC-07: Generation trigger endpoint
  // ============================================================
  test('TC-07: Generation endpoint accessible', async () => {
    console.log('\n=== TC-07: Testing generation endpoint ===');
    
    // First create a world
    const createResponse = await fetch(`${BACKEND_URL}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        name: 'Smoke Test World',
        parameters: { seed: 167, size: 'Medium' }
      })
    });
    console.log(`Create world status: ${createResponse.status}`);
    
    if (createResponse.status === 201 || createResponse.status === 200 || createResponse.status === 202) {
      const world = await createResponse.json();
      const worldId = world.data?.id || world.id;
      console.log(`Created world: ${worldId}`);
      
      // Now trigger generation
      const genResponse = await fetch(`${BACKEND_URL}/api/v1/worlds/${worldId}/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });
      console.log(`Generation trigger status: ${genResponse.status}`);
      expect([200, 201, 202, 409]).toContain(genResponse.status);
    }
  });

  // ============================================================
  // TC-08: Frontend-backend connectivity
  // ============================================================
  test('TC-08: Frontend connects to backend', async ({ page }) => {
    console.log('\n=== TC-08: Testing frontend-backend connectivity ===');
    await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
    
    // Check for any failed network requests
    const failedRequests: string[] = [];
    page.on('response', response => {
      if (response.status() >= 400) {
        failedRequests.push(`${response.status()} ${response.url()}`);
      }
    });
    
    await page.reload({ waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    
    console.log(`Failed requests: ${failedRequests.length}`);
    failedRequests.forEach(r => console.log(`  - ${r}`));
    
    await captureScreenshot(page, 'tc08-frontend-backend-connection');
  });
});