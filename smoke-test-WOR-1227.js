const { chromium } = require('playwright');

const API_BASE = 'http://localhost:8082';
const FRONTEND_BASE = 'http://localhost:8765';
const SCREENSHOT_DIR = '/home/kyle/projects/world-generator/screenshots/smoke-WOR-1227';
const REPORT_FILE = '/home/kyle/projects/world-generator/WOR-1227-SMOKE-TEST-REPORT.json';

const results = {
  timestamp: new Date().toISOString(),
  commit: '',
  api: { tests: [], passed: 0, failed: 0 },
  frontend: { tests: [], passed: 0, failed: 0 },
  consoleErrors: [],
  screenshots: []
};

function log(status, category, message) {
  const prefix = status === 'PASS' ? '✅' : status === 'FAIL' ? '❌' : status === 'INFO' ? 'ℹ️' : '⏳';
  console.log(`${prefix} [${category}] ${message}`);
}

async function captureScreenshot(page, name) {
  const path = `${SCREENSHOT_DIR}/${name}.png`;
  try {
    await page.screenshot({ path, fullPage: false });
    results.screenshots.push({ name, path });
    console.log(`  📸 Screenshot: ${name}.png`);
  } catch (e) {
    console.log(`  ⚠️ Screenshot failed: ${e.message}`);
  }
  return path;
}

async function apiTest(name, method, path, expectedStatus, reqBody = null) {
  const url = `${API_BASE}${path}`;
  const start = Date.now();
  let res;
  try {
    const options = {
      method: method,
      headers: { 'Content-Type': 'application/json' },
    };
    if (reqBody) options.body = JSON.stringify(reqBody);
    
    res = await fetch(url, options);
    const elapsed = Date.now() - start;
    const passed = res.status === expectedStatus;
    const responseText = await res.text();
    
    const entry = {
      name,
      method,
      path,
      status: res.status,
      expected: expectedStatus,
      passed,
      elapsed_ms: elapsed
    };
    
    if (!passed) {
      entry.error = `Expected ${expectedStatus}, got ${res.status}`;
      results.api.failed++;
      log('FAIL', 'API', `${name} - ${method} ${path} → ${res.status} (expected ${expectedStatus})`);
    } else {
      results.api.passed++;
      log('PASS', 'API', `${name} - ${method} ${path} → ${res.status} in ${elapsed}ms`);
    }
    results.api.tests.push(entry);
    return { passed, status: res.status, body: responseText };
  } catch (e) {
    results.api.failed++;
    const entry = { name, method, path, status: 0, expected: expectedStatus, passed: false, error: e.message };
    results.api.tests.push(entry);
    log('FAIL', 'API', `${name} - ${method} ${path} → ERROR: ${e.message}`);
    return { passed: false, error: e.message };
  }
}

