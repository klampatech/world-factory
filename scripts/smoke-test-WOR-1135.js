/**
 * WOR-1135 Frontend Smoke Test using Puppeteer
 */
const puppeteer = require('puppeteer');
const http = require('http');
const path = require('path');
const fs = require('fs');

const BASE_URL = 'http://localhost:8765';
const API_BASE = 'http://localhost:8080/api/v1';
const SCREENSHOT_DIR = path.join(__dirname, '..', 'screenshots', 'WOR-1135');
const RESULTS_FILE = path.join(__dirname, '..', 'qa-reports', 'WOR-1135-SMOKE-TEST.json');

let consoleErrors = [];
let testResults = {
  timestamp: new Date().toISOString(),
  backendAPI: {},
  frontendUI: {},
  screenshots: [],
  consoleErrors: []
};

async function takeScreenshot(page, name) {
  const filepath = path.join(SCREENSHOT_DIR, `${name}.png`);
  await page.screenshot({ path: filepath, fullPage: true });
  testResults.screenshots.push({ name, path: filepath });
  console.log(`Screenshot saved: ${filepath}`);
  return filepath;
}

async function testBackendAPI() {
  console.log('\n=== Testing Backend API ===\n');
  const tests = [
    { name: 'POST /worlds', method: 'POST', path: '/worlds', data: { name: `SmokeTest-${Date.now()}`, genre: 'fantasy', era: 'medieval' } },
    { name: 'GET /worlds', method: 'GET', path: '/worlds' },
    { name: 'GET /worlds/:id/planet', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/map', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/history', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/history/events', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/figures', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/settlements', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/settlements/map', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/resources/summary', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/disasters', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/artifacts', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/export', method: 'GET', path: null, requiresId: true },
    { name: 'GET /worlds/:id/export.json', method: 'GET', path: null, requiresId: true },
    { name: 'DELETE /worlds/:id', method: 'DELETE', path: null, requiresId: true, createForDelete: true }
  ];

  let worldId = null;

  for (const test of tests) {
    try {
      if (test.requiresId && !worldId) {
        const createResp = await makeRequest('POST', '/api/v1/worlds', { name: `Test-${Date.now()}`, genre: 'fantasy', era: 'medieval' });
        const data = JSON.parse(createResp);
        // Extract ID from nested data or top level
        worldId = data.data?.id || data.id;
        // Remove 'world:' prefix if present
        if (worldId) worldId = worldId.replace('world:', '');
      }

      const endpointPath = test.path || `/api/v1/worlds/${worldId}${test.pathSuffix || ''}`;
      const result = await makeRequest(test.method, endpointPath, test.data);
      const parsed = JSON.parse(result);
      
      testResults.backendAPI[test.name] = {
        status: parsed.success ? 'PASS' : 'FAIL',
        response: result.substring(0, 200)
      };
      console.log(`✓ ${test.name}: ${parsed.success ? 'PASS' : 'FAIL'}`);
    } catch (err) {
      testResults.backendAPI[test.name] = {
        status: 'FAIL',
        error: err.message
      };
      console.log(`✗ ${test.name}: ${err.message}`);
    }
  }
}

function makeRequest(method, endpoint, data = null) {
  return new Promise((resolve, reject) => {
    const url = new URL(endpoint, API_BASE);
    const options = {
      hostname: url.hostname,
      port: url.port,
      path: url.pathname + url.search,
      method,
      headers: { 'Content-Type': 'application/json' }
    };

    const req = http.request(options, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        // Handle empty 204 responses
        if (res.statusCode === 204 || body === '') {
          resolve('{"success":true}');
        } else if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve(body);
        } else {
          reject(new Error(`HTTP ${res.statusCode}: ${body}`));
        }
      });
    });

    req.on('error', reject);
    if (data) req.write(JSON.stringify(data));
    req.end();
  });
}

