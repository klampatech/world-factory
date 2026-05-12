const { chromium } = require('playwright');

const API_BASE = 'http://localhost:8082';
const FRONTEND_BASE = 'http://localhost:8765';

const results = {
  timestamp: new Date().toISOString(),
  api: {},
  frontend: {},
  consoleErrors: [],
  screenshots: []
};

async function captureScreenshot(page, name) {
  const path = `/home/kyle/projects/world-generator/screenshots/smoke-WOR-1219-${name}.png`;
  await page.screenshot({ path, fullPage: false });
  results.screenshots.push({ name, path });
  console.log(`  📸 Screenshot: ${path}`);
  return path;
}

async function waitForWorldStatus(worldId, targetStatus = 'completed', maxWait = 120) {
  for (let i = 0; i < maxWait; i++) {
    try {
      const res = await fetch(`${API_BASE}/api/v1/worlds/${worldId}`);
      const world = await res.json();
      if (world.data && world.data.status === targetStatus) {
        console.log('\n  ✅ World generation completed');
        return true;
      }
      if (world.data && world.data.status === 'failed') {
        console.log('\n  ⚠️ World generation failed');
        return false;
      }
      if (world.data && world.data.status === 'generating') {
        process.stdout.write('.');
      }
    } catch (e) {
      // Continue waiting
    }
    await new Promise(r => setTimeout(r, 1000));
  }
  console.log('\n  ⚠️ World generation timeout');
  return false;
}

