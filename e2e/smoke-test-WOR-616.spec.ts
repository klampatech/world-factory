import { test, expect } from '@playwright/test';

/**
 * WOR-616 Smoke Test - Backend & Frontend Health Verification
 * 
 * Tests the current state of the World Factory application after WOR-468
 * World Selector landing page integration.
 */

test.describe('WOR-616 Smoke Test - Backend & Frontend Health', () => {
  
  test.beforeEach(async ({ page }) => {
    // Capture console errors for analysis
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.log(`[CONSOLE ERROR] ${msg.text()}`);
      }
    });
  });

  test('TC-001: Backend health check', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8080/health');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('ok');
    expect(data.version).toBeDefined();
    console.log('✅ Backend health: ' + JSON.stringify(data));
  });

  test('TC-002: Backend worlds list endpoint', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8080/api/v1/worlds');
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    expect(Array.isArray(data.data.worlds)).toBeTruthy();
    console.log('✅ Backend worlds list: ' + data.data.totalWorlds + ' worlds');
  });

  test('TC-003: Backend world creation endpoint', async ({ request }) => {
    const testWorld = {
      name: 'WOR-616 Test World',
      width: 64,
      height: 64,
      seed: 616616,
      config: {
        elevation_scale: 1.0,
        temperature_scale: 1.0,
        moisture_scale: 1.0
      }
    };
    
    const response = await request.post('http://127.0.0.1:8080/api/v1/worlds', {
      data: testWorld,
      headers: { 'Content-Type': 'application/json' }
    });
    
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    // Response structure varies; handle both world:id and id formats
    const worldId = data.data.id || data.data.world?.id;
    expect(worldId).toBeDefined();
    expect(worldId.toString()).toMatch(/^(world:)?[a-f0-9-]+$/i);
    
    // Cleanup: Delete the test world
    const deleteRes = await request.delete(`http://127.0.0.1:8080/api/v1/worlds/${worldId}`);
    expect(deleteRes.ok() || deleteRes.status() === 204).toBeTruthy();
    
    console.log('✅ World creation: ' + worldId);
  });

  test('TC-004: Backend API integration file accessible', async ({ request }) => {
    // The frontend serves api-integration.js from its static file server
    const response = await request.get('http://localhost:8765/api-integration.js');
    expect(response.ok()).toBeTruthy();
    const content = await response.text();
    expect(content).toContain('API_BASE_URL');
    expect(content).toContain('WorldApiClient');
    console.log('✅ API integration file accessible');
  });

  test('TC-005: Frontend landing page loads', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    
    // Check title
    const title = await page.title();
    expect(title).toContain('World Selector');
    
    // Check header is present
    const header = page.locator('header');
    await expect(header).toBeVisible();
    
    // Check for server status indicator
    const serverStatus = page.locator('.server-status, .status-indicator');
    const statusVisible = await serverStatus.count() > 0;
    
    console.log('✅ Frontend landing page loads (title: ' + title + ', status indicator: ' + (statusVisible ? 'present' : 'not present') + ')');
  });

  test('TC-006: Frontend page structure', async ({ page }) => {
    await page.goto('http://localhost:8765');
    await page.waitForLoadState('networkidle');
    
    // Check for body content
    const bodyText = await page.locator('body').innerText();
    expect(bodyText.length).toBeGreaterThan(10);
    
    // Log page structure
    const heading = page.locator('h1, h2').first();
    if (await heading.count() > 0) {
      console.log('✅ Page heading: ' + await heading.textContent());
    }
    
    console.log('✅ Frontend structure verified');
  });
});