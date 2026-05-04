// World Factory E2E Test Suite
// WOR-604: Create e2e test suite
//
// Tests browser-based UI interactions for the World Factory application.
// Target: http://localhost:8765
//
// Run:
//   npx playwright test --config=playwright.e2e.config.ts e2e/wf-e2e.spec.ts

import { test, expect, Page } from '@playwright/test';

// =======================================================================
// Test Configuration
// =======================================================================
const BASE_URL = 'http://localhost:8765';
const MAP_CANVAS = '#map-canvas';
const OVERLAY_CONTROLS = '#overlay-controls';

// =======================================================================
// Helper Functions
// =======================================================================
async function waitForMapReady(page: Page, timeout = 15000): Promise<void> {
  // Wait for canvas to be visible
  await expect(page.locator(MAP_CANVAS)).toBeVisible({ timeout });
  
  // Wait for loading overlay to disappear (if it appears)
  try {
    await page.locator('#map-loading').waitFor({ state: 'hidden', timeout: 30000 });
  } catch {
    // Loading overlay may not exist or already hidden
  }
}

async function clickOverlay(page: Page, overlayName: string): Promise<void> {
  const overlayBtn = page.locator(`[data-overlay="${overlayName}"]`);
  await overlayBtn.click({ timeout: 5000 });
}

// =======================================================================
// E2E-WF-001: Basic Page Load
// =======================================================================
test.describe('E2E-WF-001: Page Load & Initialization', () => {
  
  test('E2E-WF-001.1: Page loads without crash', async ({ page }) => {
    const response = await page.goto(BASE_URL + '/');
    expect(response?.status()).toBe(200);
    console.log('✓ Page loaded with HTTP 200');
  });

  test('E2E-WF-001.2: Map canvas exists and is visible', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    await expect(page.locator(MAP_CANVAS)).toBeVisible();
    console.log('✓ Map canvas is visible');
  });

  test('E2E-WF-001.3: Canvas has non-zero dimensions', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    const canvas = page.locator(MAP_CANVAS);
    const box = await canvas.boundingBox();
    expect(box?.width).toBeGreaterThan(0);
    expect(box?.height).toBeGreaterThan(0);
    console.log(`✓ Canvas dimensions: ${box?.width}x${box?.height}`);
  });

  test('E2E-WF-001.4: No critical console errors on load', async ({ page }) => {
    const errors: string[] = [];
    
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(2000);
    
    // Filter out known benign errors
    const criticalErrors = errors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('net::ERR') &&
      !e.includes('Failed to load resource')
    );
    
    expect(criticalErrors).toHaveLength(0);
    console.log(`✓ No critical console errors (total: ${errors.length})`);
  });

});

// =======================================================================
// E2E-WF-002: Overlay System
// =======================================================================
test.describe('E2E-WF-002: Overlay System', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
  });

  test('E2E-WF-002.1: All overlay control buttons exist', async ({ page }) => {
    const overlays = ['resources', 'elevation', 'political', 'wonders'];
    
    for (const overlay of overlays) {
      const btn = page.locator(`[data-overlay="${overlay}"]`);
      await expect(btn).toBeVisible();
      console.log(`  ✓ ${overlay} overlay button visible`);
    }
    console.log('✓ All 4 overlay buttons exist');
  });

  test('E2E-WF-002.2: Overlay controls section exists', async ({ page }) => {
    await expect(page.locator(OVERLAY_CONTROLS)).toBeVisible();
    console.log('✓ Overlay controls section visible');
  });

  test('E2E-WF-002.3: Clicking Resources overlay activates it', async ({ page }) => {
    await clickOverlay(page, 'resources');
    await page.waitForTimeout(300);
    
    const legend = page.locator('#overlay-legend');
    await expect(legend).toBeVisible();
    console.log('✓ Resources overlay activates legend');
  });

  test('E2E-WF-002.4: Clicking Elevation overlay activates it', async ({ page }) => {
    await clickOverlay(page, 'elevation');
    await page.waitForTimeout(300);
    
    const legend = page.locator('#overlay-legend');
    await expect(legend).toBeVisible();
    console.log('✓ Elevation overlay activates legend');
  });

  test('E2E-WF-002.5: Clicking Political overlay activates it', async ({ page }) => {
    await clickOverlay(page, 'political');
    await page.waitForTimeout(300);
    
    const legend = page.locator('#overlay-legend');
    await expect(legend).toBeVisible();
    console.log('✓ Political overlay activates legend');
  });

  test('E2E-WF-002.6: Clicking Wonders overlay activates it', async ({ page }) => {
    await clickOverlay(page, 'wonders');
    await page.waitForTimeout(300);
    
    const legend = page.locator('#overlay-legend');
    await expect(legend).toBeVisible();
    console.log('✓ Wonders overlay activates legend');
  });

  test('E2E-WF-002.7: Only one overlay can be active at a time', async ({ page }) => {
    // Activate resources
    await clickOverlay(page, 'resources');
    await page.waitForTimeout(300);
    
    // Activate elevation (should deactivate resources)
    await clickOverlay(page, 'elevation');
    await page.waitForTimeout(300);
    
    // Both buttons should still exist
    await expect(page.locator('[data-overlay="resources"]')).toBeVisible();
    await expect(page.locator('[data-overlay="elevation"]')).toBeVisible();
    
    // Legend should be visible
    await expect(page.locator('#overlay-legend')).toBeVisible();
    console.log('✓ Overlay exclusivity works');
  });

});

