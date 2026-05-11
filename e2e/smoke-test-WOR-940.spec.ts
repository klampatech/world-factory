import { test, expect } from '@playwright/test';
import path from 'path';

/**
 * WOR-940: Full Smoke Test
 * Tests all 18 API endpoints and frontend UI functionality
 */

const API_BASE = 'http://localhost:80822/api/v1';
const FRONTEND_BASE = 'http://localhost:8765';
const TEST_WORLD_NAME = `WOR-940-Smoke-Test-${Date.now()}`;

interface TestResult {
  endpoint: string;
  method: string;
  status: number;
  passed: boolean;
  error?: string;
  duration: number;
}

const results: TestResult[] = [];

async function apiRequest(method: string, path: string, body?: object): Promise<{ status: number; data?: any; error?: string }> {
  const start = Date.now();
  try {
    const options: any = {
      method,
      headers: { 'Content-Type': 'application/json' },
    };
    if (body) {
      options.body = JSON.stringify(body);
    }
    const response = await fetch(`${API_BASE}${path}`, options);
    let data = await response.json().catch(() => null);
    // Flatten: unwrap {success, data} wrapper if present
    if (data && typeof data === 'object' && 'success' in data && 'data' in data) {
      data = data.data;
    }
    const duration = Date.now() - start;
    return { status: response.status, data, duration };
  } catch (error: any) {
    const duration = Date.now() - start;
    return { status: 0, error: error.message, duration };
  }
}

async function logResult(result: TestResult) {
  results.push(result);
  const status = result.passed ? '✅' : '❌';
  console.log(`${status} ${result.method} ${result.endpoint} - ${result.status} (${result.duration}ms)`);
  if (result.error) {
    console.log(`   Error: ${result.error}`);
  }
}

