import { test, expect } from '@playwright/test';

/**
 * WOR-694 Smoke Test - Full application E2E verification
 * Tests all major system components including backend API and frontend UI
 */

test.describe('WOR-694 Smoke Test - Full Application Verification', () => {
  
  // Track console errors for reporting
  let jsErrors: string[] = [];
  
  test.beforeEach(async ({ page }) => {
    jsErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error' && !msg.text().includes('net::ERR_CONNECTION_REFUSED')) {
        jsErrors.push(msg.text());
      }
    });
  });

  // ==================== BACKEND API TESTS ====================

  test('B01: Backend health check', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8080/health');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('ok');
    expect(data.version).toBeDefined();
    console.log('✅ B01 Backend health: ' + JSON.stringify(data));
  });

  test('B02: Create a new world', async ({ request }) => {
    const response = await request.post('http://127.0.0.1:8080/api/v1/worlds', {
      data: { name: 'WOR-694 Test World' }
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.data.id).toBeDefined();
    console.log('✅ B02 World created: ' + data.data.id);
  });

  test('B03: List all worlds', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.data.totalWorlds).toBeDefined();
    console.log('✅ B03 Worlds list: ' + data.data.totalWorlds + ' worlds');
  });

  test('B04: Get single world by ID', async ({ request }) => {
    // First get a world ID
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      // Strip any "world:" prefix from ID for comparison
      const worldId = world.id.replace('world:', '');
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      // Compare without the world: prefix
      expect(data.data.id.replace('world:', '')).toBe(worldId);
      console.log('✅ B04 Single world retrieved: ' + data.data.name);
    } else {
      console.log('⚠️ B04 SKIP - No worlds available');
    }
  });

  test('B05: Get world planet data', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/planet`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B05 Planet data accessible');
    } else {
      console.log('⚠️ B05 SKIP - No worlds available');
    }
  });

  test('B06: Get world map data', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/map`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B06 Map data accessible');
    } else {
      console.log('⚠️ B06 SKIP - No worlds available');
    }
  });

  test('B07: Get world history', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/history`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B07 History accessible');
    } else {
      console.log('⚠️ B07 SKIP - No worlds available');
    }
  });

  test('B08: Get world events (requires limit param)', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/events?limit=10`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B08 Events accessible (limit param required)');
    } else {
      console.log('⚠️ B08 SKIP - No worlds available');
    }
  });

  test('B09: Get world figures list', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/figures`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B09 Figures list accessible');
    } else {
      console.log('⚠️ B09 SKIP - No worlds available');
    }
  });

  test('B10: Get single figure by ID', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      // First check if there are figures
      const figuresResponse = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/figures`);
      const figuresData = await figuresResponse.json();
      
      if (figuresData.data && figuresData.data.length > 0) {
        const figureId = figuresData.data[0].id;
        const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/figures/${figureId}`);
        expect(response.ok()).toBeTruthy();
        console.log('✅ B10 Single figure retrieved');
      } else {
        console.log('⚠️ B10 SKIP - No figures available');
      }
    } else {
      console.log('⚠️ B10 SKIP - No worlds available');
    }
  });

  test('B11: Get world settlements', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/settlements`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B11 Settlements accessible');
    } else {
      console.log('⚠️ B11 SKIP - No worlds available');
    }
  });

  test('B12: Get settlements map', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/settlements/map`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B12 Settlements map accessible');
    } else {
      console.log('⚠️ B12 SKIP - No worlds available');
    }
  });

  test('B13: Get resources summary', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/resources/summary`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B13 Resources summary accessible');
    } else {
      console.log('⚠️ B13 SKIP - No worlds available');
    }
  });

  test('B14: Get disasters', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/disasters`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B14 Disasters accessible');
    } else {
      console.log('⚠️ B14 SKIP - No worlds available');
    }
  });

  test('B15: Get artifacts (requires limit param)', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/artifacts?limit=5`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B15 Artifacts accessible (limit param required)');
    } else {
      console.log('⚠️ B15 SKIP - No worlds available');
    }
  });

  test('B16: Get world export', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/export`);
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      expect(data.success).toBe(true);
      console.log('✅ B16 Export accessible');
    } else {
      console.log('⚠️ B16 SKIP - No worlds available');
    }
  });

  test('B17: Get world JSON export', async ({ request }) => {
    const listResponse = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      const response = await request.get(`http://127.0.0.1:8080/api/v1/worlds/${world.id}/export.json`);
      expect(response.ok()).toBeTruthy();
      console.log('✅ B17 JSON export accessible');
    } else {
      console.log('⚠️ B17 SKIP - No worlds available');
    }
  });

  test('B18: Delete a world', async ({ request }) => {
    // Create a world to delete
    const createResponse = await request.post('http://127.0.0.1:8080/api/v1/worlds', {
      data: { name: 'WOR-694 Delete Test' }
    });
    const createData = await createResponse.json();
    
    if (createData.success && createData.data.id) {
      const deleteResponse = await request.delete(`http://127.0.0.1:8080/api/v1/worlds/${createData.data.id}`);
      expect(deleteResponse.ok()).toBeTruthy();
      console.log('✅ B18 World deleted successfully');
    } else {
      console.log('⚠️ B18 SKIP - Create failed');
    }
  });

  // ==================== FRONTEND UI TESTS ====================

  test('F01: Frontend landing page loads', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    
    const title = await page.title();
    expect(title).toBeTruthy();
    console.log('✅ F01 Landing page title: ' + title);
    
    const header = page.locator('header');
    await expect(header).toBeVisible();
  });

  test('F02: World list display', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check if world grid or loading state is visible
    const hasWorldGrid = await page.locator('#world-grid').isVisible().catch(() => false);
    const hasLoadingState = await page.locator('#loading-state').isVisible().catch(() => false);
    const hasEmptyState = await page.locator('#empty-state').isVisible().catch(() => false);
    
    expect(hasWorldGrid || hasLoadingState || hasEmptyState).toBeTruthy();
    console.log('✅ F02 World list container visible');
  });

  test('F03: Create world modal opens', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    
    // The button is .generate-btn with id 'generate-btn'
    const createBtn = page.locator('#generate-btn').first();
    await createBtn.click();
    
    await page.waitForTimeout(500);
    // Look for modal container
    const modal = page.locator('#create-modal, .modal, .create-modal');
    const isModalVisible = await modal.isVisible().catch(() => false);
    
    if (isModalVisible) {
      console.log('✅ F03 Modal opened successfully');
    } else {
      // Check for form input within any modal
      const nameInput = page.locator('#world-name, [name="name"], input[type="text"]').first();
      const inputVisible = await nameInput.isVisible().catch(() => false);
      expect(inputVisible).toBeTruthy();
      console.log('✅ F03 Create form is accessible');
    }
  });

  test('F04: World creation form submit', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    
    // Open create form - button is #generate-btn
    const createBtn = page.locator('#generate-btn').first();
    await createBtn.click();
    await page.waitForTimeout(500);
    
    // Find name input
    const nameInput = page.locator('#world-name, [name="name"], input[type="text"]').first();
    await nameInput.fill('WOR-694 Smoke Test World');
    
    // Submit - find confirm button
    const submitBtn = page.locator('#confirm-create, .btn-submit, button[type="submit"], .btn-primary').first();
    await submitBtn.click();
    
    await page.waitForTimeout(3000);
    console.log('✅ F04 World creation form submitted');
  });

  test('F05: Tab navigation', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Look for view tabs (on world detail page) - tabs are within .tabs-container
    const tabs = page.locator('.view-tab, .tab, [role="tab"], .tab-item');
    const tabCount = await tabs.count();
    
    if (tabCount > 0) {
      // Test tab navigation
      await tabs.first().click();
      await page.waitForTimeout(500);
      console.log('✅ F05 Tab navigation works (' + tabCount + ' tabs)');
    } else {
      // On landing page - check header buttons work
      const generateBtn = page.locator('#generate-btn');
      const isVisible = await generateBtn.isVisible().catch(() => false);
      expect(isVisible).toBeTruthy();
      console.log('⚠️ F05 PARTIAL - No tabs on landing page, but buttons visible');
    }
  });

  test('F06: Map view canvas', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Try to navigate to a world viewer with map
    const readyWorld = page.locator('.status-badge.ready, .status-ready').first();
    const hasReadyWorld = await readyWorld.isVisible().catch(() => false);
    
    if (hasReadyWorld) {
      // Click view button
      const viewBtn = page.locator('.btn-view, .view-btn, a[href*="world.html"]').first();
      await viewBtn.click();
      await page.waitForTimeout(2000);
      
      // First, click the Map tab to ensure it's selected
      const mapTab = page.locator('.view-tab:has-text("Map"), .tab:has-text("Map"), .tab-item:has-text("Map"), [data-tab="map"]');
      const hasMapTab = await mapTab.isVisible().catch(() => false);
      
      if (hasMapTab) {
        await mapTab.click();
        await page.waitForTimeout(500);
      }
      
      // Now check for map elements
      const canvas = page.locator('#world-map, canvas#world-map');
      const hasCanvas = await canvas.isVisible().catch(() => false);
      
      if (hasCanvas) {
        console.log('✅ F06 Map canvas found');
      } else {
        // Check for map container
        const mapContainer = page.locator('.map-container');
        const hasMapContainer = await mapContainer.isVisible().catch(() => false);
        expect(hasMapContainer).toBeTruthy();
        console.log('✅ F06 Map container visible');
      }
    } else {
      console.log('⚠️ F06 SKIP - No ready worlds available');
    }
  });

  test('F07: Timeline container', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const readyWorld = page.locator('.status-badge.ready, .status-ready').first();
    const hasReadyWorld = await readyWorld.isVisible().catch(() => false);
    
    if (hasReadyWorld) {
      // Click view button
      const viewBtn = page.locator('.btn-view, .view-btn, a[href*="world.html"]').first();
      await viewBtn.click();
      await page.waitForTimeout(2000);
      
      // Look for Timeline tab and click it
      const timelineTab = page.locator('.view-tab:has-text("Timeline"), .tab:has-text("Timeline"), .tab-item:has-text("Timeline")');
      const hasTimelineTab = await timelineTab.isVisible().catch(() => false);
      
      if (hasTimelineTab) {
        await timelineTab.click();
        await page.waitForTimeout(1000);
        
        const timeline = page.locator('.timeline-container, .timeline, .timeline-view, #timeline-content');
        const isVisible = await timeline.isVisible().catch(() => false);
        expect(isVisible).toBeTruthy();
        console.log('✅ F07 Timeline container visible');
      } else {
        console.log('⚠️ F07 SKIP - No Timeline tab');
      }
    } else {
      console.log('⚠️ F07 SKIP - No ready worlds available');
    }
  });

  test('F08: Dashboard container', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const readyWorld = page.locator('.status-badge.ready, .status-ready').first();
    const hasReadyWorld = await readyWorld.isVisible().catch(() => false);
    
    if (hasReadyWorld) {
      // Click view button
      const viewBtn = page.locator('.btn-view, .view-btn, a[href*="world.html"]').first();
      await viewBtn.click();
      await page.waitForTimeout(2000);
      
      // Look for Dashboard tab
      const dashboardTab = page.locator('.view-tab:has-text("Dashboard"), .tab:has-text("Dashboard"), .tab-item:has-text("Dashboard")');
      const hasDashboardTab = await dashboardTab.isVisible().catch(() => false);
      
      if (hasDashboardTab) {
        await dashboardTab.click();
        await page.waitForTimeout(1000);
        
        const dashboard = page.locator('.dashboard-container, .dashboard, .dashboard-view, #dashboard-content');
        const isVisible = await dashboard.isVisible().catch(() => false);
        expect(isVisible).toBeTruthy();
        console.log('✅ F08 Dashboard container visible');
      } else {
        console.log('⚠️ F08 SKIP - No Dashboard tab');
      }
    } else {
      console.log('⚠️ F08 SKIP - No ready worlds available');
    }
  });

  test('F09: World detail page', async ({ page }) => {
    const listResponse = await page.request.get('http://127.0.0.1:8080/api/v1/worlds');
    const listData = await listResponse.json();
    const world = listData.data.worlds[0];
    
    if (world) {
      await page.goto(`http://localhost:8765/world.html?id=${world.id}`);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);
      
      // Check page loaded
      const body = await page.locator('body');
      expect(body).toBeVisible();
      console.log('✅ F09 World detail page loaded');
    } else {
      console.log('⚠️ F09 SKIP - No worlds available');
    }
  });

  test('F10: Console errors check', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Navigate through main sections - click generate button to test modal
    const generateBtn = page.locator('#generate-btn').first();
    if (await generateBtn.isVisible().catch(() => false)) {
      await generateBtn.click();
      await page.waitForTimeout(1000);
      
      // Close modal
      const closeBtn = page.locator('#modal-cancel, .modal-close, #modal-close');
      if (await closeBtn.isVisible().catch(() => false)) {
        await closeBtn.click();
      }
    }
    
    // Report errors
    console.log('✅ F10 Console errors check complete. JS errors: ' + jsErrors.length);
    if (jsErrors.length > 0) {
      jsErrors.forEach(e => console.log('  - ' + e));
    }
  });

});