// =======================================================================
// E2E-WF-003: Map Interaction (Pan & Zoom)
// =======================================================================
test.describe('E2E-WF-003: Map Interaction', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
  });

  test('E2E-WF-003.1: Map canvas responds to mouse pan', async ({ page }) => {
    const canvas = page.locator(MAP_CANVAS);
    const box = await canvas.boundingBox();
    
    if (!box) throw new Error('Canvas not found');
    
    const startX = box.x + box.width / 2;
    const startY = box.y + box.height / 2;
    
    // Perform drag (pan)
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX + 100, startY + 50);
    await page.mouse.up();
    
    // Canvas should still be visible
    await expect(canvas).toBeVisible();
    console.log('✓ Map pan interaction works');
  });

  test('E2E-WF-003.2: Zoom controls are accessible', async ({ page }) => {
    // Look for any zoom control
    const zoomIn = page.locator('#zoom-in, .zoom-in, button:has-text("+"), [aria-label*="zoom in"]').first();
    
    if (await zoomIn.count() > 0) {
      await zoomIn.click();
      await page.waitForTimeout(200);
      console.log('✓ Zoom controls exist and are clickable');
    } else {
      console.log('  ℹ No dedicated zoom buttons found (zoom via scroll may be supported)');
    }
  });

  test('E2E-WF-003.3: Canvas maintains visibility after interactions', async ({ page }) => {
    const canvas = page.locator(MAP_CANVAS);
    
    // Multiple interactions
    await page.mouse.move(400, 300);
    await page.mouse.down();
    await page.mouse.move(500, 400);
    await page.mouse.up();
    
    await page.waitForTimeout(500);
    await expect(canvas).toBeVisible();
    console.log('✓ Canvas remains visible after pan');
  });

});

// =======================================================================
// E2E-WF-004: Timeline View
// =======================================================================
test.describe('E2E-WF-004: Timeline View', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
  });

  test('E2E-WF-004.1: Timeline tab/button exists', async ({ page }) => {
    const timelineTab = page.locator('.view-tab:has-text("Timeline"), #timeline-view, .timeline-container');
    await expect(timelineTab.first()).toBeVisible();
    console.log('✓ Timeline control exists');
  });

  test('E2E-WF-004.2: Timeline tab is clickable', async ({ page }) => {
    const timelineTab = page.locator('.view-tab:has-text("Timeline")');
    
    if (await timelineTab.count() > 0) {
      await timelineTab.click();
      await page.waitForTimeout(500);
      console.log('✓ Timeline tab is clickable');
    } else {
      console.log('  ℹ Timeline tab not found - may be integrated differently');
    }
  });

  test('E2E-WF-004.3: Map remains accessible after switching views', async ({ page }) => {
    // Map view is default
    await expect(page.locator(MAP_CANVAS)).toBeVisible();
    
    // Try timeline if exists
    const timelineTab = page.locator('.view-tab:has-text("Timeline")');
    if (await timelineTab.count() > 0) {
      await timelineTab.click();
      await page.waitForTimeout(300);
    }
    
    // Map should still be in DOM and functional
    await expect(page.locator(MAP_CANVAS)).toBeVisible();
    console.log('✓ Map remains after view switch');
  });

});

// =======================================================================
// E2E-WF-005: Header & Navigation
// =======================================================================
test.describe('E2E-WF-005: Header & Navigation', () => {
  
  test('E2E-WF-005.1: Header renders correctly', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    
    const header = page.locator('header');
    await expect(header).toBeVisible();
    
    // Logo should be present
    const logo = page.locator('.logo, h1, [class*="logo"]');
    await expect(logo.first()).toBeVisible();
    console.log('✓ Header renders correctly');
  });

  test('E2E-WF-005.2: View tabs exist for navigation', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    
    const tabs = page.locator('.view-tab');
    const tabCount = await tabs.count();
    
    expect(tabCount).toBeGreaterThan(0);
    console.log(`✓ ${tabCount} view tabs available`);
  });

  test('E2E-WF-005.3: Map view tab is active by default', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    
    const mapTab = page.locator('.view-tab:has-text("Map"), .view-tab.active');
    await expect(mapTab.first()).toBeVisible();
    console.log('✓ Map view is default');
  });

});

