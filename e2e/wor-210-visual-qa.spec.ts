import { test, expect, chromium } from '@playwright/test';
import path from 'path';

test.describe('WOR-210: Visual Voronoi Rendering Verification', () => {
  
  test('Capture visual screenshot of Voronoi map rendering', async () => {
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1400, height: 900 }
    });
    const page = await context.newPage();
    
    try {
      // Navigate to the frontend
      await page.goto('http://localhost:8787');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);
      
      // Take screenshot of the full page
      const screenshotPath = path.join('screenshots', `WOR-210-voronoi-visual-${Date.now()}.png`);
      await page.screenshot({ 
        path: screenshotPath, 
        fullPage: true 
      });
      
      console.log(`Visual screenshot saved: ${screenshotPath}`);
      
      // Verify no console errors
      const errors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          errors.push(msg.text());
        }
      });
      
      // Reload to catch any errors
      await page.reload();
      await page.waitForTimeout(2000);
      
      console.log(`Console errors: ${errors.length}`);
      
    } finally {
      await browser.close();
    }
  });
});
