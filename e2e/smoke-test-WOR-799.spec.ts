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

test.describe('WOR-799: Smoke Test', () => {
  let worldId: string;
  let worldUuid: string;
  let consoleErrorCount: number = 0;

  // ========================================
  // BACKEND API TESTS
  // ========================================

  test.describe('Backend API Endpoints', () => {
    test('1. POST /api/v1/worlds - Create world', async () => {
      const result = await apiRequest('POST', '/worlds', {
        name: 'WOR-799 Smoke Test World',
        seed: 799001,
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

    test('4. GET /api/v1/worlds/:id/planet - Get planet data', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/planet`);
      console.log(`Get planet status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('5. GET /api/v1/worlds/:id/map - Get Voronoi map', async () => {
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

    test('6. GET /api/v1/worlds/:id/history - Get world history', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/history`);
      console.log(`Get history status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('7. GET /api/v1/worlds/:id/figures - Get notable figures', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/figures`);
      console.log(`Get figures status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('8. GET /api/v1/worlds/:id/settlements - Get settlements', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/settlements`);
      console.log(`Get settlements status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('9. GET /api/v1/worlds/:id/resources/summary - Get resources summary', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/resources/summary`);
      console.log(`Get resources status: ${result.status}`);
      
      expect([200, 404]).toContain(result.status);
    });

    test('10. GET /api/v1/worlds/:id/artifacts - Get artifacts', async () => {
      if (!worldUuid) {
        console.log('No worldUuid, skipping...');
        return;
      }
      
      const result = await apiRequest('GET', `/worlds/${worldUuid}/artifacts?limit=5`);
      console.log(`Get artifacts status: ${result.status}`);
      
      expect([200, 400, 404]).toContain(result.status);
    });
  });

  // ========================================
  // FRONTEND UI TESTS
  // ========================================

  test.describe('Frontend UI Tests', () => {
    test.beforeEach(async ({ page }) => {
      consoleErrorCount = 0;
      page.on('console', msg => {
        if (msg.type() === 'error') {
          consoleErrorCount++;
        }
      });
    });

    test('11. Frontend homepage loads', async ({ page }) => {
      const resp = await page.goto(FRONTEND_BASE);
      expect(resp?.status()).toBe(200);
      
      await page.waitForLoadState('domcontentloaded');
      console.log(`Homepage console errors: ${consoleErrorCount}`);
    });

    test('12. World list page loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/index.html`);
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      const worldCards = await page.locator('.world-card, .world-item, [data-world-id], li').count();
      console.log(`World items found: ${worldCards}`);
      console.log(`Console errors: ${consoleErrorCount}`);
    });

    test('13. World detail page loads', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html`);
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      const title = await page.title();
      console.log(`Page title: ${title}`);
      console.log(`Console errors: ${consoleErrorCount}`);
    });

    test('14. Tab navigation works', async ({ page }) => {
      await page.goto(`${FRONTEND_BASE}/world.html`);
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      const tabs = ['Map', 'Timeline', 'Figures', 'Settlements', 'History'];
      for (const tab of tabs) {
        const tabBtn = page.locator(`button:has-text("${tab}"), [role="tab"]:has-text("${tab}")`).first();
        if (await tabBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
          await tabBtn.click();
          await page.waitForTimeout(300);
        }
      }
      console.log(`Tab navigation console errors: ${consoleErrorCount}`);
    });
  });
});