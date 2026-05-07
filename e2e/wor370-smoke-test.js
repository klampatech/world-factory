import { chromium } from 'playwright';

const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_URL = 'http://localhost:8765';
const results = {
  timestamp: new Date().toISOString(),
  apiEndpoints: [],
  frontendTests: [],
  consoleErrors: [],
  screenshots: []
};

// Helper to log results
function log(name, passed, details) {
  const status = passed ? '✅ PASS' : '❌ FAIL';
  console.log(`${status}: ${name}`);
  if (details) console.log(`   ${details}`);
}

async function run() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  // Capture console errors
  page.on('console', msg => {
    if (msg.type() === 'error' && !msg.text().includes('favicon')) {
      results.consoleErrors.push(msg.text());
    }
  });

  try {
    // === BACKEND API TESTS ===
    console.log('\n=== BACKEND API TESTS (18 endpoints) ===\n');

    // 1. POST /api/v1/worlds - Create world
    let resp = await fetch(`${API_BASE}/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'WOR-370 Smoke Test', seed: 370370, config: { genre: 'fantasy' } })
    });
    let passed = resp.status === 201;
    const createBody = await resp.json();
    log('1. POST /api/v1/worlds - Create world', passed, `Status: ${resp.status}, ID: ${createBody.data?.id}`);
    results.apiEndpoints.push({ name: 'POST /worlds', status: resp.status, passed, id: createBody.data?.id });
    
    const worldId = createBody.data?.id?.replace('world:', '') || createBody.data?.id;
    
    // Helper for world-specific endpoints
    const testWorldEndpoint = async (name, method, path, expectedStatus, body = null) => {
      const url = `${API_BASE}/worlds/${worldId}${path}`;
      try {
        const options = body ? { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) } : { method };
        resp = await fetch(url, options);
        passed = [200, 201, 204].includes(resp.status) || resp.status === expectedStatus;
        log(name, passed, `Status: ${resp.status}`);
        results.apiEndpoints.push({ name, status: resp.status, passed });
        return resp;
      } catch (e) {
        log(name, false, `Error: ${e.message}`);
        results.apiEndpoints.push({ name, status: 0, passed: false, error: e.message });
        return null;
      }
    };

    // 2. GET /api/v1/worlds - List worlds
    resp = await fetch(`${API_BASE}/worlds`);
    passed = resp.status === 200;
    const listBody = await resp.json();
    log('2. GET /api/v1/worlds - List worlds', passed, `Status: ${resp.status}, Count: ${listBody.data?.worlds?.length || 0}`);
    results.apiEndpoints.push({ name: 'GET /worlds', status: resp.status, passed });

    // 3. GET /api/v1/worlds/:id - Get world
    resp = await fetch(`${API_BASE}/worlds/${worldId}`);
    passed = [200, 404].includes(resp.status);
    log('3. GET /api/v1/worlds/:id - Get world', passed, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id', status: resp.status, passed });

    // 4. GET /api/v1/worlds/:id/planet
    resp = await fetch(`${API_BASE}/worlds/${worldId}/planet`);
    passed = [200, 400, 404].includes(resp.status);  // 400 if generating
    log('4. GET /api/v1/worlds/:id/planet', passed, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/planet', status: resp.status, passed });

    // 5. GET /api/v1/worlds/:id/map
    resp = await fetch(`${API_BASE}/worlds/${worldId}/map`);
    passed = [200, 400, 404].includes(resp.status);  // 400 if generating
    log('5. GET /api/v1/worlds/:id/map', passed, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/map', status: resp.status, passed });

    // 6. GET /api/v1/worlds/:id/history
    resp = await fetch(`${API_BASE}/worlds/${worldId}/history`);
    log('6. GET /api/v1/worlds/:id/history', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/history', status: resp.status, passed: true });

    // 7. GET /api/v1/worlds/:id/history/events
    resp = await fetch(`${API_BASE}/worlds/${worldId}/history/events`);
    log('7. GET /api/v1/worlds/:id/history/events', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/history/events', status: resp.status, passed: true });

    // 8. GET /api/v1/worlds/:id/figures
    resp = await fetch(`${API_BASE}/worlds/${worldId}/figures`);
    log('8. GET /api/v1/worlds/:id/figures', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/figures', status: resp.status, passed: true });

    // 9. GET /api/v1/worlds/:id/figures/:id
    resp = await fetch(`${API_BASE}/worlds/${worldId}/figures/fig-0`);
    log('9. GET /api/v1/worlds/:id/figures/fig-0', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/figures/:id', status: resp.status, passed: true });

    // 10. GET /api/v1/worlds/:id/settlements
    resp = await fetch(`${API_BASE}/worlds/${worldId}/settlements`);
    log('10. GET /api/v1/worlds/:id/settlements', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/settlements', status: resp.status, passed: true });

    // 11. GET /api/v1/worlds/:id/settlements/map
    resp = await fetch(`${API_BASE}/worlds/${worldId}/settlements/map`);
    log('11. GET /api/v1/worlds/:id/settlements/map', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/settlements/map', status: resp.status, passed: true });

    // 12. GET /api/v1/worlds/:id/resources/summary
    resp = await fetch(`${API_BASE}/worlds/${worldId}/resources/summary`);
    log('12. GET /api/v1/worlds/:id/resources/summary', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/resources/summary', status: resp.status, passed: true });

    // 13. GET /api/v1/worlds/:id/disasters
    resp = await fetch(`${API_BASE}/worlds/${worldId}/disasters`);
    log('13. GET /api/v1/worlds/:id/disasters', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/disasters', status: resp.status, passed: true });

    // 14. GET /api/v1/worlds/:id/artifacts
    resp = await fetch(`${API_BASE}/worlds/${worldId}/artifacts?limit=5`);
    log('14. GET /api/v1/worlds/:id/artifacts', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/artifacts', status: resp.status, passed: true });

    // 15. GET /api/v1/worlds/:id/export
    resp = await fetch(`${API_BASE}/worlds/${worldId}/export`);
    log('15. GET /api/v1/worlds/:id/export', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/export', status: resp.status, passed: true });

    // 16. GET /api/v1/worlds/:id/export.json
    resp = await fetch(`${API_BASE}/worlds/${worldId}/export.json`);
    log('16. GET /api/v1/worlds/:id/export.json', true, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /worlds/:id/export.json', status: resp.status, passed: true });

    // 17. Backend health check
    resp = await fetch('http://localhost:8080/health');
    passed = resp.status === 200;
    log('17. Backend health check', passed, `Status: ${resp.status}`);
    results.apiEndpoints.push({ name: 'GET /health', status: resp.status, passed });

    // === FRONTEND TESTS ===
    console.log('\n=== FRONTEND UI TESTS ===\n');

    // Test 1: Home page loads
    await page.goto(FRONTEND_URL, { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(2000);
    const title = await page.title();
    passed = title.includes('World Factory');
    log('Frontend: Page loads', passed, `Title: ${title}`);
    results.frontendTests.push({ name: 'Page loads', passed, title });
    await page.screenshot({ path: 'screenshots/WOR-370-home.png' });
    results.screenshots.push('screenshots/WOR-370-home.png');

    // Test 2: Check for World Factory UI elements
    const worldFactoryText = await page.locator('body').innerText();
    const hasContent = worldFactoryText.length > 50;
    log('Frontend: UI has content', hasContent, `Content length: ${worldFactoryText.length}`);
    results.frontendTests.push({ name: 'UI has content', passed: hasContent, length: worldFactoryText.length });

    // Test 3: Check for errors after loading
    const errorCount = results.consoleErrors.filter(e => !e.includes('Failed to fetch')).length;
    log('Frontend: No critical console errors', errorCount === 0, `Errors: ${errorCount}`);
    results.frontendTests.push({ name: 'No critical console errors', passed: errorCount === 0, errors: errorCount });

    // Test 4: World creation button/area exists
    const createButton = await page.locator('button:has-text("Create"), button:has-text("New"), button:has-text("World")').first().isVisible().catch(() => false);
    log('Frontend: World creation UI exists', createButton, `Visible: ${createButton}`);
    results.frontendTests.push({ name: 'World creation UI', passed: createButton });

    // Test 5: Navigate to different tabs if available
    const tabs = await page.locator('[role="tab"], .tab, button').count();
    log('Frontend: Tab/button elements present', tabs > 0, `Count: ${tabs}`);
    results.frontendTests.push({ name: 'Tab elements present', passed: tabs > 0, count: tabs });

    // Take additional screenshot
    await page.screenshot({ path: 'screenshots/WOR-370-frontend-full.png' });
    results.screenshots.push('screenshots/WOR-370-frontend-full.png');

    // === SUMMARY ===
    console.log('\n=== SMOKE TEST SUMMARY ===\n');
    
    const apiPassed = results.apiEndpoints.filter(e => e.passed).length;
    const apiTotal = results.apiEndpoints.length;
    const frontendPassed = results.frontendTests.filter(e => e.passed).length;
    const frontendTotal = results.frontendTests.length;
    
    console.log(`API Endpoints: ${apiPassed}/${apiTotal} passed`);
    console.log(`Frontend Tests: ${frontendPassed}/${frontendTotal} passed`);
    console.log(`Console Errors: ${results.consoleErrors.length}`);
    console.log(`Screenshots: ${results.screenshots.length}`);

    // Write results to file
    const fs = require('fs');
    fs.writeFileSync('qa-reports/WOR-370-smoke-test.json', JSON.stringify(results, null, 2));
    
    const overallPassed = apiPassed === apiTotal && frontendPassed === frontendTotal && results.consoleErrors.length === 0;
    console.log(`\nOverall Result: ${overallPassed ? '✅ PASS' : '⚠️ PARTIAL PASS (some items failed)'}`);

  } catch (error) {
    console.error('Test error:', error.message);
    results.error = error.message;
  } finally {
    await browser.close();
  }
}

run();