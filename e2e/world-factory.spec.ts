// World Factory Frontend E2E Tests
// 
// Playwright tests for UI validation
//
// Setup:
//   npm install -D @playwright/test
//   npx playwright install chromium
//
// Run:
//   npx playwright test e2e/world-factory.spec.ts

import { test, expect } from '@playwright/test';

// =======================================================================
// E2E-FRONT-001: World Generation UI Flow (P0 Smoke Test)
// =======================================================================
test.describe('World Generation', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
  });

  test('E2E-FRONT-001: World Generation UI Flow', async ({ page }) => {
    // Step 1: Navigate to /new-world
    await page.getByTestId('new-world-btn').click();
    await expect(page).toHaveURL(/\/new-world/);
    
    // Step 2: Enter seed=42
    await page.getByTestId('seed-input').fill('42');
    
    // Step 3: Select size 32x32
    await page.getByTestId('size-select').selectOption('32x32');
    
    // Step 4: Click "Generate"
    await page.getByTestId('generate-btn').click();
    
    // Step 5: Wait for map to render
    await page.waitForSelector('[data-testid="map-canvas"]', { timeout: 30000 });
    
    // Verify world name is displayed
    await expect(page.locator('[data-testid="world-name"]')).toContainText('World');
    
    console.log('✓ E2E-FRONT-001: World Generation UI Flow PASSED');
  });

  test('E2E-FRONT-001: Large World Generation (64x64)', async ({ page }) => {
    // Generate a large world and verify it completes
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('12345');
    await page.getByTestId('size-select').selectOption('64x64');
    await page.getByTestId('generate-btn').click();
    
    // Wait for generation (longer timeout for large world)
    await page.waitForSelector('[data-testid="map-canvas"]', { timeout: 60000 });
    
    // Verify loading indicator is gone
    await expect(page.locator('[data-testid="loading-indicator"]')).not.toBeVisible();
    
    console.log('✓ Large World Generation PASSED');
  });
});

// =======================================================================
// E2E-FRONT-002: Map Overlay Toggle
// =======================================================================
test.describe('Map Overlays', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate and generate a world
    await page.goto('/');
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('42');
    await page.getByTestId('generate-btn').click();
    await page.waitForSelector('[data-testid="map-canvas"]', { timeout: 30000 });
  });

  test('E2E-FRONT-002: Map Overlay Toggle', async ({ page }) => {
    // Step 1: Click "Resources" overlay
    await page.getByTestId('overlay-resources').click();
    await expect(page.locator('[data-testid="resource-markers"]')).toBeVisible();
    
    // Step 2: Click "Elevation" overlay
    await page.getByTestId('overlay-elevation').click();
    await expect(page.locator('[data-testid="elevation-colors"]')).toBeVisible();
    
    // Step 3: Click "Political" overlay
    await page.getByTestId('overlay-political').click();
    await expect(page.locator('[data-testid="faction-boundaries"]')).toBeVisible();
    
    // Step 4: Verify only one overlay is active at a time
    const activeOverlays = await page.locator('[data-testid^="overlay-"][data-active="true"]').count();
    expect(activeOverlays).toBe(1);
    
    console.log('✓ E2E-FRONT-002: Map Overlay Toggle PASSED');
  });

  test('E2E-FRONT-002: Legend Updates on Overlay Change', async ({ page }) => {
    // Verify legend changes when overlay changes
    const legend = page.locator('[data-testid="overlay-legend"]');
    
    // Switch to Resources overlay
    await page.getByTestId('overlay-resources').click();
    await expect(legend).toContainText('Resources');
    
    // Switch to Elevation overlay
    await page.getByTestId('overlay-elevation').click();
    await expect(legend).toContainText('Elevation');
    
    // Switch to Political overlay
    await page.getByTestId('overlay-political').click();
    await expect(legend).toContainText('Factions');
    
    console.log('✓ Legend Updates PASSED');
  });
});

// =======================================================================
// E2E-FRONT-003: Region Click Tooltip
// =======================================================================
test.describe('Region Interaction', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('42');
    await page.getByTestId('generate-btn').click();
    await page.waitForSelector('[data-testid="map-canvas"]', { timeout: 30000 });
  });

  test('E2E-FRONT-003: Region Click Tooltip', async ({ page }) => {
    // Step 1: Hover over region polygon
    const region = page.locator('[data-testid="region-polygon"]').first();
    await region.hover();
    
    // Verify polygon is highlighted
    await expect(region).toHaveAttribute('data-highlighted', 'true');
    
    // Step 2: Click on region
    await region.click();
    
    // Step 3: Verify tooltip appears
    const tooltip = page.locator('[data-testid="region-tooltip"]');
    await expect(tooltip).toBeVisible();
    
    // Step 4: Verify tooltip content
    await expect(tooltip.locator('[data-testid="tooltip-name"]')).toBeVisible();
    await expect(tooltip.locator('[data-testid="tooltip-population"]')).toBeVisible();
    await expect(tooltip.locator('[data-testid="tooltip-faction"]')).toBeVisible();
    
    console.log('✓ E2E-FRONT-003: Region Click Tooltip PASSED');
  });

  test('E2E-FRONT-003: Tooltip Positioning', async ({ page }) => {
    // Click on a region near the edge of the viewport
    const edgeRegion = page.locator('[data-testid="region-polygon"]').last();
    await edgeRegion.click();
    
    const tooltip = page.locator('[data-testid="region-tooltip"]');
    
    // Verify tooltip is within viewport
    const tooltipBox = await tooltip.boundingBox();
    expect(tooltipBox?.x).toBeGreaterThanOrEqual(0);
    expect(tooltipBox?.y).toBeGreaterThanOrEqual(0);
    
    console.log('✓ Tooltip Positioning PASSED');
  });
});

