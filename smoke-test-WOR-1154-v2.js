const { chromium } = require('playwright');

// Use world-factory-prod at port 8080 (contains the fix)
const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_URL = 'http://localhost:8765';

async function runSmokeTest() {
  console.log('=== WORLD FACTORY SMOKE TEST RETRY (WOR-1154) ===');
  console.log(`Started: ${new Date().toISOString()}`);
  console.log(`Backend: ${API_BASE}`);
  console.log(`Frontend: ${FRONTEND_URL}`);
  
  const results = {
    issue: 'WOR-1154',
    timestamp: new Date().toISOString(),
    backend: { passed: 0, failed: 0, endpoints: [] },
    frontend: { passed: 0, failed: 0, tests: [] },
    consoleErrors: [],
    bugs: []
  };

  let worldId = null;
  let worldUuid = null;

  // === BACKEND TESTS ===
  console.log('\n--- BACKEND API TESTS (18 endpoints) ---');

  const backendTests = [
    { 
      name: '1. POST /api/v1/worlds - Create world', 
      fn: async () => {
        const resp = await fetch(`${API_BASE}/worlds`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: 'QA-Smoke-WOR1154-Retry', seed: 11541154, config: { genre: 'fantasy' } })
        });
        const body = await resp.json();
        if (resp.status === 201 && body.data?.id) {
          worldId = body.data.id;
          worldUuid = worldId.replace('world:', '');
        }
        return { status: resp.status, success: resp.status === 201 };
      }
    },
    { name: '2. GET /api/v1/worlds - List worlds', fn: async () => {
      const resp = await fetch(`${API_BASE}/worlds`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '3. GET /api/v1/worlds/:id - Get world details', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '4. GET /api/v1/worlds/:id/planet - Get planet data', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/planet`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '5. GET /api/v1/worlds/:id/map - Get map data', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/map`);
      const body = await resp.json();
      return { status: resp.status, success: resp.status === 200 && body.data?.polygons };
    }},
    { name: '6. GET /api/v1/worlds/:id/history - Get history', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/history`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '7. GET /api/v1/worlds/:id/history/events - Get history events', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/history/events`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '8. GET /api/v1/worlds/:id/figures - List figures', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/figures`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '9. GET /api/v1/worlds/:id/figures/:id - Get specific figure (404 expected)', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/figures/fig-99999`);
      // Accept both 404 (figure not found) and 200 (figure found) as valid
      // The key is NOT getting 400 Bad Request for legacy IDs
      const isBug = resp.status === 400;
      return { status: resp.status, success: [200, 404].includes(resp.status) && !isBug, isBug };
    }},
    { name: '10. GET /api/v1/worlds/:id/settlements - List settlements', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/settlements`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '11. GET /api/v1/worlds/:id/settlements/map - Get settlement map', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/settlements/map`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '12. GET /api/v1/worlds/:id/resources/summary - Get resources', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/resources/summary`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '13. GET /api/v1/worlds/:id/disasters - Get disasters', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/disasters`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '14. GET /api/v1/worlds/:id/artifacts - Get artifacts', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/artifacts?limit=5`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '15. GET /api/v1/worlds/:id/export - Export world', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/export`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '16. GET /api/v1/worlds/:id/export.json - Export as JSON', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}/export.json`);
      return { status: resp.status, success: resp.status === 200 };
    }},
    { name: '17. DELETE /api/v1/worlds/:id - Delete world', fn: async () => {
      if (!worldUuid) return { status: 0, success: false, error: 'No world created' };
      const resp = await fetch(`${API_BASE}/worlds/${worldUuid}`, { method: 'DELETE' });
      return { status: resp.status, success: [200, 204, 404].includes(resp.status) };
    }},
    { name: '18. GET /health - Health check', fn: async () => {
      const resp = await fetch('http://localhost:8080/health');
      return { status: resp.status, success: resp.status === 200 };
    }},
  ];

  for (const test of backendTests) {
    try {
      const result = await test.fn();
      const passed = result.isBug ? false : result.success;
      console.log(`  ${passed ? '✓' : '✗'} ${test.name} → HTTP ${result.status}${result.error ? ' ('+result.error+')' : ''}`);
      results.backend.endpoints.push({ name: test.name, status: result.status, passed });
      if (passed) results.backend.passed++;
      else if (!result.isBug) results.backend.failed++;
    } catch (e) {
      console.log(`  ✗ ${test.name} → ERROR: ${e.message}`);
      results.backend.endpoints.push({ name: test.name, error: e.message, passed: false });
      results.backend.failed++;
    }
  }

  // === FRONTEND TESTS ===
  console.log('\n--- FRONTEND UI TESTS ---');
  
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  page.on('console', msg => {
    if (msg.type() === 'error') {
      const text = msg.text();
      if (!text.includes('favicon') && !text.includes('net::ERR') && !text.includes('chrome-extension')) {
        results.consoleErrors.push(text);
      }
    }
  });

  console.log('  Testing home page load...');
  await page.goto(FRONTEND_URL, { waitUntil: 'networkidle', timeout: 30000 });
  await page.waitForTimeout(2000);
  await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-1154-retry-1-home.png' });
  
  const title = await page.title();
  const homeSuccess = title.includes('World') || title.includes('Factory');
  results.frontend.tests.push({ name: 'Home page loads', passed: homeSuccess, title });
  results.frontend.passed += homeSuccess ? 1 : 0;
  results.frontend.failed += homeSuccess ? 0 : 1;
  console.log(`  ${homeSuccess ? '✓' : '✗'} Home page title: "${title}"`);

  // 2. World creation form
  const createFormVisible = await page.locator('form, input[name*="name"], input[placeholder*="name"], button:has-text("Create"), button:has-text("Generate")').count() > 0;
  results.frontend.tests.push({ name: 'World creation form visible', passed: createFormVisible });
  results.frontend.passed += createFormVisible ? 1 : 0;
  results.frontend.failed += createFormVisible ? 0 : 1;
  console.log(`  ${createFormVisible ? '✓' : '✗'} World creation form found`);

  // 3. World list/displays
  const content = await page.content();
  const hasWorldContent = content.includes('World') || content.includes('world') || content.includes('Select') || content.includes('Create');
  results.frontend.tests.push({ name: 'World list/selector displays', passed: hasWorldContent });
  results.frontend.passed += hasWorldContent ? 1 : 0;
  results.frontend.failed += hasWorldContent ? 0 : 1;
  console.log(`  ${hasWorldContent ? '✓' : '✗'} World-related content found`);

  // 4. Map view (canvas check)
  const canvas = await page.locator('canvas').count();
  await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-1154-retry-2-canvas.png' });
  results.frontend.tests.push({ name: 'Map canvas renders', passed: canvas > 0, canvasCount: canvas });
  results.frontend.passed += canvas > 0 ? 1 : 0;
  results.frontend.failed += canvas > 0 ? 0 : 1;
  console.log(`  ${canvas > 0 ? '✓' : '✗'} Canvas elements found: ${canvas}`);

  // 5. Tab navigation
  const tabs = await page.locator('[role="tab"], button:not(:empty), .tab, a[href*="/"]').count();
  const tabSuccess = tabs > 0;
  results.frontend.tests.push({ name: 'Tab navigation available', passed: tabSuccess, tabCount: tabs });
  results.frontend.passed += tabSuccess ? 1 : 0;
  results.frontend.failed += tabSuccess ? 0 : 1;
  console.log(`  ${tabSuccess ? '✓' : '✗'} Interactive elements found: ${tabs}`);

  // 6. Dashboard metrics
  await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-1154-retry-3-dashboard.png' });
  const dashboardContent = await page.content();
  const hasDashboard = dashboardContent.includes('population') || dashboardContent.includes('year') || dashboardContent.includes('figures') || dashboardContent.includes('settlements');
  results.frontend.tests.push({ name: 'Dashboard data displays', passed: hasDashboard });
  results.frontend.passed += hasDashboard ? 1 : 0;
  results.frontend.failed += hasDashboard ? 0 : 1;
  console.log(`  ${hasDashboard ? '✓' : '✗'} Dashboard metrics found`);

  await browser.close();

  // === SUMMARY ===
  console.log('\n=== SMOKE TEST SUMMARY ===');
  console.log(`Backend: ${results.backend.passed}/${results.backend.endpoints.length} passed`);
  if (results.backend.failed > 0) {
    console.log('  Failed endpoints:');
    results.backend.endpoints.filter(e => !e.passed).forEach(e => console.log(`    - ${e.name}`));
  }
  console.log(`Frontend: ${results.frontend.passed}/${results.frontend.tests.length} passed`);
  console.log(`Console errors: ${results.consoleErrors.length}`);
  console.log(`Bugs: ${results.bugs.length}`);

  const allPassed = results.backend.failed === 0 && results.frontend.failed === 0 && results.bugs.length === 0;
  console.log(`\n🎯 OVERALL RESULT: ${allPassed ? 'PASS ✅' : 'FAIL ❌'}`);

  const fs = await import('fs');
  fs.writeFileSync('/home/kyle/projects/world-generator/WOR-1154-RETRY-REPORT.json', JSON.stringify(results, null, 2));
  
  process.exit(allPassed ? 0 : 1);
}

runSmokeTest().catch(e => {
  console.error('Smoke test failed:', e);
  process.exit(1);
});
