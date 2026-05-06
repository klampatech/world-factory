import { test, expect, Page } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:8765';

test('Capture screenshots for WOR-141 QA evidence', async ({ page }) => {
  // 1. Frontend landing page
  await page.goto(FRONTEND_URL);
  await page.waitForTimeout(2000);
  await page.screenshot({ path: '../screenshots/WOR-141/01-frontend-landing.png', fullPage: true });
  
  // 2. World Selector with cards
  await page.screenshot({ path: '../screenshots/WOR-141/02-world-selector-view.png', fullPage: true });
  
  // 3. Header section
  const header = page.locator('header');
  await header.screenshot({ path: '../screenshots/WOR-141/03-header-section.png' });
  
  // 4. Stats bar
  const statsBar = page.locator('.stats-bar');
  if (await statsBar.count() > 0) {
    await statsBar.screenshot({ path: '../screenshots/WOR-141/04-stats-bar.png' });
  }
  
  // 5. Navigate to map viewer if possible
  const viewMapBtn = page.locator('button:has-text("View Map")').first();
  if (await viewMapBtn.count() > 0) {
    await viewMapBtn.click();
    await page.waitForTimeout(3000);
    await page.screenshot({ path: '../screenshots/WOR-141/05-map-viewer.png', fullPage: true });
    
    // 6. Map canvas
    const mapCanvas = page.locator('#map-canvas');
    if (await mapCanvas.count() > 0) {
      await mapCanvas.screenshot({ path: '../screenshots/WOR-141/06-map-canvas.png' });
    }
  }
  
  console.log('Screenshots captured to screenshots/WOR-141/');
});
