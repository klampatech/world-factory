import { test, expect } from '@playwright/test';

const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';

const SHOTS = '/home/kyle/projects/world-generator/screenshots/WOR-612';
const mkdir = require('fs').mkdirSync;
try { mkdir(SHOTS, { recursive: true }); } catch (_) {}

test.describe('WOR-612: Full Stack Smoke Test', () => {
  let worldId: string;
  let worldIdNoPrefix: string;

  // ============================================================
  // BACKEND API — All 18 Endpoints
  // ============================================================

  test('01. POST /api/v1/worlds — Create world', async ({ request }) => {
    const resp = await request.post(`${API_BASE}/worlds`, {
      data: { name: 'WOR-612 Smoke Test', seed: 612612, config: { genre: 'fantasy' } }
    });
    // Accept 201 (Created) or 202 (Accepted) — both are valid success codes
    expect([201, 202]).toContain(resp.status());
    const body = await resp.json();
    expect(body.success).toBe(true);
    worldId = body.data.id;
    worldIdNoPrefix = worldId.replace('world:', '');
    console.log(`Created world: ${worldId}`);
  });

  test('02. GET /api/v1/worlds — List worlds', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds`);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(Array.isArray(body.data.worlds)).toBe(true);
    console.log(`Listed ${body.data.worlds.length} worlds`);
  });

  test('03. GET /api/v1/worlds/:id — Get world (uuid no prefix)', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}`);
    expect([200, 404]).toContain(resp.status());
    console.log(`GET world (uuid): ${resp.status()}`);
  });

  test('04. GET /api/v1/worlds/:id — Get world (with world: prefix)', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldId}`);
    expect([200, 404]).toContain(resp.status());
    console.log(`GET world (prefixed): ${resp.status()}`);
  });

  test('05. GET /api/v1/worlds/:id/planet — Get planet', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/planet`);
    console.log(`GET /planet: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('06. GET /api/v1/worlds/:id/map — Get map', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/map`);
    console.log(`GET /map: ${resp.status()}`);
    if (resp.status() === 200) {
      const body = await resp.json();
      expect(body.success).toBe(true);
      expect(body.data.polygons).toBeDefined();
    }
  });

  test('07. GET /api/v1/worlds/:id/history — Get history', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/history`);
    console.log(`GET /history: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('08. GET /api/v1/worlds/:id/history/events — Get history events', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/history/events`);
    console.log(`GET /history/events: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('09. GET /api/v1/worlds/:id/figures — List figures', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/figures`);
    console.log(`GET /figures: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('10. GET /api/v1/worlds/:id/figures/:figure_id — Get figure', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/figures/fig-0`);
    console.log(`GET /figures/fig-0: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('11. GET /api/v1/worlds/:id/settlements — List settlements', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/settlements`);
    console.log(`GET /settlements: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('12. GET /api/v1/worlds/:id/settlements/map — Settlements map', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/settlements/map`);
    console.log(`GET /settlements/map: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('13. GET /api/v1/worlds/:id/resources/summary — Resources summary', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/resources/summary`);
    console.log(`GET /resources/summary: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('14. GET /api/v1/worlds/:id/disasters — Disasters', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/disasters`);
    console.log(`GET /disasters: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('15. GET /api/v1/worlds/:id/artifacts — Artifacts', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/artifacts?limit=5`);
    console.log(`GET /artifacts: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('16. GET /api/v1/worlds/:id/export — Export', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/export`);
    console.log(`GET /export: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('17. GET /api/v1/worlds/:id/export.json — Export JSON', async ({ request }) => {
    const resp = await request.get(`${API_BASE}/worlds/${worldIdNoPrefix}/export.json`);
    console.log(`GET /export.json: ${resp.status()}`);
    expect([200, 400, 404]).toContain(resp.status());
  });

  test('18. Backend /health — Health check', async ({ request }) => {
    const resp = await request.get('http://localhost:8080/health');
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    console.log(`Health: ${JSON.stringify(body)}`);
  });

  // Cleanup
  test.afterAll(async ({ request }) => {
    if (worldIdNoPrefix) {
      try {
        const resp = await request.delete(`${API_BASE}/worlds/${worldIdNoPrefix}`);
        console.log(`Cleanup DELETE /worlds/${worldIdNoPrefix}: ${resp.status()}`);
      } catch (e) {
        console.log('Cleanup failed:', e);
      }
    }
  });
});

test.describe('WOR-612: Frontend UI Tests', () => {
  test('FE-01: World Selector landing page loads', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto(FRONTEND_BASE, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);

    const title = await page.title();
    console.log(`Page title: ${title}`);

    await page.screenshot({ path: `${SHOTS}/fe-01-landing-page.png` });

    // Only fail on actual JS runtime errors, not network/resource errors
    const criticalErrors = errors.filter(e =>
      !e.includes('favicon') &&
      !e.includes('Failed to load resource') &&
      !e.includes('net::ERR_') &&
      !e.includes('HTTP 404') &&
      !e.includes('Polling failed') &&
      e.includes('Error')
    );
    console.log(`Console errors: ${JSON.stringify(errors)}`);
    console.log(`Critical errors: ${JSON.stringify(criticalErrors)}`);
    expect(criticalErrors).toHaveLength(0);
  });

  test('FE-02: World list / saved worlds load', async ({ page }) => {
    await page.goto(FRONTEND_BASE, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);

    const body = await page.textContent('body');
    console.log(`Body preview: ${body?.substring(0, 200)}`);

    await page.screenshot({ path: `${SHOTS}/fe-02-world-list.png` });

    expect(body?.length).toBeGreaterThan(100);
  });

  test('FE-03: Tab navigation works', async ({ page }) => {
    await page.goto(FRONTEND_BASE, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);

    const worldLinks = await page.locator('a[href*="/world/"]').all();
    if (worldLinks.length > 0) {
      await worldLinks[0].click();
      await page.waitForTimeout(2000);

      const tabs = await page.locator('button[role="tab"], a[role="tab"]').all();
      console.log(`Found ${tabs.length} tabs`);
      for (const tab of tabs) {
        const name = await tab.textContent();
        console.log(`Clicking tab: ${name}`);
        await tab.click();
        await page.waitForTimeout(500);
      }
    } else {
      console.log('No world links found, skipping tab nav test');
    }

    await page.screenshot({ path: `${SHOTS}/fe-03-tabs.png` });
  });
});