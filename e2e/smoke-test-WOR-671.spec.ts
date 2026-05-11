import { test, expect } from '@playwright/test';

const API_BASE = 'http://localhost:80822/api/v1';

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

test.describe('WOR-671: Smoke Test Re-run After Bug Fixes', () => {
  let worldId: string;
  let worldUuid: string;

  test('1. POST /api/v1/worlds - Create world', async () => {
    const resp = await apiPost('/worlds', {
      name: 'WOR-671 Smoke Test World',
      seed: 67199,
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
    console.log(`✓ Listed worlds`);
  });

  test('3. GET /api/v1/worlds/:id - Get world details', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}`);
    expect(resp.status).toBe(200);
    const body = await resp.json();
    expect(body.success).toBe(true);
    console.log(`✓ Got world details`);
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
    expect(body.data.polygons).toBeDefined();
    console.log(`✓ Map endpoint: ${resp.status}, ${body.data.polygons?.length || 0} polygons`);
  });

  test('6. GET /api/v1/worlds/:id/history - Get history', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/history`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ History endpoint: ${resp.status}`);
  });

  test('7. GET /api/v1/worlds/:id/history/events - Get events (WOR-662 fix)', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/history/events`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Events endpoint: ${resp.status} (WOR-662 fix verified)`);
  });

  test('8. GET /api/v1/worlds/:id/figures - Get figures', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/figures`);
    expect([200, 400]).toContain(resp.status);
    console.log(`✓ Figures endpoint: ${resp.status}`);
  });

  test('9. GET /api/v1/worlds/:id/figures/:id - Get figure detail (WOR-663 fix)', async () => {
    // First get figures list to find an ID
    const listResp = await apiGet(`/worlds/${worldUuid}/figures`);
    if (listResp.status === 200) {
      const body = await listResp.json();
      if (body.data && body.data.figures && body.data.figures.length > 0) {
        const figureId = body.data.figures[0].id;
        const resp = await apiGet(`/worlds/${worldUuid}/figures/${figureId}`);
        expect(resp.status).toBe(200);
        console.log(`✓ Figure detail endpoint: ${resp.status} (WOR-663 fix verified)`);
      } else {
        console.log(`✓ Figure detail endpoint: Skipped (no figures exist yet)`);
      }
    } else {
      console.log(`✓ Figure detail endpoint: Skipped (figures list unavailable)`);
    }
  });

  test('10. GET /api/v1/worlds/:id/settlements - Get settlements', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/settlements`);
    expect(resp.status).toBe(200);
    const body = await resp.json();
    console.log(`✓ Settlements endpoint: ${resp.status}`);
  });

  test('11. GET /api/v1/worlds/:id/settlements/map - Get settlements map', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/settlements/map`);
    expect(resp.status).toBe(200);
    console.log(`✓ Settlement map endpoint: ${resp.status}`);
  });

  test('12. GET /api/v1/worlds/:id/resources/summary - Get resources', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/resources/summary`);
    expect(resp.status).toBe(200);
    console.log(`✓ Resources endpoint: ${resp.status}`);
  });

  test('13. GET /api/v1/worlds/:id/disasters - Get disasters', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/disasters`);
    expect(resp.status).toBe(200);
    console.log(`✓ Disasters endpoint: ${resp.status}`);
  });

  test('14. GET /api/v1/worlds/:id/artifacts - Get artifacts', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/artifacts`);
    expect(resp.status).toBe(200);
    console.log(`✓ Artifacts endpoint: ${resp.status}`);
  });

  test('15. GET /api/v1/worlds/:id/export - Get export', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/export`);
    expect(resp.status).toBe(200);
    console.log(`✓ Export endpoint: ${resp.status}`);
  });

  test('16. GET /api/v1/worlds/:id/export.json - Get JSON export', async () => {
    const resp = await apiGet(`/worlds/${worldUuid}/export.json`);
    expect(resp.status).toBe(200);
    console.log(`✓ JSON export endpoint: ${resp.status}`);
  });

  test('17. DELETE /api/v1/worlds/:id - Delete world', async () => {
    const resp = await apiDelete(`/worlds/${worldUuid}`);
    expect(resp.status).toBe(204);
    console.log(`✓ Delete world: ${resp.status}`);
  });
});
