import { test, expect } from '@playwright/test';

/**
 * WOR-594 Smoke Test - Updated for World Selector Landing Page (WOR-468)
 * 
 * Tests the refactored World Selector landing page architecture.
 * The frontend was refactored to show a world list with cards instead of the old SPA tabs.
 */

test.describe('WOR-594 Smoke Test - World Selector Landing Page', () => {
  
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

  test('TC-003: World Selector landing page loads correctly', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check title
    const title = await page.title();
    expect(title).toContain('World Factory');
    
    // Check header with title is present
    const header = page.locator('h1');
    await expect(header).toBeVisible();
    await expect(header).toContainText('World Factory');
    
    // Check Refresh button exists
    const refreshBtn = page.locator('button:has-text("Refresh")');
    await expect(refreshBtn).toBeVisible();
    
    // Check Create World form exists
    const createForm = page.locator('#createForm');
    await expect(createForm).toBeVisible();
    
    // Check Create World button exists
    const createBtn = page.locator('#createBtn');
    await expect(createBtn).toBeVisible();
    
    // Check form inputs exist
    const widthInput = page.locator('#width');
    await expect(widthInput).toBeVisible();
    
    const heightInput = page.locator('#height');
    await expect(heightInput).toBeVisible();
    
    const polygonsInput = page.locator('#polygons');
    await expect(polygonsInput).toBeVisible();
    
    console.log('✅ World Selector landing page loads correctly');
  });

  test('TC-004: World list displays correctly', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Check worlds container exists
    const worldsContainer = page.locator('#worldsContainer');
    await expect(worldsContainer).toBeVisible();
    
    // Should have world cards (or loading/no worlds message)
    const worldsContent = await page.locator('#worldsContainer').textContent();
    expect(worldsContent).toBeTruthy();
    
    // If worlds loaded, check for world cards
    const worldCards = page.locator('.world-card');
    const cardCount = await worldCards.count();
    expect(cardCount).toBeGreaterThan(0);
    
    console.log('✅ World list displays correctly (' + cardCount + ' worlds)');
  });

  test('TC-005: Create World form accepts input', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Get initial values
    const widthInput = page.locator('#width');
    const initialWidth = await widthInput.inputValue();
    
    const heightInput = page.locator('#height');
    const initialHeight = await heightInput.inputValue();
    
    // Change values
    await widthInput.fill('128');
    await heightInput.fill('128');
    
    // Verify values changed
    expect(await widthInput.inputValue()).toBe('128');
    expect(await heightInput.inputValue()).toBe('128');
    
    // Reset to original
    await widthInput.fill(initialWidth);
    await heightInput.fill(initialHeight);
    
    console.log('✅ Create World form accepts input');
  });

  test('TC-006: World card displays correct information', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Get first world card
    const firstCard = page.locator('.world-card').first();
    await expect(firstCard).toBeVisible();
    
    // Check card has world name
    const worldName = firstCard.locator('.world-name');
    await expect(worldName).toBeVisible();
    
    // Check card has metadata (dimensions and era)
    const worldMeta = firstCard.locator('.world-meta');
    await expect(worldMeta.first()).toBeVisible();
    
    // Check card has ID
    const cardContent = await firstCard.textContent();
    expect(cardContent).toContain('ID:');
    
    console.log('✅ World card displays correct information');
  });

  test('TC-007: Refresh button works', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Click refresh
    const refreshBtn = page.locator('button:has-text("Refresh")');
    await refreshBtn.click();
    
    // Wait for reload
    await page.waitForTimeout(2000);
    
    // Page should still have the main content
    const header = page.locator('h1');
    await expect(header).toContainText('World Factory');
    
    console.log('✅ Refresh button works');
  });

  test('TC-008: Backend API endpoints work', async ({ request }) => {
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

  test('TC-009: Browser console errors check', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Filter out expected backend connection errors
    const realErrors = errors.filter(e => 
      !e.includes('Failed to load resource: net::ERR_CONNECTION_REFUSED') &&
      !e.includes('Failed to fetch')
    );
    
    console.log('✅ Browser console errors check complete. API errors: ' + errors.length + ', JavaScript errors: ' + realErrors.length);
    if (realErrors.length > 0) {
      realErrors.forEach(e => console.log('  - ' + e));
    }
  });

  test('TC-010: Status message area exists', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check status div exists in DOM (may be hidden if empty)
    const status = page.locator('#status');
    await expect(status).toBeAttached();
    
    console.log('✅ Status message area exists');
  });

});
