const { chromium } = require('playwright');

const BASE_URL = 'http://localhost:8765';

async function runTests() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
  const page = await context.newPage();

  const errors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') errors.push(msg.text());
  });

  console.log('Loading page...');
  await page.goto(BASE_URL + '/', { waitUntil: 'domcontentloaded', timeout: 10000 });
  await page.waitForTimeout(2000);
  
  // Find a ready world
  const readyWorld = await page.evaluate(() => {
    const cards = document.querySelectorAll('.world-card');
    for (const card of cards) {
      const statusEl = card.querySelector('.world-status, [class*="status"]');
      const text = card.textContent || '';
      if (text.includes('ready')) {
        // Find the View Map button
        const btn = card.querySelector('button[onclick*="router.navigate"]');
        if (btn) {
          return { text: text.trim().substring(0, 50), onclick: btn.getAttribute('onclick') };
        }
      }
    }
    return null;
  });
  console.log('Ready world found:', JSON.stringify(readyWorld));
  
  // Try to click the View Map button directly
  if (readyWorld && readyWorld.onclick) {
    console.log('Clicking View Map button...');
    // Use evaluate to click
    await page.evaluate(() => {
      const btn = document.querySelector('button[onclick*="router.navigate"]');
      if (btn) btn.click();
    });
    await page.waitForTimeout(3000);
    console.log('URL after click:', page.url());
  }
  
  // Screenshot
  await page.screenshot({ path: './screenshots/WOR-115/debug-after-view-click.png' });
  
  // Check for canvas now
  const canvasCount = await page.evaluate(() => document.querySelectorAll('canvas').length);
  console.log('Canvas elements:', canvasCount);
  
  // Check what's in main
  const mainContent = await page.evaluate(() => {
    const main = document.querySelector('main');
    return main ? main.innerHTML.substring(0, 1000) : 'no main';
  });
  console.log('Main content:', mainContent.substring(0, 500));
  
  await browser.close();
}

runTests().catch(e => {
  console.error(e);
  process.exit(1);
});
