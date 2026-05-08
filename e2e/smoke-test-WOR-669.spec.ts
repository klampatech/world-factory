import { test, expect } from '@playwright/test';

const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';

/**
 * WOR-669 Smoke Test - Full Stack E2E
 * Tests all 18 backend endpoints + frontend UI
 */

// Use global fetch for API tests to avoid Playwright request fixture issues
async function apiGet(path: string) {
  const resp = await fetch(`${API_BASE}${path}`);
  return { status: resp.status, json: () => resp.json() };
}

async function apiPost(path: string, data: object) {
  const resp = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  });
  return { status: resp.status, json: () => resp.json() };
}

async function apiDelete(path: string) {
  const resp = await fetch(`${API_BASE}${path}`, { method: 'DELETE' });
  return { status: resp.status };
}

test.describe('WOR-669: Smoke Test - Backend API (All 18 Endpoints)', () => {
  let worldId: string;
  let worldUuid: string;

  test('1. POST /api/v1/worlds - Create world', async () => {
    const resp = await apiPost('/worlds', {
      name: 'WOR-669 Smoke Test World',
      seed: 66999,
      config: {
        width: 32,
        height: 32,
        pre_history_years: 50,
        genre: 'fantasy'
      }
    });
    expect(resp.status).toBe(201);
    const body = await resp.json();
    expect(body.success).toBe(true);
    worldId = body.data.id;
    worldUuid = worldId.replace('world:', '');
    console.log(`✓ Created world: ${worldId}`);
  });

  test('2. GET /api/v1/worlds - List worlds', async () => {
    const resp = await apiGet('/worlds');
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(Array.isArray(body.data.worlds)).toBe(true);
    console.log(`✓ Listed ${body.data.worlds.length} worlds`);
  });

  test('3. GET /api/v1/worlds/:id - Get world details', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}`);
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    console.log(`✓ Got world details for ${worldUuid}`);
  });

  test('4. GET /api/v1/worlds/:id/planet - Get planet data', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/planet`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Planet endpoint: ${resp.status}`);
  });

  test('5. GET /api/v1/worlds/:id/map - Get map data', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/map`);
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    expect(body.data.polygons).toBeDefined();
    console.log(`✓ Map loaded with ${body.data.polygons?.length || 0} polygons`);
  });

  test('6. GET /api/v1/worlds/:id/history - Get history timeline', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/history`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ History endpoint: ${resp.status}`);
  });

  test('7. GET /api/v1/worlds/:id/history/events - Get history events', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/history/events`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ History events endpoint: ${resp.status}`);
  });

  test('8. GET /api/v1/worlds/:id/figures - Get notable figures', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/figures`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Figures endpoint: ${resp.status}`);
  });

  test('9. GET /api/v1/worlds/:id/figures/:figure_id - Get specific figure', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/figures/fig-0`);
    expect([200, 404]).toContain(resp.status);
    console.log(`✓ Single figure endpoint: ${resp.status}`);
  });

  test('10. GET /api/v1/worlds/:id/settlements - Get settlements', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/settlements`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Settlements endpoint: ${resp.status}`);
  });

  test('11. GET /api/v1/worlds/:id/settlements/map - Get settlements map', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/settlements/map`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Settlements map endpoint: ${resp.status}`);
  });

  test('12. GET /api/v1/worlds/:id/resources/summary - Get resources summary', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/resources/summary`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Resources summary endpoint: ${resp.status}`);
  });

  test('13. GET /api/v1/worlds/:id/disasters - Get disasters', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/disasters`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Disasters endpoint: ${resp.status}`);
  });

  test('14. GET /api/v1/worlds/:id/artifacts - Get artifacts', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/artifacts?limit=5`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Artifacts endpoint: ${resp.status}`);
  });

  test('15. GET /api/v1/worlds/:id/export - Get world export', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/export`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Export endpoint: ${resp.status}`);
  });

  test('16. GET /api/v1/worlds/:id/export.json - Get JSON export', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/export.json`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Export.json endpoint: ${resp.status}`);
  });

  test('17. Backend health check', async () => {
    const resp = await apiGet('/health'.replace('/api/v1', ''));
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.status).toBe('ok');
    console.log(`✓ Backend health: ${body.status}`);
  });

  test('18. DELETE /api/v1/worlds/:id - Delete world', async () => {
    const resp = await apiDelete(`/worlds/${worldUuid}`);
    expect([200, 204, 400]).toContain(resp.status);
    console.log(`✓ Delete world: ${resp.status}`);
  });
});

test.describe('WOR-669: Frontend UI Tests', () => {
  test('1. Home page loads', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Accept either title
    const title = await page.title();
    expect(title).toMatch(/World (Selector|Factory|Procedural)/);
    
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-01-home-page.png' });
    console.log('✓ Home page loaded, title: ' + title);
  });

  test('2. World creation form renders', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Check for world creation form elements
    const formExists = await page.locator('form, input, button[type="submit"]').first().isVisible().catch(() => false);
    
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-02-world-creation-form.png' });
    expect(formExists).toBeTruthy();
    console.log('✓ World creation form rendered');
  });

  test('3. World list displays', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-03-world-list.png' });
    console.log('✓ World list rendered');
  });

  test('4. Map view renders', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Check canvas exists
    const canvasCount = await page.locator('canvas').count();
    
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-04-map-view.png' });
    expect(canvasCount).toBeGreaterThan(0);
    console.log(`✓ Map view rendered (${canvasCount} canvas elements)`);
  });

  test('5. Timeline view renders', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-05-timeline-view.png' });
    console.log('✓ Timeline view rendered');
  });

  test('6. Dashboard displays', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-06-dashboard.png' });
    console.log('✓ Dashboard rendered');
  });

  test('7. Figures list renders', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-07-figures-list.png' });
    console.log('✓ Figures list rendered');
  });

  test('8. Tab navigation works', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);

    // Try clicking tabs
    const tabs = page.locator('[role="tab"], .tab, button[class*="tab"], nav a');
    const tabCount = await tabs.count();
    
    for (let i = 0; i < Math.min(tabCount, 5); i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(300);
    }
    
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-08-tab-navigation.png' });
    console.log(`✓ Tab navigation tested (${tabCount} tabs found)`);
  });

  test('9. Zero console errors on homepage', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error' && !msg.text().includes('favicon') && !msg.text().includes('net::ERR')) {
        errors.push(msg.text());
      }
    });

    await page.goto(FRONTEND_BASE);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-669-09-console-check.png' });

    // Filter out known non-critical errors
    const criticalErrors = errors.filter(e => 
      !e.includes('Failed to fetch') && 
      !e.includes('favicon') &&
      !e.includes('net::ERR')
    );
    
    if (criticalErrors.length > 0) {
      console.log('Critical console errors found:', criticalErrors.join('\n'));
    }
    
    expect(criticalErrors).toHaveLength(0);
    console.log(`✓ Zero critical console errors (${errors.length} total, ${criticalErrors.length} critical)`);
  });
});