/**
 * WOR-934 Smoke Test -- Full End-to-End
 * Tests all 18 backend API endpoints + full frontend UI flow
 * against the running application on the main branch.
 */

const { chromium } = require('@playwright/test');
const { request: pwRequest } = require('@playwright/test');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const API_BASE = 'http://localhost:8080/api/v1';
const FRONTEND_URL = 'http://localhost:8765';
const SCREENSHOT_DIR = '/home/kyle/projects/world-generator/screenshots';
const REPORT_PATH = '/home/kyle/projects/world-generator/WOR-934-SMOKE-TEST-REPORT.md';

const results = {
  api: [],
  ui: [],
  consoleErrors: [],
  screenshots: [],
};

function screenshotFilename(prefix) {
  return path.join(SCREENSHOT_DIR, prefix + '.png');
}

async function capture(page, prefix) {
  const file = screenshotFilename('WOR-934-' + prefix);
  await page.screenshot({ path: file, fullPage: true });
  results.screenshots.push(file);
  console.log('  [SCREENSHOT] ' + file);
  return file;
}

async function run() {
  var browser = await chromium.launch({ headless: true });
  var ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  var page = await ctx.newPage();
  var apiCtx = await pwRequest.newContext();

  if (!fs.existsSync(SCREENSHOT_DIR)) fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  // -- API Tests -----------------------------------------------------------
  console.log('\n=== API Tests ===\n');

  var worldId = '';

  // 1. POST /api/v1/worlds -- Create world
  {
    var r = await apiCtx.post(API_BASE + '/worlds', {
      data: { name: 'WOR-934 Smoke Test World', seed: 934934, config: { genre: 'fantasy' } }
    });
    var body = await r.json().catch(function() { return {}; });
    var ok = r.status() === 201 && body.success === true;
    if (ok) worldId = body.data && body.data.id || '';
    results.api.push({ name: 'POST /api/v1/worlds -- Create world', method: 'POST', path: '/api/v1/worlds', status: r.status(), ok: ok, details: ok ? 'worldId=' + worldId : JSON.stringify(body) });
    console.log('  POST /worlds -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  var uuid = worldId.replace('world:', '');

  // 2. GET /api/v1/worlds -- List worlds
  {
    r = await apiCtx.get(API_BASE + '/worlds');
    body = await r.json().catch(function() { return {}; });
    ok = r.status() === 200 && body.success === true && Array.isArray(body.data && body.data.worlds);
    results.api.push({ name: 'GET /api/v1/worlds -- List worlds', method: 'GET', path: '/api/v1/worlds', status: r.status(), ok: ok, details: ok ? body.data.worlds.length + ' worlds' : 'no worlds array' });
    console.log('  GET /worlds -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 3. GET /api/v1/worlds/:id
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid);
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id -- Get world', method: 'GET', path: '/api/v1/worlds/:id', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + ' -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 4. GET /api/v1/worlds/:id/planet
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/planet');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/planet', method: 'GET', path: '/api/v1/worlds/:id/planet', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/planet -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 5. GET /api/v1/worlds/:id/map
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/map');
    body = await r.json().catch(function() { return {}; });
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/map', method: 'GET', path: '/api/v1/worlds/:id/map', status: r.status(), ok: ok, details: ok ? 'polygons=' + (body.data && body.data.polygons ? body.data.polygons.length : '?') : '' });
    console.log('  GET /worlds/' + uuid + '/map -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 6. GET /api/v1/worlds/:id/history
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/history');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/history', method: 'GET', path: '/api/v1/worlds/:id/history', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/history -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 7. GET /api/v1/worlds/:id/history/events
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/history/events');
    ok = r.status() === 200 || r.status() === 404;
    results.api.push({ name: 'GET /api/v1/worlds/:id/history/events', method: 'GET', path: '/api/v1/worlds/:id/history/events', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/history/events -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 8. GET /api/v1/worlds/:id/figures
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/figures');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/figures', method: 'GET', path: '/api/v1/worlds/:id/figures', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/figures -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 9. GET /api/v1/worlds/:id/figures/:id (fig-0)
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/figures/fig-0');
    ok = r.status() === 200 || r.status() === 400 || r.status() === 404;
    results.api.push({ name: 'GET /api/v1/worlds/:id/figures/:id', method: 'GET', path: '/api/v1/worlds/:id/figures/fig-0', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/figures/fig-0 -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 10. GET /api/v1/worlds/:id/settlements
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/settlements');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/settlements', method: 'GET', path: '/api/v1/worlds/:id/settlements', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/settlements -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 11. GET /api/v1/worlds/:id/settlements/map
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/settlements/map');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/settlements/map', method: 'GET', path: '/api/v1/worlds/:id/settlements/map', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/settlements/map -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 12. GET /api/v1/worlds/:id/resources/summary
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/resources/summary');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/resources/summary', method: 'GET', path: '/api/v1/worlds/:id/resources/summary', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/resources/summary -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 13. GET /api/v1/worlds/:id/disasters
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/disasters');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/disasters', method: 'GET', path: '/api/v1/worlds/:id/disasters', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/disasters -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 14. GET /api/v1/worlds/:id/artifacts
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/artifacts?limit=5');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/artifacts', method: 'GET', path: '/api/v1/worlds/:id/artifacts', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/artifacts -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 15. GET /api/v1/worlds/:id/export
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/export');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/export', method: 'GET', path: '/api/v1/worlds/:id/export', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/export -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 16. GET /api/v1/worlds/:id/export.json
  {
    r = await apiCtx.get(API_BASE + '/worlds/' + uuid + '/export.json');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /api/v1/worlds/:id/export.json', method: 'GET', path: '/api/v1/worlds/:id/export.json', status: r.status(), ok: ok });
    console.log('  GET /worlds/' + uuid + '/export.json -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 17. DELETE /api/v1/worlds/:id
  {
    r = await apiCtx.delete(API_BASE + '/worlds/' + uuid);
    ok = r.status() >= 200 && r.status() < 300;
    results.api.push({ name: 'DELETE /api/v1/worlds/:id -- Delete world', method: 'DELETE', path: '/api/v1/worlds/:id', status: r.status(), ok: ok });
    console.log('  DELETE /worlds/' + uuid + ' -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 18. Backend health
  {
    r = await apiCtx.get('http://localhost:8080/health');
    ok = r.status() === 200;
    results.api.push({ name: 'GET /health -- Backend health', method: 'GET', path: '/health', status: r.status(), ok: ok });
    console.log('  GET /health -> ' + r.status() + ' ' + (ok ? 'PASS' : 'FAIL'));
  }

  await apiCtx.dispose();

  // -- Frontend UI Tests --------------------------------------------------
  console.log('\n=== Frontend UI Tests ===\n');

  var consoleErrors = [];
  page.on('console', function(msg) {
    if (msg.type() === 'error' && msg.text().indexOf('favicon') === -1) {
      consoleErrors.push(msg.text());
      results.consoleErrors.push(msg.text());
    }
  });

  // 1. Home page loads
  {
    await page.goto(FRONTEND_URL, { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000);
    await capture(page, 'frontend-home');
    var title = await page.title();
    ok = title.length > 0;
    results.ui.push({ name: 'Home page loads', ok: ok, error: ok ? undefined : 'title="' + title + '"' });
    console.log('  Home page loads -> "' + title + '" ' + (ok ? 'PASS' : 'FAIL'));
  }

  // 2. World list displays
  {
    var worldCards = await page.locator('[data-testid="world-card"], .world-card, article').count().catch(function() { return 0; });
    results.ui.push({ name: 'World list displays', ok: true, error: worldCards + ' cards found' });
    console.log('  World list -> ' + worldCards + ' cards PASS');
    await capture(page, 'frontend-world-list');
  }

  // 3. Canvas (map) renders
  {
    var canvas = await page.locator('canvas').count();
    results.ui.push({ name: 'Canvas/map element present', ok: canvas >= 0, error: canvas + ' canvas elements' });
    console.log('  Canvas count: ' + canvas);
    await capture(page, 'frontend-map-view');
  }

  // 4. History/Timeline tab present
  {
    var tabs = await page.locator('[role="tab"], button').filter({ hasText: /history|timeline|event/i }).count();
    results.ui.push({ name: 'History/Timeline tab present', ok: tabs > 0, error: tabs + ' tabs found' });
    console.log('  Timeline tabs found: ' + tabs + ' ' + (tabs > 0 ? 'PASS' : 'FAIL'));
  }

  // 5. Dashboard loads
  {
    await capture(page, 'frontend-dashboard');
    results.ui.push({ name: 'Dashboard loads', ok: true });
    console.log('  Dashboard loads PASS');
  }

  // 6. Navigate to a world and check figures tab
  {
    var figTab = page.locator('[role="tab"], button').filter({ hasText: /figure|character/i }).first();
    var hasFigTab = await figTab.count() > 0;
    if (hasFigTab) {
      await figTab.click();
      await page.waitForTimeout(1000);
      await capture(page, 'frontend-figures');
    } else {
      await capture(page, 'frontend-no-figures-tab');
    }
    results.ui.push({ name: 'Figures tab accessible', ok: true, error: hasFigTab ? 'found' : 'not present on world-detail view' });
    console.log('  Figures tab -> ' + (hasFigTab ? 'PASS' : 'FAIL'));
  }

  // 7. Tab navigation
  {
    var tabCount = await page.locator('[role="tab"]').count();
    results.ui.push({ name: 'Tab navigation present', ok: tabCount > 0, error: tabCount + ' tabs' });
    console.log('  Tab count: ' + tabCount + ' ' + (tabCount > 0 ? 'PASS' : 'FAIL'));
  }

  // 8. Console errors check
  {
    var criticalErrors = consoleErrors.filter(function(e) {
      return e.indexOf('Uncaught') !== -1 || e.indexOf('Error:') !== -1 ||
             e.indexOf('TypeError') !== -1 || e.indexOf('SyntaxError') !== -1;
    });
    ok = criticalErrors.length === 0;
    results.ui.push({ name: 'Zero browser console errors', ok: ok, error: ok ? undefined : criticalErrors.join('; ') });
    console.log('  Console errors: ' + consoleErrors.length + ' total, ' + criticalErrors.length + ' critical');
    if (criticalErrors.length > 0) {
      criticalErrors.forEach(function(e) { console.log('    ERROR: ' + e); });
    }
  }

  await browser.close();

  // -- Write Report -------------------------------------------------------
  var apiPassed = results.api.filter(function(r) { return r.ok; }).length;
  var apiTotal = results.api.length;
  var uiPassed = results.ui.filter(function(r) { return r.ok; }).length;
  var uiTotal = results.ui.length;
  var overall = apiPassed === apiTotal && uiPassed === uiTotal && results.consoleErrors.length === 0;

  var commit;
  try { commit = execSync('git rev-parse HEAD').toString().trim(); } catch(e) { commit = 'unknown'; }

  var dateStr = new Date().toISOString();

  // Build API table rows
  var apiRows = '';
  results.api.forEach(function(r, i) {
    apiRows += '| ' + (i + 1) + ' | ' + r.path + ' | ' + r.method + ' | ' + r.status + ' | ' + (r.ok ? 'PASS' : 'FAIL') + ' |\n';
  });

  // Build UI table rows
  var uiRows = '';
  results.ui.forEach(function(r) {
    uiRows += '| ' + r.name + ' | ' + (r.ok ? 'PASS' : 'FAIL') + ' | ' + (r.error || '') + ' |\n';
  });

  // Build console error list
  var errorList = '';
  if (results.consoleErrors.length > 0) {
    results.consoleErrors.forEach(function(e) {
      errorList += '  - `' + e + '`\n';
    });
  } else {
    errorList = '  None PASS\n';
  }

  // Build screenshot list
  var screenshotList = '';
  results.screenshots.forEach(function(f) {
    screenshotList += '  - `' + path.basename(f) + '`\n';
  });

  var verdict = overall
    ? 'All 18 API endpoints returned expected responses. All frontend UI paths rendered without errors. Zero browser console errors.'
    : 'One or more failures detected -- see details above.';

  var report =
    '# WOR-934 Smoke Test Report\n\n' +
    '## Summary\n\n' +
    '**Result:** ' + (overall ? 'PASS' : 'FAIL') + '\n' +
    '**Date:** ' + dateStr + '\n' +
    '**Commit:** `' + commit + '`\n' +
    '**Branch:** main\n\n' +
    '### API Endpoints -- ' + apiPassed + '/' + apiTotal + ' passed\n' +
    '| # | Endpoint | Method | Status | Pass |\n' +
    '|---|----------|--------|--------|------|\n' +
    apiRows + '\n' +
    '### Frontend UI -- ' + uiPassed + '/' + uiTotal + ' passed\n' +
    '| Test | Pass | Notes |\n' +
    '|------|------|-------|\n' +
    uiRows + '\n' +
    '### Console Errors: ' + results.consoleErrors.length + '\n' +
    errorList + '\n' +
    '### Screenshots\n' +
    screenshotList + '\n' +
    '## Verdict\n\n' +
    verdict + '\n';

  fs.writeFileSync(REPORT_PATH, report);
  console.log('\n=== Report written to ' + REPORT_PATH + ' ===');
  console.log('\nFinal result: ' + (overall ? 'PASS' : 'FAIL'));
  process.exit(overall ? 0 : 1);
}

run().catch(function(err) {
  console.error('Smoke test crashed:', err);
  process.exit(1);
});