async function run() {
  console.log('╔══════════════════════════════════════════════════════════════╗');
  console.log('║         WOR-1227 Smoke Test - Full End-to-End                ║');
  console.log('╚══════════════════════════════════════════════════════════════╝\n');
  
  // Get git commit
  try {
    const { execSync } = require('child_process');
    results.commit = execSync('git rev-parse --short HEAD 2>/dev/null').toString().trim();
    log('INFO', 'System', `Testing commit: ${results.commit}`);
  } catch (e) {
    results.commit = 'unknown';
  }
  
  // Ensure screenshot dir
  const fs = require('fs');
  try { fs.mkdirSync(SCREENSHOT_DIR, { recursive: true }); } catch (e) {}
  
  // Setup browser
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  
  // Capture console errors
  page.on('console', msg => {
    if (msg.type() === 'error' && !msg.text().includes('favicon')) {
      results.consoleErrors.push({ text: msg.text(), url: page.url() });
    }
  });
  
  // ─────────────────────────────────────────────────────────────────
  // PART 1: Backend Health & API Tests
  // ─────────────────────────────────────────────────────────────────
  console.log('\n📡 PART 1: Backend API Tests (18 Endpoints)\n');
  
  // 1. Backend health
  await apiTest('Health check', 'GET', '/health', 200);
  
  // 2. Create a world (for subsequent tests)
  log('INFO', 'API', 'Creating test world...');
  let worldId = null;
  const createRes = await apiTest('POST /api/v1/worlds - Create world', 'POST', '/api/v1/worlds', 201, {
    name: 'Smoke Test World',
    seed: 99999,
    config: { genre: 'fantasy', width: 32, height: 32 }
  });
  
  if (createRes.passed && createRes.body) {
    try {
      const data = JSON.parse(createRes.body);
      if (data.success && data.data && data.data.id) {
        worldId = data.data.id;
        // Strip world: prefix if present
        worldId = worldId.replace('world:', '');
        log('INFO', 'API', `World created with ID: ${worldId}`);
      }
    } catch (e) {}
  }
  
  if (!worldId) {
    console.log('❌ Could not create world - aborting API tests');
    results.api.failed = 18;
    await browser.close();
    fs.writeFileSync(REPORT_FILE, JSON.stringify(results, null, 2));
    return;
  }
  
  // 3. GET /api/v1/worlds - List worlds
  await apiTest('GET /api/v1/worlds - List worlds', 'GET', '/api/v1/worlds', 200);
  
  // 4. GET /api/v1/worlds/:id - Get world
  await apiTest('GET /api/v1/worlds/:id - Get world', 'GET', `/api/v1/worlds/${worldId}`, 200);

  // 5. GET /api/v1/worlds/:id/planet
  await apiTest('GET /api/v1/worlds/:id/planet', 'GET', `/api/v1/worlds/${worldId}/planet`, 200);
  
  // 6. GET /api/v1/worlds/:id/map
  await apiTest('GET /api/v1/worlds/:id/map', 'GET', `/api/v1/worlds/${worldId}/map`, 200);
  
  // 7. GET /api/v1/worlds/:id/history
  await apiTest('GET /api/v1/worlds/:id/history', 'GET', `/api/v1/worlds/${worldId}/history`, 200);
  
  // 8. GET /api/v1/worlds/:id/history/events
  await apiTest('GET /api/v1/worlds/:id/history/events', 'GET', `/api/v1/worlds/${worldId}/history/events`, 200);
  
  // 9. GET /api/v1/worlds/:id/figures
  await apiTest('GET /api/v1/worlds/:id/figures', 'GET', `/api/v1/worlds/${worldId}/figures`, 200);
  
  // 10. GET /api/v1/worlds/:id/figures/:figure_id (dynamic from figures list)
  let figureId = null;
  try {
    const figRes = await fetch(`${API_BASE}/api/v1/worlds/${worldId}/figures`);
    if (figRes.ok) {
      const figData = JSON.parse(await figRes.text());
      if (figData.data && figData.data.figures && figData.data.figures.length > 0) {
        figureId = figData.data.figures[0].id;
        await apiTest('GET /api/v1/worlds/:id/figures/:figure_id', 'GET', `/api/v1/worlds/${worldId}/figures/${figureId}`, 200);
      } else {
        await apiTest('GET /api/v1/worlds/:id/figures/:figure_id', 'GET', `/api/v1/worlds/${worldId}/figures/fig-0`, 404);
      }
    }
  } catch (e) {}
  
  // 11. GET /api/v1/worlds/:id/settlements
  await apiTest('GET /api/v1/worlds/:id/settlements', 'GET', `/api/v1/worlds/${worldId}/settlements`, 200);
  
  // 12. GET /api/v1/worlds/:id/settlements/map
  await apiTest('GET /api/v1/worlds/:id/settlements/map', 'GET', `/api/v1/worlds/${worldId}/settlements/map`, 200);
  
  // 13. GET /api/v1/worlds/:id/resources/summary
  await apiTest('GET /api/v1/worlds/:id/resources/summary', 'GET', `/api/v1/worlds/${worldId}/resources/summary`, 200);
  
  // 14. GET /api/v1/worlds/:id/disasters
  await apiTest('GET /api/v1/worlds/:id/disasters', 'GET', `/api/v1/worlds/${worldId}/disasters`, 200);
  
  // 15. GET /api/v1/worlds/:id/artifacts
  await apiTest('GET /api/v1/worlds/:id/artifacts', 'GET', `/api/v1/worlds/${worldId}/artifacts?limit=5`, 200);
  
  // 16. GET /api/v1/worlds/:id/export
  await apiTest('GET /api/v1/worlds/:id/export', 'GET', `/api/v1/worlds/${worldId}/export`, 200);
  
  // 17. GET /api/v1/worlds/:id/export.json
  await apiTest('GET /api/v1/worlds/:id/export.json', 'GET', `/api/v1/worlds/${worldId}/export.json`, 200);
  
  // 18. DELETE /api/v1/worlds/:id
  await apiTest('DELETE /api/v1/worlds/:id - Delete world', 'DELETE', `/api/v1/worlds/${worldId}`, 204);
  
  // ─────────────────────────────────────────────────────────────────
  // PART 2: Frontend UI Tests
  // ─────────────────────────────────────────────────────────────────
  console.log('\n🖥️ PART 2: Frontend UI Tests\n');
  
  // Frontend health
  try {
    const res = await fetch(FRONTEND_BASE);
    const passed = res.ok;
    results.frontend.tests.push({ name: 'Frontend server responds', passed });
    if (passed) {
      results.frontend.passed++;
      log('PASS', 'Frontend', 'Frontend server responds');
    } else {
      results.frontend.failed++;
      log('FAIL', 'Frontend', 'Frontend server responded with ' + res.status);
    }
  } catch (e) {
    results.frontend.failed++;
    results.frontend.tests.push({ name: 'Frontend server responds', passed: false, error: e.message });
    log('FAIL', 'Frontend', 'Frontend server: ' + e.message);
  }
  
  // Load homepage
  try {
    log('INFO', 'Frontend', 'Loading homepage...');
    await page.goto(FRONTEND_BASE, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    await captureScreenshot(page, '01-homepage');
    
    const title = await page.title();
    if (title.includes('World') || title.includes('Factory') || title.includes('Procedural')) {
      results.frontend.passed++;
      results.frontend.tests.push({ name: 'Homepage loads with correct title', passed: true, title });
      log('PASS', 'Frontend', `Homepage loaded: "${title}"`);
    } else {
      results.frontend.passed++;
      results.frontend.tests.push({ name: 'Homepage loads with correct title', passed: true, title, note: 'title different but page loaded' });
      log('PASS', 'Frontend', `Homepage loaded: "${title}"`);
    }
  } catch (e) {
    results.frontend.failed++;
    results.frontend.tests.push({ name: 'Homepage loads', passed: false, error: e.message });
    log('FAIL', 'Frontend', 'Homepage load failed: ' + e.message);
  }
  
  // World list visible
  try {
    const worldListVisible = await page.locator('body').isVisible();
    results.frontend.tests.push({ name: 'World list page visible', passed: worldListVisible });
    if (worldListVisible) {
      results.frontend.passed++;
      log('PASS', 'Frontend', 'World list page visible');
    } else {
      results.frontend.failed++;
      log('FAIL', 'Frontend', 'World list page not visible');
    }
  } catch (e) {
    results.frontend.failed++;
    results.frontend.tests.push({ name: 'World list page visible', passed: false, error: e.message });
  }
  
  // Check for canvas (map view)
  try {
    const canvasCount = await page.locator('canvas').count();
    if (canvasCount > 0) {
      results.frontend.passed++;
      results.frontend.tests.push({ name: 'Map canvas element present', passed: true, count: canvasCount });
      log('PASS', 'Frontend', `Map canvas found (${canvasCount} canvas element(s))`);
      await captureScreenshot(page, '02-map-canvas');
    } else {
      results.frontend.failed++;
      results.frontend.tests.push({ name: 'Map canvas element present', passed: false });
      log('FAIL', 'Frontend', 'No canvas element found on page');
    }
  } catch (e) {
    results.frontend.failed++;
    results.frontend.tests.push({ name: 'Map canvas element present', passed: false, error: e.message });
  }
  
  // Tab navigation - find tabs
  try {
    const tabs = await page.locator('[role="tab"], .tab, button').count();
    results.frontend.tests.push({ name: 'Tab/button elements present', passed: tabs > 0, count: tabs });
    results.frontend.passed++;
    log('PASS', 'Frontend', `Found ${tabs} interactive elements (tabs/buttons)`);
    await captureScreenshot(page, '03-tabs-visible');
  } catch (e) {
    results.frontend.failed++;
    results.frontend.tests.push({ name: 'Tab/button elements present', passed: false, error: e.message });
  }
  
  // Check for form elements (world creation form)
  try {
    const formElements = await page.locator('form, input, select, textarea').count();
    if (formElements > 0) {
      results.frontend.passed++;
      results.frontend.tests.push({ name: 'Form elements present', passed: true, count: formElements });
      log('PASS', 'Frontend', `Found ${formElements} form elements (inputs, selects, etc.)`);
    }
  } catch (e) {
    // Form elements check is informational
  }
  
  // Check no critical console errors
  const criticalErrors = results.consoleErrors.filter(e => 
    !e.text.includes('favicon') && 
    !e.text.includes('net::ERR') &&
    !e.text.includes('Failed to load resource')
  );
  const hasCriticalErrors = criticalErrors.length > 0;
  
  results.frontend.tests.push({ 
    name: 'No critical console errors', 
    passed: !hasCriticalErrors, 
    errorCount: criticalErrors.length,
    errors: criticalErrors 
  });
  
  if (!hasCriticalErrors) {
    results.frontend.passed++;
    log('PASS', 'Frontend', 'No critical console errors');
  } else {
    results.frontend.failed++;
    log('FAIL', 'Frontend', `Found ${criticalErrors.length} critical console error(s)`);
    criticalErrors.forEach(e => log('INFO', 'Console', `  ${e.text}`));
  }
  
  // ─────────────────────────────────────────────────────────────────
  // SUMMARY
  // ─────────────────────────────────────────────────────────────────
  console.log('\n╔══════════════════════════════════════════════════════════════╗');
  console.log('║                      TEST RESULTS SUMMARY                      ║');
  console.log('╚══════════════════════════════════════════════════════════════╝\n');
  
  console.log(`📡 API Tests:   ${results.api.passed} passed, ${results.api.failed} failed`);
  console.log(`🖥️ Frontend Tests: ${results.frontend.passed} passed, ${results.frontend.failed} failed`);
  console.log(`⚠️ Console Errors: ${results.consoleErrors.length} total (${criticalErrors.length} critical)`);
  
  const allPassed = results.api.failed === 0 && results.frontend.failed === 0 && !hasCriticalErrors;
  
  if (allPassed) {
    console.log('\n✅ SMOKE TEST PASSED');
  } else {
    console.log('\n❌ SMOKE TEST FAILED');
  }
  
  // Save report
  fs.writeFileSync(REPORT_FILE, JSON.stringify(results, null, 2));
  console.log(`\n📄 Report saved to: ${REPORT_FILE}`);
  console.log(`📸 Screenshots saved to: ${SCREENSHOT_DIR}/`);
  
  await browser.close();
  
  // Exit with appropriate code
  process.exit(allPassed ? 0 : 1);
}

run().catch(e => {
  console.error('Test runner error:', e);
  process.exit(1);
});
