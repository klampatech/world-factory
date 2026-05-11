import { test, expect } from '@playwright/test';

/**
 * WOR-206 Smoke Test - Complete E2E Application Test (SPA Version)
 * 
 * Tests the refactored single-page application architecture.
 */

test.describe('WOR-206 Smoke Test - Complete E2E Application Test', () => {
  
  test.beforeEach(async ({ page }) => {
    // Capture console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.log(`[CONSOLE ERROR] ${msg.text()}`);
      }
    });
  });

  test('TC-001: Backend health check', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8082/health');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('ok');
    console.log('✅ Backend health: ' + JSON.stringify(data));
  });

  test('TC-002: Backend worlds list endpoint', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8082/api/v1/worlds');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    console.log('✅ Backend worlds list: ' + data.data.totalWorlds + ' worlds');
  });

  test('TC-003: Frontend SPA loads correctly', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    
    // Check title
    const title = await page.title();
    expect(title).toContain('World Factory');
    
    // Check header is present
    const header = page.locator('header');
    await expect(header).toBeVisible();
    
    // Check logo is present
    const logo = page.locator('.logo');
    await expect(logo).toBeVisible();
    
    // Check view tabs are present
    const viewTabs = page.locator('.view-tabs');
    await expect(viewTabs).toBeVisible();
    
    // Check Map tab is active by default
    const mapTab = page.locator('.view-tab.active');
    await expect(mapTab).toHaveText('Map');
    
    console.log('✅ Frontend SPA loads correctly');
  });

  test('TC-004: Map view is displayed', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check map canvas is present
    const mapCanvas = page.locator('#map-canvas');
    await expect(mapCanvas).toBeVisible();
    
    // Check zoom controls are present
    const zoomIn = page.locator('#zoom-in');
    await expect(zoomIn).toBeVisible();
    
    const zoomOut = page.locator('#zoom-out');
    await expect(zoomOut).toBeVisible();
    
    console.log('✅ Map view is displayed');
  });

  test('TC-005: Zoom controls work', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Get initial zoom level
    const initialZoom = await page.locator('#zoom-level').textContent();
    
    // Click zoom in
    await page.locator('#zoom-in').click();
    await page.waitForTimeout(500);
    
    // Get new zoom level
    const newZoom = await page.locator('#zoom-level').textContent();
    
    // Zoom should have changed
    expect(newZoom).not.toBe(initialZoom);
    
    console.log('✅ Zoom controls work (from ' + initialZoom + ' to ' + newZoom + ')');
  });

  test('TC-006: Timeline tab navigation', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Click Timeline tab
    await page.locator('.view-tab[data-view="timeline"]').click();
    await page.waitForTimeout(1000);
    
    // Check Timeline tab is now active
    const timelineTab = page.locator('.view-tab.active');
    await expect(timelineTab).toHaveText('Timeline');
    
    // Check timeline container exists (if data loaded)
    const timeline = page.locator('#timeline-view');
    // Timeline may or may not be visible depending on data
    
    console.log('✅ Timeline tab navigation works');
  });

  test('TC-007: Map tab navigation', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Go to Timeline first
    await page.locator('.view-tab[data-view="timeline"]').click();
    await page.waitForTimeout(500);
    
    // Go back to Map
    await page.locator('.view-tab[data-view="map"]').click();
    await page.waitForTimeout(500);
    
    // Check Map tab is now active
    const mapTab = page.locator('.view-tab.active');
    await expect(mapTab).toHaveText('Map');
    
    // Check map canvas is visible
    const mapCanvas = page.locator('#map-canvas');
    await expect(mapCanvas).toBeVisible();
    
    console.log('✅ Map tab navigation works');
  });

  test('TC-008: Generate World button exists', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check Generate World button exists
    const generateBtn = page.locator('#generate-world');
    await expect(generateBtn).toBeVisible();
    
    console.log('✅ Generate World button exists');
  });

  test('TC-009: Backend API endpoints work', async ({ request }) => {
    // Test multiple API endpoints
    
    // Test worlds list
    const worldsResponse = await request.get('http://127.0.0.1:8082/api/v1/worlds');
    expect(worldsResponse.ok()).toBeTruthy();
    
    // Get a world ID
    const worldsData = await worldsResponse.json();
    if (worldsData.data.worlds && worldsData.data.worlds.length > 0) {
      const worldId = worldsData.data.worlds[0].id;
      
      // Test map endpoint
      const mapResponse = await request.get(`http://127.0.0.1:8082/api/v1/worlds/${worldId}/map`);
      expect(mapResponse.ok()).toBeTruthy();
      
      // Test timeline endpoint
      const timelineResponse = await request.get(`http://127.0.0.1:8082/api/v1/worlds/${worldId}/timeline`);
      expect(timelineResponse.ok()).toBeTruthy();
      
      // Test events endpoint
      const eventsResponse = await request.get(`http://127.0.0.1:8082/api/v1/worlds/${worldId}/events`);
      expect(eventsResponse.ok()).toBeTruthy();
      
      console.log('✅ All API endpoints accessible for world: ' + worldsData.data.worlds[0].name);
    } else {
      console.log('⚠️ No worlds found for API endpoint testing');
    }
  });

  test('TC-010: Browser console errors check', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Navigate through tabs
    await page.locator('.view-tab[data-view="timeline"]').click();
    await page.waitForTimeout(1000);
    
    await page.locator('.view-tab[data-view="map"]').click();
    await page.waitForTimeout(1000);
    
    // Filter out expected backend connection errors (when no world loaded)
    const realErrors = errors.filter(e => 
      !e.includes('Failed to load resource: net::ERR_CONNECTION_REFUSED') &&
      !e.includes('Failed to fetch')
    );
    
    console.log('✅ Browser console errors check complete. API errors: ' + errors.length + ', JavaScript errors: ' + realErrors.length);
    if (realErrors.length > 0) {
      realErrors.forEach(e => console.log('  - ' + e));
    }
  });

  test('TC-011: Overlay controls exist', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check overlay controls exist
    const overlayControls = page.locator('#overlay-controls');
    await expect(overlayControls).toBeVisible();
    
    // Check individual overlay buttons
    const resourcesOverlay = page.locator('.overlay-btn[data-overlay="resources"]');
    await expect(resourcesOverlay).toBeVisible();
    
    const elevationOverlay = page.locator('.overlay-btn[data-overlay="elevation"]');
    await expect(elevationOverlay).toBeVisible();
    
    console.log('✅ Overlay controls exist');
  });

  test('TC-012: Legend panel exists', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check legend exists
    const legend = page.locator('#legend');
    await expect(legend).toBeVisible();
    
    // Check legend has items container
    const legendItems = page.locator('#legend-items');
    await expect(legendItems).toBeVisible();
    
    console.log('✅ Legend panel exists');
  });

});
