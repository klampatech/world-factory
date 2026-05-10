import { chromium } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:8787';

async function main() {
  console.log('Full console error debug...\n');
  
  // Get a ready world first
  const listResp = await fetch('http://localhost:8080/api/v1/worlds');
  const listData = await listResp.json();
  const worlds = listData.data?.worlds || [];
  let worldId = worlds.find(w => w.status === 'ready')?.id;
  
  if (!worldId) {
    const createResp = await fetch('http://localhost:8080/api/v1/worlds', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Debug', config: { width: 32, height: 32 } })
    });
    const newWorld = await createResp.json();
    worldId = newWorld.data?.id || newWorld.id;
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 2000));
      const s = await fetch(`http://localhost:8080/api/v1/worlds/${worldId}`);
      const d = await s.json();
      if (d.data?.status === 'ready') break;
    }
  }
  console.log(`Using world: ${worldId}`);
  
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  const allErrors = [];
  const all404 = [];
  
  page.on('console', msg => {
    if (msg.type() === 'error') allErrors.push(msg.text());
  });
  
  page.on('response', r => {
    if (r.status() === 404) all404.push(r.url());
  });
  
  // Test all pages
  const pages = [
    '/',
    `/worlds/${worldId}`,
    `/worlds/${worldId}/map`,
    `/worlds/${worldId}/timeline`,
    `/worlds/${worldId}/dashboard`,
    `/worlds/${worldId}/figures`
  ];
  
  for (const path of pages) {
    console.log(`\n=== Testing ${path} ===`);
    allErrors.length = 0;
    all404.length = 0;
    
    await page.goto(`${FRONTEND_URL}${path}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);
    
    if (allErrors.length > 0) {
      console.log('Errors:', allErrors);
    } else {
      console.log('✅ No errors');
    }
    
    if (all404.length > 0) {
      console.log('404s:', all404);
    }
  }
  
  console.log('\n=== Final Summary ===');
  console.log('Total 404s:', all404.length);
  
  await browser.close();
}

main().catch(console.error);
