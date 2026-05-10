#!/usr/bin/env node
/**
 * WOR-970: Smoke Test - World Factory Core Functionality
 */

const http = require('http');

const BASE_URL = 'http://localhost:8765';
const API_URL = 'http://localhost:8080';

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
          resolve({ status: res.statusCode, data: json });
        } catch {
          resolve({ status: res.statusCode, data: data });
        }
      });
    });
    
    req.on('error', reject);
    if (options.body) req.write(options.body);
    req.end();
  });
}

async function runSmokeTests() {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║         WORLD FACTORY SMOKE TEST - WOR-970            ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log(`Started: ${new Date().toISOString()}\n`);

  // Test 1: Backend health
  console.log('=== Testing Backend Health ===');
  try {
    const health = await httpRequest(`${API_URL}/health`);
    const passed = health.status === 200;
    results.push({ test: 'TC-001', name: 'Backend health check', passed, message: `HTTP ${health.status}` });
    console.log(`${passed ? '✅' : '❌'} Backend health: HTTP ${health.status}`);
  } catch (e) {
    results.push({ test: 'TC-001', name: 'Backend health check', passed: false, message: e.message });
    console.log(`❌ Backend health: ${e.message}`);
  }

  // Test 2: Frontend landing page
  console.log('\n=== Testing Frontend ===');
  try {
    const frontend = await httpRequest(BASE_URL);
    const passed = frontend.status === 200;
    results.push({ test: 'TC-002', name: 'Frontend loads', passed, message: `HTTP ${frontend.status}` });
    console.log(`${passed ? '✅' : '❌'} Frontend landing: HTTP ${frontend.status}`);
  } catch (e) {
    results.push({ test: 'TC-002', name: 'Frontend loads', passed: false, message: e.message });
    console.log(`❌ Frontend landing: ${e.message}`);
  }

  // Test 3: World creation
  console.log('\n=== Testing World API ===');
  let worldId = null;
  try {
    const createRes = await httpRequest(`${API_URL}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Test World for WOR-970' })
    });
    
    if (createRes.status === 201) {
      // Handle both data.id and data.world_id formats
      worldId = createRes.data?.data?.id || createRes.data?.id || createRes.data?.world_id;
      results.push({ test: 'TC-003', name: 'Create world', passed: !!worldId, message: `World ID: ${worldId}` });
      console.log(`✅ Create world: HTTP 201, World ID: ${worldId}`);
    } else {
      results.push({ test: 'TC-003', name: 'Create world', passed: false, message: `HTTP ${createRes.status}` });
      console.log(`❌ Create world: HTTP ${createRes.status}`);
    }
  } catch (e) {
    results.push({ test: 'TC-003', name: 'Create world', passed: false, message: e.message });
    console.log(`❌ Create world: ${e.message}`);
  }

  // Test 4: Get worlds list
  try {
    const listRes = await httpRequest(`${API_URL}/api/v1/worlds`);
    const passed = listRes.status === 200;
    results.push({ test: 'TC-004', name: 'List worlds', passed, message: `HTTP ${listRes.status}` });
    console.log(`${passed ? '✅' : '❌'} List worlds: HTTP ${listRes.status}`);
  } catch (e) {
    results.push({ test: 'TC-004', name: 'List worlds', passed: false, message: e.message });
    console.log(`❌ List worlds: ${e.message}`);
  }

  // Test 5-11: Get world by ID and related resources
  if (worldId) {
    try {
      // Wait for world to be ready
      console.log('\n⏳ Waiting for world to be ready...');
      await new Promise(r => setTimeout(r, 2000));
      
      const getRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}`);
      const passed = getRes.status === 200;
      results.push({ test: 'TC-005', name: 'Get world by ID', passed, message: `HTTP ${getRes.status}` });
      console.log(`${passed ? '✅' : '❌'} Get world by ID: HTTP ${getRes.status}`);
      
      // Test 6: Get planet
      const planetRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/planet`);
      const planetPassed = planetRes.status === 200;
      results.push({ test: 'TC-006', name: 'Get planet', passed: planetPassed, message: `HTTP ${planetRes.status}` });
      console.log(`${planetPassed ? '✅' : '❌'} Get planet: HTTP ${planetRes.status}`);
      
      // Test 7: Get map
      const mapRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/map`);
      const mapPassed = mapRes.status === 200;
      results.push({ test: 'TC-007', name: 'Get map', passed: mapPassed, message: `HTTP ${mapRes.status}` });
      console.log(`${mapPassed ? '✅' : '❌'} Get map: HTTP ${mapRes.status}`);
      
      // Test 8: Get history
      const histRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/history`);
      const histPassed = histRes.status === 200;
      results.push({ test: 'TC-008', name: 'Get history', passed: histPassed, message: `HTTP ${histRes.status}` });
      console.log(`${histPassed ? '✅' : '❌'} Get history: HTTP ${histRes.status}`);
      
      // Test 9: Get settlements
      const settlementsRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/settlements`);
      const settlementsPassed = settlementsRes.status === 200;
      results.push({ test: 'TC-009', name: 'Get settlements', passed: settlementsPassed, message: `HTTP ${settlementsRes.status}` });
      console.log(`${settlementsPassed ? '✅' : '❌'} Get settlements: HTTP ${settlementsRes.status}`);
      
      // Test 10: Get resources
      const resourcesRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}/resources/summary`);
      const resourcesPassed = resourcesRes.status === 200;
      results.push({ test: 'TC-010', name: 'Get resources', passed: resourcesPassed, message: `HTTP ${resourcesRes.status}` });
      console.log(`${resourcesPassed ? '✅' : '❌'} Get resources: HTTP ${resourcesRes.status}`);
      
      // Test 11: Delete world
      const deleteRes = await httpRequest(`${API_URL}/api/v1/worlds/${worldId}`, { method: 'DELETE' });
      const deletePassed = deleteRes.status === 204;
      results.push({ test: 'TC-011', name: 'Delete world', passed: deletePassed, message: `HTTP ${deleteRes.status}` });
      console.log(`${deletePassed ? '✅' : '❌'} Delete world: HTTP ${deleteRes.status}`);
      
    } catch (e) {
      console.log(`❌ World operations error: ${e.message}`);
    }
  } else {
    console.log('\n⚠️  Skipping world-specific tests (no world ID)');
  }

  // Print summary
  console.log('\n' + '═'.repeat(62));
  console.log('                    TEST SUMMARY');
  console.log('═'.repeat(62));
  
  const passed = results.filter(r => r.passed).length;
  const total = results.length;
  
  console.log(`\nAPI Endpoints: ${passed}/${total} passed\n`);
  
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
  const logOutput = results.map(r => 
    `${r.passed ? '✅' : '❌'} ${r.test} ${r.name}: ${r.message}`
  ).join('\n');
  
  require('fs').writeFileSync('smoke-test-WOR-970-output.log', logOutput);
  
  return overallPassed ? 0 : 1;
}

runSmokeTests().then(code => process.exit(code)).catch(e => {
  console.error(e);
  process.exit(1);
});
