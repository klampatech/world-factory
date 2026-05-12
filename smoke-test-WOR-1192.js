#!/usr/bin/env node
/**
 * WOR-1192: Smoke Test - Verify /worlds/:id/map route works
 * 
 * Issue: Dedicated /map route was returning 404 or not properly
 * loading the world ID. The backend was injecting `const WORLD_ID`
 * but map.html expected `window.WORLD_ID`.
 * 
 * Fix: Changed backend to inject `window.WORLD_ID = '...';` instead.
 */

const API_URL = 'http://localhost:3000';

async function apiRequest(method, endpoint, body = null) {
  const url = `${API_URL}${endpoint}`;
  const options = {
    method,
    headers: { 'Content-Type': 'application/json' },
  };
  if (body) options.body = JSON.stringify(body);
  try {
    const response = await fetch(url, options);
    let data = null;
    if (response.status !== 204) {
      try { data = await response.json(); } 
      catch (e) { data = await response.text(); }
    }
    return { status: response.status, data };
  } catch (e) {
    return { status: 0, error: e.message };
  }
}

async function runTest() {
  console.log('===========================================');
  console.log('WOR-1192: Dedicated /map route smoke test');
  console.log('===========================================\n');

  // 1. Create a world to get a valid world ID
  console.log('[1] Creating test world...');
  const createResp = await apiRequest('POST', '/api/v1/worlds', {
    name: 'WOR-1192 Test World',
    seed: 12345,
    size: 'medium'
  });
  
  if (createResp.status !== 201) {
    console.log('ERROR: Could not create test world:', createResp.status);
    process.exit(1);
  }
  
  // Handle nested response structure: { success: true, data: { id: "...", ... } }
  const nestedData = createResp.data?.data || createResp.data;
  const worldId = nestedData?.id || nestedData?.uuid;
  if (!worldId) {
    console.log('ERROR: Could not extract world ID from:', JSON.stringify(createResp.data).substring(0, 300));
    process.exit(1);
  }
  console.log(`   World ID: ${worldId}`);

  // 2. Test /api/v1/worlds/:id/map returns 200
  console.log('\n[2] Testing /api/v1/worlds/:id/map endpoint...');
  const mapApiResp = await apiRequest('GET', `/api/v1/worlds/${worldId}/map`);
  const mapApiOk = mapApiResp.status === 200;
  console.log(`   Status: ${mapApiResp.status} ${mapApiOk ? '✅' : '❌'}`);
  if (!mapApiOk) {
    console.log('   Response:', JSON.stringify(mapApiResp.data)?.substring(0, 200));
  }

  // 3. Test static /worlds/:id/map route (this was the bug)
  console.log('\n[3] Testing static /worlds/:id/map route...');
  const staticMapResp = await fetch(`${API_URL}/worlds/${worldId}/map`);
  const staticMapOk = staticMapResp.status === 200;
  console.log(`   Status: ${staticMapResp.status} ${staticMapOk ? '✅' : '❌'}`);
  
  if (staticMapOk) {
    const html = await staticMapResp.text();
    
    // Check for window.WORLD_ID injection (the fix)
    const hasWorldIdInjection = html.includes('window.WORLD_ID');
    console.log(`   World ID injected as window.WORLD_ID: ${hasWorldIdInjection ? '✅' : '❌'}`);
    
    // Check that the map page contains the canvas element
    const hasCanvas = html.includes('<canvas');
    console.log(`   Canvas present: ${hasCanvas ? '✅' : '❌'}`);
    
    if (!hasWorldIdInjection) {
      console.log('   ERROR: window.WORLD_ID not found in response');
      console.log('   Looking for WORLD_ID pattern:', html.includes('WORLD_ID') ? 'found' : 'not found');
    }
  } else {
    console.log('   ERROR: Static map route returned non-200 status');
  }

  console.log('\n===========================================');
  const passed = mapApiOk && staticMapOk;
  console.log(passed ? '✅ TEST PASSED' : '❌ TEST FAILED');
  console.log('===========================================');
  
  if (!passed) process.exit(1);
}

runTest().catch(e => {
  console.error('Test error:', e);
  process.exit(1);
});