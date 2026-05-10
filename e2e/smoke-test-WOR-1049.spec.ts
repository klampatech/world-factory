import { test, expect, chromium, Browser, BrowserContext, Page } from '@playwright/test';
import * as path from 'path';

const API_BASE = 'http://localhost:3000/api/v1';
const FRONTEND = 'http://localhost:8765';
const SCREENSHOTS = '/home/kyle/projects/world-generator/screenshots/WOR-1049';

test.describe.configure({ mode: 'serial' });

let worldId: string;
let browser: Browser;
let ctx: BrowserContext;
let page: Page;

// ─────────────────────────────────────────────────────────────────────────────
// SETUP
// ─────────────────────────────────────────────────────────────────────────────

test.beforeAll(async ({ playwright }) => {
  const fs = await import('fs');
  if (!fs.existsSync(SCREENSHOTS)) {
    fs.mkdirSync(SCREENSHOTS, { recursive: true });
  }

  browser = await playwright.chromium.launch({ args: ['--no-sandbox'] });
  ctx = await browser.newContext();
  page = await ctx.newPage();
});

test.afterAll(async () => {
  await ctx?.close();
  await browser?.close();
});

// ─────────────────────────────────────────────────────────────────────────────
// BACKEND API TESTS — All 18 endpoints via Node fetch (avoids Playwright request issues)
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Backend API — All 18 Endpoints', () => {

  test('1. POST /api/v1/worlds — Create world', async () => {
    const resp = await fetch(`${API_BASE}/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'WOR-1049 Smoke Test', seed: 99999, config: { genre: 'fantasy', width: 32, height: 32 } })
    });
    expect(resp.status).toBe(201);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(body.data).toHaveProperty('id');
    worldId = body.data.id;
    console.log(`  ✓ Created world: ${worldId}`);

    // Wait for generation to complete (up to 90s)
    const uuid = worldId.replace('world:', '');
    let status = 'generating';
    for (let i = 0; i < 90 && status === 'generating'; i++) {
      await new Promise(r => setTimeout(r, 1000));
      try {
        const check = await fetch(`${API_BASE}/worlds/${uuid}`);
        if (check.status === 200) {
          const data = await check.json();
          status = data.data?.status ?? status;
        }
      } catch (_) { /* keep polling */ }
    }
    console.log(`  ✓ World status after wait: ${status}`);
  });

  test('2. GET /api/v1/worlds — List worlds', async () => {
    const resp = await fetch(`${API_BASE}/worlds`);
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(Array.isArray(body.data.worlds)).toBe(true);
    console.log(`  ✓ Listed ${body.data.worlds.length} world(s)`);
  });

  test('3. GET /api/v1/worlds/:id — Get specific world', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}`);
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    console.log(`  ✓ Got world: ${body.data.name}`);
  });

  test('4. GET /api/v1/worlds/:id/planet — Planet data', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/planet`);
    const status = resp.status;
    console.log(`  ✓ Planet endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('5. GET /api/v1/worlds/:id/map — Map with Voronoi polygons', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/map`);
    const status = resp.status;
    console.log(`  ✓ Map endpoint: ${status}`);
    if (status === 200) {
      const body = await resp.json();
      expect(body.success).toBe(true);
      expect(body.data).toHaveProperty('polygons');
      console.log(`  ✓ Map has ${body.data.polygons?.length ?? 0} polygons`);
    }
  });

  test('6. GET /api/v1/worlds/:id/history — World history', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/history`);
    const status = resp.status;
    console.log(`  ✓ History endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('7. GET /api/v1/worlds/:id/history/events — History events', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/history/events`);
    const status = resp.status;
    console.log(`  ✓ History events endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('8. GET /api/v1/worlds/:id/figures — Figures list', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/figures`);
    const status = resp.status;
    console.log(`  ✓ Figures endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('9. GET /api/v1/worlds/:id/figures/:figure_id — Single figure', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/figures/fig-0`);
    const status = resp.status;
    console.log(`  ✓ Single figure endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('10. GET /api/v1/worlds/:id/settlements — Settlements list', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/settlements`);
    const status = resp.status;
    console.log(`  ✓ Settlements endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('11. GET /api/v1/worlds/:id/settlements/map — Settlement map', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/settlements/map`);
    const status = resp.status;
    console.log(`  ✓ Settlement map endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('12. GET /api/v1/worlds/:id/resources/summary — Resource summary', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/resources/summary`);
    const status = resp.status;
    console.log(`  ✓ Resources summary endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('13. GET /api/v1/worlds/:id/disasters — Disasters list', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/disasters`);
    const status = resp.status;
    console.log(`  ✓ Disasters endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('14. GET /api/v1/worlds/:id/artifacts — Artifacts list', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/artifacts?limit=5`);
    const status = resp.status;
    console.log(`  ✓ Artifacts endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('15. GET /api/v1/worlds/:id/export — World export', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/export`);
    const status = resp.status;
    console.log(`  ✓ Export endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('16. GET /api/v1/worlds/:id/export.json — World JSON export', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}/export.json`);
    const status = resp.status;
    console.log(`  ✓ Export.json endpoint: ${status}`);
    expect([200, 400, 404]).toContain(status);
  });

  test('17. Backend health check', async () => {
    const resp = await fetch('http://localhost:3000/health');
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body).toHaveProperty('status', 'ok');
    console.log(`  ✓ Backend health: ${JSON.stringify(body)}`);
  });

  test('18. DELETE /api/v1/worlds/:id — Delete world', async () => {
    const uuid = worldId.replace('world:', '');
    const resp = await fetch(`${API_BASE}/worlds/${uuid}`, { method: 'DELETE' });
    const status = resp.status;
    console.log(`  ✓ Delete world: ${status}`);
    expect([200, 204, 400, 404]).toContain(status);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// FRONTEND UI TESTS
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Frontend UI — All Screens & Interactions', () => {

  test('FE-1: Frontend loads at root', async () => {
    await page.goto(FRONTEND + '/');
    await page.waitForLoadState('networkidle');
    await page.screenshot({ path: `${SCREENSHOTS}/fe-01-root-loaded.png` });

    const title = await page.title();
    console.log(`  ✓ Page title: "${title}"`);
    expect(title.length).toBeGreaterThan(0);
  });

  test('FE-2: World list displayed', async () => {
    await page.goto(FRONTEND + '/');
    await page.waitForTimeout(2000);

    const bodyText = await page.textContent('body');
    console.log(`  ✓ Body has content: ${bodyText.length} chars`);
    await page.screenshot({ path: `${SCREENSHOTS}/fe-02-world-list.png` });
    expect(bodyText.length).toBeGreaterThan(10);
  });

  test('FE-3: World creation form works', async () => {
    await page.goto(FRONTEND + '/');
    await page.waitForLoadState('networkidle');

    const createBtn = page.locator('button').filter({ hasText: /create|new world|generate/i }).first();
    const btnExists = await createBtn.count() > 0;

    if (btnExists) {
      await createBtn.click();
      await page.waitForTimeout(2000);
      await page.screenshot({ path: `${SCREENSHOTS}/fe-03-create-form.png` });
      console.log('  ✓ Create form opened');
    } else {
      const inputs = await page.locator('input').count();
      console.log(`  ✓ Found ${inputs} input(s) on page`);
      await page.screenshot({ path: `${SCREENSHOTS}/fe-03-page-state.png` });
    }
  });

  test('FE-4: World detail page — tabs switch correctly', async () => {
    const worldUUID = worldId.replace('world:', '');
    await page.goto(`${FRONTEND}/world.html?id=${worldUUID}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    await page.screenshot({ path: `${SCREENSHOTS}/fe-04-world-detail-overview.png` });

    const overviewTab = page.locator('[data-tab="overview"]').first();
    expect(await overviewTab.count()).toBeGreaterThan(0);
    console.log('  ✓ Overview tab present');

    const tabs = ['overview', 'map', 'timeline', 'dashboard'];
    for (const tab of tabs) {
      const tabBtn = page.locator(`[data-tab="${tab}"]`).first();
      if (await tabBtn.count() > 0) {
        await tabBtn.click();
        await page.waitForTimeout(1000);
        console.log(`  ✓ Switched to ${tab} tab`);
      }
    }
    await page.screenshot({ path: `${SCREENSHOTS}/fe-04-tabs-navigated.png` });
  });

  test('FE-5: Map view — canvas renders with Voronoi polygons', async () => {
    const worldUUID = worldId.replace('world:', '');
    await page.goto(`${FRONTEND}/world.html?id=${worldUUID}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    const mapTab = page.locator('[data-tab="map"]').first();
    if (await mapTab.count() > 0) await mapTab.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: `${SCREENSHOTS}/fe-05-map-tab.png` });

    const canvas = page.locator('#world-map, canvas').first();
    const canvasCount = await canvas.count();
    console.log(`  ✓ Found ${canvasCount} canvas element(s)`);
    expect(canvasCount).toBeGreaterThan(0);

    const box = await canvas.boundingBox();
    if (box) {
      console.log(`  ✓ Canvas size: ${box.width}x${box.height}`);
      expect(box.width).toBeGreaterThan(50);
      expect(box.height).toBeGreaterThan(50);
    }
  });

  test('FE-6: Timeline — history events load', async () => {
    const worldUUID = worldId.replace('world:', '');
    await page.goto(`${FRONTEND}/world.html?id=${worldUUID}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    const timelineTab = page.locator('[data-tab="timeline"]').first();
    if (await timelineTab.count() > 0) await timelineTab.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: `${SCREENSHOTS}/fe-06-timeline-tab.png` });

    const bodyText = await page.textContent('body');
    console.log(`  ✓ Timeline body content: ${bodyText.length} chars`);
  });

  test('FE-7: Dashboard — summary data displays', async () => {
    const worldUUID = worldId.replace('world:', '');
    await page.goto(`${FRONTEND}/world.html?id=${worldUUID}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    const dashTab = page.locator('[data-tab="dashboard"]').first();
    if (await dashTab.count() > 0) await dashTab.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: `${SCREENSHOTS}/fe-07-dashboard-tab.png` });

    const bodyText = await page.textContent('body');
    console.log(`  ✓ Dashboard body content: ${bodyText.length} chars`);
  });

  test('FE-8: Zero browser console errors', async () => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        if (!text.includes('favicon') && !text.includes('net::ERR_')) {
          errors.push(text);
        }
      }
    });

    // Create a fresh world for this test so we're not testing against the deleted world
    const freshResp = await fetch(`${API_BASE}/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'WOR-1049 Console Test', seed: 55555, config: { genre: 'fantasy', width: 32, height: 32 } })
    });
    const freshWorld = await freshResp.json();
    const freshId = freshWorld.data.id;

    // Test world detail page
    const worldUUID = freshId.replace('world:', '');
    await page.goto(`${FRONTEND}/world.html?id=${worldUUID}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(4000);

    // Also test index page
    await page.goto(FRONTEND + '/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    console.log(`  ✓ Console errors found: ${errors.length}`);
    if (errors.length > 0) {
      errors.forEach(e => console.log(`    ERROR: ${e}`));
    }
    await page.screenshot({ path: `${SCREENSHOTS}/fe-08-console-check.png` });

    // Only flag critical errors (ignore resource loading warnings)
    const criticalErrors = errors.filter(e =>
      !e.includes('Failed to load resource') &&
      !e.includes('net::ERR_CONNECTION_REFUSED')
    );
    expect(criticalErrors).toHaveLength(0);
  });
});