import { test, expect } from '@playwright/test';
import path from 'path';

test.describe('WOR-134 Smoke Test Screenshots', () => {
  const screenshotsDir = path.join(__dirname, '..', 'screenshots', 'WOR-134');
  
  test('Capture homepage screenshot', async ({ page }) => {
    await page.goto('http://localhost:8787');
    await page.waitForLoadState('networkidle');
    
    // Check for console errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.screenshot({ path: `${screenshotsDir}/01-homepage.png` });
    console.log('Console errors:', errors.length > 0 ? errors : 'None');
  });
  
  test('Capture map view screenshot', async ({ page }) => {
    await page.goto('http://localhost:8787/#/world/0d182486-b794-4d35-b8a9-f1747af28907');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000); // Wait for map to render
    
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.screenshot({ path: `${screenshotsDir}/02-map-view.png`, fullPage: true });
    console.log('Console errors:', errors.length > 0 ? errors : 'None');
  });
  
  test('Capture timeline screenshot', async ({ page }) => {
    await page.goto('http://localhost:8787/#/world/0d182486-b794-4d35-b8a9-f1747af28907/timeline');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    
    await page.screenshot({ path: `${screenshotsDir}/03-timeline.png`, fullPage: true });
  });
  
  test('Capture selector view screenshot', async ({ page }) => {
    await page.goto('http://localhost:8787/#/selector');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    
    await page.screenshot({ path: `${screenshotsDir}/04-selector-view.png`, fullPage: true });
  });
});
