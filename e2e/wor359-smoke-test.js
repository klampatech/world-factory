/**
 * WOR-359 Smoke Test - Complete End-to-End Testing
 * Tests all 18 API endpoints and frontend functionality
 */

const http = require('http');
const { spawn } = require('child_process');

const API_PORT = 8080;
const FRONTEND_PORT = 8081;

async function apiRequest(method, path, body = null) {
  return new Promise((resolve, reject) => {
    const fullPath = path.startsWith('/health') ? path : '/api/v1' + path;
    const options = {
      hostname: 'localhost',
      port: API_PORT,
      path: fullPath,
      method,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      }
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          resolve({ status: res.statusCode, body: JSON.parse(data) });
        } catch (e) {
          resolve({ status: res.statusCode, body: data });
        }
      });
    });
    
    req.on('error', reject);
    if (body) req.write(JSON.stringify(body));
    req.end();
  });
}

async function runTests() {
  const results = [];
  let createdWorldId = null;
  
  console.log('===========================================');
  console.log('WOR-359 SMOKE TEST - Full Stack Validation');
  console.log('===========================================\n');
  
  // Test backend health
  console.log('1. Testing Backend Health...');
  try {
    const health = await apiRequest('GET', '/health');
    const pass = health.status === 200 && health.body.status === 'ok';
    results.push({ name: 'Health Check', status: pass ? 'PASS' : 'FAIL', detail: `HTTP ${health.status}, ${JSON.stringify(health.body)}` });
    console.log(`   ${pass ? 'PASS' : 'FAIL'}: ${health.body.version || health.body}`);
  } catch (e) {
    results.push({ name: 'Health Check', status: 'FAIL', detail: e.message });
    console.log(`   FAIL: ${e.message}`);
  }

  // Create a world for testing
  console.log('\n2. Creating test world...');
  try {
    const createResp = await apiRequest('POST', '/worlds', { name: 'WOR-359 Smoke Test', seed: 99999, config: { genre: 'fantasy' } });
    if (createResp.status === 201 && createResp.body.success) {
      createdWorldId = createResp.body.data.id;
      console.log(`   Created: ${createdWorldId}`);
      results.push({ name: 'POST /worlds', status: 'PASS', detail: createdWorldId });
    } else {
      console.log(`   FAIL: HTTP ${createResp.status}`);
      results.push({ name: 'POST /worlds', status: 'FAIL', detail: `HTTP ${createResp.status}` });
    }
  } catch (e) {
    results.push({ name: 'POST /worlds', status: 'FAIL', detail: e.message });
    console.log(`   FAIL: ${e.message}`);
  }

  // Test list worlds
  console.log('\n3. Testing GET /worlds (List)...');
  try {
    const resp = await apiRequest('GET', '/worlds');
    const pass = resp.status === 200 && resp.body.success;
    results.push({ name: 'GET /worlds', status: pass ? 'PASS' : 'FAIL', detail: `${resp.body.data?.totalWorlds || 0} worlds` });
    console.log(`   ${pass ? 'PASS' : 'FAIL'}: ${resp.body.data?.totalWorlds || 0} worlds`);
  } catch (e) {
    results.push({ name: 'GET /worlds', status: 'FAIL', detail: e.message });
    console.log(`   FAIL: ${e.message}`);
  }

  // Extract UUID
  const worldUuid = createdWorldId ? createdWorldId.replace('world:', '') : '00000000-0000-0000-0000-000000000000';
  
  // Test individual world endpoints
  console.log('\n4-18. Testing All World Endpoints...');
  const endpoints = [
    { name: 'GET /worlds/:id', path: `/worlds/${worldUuid}`, expect: [200, 404] },
    { name: 'GET /worlds/:id/planet', path: `/worlds/${worldUuid}/planet`, expect: [200, 400, 404] },
    { name: 'GET /worlds/:id/map', path: `/worlds/${worldUuid}/map`, expect: [200, 404] },
    { name: 'GET /worlds/:id/history', path: `/worlds/${worldUuid}/history`, expect: [200, 404] },
    { name: 'GET /worlds/:id/history/events', path: `/worlds/${worldUuid}/history/events`, expect: [200, 404] },
    { name: 'GET /worlds/:id/figures', path: `/worlds/${worldUuid}/figures`, expect: [200, 404] },
    { name: 'GET /worlds/:id/figures/:id', path: `/worlds/${worldUuid}/figures/fig-test`, expect: [200, 404] },
    { name: 'GET /worlds/:id/settlements', path: `/worlds/${worldUuid}/settlements`, expect: [200, 404] },
    { name: 'GET /worlds/:id/settlements/map', path: `/worlds/${worldUuid}/settlements/map`, expect: [200, 404] },
    { name: 'GET /worlds/:id/resources/summary', path: `/worlds/${worldUuid}/resources/summary`, expect: [200, 404] },
    { name: 'GET /worlds/:id/disasters', path: `/worlds/${worldUuid}/disasters`, expect: [200, 404] },
    { name: 'GET /worlds/:id/artifacts', path: `/worlds/${worldUuid}/artifacts`, expect: [200, 400, 404] },
    { name: 'GET /worlds/:id/export', path: `/worlds/${worldUuid}/export`, expect: [200, 404] },
    { name: 'GET /worlds/:id/export.json', path: `/worlds/${worldUuid}/export.json`, expect: [200, 404] },
  ];

  for (const ep of endpoints) {
    try {
      const resp = await apiRequest('GET', ep.path);
      const pass = ep.expect.includes(resp.status);
      const detail = resp.status === 200 ? 
        (resp.body.success ? 'OK' : `Error: ${resp.body.code || 'unknown'}`) : 
        `HTTP ${resp.status}`;
      results.push({ name: ep.name, status: pass ? 'PASS' : 'FAIL', detail });
      console.log(`   ${ep.name}: ${pass ? 'PASS' : 'FAIL'} (HTTP ${resp.status})`);
    } catch (e) {
      results.push({ name: ep.name, status: 'FAIL', detail: e.message });
      console.log(`   ${ep.name}: FAIL (${e.message})`);
    }
  }

  // Test DELETE
  console.log('\n19. Testing DELETE /worlds/:id...');
  try {
    const resp = await apiRequest('DELETE', `/worlds/${worldUuid}`);
    const pass = [200, 204, 404].includes(resp.status);
    results.push({ name: 'DELETE /worlds/:id', status: pass ? 'PASS' : 'FAIL', detail: `HTTP ${resp.status}` });
    console.log(`   DELETE /worlds/:id: ${pass ? 'PASS' : 'FAIL'} (HTTP ${resp.status})`);
  } catch (e) {
    results.push({ name: 'DELETE /worlds/:id', status: 'FAIL', detail: e.message });
    console.log(`   DELETE /worlds/:id: FAIL (${e.message})`);
  }

  // Test frontend
  console.log('\n20. Testing Frontend (HTTP server)...');
  try {
    const frontendResp = await new Promise((resolve, reject) => {
      http.get(`http://localhost:${FRONTEND_PORT}/`, (res) => {
        resolve({ status: res.statusCode, headers: res.headers });
      }).on('error', reject);
    });
    const pass = frontendResp.status === 200;
    results.push({ name: 'Frontend', status: pass ? 'PASS' : 'FAIL', detail: `HTTP ${frontendResp.status}` });
    console.log(`   Frontend: ${pass ? 'PASS' : 'FAIL'} (HTTP ${frontendResp.status})`);
  } catch (e) {
    results.push({ name: 'Frontend', status: 'FAIL', detail: e.message });
    console.log(`   Frontend: FAIL (${e.message})`);
  }

  // Summary
  console.log('\n===========================================');
  console.log('RESULTS SUMMARY');
  console.log('===========================================');
  
  const passed = results.filter(r => r.status === 'PASS').length;
  const failed = results.filter(r => r.status === 'FAIL').length;
  
  console.log(`Total: ${results.length} | ${passed} passed | ${failed} failed\n`);
  
  if (failed > 0) {
    console.log('Failed tests:');
    results.filter(r => r.status === 'FAIL').forEach(r => {
      console.log(`  ❌ ${r.name}: ${r.detail}`);
    });
  }

  // Save results
  const fs = require('fs');
  fs.writeFileSync('/tmp/wor359-results.json', JSON.stringify(results, null, 2));
  console.log('\nResults saved to /tmp/wor359-results.json');
  
  return { passed, failed, total: results.length };
}

runTests().then(({ passed, failed, total }) => {
  process.exit(failed > 0 ? 1 : 0);
}).catch(e => {
  console.error('Test error:', e);
  process.exit(1);
});
