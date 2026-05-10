import { chromium } from '@playwright/test';

const BASE_URL = 'http://localhost:8080';
const FRONTEND_URL = 'http://localhost:5173';

async function main() {
  console.log('Investigating console 404 errors on real world pages...\n');
  
  // Get a ready world
  console.log('Getting a ready world...');
  const listResp = await fetch(`${BASE_URL}/api/v1/worlds`);
  const listData = await listResp.json();
  const worlds = listData.data?.worlds || [];
  const readyWorld = worlds.find(w => w.status === 'ready') || worlds[0];
  
  if (!readyWorld) {
    console.log('No ready world found, creating one...');
    const createResp = await fetch(`${BASE_URL}/api/v1/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Debug World', config: { width: 32, height: 32 } })
    });
    const newWorld = await createResp.json();
    const worldId = newWorld.data?.id || newWorld.id;
    
    // Wait for ready
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 2000));
      const statusResp = await fetch(`${BASE_URL}/api/v1/worlds/${worldId}`);
      const statusData = await statusResp.json();
      if (statusData.data?.status === 'ready') {
        readyWorld = { id: worldId };
        break;
      }
    }
  }
  
  const worldId = readyWorld?.id || 'test';
  console.log(`Testing with world ID: ${worldId}\n`);
  
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
  const page = await context.newPage();
  
  const failedResources = [];
  const consoleErrors = [];
  
  page.on('response', response => {
    if (response.status() === 404) {
      failedResources.push(response.url());
    }
  });
  
  page.on('console', msg => {
    if (msg.type() === 'error') {
      consoleErrors.push(msg.text());
    }
  });
  
  // Test different pages
  const pages = ['/', `/worlds/${worldId}`, `/worlds/${worldId}/map`, `/worlds/${worldId}/timeline`];
  
  for (const path of pages) {
    console.log(`\nTesting: ${path}`);
    failedResources.length = 0;
    consoleErrors.length = 0;
    
    try {
      await page.goto(`${FRONTEND_URL}${path}`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      
      if (failedResources.length > 0) {
        console.log('  404 Resources:');
        failedResources.forEach(url => console.log(`    - ${url}`));
      } else {
        console.log('  ✅ No 404 resources');
      }
      
      if (consoleErrors.length > 0) {
        console.log('  Console Errors:');
        consoleErrors.forEach(e => console.log(`    - ${e}`));
      }
    } catch (e) {
      console.log(`  ❌ Error loading page: ${e.message}`);
    }
  }
  
  await browser.close();
}

main().catch(console.error);
