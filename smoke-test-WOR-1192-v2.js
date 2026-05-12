#!/usr/bin/env node
/**
 * WOR-1192: Smoke Test v2 - Verify /worlds/:id/map route works with fix
 * 
 * Issue: The static_pages.rs was using std::env::current_dir() to find
 * web/static/ files, which fails when the binary runs from a different
 * directory than the source.
 * 
 * Fix: Changed to use std::env::current_exe() parent directory instead.
 */

const API_URL = 'http://localhost:3000';

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
  console.log('WOR-1192 v2: /map route fix verification');
  console.log('===========================================\n');

  // Test 1: Create a world
  console.log('[1] Creating test world...');
  const createResp = await apiRequest('POST', '/api/v1/worlds', {
    name: 'WOR-1192 v2 Test World',
    seed: 99999,
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

  // Test 2: Static /worlds/:id/map route
  console.log('\n[2] Testing static /worlds/:id/map route...');
  const staticMapResp = await fetch(`${API_URL}/worlds/${worldId}/map`);
  const staticMapOk = staticMapResp.status === 200;
  console.log(`   Status: ${staticMapResp.status} ${staticMapOk ? '✅' : '❌'}`);
  
  if (staticMapOk) {
    const html = await staticMapResp.text();
    
    // Verify window.WORLD_ID is present
    const hasWorldId = html.includes('window.WORLD_ID');
    console.log(`   window.WORLD_ID injected: ${hasWorldId ? '✅' : '❌'}`);
    
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
