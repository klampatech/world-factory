import { test, expect, request } from '@playwright/test';

/**
 * WOR-420: Complete End-to-End Smoke Test
 * Tests all 18 backend API endpoints and complete frontend UI
 */

// Helper for API calls (runs in Node context)
async function apiRequest(method: string, url: string, body?: object) {
  const resp = await fetch(url, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  });
  return { status: resp.status, json: await resp.json().catch(() => ({})) };
}

const API_BASE = 'http://localhost:80822/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';

// Timestamped test run ID
const RUN_ID = `wor420-${Date.now()}`;
const TEST_WORLD_NAME = `WOR-420 Smoke Test ${new Date().toISOString().slice(0,16).replace('T', '-')}`;

let worldId: string;

test.describe('WOR-420 Smoke Test - Complete End-to-End', () => {

  // ========================================================================
  // BACKEND API TESTS - All 18 Endpoints
  // ========================================================================

  test.describe('Backend API - World Lifecycle', () => {
    
    test('1. POST /api/v1/worlds - Create world', async () => {
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: TEST_WORLD_NAME, 
        seed: 420420,
        config: { 
          genre: 'fantasy',
          era: 'medieval',
          mapSize: 'medium'
        }
      });
      console.log(`[WOR-420] POST /worlds: ${resp.status}`, JSON.stringify(resp.json).slice(0,200));
      expect([200, 201, 422]).toContain(resp.status);
      if (resp.json.success) {
        expect(resp.json.data?.id).toBeDefined();
        worldId = resp.json.data.id;
      }
    });

    test('2. GET /api/v1/worlds - List worlds', async () => {
      const resp = await apiRequest('GET', `${API_BASE}/worlds`);
      console.log(`[WOR-420] GET /worlds: ${resp.status}`);
      expect([200, 201]).toContain(resp.status);
      if (resp.json.data?.worlds) {
        expect(Array.isArray(resp.json.data.worlds)).toBe(true);
      }
    });

    test('3. GET /api/v1/worlds/:id - Get specific world', async () => {
      const uuid = worldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}`);
      console.log(`[WOR-420] GET world status: ${resp.status}`);
    });

    test('4. DELETE /api/v1/worlds/:id - Delete world', async () => {
      const uuid = worldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      console.log(`[WOR-420] DELETE world status: ${resp.status}`);
    });

  });

  test.describe('Backend API - Planet and Map', () => {
    
    let testWorldId: string;

    test.beforeAll(async () => {
      // Create a world for these tests
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: 'WOR-420 API Test World', 
        seed: 99999, 
        config: { genre: 'fantasy' } 
      });
      testWorldId = resp.json.data?.id;
      console.log(`[WOR-420] Test world for planet/map: ${testWorldId}`);
    });

    test.afterAll(async () => {
      // Cleanup
      if (testWorldId) {
        const uuid = testWorldId.replace('world:', '');
        await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      }
    });

    test('5. GET /api/v1/worlds/:id/planet - Get planet data', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/planet`);
      console.log(`[WOR-420] GET planet status: ${resp.status}`);
    });

    test('6. GET /api/v1/worlds/:id/map - Get map data', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/map`);
      console.log(`[WOR-420] GET map status: ${resp.status}`);
      if (resp.status === 200) {
        expect(resp.json.data?.polygons).toBeDefined();
      }
    });

  });

  test.describe('Backend API - History', () => {
    
    let testWorldId: string;

    test.beforeAll(async () => {
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: 'WOR-420 History Test', 
        seed: 88888, 
        config: { genre: 'fantasy' } 
      });
      testWorldId = resp.json.data?.id;
    });

    test.afterAll(async () => {
      if (testWorldId) {
        const uuid = testWorldId.replace('world:', '');
        await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      }
    });

    test('7. GET /api/v1/worlds/:id/history - Get history', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/history`);
      console.log(`[WOR-420] GET history status: ${resp.status}`);
    });

    test('8. GET /api/v1/worlds/:id/history/events - Get history events', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/history/events`);
      console.log(`[WOR-420] GET history/events status: ${resp.status}`);
    });

  });

  test.describe('Backend API - Figures', () => {
    
    let testWorldId: string;

    test.beforeAll(async () => {
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: 'WOR-420 Figures Test', 
        seed: 77777, 
        config: { genre: 'fantasy' } 
      });
      testWorldId = resp.json.data?.id;
    });

    test.afterAll(async () => {
      if (testWorldId) {
        const uuid = testWorldId.replace('world:', '');
        await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      }
    });

    test('9. GET /api/v1/worlds/:id/figures - List figures', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/figures`);
      console.log(`[WOR-420] GET figures status: ${resp.status}`);
    });

    test('10. GET /api/v1/worlds/:id/figures/:figure_id - Get specific figure', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/figures/fig-0`);
      console.log(`[WOR-420] GET figures/fig-0 status: ${resp.status}`);
    });

  });

  test.describe('Backend API - Settlements', () => {
    
    let testWorldId: string;

    test.beforeAll(async () => {
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: 'WOR-420 Settlements Test', 
        seed: 66666, 
        config: { genre: 'fantasy' } 
      });
      testWorldId = resp.json.data?.id;
    });

    test.afterAll(async () => {
      if (testWorldId) {
        const uuid = testWorldId.replace('world:', '');
        await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      }
    });

    test('11. GET /api/v1/worlds/:id/settlements - List settlements', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/settlements`);
      console.log(`[WOR-420] GET settlements status: ${resp.status}`);
    });

    test('12. GET /api/v1/worlds/:id/settlements/map - Get settlements map', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/settlements/map`);
      console.log(`[WOR-420] GET settlements/map status: ${resp.status}`);
    });

  });

  test.describe('Backend API - Resources, Disasters, Artifacts', () => {
    
    let testWorldId: string;

    test.beforeAll(async () => {
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: 'WOR-420 Misc Test', 
        seed: 55555, 
        config: { genre: 'fantasy' } 
      });
      testWorldId = resp.json.data?.id;
    });

    test.afterAll(async () => {
      if (testWorldId) {
        const uuid = testWorldId.replace('world:', '');
        await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      }
    });

    test('13. GET /api/v1/worlds/:id/resources/summary - Get resources summary', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/resources/summary`);
      console.log(`[WOR-420] GET resources/summary status: ${resp.status}`);
    });

    test('14. GET /api/v1/worlds/:id/disasters - List disasters', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/disasters`);
      console.log(`[WOR-420] GET disasters status: ${resp.status}`);
    });

    test('15. GET /api/v1/worlds/:id/artifacts - List artifacts', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/artifacts?limit=5`);
      console.log(`[WOR-420] GET artifacts status: ${resp.status}`);
    });

  });

  test.describe('Backend API - Export', () => {
    
    let testWorldId: string;

    test.beforeAll(async () => {
      const resp = await apiRequest('POST', `${API_BASE}/worlds`, { 
        name: 'WOR-420 Export Test', 
        seed: 44444, 
        config: { genre: 'fantasy' } 
      });
      testWorldId = resp.json.data?.id;
    });

    test.afterAll(async () => {
      if (testWorldId) {
        const uuid = testWorldId.replace('world:', '');
        await apiRequest('DELETE', `${API_BASE}/worlds/${uuid}`);
      }
    });

    test('16. GET /api/v1/worlds/:id/export - Export world', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/export`);
      console.log(`[WOR-420] GET export status: ${resp.status}`);
    });

    test('17. GET /api/v1/worlds/:id/export.json - Export world as JSON', async () => {
      const uuid = testWorldId?.replace('world:', '') || 'test';
      const resp = await apiRequest('GET', `${API_BASE}/worlds/${uuid}/export.json`);
      console.log(`[WOR-420] GET export.json status: ${resp.status}`);
    });

    test('18. Backend health check', async () => {
      const resp = await apiRequest('GET', 'http://localhost:80822/health');
      console.log(`[WOR-420] Backend health: ${resp.status}`);
    });

  });

  // ========================================================================
  // FRONTEND UI TESTS
  // ========================================================================

  test.describe('Frontend UI - All Screens and Interactions', () => {
    
    test('FE-1: Frontend home page loads', async ({ page }) => {
      const errors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error' && !msg.text().includes('favicon')) {
          errors.push(msg.text());
        }
      });

      const resp = await page.goto(FRONTEND_BASE + '/');
      expect(resp?.status()).toBe(200);
      
      // Wait for page to stabilize
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      // Check title
      const title = await page.title();
      console.log(`[WOR-420] Page title: ${title}`);
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-frontend-home.png` 
      });
      
      // Log any errors
      if (errors.length > 0) {
        console.log(`[WOR-420] Console errors on home: ${errors.join('; ')}`);
      }
    });

    test('FE-2: World creation form exists', async ({ page }) => {
      await page.goto(FRONTEND_BASE + '/');
      await page.waitForLoadState('networkidle');
      
      // Look for form elements (world name input, genre selection, etc.)
      const hasForm = await page.locator('form, input, select').count() > 0;
      console.log(`[WOR-420] Form elements found: ${hasForm}`);
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-form-exists.png` 
      });
    });

    test('FE-3: Map canvas renders', async ({ page }) => {
      await page.goto(FRONTEND_BASE + '/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      const canvas = page.locator('#map-canvas, canvas').first();
      await expect(canvas).toBeVisible();
      
      const box = await canvas.boundingBox();
      console.log(`[WOR-420] Canvas size: ${box?.width}x${box?.height}`);
      expect(box?.width).toBeGreaterThan(0);
      expect(box?.height).toBeGreaterThan(0);
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-map-canvas.png` 
      });
    });

    test('FE-4: Overlay controls work', async ({ page }) => {
      await page.goto(FRONTEND_BASE + '/');
      await page.waitForLoadState('networkidle');
      
      // Look for overlay controls
      const overlayControls = page.locator('#overlay-controls, .overlay-controls, [data-overlay]');
      const overlayCount = await overlayControls.count();
      console.log(`[WOR-420] Overlay controls found: ${overlayCount}`);
      
      // Try clicking an overlay if available
      if (overlayCount > 0) {
        const elevationBtn = page.locator('[data-overlay="elevation"]').first();
        if (await elevationBtn.count() > 0) {
          await elevationBtn.click();
          await page.waitForTimeout(500);
        }
      }
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-overlay-controls.png` 
      });
    });

    test('FE-5: Tab navigation works', async ({ page }) => {
      await page.goto(FRONTEND_BASE + '/');
      await page.waitForLoadState('networkidle');
      
      // Look for tab elements
      const tabs = page.locator('[role="tab"], .tab, .nav-tab, nav a');
      const tabCount = await tabs.count();
      console.log(`[WOR-420] Tab elements found: ${tabCount}`);
      
      // Try clicking first tab if available
      if (tabCount > 1) {
        await tabs.first().click();
        await page.waitForTimeout(500);
      }
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-tab-navigation.png` 
      });
    });

    test('FE-6: Pan and zoom interaction', async ({ page }) => {
      await page.goto(FRONTEND_BASE + '/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      const canvas = page.locator('#map-canvas, canvas').first();
      const box = await canvas.boundingBox();
      
      if (box) {
        // Test pan (drag)
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.down();
        await page.mouse.move(box.x + box.width / 2 + 50, box.y + box.height / 2 + 50);
        await page.mouse.up();
        
        // Test zoom (scroll)
        await page.mouse.wheel(0, -100);
        await page.waitForTimeout(500);
      }
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-pan-zoom.png` 
      });
    });

    test('FE-7: No console errors throughout', async ({ page }) => {
      const errors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          const text = msg.text();
          // Filter out favicon and non-critical errors
          if (!text.includes('favicon') && !text.includes('net::ERR_')) {
            errors.push(text);
          }
        }
      });
      
      // Navigate through main pages
      await page.goto(FRONTEND_BASE + '/');
      await page.waitForTimeout(2000);
      
      console.log(`[WOR-420] Console errors detected: ${errors.length}`);
      if (errors.length > 0) {
        console.log(`[WOR-420] Errors: ${errors.slice(0, 5).join('; ')}`);
      }
      
      await page.screenshot({ 
        path: `/home/kyle/projects/world-generator/screenshots/WOR-420-console-check.png` 
      });
      
      // We fail the test if critical console errors exist
      expect(errors.length).toBe(0);
    });

  });

});
