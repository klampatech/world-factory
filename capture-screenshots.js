const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  // Capture selector view
  console.log('Capturing selector view...');
  await page.goto('http://localhost:8765');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'screenshots/WOR-179-selector-view.png' });
  
  // Navigate to a world viewer
  console.log('Capturing viewer page...');
  await page.locator('.btn-view.primary').first().click();
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshots/WOR-179-map-view.png' });
  
  // Timeline view
  console.log('Capturing timeline view...');
  await page.click('.view-tab:has-text("Timeline")');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshots/WOR-179-timeline-view.png' });
  
  // Dashboard view
  console.log('Capturing dashboard view...');
  await page.click('.view-tab:has-text("Dashboard")');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshots/WOR-179-dashboard-view.png' });
  
  await browser.close();
  console.log('Screenshots captured!');
})();