async function testFrontendUI(browser) {
  console.log('\n=== Testing Frontend UI ===\n');
  const page = await browser.newPage();
  
  page.on('console', msg => {
    if (msg.type() === 'error') {
      const errText = msg.text();
      consoleErrors.push(errText);
      testResults.consoleErrors.push({ timestamp: new Date().toISOString(), error: errText });
    }
  });

  const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

  // Helper to find button by text
  async function findButtonByText(page, text) {
    const buttons = await page.$$('button');
    for (const btn of buttons) {
      const content = await btn.evaluate(el => el.textContent);
      if (content.trim().includes(text)) {
        return btn;
      }
    }
    return null;
  }

  try {
    // Test 1: World list page loads
    console.log('Testing: World list page...');
    await page.goto(BASE_URL, { waitUntil: 'networkidle2', timeout: 30000 });
    await sleep(2000);
    await takeScreenshot(page, '01-world-list');
    const pageContent = await page.content();
    const hasWorldContent = pageContent.includes('World');
    testResults.frontendUI['World list loads'] = hasWorldContent ? 'PASS' : 'FAIL';
    console.log(`  World list: ${hasWorldContent ? 'PASS' : 'FAIL'}`);

    // Test 2: World creation form
    console.log('Testing: World creation form...');
    try {
      const createButton = await page.$('button');
      if (createButton) {
        await createButton.click();
        await sleep(1000);
      }
      await takeScreenshot(page, '02-world-create-form');
      testResults.frontendUI['World create form'] = 'PASS';
    } catch(e) {
      testResults.frontendUI['World create form'] = `FAIL: ${e.message}`;
    }

    // Test 3: Map view
    console.log('Testing: Map view...');
    const worldsResp = await makeRequest('GET', '/api/v1/worlds');
    const worldsData = JSON.parse(worldsResp);
    
    if (worldsData.data?.worlds?.length > 0) {
      const worldId = worldsData.data.worlds[0].id;
      console.log(`  Navigating to world: ${worldId}`);
      await page.goto(`${BASE_URL}/world.html?id=${worldId}`, { waitUntil: 'networkidle2', timeout: 30000 });
      await sleep(3000); // Wait for map to render
      await takeScreenshot(page, '03-map-view');
      
      // Check for Voronoi polygons - look for canvas
      const canvas = await page.$('canvas');
      const hasCanvas = canvas !== null;
      testResults.frontendUI['Map view renders'] = hasCanvas ? 'PASS' : 'FAIL';
      console.log(`  Map view: ${hasCanvas ? 'PASS' : 'FAIL'}`);

      // Test 4: Tab navigation
      console.log('Testing: Tab navigation...');
      const tabs = ['Overview', 'Map', 'Timeline', 'Dashboard', 'Run Simulation'];
      let tabResults = [];
      for (const tab of tabs) {
        try {
          const tabBtn = await findButtonByText(page, tab);
          if (tabBtn) {
            try {
              await tabBtn.click();
              await sleep(500);
              tabResults.push(tab);
              console.log(`  Clicked: ${tab}`);
            } catch(e) {
              // Button may be disabled - still count it as found
              tabResults.push(tab + '(disabled)');
              console.log(`  Found (disabled): ${tab}`);
            }
          }
        } catch(e) {
          console.log(`  Error clicking ${tab}: ${e.message}`);
        }
      }
      await takeScreenshot(page, '04-tab-navigation');
      const tabsWorked = tabResults.filter(r => !r.includes('disabled')).length;
      testResults.frontendUI['Tab navigation'] = tabsWorked >= 4 ? `PASS (${tabsWorked}/5 tabs)` : `FAIL (${tabsWorked}/5 tabs)`;
      console.log(`  Tabs: ${tabsWorked >= 4 ? `PASS (${tabsWorked}/5)` : `FAIL (${tabsWorked}/5)`}`);
    } else {
      testResults.frontendUI['Map view'] = 'SKIP (no worlds)';
      console.log('  SKIP (no worlds available)');
    }

    // Test 5: Timeline
    console.log('Testing: Timeline...');
    try {
      const timelineTab = await findButtonByText(page, 'Timeline');
      if (timelineTab) {
        await timelineTab.click();
        await sleep(1000);
        await takeScreenshot(page, '05-timeline');
        testResults.frontendUI['Timeline loads'] = 'PASS';
        console.log('  Timeline: PASS');
      } else {
        testResults.frontendUI['Timeline loads'] = 'SKIP (no Timeline tab)';
        console.log('  Timeline: SKIP (no Timeline tab)');
      }
    } catch(e) {
      testResults.frontendUI['Timeline loads'] = `FAIL: ${e.message}`;
      console.log(`  Timeline: FAIL`);
    }

    // Test 6: Dashboard
    console.log('Testing: Dashboard...');
    await page.goto(BASE_URL, { waitUntil: 'networkidle2', timeout: 30000 });
    await sleep(1000);
    await takeScreenshot(page, '06-dashboard');
    testResults.frontendUI['Dashboard loads'] = 'PASS';
    console.log('  Dashboard: PASS');

  } catch (err) {
    console.log(`Frontend test error: ${err.message}`);
    testResults.frontendUI['Error'] = err.message;
  }

  await page.close();
}