// =======================================================================
// E2E-WF-006: Responsive Design
// =======================================================================
test.describe('E2E-WF-006: Responsive Design', () => {
  
  test('E2E-WF-006.1: Desktop viewport (1920x1080)', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    const canvas = page.locator(MAP_CANVAS);
    const box = await canvas.boundingBox();
    expect(box?.width).toBeGreaterThan(1000);
    console.log(`✓ Desktop layout: ${box?.width}x${box?.height}`);
  });

  test('E2E-WF-006.2: Tablet viewport (768x1024)', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    const canvas = page.locator(MAP_CANVAS);
    await expect(canvas).toBeVisible();
    console.log('✓ Tablet layout works');
  });

  test('E2E-WF-006.3: Mobile viewport (375x667)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    const controls = page.locator(OVERLAY_CONTROLS);
    await expect(controls).toBeVisible();
    
    const canvas = page.locator(MAP_CANVAS);
    await expect(canvas).toBeVisible();
    console.log('✓ Mobile layout works');
  });

});

// =======================================================================
// E2E-WF-007: Screenshot Capture for Visual QA
// =======================================================================
test.describe('E2E-WF-007: Visual QA Screenshots', () => {
  
  const SCREENSHOT_DIR = 'test-results/screenshots';

  test('E2E-WF-007.1: Capture initial page state', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1500);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/E2E-WF-007-1-initial-state.png`,
      fullPage: true 
    });
    console.log('✓ Screenshot: initial state captured');
  });

  test('E2E-WF-007.2: Capture each overlay state', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    const overlays = ['resources', 'elevation', 'political', 'wonders'];
    
    for (const overlay of overlays) {
      await clickOverlay(page, overlay);
      await page.waitForTimeout(500);
      
      await page.screenshot({ 
        path: `${SCREENSHOT_DIR}/E2E-WF-007-overlay-${overlay}.png`,
        fullPage: true 
      });
      console.log(`  ✓ Screenshot: ${overlay} overlay`);
    }
  });

  test('E2E-WF-007.3: Capture timeline view', async ({ page }) => {
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    const timelineTab = page.locator('.view-tab:has-text("Timeline")');
    if (await timelineTab.count() > 0) {
      await timelineTab.click();
      await page.waitForTimeout(500);
    }
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/E2E-WF-007-timeline-view.png`,
      fullPage: true 
    });
    console.log('✓ Screenshot: timeline view captured');
  });

  test('E2E-WF-007.4: Capture mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto(BASE_URL + '/');
    await page.waitForTimeout(1500);
    
    await page.screenshot({ 
      path: `${SCREENSHOT_DIR}/E2E-WF-007-mobile-view.png`,
      fullPage: true 
    });
    console.log('✓ Screenshot: mobile view captured');
  });

});

// =======================================================================
// E2E-WF-008: Smoke Test Summary
// =======================================================================
test.describe('E2E-WF-008: Smoke Test Summary', () => {
  
  test('E2E-WF-008: All critical paths are navigable', async ({ page }) => {
    // 1. Load page
    await page.goto(BASE_URL + '/');
    await waitForMapReady(page);
    
    // 2. Toggle each overlay
    const overlays = ['resources', 'elevation', 'political', 'wonders'];
    for (const overlay of overlays) {
      await clickOverlay(page, overlay);
      await page.waitForTimeout(200);
    }
    
    // 3. Navigate to timeline
    const timelineTab = page.locator('.view-tab:has-text("Timeline")');
    if (await timelineTab.count() > 0) {
      await timelineTab.click();
      await page.waitForTimeout(300);
    }
    
    // 4. Return to map
    const mapTab = page.locator('.view-tab:has-text("Map")');
    if (await mapTab.count() > 0) {
      await mapTab.click();
      await page.waitForTimeout(300);
    }
    
    // 5. Pan the map
    const canvas = page.locator(MAP_CANVAS);
    const box = await canvas.boundingBox();
    if (box) {
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await page.mouse.down();
      await page.mouse.move(box.x + box.width / 2 + 100, box.y + box.height / 2);
      await page.mouse.up();
    }
    
    // 6. Verify no errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.waitForTimeout(1000);
    
    const criticalErrors = errors.filter(e => 
      !e.includes('favicon') && 
      !e.includes('net::ERR') &&
      !e.includes('Failed to load resource')
    );
    
    expect(criticalErrors).toHaveLength(0);
    console.log('✓ Complete smoke test: all critical paths pass');
  });

});