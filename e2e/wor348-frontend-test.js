const { chromium } = require('playwright');

const BASE_URL = 'http://localhost:8765';

async function captureScreenshots() {
  console.log('=== WOR-348 Frontend Testing ===\n');
  
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
  const page = await context.newPage();
  
  const errors = [];
  page.on('console', msg => {
    if (msg.type() === 'error' && !msg.text().includes('favicon') && !msg.text().includes('DevTools')) {
      errors.push(msg.text());
    }
  });
  
  // 1. Test World List
  console.log('1. Loading world list...');
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 15000 });
  await page.waitForTimeout(2000);
  await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-348-world-list.png', fullPage: false });
  
  const title = await page.title();
  console.log(`   Title: ${title}`);
  
  // Check for world cards
  const worldCards = await page.$$('.world-card, [class*="card"]');
  console.log(`   World cards found: ${worldCards.length}`);
  
  // 2. Test Map View (try to find and click View Map button)
  console.log('\n2. Testing map view navigation...');
  const mapBtn = await page.$('button:has-text("Map"), button:has-text("View Map")');
  if (mapBtn) {
    await mapBtn.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-348-map-view.png', fullPage: false });
    console.log('   Map view screenshot captured');
  } else {
    console.log('   No map button found on current page');
    // Try navigating directly
    await page.goto(BASE_URL + '/map', { timeout: 10000 }).catch(() => {});
    await page.waitForTimeout(2000);
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-348-map-view.png', fullPage: false });
  }
  
  // 3. Test Timeline View
  console.log('\n3. Testing timeline view...');
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 15000 });
  await page.waitForTimeout(1000);
  const timelineBtn = await page.$('button:has-text("Timeline"), button:has-text("History")');
  if (timelineBtn) {
    await timelineBtn.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: '/home/kyle/projects/world-generator/screenshots/WOR-348-timeline.png', fullPage: false });
    console.log('   Timeline screenshot captured');
  }
  
  // 4. Console Error Summary
  console.log('\n4. Console Error Summary:');
  if (errors.length === 0) {
    console.log('   ✅ No console errors detected');
  } else {
    console.log(`   ❌ ${errors.length} console errors found:`);
    errors.forEach((e, i) => console.log(`      ${i+1}. ${e.substring(0, 100)}`));
  }
  
  await browser.close();
  
  console.log('\n=== Screenshots saved to /screenshots/WOR-348-*.png ===');
  
  return errors.length;
}

captureScreenshots()
  .then(errCount => process.exit(errCount > 3 ? 1 : 0))
  .catch(e => { console.error(e); process.exit(1); });
