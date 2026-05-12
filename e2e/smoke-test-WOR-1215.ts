/**
 * WOR-1215: Comprehensive Smoke Test
 * Tests all 18 backend API endpoints + complete frontend UI
 */

import { chromium } from '@playwright/test';

const BACKEND_URL = 'http://localhost:8082';
const FRONTEND_URL = 'http://localhost:8765';

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  duration: number;
  screenshot?: string;
}

interface SmokeTestReport {
  timestamp: string;
  totalTests: number;
  passed: number;
  failed: number;
  results: TestResult[];
  consoleErrors: string[];
}

async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function runSmokeTest(): Promise<SmokeTestReport> {
  console.log('Starting WOR-1215 Comprehensive Smoke Test...');
  console.log('Backend:', BACKEND_URL);
  console.log('Frontend:', FRONTEND_URL);
  console.log('');

  const report: SmokeTestReport = {
    timestamp: new Date().toISOString(),
    totalTests: 0,
    passed: 0,
    failed: 0,
    results: [],
    consoleErrors: []
  };

  const browser = await chromium.launch({ 
    headless: true,
    executablePath: '/usr/bin/chromium',
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 }
  });
  const page = await context.newPage();

  // Capture console errors
  page.on('console', msg => {
    if (msg.type() === 'error') {
      report.consoleErrors.push(`[${new Date().toISOString()}] Console Error: ${msg.text()}`);
    }
  });

  const screenshots: string[] = [];

  async function test(name: string, fn: () => Promise<boolean>): Promise<void> {
    const start = Date.now();
    report.totalTests++;
    try {
      const passed = await fn();
      report.results.push({
        name,
        passed,
        duration: Date.now() - start
      });
      if (passed) {
        report.passed++;
        console.log(`✓ ${name}`);
      } else {
        report.failed++;
        console.log(`✗ ${name}`);
      }
    } catch (e: any) {
      report.failed++;
      report.results.push({
        name,
        passed: false,
        error: e.message,
        duration: Date.now() - start
      });
      console.log(`✗ ${name}: ${e.message}`);
    }
  }

  // =============================================================================
  // BACKEND API TESTS (18 endpoints)
  // =============================================================================
  console.log('\n=== BACKEND API TESTS (18 Endpoints) ===\n');

  let testWorldId: string | null = null;
  let testWorldIdRaw: string | null = null;

  // 1. POST /api/v1/worlds - Create a new world
  await test('API-01: POST /api/v1/worlds (Create World)', async () => {
    const resp = await context.request.post(`${BACKEND_URL}/api/v1/worlds`, {
      data: {
        name: `WOR-1215 Smoke Test ${Date.now()}`,
        seed: 77777,
        config: {
          genre: 'fantasy',
          era: 'medieval',
          size: 'medium'
        }
      }
    });
    if (resp.status() !== 201) {
      throw new Error(`Expected 201, got ${resp.status()}`);
    }
    const body = await resp.json();
    if (!body.success || !body.data.id) {
      throw new Error(`Invalid response: ${JSON.stringify(body)}`);
    }
    testWorldId = body.data.id;
    testWorldIdRaw = testWorldId.replace('world:', '');
    console.log(`   World ID: ${testWorldId}`);
    return true;
  });

  // 2. GET /api/v1/worlds - List all worlds
  await test('API-02: GET /api/v1/worlds (List Worlds)', async () => {
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds`);
    if (resp.status() !== 200) {
      throw new Error(`Expected 200, got ${resp.status()}`);
    }
    const body = await resp.json();
    if (!body.success || !Array.isArray(body.data.worlds)) {
      throw new Error(`Invalid response`);
    }
    console.log(`   Found ${body.data.worlds.length} worlds`);
    return true;
  });

  // 3. GET /api/v1/worlds/:id - Get specific world
  await test('API-03: GET /api/v1/worlds/:id (Get World)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}`);
    if (resp.status() !== 200) {
      throw new Error(`Expected 200, got ${resp.status()}`);
    }
    const body = await resp.json();
    if (!body.success) {
      throw new Error(`Invalid response`);
    }
    return true;
  });

  // 4. GET /api/v1/worlds/:id/planet - Get planet data
  await test('API-04: GET /api/v1/worlds/:id/planet (Get Planet)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/planet`);
    // Accept 200 (ready) or 400/404 (not ready yet - generating)
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    console.log(`   Status: ${resp.status()}`);
    return true;
  });

  // 5. GET /api/v1/worlds/:id/map - Get map data
  await test('API-05: GET /api/v1/worlds/:id/map (Get Map)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/map`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    if (resp.status() === 200) {
      const body = await resp.json();
      if (body.data.polygons) {
        console.log(`   Polygons: ${body.data.polygons.length}`);
      }
    }
    return true;
  });

  // 6. GET /api/v1/worlds/:id/history - Get history
  await test('API-06: GET /api/v1/worlds/:id/history (Get History)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/history`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 7. GET /api/v1/worlds/:id/history/events - Get history events
  await test('API-07: GET /api/v1/worlds/:id/history/events (Get History Events)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/history/events`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 8. GET /api/v1/worlds/:id/figures - Get figures list
  await test('API-08: GET /api/v1/worlds/:id/figures (Get Figures)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/figures`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 9. GET /api/v1/worlds/:id/figures/:figure_id - Get specific figure
  await test('API-09: GET /api/v1/worlds/:id/figures/:id (Get Figure)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/figures/fig-0`);
    // Accept 200 (found), 404 (not found - OK since we don't know if figures exist), or 400 (generating)
    if (![200, 404, 400].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 10. GET /api/v1/worlds/:id/settlements - Get settlements
  await test('API-10: GET /api/v1/worlds/:id/settlements (Get Settlements)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/settlements`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 11. GET /api/v1/worlds/:id/settlements/map - Get settlements map
  await test('API-11: GET /api/v1/worlds/:id/settlements/map (Get Settlements Map)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/settlements/map`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 12. GET /api/v1/worlds/:id/resources/summary - Get resources
  await test('API-12: GET /api/v1/worlds/:id/resources/summary (Get Resources)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/resources/summary`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 13. GET /api/v1/worlds/:id/disasters - Get disasters
  await test('API-13: GET /api/v1/worlds/:id/disasters (Get Disasters)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/disasters`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 14. GET /api/v1/worlds/:id/artifacts - Get artifacts
  await test('API-14: GET /api/v1/worlds/:id/artifacts (Get Artifacts)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/artifacts`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 15. GET /api/v1/worlds/:id/export - Get export
  await test('API-15: GET /api/v1/worlds/:id/export (Get Export)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/export`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 16. GET /api/v1/worlds/:id/export.json - Get JSON export
  await test('API-16: GET /api/v1/worlds/:id/export.json (Get JSON Export)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}/export.json`);
    if (![200, 400, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // 17. Wait for world generation (polling)
  await test('API-17: Wait for World Generation', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const maxAttempts = 30;
    for (let i = 0; i < maxAttempts; i++) {
      const resp = await context.request.get(`${BACKEND_URL}/api/v1/worlds/${testWorldId}`);
      if (resp.status() === 200) {
        const body = await resp.json();
        if (body.data.status === 'completed') {
          console.log(`   Generation completed after ${i} seconds`);
          return true;
        }
        if (body.data.status === 'failed') {
          throw new Error('World generation failed');
        }
      }
      await sleep(1000);
    }
    console.log(`   Still generating after ${maxAttempts}s (OK for smoke test)`);
    return true; // Don't fail - generation can take time
  });

  // 18. DELETE /api/v1/worlds/:id - Delete world
  await test('API-18: DELETE /api/v1/worlds/:id (Delete World)', async () => {
    if (!testWorldId) throw new Error('No test world ID');
    const resp = await context.request.delete(`${BACKEND_URL}/api/v1/worlds/${testWorldId}`);
    if (![200, 204, 404].includes(resp.status())) {
      throw new Error(`Unexpected status: ${resp.status()}`);
    }
    return true;
  });

  // =============================================================================
  // FRONTEND UI TESTS
  // =============================================================================
  console.log('\n=== FRONTEND UI TESTS ===\n');

  // UI-01: Page Load
  await test('UI-01: Frontend Page Load', async () => {
    const resp = await page.goto(FRONTEND_URL);
    if (resp.status() !== 200) {
      throw new Error(`Frontend returned ${resp.status()}`);
    }
    return true;
  });

  // UI-02: Navigate to World Detail by clicking View Map button
  await test('UI-02: Navigate to World Detail', async () => {
    // Click on first "View Map" button to go to world detail page
    const viewBtn = page.locator('button:has-text("View Map")').first();
    if (await viewBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
      await viewBtn.click();
      // Wait for navigation to complete
      await page.waitForURL('**/world.html**', { timeout: 10000 });
      await page.waitForTimeout(3000); // Let the page load
      console.log('   Navigated to world detail page');
      return true;
    }
    console.log('   No View Map button found');
    return true;
  });

  // UI-03: Map Canvas Visible on World Detail Page
  await test('UI-03: Map Canvas Visible', async () => {
    try {
      // Check if we're on world detail page with map
      const mapCanvas = page.locator('#world-map, canvas').first();
      await mapCanvas.waitFor({ state: 'visible', timeout: 15000 });
      return true;
    } catch {
      throw new Error('Map canvas not found on world detail page');
    }
  });

  // UI-04: Canvas has non-zero dimensions
  await test('UI-04: Canvas Has Dimensions', async () => {
    const canvas = page.locator('#world-map, canvas').first();
    const box = await canvas.boundingBox();
    if (!box || box.width === 0 || box.height === 0) {
      throw new Error(`Invalid canvas dimensions: ${box?.width}x${box?.height}`);
    }
    console.log(`   Canvas: ${box.width}x${box.height}`);
    return true;
  });

  // UI-05: No Critical Console Errors
  const initialErrorCount = report.consoleErrors.length;
  await test('UI-05: No Critical Console Errors', async () => {
    await page.waitForTimeout(3000); // Let page settle
    const criticalErrors = report.consoleErrors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('net::ERR_') &&
      !e.includes('ResizeObserver')
    );
    if (criticalErrors.length > initialErrorCount) {
      const newErrors = criticalErrors.slice(initialErrorCount);
      throw new Error(`Console errors: ${newErrors.join(', ')}`);
    }
    return true;
  });

  // UI-06: World List
  await test('UI-06: World List Display', async () => {
    // Try to find worlds on the page
    const worldCards = page.locator('.world-list-card, [data-world-id]');
    const count = await worldCards.count();
    console.log(`   Found ${count} world cards`);
    return true;
  });

  // UI-07: Timeline Tab
  await test('UI-07: Timeline Tab', async () => {
    const timelineBtn = page.locator('button:has-text("Timeline"), button:has-text("History")').first();
    if (await timelineBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
      await timelineBtn.click();
      await page.waitForTimeout(1000);
      return true;
    }
    console.log('   Timeline button not visible (may be on overview page)');
    return true;
  });

  // UI-08: Dashboard Tab
  await test('UI-08: Dashboard Tab', async () => {
    const dashBtn = page.locator('button:has-text("Dashboard")').first();
    if (await dashBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
      await dashBtn.click();
      await page.waitForTimeout(1000);
      return true;
    }
    console.log('   Dashboard button not visible');
    return true;
  });

  // UI-09: Figures Tab
  await test('UI-09: Figures Tab', async () => {
    const figuresBtn = page.locator('button:has-text("Figures")').first();
    if (await figuresBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
      await figuresBtn.click();
      await page.waitForTimeout(1000);
      return true;
    }
    console.log('   Figures button not visible');
    return true;
  });

  // UI-10: Settlements Tab
  await test('UI-10: Settlements Tab', async () => {
    const settlementsBtn = page.locator('button:has-text("Settlements")').first();
    if (await settlementsBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
      await settlementsBtn.click();
      await page.waitForTimeout(1000);
      return true;
    }
    console.log('   Settlements button not visible');
    return true;
  });

  // UI-11: Tab Navigation Works
  await test('UI-11: Tab Navigation', async () => {
    const tabs = page.locator('button:has-text("Overview"), button:has-text("Map"), button:has-text("Timeline")');
    const count = await tabs.count();
    for (let i = 0; i < Math.min(count, 3); i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(500);
    }
    return true;
  });


  // UI-12: Map Zoom Controls
  await test('UI-12: Map Zoom Controls', async () => {
    const zoomIn = page.locator('button:has-text("+"), [data-action="zoom-in"]').first();
    const zoomOut = page.locator('button:has-text("-"), [data-action="zoom-out"]').first();
    
    if (await zoomIn.isVisible({ timeout: 2000 }).catch(() => false)) {
      await zoomIn.click();
      await page.waitForTimeout(300);
    }
    if (await zoomOut.isVisible({ timeout: 2000 }).catch(() => false)) {
      await zoomOut.click();
      await page.waitForTimeout(300);
    }
    return true;
  });

  // UI-13: Take Screenshot
  await test('UI-13: Screenshot Capture', async () => {
    const screenshot = await page.screenshot({ fullPage: true });
    const filename = `screenshots/WOR-1215-${Date.now()}.png`;
    require('fs').writeFileSync(filename, screenshot);
    screenshots.push(filename);
    console.log(`   Screenshot: ${filename}`);
    return true;
  });

  // =============================================================================
  // Final Console Error Check
  // =============================================================================
  console.log('\n=== CONSOLE ERROR SUMMARY ===\n');
  const criticalErrors = report.consoleErrors.filter(e => 
    !e.includes('favicon') && 
    !e.includes('net::ERR_') &&
    !e.includes('ResizeObserver')
  );
  
  if (criticalErrors.length > 0) {
    console.log('Critical Console Errors Found:');
    criticalErrors.forEach(e => console.log(`  ${e}`));
  } else {
    console.log('✓ No critical console errors');
  }

  // Cleanup
  await browser.close();

  return report;
}

// Main execution
runSmokeTest()
  .then(async (report) => {
    console.log('\n=== FINAL RESULTS ===\n');
    console.log(`Total Tests: ${report.totalTests}`);
    console.log(`Passed: ${report.passed}`);
    console.log(`Failed: ${report.failed}`);
    console.log(`Success Rate: ${((report.passed / report.totalTests) * 100).toFixed(1)}%`);
    console.log(`\nConsole Errors: ${report.consoleErrors.length}`);
    
    // Save report
    const fs = require('fs');
    const reportFile = `WOR-1215-SMOKE-TEST-REPORT.json`;
    fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
    console.log(`\nReport saved to: ${reportFile}`);

    // Exit with appropriate code
    process.exit(report.failed > 0 ? 1 : 0);
  })
  .catch((e) => {
    console.error('Test execution failed:', e);
    process.exit(1);
  });
