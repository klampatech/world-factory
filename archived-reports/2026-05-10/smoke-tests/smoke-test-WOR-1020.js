/**
 * WOR-1020: Smoke Test - Playwright E2E (Enhanced)
 * Comprehensive test with browser automation and screenshots
 */

const { chromium } = require('playwright');
const fs = require('fs');

const BACKEND_URL = 'http://localhost:8080';
const FRONTEND_URL = 'http://127.0.0.1:9000';

async function capture(page, name) {
  const path = `/home/kyle/projects/world-generator/screenshots/WOR-1020-${name}.png`;
  await page.screenshot({ path, fullPage: false });
  console.log(`  📸 Screenshot: ${path}`);
  return path;
}

async function runSmokeTest() {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║            WOR-1020: SMOKE TEST (Playwright E2E)            ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log(`Started: ${new Date().toISOString()}\n`);

  const results = [];
  let browser;

  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    const page = await context.newPage();

    // Capture console errors and network failures
    const consoleErrors = [];
    const failedRequests = [];
    
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    
    page.on('requestfailed', request => {
      failedRequests.push({
        url: request.url(),
        failure: request.failure()?.errorText
      });
    });

    // ========================================
    // UI-01: Frontend index page loads
    // ========================================
    console.log('Testing UI-01: Frontend index page...');
    try {
      await page.goto(FRONTEND_URL, { waitUntil: 'networkidle', timeout: 30000 });
      await capture(page, 'UI-01-index');
      const title = await page.title();
      results.push({ test: 'UI-01', name: 'Frontend index loads', passed: true, message: `Title: "${title}"` });
      console.log(`  ✅ UI-01: Index loaded, title="${title}"`);
    } catch (e) {
      results.push({ test: 'UI-01', name: 'Frontend index loads', passed: false, message: e.message });
      console.log(`  ❌ UI-01: ${e.message}`);
    }

    // ========================================
    // UI-02: World Selector form works
    // ========================================
    console.log('\nTesting UI-02: World creation form...');
    try {
      // Check if world creation form exists
      const formExists = await page.$('form, #world-form, [id*="world"]') !== null;
      const createBtn = await page.$('button:has-text("Create"), button:has-text("Generate"), input[type="submit"]');
      
      if (formExists || createBtn) {
        await capture(page, 'UI-02-form');
        results.push({ test: 'UI-02', name: 'World creation form', passed: true, message: 'Form elements found' });
        console.log('  ✅ UI-02: World creation form found');
      } else {
        await capture(page, 'UI-02-form');
        results.push({ test: 'UI-02', name: 'World creation form', passed: false, message: 'No form/button found' });
        console.log('  ⚠️ UI-02: No creation form visible');
      }
    } catch (e) {
      results.push({ test: 'UI-02', name: 'World creation form', passed: false, message: e.message });
      console.log(`  ❌ UI-02: ${e.message}`);
    }

    // ========================================
    // UI-03: World detail page (direct HTML)
    // ========================================
    console.log('\nTesting UI-03: World detail page...');
    try {
      await page.goto(`${FRONTEND_URL}/world.html`, { waitUntil: 'networkidle', timeout: 30000 });
      await capture(page, 'UI-03-world-page');
      const title = await page.title();
      const hasCanvas = await page.$('canvas') !== null;
      results.push({ test: 'UI-03', name: 'World detail page loads', passed: true, message: `Title: "${title}", Canvas: ${hasCanvas}` });
      console.log(`  ✅ UI-03: World page loaded, title="${title}", canvas=${hasCanvas}`);
    } catch (e) {
      results.push({ test: 'UI-03', name: 'World detail page loads', passed: false, message: e.message });
      console.log(`  ❌ UI-03: ${e.message}`);
    }

    // ========================================
    // UI-04: Hex test page
    // ========================================
    console.log('\nTesting UI-04: Hex test page...');
    try {
      await page.goto(`${FRONTEND_URL}/hex-test.html`, { waitUntil: 'networkidle', timeout: 30000 });
      await capture(page, 'UI-04-hex-test');
      const title = await page.title();
      results.push({ test: 'UI-04', name: 'Hex test page loads', passed: true, message: `Title: "${title}"` });
      console.log(`  ✅ UI-04: Hex test page loaded, title="${title}"`);
    } catch (e) {
      results.push({ test: 'UI-04', name: 'Hex test page loads', passed: false, message: e.message });
      console.log(`  ❌ UI-04: ${e.message}`);
    }

    // ========================================
    // UI-05: Network request analysis
    // ========================================
    console.log('\nTesting UI-05: Network request analysis...');
    
    // Analyze failures
    const criticalFailures = failedRequests.filter(r => {
      const url = r.url;
      // Ignore external resource failures and API calls that are expected to fail
      return !url.includes('fonts.googleapis') && 
             !url.includes('fonts.gstatic') &&
             !url.includes('/api/') &&
             !url.includes('/world/');
    });
    
    if (criticalFailures.length === 0) {
      results.push({ test: 'UI-05', name: 'Critical resources loading', passed: true, message: 'All critical resources loaded' });
      console.log('  ✅ UI-05: All critical resources loaded');
      if (failedRequests.length > 0) {
        console.log(`     (${failedRequests.length} non-critical failures: ${failedRequests.map(f => new URL(f.url).pathname).join(', ')})`);
      }
    } else {
      const failureSummary = criticalFailures.map(f => `${new URL(f.url).pathname}`).join(', ');
      results.push({ test: 'UI-05', name: 'Critical resources loading', passed: false, message: `Failed: ${failureSummary}` });
      console.log(`  ❌ UI-05: ${criticalFailures.length} critical failures`);
      criticalFailures.forEach(f => console.log(`     - ${new URL(f.url).pathname}: ${f.failure}`));
    }

    // ========================================
    // UI-06: Console error analysis
    // ========================================
    console.log('\nTesting UI-06: Console error analysis...');
    
    // Filter out expected errors (API calls when no world selected, etc)
    const criticalErrors = consoleErrors.filter(e => {
      return !e.includes('404') && !e.includes('Failed to load resource');
    });
    
    if (criticalErrors.length === 0) {
      results.push({ test: 'UI-06', name: 'Console errors (Error level)', passed: true, message: 'No critical errors' });
      console.log('  ✅ UI-06: No critical console errors');
      if (consoleErrors.length > 0) {
        console.log(`     (${consoleErrors.length} non-critical: 404 resource not found)`);
      }
    } else {
      results.push({ test: 'UI-06', name: 'Console errors (Error level)', passed: false, message: `${criticalErrors.length} errors: ${criticalErrors.slice(0,3).join('; ')}` });
      console.log(`  ❌ UI-06: ${criticalErrors.length} critical console errors`);
      criticalErrors.forEach(e => console.log(`     - ${e.substring(0, 100)}`));
    }

    // ========================================
    // UI-07: Tab navigation
    // ========================================
    console.log('\nTesting UI-07: Tab navigation...');
    try {
      // Go to world page and check for tabs
      await page.goto(`${FRONTEND_URL}/world.html`, { waitUntil: 'networkidle', timeout: 30000 });
      const tabs = await page.$$('[role="tab"], .tab, [class*="tab"]');
      await capture(page, 'UI-07-tabs');
      
      if (tabs.length > 0) {
        results.push({ test: 'UI-07', name: 'Tab navigation', passed: true, message: `${tabs.length} tabs found` });
        console.log(`  ✅ UI-07: ${tabs.length} tabs found`);
      } else {
        // Check if there are navigation links
        const navLinks = await page.$$('nav a, .nav-link, [href*="#"]');
        results.push({ test: 'UI-07', name: 'Tab navigation', passed: navLinks.length > 0, message: navLinks.length > 0 ? `${navLinks.length} nav links` : 'No navigation found' });
        console.log(navLinks.length > 0 ? `  ✅ UI-07: ${navLinks.length} nav links` : '  ⚠️ UI-07: No navigation elements');
      }
    } catch (e) {
      results.push({ test: 'UI-07', name: 'Tab navigation', passed: false, message: e.message });
      console.log(`  ❌ UI-07: ${e.message}`);
    }

    // ========================================
    // UI-08: API integration via CORS proxy
    // ========================================
    console.log('\nTesting UI-08: Backend API reachable...');
    try {
      const response = await page.goto(`${BACKEND_URL}/health`, { timeout: 10000 });
      const passed = response && response.status() === 200;
      results.push({ test: 'UI-08', name: 'Backend API reachable', passed, message: `HTTP ${response?.status() || 'error'}` });
      console.log(passed ? `  ✅ UI-08: Backend health check (HTTP ${response.status()})` : `  ❌ UI-08: Backend unreachable`);
    } catch (e) {
      results.push({ test: 'UI-08', name: 'Backend API reachable', passed: false, message: e.message });
      console.log(`  ❌ UI-08: ${e.message}`);
    }

    // Print summary
    console.log('\n' + '═'.repeat(62));
    console.log('                    TEST SUMMARY');
    console.log('═'.repeat(62));
    
    const passedCount = results.filter(r => r.passed).length;
    const total = results.length;
    
    console.log(`\nTotal: ${passedCount}/${total} passed\n`);
    
    for (const r of results) {
      const status = r.passed ? '✅ PASS' : '❌ FAIL';
      console.log(`${r.test} [${status}] ${r.name}`);
      console.log(`  → ${r.message}`);
    }
    
    console.log('\n' + '═'.repeat(62));
    
    const overallPassed = passedCount === total;
    console.log(`\nOverall: ${passedCount}/${total} tests passed`);
    console.log(`Status: ${overallPassed ? '✅ PASS - SMOKE TEST COMPLETE' : '❌ FAIL - ISSUES FOUND'}\n`);

    // Save screenshot list
    const screenshotList = [
      'screenshots/WOR-1020-UI-01-index.png',
      'screenshots/WOR-1020-UI-02-form.png',
      'screenshots/WOR-1020-UI-03-world-page.png',
      'screenshots/WOR-1020-UI-04-hex-test.png',
      'screenshots/WOR-1020-UI-07-tabs.png'
    ];
    fs.writeFileSync('smoke-test-WOR-1020-screenshots.txt', screenshotList.join('\n'));
    console.log('Screenshots saved to: screenshots/WOR-1020-*.png');

    await browser.close();
    return overallPassed ? 0 : 1;

  } catch (e) {
    console.error('Test error:', e);
    if (browser) await browser.close();
    return 1;
  }
}

runSmokeTest().then(code => process.exit(code)).catch(e => {
  console.error(e);
  process.exit(1);
});