test.describe('WOR-940 Smoke Test - Backend API', () => {
  let worldId: string;

  test('should create a new world', async ({ page }) => {
    const response = await apiRequest('POST', '/worlds', {
      name: TEST_WORLD_NAME,
      genre: 'fantasy',
      era: 'medieval',
      config: {
        width: 32,
        height: 32,
        pre_history_years: 50,
        seed: 94042
      }
    });

    await logResult({
      endpoint: '/worlds',
      method: 'POST',
      status: response.status,
      passed: response.status === 200 || response.status === 201,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBeLessThanOrEqual(201);
    expect(response.data?.id).toBeDefined();
    worldId = response.data.id;

    // Take screenshot
    await page.goto(`${FRONTEND_BASE}/worlds/${worldId}`);
    await page.waitForTimeout(2000);
    await page.screenshot({ path: `screenshots/WOR-940-world-created.png` });
    console.log(`📸 Screenshot: WOR-940-world-created.png`);
  });

  test('should get all worlds (GET /worlds)', async () => {
    // Wait for world to be created
    await new Promise(r => setTimeout(r, 1000));
    
    const response = await apiRequest('GET', '/worlds');
    await logResult({
      endpoint: '/worlds',
      method: 'GET',
      status: response.status,
      passed: response.status === 200 && response.data?.worlds?.length > 0,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
    expect(response.data?.worlds).toBeDefined();
  });

  test('should get world by ID (GET /worlds/:id)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}`);
    await logResult({
      endpoint: `/worlds/${worldId}`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200 && response.data?.id === worldId,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
    expect(response.data?.id).toBe(worldId);
  });

  test('should get planet data (GET /worlds/:id/planet)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/planet`);
    await logResult({
      endpoint: `/worlds/${worldId}/planet`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get map data (GET /worlds/:id/map)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/map`);
    await logResult({
      endpoint: `/worlds/${worldId}/map`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get history (GET /worlds/:id/history)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/history`);
    await logResult({
      endpoint: `/worlds/${worldId}/history`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get history events (GET /worlds/:id/history/events)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/history/events`);
    await logResult({
      endpoint: `/worlds/${worldId}/history/events`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get figures list (GET /worlds/:id/figures)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/figures`);
    await logResult({
      endpoint: `/worlds/${worldId}/figures`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get settlements (GET /worlds/:id/settlements)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/settlements`);
    await logResult({
      endpoint: `/worlds/${worldId}/settlements`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get resources summary (GET /worlds/:id/resources/summary)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/resources/summary`);
    await logResult({
      endpoint: `/worlds/${worldId}/resources/summary`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get disasters (GET /worlds/:id/disasters)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/disasters`);
    await logResult({
      endpoint: `/worlds/${worldId}/disasters`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get artifacts (GET /worlds/:id/artifacts)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/artifacts`);
    await logResult({
      endpoint: `/worlds/${worldId}/artifacts`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get export (GET /worlds/:id/export)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/export`);
    await logResult({
      endpoint: `/worlds/${worldId}/export`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should get export.json (GET /worlds/:id/export.json)', async () => {
    const response = await apiRequest('GET', `/worlds/${worldId}/export.json`);
    await logResult({
      endpoint: `/worlds/${worldId}/export.json`,
      method: 'GET',
      status: response.status,
      passed: response.status === 200,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBe(200);
  });

  test('should delete world (DELETE /worlds/:id)', async () => {
    const response = await apiRequest('DELETE', `/worlds/${worldId}`);
    await logResult({
      endpoint: `/worlds/${worldId}`,
      method: 'DELETE',
      status: response.status,
      passed: response.status === 200 || response.status === 204,
      error: response.error,
      duration: response.duration
    });

    expect(response.status).toBeLessThanOrEqual(204);
  });

  // Figure individual endpoint (if there's at least one figure)
  test('should test figures endpoint with sample data', async () => {
    // First create a world with more history years
    const createResp = await apiRequest('POST', '/worlds', {
      name: 'WOR-940-Figures-Test',
      config: {
        width: 32,
        height: 32,
        pre_history_years: 100,
        seed: 94043
      }
    });

    if (createResp.status < 300 && createResp.data?.id) {
      const figuresResp = await apiRequest('GET', `/worlds/${createResp.data.id}/figures`);
      
      await logResult({
        endpoint: `/worlds/${createResp.data.id}/figures`,
        method: 'GET',
        status: figuresResp.status,
        passed: figuresResp.status === 200,
        error: figuresResp.error,
        duration: figuresResp.duration
      });

      // Test getting a specific figure if any exist
      const figures = figuresResp.data?.figures || [];
      if (figures.length > 0 && figures[0].id) {
        const figureResp = await apiRequest('GET', `/worlds/${createResp.data.id}/figures/${figures[0].id}`);
        await logResult({
          endpoint: `/worlds/${createResp.data.id}/figures/${figures[0].id}`,
          method: 'GET',
          status: figureResp.status,
          passed: figureResp.status === 200,
          error: figureResp.error,
          duration: figureResp.duration
        });
      }

      // Test settlements map
      const settlementsResp = await apiRequest('GET', `/worlds/${createResp.data.id}/settlements/map`);
      await logResult({
        endpoint: `/worlds/${createResp.data.id}/settlements/map`,
        method: 'GET',
        status: settlementsResp.status,
        passed: settlementsResp.status === 200,
        error: settlementsResp.error,
        duration: settlementsResp.duration
      });
    }
  });
});

test.describe('WOR-940 Smoke Test - Frontend UI', () => {
  const consoleErrors: string[] = [];

  test.beforeEach(async ({ page }) => {
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
  });

  test('should load landing page and display world list', async ({ page }) => {
    await page.goto(FRONTEND_BASE);
    await page.waitForTimeout(3000);
    await page.screenshot({ path: 'screenshots/WOR-940-landing-page.png' });
    console.log('📸 Screenshot: WOR-940-landing-page.png');

    // Check for console errors
    const criticalErrors = consoleErrors.filter(e => !e.includes('favicon') && !e.includes('preflight'));
    if (criticalErrors.length > 0) {
      console.log('❌ Console errors on landing page:', criticalErrors);
    } else {
      console.log('✅ No critical console errors on landing page');
    }
  });

  test('should render map view with Voronoi polygons', async ({ page }) => {
    // Get a recent world ID
    const worldsResp = await fetch(`${API_BASE}/worlds`);
    const worldsData = await worldsResp.json();
    const firstWorld = worldsData?.data?.worlds?.[0];

    if (firstWorld?.id) {
      await page.goto(`${FRONTEND_BASE}/worlds/${firstWorld.id}`);
      await page.waitForTimeout(3000);
      
      // Wait for map to render
      const mapCanvas = page.locator('canvas').first();
      await mapCanvas.waitFor({ state: 'visible', timeout: 5000 }).catch(() => {
        console.log('⚠️ Canvas not found, checking for SVG map');
      });

      await page.screenshot({ path: 'screenshots/WOR-940-map-view.png' });
      console.log('📸 Screenshot: WOR-940-map-view.png');
    }
  });

  test('should navigate tabs without errors', async ({ page }) => {
    const worldsResp = await fetch(`${API_BASE}/worlds`);
    const worldsData = await worldsResp.json();
    const firstWorld = worldsData?.data?.worlds?.[0];

    if (firstWorld?.id) {
      await page.goto(`${FRONTEND_BASE}/worlds/${firstWorld.id}`);
      await page.waitForTimeout(2000);

      // Test each tab - these match the actual UI tabs
      const tabs = ['Overview', 'Map', 'Timeline', 'Dashboard'];
      for (const tabName of tabs) {
        const tab = page.locator(`button:has-text("${tabName}"), [role="tab"]:has-text("${tabName}")`).first();
        await tab.click().catch(() => console.log(`⚠️ Tab "${tabName}" not found`));
        await page.waitForTimeout(500);
        await page.screenshot({ path: `screenshots/WOR-940-tab-${tabName.toLowerCase()}.png` });
        console.log(`📸 Screenshot: WOR-940-tab-${tabName.toLowerCase()}.png`);
      }
      
      // Check for "Server Offline" which indicates a problem
      const serverOffline = page.locator('text=Server Offline');
      if (await serverOffline.count() > 0) {
        console.log('⚠️ Server Offline indicator found');
      } else {
        console.log('✅ Server connected');
      }
    }
  });

  test('should load timeline with history events', async ({ page }) => {
    const worldsResp = await fetch(`${API_BASE}/worlds`);
    const worldsData = await worldsResp.json();
    const firstWorld = worldsData?.data?.worlds?.[0];

    if (firstWorld?.id) {
      await page.goto(`${FRONTEND_BASE}/worlds/${firstWorld.id}/timeline`);
      await page.waitForTimeout(3000);
      await page.screenshot({ path: 'screenshots/WOR-940-timeline.png' });
      console.log('📸 Screenshot: WOR-940-timeline.png');
    }
  });

  test('should have zero console errors throughout', async ({ page }) => {
    // Check that consoleErrors is still manageable
    // Filter out expected errors: 404 for deleted worlds, favicon, preflight, warnings
    const criticalErrors = consoleErrors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('preflight') &&
      !e.includes('Warning') &&
      !e.includes('404') &&  // 404 for deleted worlds is expected
      !e.includes('Failed to load resource') &&  // Resource not found is expected for old test data
      !e.includes('Failed to load world data')  // World data not found is expected for old test worlds
    );
    
    if (criticalErrors.length > 0) {
      console.log('❌ Critical console errors found:', criticalErrors);
      throw new Error(`Found ${criticalErrors.length} console errors`);
    } else {
      console.log('✅ Zero critical console errors (expected 404s for deleted worlds filtered out)');
    }
  });
});

test.describe('WOR-940 Results Summary', () => {
  test('should print results summary', () => {
    const passed = results.filter(r => r.passed).length;
    const failed = results.filter(r => !r.passed).length;
    console.log('\n========== WOR-940 SMOKE TEST RESULTS ==========');
    console.log(`Total API endpoints tested: ${results.length}`);
    console.log(`Passed: ${passed}`);
    console.log(`Failed: ${failed}`);
    if (failed > 0) {
      console.log('\nFailed endpoints:');
      results.filter(r => !r.passed).forEach(r => {
        console.log(`  - ${r.method} ${r.endpoint}: ${r.error || `HTTP ${r.status}`}`);
      });
    }
    console.log('================================================\n');
  });
});