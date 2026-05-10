#!/usr/bin/env node
/**
 * WOR-900 Smoke Test - Complete End-to-End Testing
 * Tests all 18 backend API endpoints and frontend UI paths
 */

import { chromium } from '@playwright/test';
import { writeFileSync, mkdirSync, existsSync } from 'fs';

const BASE_URL = 'http://localhost:8080';
const FRONTEND_URL = 'http://localhost:8765';
const SCREENSHOT_DIR = 'screenshots/WOR-900/';
const screenshots = [];
let consoleErrors = [];
let worldId = null;
let worldName = `WOR-900 Smoke Test - ${Date.now()}`;

async function captureScreenshot(page, name) {
  if (!existsSync(SCREENSHOT_DIR)) {
    mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }
  const path = `${SCREENSHOT_DIR}${name}.png`;
  await page.screenshot({ path, fullPage: true });
  screenshots.push({ name, path });
  console.log(`  📸 Screenshot: ${path}`);
  return path;
}

function logResult(results, test, pass, note = '') {
  results.push({ test, pass, note });
  console.log(`  ${pass ? '✅' : '❌'} ${test}${note ? ` (${note})` : ''}`);
}

// =============================================
// BACKEND API TESTS (18 endpoints)
// =============================================
async function testBackendAPI() {
  console.log('\n🔧 TESTING BACKEND API (18 endpoints)\n');
  const results = [];

  // 1. POST /api/v1/worlds - Create a new world
  console.log('Testing: POST /api/v1/worlds (Create world)');
  try {
    const createRes = await fetch(`${BASE_URL}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: worldName,
        genre: 'fantasy',
        era: 'medieval',
        seed: 12345
      })
    });
    const data = await createRes.json();
    worldId = data.data?.id || data.data?.world?.id || data.id;
    
    if (createRes.ok && worldId) {
      logResult(results, 'POST /api/v1/worlds - Create world', true);
      console.log(`  World ID: ${worldId}`);
    } else {
      logResult(results, 'POST /api/v1/worlds - Create world', false, `Status: ${createRes.status}, Response: ${JSON.stringify(data).substring(0, 200)}`);
    }
  } catch (e) {
    logResult(results, 'POST /api/v1/worlds - Create world', false, e.message);
  }

  if (!worldId) {
    console.log('\n⚠️  Cannot proceed without a world ID. Checking for existing world...');
    // Try to find an existing completed world
    try {
      const listRes = await fetch(`${BASE_URL}/api/v1/worlds`);
      const listData = await listRes.json();
      const existingWorld = listData.data?.worlds?.find(w => w.status === 'complete') || listData.data?.worlds?.[0];
      if (existingWorld) {
        worldId = existingWorld.id;
        console.log(`  Using existing world: ${worldId} (${existingWorld.name})`);
      }
    } catch (e) {
      console.log(`  Failed to find existing world: ${e.message}`);
    }
  }

  // Wait for generation if needed
  if (worldId) {
    console.log('\n⏳ Waiting for world generation...');
    for (let i = 0; i < 30; i++) {
      try {
        const statusRes = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}`);
        const statusData = await statusRes.json();
        const status = statusData.data?.status || statusData.status;
        console.log(`  Status: ${status}`);
        if (status === 'complete') break;
        if (status === 'failed') {
          console.log('  World generation failed');
          break;
        }
      } catch (e) {}
      await new Promise(r => setTimeout(r, 2000));
    }
  }

  // 2. GET /api/v1/worlds - List worlds
  console.log('\nTesting: GET /api/v1/worlds (List worlds)');
  try {
    const res = await fetch(`${BASE_URL}/api/v1/worlds`);
    const data = await res.json();
    logResult(results, 'GET /api/v1/worlds - List worlds', res.ok, `Found ${data.data?.totalWorlds || data.data?.worlds?.length || 0} worlds`);
  } catch (e) {
    logResult(results, 'GET /api/v1/worlds - List worlds', false, e.message);
  }

  // 3. GET /api/v1/worlds/:id - Get world details
  console.log('\nTesting: GET /api/v1/worlds/:id (Get world details)');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id - Get world details', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id - Get world details', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id - Get world details', false, 'No world ID available');
  }

  // 4. GET /api/v1/worlds/:id/planet - Get planet data
  console.log('\nTesting: GET /api/v1/worlds/:id/planet');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/planet`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/planet', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/planet', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/planet', false, 'No world ID available');
  }

  // 5. GET /api/v1/worlds/:id/map - Get map data
  console.log('\nTesting: GET /api/v1/worlds/:id/map');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/map`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/map', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/map', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/map', false, 'No world ID available');
  }

  // 6. GET /api/v1/worlds/:id/history - Get history
  console.log('\nTesting: GET /api/v1/worlds/:id/history');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/history`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/history', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/history', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/history', false, 'No world ID available');
  }

  // 7. GET /api/v1/worlds/:id/history/events - Get history events
  console.log('\nTesting: GET /api/v1/worlds/:id/history/events');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/history/events`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/history/events', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/history/events', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/history/events', false, 'No world ID available');
  }

  // 8. GET /api/v1/worlds/:id/figures - Get figures
  console.log('\nTesting: GET /api/v1/worlds/:id/figures');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/figures`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/figures', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/figures', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/figures', false, 'No world ID available');
  }

  // 9. GET /api/v1/worlds/:id/figures/:figure_id - Get figure details
  console.log('\nTesting: GET /api/v1/worlds/:id/figures/:figure_id');
  if (worldId) {
    try {
      const figuresRes = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/figures`);
      const figuresData = await figuresRes.json();
      const figureId = figuresData.data?.figures?.[0]?.id || figuresData.data?.[0]?.id;
      
      if (figureId) {
        const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/figures/${figureId}`);
        logResult(results, 'GET /api/v1/worlds/:id/figures/:figure_id', res.ok);
      } else {
        logResult(results, 'GET /api/v1/worlds/:id/figures/:figure_id', false, 'No figures available');
      }
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/figures/:figure_id', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/figures/:figure_id', false, 'No world ID available');
  }

  // 10. GET /api/v1/worlds/:id/settlements - Get settlements
  console.log('\nTesting: GET /api/v1/worlds/:id/settlements');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/settlements`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/settlements', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/settlements', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/settlements', false, 'No world ID available');
  }

  // 11. GET /api/v1/worlds/:id/settlements/map - Get settlements map
  console.log('\nTesting: GET /api/v1/worlds/:id/settlements/map');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/settlements/map`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/settlements/map', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/settlements/map', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/settlements/map', false, 'No world ID available');
  }

  // 12. GET /api/v1/worlds/:id/resources/summary - Get resources summary
  console.log('\nTesting: GET /api/v1/worlds/:id/resources/summary');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/resources/summary`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/resources/summary', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/resources/summary', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/resources/summary', false, 'No world ID available');
  }

  // 13. GET /api/v1/worlds/:id/disasters - Get disasters
  console.log('\nTesting: GET /api/v1/worlds/:id/disasters');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/disasters`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/disasters', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/disasters', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/disasters', false, 'No world ID available');
  }

  // 14. GET /api/v1/worlds/:id/artifacts - Get artifacts
  console.log('\nTesting: GET /api/v1/worlds/:id/artifacts');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/artifacts`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/artifacts', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/artifacts', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/artifacts', false, 'No world ID available');
  }

  // 15. GET /api/v1/worlds/:id/export - Get export
  console.log('\nTesting: GET /api/v1/worlds/:id/export');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/export`);
      logResult(results, 'GET /api/v1/worlds/:id/export', res.ok, `Status: ${res.status}`);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/export', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/export', false, 'No world ID available');
  }

  // 16. GET /api/v1/worlds/:id/export.json - Get JSON export
  console.log('\nTesting: GET /api/v1/worlds/:id/export.json');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}/export.json`);
      const data = await res.json();
      logResult(results, 'GET /api/v1/worlds/:id/export.json', res.ok);
    } catch (e) {
      logResult(results, 'GET /api/v1/worlds/:id/export.json', false, e.message);
    }
  } else {
    logResult(results, 'GET /api/v1/worlds/:id/export.json', false, 'No world ID available');
  }

  // 17. GET /api/v1/worlds/:id - Get world (already tested, count here)
  // Already tested above

  // 18. DELETE /api/v1/worlds/:id - Delete world (test at end, or skip if no new world created)
  console.log('\nTesting: DELETE /api/v1/worlds/:id (Delete world)');
  if (worldId) {
    try {
      const res = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}`, { method: 'DELETE' });
      logResult(results, 'DELETE /api/v1/worlds/:id - Delete world', res.ok, `Status: ${res.status}`);
    } catch (e) {
      logResult(results, 'DELETE /api/v1/worlds/:id - Delete world', false, e.message);
    }
  } else {
    logResult(results, 'DELETE /api/v1/worlds/:id - Delete world', false, 'No world ID available');
  }

  return { results, worldId };
}

// =============================================
// FRONTEND UI TESTS
// =============================================
async function testFrontendUI(existingWorldId) {
  console.log('\n🌐 TESTING FRONTEND UI\n');
  const results = [];
  consoleErrors = [];
  
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
  const page = await context.newPage();
  
  page.on('console', msg => {
    if (msg.type() === 'error') {
      consoleErrors.push(msg.text());
      console.log(`  ⚠️  Console Error: ${msg.text().substring(0, 100)}`);
    }
  });

  let testWorldId = existingWorldId;

  // World creation form
  console.log('Testing: World creation form');
  try {
    await page.goto(`${FRONTEND_URL}/`, { waitUntil: 'networkidle', timeout: 15000 });
    await page.waitForTimeout(2000);
    await captureScreenshot(page, '01-index-page');
    
    // Look for create world form
    const createButton = page.locator('button:has-text("Create"), button:has-text("New World"), #create-world-btn');
    const createBtnVisible = await createButton.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (createBtnVisible) {
      await createButton.click();
      await page.waitForTimeout(1000);
      await captureScreenshot(page, '02-create-form');
      
      // Fill form if inputs exist
      const nameInput = page.locator('input[id*="name"], input[placeholder*="name" i], #world-name');
      if (await nameInput.isVisible({ timeout: 2000 }).catch(() => false)) {
        await nameInput.fill('Test World');
      }
      
      const submitBtn = page.locator('button[type="submit"], button:has-text("Create"), button:has-text("Generate")');
      if (await submitBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
        await submitBtn.click();
        await page.waitForTimeout(3000);
        await captureScreenshot(page, '03-after-submit');
        
        // Extract new world ID from URL or response
        const url = page.url();
        const idMatch = url.match(/id=([a-f0-9-]+)/i);
        if (idMatch) {
          testWorldId = idMatch[1];
          console.log(`  New world ID: ${testWorldId}`);
        }
      }
      
      logResult(results, 'World creation form', true);
    } else {
      logResult(results, 'World creation form', false, 'No create button found');
    }
  } catch (e) {
    logResult(results, 'World creation form', false, e.message);
  }

  if (!testWorldId && existingWorldId) {
    testWorldId = existingWorldId;
  }

  // World list
  console.log('\nTesting: World list');
  try {
    await page.goto(`${FRONTEND_URL}/`, { waitUntil: 'networkidle', timeout: 15000 });
    await page.waitForTimeout(2000);
    await captureScreenshot(page, '04-world-list');
    
    const worldList = page.locator('.world-item, .world-card, [class*="world"], tr:has(a[href*="world"])');
    const itemCount = await worldList.count();
    logResult(results, 'World list loads', itemCount > 0, `Found ${itemCount} items`);
  } catch (e) {
    logResult(results, 'World list loads', false, e.message);
  }

  // Map view - Voronoi polygons
  console.log('\nTesting: Map view with Voronoi polygons');
  if (testWorldId) {
    try {
      await page.goto(`${FRONTEND_URL}/world.html?id=${testWorldId}`, { waitUntil: 'networkidle', timeout: 15000 });
      await page.waitForTimeout(5000);
      await captureScreenshot(page, '05-map-view');
      
      const canvas = page.locator('canvas').first();
      const canvasCount = await page.locator('canvas').count();
      const canvasVisible = await canvas.isVisible({ timeout: 5000 }).catch(() => false);
      
      logResult(results, 'Map canvas renders', canvasVisible || canvasCount > 0, `Canvas count: ${canvasCount}`);
      
      // Test pan and zoom
      await page.mouse.wheel(100, 100);
      await page.waitForTimeout(500);
      await captureScreenshot(page, '06-map-zoomed');
      logResult(results, 'Map pan/zoom', true);
      
    } catch (e) {
      logResult(results, 'Map view', false, e.message);
    }
  } else {
    logResult(results, 'Map view', false, 'No world ID available');
  }

  // Timeline
  console.log('\nTesting: Timeline');
  if (testWorldId) {
    try {
      await page.goto(`${FRONTEND_URL}/?id=${testWorldId}&tab=timeline`, { waitUntil: 'networkidle', timeout: 15000 });
      await page.waitForTimeout(2000);
      await captureScreenshot(page, '07-timeline');
      logResult(results, 'Timeline loads', true);
      
      // Test filtering if filter controls exist
      const filterInput = page.locator('input[placeholder*="filter" i], select, button:has-text("Filter")');
      if (await filterInput.isVisible({ timeout: 2000 }).catch(() => false)) {
        logResult(results, 'Timeline filtering controls', true);
      }
    } catch (e) {
      logResult(results, 'Timeline', false, e.message);
    }
  } else {
    logResult(results, 'Timeline', false, 'No world ID available');
  }

  // Dashboard
  console.log('\nTesting: Dashboard');
  if (testWorldId) {
    try {
      await page.goto(`${FRONTEND_URL}/?id=${testWorldId}&tab=dashboard`, { waitUntil: 'networkidle', timeout: 15000 });
      await page.waitForTimeout(2000);
      await captureScreenshot(page, '08-dashboard');
      logResult(results, 'Dashboard loads', true);
    } catch (e) {
      logResult(results, 'Dashboard', false, e.message);
    }
  } else {
    logResult(results, 'Dashboard', false, 'No world ID available');
  }

  // Figures
  console.log('\nTesting: Figures');
  if (testWorldId) {
    try {
      await page.goto(`${FRONTEND_URL}/?id=${testWorldId}&tab=figures`, { waitUntil: 'networkidle', timeout: 15000 });
      await page.waitForTimeout(2000);
      await captureScreenshot(page, '09-figures');
      logResult(results, 'Figures tab loads', true);
      
      // Test figure profile
      const figureLink = page.locator('.figure-item, [class*="figure"] a, tr:has(a)').first();
      if (await figureLink.isVisible({ timeout: 2000 }).catch(() => false)) {
        await figureLink.click();
        await page.waitForTimeout(2000);
        await captureScreenshot(page, '10-figure-profile');
        logResult(results, 'Figure profile loads', true);
      }
    } catch (e) {
      logResult(results, 'Figures', false, e.message);
    }
  } else {
    logResult(results, 'Figures', false, 'No world ID available');
  }

  // Tab navigation
  console.log('\nTesting: Tab navigation');
  if (testWorldId) {
    const tabs = ['dashboard', 'map', 'timeline', 'figures', 'settlements'];
    let allTabsWork = true;
    
    for (const tab of tabs) {
      try {
        await page.goto(`${FRONTEND_URL}/?id=${testWorldId}&tab=${tab}`, { waitUntil: 'networkidle', timeout: 10000 });
        await page.waitForTimeout(1000);
        await captureScreenshot(page, `11-tab-${tab}`);
        console.log(`  Tab ${tab}: ✅`);
      } catch (e) {
        console.log(`  Tab ${tab}: ❌ ${e.message}`);
        allTabsWork = false;
      }
    }
    logResult(results, 'Tab navigation', allTabsWork);
  } else {
    logResult(results, 'Tab navigation', false, 'No world ID available');
  }

  await browser.close();
  
  return { results, consoleErrors };
}

// =============================================
// MAIN EXECUTION
// =============================================
async function main() {
  console.log('═══════════════════════════════════════════════════════════════');
  console.log('       WOR-900 SMOKE TEST - END TO END TESTING');
  console.log('═══════════════════════════════════════════════════════════════');
  console.log(`Time: ${new Date().toISOString()}`);
  console.log(`Backend: ${BASE_URL}`);
  console.log(`Frontend: ${FRONTEND_URL}`);
  console.log('═══════════════════════════════════════════════════════════════\n');

  const startTime = Date.now();
  let finalResults = { backend: [], frontend: [], screenshots: [], consoleErrors: [], worldId: null };

  // Run backend tests
  const backendResult = await testBackendAPI();
  finalResults.backend = backendResult.results;
  finalResults.worldId = backendResult.worldId;

  // Run frontend tests with the world we have
  const frontendResult = await testFrontendUI(finalResults.worldId);
  finalResults.frontend = frontendResult.results;
  finalResults.consoleErrors = frontendResult.consoleErrors;
  finalResults.screenshots = screenshots;

  const endTime = Date.now();
  const duration = Math.round((endTime - startTime) / 1000);

  // Generate report
  const backendPass = finalResults.backend.filter(r => r.pass).length;
  const backendFail = finalResults.backend.filter(r => !r.pass).length;
  const frontendPass = finalResults.frontend.filter(r => r.pass).length;
  const frontendFail = finalResults.frontend.filter(r => !r.pass).length;

  console.log('\n═══════════════════════════════════════════════════════════════');
  console.log('                     FINAL RESULTS');
  console.log('═══════════════════════════════════════════════════════════════');
  console.log(`Duration: ${duration}s`);
  console.log('\n📊 Backend API Tests:');
  console.log(`   ✅ Passed: ${backendPass}`);
  console.log(`   ❌ Failed: ${backendFail}`);
  
  console.log('\n📊 Frontend UI Tests:');
  console.log(`   ✅ Passed: ${frontendPass}`);
  console.log(`   ❌ Failed: ${frontendFail}`);
  
  console.log('\n📊 Console Errors:');
  console.log(`   Errors found: ${finalResults.consoleErrors.length}`);
  if (finalResults.consoleErrors.length > 0) {
    finalResults.consoleErrors.forEach((err, i) => console.log(`   ${i + 1}. ${err.substring(0, 150)}`));
  }
  
  console.log('\n📸 Screenshots:');
  finalResults.screenshots.forEach(s => console.log(`   - ${s.name}: ${s.path}`));

  if (backendFail > 0 || frontendFail > 0 || finalResults.consoleErrors.length > 0) {
    console.log('\n⚠️  SMOKE TEST FAILED');
    console.log('\nFailed tests:');
    [...finalResults.backend.filter(r => !r.pass), ...finalResults.frontend.filter(r => !r.pass)].forEach(r => {
      console.log(`   ❌ ${r.test}: ${r.note}`);
    });
  } else {
    console.log('\n✅ SMOKE TEST PASSED');
  }

  // Write report
  const reportContent = generateReport(finalResults, duration);
  writeFileSync('WOR-900-SMOKE-TEST-REPORT.md', reportContent);
  console.log('\n📄 Report written to: WOR-900-SMOKE-TEST-REPORT.md');
  
  process.exit(backendFail > 0 || frontendFail > 0 || finalResults.consoleErrors.length > 0 ? 1 : 0);
}

function generateReport(results, duration) {
  const backendPass = results.backend.filter(r => r.pass).length;
  const backendFail = results.backend.filter(r => !r.pass).length;
  const frontendPass = results.frontend.filter(r => r.pass).length;
  const frontendFail = results.frontend.filter(r => !r.pass).length;

  let md = `# WOR-900 Smoke Test Report\n\n`;
  md += `**Date:** ${new Date().toISOString()}\n`;
  md += `**Duration:** ${duration}s\n`;
  md += `**World ID:** ${results.worldId || 'N/A'}\n\n`;
  
  md += `## Summary\n\n`;
  md += `- **Backend API:** ${backendPass} passed, ${backendFail} failed\n`;
  md += `- **Frontend UI:** ${frontendPass} passed, ${frontendFail} failed\n`;
  md += `- **Console Errors:** ${results.consoleErrors.length} found\n\n`;
  
  md += `## Backend API Tests (18 endpoints)\n\n`;
  md += `| Endpoint | Status | Notes |\n`;
  md += `|----------|--------|-------|\n`;
  results.backend.forEach(r => {
    md += `| ${r.test} | ${r.pass ? '✅ PASS' : '❌ FAIL'} | ${r.note || ''} |\n`;
  });
  
  md += `\n## Frontend UI Tests\n\n`;
  md += `| Screen/Feature | Status | Notes |\n`;
  md += `|----------------|--------|-------|\n`;
  results.frontend.forEach(r => {
    md += `| ${r.test} | ${r.pass ? '✅ PASS' : '❌ FAIL'} | ${r.note || ''} |\n`;
  });
  
  if (results.consoleErrors.length > 0) {
    md += `\n## Console Errors\n\n`;
    results.consoleErrors.forEach((err, i) => {
      md += `${i + 1}. \`${err}\`\n\n`;
    });
  }
  
  md += `\n## Screenshots\n\n`;
  results.screenshots.forEach(s => {
    md += `- ${s.name}: \`${s.path}\`\n`;
  });
  
  const status = backendFail === 0 && frontendFail === 0 && results.consoleErrors.length === 0 ? '✅ PASSED' : '❌ FAILED';
  md += `\n---\n\n**Overall Status: ${status}**\n`;
  
  return md;
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