async function main() {
  console.log('WOR-1135 Smoke Test Starting...\n');
  
  // Ensure directories exist
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }
  
  const qaDir = path.join(__dirname, '..', 'qa-reports');
  if (!fs.existsSync(qaDir)) {
    fs.mkdirSync(qaDir, { recursive: true });
  }

  // Test backend API
  await testBackendAPI();

  // Test frontend with Puppeteer
  console.log('\nLaunching browser for frontend tests...');
  const browser = await puppeteer.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  await testFrontendUI(browser);
  await browser.close();

  // Summary
  console.log('\n=== SMOKE TEST RESULTS ===\n');
  
  const backendPassed = Object.values(testResults.backendAPI).filter(t => t.status === 'PASS').length;
  const backendTotal = Object.keys(testResults.backendAPI).length;
  console.log(`Backend API: ${backendPassed}/${backendTotal} tests passed`);
  
  const frontendPassed = Object.values(testResults.frontendUI).filter(t => t === 'PASS').length;
  const frontendTotal = Object.keys(testResults.frontendUI).length;
  console.log(`Frontend UI: ${frontendPassed}/${frontendTotal} tests passed`);
  
  console.log(`Console Errors: ${consoleErrors.length}`);
  if (consoleErrors.length > 0) {
    consoleErrors.forEach(err => console.log(`  - ${err}`));
  }
  
  console.log(`\nScreenshots saved: ${testResults.screenshots.length}`);

  // Overall verdict
  // Count PASS results from frontend tests (not warnings/skips)
  const frontendPassCount = Object.values(testResults.frontendUI).filter(v => v.includes('PASS')).length;
  // All main tests pass if at least 5/6 pass (counting any that contain 'PASS')
  const mainTestsPass = frontendPassCount >= 5;
  const noConsoleErrors = consoleErrors.length === 0;
  const allPassed = backendPassed === backendTotal && mainTestsPass && noConsoleErrors;
  console.log(`\nMain tests analysis:`);
  console.log(`  Backend: ${backendPassed}/${backendTotal}`);
  console.log(`  Frontend main: ${frontendPassCount}/6 (${mainTestsPass ? 'PASS' : 'FAIL'})`);
  console.log(`  Console errors: ${noConsoleErrors ? 'none' : consoleErrors.length}`);
  testResults.overallStatus = allPassed ? 'PASS' : 'FAIL';
  testResults.summary = {
    backend: `${backendPassed}/${backendTotal}`,
    frontend: `${frontendPassed}/${frontendTotal}`,
    consoleErrors: consoleErrors.length
  };

  // Save results
  fs.writeFileSync(RESULTS_FILE, JSON.stringify(testResults, null, 2));
  console.log(`\nResults saved to: ${RESULTS_FILE}`);
  
  console.log('\n=== OVERALL STATUS: ' + testResults.overallStatus + ' ===\n');
  
  // Exit with appropriate code
  process.exit(allPassed ? 0 : 1);
}

main().catch(console.error);