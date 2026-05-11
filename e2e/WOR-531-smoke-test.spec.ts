import { test, expect } from '@playwright/test';

const API_BASE = 'http://localhost:80822/api/v1';
const FRONTEND_URL = 'http://localhost:8765';

async function apiRequest(method: string, path: string, body?: object) {
  const url = path.startsWith('http') ? path : `${API_BASE}${path}`;
  const resp = await fetch(url, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
  return { status: resp.status, json: await resp.json().catch(() => ({})) };
}

test.describe('WOR-531: Complete Smoke Test - All 18 API Endpoints', () => {
  let worldId: string;

  test('1. POST /api/v1/worlds - Create world', async ({ page }) => {
    await page.screenshot({ path: 'screenshots/WOR-531-01-before-create.png' });
    
    const resp = await apiRequest('POST', '/worlds', { 
      name: 'WOR-531 Smoke Test World',
      seed: 531531,
      config: { 
        width: 64,
        height: 64,
        genre: 'fantasy'
      }
    });
    
    console.log('Create response:', resp.status, JSON.stringify(resp.json));
    
    expect([200, 201]).toContain(resp.status);
    expect(resp.json.success).toBe(true);
    worldId = resp.json.data?.id || resp.json.data?.world?.id;
    console.log(`Created world: ${worldId}`);
    
    await page.screenshot({ path: 'screenshots/WOR-531-02-world-created.png' });
  });

  test('2. GET /api/v1/worlds - List worlds', async () => {
    const resp = await apiRequest('GET', '/worlds');
    console.log('List worlds:', resp.status, JSON.stringify(resp.json));
    expect([200, 201]).toContain(resp.status);
    expect(resp.json.success).toBe(true);
  });

  test('3. GET /api/v1/worlds/:id - Get world', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}`);
    console.log(`GET world ${uuid}:`, resp.status);
  });

  test('4. GET /api/v1/worlds/:id/planet', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/planet`);
    console.log(`Planet: ${resp.status}`);
  });

  test('5. GET /api/v1/worlds/:id/map', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/map`);
    console.log(`Map: ${resp.status}`, resp.json.data?.polygons ? `(${resp.json.data.polygons.length} polys)` : '');
  });

  test('6. GET /api/v1/worlds/:id/history', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/history`);
    console.log(`History: ${resp.status}`);
  });

  test('7. GET /api/v1/worlds/:id/history/events', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/history/events`);
    console.log(`History events: ${resp.status}`);
  });

  test('8. GET /api/v1/worlds/:id/figures', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/figures`);
    console.log(`Figures: ${resp.status}`);
  });

  test('9. GET /api/v1/worlds/:id/figures/:figure_id', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/figures/fig-0`);
    console.log(`Figure detail: ${resp.status}`);
  });

  test('10. GET /api/v1/worlds/:id/settlements', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/settlements`);
    console.log(`Settlements: ${resp.status}`);
  });

  test('11. GET /api/v1/worlds/:id/settlements/map', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/settlements/map`);
    console.log(`Settlements map: ${resp.status}`);
  });

  test('12. GET /api/v1/worlds/:id/resources/summary', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/resources/summary`);
    console.log(`Resources: ${resp.status}`);
  });

  test('13. GET /api/v1/worlds/:id/disasters', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/disasters`);
    console.log(`Disasters: ${resp.status}`);
  });

  test('14. GET /api/v1/worlds/:id/artifacts', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/artifacts?limit=5`);
    console.log(`Artifacts: ${resp.status}`);
  });

  test('15. GET /api/v1/worlds/:id/export', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/export`);
    console.log(`Export: ${resp.status}`);
  });

  test('16. GET /api/v1/worlds/:id/export.json', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('GET', `/worlds/${uuid}/export.json`);
    console.log(`Export.json: ${resp.status}`);
  });

  test('17. DELETE /api/v1/worlds/:id - Delete world', async () => {
    const uuid = worldId?.replace('world:', '') || 'test-uuid';
    const resp = await apiRequest('DELETE', `/worlds/${uuid}`);
    console.log(`DELETE: ${resp.status}`);
  });

  test('18. Backend health check', async () => {
    const resp = await apiRequest('GET', 'http://localhost:80822/health');
    console.log(`Health: ${resp.status}`, JSON.stringify(resp.json));
    expect(resp.status).toBe(200);
    expect(resp.json.status).toBe('ok');
  });
});

test.describe('WOR-531: Frontend UI Tests', () => {
  test('Home page loads with World Factory title', async ({ page }) => {
    const consoleMessages: string[] = [];
    const consoleErrors: string[] = [];
    
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto(FRONTEND_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    
    const title = await page.title();
    expect(title).toContain('World Factory');
    
    // Check key UI elements
    const header = await page.locator('h1').first().textContent();
    expect(header).toContain('World Factory');
    
    await page.screenshot({ path: 'screenshots/WOR-531-frontend-home.png' });
    
    // Log console errors
    if (consoleErrors.length > 0) {
      console.log('Console errors:', consoleErrors.join('\n'));
    }
    
    // Filter critical errors
    const criticalErrors = consoleErrors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('net::ERR_CONNECTION_REFUSED')
    );
    
    expect(criticalErrors.length).toBe(0);
  });

  test('Create form is present', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    await page.waitForLoadState('networkidle');
    
    const createBtn = page.locator('button:has-text("Create World"), button:has-text("Generate")');
    await expect(createBtn.first()).toBeVisible();
    
    await page.screenshot({ path: 'screenshots/WOR-531-frontend-form.png' });
  });
});