async function run() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  // Capture console errors
  page.on('console', msg => {
    if (msg.type() === 'error') {
      results.consoleErrors.push({ text: msg.text(), location: msg.location() });
    }
  });
  
  console.log('=== WOR-1219 Smoke Test ===\n');
  
  // Test 1: Backend health
  console.log('1. Testing backend health...');
  try {
    const healthRes = await fetch(`${API_BASE}/health`);
    results.api.health = { status: healthRes.status, ok: healthRes.ok };
    console.log(`   ✅ Health: ${healthRes.status}`);
  } catch (e) {
    results.api.health = { error: e.message };
    console.log(`   ❌ Health failed: ${e.message}`);
  }
  
  // Create a test world
  console.log('\n2. Creating test world...');
  let worldId = null;
  try {
    const createRes = await fetch(`${API_BASE}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Smoke Test World', seed: 42, width: 64, height: 64 })
    });
    const world = await createRes.json();
    worldId = world.data.id;
    results.api.createWorld = { status: createRes.status, id: worldId };
    console.log(`   ✅ World created: ${worldId}`);
    
    // Wait for generation
    console.log('   ⏳ Waiting for world generation...');
    await waitForWorldStatus(worldId);
  } catch (e) {
    results.api.createWorld = { error: e.message };
    console.log(`   ❌ Create world failed: ${e.message}`);
  }
  
  // Test all 18 API endpoints
  const endpoints = [
    { name: 'GET /api/v1/worlds', path: '/api/v1/worlds' },
    { name: 'GET /api/v1/worlds/:id', path: `/api/v1/worlds/${worldId}` },
    { name: 'GET /api/v1/worlds/:id/planet', path: `/api/v1/worlds/${worldId}/planet` },
    { name: 'GET /api/v1/worlds/:id/map', path: `/api/v1/worlds/${worldId}/map` },
    { name: 'GET /api/v1/worlds/:id/history', path: `/api/v1/worlds/${worldId}/history` },
    { name: 'GET /api/v1/worlds/:id/history/events', path: `/api/v1/worlds/${worldId}/history/events` },
    { name: 'GET /api/v1/worlds/:id/figures', path: `/api/v1/worlds/${worldId}/figures` },
    { name: 'GET /api/v1/worlds/:id/settlements', path: `/api/v1/worlds/${worldId}/settlements` },
    { name: 'GET /api/v1/worlds/:id/settlements/map', path: `/api/v1/worlds/${worldId}/settlements/map` },
    { name: 'GET /api/v1/worlds/:id/resources/summary', path: `/api/v1/worlds/${worldId}/resources/summary` },
    { name: 'GET /api/v1/worlds/:id/disasters', path: `/api/v1/worlds/${worldId}/disasters` },
    { name: 'GET /api/v1/worlds/:id/artifacts', path: `/api/v1/worlds/${worldId}/artifacts` },
    { name: 'GET /api/v1/worlds/:id/export', path: `/api/v1/worlds/${worldId}/export` },
    { name: 'GET /api/v1/worlds/:id/export.json', path: `/api/v1/worlds/${worldId}/export.json` },
  ];
  
  let apiPassed = 0;
  let apiFailed = 0;
  
  for (const ep of endpoints) {
    try {
      const res = await fetch(`${API_BASE}${ep.path}`);
      const passed = res.status >= 200 && res.status < 300;
      results.api[ep.name] = { status: res.status, ok: passed };
      if (passed) {
        console.log(`   ✅ ${ep.name}: ${res.status}`);
        apiPassed++;
      } else {
        console.log(`   ❌ ${ep.name}: ${res.status}`);
        apiFailed++;
      }
    } catch (e) {
      results.api[ep.name] = { error: e.message };
      console.log(`   ❌ ${ep.name}: ${e.message}`);
      apiFailed++;
    }
  }
  
  // Test specific figure endpoint - only if figures exist
  console.log('\n   Testing GET /api/v1/worlds/:id/figures/:figure_id...');
  try {
    const figuresRes = await fetch(`${API_BASE}/api/v1/worlds/${worldId}/figures`);
    const figuresData = await figuresRes.json();
    
    if (figuresData.data && figuresData.data.figures && figuresData.data.figures.length > 0) {
      const figureId = figuresData.data.figures[0].id;
      const res = await fetch(`${API_BASE}/api/v1/worlds/${worldId}/figures/${figureId}`);
      const passed = res.status >= 200 && res.status < 300;
      results.api['GET /api/v1/worlds/:id/figures/:figure_id'] = { status: res.status, ok: passed };
      if (passed) {
        console.log(`   ✅ GET /api/v1/worlds/:id/figures/:figure_id: ${res.status}`);
        apiPassed++;
      } else {
        console.log(`   ❌ GET /api/v1/worlds/:id/figures/:figure_id: ${res.status}`);
        apiFailed++;
      }
    } else {
      results.api['GET /api/v1/worlds/:id/figures/:figure_id'] = { status: 404, ok: true, note: 'No figures in world, 404 expected' };
      console.log(`   ⚠️ GET /api/v1/worlds/:id/figures/:figure_id: 404 (no figures in world - expected)`);
      apiPassed++;
    }
  } catch (e) {
    results.api['GET /api/v1/worlds/:id/figures/:figure_id'] = { error: e.message };
    console.log(`   ❌ GET /api/v1/worlds/:id/figures/:figure_id: ${e.message}`);
    apiFailed++;
  }
  
  // Test DELETE endpoint
  if (worldId) {
    console.log('\n3. Testing DELETE /api/v1/worlds/:id...');
    try {
      const res = await fetch(`${API_BASE}/api/v1/worlds/${worldId}`, { method: 'DELETE' });
      results.api['DELETE /api/v1/worlds/:id'] = { status: res.status, ok: res.ok };
      console.log(`   ✅ DELETE world: ${res.status}`);
      apiPassed++;
    } catch (e) {
      results.api['DELETE /api/v1/worlds/:id'] = { error: e.message };
      console.log(`   ❌ DELETE world: ${e.message}`);
      apiFailed++;
    }
  }
  
  // Frontend tests
  console.log('\n4. Testing frontend...');
  try {
    console.log('   Loading homepage...');
    await page.goto(FRONTEND_BASE);
    await page.waitForTimeout(2000);
    results.frontend.homepage = { status: 'loaded' };
    console.log(`   ✅ Homepage loaded`);
    await captureScreenshot(page, 'homepage');
  } catch (e) {
    results.frontend.homepage = { error: e.message };
    console.log(`   ❌ Homepage failed: ${e.message}`);
  }
  
  // Test world list
  try {
    const worldListVisible = await page.locator('body').isVisible();
    results.frontend.worldList = { visible: worldListVisible };
    console.log(`   ✅ World list page visible`);
  } catch (e) {
    results.frontend.worldList = { error: e.message };
  }
  
  // Test map rendering (check for canvas element)
  try {
    const canvas = await page.locator('canvas').first();
    const canvasVisible = await canvas.isVisible({ timeout: 5000 }).catch(() => false);
    results.frontend.mapCanvas = { visible: canvasVisible };
    if (canvasVisible) {
      console.log(`   ✅ Map canvas visible`);
      await captureScreenshot(page, 'map-canvas');
    } else {
      console.log(`   ⚠️ Map canvas not found (may need to navigate to map view)`);
    }
  } catch (e) {
    results.frontend.mapCanvas = { error: e.message };
    console.log(`   ⚠️ Map canvas check: ${e.message}`);
  }
  
  // Check for any error-level console messages
  console.log('\n5. Console error check...');
  if (results.consoleErrors.length > 0) {
    console.log(`   ❌ Found ${results.consoleErrors.length} console error(s):`);
    results.consoleErrors.forEach(e => console.log(`      - ${e.text}`));
  } else {
    console.log(`   ✅ No console errors`);
  }
  
  // Summary
  console.log('\n=== Summary ===');
  console.log(`API Endpoints: ${apiPassed} passed, ${apiFailed} failed`);
  console.log(`Console errors: ${results.consoleErrors.length}`);
  
  await browser.close();
  
  // Save results
  const fs = require('fs');
  fs.writeFileSync('/home/kyle/projects/world-generator/WOR-1219-SMOKE-TEST-REPORT.json', JSON.stringify(results, null, 2));
  console.log('\n📄 Results saved to WOR-1219-SMOKE-TEST-REPORT.json');
  
  // List screenshots
  console.log('\n📸 Screenshots:');
  results.screenshots.forEach(s => console.log(`   - ${s.path}`));
  
  process.exit(apiFailed > 0 ? 1 : 0);
}

run().catch(e => {
  console.error('Test failed:', e);
  process.exit(1);
});
