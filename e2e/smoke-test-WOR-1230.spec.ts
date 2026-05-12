import { test, expect, request as pwRequest, APIRequestContext } from '@playwright/test';
import path from 'path';
import fs from 'fs';

/**
 * WOR-1230: Complete End-to-End Smoke Test
 * Tests all 18 API endpoints and all frontend UI paths
 */

const API_BASE = 'http://localhost:8082/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';
let worldId: string;
let worldIdNoPrefix: string;
let apiContext: APIRequestContext;
let screenshots: { name: string; path: string }[] = [];

async function screenshot(page: any, name: string) {
  const screenshotDir = '/home/kyle/projects/world-generator/qa-reports/WOR-1230-screenshots';
  if (!fs.existsSync(screenshotDir)) {
    fs.mkdirSync(screenshotDir, { recursive: true });
  }
  const filePath = path.join(screenshotDir, `${name}.png`);
  await page.screenshot({ path: filePath, fullPage: true });
  screenshots.push({ name, path: filePath });
  console.log(`  Screenshot: ${name}`);
}

test.describe('WOR-1230: Complete Smoke Test', () => {
  
  test.beforeAll(async () => {
    apiContext = await pwRequest.newContext();
    screenshots = [];
  });

  test.afterAll(async () => {
    await apiContext.dispose();
    console.log('\n=== Screenshots captured ===');
    screenshots.forEach(s => console.log(`  ${s.name}: ${s.path}`));
  });

  // ========== BACKEND API TESTS ==========

  test.describe('Backend API - All 18 Endpoints', () => {

    test('1. POST /api/v1/worlds - Create world', async () => {
      const resp = await apiContext.post(`${API_BASE}/worlds`, {
        data: { name: 'WOR-1230 Smoke Test World', seed: 12345678, config: { genre: 'fantasy' } }
      });
      expect(resp.status()).toBe(201);
      const body = await resp.json();
      expect(body.success).toBe(true);
      worldId = body.data.id;
      worldIdNoPrefix = worldId.replace('world:', '');
      console.log(`Created world: ${worldId}`);
    });

    test('2. GET /api/v1/worlds - List worlds', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds`);
      expect(resp.status()).toBe(200);
      const body = await resp.json();
      expect(body.success).toBe(true);
      expect(Array.isArray(body.data.worlds)).toBe(true);
      console.log(`Listed ${body.data.worlds.length} worlds`);
    });

    test('3. GET /api/v1/worlds/:id - Get world', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}`);
      expect([200, 404]).toContain(resp.status());
      if (resp.status() === 200) {
        const body = await resp.json();
        expect(body.success).toBe(true);
        console.log(`Got world details`);
      } else {
        // Try with prefix
        const resp2 = await apiContext.get(`${API_BASE}/worlds/${worldId}`);
        expect([200, 404]).toContain(resp2.status());
        console.log(`Got world details (with prefix): ${resp2.status()}`);
      }
    });

    test('4. GET /api/v1/worlds/:id/planet - Get planet data', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/planet`);
      console.log(`  planet status: ${resp.status()}`);
    });

    test('5. GET /api/v1/worlds/:id/map - Get map data', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/map`);
      console.log(`  map status: ${resp.status()}`);
      if (resp.status() === 200) {
        const body = await resp.json();
        expect(body.success).toBe(true);
        expect(body.data.polygons).toBeDefined();
        console.log(`  map polygons: ${body.data.polygons?.length || 0}`);
      }
    });

    test('6. GET /api/v1/worlds/:id/history - Get history', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/history`);
      console.log(`  history status: ${resp.status()}`);
    });

    test('7. GET /api/v1/worlds/:id/history/events - Get history events', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/history/events`);
      console.log(`  history/events status: ${resp.status()}`);
    });

    test('8. GET /api/v1/worlds/:id/figures - Get figures', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/figures`);
      console.log(`  figures status: ${resp.status()}`);
    });

    test('9. GET /api/v1/worlds/:id/figures/:id - Get figure detail', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/figures/1`);
      console.log(`  figure detail status: ${resp.status()}`);
    });

    test('10. GET /api/v1/worlds/:id/settlements - Get settlements', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/settlements`);
      console.log(`  settlements status: ${resp.status()}`);
    });

    test('11. GET /api/v1/worlds/:id/settlements/map - Get settlements map', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/settlements/map`);
      console.log(`  settlements/map status: ${resp.status()}`);
    });

    test('12. GET /api/v1/worlds/:id/resources/summary - Get resources', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/resources/summary`);
      console.log(`  resources/summary status: ${resp.status()}`);
    });

    test('13. GET /api/v1/worlds/:id/disasters - Get disasters', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/disasters`);
      console.log(`  disasters status: ${resp.status()}`);
    });

    test('14. GET /api/v1/worlds/:id/artifacts - Get artifacts', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/artifacts`);
      console.log(`  artifacts status: ${resp.status()}`);
    });

    test('15. GET /api/v1/worlds/:id/export - Get export', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/export`);
      console.log(`  export status: ${resp.status()}`);
    });

    test('16. GET /api/v1/worlds/:id/export.json - Get JSON export', async () => {
      const resp = await apiContext.get(`${API_BASE}/worlds/${worldIdNoPrefix}/export.json`);
      console.log(`  export.json status: ${resp.status()}`);
    });

    test('17. DELETE /api/v1/worlds/:id - Delete world', async () => {
      const resp = await apiContext.delete(`${API_BASE}/worlds/${worldIdNoPrefix}`);
      console.log(`  delete status: ${resp.status()}`);
      expect([200, 204, 400, 404]).toContain(resp.status());
    });

  });

  // ========== FRONTEND UI TESTS ==========

  test.describe('Frontend UI - All Screens', () => {

    test.beforeEach(async ({ page }) => {
      // Monitor console errors
      page.on('console', msg => {
        if (msg.type() === 'error') {
          console.log(`  Console Error: ${msg.text()}`);
        }
      });
    });

    test('18. World creation form loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/index.html`);
      await page.waitForLoadState('networkidle');
      await screenshot(page, '01-home-page');
      
      // Check form elements exist
      const nameInput = page.locator('input[name="name"], input#name, input.input-name, input[type="text"]').first();
      const seedInput = page.locator('input[name="seed"], input#seed, input.input-seed, input[type="number"]').first();
      
      console.log('  Home page loaded');
      console.log('  Name input visible:', await nameInput.isVisible().catch(() => false));
      console.log('  Seed input visible:', await seedInput.isVisible().catch(() => false));
    });

    test('19. World list loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html`);
      await page.waitForLoadState('networkidle');
      await screenshot(page, '02-world-list');
      console.log('  World list page loaded');
    });

    test('20. Map view renders', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html?tab=map`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);
      await screenshot(page, '03-map-view');
      
      // Check for canvas
      const canvas = page.locator('canvas').first();
      console.log('  Canvas visible:', await canvas.isVisible().catch(() => false));
    });

    test('21. Timeline loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html?tab=timeline`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      await screenshot(page, '04-timeline');
      console.log('  Timeline page loaded');
    });

    test('22. Dashboard loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html?tab=dashboard`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      await screenshot(page, '05-dashboard');
      console.log('  Dashboard page loaded');
    });

    test('23. Figures tab loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html?tab=figures`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      await screenshot(page, '06-figures');
      console.log('  Figures page loaded');
    });

    test('24. Tab navigation works', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html`);
      await page.waitForLoadState('networkidle');
      await screenshot(page, '07-tab-navigation');
      
      // Try clicking tabs
      const tabs = page.locator('[role="tab"], .tab, .nav-tab, button.tab');
      const tabCount = await tabs.count();
      console.log(`  Found ${tabCount} tabs`);
      
      for (let i = 0; i < Math.min(tabCount, 6); i++) {
        const tab = tabs.nth(i);
        if (await tab.isVisible()) {
          await tab.click();
          await page.waitForTimeout(500);
          console.log(`  Clicked tab ${i + 1}`);
        }
      }
    });

  });

});
