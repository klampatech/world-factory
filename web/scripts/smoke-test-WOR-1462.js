/**
 * Smoke test for WOR-1462: Browser POST fails with ERR_CONNECTION_REFUSED
 * 
 * This test verifies that:
 * 1. The frontend proxy correctly handles POST requests
 * 2. Content-Type headers are properly forwarded
 * 3. API endpoints respond correctly
 * 
 * Run: node scripts/smoke-test-WOR-1462.js
 */

const http = require('http');

const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:8765';
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8082';

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function httpRequest(url, method, headers, body, retries = 3) {
  return new Promise((resolve, reject) => {
    const parsedUrl = new URL(url);
    const options = {
      hostname: parsedUrl.hostname,
      port: parsedUrl.port || 80,
      path: parsedUrl.pathname + parsedUrl.search,
      method: method,
      headers: headers
    };

    let attempts = 0;
    
    function attempt() {
      attempts++;
      const req = http.request(options, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try {
            resolve({ status: res.statusCode, headers: res.headers, body: JSON.parse(data) });
          } catch {
            resolve({ status: res.statusCode, headers: res.headers, body: data });
          }
        });
      });

      req.on('error', (err) => {
        if (attempts < retries && (err.code === 'ECONNREFUSED' || err.message.includes('socket hang up'))) {
          console.log(`   Retry ${attempts}/${retries} after socket error...`);
          sleep(1000).then(attempt);
        } else {
          reject(err);
        }
      });

      if (body) req.write(body);
      req.end();
    }
    
    attempt();
  });
}

async function runTests() {
  console.log('=== WOR-1462 Smoke Test ===\n');
  console.log(`Frontend: ${FRONTEND_URL}`);
  console.log(`Backend:  ${BACKEND_URL}\n`);

  let passed = 0;
  let failed = 0;

  // Test 1: Frontend health check
  try {
    const res = await httpRequest(`${FRONTEND_URL}/health`, 'GET', {});
    if (res.status === 200) {
      console.log('✅ Test 1: Frontend health check - PASSED');
      passed++;
    } else {
      console.log(`❌ Test 1: Frontend health check - FAILED (status: ${res.status})`);
      failed++;
    }
  } catch (e) {
    console.log(`❌ Test 1: Frontend health check - FAILED (${e.message})`);
    failed++;
  }

  // Test 2: Frontend proxy GET /api/v1/worlds
  try {
    const res = await httpRequest(`${FRONTEND_URL}/api/v1/worlds`, 'GET', {});
    if (res.status === 200 && res.body.success) {
      console.log('✅ Test 2: Frontend proxy GET /api/v1/worlds - PASSED');
      passed++;
    } else {
      console.log(`❌ Test 2: Frontend proxy GET /api/v1/worlds - FAILED (status: ${res.status})`);
      failed++;
    }
  } catch (e) {
    console.log(`❌ Test 2: Frontend proxy GET /api/v1/worlds - FAILED (${e.message})`);
    failed++;
  }

  // Test 3: Frontend proxy POST /api/v1/worlds (the bug scenario)
  try {
    const body = JSON.stringify({ name: 'Smoke Test World', width: 32, height: 32 });
    const res = await httpRequest(`${FRONTEND_URL}/api/v1/worlds`, 'POST', {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    }, body);
    
    if (res.status === 201 && res.body.success && res.body.data && res.body.data.id) {
      console.log('✅ Test 3: Frontend proxy POST /api/v1/worlds - PASSED (HTTP 201)');
      console.log(`   Created world: ${res.body.data.id}`);
      passed++;
    } else {
      console.log(`❌ Test 3: Frontend proxy POST /api/v1/worlds - FAILED (status: ${res.status})`);
      if (res.body && res.body.error) {
        console.log(`   Error: ${res.body.error}`);
      }
      failed++;
    }
  } catch (e) {
    console.log(`❌ Test 3: Frontend proxy POST /api/v1/worlds - FAILED (${e.message})`);
    if (e.message.includes('ECONNREFUSED')) {
      console.log('   ⚠️  This is the ERR_CONNECTION_REFUSED bug!');
    }
    failed++;
  }

  // Test 4: Backend directly (for comparison)
  try {
    const body = JSON.stringify({ name: 'Direct Backend Test', width: 32, height: 32 });
    const res = await httpRequest(`${BACKEND_URL}/api/v1/worlds`, 'POST', {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    }, body);
    
    if (res.status === 201) {
      console.log('✅ Test 4: Backend direct POST /api/v1/worlds - PASSED');
      passed++;
    } else {
      console.log(`❌ Test 4: Backend direct POST /api/v1/worlds - FAILED (status: ${res.status})`);
      failed++;
    }
  } catch (e) {
    console.log(`❌ Test 4: Backend direct POST /api/v1/worlds - FAILED (${e.message})`);
    failed++;
  }

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
  
  if (failed === 0) {
    console.log('\n✅ All tests passed! WOR-1462 is fixed.\n');
    process.exit(0);
  } else {
    console.log('\n❌ Some tests failed. Please check the issue.\n');
    process.exit(1);
  }
}

runTests().catch(e => {
  console.error('Test runner error:', e);
  process.exit(1);
});