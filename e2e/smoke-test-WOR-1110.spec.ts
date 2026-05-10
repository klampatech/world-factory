import { test, expect, request as pwRequest, APIRequestContext } from '@playwright/test';

const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';

/**
 * WOR-1110: Smoke Test - Verify application health after latest formatting changes
 * Tests core endpoints and UI load after WOR-1109 formatting and CI changes
 */
test.describe('WOR-1110: Smoke Test', () => {
  let worldId: string;
  let worldName: string;
  let apiContext: APIRequestContext;

  test.beforeAll(async () => {
    apiContext = await pwRequest.newContext();
  });

  test.afterAll(async () => {
    await apiContext.dispose();
  });

  // ─── API TESTS ──────────────────────────────────────────────────────────────

  test('API-01: POST /api/v1/worlds - Create new world', async ({ page }) => {
    worldName = `WOR-1110-Smoke-${Date.now()}`;
    const resp = await apiContext.post(`${API_BASE}/worlds`, {
      data: {
        name: worldName,
        seed: 11100111,
        config: {
          genre: 'fantasy',
          era: 'medieval',
          width: 32,
          height: 32,
        }
      }
    });
    expect(resp.status()).toBe(201);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(body.data.id).toBeDefined();
    worldId = body.data.id;
    console.log(`Created world: ${worldId} — ${worldName}`);
    
    await page.goto(`${FRONTEND_BASE}/`);
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'screenshots/WOR-1110-01-frontend-load.png' });
  });

  test('API-02: GET /api/v1/worlds - List worlds', async () => {
    const resp = await apiContext.get(`${API_BASE}/worlds`);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(Array.isArray(body.data.worlds)).toBe(true);
    console.log(`World list: ${body.data.totalWorlds} total`);
  });

  test('API-03: GET /api/v1/worlds/:id - Get world details', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}`);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(body.data.id).toBeDefined();
  });

  test('API-04: GET /api/v1/worlds/:id/planet - Get planet data', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/planet`);
    const status = resp.status();
    console.log(`  planet endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('API-05: GET /api/v1/worlds/:id/map - Get map data', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/map`);
    const status = resp.status();
    console.log(`  map endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('API-06: GET /api/v1/worlds/:id/figures - Get figures list', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/figures`);
    const status = resp.status();
    console.log(`  figures endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('API-07: GET /api/v1/worlds/:id/settlements - Get settlements', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/settlements`);
    const status = resp.status();
    console.log(`  settlements endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('API-08: GET /api/v1/worlds/:id/history/events - Get history events', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/history/events`);
    const status = resp.status();
    console.log(`  history/events endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('API-09: GET /api/v1/worlds/:id/resources/summary - Get resources summary', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/resources/summary`);
    const status = resp.status();
    console.log(`  resources/summary endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('API-10: GET /api/v1/worlds/:id/export - Get export', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.get(`${API_BASE}/worlds/${uuid}/export`);
    const status = resp.status();
    console.log(`  export endpoint: HTTP ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  // ─── FRONTEND UI TESTS ─────────────────────────────────────────────────────

  test('UI-01: Frontend loads without console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(`${FRONTEND_BASE}/`);
    await page.waitForTimeout(2000);
    
    console.log(`Console errors on index: ${errors.length}`);
    if (errors.length > 0) {
      console.log('Errors:', errors.join('\n'));
    }
    
    await page.screenshot({ path: 'screenshots/WOR-1110-02-index-page.png' });
    
    const jsErrors = errors.filter(e => !e.includes('favicon') && !e.includes('net::'));
    expect(jsErrors).toHaveLength(0);
  });

  test('UI-02: World detail page loads without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    const uuid = worldId.replace('world:', '');
    await page.goto(`${FRONTEND_BASE}/world.html?id=${uuid}`);
    await page.waitForTimeout(3000);
    await page.screenshot({ path: 'screenshots/WOR-1110-03-world-detail.png' });

    const jsErrors = errors.filter(e => !e.includes('favicon') && !e.includes('net::'));
    expect(jsErrors).toHaveLength(0);
  });

  test('UI-03: Map view renders without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    const uuid = worldId.replace('world:', '');
    await page.goto(`${FRONTEND_BASE}/world.html?id=${uuid}`);
    await page.waitForTimeout(4000);
    await page.screenshot({ path: 'screenshots/WOR-1110-04-map-view.png' });

    const jsErrors = errors.filter(e => !e.includes('favicon') && !e.includes('net::'));
    expect(jsErrors).toHaveLength(0);
  });

  test('UI-04: Tab navigation works without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    const uuid = worldId.replace('world:', '');
    await page.goto(`${FRONTEND_BASE}/world.html?id=${uuid}`);
    await page.waitForTimeout(3000);

    const tabs = page.locator('[role="tab"], .tab, button:has-text("Map"), button:has-text("History"), button:has-text("Figures"), button:has-text("Settlements")');
    const tabCount = await tabs.count();

    for (let i = 0; i < Math.min(tabCount, 5); i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(1000);
    }

    await page.screenshot({ path: 'screenshots/WOR-1110-05-tab-nav.png' });

    const jsErrors = errors.filter(e => !e.includes('favicon') && !e.includes('net::'));
    console.log(`JS errors during tab nav: ${jsErrors.length}`);
    expect(jsErrors).toHaveLength(0);
  });

  test('UI-05: Timeline renders without errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    const uuid = worldId.replace('world:', '');
    await page.goto(`${FRONTEND_BASE}/world.html?id=${uuid}`);
    await page.waitForTimeout(3000);

    const historyLink = page.locator('a:has-text("History"), a:has-text("Timeline"), button:has-text("History")');
    if (await historyLink.count() > 0) {
      await historyLink.first().click();
      await page.waitForTimeout(2000);
    }

    await page.screenshot({ path: 'screenshots/WOR-1110-06-timeline.png' });

    const jsErrors = errors.filter(e => !e.includes('favicon') && !e.includes('net::'));
    expect(jsErrors).toHaveLength(0);
  });

  // ─── CLEANUP ───────────────────────────────────────────────────────────────

  test('Cleanup: DELETE /api/v1/worlds/:id - Delete test world', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await apiContext.delete(`${API_BASE}/worlds/${uuid}`);
    console.log(`Deleted ${worldId}: HTTP ${resp.status()}`);
  });
});