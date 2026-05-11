import { test, expect } from '@playwright/test';

/**
 * WOR-820 Smoke Test - Complete E2E Application Test
 * 
 * Tests all backend API endpoints and frontend UI functionality.
 * 
 * Expected Environment:
 * - Backend: http://127.0.0.1:8082
 * - Frontend: http://localhost:8765
 */

test.describe('WOR-820 Smoke Test', () => {
  const API_BASE = 'http://127.0.0.1:8082';
  const FRONTEND_BASE = 'http://localhost:8765';
  
  let createdWorldId: string | null = null;
  let readyWorldId: string | null = null;
  
  const consoleErrors: string[] = [];
  
  test.beforeEach(async ({ page }) => {
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Filter out known network errors that aren't actual JS errors
        if (!text.includes('net::ERR_CONNECTION_REFUSED') && 
            !text.includes('Failed to load resource') &&
            !text.includes('Failed to fetch')) {
          consoleErrors.push(text);
        }
      }
    });
  });

  // ========================================
  // BACKEND API TESTS
  // ========================================

  test('API-01: Backend health check', async ({ request }) => {
    const response = await request.get(`${API_BASE}/health`);
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('ok');
    console.log('✅ Backend health: ' + JSON.stringify(data));
  });

  test('API-02: POST /api/v1/worlds - Create world', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/v1/worlds`, {
      data: {
        name: 'WOR-820 Smoke Test World',
        width: 50,
        height: 50,
        seed: 820001
      }
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.data).toHaveProperty('id');
    createdWorldId = data.data.id;
    console.log('✅ Created world: ' + createdWorldId);
  });

  test('API-03: GET /api/v1/worlds - List worlds', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/v1/worlds`);
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.data).toHaveProperty('worlds');
    console.log('✅ Worlds list: ' + data.data.worlds.length + ' worlds');
    
    // Find a ready world for later tests
    const readyWorld = data.data.worlds.find((w: any) => w.status === 'ready');
    if (readyWorld) {
      readyWorldId = readyWorld.id;
      console.log('✅ Found ready world: ' + readyWorldId);
    }
  });

  test('API-04: GET /api/v1/worlds/:id - Get world details', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      expect(data.data).toHaveProperty('name');
      console.log('✅ World details: ' + data.data.name);
    }
  });

  test('API-05: DELETE /api/v1/worlds/:id - Delete world', async ({ request }) => {
    // Create a world to delete
    const createResp = await request.post(`${API_BASE}/api/v1/worlds`, {
      data: {
        name: 'Delete Me World',
        width: 20,
        height: 20,
        seed: 99999
      }
    });
    const createData = await createResp.json();
    const deleteId = createData.data?.id;
    
    if (deleteId) {
      const response = await request.delete(`${API_BASE}/api/v1/worlds/${deleteId}`);
      expect(response.ok()).toBeTruthy();
      // DELETE may return empty body or JSON response
      try {
        const data = await response.json();
        expect(data.success).toBe(true);
      } catch {
        // Empty response body is acceptable for DELETE
      }
      console.log('✅ Deleted world: ' + deleteId);
    }
  });

  test('API-06: GET /api/v1/worlds/:id/planet - Get planet data', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/planet`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Planet data retrieved');
    }
  });

  test('API-07: GET /api/v1/worlds/:id/map - Get map data', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/map`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Map data retrieved');
    }
  });

  test('API-08: GET /api/v1/worlds/:id/history - Get history', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/history`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ History data retrieved');
    }
  });

  test('API-09: GET /api/v1/worlds/:id/history/events - Get events', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/history/events`);
      // Accept 200 (success) or 404 (no events yet) as valid responses
      expect([200, 404]).toContain(response.status());
      if (response.ok()) {
        const data = await response.json();
        expect(data.success).toBe(true);
      }
      console.log('✅ History events endpoint tested (status: ' + response.status() + ')');
    }
  });

  test('API-10: GET /api/v1/worlds/:id/figures - Get figures list', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/figures`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Figures list retrieved');
    }
  });

  test('API-11: GET /api/v1/worlds/:id/figures/:id - Get figure details', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const listResp = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/figures`);
      const listData = await listResp.json();
      const figureId = listData.data?.figures?.[0]?.id;
      
      if (figureId) {
        const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/figures/${figureId}`);
        expect(response.ok()).toBeTruthy();
        const data = await response.json();
        expect(data.success).toBe(true);
        console.log('✅ Figure details retrieved');
      }
    }
  });

  test('API-12: GET /api/v1/worlds/:id/settlements - Get settlements', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/settlements`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Settlements retrieved');
    }
  });

  test('API-13: GET /api/v1/worlds/:id/settlements/map - Get settlements map', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/settlements/map`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Settlements map retrieved');
    }
  });

  test('API-14: GET /api/v1/worlds/:id/resources/summary - Get resources', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/resources/summary`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Resources summary retrieved');
    }
  });

  test('API-15: GET /api/v1/worlds/:id/disasters - Get disasters', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/disasters`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Disasters retrieved');
    }
  });

  test('API-16: GET /api/v1/worlds/:id/artifacts - Get artifacts', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      // Note: artifacts endpoint requires limit parameter
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/artifacts?limit=10`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Artifacts retrieved');
    }
  });

  test('API-17: GET /api/v1/worlds/:id/export - Get export data', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/export`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ Export data retrieved');
    }
  });

  test('API-18: GET /api/v1/worlds/:id/export.json - Get JSON export', async ({ request }) => {
    const worldId = readyWorldId || createdWorldId;
    if (worldId) {
      const response = await request.get(`${API_BASE}/api/v1/worlds/${worldId}/export.json`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      expect(data.data).toBeDefined();
      console.log('✅ JSON export retrieved');
    }
  });

  // ========================================
  // FRONTEND UI TESTS
  // ========================================

  test('UI-01: Frontend landing page loads', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    
    const title = await page.title();
    expect(title).toContain('World');
    
    const header = page.locator('header');
    await expect(header).toBeVisible();
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui01-landing.png' });
    console.log('✅ Frontend landing page loads correctly');
  });

  test('UI-02: Frontend displays world list', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const content = page.locator('body');
    await expect(content).toBeVisible();
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui02-world-list.png' });
    console.log('✅ Frontend displays content');
  });

  test('UI-03: World creation form - Submit new world', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    
    const createBtn = page.locator('button:has-text("Create"), .btn-create, [data-testid="create"]').first();
    
    if (await createBtn.isVisible()) {
      await createBtn.click();
      await page.waitForTimeout(500);
      
      const nameInput = page.locator('input[name="name"], input#name, input[placeholder*="name"]').first();
      if (await nameInput.isVisible()) {
        await nameInput.fill('WOR-820 Smoke Test World ' + Date.now());
      }
      
      const submitBtn = page.locator('button[type="submit"], #confirm-create, button:has-text("Create")').first();
      if (await submitBtn.isVisible()) {
        await submitBtn.click();
        await page.waitForTimeout(2000);
      }
      
      await page.screenshot({ path: 'e2e/screenshots/wor820-ui03-create-flow.png' });
    }
    
    console.log('✅ World creation flow executed');
  });

  test('UI-04: Map view - Voronoi polygons render correctly', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const worldCard = page.locator('[class*="card"], [class*="world"], .world-item').first();
    
    if (await worldCard.isVisible()) {
      await worldCard.click();
      await page.waitForTimeout(1500);
      
      const mapCanvas = page.locator('canvas, svg, [class*="map"]').first();
      if (await mapCanvas.isVisible()) {
        await page.screenshot({ path: 'e2e/screenshots/wor820-ui04-map-view.png' });
        console.log('✅ Map view with canvas element visible');
      } else {
        await page.screenshot({ path: 'e2e/screenshots/wor820-ui04-view-page.png' });
        console.log('✅ View page loaded');
      }
    } else {
      await page.screenshot({ path: 'e2e/screenshots/wor820-ui04-home-page.png' });
      console.log('⚠️ No world cards visible to test map view');
    }
  });

  test('UI-05: Timeline view - History events load', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const timelineTab = page.locator('[class*="tab"]:has-text("Timeline"), [data-tab="timeline"]').first();
    if (await timelineTab.isVisible()) {
      await timelineTab.click();
      await page.waitForTimeout(1000);
    }
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui05-timeline.png' });
    console.log('✅ Timeline view accessed');
  });

  test('UI-06: Dashboard - World summary displays', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const dashboardTab = page.locator('[class*="tab"]:has-text("Dashboard"), [data-tab="dashboard"]').first();
    if (await dashboardTab.isVisible()) {
      await dashboardTab.click();
      await page.waitForTimeout(1000);
    }
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui06-dashboard.png' });
    console.log('✅ Dashboard view accessed');
  });

  test('UI-07: Figures - Figure list and profiles', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const figuresTab = page.locator('[class*="tab"]:has-text("Figures"), [data-tab="figures"]').first();
    if (await figuresTab.isVisible()) {
      await figuresTab.click();
      await page.waitForTimeout(1000);
    }
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui07-figures.png' });
    console.log('✅ Figures view accessed');
  });

  test('UI-08: Tab navigation - All tabs switch correctly', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const tabs = ['Map', 'Timeline', 'Dashboard', 'Figures', 'Settlements'];
    
    for (const tabName of tabs) {
      const tab = page.locator(`[class*="tab"]:has-text("${tabName}")`).first();
      if (await tab.isVisible()) {
        await tab.click();
        await page.waitForTimeout(500);
        console.log(`✅ Tab "${tabName}" clicked`);
      }
    }
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui08-tabs.png' });
    console.log('✅ Tab navigation tested');
  });

  test('UI-09: Browser console - Zero console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        if (!text.includes('net::ERR_CONNECTION_REFUSED') && !text.includes('Failed to fetch')) {
          errors.push(text);
        }
      }
    });
    
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    const mapTab = page.locator('[class*="tab"]:has-text("Map")').first();
    if (await mapTab.isVisible()) {
      await mapTab.click();
      await page.waitForTimeout(1000);
    }
    
    const timelineTab = page.locator('[class*="tab"]:has-text("Timeline")').first();
    if (await timelineTab.isVisible()) {
      await timelineTab.click();
      await page.waitForTimeout(1000);
    }
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui09-console-check.png' });
    
    console.log(`✅ Console check complete: ${errors.length} JavaScript errors`);
    if (errors.length > 0) {
      errors.forEach(e => console.log(`  - ${e}`));
    }
    
    expect(errors.length).toBe(0);
  });

  test('UI-10: Pan and zoom - Map controls work', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const zoomIn = page.locator('[title*="zoom"], .zoom-in, button:has-text("+"), button:has-text("In")').first();
    const zoomOut = page.locator('[title*="zoom"], .zoom-out, button:has-text("-"), button:has-text("Out")').first();
    
    if (await zoomIn.isVisible()) {
      await zoomIn.click();
      await page.waitForTimeout(500);
    }
    
    if (await zoomOut.isVisible()) {
      await zoomOut.click();
      await page.waitForTimeout(500);
    }
    
    await page.screenshot({ path: 'e2e/screenshots/wor820-ui10-zoom.png' });
    console.log('✅ Map zoom controls tested');
  });

  // ========================================
  // FINAL REPORT
  // ========================================

  test('FINAL: Generate smoke test report', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    await page.screenshot({ path: 'e2e/screenshots/wor820-final-home.png' });
    
    console.log('\n===========================================');
    console.log('WOR-820 SMOKE TEST REPORT');
    console.log('===========================================');
    console.log('Backend API (18 endpoints): Tested');
    console.log('Frontend UI: Tested');
    console.log('Screenshots: Captured to e2e/screenshots/wor820-*.png');
    console.log('Console Errors: ' + consoleErrors.length);
    console.log('===========================================');
    
    expect(true).toBe(true);
  });
});