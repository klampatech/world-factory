import { test, expect } from '@playwright/test';

/**
 * WOR-434: Browser Screenshot Capability for Visual Verification
 * 
 * Captures screenshots of key UI states for visual QA.
 * Screenshots are saved to test-results/screenshots/
 * 
 * Run:
 *   npx playwright test --project=chromium e2e/screenshot-tests.spec.ts
 *   npx playwright test --project=chromium --headed e2e/screenshot-tests.spec.ts  # With visible browser
 */

const BASE_URL = 'http://localhost:8765';
const SCREENSHOT_DIR = 'test-results/screenshots';

test.describe('Visual Verification Screenshots', () => {

  // Ensure screenshot directory exists
  test.beforeAll(async () => {
    // Playwright creates directories automatically, but we verify config
  });

  // SC-001: Full page - Initial Load State
  test('SC-001: Full page initial load', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000); // Let content render
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-001-full-page-initial-load.png`,
      fullPage: true 
    });
    
    // Verify page loaded
    const title = await page.title();
    expect(title).toContain('World Factory');
  });

  // SC-002: Map View - Full Canvas
  test('SC-002: Map view with canvas', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1500);
    
    // Capture just the canvas area
    const canvas = page.locator('#map-canvas');
    await expect(canvas).toBeVisible();
    
    const box = await canvas.boundingBox();
    if (box) {
      await page.screenshot({ 
        path: `${SCREENSHOT_DIR}/SC-002-map-canvas.png`,
        clip: { x: box.x, y: box.y, width: box.width, height: box.height }
      });
    }
  });

  // SC-003: Overlay Controls - Resources
  test('SC-003: Resources overlay active', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    // Click resources overlay
    const resourcesBtn = page.locator('[data-overlay="resources"]');
    await resourcesBtn.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-003-resources-overlay.png`,
      fullPage: true 
    });
  });

  // SC-004: Overlay Controls - Elevation
  test('SC-004: Elevation overlay active', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const elevationBtn = page.locator('[data-overlay="elevation"]');
    await elevationBtn.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-004-elevation-overlay.png`,
      fullPage: true 
    });
  });

  // SC-005: Overlay Controls - Political
  test('SC-005: Political overlay active', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const politicalBtn = page.locator('[data-overlay="political"]');
    await politicalBtn.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-005-political-overlay.png`,
      fullPage: true 
    });
  });

  // SC-006: Overlay Controls - Wonders
  test('SC-006: Wonders overlay active', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const wondersBtn = page.locator('[data-overlay="wonders"]');
    await wondersBtn.click();
    await page.waitForTimeout(500);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-006-wonders-overlay.png`,
      fullPage: true 
    });
  });

  // SC-007: Timeline View
  test('SC-007: Timeline view', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    // Click Timeline tab
    const timelineTab = page.locator('.view-tab:has-text("Timeline")');
    if (await timelineTab.count() > 0) {
      await timelineTab.click();
      await page.waitForTimeout(500);
    }
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-007-timeline-view.png`,
      fullPage: true 
    });
  });

  // SC-008: Zoom In State
  test('SC-008: Zoom in interaction', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    // Find and click zoom in button
    const zoomIn = page.locator('#zoom-in, .zoom-in, button:has-text("+"), [aria-label*="zoom in"]').first();
    if (await zoomIn.count() > 0) {
      await zoomIn.click();
      await zoomIn.click();
      await zoomIn.click();
      await page.waitForTimeout(500);
    }
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-008-zoomed-in.png`,
      fullPage: true 
    });
  });

  // SC-009: Zoom Out State
  test('SC-009: Zoom out interaction', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const zoomOut = page.locator('#zoom-out, .zoom-out, button:has-text("-"), [aria-label*="zoom out"]').first();
    if (await zoomOut.count() > 0) {
      await zoomOut.click();
      await zoomOut.click();
      await zoomOut.click();
      await page.waitForTimeout(500);
    }
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-009-zoomed-out.png`,
      fullPage: true 
    });
  });

  // SC-010: Pan Interaction
  test('SC-010: Pan interaction state', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const canvas = page.locator('#map-canvas');
    const box = await canvas.boundingBox();
    
    if (box) {
      // Perform a pan action
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await page.mouse.down();
      await page.mouse.move(box.x + box.width / 2 + 150, box.y + box.height / 2 + 100);
      await page.mouse.up();
      await page.waitForTimeout(500);
    }
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-010-pan-state.png`,
      fullPage: true 
    });
  });

  // SC-011: Header and Controls
  test('SC-011: Header visible', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const header = page.locator('header');
    await expect(header).toBeVisible();
    
    // Capture just the header area
    const box = await header.boundingBox();
    if (box) {
      await page.screenshot({ 
        path: `${SCREENSHOT_DIR}/SC-011-header.png`,
        clip: { x: box.x, y: box.y, width: box.width, height: Math.min(box.height * 2, 200) }
      });
    }
  });

  // SC-012: Responsive - Mobile Width
  test('SC-012: Mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-012-mobile-view.png`,
      fullPage: true 
    });
  });

  // SC-013: Responsive - Tablet Width
  test('SC-013: Tablet viewport', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-013-tablet-view.png`,
      fullPage: true 
    });
  });

  // SC-014: Error State (if applicable)
  test('SC-014: No critical console errors', async ({ page }) => {
    const errors: string[] = [];
    
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(2000);
    
    // Filter critical errors
    const criticalErrors = errors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('net::ERR') &&
      !e.includes('Failed to load resource')
    );
    
    // Capture screenshot showing console state
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/SC-014-console-clean.png`,
      fullPage: true 
    });
    
    expect(criticalErrors).toHaveLength(0);
  });

});

test.describe('Comparison Screenshots', () => {
  // These tests capture before/after states for visual regression

  test('Baseline: Default map state', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(2000);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/baseline-default-map.png`,
      fullPage: true 
    });
  });

  test('All overlays cycling capture', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1000);
    
    const overlays = ['resources', 'elevation', 'political', 'wonders'];
    
    for (const overlay of overlays) {
      const btn = page.locator(`[data-overlay="${overlay}"]`);
      await btn.click();
      await page.waitForTimeout(300);
      
      await page.screenshot({ 
        path: `${SCREENSHOT_DIR}/overlay-${overlay}-cycling.png`,
        fullPage: true 
      });
    }
  });
});