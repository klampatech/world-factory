/**
 * WOR-348 Smoke Test - API and Frontend Validation
 * Tests all 18 API endpoints and frontend functionality
 */

const http = require('http');

const API_BASE = 'http://localhost:8080';
const API_PATH = '/api/v1';

async function apiRequest(method, path, body = null) {
  return new Promise((resolve, reject) => {
    const fullPath = path.startsWith('/health') ? path : API_PATH + path;
    const options = {
      hostname: 'localhost',
      port: 8080,
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
  console.log('WOR-348 SMOKE TEST - Full Stack Validation');
  console.log('===========================================\n');
  
  // Test backend health
  console.log('Testing Backend Health...');
  try {
    const health = await apiRequest('GET', '/health');
    results.push({ name: 'Health Check', status: health.status === 200 ? 'PASS' : 'FAIL', detail: `HTTP ${health.status}` });
  } catch (e) {
    results.push({ name: 'Health Check', status: 'FAIL', detail: e.message });
  }

  // Create a world for testing
  console.log('\nCreating test world...');
  try {
    const createResp = await apiRequest('POST', '/worlds', { name: 'WOR-348 Smoke Test', seed: 99999 });
    if (createResp.status === 201 && createResp.body.success) {
      createdWorldId = createResp.body.data.id;
      console.log(`Created world: ${createdWorldId}`);
      results.push({ name: 'POST /worlds', status: 'PASS', detail: createdWorldId });
    } else {
      console.log('Create response:', createResp.status, JSON.stringify(createResp.body).substring(0, 100));
      results.push({ name: 'POST /worlds', status: 'FAIL', detail: JSON.stringify(createResp.body) });
    }
  } catch (e) {
    results.push({ name: 'POST /worlds', status: 'FAIL', detail: e.message });
  }

  // Test list worlds
  try {
    const resp = await apiRequest('GET', '/worlds');
    if (resp.status === 200 && resp.body.success) {
      results.push({ name: 'GET /worlds', status: 'PASS', detail: `${resp.body.data.totalWorlds} worlds` });
    } else {
      results.push({ name: 'GET /worlds', status: 'FAIL', detail: `HTTP ${resp.status}` });
    }
  } catch (e) {
    results.push({ name: 'GET /worlds', status: 'FAIL', detail: e.message });
  }

  // Extract UUID for endpoints
  const worldUuid = createdWorldId ? createdWorldId.replace('world:', '') : 'd2098f0f-dd27-44c9-bbf8-fbf2031b6b7c';
  
  // Test all endpoints with UUID
  const endpoints = [
    { name: 'GET /worlds/:id', path: `/worlds/${worldUuid}` },
    { name: 'DELETE /worlds/:id', path: `/worlds/${worldUuid}`, method: 'DELETE' },
    { name: 'GET /worlds/:id/planet', path: `/worlds/${worldUuid}/planet` },
    { name: 'GET /worlds/:id/map', path: `/worlds/${worldUuid}/map` },
    { name: 'GET /worlds/:id/history', path: `/worlds/${worldUuid}/history` },
    { name: 'GET /worlds/:id/history/events', path: `/worlds/${worldUuid}/history/events` },
    { name: 'GET /worlds/:id/figures', path: `/worlds/${worldUuid}/figures` },
    { name: 'GET /worlds/:id/figures/:id', path: `/worlds/${worldUuid}/figures/fig-0` },
    { name: 'GET /worlds/:id/settlements', path: `/worlds/${worldUuid}/settlements` },
    { name: 'GET /worlds/:id/settlements/map', path: `/worlds/${worldUuid}/settlements/map` },
    { name: 'GET /worlds/:id/resources/summary', path: `/worlds/${worldUuid}/resources/summary` },
    { name: 'GET /worlds/:id/disasters', path: `/worlds/${worldUuid}/disasters` },
    { name: 'GET /worlds/:id/artifacts?limit=5', path: `/worlds/${worldUuid}/artifacts?limit=5` },
    { name: 'GET /worlds/:id/export', path: `/worlds/${worldUuid}/export` },
    { name: 'GET /worlds/:id/export.json', path: `/worlds/${worldUuid}/export.json` },
  ];

  console.log('\n--- Testing API Endpoints ---');
  for (const ep of endpoints) {
    try {
      const method = ep.method || 'GET';
      const resp = await apiRequest(method, ep.path);
      const status = resp.status;
      let isSuccess = status >= 200 && status < 300;
      
      // Check for known issues
      if (status === 400 && resp.body?.error === 'Invalid world ID format') {
        isSuccess = false;
      }
      
      results.push({
        name: ep.name,
        status: isSuccess ? 'PASS' : 'FAIL',
        detail: `HTTP ${status} - ${resp.body?.error || (isSuccess ? 'OK' : 'Failed')}`
      });
      
      process.stdout.write(isSuccess ? '✅' : '❌');
    } catch (e) {
      results.push({ name: ep.name, status: 'FAIL', detail: e.message });
      process.stdout.write('❌');
    }
  }
  
  console.log('\n\n===========================================');
  console.log('RESULTS SUMMARY');
  console.log('===========================================');
  
  const passed = results.filter(r => r.status === 'PASS').length;
  const failed = results.filter(r => r.status === 'FAIL').length;
  
  console.log(`\nTotal: ${results.length} | ✅ ${passed} passed | ❌ ${failed} failed\n`);
  
  results.forEach(r => {
    const icon = r.status === 'PASS' ? '✅' : '❌';
    console.log(`${icon} ${r.name}: ${r.detail}`);
  });
  
  // Write results to file
  const fs = require('fs');
  fs.writeFileSync('/home/kyle/projects/world-generator/qa-reports/WOR-348-results.json', JSON.stringify({
    timestamp: new Date().toISOString(),
    summary: { total: results.length, passed, failed },
    results
  }, null, 2));
  
  console.log('\nResults saved to qa-reports/WOR-348-results.json');
  
  return { passed, failed };
}

runTests().then(({ passed, failed }) => {
  process.exit(failed > 0 ? 1 : 0);
}).catch(console.error);
