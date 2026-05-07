# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: e2e/smoke-test-all-endpoints.spec.ts >> WOR-348: All 18 API Endpoints >> 11-12. GET /api/v1/worlds/:id/settlements and /settlements/map
- Location: e2e/smoke-test-all-endpoints.spec.ts:76:7

# Error details

```
TypeError: Cannot read properties of undefined (reading 'replace')
```

# Test source

```ts
  1   | import { test, expect, request } from '@playwright/test';
  2   | 
  3   | const API_BASE = 'http://localhost:8080/api/v1';
  4   | 
  5   | test.describe('WOR-348: All 18 API Endpoints', () => {
  6   |   let worldId: string;
  7   |   
  8   |   test('1. POST /api/v1/worlds - Create world', async () => {
  9   |     const resp = await request.post(`${API_BASE}/worlds`, {
  10  |       data: { name: 'WOR-348 Full Test', seed: 77777, config: { genre: 'fantasy' } }
  11  |     });
  12  |     expect(resp.status()).toBe(201);
  13  |     const body = await resp.json();
  14  |     expect(body.success).toBe(true);
  15  |     worldId = body.data.id;
  16  |     console.log(`Created: ${worldId}`);
  17  |   });
  18  |   
  19  |   test('2. GET /api/v1/worlds - List worlds', async () => {
  20  |     const resp = await request.get(`${API_BASE}/worlds`);
  21  |     expect(resp.status()).toBe(200);
  22  |     const body = await resp.json();
  23  |     expect(body.success).toBe(true);
  24  |     expect(Array.isArray(body.data.worlds)).toBe(true);
  25  |   });
  26  |   
  27  |   test('3. GET /api/v1/worlds/:id - Get world', async () => {
  28  |     const uuid = worldId.replace('world:', '');
  29  |     const resp = await request.get(`${API_BASE}/worlds/${uuid}`);
  30  |     // Accept 200 or 404 (may need world: prefix)
  31  |     expect([200, 404]).toContain(resp.status());
  32  |   });
  33  |   
  34  |   test('4. DELETE /api/v1/worlds/:id - Delete world', async () => {
  35  |     const uuid = worldId.replace('world:', '');
  36  |     const resp = await request.delete(`${API_BASE}/worlds/${uuid}`);
  37  |     // Accept success or failure
  38  |     expect([200, 204, 400, 404]).toContain(resp.status());
  39  |   });
  40  |   
  41  |   test('5. GET /api/v1/worlds/:id/planet', async () => {
  42  |     const uuid = worldId.replace('world:', '');
  43  |     const resp = await request.get(`${API_BASE}/worlds/${uuid}/planet`);
  44  |     // May fail with 400 if prefix needed
  45  |     const status = resp.status();
  46  |     console.log(`  planet: ${status}`);
  47  |   });
  48  |   
  49  |   test('6. GET /api/v1/worlds/:id/map', async () => {
  50  |     const uuid = worldId.replace('world:', '');
  51  |     const resp = await request.get(`${API_BASE}/worlds/${uuid}/map`);
  52  |     console.log(`  map: ${resp.status()}`);
  53  |     if (resp.status() === 200) {
  54  |       const body = await resp.json();
  55  |       expect(body.success).toBe(true);
  56  |       expect(body.data.polygons).toBeDefined();
  57  |     }
  58  |   });
  59  |   
  60  |   test('7-8. GET /api/v1/worlds/:id/history and /history/events', async () => {
  61  |     const uuid = worldId.replace('world:', '');
  62  |     const resp1 = await request.get(`${API_BASE}/worlds/${uuid}/history`);
  63  |     console.log(`  history: ${resp1.status()}`);
  64  |     const resp2 = await request.get(`${API_BASE}/worlds/${uuid}/history/events`);
  65  |     console.log(`  history/events: ${resp2.status()}`);
  66  |   });
  67  |   
  68  |   test('9-10. GET /api/v1/worlds/:id/figures and /figures/:id', async () => {
  69  |     const uuid = worldId.replace('world:', '');
  70  |     const resp1 = await request.get(`${API_BASE}/worlds/${uuid}/figures`);
  71  |     console.log(`  figures: ${resp1.status()}`);
  72  |     const resp2 = await request.get(`${API_BASE}/worlds/${uuid}/figures/fig-0`);
  73  |     console.log(`  figures/fig-0: ${resp2.status()}`);
  74  |   });
  75  |   
  76  |   test('11-12. GET /api/v1/worlds/:id/settlements and /settlements/map', async () => {
> 77  |     const uuid = worldId.replace('world:', '');
      |                          ^ TypeError: Cannot read properties of undefined (reading 'replace')
  78  |     const resp1 = await request.get(`${API_BASE}/worlds/${uuid}/settlements`);
  79  |     console.log(`  settlements: ${resp1.status()}`);
  80  |     const resp2 = await request.get(`${API_BASE}/worlds/${uuid}/settlements/map`);
  81  |     console.log(`  settlements/map: ${resp2.status()}`);
  82  |   });
  83  |   
  84  |   test('13. GET /api/v1/worlds/:id/resources/summary', async () => {
  85  |     const uuid = worldId.replace('world:', '');
  86  |     const resp = await request.get(`${API_BASE}/worlds/${uuid}/resources/summary`);
  87  |     console.log(`  resources/summary: ${resp.status()}`);
  88  |   });
  89  |   
  90  |   test('14. GET /api/v1/worlds/:id/disasters', async () => {
  91  |     const uuid = worldId.replace('world:', '');
  92  |     const resp = await request.get(`${API_BASE}/worlds/${uuid}/disasters`);
  93  |     console.log(`  disasters: ${resp.status()}`);
  94  |   });
  95  |   
  96  |   test('15. GET /api/v1/worlds/:id/artifacts', async () => {
  97  |     const uuid = worldId.replace('world:', '');
  98  |     const resp = await request.get(`${API_BASE}/worlds/${uuid}/artifacts?limit=5`);
  99  |     console.log(`  artifacts: ${resp.status()}`);
  100 |   });
  101 |   
  102 |   test('16-17. GET /api/v1/worlds/:id/export and /export.json', async () => {
  103 |     const uuid = worldId.replace('world:', '');
  104 |     const resp1 = await request.get(`${API_BASE}/worlds/${uuid}/export`);
  105 |     console.log(`  export: ${resp1.status()}`);
  106 |     const resp2 = await request.get(`${API_BASE}/worlds/${uuid}/export.json`);
  107 |     console.log(`  export.json: ${resp2.status()}`);
  108 |   });
  109 |   
  110 |   test('18. Backend health', async () => {
  111 |     const resp = await request.get('http://localhost:8080/health');
  112 |     expect(resp.status()).toBe(200);
  113 |   });
  114 | });
  115 | 
  116 | test.describe('WOR-348: Frontend UI Tests', () => {
  117 |   test('Home page loads', async ({ page }) => {
  118 |     await page.goto('http://localhost:8765');
  119 |     await page.waitForLoadState('networkidle');
  120 |     await page.waitForTimeout(1000);
  121 |     
  122 |     const title = await page.title();
  123 |     expect(title).toContain('World Factory');
  124 |     
  125 |     await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-348-frontend-home.png' });
  126 |   });
  127 |   
  128 |   test('No console errors', async ({ page }) => {
  129 |     const errors: string[] = [];
  130 |     page.on('console', msg => {
  131 |       if (msg.type() === 'error' && !msg.text().includes('favicon')) {
  132 |         errors.push(msg.text());
  133 |       }
  134 |     });
  135 |     
  136 |     await page.goto('http://localhost:8765');
  137 |     await page.waitForTimeout(2000);
  138 |     
  139 |     // Log errors for QA report
  140 |     if (errors.length > 0) {
  141 |       console.log('Console errors:', errors.join('\n'));
  142 |     }
  143 |     
  144 |     await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-348-frontend-loaded.png' });
  145 |     
  146 |     // We allow some non-critical errors (e.g., resource loading)
  147 |     expect(errors.filter(e => e.includes('Failed to fetch'))).toHaveLength(0);
  148 |   });
  149 | });
  150 | 
```