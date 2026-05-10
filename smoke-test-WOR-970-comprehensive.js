#!/usr/bin/env node
/**
 * WOR-970: Comprehensive Smoke Test - All 18 Endpoints + Frontend UI
 */

const http = require('http');

const BASE_URL = 'http://localhost:8765';
const API_URL = 'http://localhost:8080';

const results = [];
const consoleErrors = [];

function httpRequest(url, options = {}) {
  return new Promise((resolve, reject) => {
    const urlObj = new URL(url);
    const reqOptions = {
      hostname: urlObj.hostname,
      port: urlObj.port,
      path: urlObj.pathname + urlObj.search,
      method: options.method || 'GET',
      headers: options.headers || {}
    };
    
    const req = http.request(reqOptions, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          resolve({ status: res.statusCode, data: json, headers: res.headers });
        } catch {
          resolve({ status: res.statusCode, data: data, headers: res.headers });
        }
      });
    });
    
    req.on('error', reject);
    if (options.body) req.write(options.body);
    req.end();
  });
}

async function waitForWorldReady(worldId, maxAttempts = 10) {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}`);
      if (res.status === 200 && res.data?.data?.status === 'ready') {
        return true;
      }
    } catch {}
    await new Promise(r => setTimeout(r, 1000));
  }
  return false;
}

async function runComprehensiveTests() {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║    WOR-970: COMPREHENSIVE SMOKE TEST - ALL 18 ENDPOINTS  ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log(`Started: ${new Date().toISOString()}\n`);

  let worldId = null;
  
  // === BACKEND API TESTS ===
  console.log('══════════════════════════════════════════════');
  console.log('           BACKEND API TESTS (18 endpoints)');
  console.log('══════════════════════════════════════════════\n');

  // Test 1: Health
  try {
    const res = await httpRequest(`${API_URL}/health`);
    const passed = res.status === 200;
    results.push({ test: 'API-001', name: 'GET /health', passed, message: `HTTP ${res.status}` });
    console.log(`${passed ? '✅' : '❌'} GET /health: HTTP ${res.status}`);
  } catch (e) {
    results.push({ test: 'API-001', name: 'GET /health', passed: false, message: e.message });
    console.log(`❌ GET /health: ${e.message}`);
  }

  // Test 2: Create World
  try {
    const res = await httpRequest(`${API_URL}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'WOR-970 Comprehensive Test' })
    });
    if (res.status === 201) {
      worldId = res.data?.data?.id;
      results.push({ test: 'API-002', name: 'POST /api/v1/worlds', passed: !!worldId, message: `HTTP 201, World ID: ${worldId}` });
      console.log(`✅ POST /api/v1/worlds: HTTP 201, World ID: ${worldId}`);
    } else {
      results.push({ test: 'API-002', name: 'POST /api/v1/worlds', passed: false, message: `HTTP ${res.status}` });
      console.log(`❌ POST /api/v1/worlds: HTTP ${res.status}`);
    }
  } catch (e) {
    results.push({ test: 'API-002', name: 'POST /api/v1/worlds', passed: false, message: e.message });
    console.log(`❌ POST /api/v1/worlds: ${e.message}`);
  }

  // Test 3: List Worlds
  try {
    const res = await httpRequest(`${API_URL}/api/v1/worlds`);
    const passed = res.status === 200;
    results.push({ test: 'API-003', name: 'GET /api/v1/worlds', passed, message: `HTTP ${res.status}` });
    console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds: HTTP ${res.status}`);
  } catch (e) {
    results.push({ test: 'API-003', name: 'GET /api/v1/worlds', passed: false, message: e.message });
    console.log(`❌ GET /api/v1/worlds: ${e.message}`);
  }

  // Wait for world to be ready
  if (worldId) {
    console.log('\n⏳ Waiting for world to be ready...');
    const ready = await waitForWorldReady(worldId);
    console.log(`${ready ? '✅' : '⚠️'} World ready status: ${ready ? 'ready' : 'timeout (may still work)'}\n`);
    
    // Test 4: Get World by ID
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}`);
      const passed = res.status === 200;
      results.push({ test: 'API-004', name: 'GET /api/v1/worlds/:id', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/${worldId.slice(0,8)}...: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-004', name: 'GET /api/v1/worlds/:id', passed: false, message: e.message });
    }

    // Test 5: Get Planet
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/planet`);
      const passed = res.status === 200;
      results.push({ test: 'API-005', name: 'GET /api/v1/worlds/:id/planet', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/planet: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-005', name: 'GET /api/v1/worlds/:id/planet', passed: false, message: e.message });
    }

    // Test 6: Get Map
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/map`);
      const passed = res.status === 200;
      results.push({ test: 'API-006', name: 'GET /api/v1/worlds/:id/map', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/map: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-006', name: 'GET /api/v1/worlds/:id/map', passed: false, message: e.message });
    }

    // Test 7: Get History
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/history`);
      const passed = res.status === 200;
      results.push({ test: 'API-007', name: 'GET /api/v1/worlds/:id/history', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/history: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-007', name: 'GET /api/v1/worlds/:id/history', passed: false, message: e.message });
    }

    // Test 8: Get History Events
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/history/events`);
      const passed = res.status === 200;
      results.push({ test: 'API-008', name: 'GET /api/v1/worlds/:id/history/events', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/history/events: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-008', name: 'GET /api/v1/worlds/:id/history/events', passed: false, message: e.message });
    }

    // Test 9: Get Figures
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/figures`);
      const passed = res.status === 200;
      results.push({ test: 'API-009', name: 'GET /api/v1/worlds/:id/figures', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/figures: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-009', name: 'GET /api/v1/worlds/:id/figures', passed: false, message: e.message });
    }

    // Test 10: Get Figure by ID (use valid UUID format for nonexistent figure)
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/figures/00000000-0000-0000-0000-000000000000`);
      // 404 is the expected behavior for nonexistent figure with valid UUID
      const passed = res.status === 404;
      results.push({ test: 'API-010', name: 'GET /api/v1/worlds/:id/figures/:figure_id', passed, message: `HTTP ${res.status} (404 expected for nonexistent)` });
      console.log(`${passed ? '✅' : '⚠️'} GET /api/v1/worlds/:id/figures/:figure_id: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-010', name: 'GET /api/v1/worlds/:id/figures/:figure_id', passed: false, message: e.message });
    }

    // Test 11: Get Settlements
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/settlements`);
      const passed = res.status === 200;
      results.push({ test: 'API-011', name: 'GET /api/v1/worlds/:id/settlements', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/settlements: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-011', name: 'GET /api/v1/worlds/:id/settlements', passed: false, message: e.message });
    }

    // Test 12: Get Settlements Map
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/settlements/map`);
      const passed = res.status === 200;
      results.push({ test: 'API-012', name: 'GET /api/v1/worlds/:id/settlements/map', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/settlements/map: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-012', name: 'GET /api/v1/worlds/:id/settlements/map', passed: false, message: e.message });
    }

    // Test 13: Get Resources Summary
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/resources/summary`);
      const passed = res.status === 200;
      results.push({ test: 'API-013', name: 'GET /api/v1/worlds/:id/resources/summary', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/resources/summary: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-013', name: 'GET /api/v1/worlds/:id/resources/summary', passed: false, message: e.message });
    }

    // Test 14: Get Disasters
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/disasters`);
      const passed = res.status === 200;
      results.push({ test: 'API-014', name: 'GET /api/v1/worlds/:id/disasters', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/disasters: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-014', name: 'GET /api/v1/worlds/:id/disasters', passed: false, message: e.message });
    }

    // Test 15: Get Artifacts
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/artifacts`);
      const passed = res.status === 200;
      results.push({ test: 'API-015', name: 'GET /api/v1/worlds/:id/artifacts', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/artifacts: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-015', name: 'GET /api/v1/worlds/:id/artifacts', passed: false, message: e.message });
    }

    // Test 16: Get Export
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/export`);
      const passed = res.status === 200;
      results.push({ test: 'API-016', name: 'GET /api/v1/worlds/:id/export', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/export: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-016', name: 'GET /api/v1/worlds/:id/export', passed: false, message: e.message });
    }

    // Test 17: Get Export JSON
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/export.json`);
      const passed = res.status === 200;
      results.push({ test: 'API-017', name: 'GET /api/v1/worlds/:id/export.json', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} GET /api/v1/worlds/:id/export.json: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-017', name: 'GET /api/v1/worlds/:id/export.json', passed: false, message: e.message });
    }

    // Test 18: Delete World (cleanup)
    try {
      const res = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}`, { method: 'DELETE' });
      const passed = res.status === 204;
      results.push({ test: 'API-018', name: 'DELETE /api/v1/worlds/:id', passed, message: `HTTP ${res.status}` });
      console.log(`${passed ? '✅' : '❌'} DELETE /api/v1/worlds/:id: HTTP ${res.status}`);
    } catch (e) {
      results.push({ test: 'API-018', name: 'DELETE /api/v1/worlds/:id', passed: false, message: e.message });
    }
  }

  // === FRONTEND UI TESTS ===
  console.log('\n══════════════════════════════════════════════');
  console.log('              FRONTEND UI TESTS');
  console.log('══════════════════════════════════════════════\n');

  // Test 19: Frontend Landing
  try {
    const res = await httpRequest(BASE_URL);
    const passed = res.status === 200;
    results.push({ test: 'UI-001', name: 'Frontend landing page', passed, message: `HTTP ${res.status}` });
    console.log(`${passed ? '✅' : '❌'} Frontend landing page: HTTP ${res.status}`);
  } catch (e) {
    results.push({ test: 'UI-001', name: 'Frontend landing page', passed: false, message: e.message });
    console.log(`❌ Frontend landing page: ${e.message}`);
  }

  // Test 20: Frontend JS loads
  try {
    const res = await httpRequest(`${BASE_URL}/dist/app.js`);
    const passed = res.status === 200;
    results.push({ test: 'UI-002', name: 'Frontend app.js loads', passed, message: `HTTP ${res.status}` });
    console.log(`${passed ? '✅' : '❌'} Frontend app.js: HTTP ${res.status}`);
  } catch (e) {
    results.push({ test: 'UI-002', name: 'Frontend app.js loads', passed: false, message: e.message });
    console.log(`❌ Frontend app.js: ${e.message}`);
  }

  // Print summary
  console.log('\n' + '═'.repeat(62));
  console.log('                    TEST SUMMARY');
  console.log('═'.repeat(62));
  
  const passed = results.filter(r => r.passed).length;
  const total = results.length;
  
  const apiPassed = results.filter(r => r.test.startsWith('API') && r.passed).length;
  const apiTotal = results.filter(r => r.test.startsWith('API')).length;
  const uiPassed = results.filter(r => r.test.startsWith('UI') && r.passed).length;
  const uiTotal = results.filter(r => r.test.startsWith('UI')).length;
  
  console.log(`\nAPI Endpoints: ${apiPassed}/${apiTotal} passed`);
  console.log(`Frontend Tests: ${uiPassed}/${uiTotal} passed`);
  console.log(`Total: ${passed}/${total} passed\n`);
  
  for (const r of results) {
    const status = r.passed ? '✅ PASS' : '❌ FAIL';
    console.log(`${r.test} [${status}] ${r.name}`);
    console.log(`  → ${r.message}`);
  }
  
  console.log('\n' + '═'.repeat(62));
  
  const overallPassed = passed === total;
  console.log(`\nOverall: ${passed}/${total} tests passed`);
  console.log(`Status: ${overallPassed ? '✅ PASS' : '❌ FAIL'}\n`);
  
  // Save log
  const logOutput = [
    `WOR-970 Comprehensive Smoke Test - ${new Date().toISOString()}`,
    `=====================================================`,
    `API Endpoints: ${apiPassed}/${apiTotal} passed`,
    `Frontend Tests: ${uiPassed}/${uiTotal} passed`,
    `Total: ${passed}/${total} passed`,
    ``,
    `Results:`,
    ...results.map(r => `${r.passed ? '✅' : '❌'} ${r.test} ${r.name}: ${r.message}`)
  ].join('\n');
  
  require('fs').writeFileSync('smoke-test-WOR-970-output.log', logOutput);
  
  return overallPassed ? 0 : 1;
}

runComprehensiveTests().then(code => process.exit(code)).catch(e => {
  console.error(e);
  process.exit(1);
});