// =======================================================================
// E2E-FRONT-004: Responsive Layout
// =======================================================================
test.describe('Responsive Layout', () => {
  test('E2E-FRONT-004: Desktop Layout (1920x1080)', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');
    
    // Verify map fills available space
    const map = page.locator('[data-testid="map-canvas"]');
    const mapBox = await map.boundingBox();
    expect(mapBox?.width).toBeGreaterThan(1000);
    
    console.log('✓ Desktop Layout PASSED');
  });

  test('E2E-FRONT-004: Tablet Layout (768x1024)', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');
    
    const map = page.locator('[data-testid="map-canvas"]');
    const mapBox = await map.boundingBox();
    expect(mapBox?.width).toBeGreaterThan(500);
    
    console.log('✓ Tablet Layout PASSED');
  });

  test('E2E-FRONT-004: Mobile Layout (375x667)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    
    // Verify controls are stacked vertically
    const controls = page.locator('[data-testid="map-controls"]');
    await expect(controls).toBeVisible();
    
    // Verify map is still usable
    const map = page.locator('[data-testid="map-canvas"]');
    await expect(map).toBeVisible();
    
    console.log('✓ Mobile Layout PASSED');
  });
});

// =======================================================================
// E2E-FRONT-005: Keyboard Navigation
// =======================================================================
test.describe('Keyboard Navigation', () => {
  test('E2E-FRONT-005: Keyboard Navigation', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('42');
    await page.getByTestId('generate-btn').click();
    await page.waitForSelector('[data-testid="map-canvas"]', { timeout: 30000 });
    
    // Step 1: Tab to overlay toggle controls
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    
    // Verify focus indicator
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toHaveAttribute('data-testid', /overlay-/);
    
    // Step 2: Use Enter/Space to toggle
    await page.keyboard.press('Enter');
    await expect(page.locator('[data-testid="overlay-resources"]')).toHaveAttribute('data-active', 'true');
    
    // Step 3: Tab to region polygons
    for (let i = 0; i < 5; i++) {
      await page.keyboard.press('Tab');
    }
    
    // Step 4: Use Enter to select region
    await page.keyboard.press('Enter');
    await expect(page.locator('[data-testid="region-tooltip"]')).toBeVisible();
    
    console.log('✓ E2E-FRONT-005: Keyboard Navigation PASSED');
  });
});

// =======================================================================
// E2E-ERR-001: Error Handling — Network Failure
// =======================================================================
test.describe('Error Handling', () => {
  test('E2E-ERR-001: Network Failure During Generation', async ({ page }) => {
    // Intercept network requests to simulate failure
    await page.route('**/api/worlds', route => {
      route.abort('failed');
    });
    
    await page.goto('/');
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('42');
    await page.getByTestId('generate-btn').click();
    
    // Verify error message is displayed
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).toContainText('network');
    
    // Verify no crash (page still functional)
    await expect(page.locator('[data-testid="retry-btn"]')).toBeVisible();
    
    console.log('✓ E2E-ERR-001: Network Failure Handling PASSED');
  });

  test('E2E-ERR-002: Generation Timeout', async ({ page }) => {
    // Slow down generation to trigger timeout
    await page.route('**/api/worlds', route => {
      route.delay(60000); // 60 second delay
    });
    
    await page.goto('/');
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('42');
    await page.getByTestId('generate-btn').click();
    
    // Wait for timeout (30 seconds)
    await page.waitForSelector('[data-testid="timeout-message"]', { timeout: 35000 });
    
    await expect(page.locator('[data-testid="timeout-message"]')).toBeVisible();
    
    console.log('✓ Generation Timeout Handling PASSED');
  });
});

// =======================================================================
// Performance Tests
// =======================================================================
test.describe('Performance', () => {
  test('Generation completes within time limit', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto('/');
    await page.getByTestId('new-world-btn').click();
    await page.getByTestId('seed-input').fill('42');
    await page.getByTestId('size-select').selectOption('64x64');
    await page.getByTestId('generate-btn').click();
    await page.waitForSelector('[data-testid="map-canvas"]', { timeout: 60000 });
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    // 64x64 world should complete in under 60 seconds
    expect(duration).toBeLessThan(60000);
    
    console.log(`✓ Generation completed in ${duration}ms`);
  });
});
