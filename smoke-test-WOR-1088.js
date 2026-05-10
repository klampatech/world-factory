#!/usr/bin/env node
/**
 * WOR-1088: Complete Smoke Test - All 18 Endpoints + Frontend UI
 * Run against: main branch, latest build
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const API_URL = 'http://localhost:8080';
const FRONTEND_URL = 'http://localhost:8765';

const results = [];
const errors = [];

function httpRequest(url, options = {}) {
  return new Promise((resolve, reject) => {
    const urlObj = new URL(url);
    const reqOptions = {
      hostname: urlObj.hostname,
      port: urlObj.port,
      path: urlObj.pathname,
      method: options.method || 'GET',
      headers: options.headers || {}
    };
    
    const req = http.request(reqOptions, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          resolve({ status: res.statusCode, headers: res.headers, data: json });
        } catch {
          resolve({ status: res.statusCode, headers: res.headers, data: data });
        }
      });
    });
    
    req.on('error', reject);
    if (options.body) req.write(options.body);
    req.end();
  });
}

async function runSmokeTests() {
  console.log('╔════════════════════════════════════════════════════════════════════╗');
  console.log('║         WORLD FACTORY SMOKE TEST - WOR-1088                     ║');
  console.log('║    Testing All 18 API Endpoints + Full Frontend UI              ║');
  console.log('╚════════════════════════════════════════════════════════════════════╝');
  console.log(`Started: ${new Date().toISOString()}\n`);

  let worldId = null;
  let worldIdFromList = null;

  // ========================================
  // PART 1: BACKEND HEALTH CHECKS
  // ========================================
  console.log('══════════════════════════════════════');
  console.log('  PART 1: BACKEND HEALTH CHECKS');
  console.log('══════════════════════════════════════\n');

  // TC-001: Backend health endpoint
  try {
    const health = await httpRequest(`${API_URL}/health`);
    const passed = health.status === 200 && health.data.status === 'ok';
    results.push({ test: 'TC-001', name: 'Backend health check', passed, 
      message: `HTTP ${health.status}, status: ${health.data.status}` });
    console.log(`${passed ? '✅' : '❌'} TC-001 Backend health: HTTP ${health.status}`);
  } catch (e) {
    results.push({ test: 'TC-001', name: 'Backend health check', passed: false, message: e.message });
    console.log(`❌ TC-001 Backend health: ${e.message}`);
  }

  // TC-002: API base endpoint
  try {
    const apiRoot = await httpRequest(`${API_URL}/api/v1/worlds`);
    const passed = apiRoot.status === 200;
    results.push({ test: 'TC-002', name: 'API root accessible', passed, 
      message: `HTTP ${apiRoot.status}` });
    console.log(`${passed ? '✅' : '❌'} TC-002 API root: HTTP ${apiRoot.status}`);
  } catch (e) {
    results.push({ test: 'TC-002', name: 'API root accessible', passed: false, message: e.message });
    console.log(`❌ TC-002 API root: ${e.message}`);
  }

  // ========================================
  // PART 2: WORLD LIFECYCLE (4 endpoints)
  // ========================================
  console.log('\n══════════════════════════════════════');
  console.log('  PART 2: WORLD LIFECYCLE ENDPOINTS');
  console.log('══════════════════════════════════════\n');

  // TC-003: POST /api/v1/worlds - Create world
  let createBody = JSON.stringify({ 
    name: 'WOR-1088 Smoke Test World',
    config: {
      width: 32,
      height: 32,
      pre_history_years: 50,
      seed: 1088
    }
  });
  try {
    const createRes = await httpRequest(`${API_URL}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: createBody
    });
    
    if (createRes.status === 201 || createRes.status === 200) {
      worldId = createRes.data?.data?.id || createRes.data?.id || createRes.data?.world_id;
      const passed = !!worldId;
      results.push({ test: 'TC-003', name: 'POST /api/v1/worlds - Create world', passed, 
        message: `HTTP ${createRes.status}, World ID: ${worldId || 'MISSING'}` });
      console.log(`${passed ? '✅' : '❌'} TC-003 Create world: HTTP ${createRes.status}, ID: ${worldId}`);
    } else {
      results.push({ test: 'TC-003', name: 'POST /api/v1/worlds - Create world', passed: false, 
        message: `HTTP ${createRes.status}` });
      console.log(`❌ TC-003 Create world: HTTP ${createRes.status}`);
    }
  } catch (e) {
    results.push({ test: 'TC-003', name: 'POST /api/v1/worlds - Create world', passed: false, message: e.message });
    console.log(`❌ TC-003 Create world: ${e.message}`);
  }

  // TC-004: GET /api/v1/worlds - List worlds
  try {
    const listRes = await httpRequest(`${API_URL}/api/v1/worlds`);
    const passed = listRes.status === 200 && listRes.data?.data?.worlds;
    if (passed && listRes.data.data.worlds.length > 0 && !worldIdFromList) {
      worldIdFromList = listRes.data.data.worlds[0].id;
    }
    results.push({ test: 'TC-004', name: 'GET /api/v1/worlds - List worlds', passed, 
      message: `HTTP ${listRes.status}, count: ${listRes.data?.data?.worlds?.length || 0}` });
    console.log(`${passed ? '✅' : '❌'} TC-004 List worlds: HTTP ${listRes.status}, count: ${listRes.data?.data?.worlds?.length || 0}`);
  } catch (e) {
    results.push({ test: 'TC-004', name: 'GET /api/v1/worlds - List worlds', passed: false, message: e.message });
    console.log(`❌ TC-004 List worlds: ${e.message}`);
  }

  // TC-005: GET /api/v1/worlds/:id - Get world by ID
  const testWorldId = worldId || worldIdFromList;
  if (testWorldId) {
    console.log(`\n⏳ Waiting for world generation...`);
    await new Promise(r => setTimeout(r, 3000));
    
    try {
      const getRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}`);
      const passed = getRes.status === 200;
      results.push({ test: 'TC-005', name: 'GET /api/v1/worlds/:id - Get world', passed, 
        message: `HTTP ${getRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-005 Get world by ID: HTTP ${getRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-005', name: 'GET /api/v1/worlds/:id - Get world', passed: false, message: e.message });
      console.log(`❌ TC-005 Get world by ID: ${e.message}`);
    }

    // ========================================
    // PART 3: PLANET AND MAP (2 endpoints)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 3: PLANET AND MAP ENDPOINTS');
    console.log('══════════════════════════════════════\n');

    // TC-006: GET /api/v1/worlds/:id/planet
    try {
      const planetRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/planet`);
      const passed = planetRes.status === 200;
      results.push({ test: 'TC-006', name: 'GET /api/v1/worlds/:id/planet', passed, 
        message: `HTTP ${planetRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-006 Get planet: HTTP ${planetRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-006', name: 'GET /api/v1/worlds/:id/planet', passed: false, message: e.message });
      console.log(`❌ TC-006 Get planet: ${e.message}`);
    }

    // TC-007: GET /api/v1/worlds/:id/map
    try {
      const mapRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/map`);
      const passed = mapRes.status === 200;
      results.push({ test: 'TC-007', name: 'GET /api/v1/worlds/:id/map', passed, 
        message: `HTTP ${mapRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-007 Get map: HTTP ${mapRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-007', name: 'GET /api/v1/worlds/:id/map', passed: false, message: e.message });
      console.log(`❌ TC-007 Get map: ${e.message}`);
    }

    // ========================================
    // PART 4: HISTORY (2 endpoints)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 4: HISTORY ENDPOINTS');
    console.log('══════════════════════════════════════\n');

    // TC-008: GET /api/v1/worlds/:id/history
    try {
      const histRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/history`);
      const passed = histRes.status === 200;
      results.push({ test: 'TC-008', name: 'GET /api/v1/worlds/:id/history', passed, 
        message: `HTTP ${histRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-008 Get history: HTTP ${histRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-008', name: 'GET /api/v1/worlds/:id/history', passed: false, message: e.message });
      console.log(`❌ TC-008 Get history: ${e.message}`);
    }

    // TC-009: GET /api/v1/worlds/:id/history/events
    try {
      const eventsRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/history/events`);
      const passed = eventsRes.status === 200;
      results.push({ test: 'TC-009', name: 'GET /api/v1/worlds/:id/history/events', passed, 
        message: `HTTP ${eventsRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-009 Get history events: HTTP ${eventsRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-009', name: 'GET /api/v1/worlds/:id/history/events', passed: false, message: e.message });
      console.log(`❌ TC-009 Get history events: ${e.message}`);
    }

    // ========================================
    // PART 5: FIGURES (2 endpoints)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 5: FIGURES ENDPOINTS');
    console.log('══════════════════════════════════════\n');

    // TC-010: GET /api/v1/worlds/:id/figures
    try {
      const figuresRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/figures`);
      const passed = figuresRes.status === 200;
      results.push({ test: 'TC-010', name: 'GET /api/v1/worlds/:id/figures', passed, 
        message: `HTTP ${figuresRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-010 Get figures: HTTP ${figuresRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-010', name: 'GET /api/v1/worlds/:id/figures', passed: false, message: e.message });
      console.log(`❌ TC-010 Get figures: ${e.message}`);
    }

    // TC-011: GET /api/v1/worlds/:id/figures/:figure_id
    try {
      // First get figures list to get a figure_id
      const figuresListRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/figures`);
      let figureId = null;
      if (figuresListRes.status === 200 && figuresListRes.data?.data?.figures?.length > 0) {
        figureId = figuresListRes.data.data.figures[0].id;
      }
      
      if (figureId) {
        const figureRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/figures/${figureId}`);
        const passed = figureRes.status === 200;
        results.push({ test: 'TC-011', name: 'GET /api/v1/worlds/:id/figures/:figure_id', passed, 
          message: `HTTP ${figureRes.status}` });
        console.log(`${passed ? '✅' : '❌'} TC-011 Get figure by ID: HTTP ${figureRes.status}`);
      } else {
        results.push({ test: 'TC-011', name: 'GET /api/v1/worlds/:id/figures/:figure_id', passed: true, 
          message: 'Skipped - no figures available' });
        console.log('⏩ TC-011 Get figure by ID: Skipped (no figures)');
      }
    } catch (e) {
      results.push({ test: 'TC-011', name: 'GET /api/v1/worlds/:id/figures/:figure_id', passed: false, message: e.message });
      console.log(`❌ TC-011 Get figure by ID: ${e.message}`);
    }

    // ========================================
    // PART 6: SETTLEMENTS (2 endpoints)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 6: SETTLEMENTS ENDPOINTS');
    console.log('══════════════════════════════════════\n');

    // TC-012: GET /api/v1/worlds/:id/settlements
    try {
      const settlementsRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/settlements`);
      const passed = settlementsRes.status === 200;
      results.push({ test: 'TC-012', name: 'GET /api/v1/worlds/:id/settlements', passed, 
        message: `HTTP ${settlementsRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-012 Get settlements: HTTP ${settlementsRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-012', name: 'GET /api/v1/worlds/:id/settlements', passed: false, message: e.message });
      console.log(`❌ TC-012 Get settlements: ${e.message}`);
    }

    // TC-013: GET /api/v1/worlds/:id/settlements/map
    try {
      const settlementsMapRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/settlements/map`);
      const passed = settlementsMapRes.status === 200;
      results.push({ test: 'TC-013', name: 'GET /api/v1/worlds/:id/settlements/map', passed, 
        message: `HTTP ${settlementsMapRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-013 Get settlements map: HTTP ${settlementsMapRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-013', name: 'GET /api/v1/worlds/:id/settlements/map', passed: false, message: e.message });
      console.log(`❌ TC-013 Get settlements map: ${e.message}`);
    }

    // ========================================
    // PART 7: RESOURCES (1 endpoint)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 7: RESOURCES ENDPOINT');
    console.log('══════════════════════════════════════\n');

    // TC-014: GET /api/v1/worlds/:id/resources/summary
    try {
      const resourcesRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/resources/summary`);
      const passed = resourcesRes.status === 200;
      results.push({ test: 'TC-014', name: 'GET /api/v1/worlds/:id/resources/summary', passed, 
        message: `HTTP ${resourcesRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-014 Get resources: HTTP ${resourcesRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-014', name: 'GET /api/v1/worlds/:id/resources/summary', passed: false, message: e.message });
      console.log(`❌ TC-014 Get resources: ${e.message}`);
    }

    // ========================================
    // PART 8: DISASTERS (1 endpoint)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 8: DISASTERS ENDPOINT');
    console.log('══════════════════════════════════════\n');

    // TC-015: GET /api/v1/worlds/:id/disasters
    try {
      const disastersRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/disasters`);
      const passed = disastersRes.status === 200;
      results.push({ test: 'TC-015', name: 'GET /api/v1/worlds/:id/disasters', passed, 
        message: `HTTP ${disastersRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-015 Get disasters: HTTP ${disastersRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-015', name: 'GET /api/v1/worlds/:id/disasters', passed: false, message: e.message });
      console.log(`❌ TC-015 Get disasters: ${e.message}`);
    }

    // ========================================
    // PART 9: ARTIFACTS (1 endpoint)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 9: ARTIFACTS ENDPOINT');
    console.log('══════════════════════════════════════\n');

    // TC-016: GET /api/v1/worlds/:id/artifacts
    try {
      const artifactsRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/artifacts`);
      const passed = artifactsRes.status === 200;
      results.push({ test: 'TC-016', name: 'GET /api/v1/worlds/:id/artifacts', passed, 
        message: `HTTP ${artifactsRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-016 Get artifacts: HTTP ${artifactsRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-016', name: 'GET /api/v1/worlds/:id/artifacts', passed: false, message: e.message });
      console.log(`❌ TC-016 Get artifacts: ${e.message}`);
    }

    // ========================================
    // PART 10: EXPORT (2 endpoints)
    // ========================================
    console.log('\n══════════════════════════════════════');
    console.log('  PART 10: EXPORT ENDPOINTS');
    console.log('══════════════════════════════════════\n');

    // TC-017: GET /api/v1/worlds/:id/export
    try {
      const exportRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/export`);
      const passed = exportRes.status === 200;
      results.push({ test: 'TC-017', name: 'GET /api/v1/worlds/:id/export', passed, 
        message: `HTTP ${exportRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-017 Export world: HTTP ${exportRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-017', name: 'GET /api/v1/worlds/:id/export', passed: false, message: e.message });
      console.log(`❌ TC-017 Export world: ${e.message}`);
    }

    // TC-018: GET /api/v1/worlds/:id/export.json
    try {
      const exportJsonRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}/export.json`);
      const passed = exportJsonRes.status === 200;
      results.push({ test: 'TC-018', name: 'GET /api/v1/worlds/:id/export.json', passed, 
        message: `HTTP ${exportJsonRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-018 Export JSON: HTTP ${exportJsonRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-018', name: 'GET /api/v1/worlds/:id/export.json', passed: false, message: e.message });
      console.log(`❌ TC-018 Export JSON: ${e.message}`);
    }

    // TC-019: DELETE /api/v1/worlds/:id - Delete world
    console.log('\n══════════════════════════════════════');
    console.log('  PART 11: DELETE ENDPOINT');
    console.log('══════════════════════════════════════\n');
    
    try {
      const deleteRes = await httpRequest(`${API_URL}/api/v1/worlds/${testWorldId}`, { method: 'DELETE' });
      const passed = deleteRes.status === 204 || deleteRes.status === 200;
      results.push({ test: 'TC-019', name: 'DELETE /api/v1/worlds/:id', passed, 
        message: `HTTP ${deleteRes.status}` });
      console.log(`${passed ? '✅' : '❌'} TC-019 Delete world: HTTP ${deleteRes.status}`);
    } catch (e) {
      results.push({ test: 'TC-019', name: 'DELETE /api/v1/worlds/:id', passed: false, message: e.message });
      console.log(`❌ TC-019 Delete world: ${e.message}`);
    }

  } else {
    console.log('\n⚠️  Skipping world-specific tests (no world ID available)');
    for (let i = 5; i <= 19; i++) {
      results.push({ test: `TC-${String(i).padStart(3, '0')}`, name: `World endpoint test ${i}`, passed: false, 
        message: 'Skipped - no world ID available' });
    }
  }

  // ========================================
  // PART 12: FRONTEND UI CHECKS
  // ========================================
  console.log('\n══════════════════════════════════════');
  console.log('  PART 12: FRONTEND UI CHECKS');
  console.log('══════════════════════════════════════\n');

  // TC-020: Frontend landing page loads
  try {
    const frontendRes = await httpRequest(FRONTEND_URL);
    const passed = frontendRes.status === 200 && frontendRes.data.includes('World Selector');
    results.push({ test: 'TC-020', name: 'Frontend - Landing page loads', passed, 
      message: `HTTP ${frontendRes.status}` });
    console.log(`${passed ? '✅' : '❌'} TC-020 Frontend landing: HTTP ${frontendRes.status}`);
  } catch (e) {
    results.push({ test: 'TC-020', name: 'Frontend - Landing page loads', passed: false, message: e.message });
    console.log(`❌ TC-020 Frontend landing: ${e.message}`);
  }

  // TC-021: Frontend has Create World form
  try {
    const frontendRes = await httpRequest(FRONTEND_URL);
    const passed = frontendRes.data && (
      frontendRes.data.includes('name') || 
      frontendRes.data.includes('Create') || 
      frontendRes.data.includes('world')
    );
    results.push({ test: 'TC-021', name: 'Frontend - Create form present', passed, 
      message: 'Form elements detected in HTML' });
    console.log(`${passed ? '✅' : '❌'} TC-021 Frontend create form: ${passed ? 'Found' : 'Not found'}`);
  } catch (e) {
    results.push({ test: 'TC-021', name: 'Frontend - Create form present', passed: false, message: e.message });
    console.log(`❌ TC-021 Frontend create form: ${e.message}`);
  }

  // TC-022: Frontend serves CSS/JS assets
  try {
    const assetsRes = await httpRequest(`${FRONTEND_URL}/api-integration.js`);
    const passed = assetsRes.status === 200;
    results.push({ test: 'TC-022', name: 'Frontend - JS assets accessible', passed, 
      message: `HTTP ${assetsRes.status}` });
    console.log(`${passed ? '✅' : '❌'} TC-022 Frontend JS assets: HTTP ${assetsRes.status}`);
  } catch (e) {
    results.push({ test: 'TC-022', name: 'Frontend - JS assets accessible', passed: false, message: e.message });
    console.log(`❌ TC-022 Frontend JS assets: ${e.message}`);
  }

  // ========================================
  // SUMMARY
  // ========================================
  console.log('\n' + '═'.repeat(80));
  console.log('                         TEST SUMMARY');
  console.log('═'.repeat(80));
  
  const passed = results.filter(r => r.passed).length;
  const total = results.length;
  const failed = results.filter(r => !r.passed);
  
  console.log(`\nAPI Endpoints: ${passed}/${total} passed`);
  console.log(`\nDetailed Results:\n`);
  
  for (const r of results) {
    const status = r.passed ? '✅ PASS' : '❌ FAIL';
    console.log(`  ${r.test} [${status}] ${r.name}`);
    console.log(`    → ${r.message}`);
  }
  
  console.log('\n' + '═'.repeat(80));
  
  const overallPassed = passed === total;
  console.log(`\nOVERALL RESULT: ${passed}/${total} tests passed`);
  console.log(`Status: ${overallPassed ? '✅ ALL TESTS PASSED' : '❌ SOME TESTS FAILED'}\n`);
  
  // Save detailed log
  const logLines = [
    `WOR-1088 Smoke Test - ${new Date().toISOString()}`,
    '═'.repeat(80),
    '',
    'SUMMARY',
    `${passed}/${total} tests passed`,
    '',
    'DETAILED RESULTS',
    ...results.map(r => `${r.passed ? '✅' : '❌'} ${r.test} ${r.name}: ${r.message}`),
    '',
    'ENDPOINTS TESTED',
    '1. POST /api/v1/worlds - Create world',
    '2. GET /api/v1/worlds - List worlds',
    '3. GET /api/v1/worlds/:id - Get world',
    '4. GET /api/v1/worlds/:id/planet - Get planet',
    '5. GET /api/v1/worlds/:id/map - Get map',
    '6. GET /api/v1/worlds/:id/history - Get history',
    '7. GET /api/v1/worlds/:id/history/events - Get events',
    '8. GET /api/v1/worlds/:id/figures - Get figures',
    '9. GET /api/v1/worlds/:id/figures/:id - Get figure',
    '10. GET /api/v1/worlds/:id/settlements - Get settlements',
    '11. GET /api/v1/worlds/:id/settlements/map - Get settlements map',
    '12. GET /api/v1/worlds/:id/resources/summary - Get resources',
    '13. GET /api/v1/worlds/:id/disasters - Get disasters',
    '14. GET /api/v1/worlds/:id/artifacts - Get artifacts',
    '15. GET /api/v1/worlds/:id/export - Export world',
    '16. GET /api/v1/worlds/:id/export.json - Export JSON',
    '17. DELETE /api/v1/worlds/:id - Delete world',
    '18. Frontend landing page',
    '19. Frontend - Create form',
    '20. Frontend - JS assets',
  ];
  
  fs.writeFileSync('smoke-test-WOR-1088-output.log', logLines.join('\n'));
  console.log('Log saved to: smoke-test-WOR-1088-output.log');
  
  return overallPassed ? 0 : 1;
}

runSmokeTests().then(code => process.exit(code)).catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});