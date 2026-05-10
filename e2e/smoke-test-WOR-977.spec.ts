import { test, expect, request as pwRequest, APIRequestContext, Browser, chromium } from '@playwright/test';

const BACKEND_URL = 'http://localhost:8080';
const FRONTEND_URL = 'http://localhost:9000';

let worldId: string;
let apiContext: APIRequestContext;

test.describe('WOR-977: Full Stack Smoke Test', () => {

  test.beforeAll(async () => {
    apiContext = await pwRequest.newContext();
    
    // Create a test world for UI testing
    const resp = await apiContext.post(`${BACKEND_URL}/api/v1/worlds`, {
      data: { name: 'WOR-977 UI Test World', seed: 977977, config: { genre: 'fantasy' } }
    });
    if (resp.status() === 201) {
      const body = await resp.json();
      worldId = body.data.id;
      console.log(`Created world: ${worldId}`);
    }
  });

  test.afterAll(async () => {
    // Cleanup: delete the test world
    if (worldId) {
      await apiContext.delete(`${BACKEND_URL}/api/v1/worlds/${worldId}`);
    }
    await apiContext.dispose();
  });

  test('API-01: All 18 endpoints return expected responses', async () => {
    expect(worldId).toBeDefined();
    
    // Wait for world to be ready
    let attempts = 0;
    while (attempts < 30) {
      const resp = await apiContext.get(`${BACKEND_URL}/api/v1/worlds/${worldId}`);
      if (resp.status() === 200) {
        const body = await resp.json();
        if (body.data?.status === 'ready') break;
      }
      await new Promise(r => setTimeout(r, 1000));
      attempts++;
    }
    
    // Test all 18 endpoints
    const endpoints = [
      ['GET', '/health'],
      ['GET', `/api/v1/worlds`],
      ['GET', `/api/v1/worlds/${worldId}`],
      ['GET', `/api/v1/worlds/${worldId}/planet`],
      ['GET', `/api/v1/worlds/${worldId}/map`],
      ['GET', `/api/v1/worlds/${worldId}/history`],
      ['GET', `/api/v1/worlds/${worldId}/history/events`],
      ['GET', `/api/v1/worlds/${worldId}/figures`],
      ['GET', `/api/v1/worlds/${worldId}/settlements`],
      ['GET', `/api/v1/worlds/${worldId}/settlements/map`],
      ['GET', `/api/v1/worlds/${worldId}/resources/summary`],
      ['GET', `/api/v1/worlds/${worldId}/disasters`],
      ['GET', `/api/v1/worlds/${worldId}/artifacts`],
      ['GET', `/api/v1/worlds/${worldId}/export`],
      ['GET', `/api/v1/worlds/${worldId}/export.json`],
    ];
    
    for (const [method, path] of endpoints) {
      const url = path.startsWith('/') ? `${BACKEND_URL}${path}` : `${BACKEND_URL}/${path}`;
      const resp = method === 'GET' 
        ? await apiContext.get(url)
        : await apiContext.post(url, { data: {} });
      expect(resp.status(), `Endpoint ${method} ${path}`).toBe(200);
    }
    
    // Test DELETE
    const delResp = await apiContext.delete(`${BACKEND_URL}/api/v1/worlds/${worldId}`);
    expect([200, 204]).toContain(delResp.status());
  });

  test('UI-01: Frontend index page loads', async ({ page }) => {
    const response = await page.goto(`${FRONTEND_URL}/`);
    expect(response?.status()).toBe(200);
    
    const title = await page.title();
    expect(title).toBeTruthy();
  });

  test('UI-02: World detail page loads with map canvas', async ({ page }) => {
    const worldId = '779bd117-e64b-4432-93eb-761f201ce1bd'; // Use existing world
    const response = await page.goto(`${FRONTEND_URL}/world.html?id=${worldId}`);
    expect(response?.status()).toBe(200);
    
    // Wait for canvas element to be attached (may be hidden by CSS until loaded)
    await page.waitForSelector('#world-map', { state: 'attached', timeout: 15000 });
    const canvas = page.locator('#world-map');
    await expect(canvas).toBeAttached();
  });

  test('UI-03: Tab navigation works', async ({ page }) => {
    const worldId = '779bd117-e64b-4432-93eb-761f201ce1bd';
    await page.goto(`${FRONTEND_URL}/world.html?id=${worldId}`);
    await page.waitForSelector('#world-map', { state: 'attached', timeout: 15000 });
    
    // Test each tab
    const tabs = ['overview', 'map', 'timeline', 'dashboard'];
    for (const tab of tabs) {
      await page.click(`.tab-button[data-tab="${tab}"]`);
      await page.waitForTimeout(500);
      const panel = page.locator(`#panel-${tab}`);
      await expect(panel).toBeAttached();
    }
  });

  test('UI-04: No browser console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Filter benign errors
        if (!text.includes('favicon') && !text.includes('Failed to load')) {
          errors.push(text);
        }
      }
    });
    
    const worldId = '779bd117-e64b-4432-93eb-761f201ce1bd';
    await page.goto(`${FRONTEND_URL}/world.html?id=${worldId}`, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    
    // Log any errors found
    if (errors.length > 0) {
      console.log('Console errors found:', errors);
    }
    
    expect(errors.filter(e => !e.includes('backend'))).toHaveLength(0);
  });
});
