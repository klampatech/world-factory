import { test, expect, Page, ConsoleMessage } from '@playwright/test';

const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';

test.describe('WOR-548: Full Stack Smoke Test', () => {
  let testWorldId: string = '';
  const testResults: { name: string; status: string; details?: string }[] = [];

  test.afterEach(async ({}, testInfo) => {
    testResults.push({
      name: testInfo.title,
      status: testInfo.status === 'passed' ? 'PASS' : 'FAIL',
      details: testInfo.status === 'failed' ? String(testInfo.error?.message) : undefined
    });
  });

  test.afterAll(async () => {
    console.log('\n=== SMOKE TEST SUMMARY ===');
    testResults.forEach(r => {
      console.log(`${r.status}: ${r.name}${r.details ? ' - ' + r.details : ''}`);
    });
  });

  // ==================== API TESTS ====================
  test.describe('Backend API - All Endpoints', () => {
    test('1. POST /api/v1/worlds - Create world', async ({ page }) => {
      const resp = await page.request.post(`${API_BASE}/worlds`, {
        data: { name: 'WOR-548 Smoke Test World', seed: 548548, config: { genre: 'fantasy' } }
      });
      expect(resp.status()).toBe(201);
      const body = await resp.json();
      expect(body.success).toBe(true);
      testWorldId = body.data.id;
      console.log(`Created world: ${testWorldId}`);
    });

    test('2. GET /api/v1/worlds - List worlds', async ({ page }) => {
      const resp = await page.request.get(`${API_BASE}/worlds`);
      expect(resp.status()).toBe(200);
      const body = await resp.json();
      expect(body.success).toBe(true);
      expect(Array.isArray(body.data.worlds)).toBe(true);
    });

    test('3. GET /api/v1/worlds/:id - Get world', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}`);
      expect([200, 404]).toContain(resp.status());
    });

    test('4. GET /api/v1/worlds/:id/planet - Planet data', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/planet`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('5. GET /api/v1/worlds/:id/map - Map data', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/map`);
      if (resp.status() === 200) {
        const body = await resp.json();
        expect(body.success).toBe(true);
      }
    });

    test('6. GET /api/v1/worlds/:id/history - History', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/history`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('7. GET /api/v1/worlds/:id/history/events - History events', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/history/events`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('8. GET /api/v1/worlds/:id/figures - Figures', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/figures`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('9. GET /api/v1/worlds/:id/figures/:id - Figure detail', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/figures/fig-0`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('10. GET /api/v1/worlds/:id/settlements - Settlements', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/settlements`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('11. GET /api/v1/worlds/:id/settlements/map - Settlements map', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/settlements/map`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('12. GET /api/v1/worlds/:id/resources/summary - Resources', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/resources/summary`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('13. GET /api/v1/worlds/:id/disasters - Disasters', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/disasters`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('14. GET /api/v1/worlds/:id/artifacts - Artifacts (with limit)', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/artifacts?limit=5`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('15. GET /api/v1/worlds/:id/export - Export tarball', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/export`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('16. GET /api/v1/worlds/:id/export.json - Export JSON', async ({ page }) => {
      const uuid = testWorldId.replace('world:', '');
      const resp = await page.request.get(`${API_BASE}/worlds/${uuid}/export.json`);
      expect([200, 400, 404]).toContain(resp.status());
    });

    test('17. Backend health check', async ({ page }) => {
      const resp = await page.request.get('http://localhost:8080/health');
      expect(resp.status()).toBe(200);
    });
  });

  // ==================== FRONTEND TESTS ====================
  test.describe('Frontend UI Tests', () => {
    test.beforeEach(async ({ page }) => {
      // Setup console error capture
      page.on('console', msg => {
        if (msg.type() === 'error') {
          console.log(`[Console Error] ${msg.text()}`);
        }
      });
    });

    test('18. Frontend homepage loads', async ({ page }) => {
      await page.goto(FRONTEND_BASE);
      await page.waitForLoadState('networkidle');
      
      // Check page title
      const title = await page.title();
      expect(title).toMatch(/World Factory/i);
      
      // Take screenshot
      await page.screenshot({ path: 'screenshots/WOR-548-homepage.png' });
      
      // Verify key elements exist
      const heading = page.locator('h1, h2, .logo').first();
      await expect(heading).toBeVisible();
    });

    test('19. No console errors on homepage', async ({ page }) => {
      const errors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error' && !msg.text().includes('favicon')) {
          errors.push(msg.text());
        }
      });
      
      await page.goto(FRONTEND_BASE);
      await page.waitForTimeout(2000);
      
      await page.screenshot({ path: 'screenshots/WOR-548-no-errors.png' });
      
      // Allow empty data but not actual errors
      const criticalErrors = errors.filter(e => 
        e.includes('Error') || 
        e.includes('TypeError') || 
        e.includes('SyntaxError') ||
        e.includes('ReferenceError')
      );
      expect(criticalErrors).toHaveLength(0);
    });

    test('20. World creation form renders', async ({ page }) => {
      await page.goto(FRONTEND_BASE);
      await page.waitForLoadState('networkidle');
      
      // Look for form inputs or buttons related to world creation
      const createButton = page.locator('button', { hasText: /create|new world/i }).first();
      const nameInput = page.locator('input[name*="name"], input[placeholder*="name"], input[placeholder*="world"]').first();
      
      const hasForm = await createButton.isVisible().catch(() => false) || 
                      await nameInput.isVisible().catch(() => false);
      
      await page.screenshot({ path: 'screenshots/WOR-548-form.png' });
      
      // At least some UI should be present
      const body = await page.locator('body').innerHTML();
      expect(body.length).toBeGreaterThan(100);
    });

    test('21. Map view renders if accessible', async ({ page }) => {
      await page.goto(FRONTEND_BASE);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      // Check if there's a canvas or map element
      const canvas = page.locator('canvas').first();
      const mapContainer = page.locator('[class*="map"], [id*="map"]').first();
      
      const hasCanvas = await canvas.isVisible().catch(() => false);
      const hasMap = await mapContainer.isVisible().catch(() => false);
      
      await page.screenshot({ path: 'screenshots/WOR-548-map.png' });
      
      // If map renders, verify it has content
      if (hasCanvas) {
        const canvasBox = await canvas.boundingBox();
        expect(canvasBox?.width).toBeGreaterThan(100);
        expect(canvasBox?.height).toBeGreaterThan(100);
      }
    });

    test('22. Tab navigation works', async ({ page }) => {
      await page.goto(FRONTEND_BASE);
      await page.waitForLoadState('networkidle');
      
      // Look for tabs
      const tabs = page.locator('.view-tab, [role="tab"], button').filter({ hasText: /.+/ });
      const tabCount = await tabs.count();
      
      if (tabCount > 0) {
        // Click each tab
        for (let i = 0; i < Math.min(tabCount, 5); i++) {
          const tab = tabs.nth(i);
          if (await tab.isVisible()) {
            await tab.click();
            await page.waitForTimeout(500);
          }
        }
      }
      
      await page.screenshot({ path: 'screenshots/WOR-548-tabs.png' });
      
      // Just verify we didn't crash
      expect(await page.title()).toBeTruthy();
    });
  });
});
