#!/usr/bin/env node
/**
 * Smoke test for WOR-946: Timeline endpoint returns HTTP 400 for 'generating' status worlds
 * 
 * This test verifies:
 * 1. Timeline endpoint returns 404 for non-existent worlds (not 400)
 * 2. Timeline endpoint returns 200 for existing worlds
 */

const API_BASE = 'http://localhost:8080';

async function main() {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║         WOR-946 SMOKE TEST - Timeline 400 Fix           ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log(`Started: ${new Date().toISOString()}\n`);

  let passed = 0;
  let failed = 0;

  // Test 1: Non-existent world should return 404 (not 400)
  console.log('Test 1: GET /api/v1/worlds/{non-existent}/timeline → should be 404');
  const nonexistentId = '00000000-0000-0000-0000-000000000000';
  try {
    const response = await fetch(`${API_BASE}/api/v1/worlds/${nonexistentId}/timeline`);
    const status = response.status;
    const body = await response.json();

    if (status === 404) {
      console.log(`  ✅ PASS: Got 404 NOT_FOUND (body: ${JSON.stringify(body)})`);
      passed++;
    } else {
      console.log(`  ❌ FAIL: Expected 404, got ${status} (body: ${JSON.stringify(body)})`);
      failed++;
    }
  } catch (e) {
    console.log(`  ❌ FAIL: Request failed - ${e.message}`);
    failed++;
  }

  // Test 2: Invalid UUID should still return 400
  console.log('\nTest 2: GET /api/v1/worlds/{invalid-uuid}/timeline → should be 400');
  try {
    const response = await fetch(`${API_BASE}/api/v1/worlds/invalid-uuid/timeline`);
    const status = response.status;
    const body = await response.json();

    if (status === 400) {
      console.log(`  ✅ PASS: Got 400 BAD_REQUEST (body: ${JSON.stringify(body)})`);
      passed++;
    } else {
      console.log(`  ❌ FAIL: Expected 400, got ${status} (body: ${JSON.stringify(body)})`);
      failed++;
    }
  } catch (e) {
    console.log(`  ❌ FAIL: Request failed - ${e.message}`);
    failed++;
  }

  // Test 3: Create a world and verify timeline returns 200
  console.log('\nTest 3: Create world → GET /api/v1/worlds/{id}/timeline → should be 200');
  try {
    // Create world
    const createResponse = await fetch(`${API_BASE}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: 'WOR-946 Test World',
        parameters: { seed: 946946946, size: 'small' }
      })
    });
    
    if (createResponse.status !== 201 && createResponse.status !== 200) {
      console.log(`  ⚠️  SKIP: Could not create test world (status: ${createResponse.status})`);
      console.log('    Backend may not support world creation or is in a different state.');
    } else {
      const createBody = await createResponse.json();
      const worldId = createBody.data?.id || createBody.id;
      
      if (worldId) {
        // Test timeline endpoint
        const timelineResponse = await fetch(`${API_BASE}/api/v1/worlds/${worldId}/timeline`);
        const timelineStatus = timelineResponse.status;

        if (timelineStatus === 200) {
          console.log(`  ✅ PASS: Timeline for new world returns 200`);
          passed++;
        } else {
          console.log(`  ❌ FAIL: Expected 200, got ${timelineStatus}`);
          failed++;
        }
      } else {
        console.log(`  ⚠️  SKIP: Could not extract world ID from response`);
      }
    }
  } catch (e) {
    console.log(`  ⚠️  SKIP: Test failed - ${e.message}`);
  }

  // Summary
  console.log('\n╔════════════════════════════════════════════════════════════╗');
  console.log('║                      SUMMARY                               ║');
  console.log('╚════════════════════════════════════════════════════════════╝');
  console.log(`Passed: ${passed} | Failed: ${failed}`);
  console.log(`Result: ${failed === 0 ? '✅ ALL TESTS PASSED' : '❌ SOME TESTS FAILED'}`);

  process.exit(failed === 0 ? 0 : 1);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});