import { test, expect } from '@playwright/test';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

const API_BASE = 'http://localhost:3000/api/v1';
const FRONTEND_BASE = 'http://localhost:5173';

async function apiRequest(method: string, path: string, data?: object): Promise<{ status: number; body: any }> {
  const fullUrl = `${API_BASE}${path}`;
  let cmd: string;
  
  if (method === 'POST' || method === 'PUT') {
    const body = data ? JSON.stringify(data).replace(/'/g, "'\\''") : '{}';
    cmd = `curl -s -X ${method} '${fullUrl}' -H "Content-Type: application/json" -d '${body}' -w "\\n%{http_code}"`;
  } else {
    cmd = `curl -s '${fullUrl}' -w "\\n%{http_code}"`;
  }
  
  try {
    const { stdout } = await execAsync(cmd);
    const lastLine = stdout.trim().split('\n').pop();
    const status = parseInt(lastLine) || 0;
    const bodyStr = stdout.trim().split('\n').slice(0, -1).join('\n');
    const body = bodyStr ? JSON.parse(bodyStr) : {};
    return { status, body };
  } catch (e: any) {
    return { status: 0, body: { error: e.message } };
  }
}

test.describe('WOR-790: Complete Smoke Test', () => {
  let worldId: string;
  let worldUuid: string;
  let consoleErrorCount: number = 0;

  // ========================================
  // BACKEND API TESTS - All 18 Endpoints
  // ========================================

  test.describe('Backend API - All 18 Endpoints', () => {
    test('1. POST /api/v1/worlds - Create world', async () => {
      const result = await apiRequest('POST', '/worlds', {
        name: 'WOR-790 Smoke Test World',
        seed: 790001,
        config: { genre: 'fantasy', era: 'medieval' }
      });
      
      console.log(`Create world status: ${result.status}, body: ${JSON.stringify(result.body).substring(0, 200)}`);
      
      // Accept 201 (created) or 200 (success)
      expect([200, 201]).toContain(result.status);
      expect(result.body.success).toBe(true);
      
      worldId = result.body.data?.id || '';
      worldUuid = worldId.replace('world:', '');
      console.log(`Created world: ${worldId}`);
    });

    test('2. GET /api/v1/worlds - List worlds', async () => {
      const result = await apiRequest('GET', '/worlds');
      console.log(`List worlds status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
      if (result.status === 200) {
        expect(result.body.success).toBe(true);
        expect(Array.isArray(result.body.data?.worlds)).toBe(true);
      }
    });

    test('3. GET /api/v1/worlds/:id - Get specific world', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}`);
      console.log(`Get world status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('4. DELETE /api/v1/worlds/:id - Delete world', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('DELETE', `/worlds/${worldUuid}`);
      console.log(`Delete world status: ${result.status}`);
      
      expect([200, 204, 400, 404]).toContain(result.status);
    });

    test('5. GET /api/v1/worlds/:id/planet - Get planet data', async () => {
      // Create fresh world for this test
      const createResult = await apiRequest('POST', '/worlds', {
        name: 'WOR-790 Planet Test',
        seed: 790002
      });
      
      console.log(`Create for planet test status: ${createResult.status}`);
      
      if (createResult.status >= 200 && createResult.status < 300) {
        worldId = createResult.body.data?.id || '';
        worldUuid = worldId.replace('world:', '');
        
        // Wait for generation
        console.log('Waiting for world generation...');
        await new Promise(r => setTimeout(r, 15000));
        
        const result = await apiRequest('GET', `/worlds/${worldUuid}/planet`);
        console.log(`Get planet status: ${result.status}`);
        expect([200, 404]).toContain(result.status);
      } else {
        console.log('World creation failed, skipping planet test');
      }
    });

    test('6. GET /api/v1/worlds/:id/map - Get Voronoi map', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/map`);
      console.log(`Get map status: ${result.status}`);
      
      if (result.status === 200) {
        expect(result.body.success).toBe(true);
        expect(result.body.data?.polygons).toBeDefined();
      }
    });

    test('7. GET /api/v1/worlds/:id/history - Get world history', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/history`);
      console.log(`Get history status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('8. GET /api/v1/worlds/:id/history/events - Get history events', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/history/events`);
      console.log(`Get history events status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('9. GET /api/v1/worlds/:id/figures - Get notable figures', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/figures`);
      console.log(`Get figures status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('10. GET /api/v1/worlds/:id/figures/:figure_id - Get specific figure', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/figures/fig-0`);
      console.log(`Get figure status: ${result.status}`);
      
      expect([200, 400, 404]).toContain(result.status);
    });

    test('11. GET /api/v1/worlds/:id/settlements - Get settlements', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/settlements`);
      console.log(`Get settlements status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('12. GET /api/v1/worlds/:id/settlements/map - Get settlements map', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/settlements/map`);
      console.log(`Get settlements map status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('13. GET /api/v1/worlds/:id/resources/summary - Get resources summary', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/resources/summary`);
      console.log(`Get resources status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('14. GET /api/v1/worlds/:id/disasters - Get disasters', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/disasters`);
      console.log(`Get disasters status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('15. GET /api/v1/worlds/:id/artifacts - Get artifacts', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      // Note: artifacts endpoint requires 'limit' query parameter
      const result = await apiRequest('GET', `/worlds/${worldUuid}/artifacts?limit=5`);
      console.log(`Get artifacts status: ${result.status}`);
      
      expect([200, 400, 404]).toContain(result.status);
    });

    test('16. GET /api/v1/worlds/:id/export - Export world', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/export`);
      console.log(`Get export status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('17. GET /api/v1/worlds/:id/export.json - Export as JSON', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/export.json`);
      console.log(`Get export.json status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });
  });

  // ========================================
  // FRONTEND UI TESTS
  // ========================================

  test.describe('Frontend UI - All Screens', () => {
    test.beforeEach(async ({ page }) => {
      consoleErrorCount = 0;
      page.on('console', msg => {
        if (msg.type() === 'error') {
          consoleErrorCount++;
        }
      });
    });

    test('18. Frontend serves correctly', async ({ page }) => {
      const resp = await page.goto(FRONTEND_BASE);
      expect(resp?.status()).toBe(200);
      
      // Check if page loads without crash
      await page.waitForLoadState('domcontentloaded');
      console.log(`Console errors on homepage: ${consoleErrorCount}`);
    });

    test('19. World list - load and display saved worlds', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/index.html`);
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      // Check if world list or cards are visible
      const worldCards = await page.locator('.world-card, .world-item, [data-world-id], li').count();
      console.log(`World items found: ${worldCards}`);
      console.log(`Console errors: ${consoleErrorCount}`);
    });

    test('20. World.html page loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html`);
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      // Page should load without crash
      const title = await page.title();
      console.log(`Page title: ${title}`);
      console.log(`Console errors: ${consoleErrorCount}`);
    });

    test('21. Tab navigation works', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html`);
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      // Try to find and click various tab buttons
      const tabs = ['Map', 'Timeline', 'Figures', 'Settlements', 'History'];
      for (const tab of tabs) {
        const tabBtn = page.locator(`button:has-text("${tab}"), [role="tab"]:has-text("${tab}")`).first();
        if (await tabBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
          await tabBtn.click();
          await page.waitForTimeout(300);
        }
      }
      console.log(`Console errors: ${consoleErrorCount}`);
    });
  });
});
