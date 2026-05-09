// WOR-827 Comprehensive Smoke Test - Playwright E2E
import { test, expect } from '@playwright/test';
import * as path from 'path';

const BASE_URL = 'http://localhost:8765';
const screenshotsDir = path.join(__dirname, 'screenshots', 'WOR-827');

test.describe('WOR-827 Smoke Test - Full Stack', () => {
  
  test.beforeAll(async () => {
    // Ensure screenshots directory exists
    const fs = await import('fs');
    if (!fs.existsSync(screenshotsDir)) {
      fs.mkdirSync(screenshotsDir, { recursive: true });
    }
  });

  test('Frontend loads without errors', async ({ page }) => {
    // Capture console errors
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Navigate to frontend
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Take screenshot
    await page.screenshot({ path: `${screenshotsDir}/01-frontend-loaded.png` });
    
    // Verify no critical errors
    console.log('Console errors:', consoleErrors);
    expect(consoleErrors.filter(e => !e.includes('favicon'))).toHaveLength(0);
  });

  test('World creation form works', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Click create new world button
    const createButton = page.locator('button:has-text("Create"), button:has-text("New World"), button:has-text("Generate")').first();
    await createButton.click();
    
    await page.screenshot({ path: `${screenshotsDir}/02-world-form.png` });
    
    // Fill form
    const nameInput = page.locator('input[name="name"], input[placeholder*="name" i], input[type="text"]').first();
    await nameInput.fill('Smoke-Test-World-827');
    
    // Submit
    const submitButton = page.locator('button:has-text("Submit"), button:has-text("Create"), button[type="submit"]').first();
    await submitButton.click();
    
    // Wait for response
    await page.waitForTimeout(3000);
    await page.screenshot({ path: `${screenshotsDir}/03-world-created.png` });
    
    // Should see success or new world in list
    const content = await page.textContent('body');
    expect(content).toMatch(/Smoke-Test-World-827|success|created/i);
  });

  test('World list loads', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.screenshot({ path: `${screenshotsDir}/04-world-list.png` });
    
    // Should see some worlds in the list
    const worldItems = page.locator('[class*="world"], [class*="card"], tr, li').filter({ hasText: /\w+/ });
    const count = await worldItems.count();
    console.log(`Found ${count} world items`);
    expect(count).toBeGreaterThan(0);
  });

  test('Map view renders', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Navigate to a world
    const worldLink = page.locator('a[href*="world"], [class*="world"], button:has-text("View")').first();
    await worldLink.click();
    await page.waitForTimeout(2000);
    
    await page.screenshot({ path: `${screenshotsDir}/05-map-view.png` });
    
    // Check if canvas or SVG exists for map
    const mapCanvas = page.locator('canvas, svg, [class*="map"]').first();
    await expect(mapCanvas).toBeVisible({ timeout: 5000 });
  });

  test('Tab navigation works', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Navigate to world
    const worldLink = page.locator('a[href*="world"], [class*="world"], button:has-text("View")').first();
    await worldLink.click();
    await page.waitForTimeout(2000);
    
    // Find and click tabs
    const tabs = page.locator('[role="tab"], [class*="tab"]');
    const tabCount = await tabs.count();
    console.log(`Found ${tabCount} tabs`);
    
    if (tabCount > 0) {
      for (let i = 0; i < Math.min(tabCount, 5); i++) {
        await tabs.nth(i).click();
        await page.waitForTimeout(500);
        await page.screenshot({ path: `${screenshotsDir}/06-tab-${i}.png` });
      }
    }
  });

  test('Timeline/History loads', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Navigate to world
    const worldLink = page.locator('a[href*="world"], [class*="world"], button:has-text("View")').first();
    await worldLink.click();
    await page.waitForTimeout(2000);
    
    // Click History or Timeline tab
    const historyTab = page.locator('[role="tab"]:has-text("History"), [role="tab"]:has-text("Timeline"), button:has-text("History"), button:has-text("Timeline")').first();
    await historyTab.click();
    await page.waitForTimeout(1000);
    
    await page.screenshot({ path: `${screenshotsDir}/07-timeline.png` });
    
    // Verify timeline content exists
    const content = await page.textContent('body');
    expect(content).toMatch(/history|timeline|event/i);
  });

  test('Dashboard loads', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Navigate to world
    const worldLink = page.locator('a[href*="world"], [class*="world"], button:has-text("View")').first();
    await worldLink.click();
    await page.waitForTimeout(2000);
    
    // Click Dashboard tab
    const dashboardTab = page.locator('[role="tab"]:has-text("Dashboard"), button:has-text("Dashboard")').first();
    await dashboardTab.click();
    await page.waitForTimeout(1000);
    
    await page.screenshot({ path: `${screenshotsDir}/08-dashboard.png` });
    
    // Verify dashboard content
    const content = await page.textContent('body');
    expect(content).toMatch(/dashboard|summary|world/i);
  });

  test('Figures list loads', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Navigate to world
    const worldLink = page.locator('a[href*="world"], [class*="world"], button:has-text("View")').first();
    await worldLink.click();
    await page.waitForTimeout(2000);
    
    // Click Figures tab
    const figuresTab = page.locator('[role="tab"]:has-text("Figure"), button:has-text("Figure")').first();
    await figuresTab.click();
    await page.waitForTimeout(1000);
    
    await page.screenshot({ path: `${screenshotsDir}/09-figures.png` });
    
    // Should see figures section
    const content = await page.textContent('body');
    expect(content).toMatch(/figure|person|character/i);
  });

  test('No console errors throughout', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Navigate through app
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Navigate to world
    const worldLink = page.locator('a[href*="world"], [class*="world"], button:has-text("View")').first();
    if (await worldLink.isVisible()) {
      await worldLink.click();
      await page.waitForTimeout(2000);
    }
    
    // Navigate through tabs
    const tabs = page.locator('[role="tab"]');
    const tabCount = await tabs.count();
    for (let i = 0; i < Math.min(tabCount, 5); i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(500);
    }
    
    // Final screenshot
    await page.screenshot({ path: `${screenshotsDir}/10-final-state.png` });
    
    // Filter out non-critical errors
    const criticalErrors = consoleErrors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('404') &&
      !e.includes('net::')
    );
    
    console.log('All console errors:', consoleErrors);
    expect(criticalErrors).toHaveLength(0);
  });
});