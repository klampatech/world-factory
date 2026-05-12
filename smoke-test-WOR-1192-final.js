#!/usr/bin/env node
/**
 * WOR-1192: Smoke Test - Verify /worlds/:id/map route works
 * 
 * Issue: Dedicated /map route was returning 404
 * Fix: Changed to use current_exe() for static file resolution
 */

const API_URL = 'http://localhost:8082';

async function apiRequest(method, endpoint, body = null) {
  const url = `${API_URL}${endpoint}`;
  const options = { method, headers: { 'Content-Type': 'application/json' } };
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

  // 1. Health check
  console.log('[1] Health check...');
  const healthResp = await apiRequest('GET', '/health');
  if (healthResp.status !== 200) {
    console.log('ERROR: Health check failed:', healthResp.status);
    process.exit(1);
  }
  console.log('   ✅ API server is healthy');

  // 2. Create a world
  console.log('\n[2] Creating test world...');
  const createResp = await apiRequest('POST', '/api/v1/worlds', {
    name: 'WOR-1192 Final Test',
    seed: 12345,
    size: 'small'
  });
  
  if (createResp.status !== 201) {
    console.log('ERROR: Could not create test world:', createResp.status, createResp.data);
    process.exit(1);
  }
  
  const nestedData = createResp.data?.data || createResp.data;
  const worldId = nestedData?.id || nestedData?.uuid;
  if (!worldId) {
    console.log('ERROR: Could not extract world ID');
    process.exit(1);
  }
  console.log(`   World ID: ${worldId}`);

  // 3. Test static /worlds/:id/map route
  console.log('\n[3] Testing static /worlds/:id/map route...');
  const staticMapResp = await fetch(`${API_URL}/worlds/${worldId}/map`);
  const staticMapOk = staticMapResp.status === 200;
  console.log(`   Status: ${staticMapResp.status} ${staticMapOk ? '✅' : '❌'}`);
  
  if (staticMapOk) {
    const html = await staticMapResp.text();
    
    // Verify window.WORLD_ID is present
    const hasWorldId = html.includes('window.WORLD_ID');
    const worldIdFound = html.includes(worldId.replace('world:', ''));
    console.log(`   window.WORLD_ID injected: ${hasWorldId ? '✅' : '❌'}`);
    console.log(`   World ID in HTML: ${worldIdFound ? '✅' : '❌'}`);
    
    // Verify HTML is not empty and has canvas
    const hasCanvas = html.includes('<canvas');
    console.log(`   Canvas element present: ${hasCanvas ? '✅' : '❌'}`);
    
    if (!hasWorldId || !hasCanvas) {
      console.log('   ERROR: Response missing expected content');
      process.exit(1);
    }
  } else {
    console.log('   ERROR: Static route returned non-200');
    process.exit(1);
  }

  console.log('\n===========================================');
  console.log('✅ TEST PASSED - Fix verified');
  console.log('===========================================');
}

runTest().catch(e => {
  console.error('Test failed:', e);
  process.exit(1);
});
