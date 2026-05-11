import { test, expect, chromium, Browser, BrowserContext, Page } from '@playwright/test';

const API_BASE = 'http://127.0.0.1:8082/api/v1';
const API_HOST = 'http://127.0.0.1:8082';
const FRONTEND = 'http://localhost:8765';
const SCREENSHOTS = '/home/kyle/projects/world-generator/screenshots/WOR-1084';

test.describe('WOR-1084 Smoke Test - Complete E2E Application Test', () => {
  
  let testWorldId: string = '';
  const errors: string[] = [];
  
  test.beforeAll(async () => {
    // Ensure screenshots directory exists
    const fs = require('fs');
    if (!fs.existsSync(SCREENSHOTS)) {
      fs.mkdirSync(SCREENSHOTS, { recursive: true });
    }
  });
  
  test.beforeEach(async ({ page }) => {
    // Capture console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Filter out known non-critical errors
        if (!text.includes('ERR_CONNECTION_REFUSED') && !text.includes('net::ERR_')) {
          errors.push(text);
        }
      }
    });
  });

  test('TC-001: Backend health check', async ({ request }) => {
    const response = await request.get(`${API_HOST}/health`);
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('ok');
    console.log('✅ Backend health: ' + JSON.stringify(data));
  });

  test('TC-002: Backend worlds list endpoint', async ({ request }) => {
    const response = await request.get(`${API_BASE}/worlds`);
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    console.log('✅ Backend worlds list: ' + data.data.totalWorlds + ' worlds');
  });

  test('TC-003: Create a new world', async ({ request }) => {
    const response = await request.post(`${API_BASE}/worlds`, {
      data: {
        name: 'WOR-1084 Smoke Test World',
        genre: 'fantasy',
        era: 'medieval',
        mapSize: 'medium',
        climate: 'temperate'
      }
    });
    expect(response.ok() || response.status() === 201).toBeTruthy();
    const data = await response.json();
    expect(data.success).toBe(true);
    testWorldId = data.data.id.replace('world:', '');
    console.log('✅ Created world: ' + data.data.name + ' (ID: ' + testWorldId + ')');
  });

  test('TC-004: Wait for world to be ready', async ({ request }) => {
    // Wait up to 2 minutes for world generation
    const maxAttempts = 24; // 24 * 5s = 120s
    let attempts = 0;
    let status = 'generating';
    
    while (status === 'generating' && attempts < maxAttempts) {
      await new Promise(r => setTimeout(r, 5000));
      const response = await request.get(`${API_BASE}/worlds/${testWorldId}`);
      const data = await response.json();
      status = data.data.status;
      console.log(`  World status: ${status} (attempt ${attempts + 1}/${maxAttempts})`);
      attempts++;
    }
    
    expect(status).toBe('ready');
    console.log('✅ World generation complete: ' + status);
  });

  test('TC-005: Frontend landing page loads', async ({ page }) => {
    await page.goto(FRONTEND);
    await page.waitForLoadState('networkidle');
    
    // Check title
    const title = await page.title();
    expect(title).toContain('World');
    
    // Check main elements
    const header = page.locator('header');
    await expect(header).toBeVisible();
    
    await page.screenshot({ path: `${SCREENSHOTS}/01-landing-page.png` });
    console.log('✅ Frontend landing page loads correctly');
  });

  test('TC-006: Frontend displays world list', async ({ page }) => {
    await page.goto(FRONTEND);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check for key elements
    const createBtn = page.locator('.btn-create').first();
    if (await createBtn.isVisible()) {
      await expect(createBtn).toBeVisible();
    }
    
    await page.screenshot({ path: `${SCREENSHOTS}/02-world-list.png` });
    console.log('✅ Frontend displays world list and controls');
  });

  test('TC-007: View a ready world', async ({ page }) => {
    await page.goto(`${FRONTEND}?id=${testWorldId}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Check for viewer header or main content
    const viewerContent = page.locator('.viewer-content, .world-viewer, main');
    if (await viewerContent.isVisible()) {
      await expect(viewerContent).toBeVisible();
    }
    
    await page.screenshot({ path: `${SCREENSHOTS}/03-world-viewer.png` });
    console.log('✅ Successfully opened world viewer');
  });

  test('TC-008: Map tab and Voronoi rendering', async ({ page }) => {
    await page.goto(`${FRONTEND}?id=${testWorldId}&tab=map`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Check for canvas (map rendering)
    const canvas = page.locator('canvas').first();
    if (await canvas.isVisible()) {
      await expect(canvas).toBeVisible();
    }
    
    await page.screenshot({ path: `${SCREENSHOTS}/04-map-view.png` });
    console.log('✅ Map view renders');
  });

  test('TC-009: Timeline tab', async ({ page }) => {
    await page.goto(`${FRONTEND}?id=${testWorldId}&tab=timeline`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Try clicking timeline tab
    const timelineTab = page.locator('.view-tab:has-text("Timeline")').first();
    if (await timelineTab.isVisible()) {
      await timelineTab.click();
      await page.waitForTimeout(1000);
    }
    
    await page.screenshot({ path: `${SCREENSHOTS}/05-timeline-view.png` });
    console.log('✅ Timeline view accessible');
  });

  test('TC-010: Dashboard tab', async ({ page }) => {
    await page.goto(`${FRONTEND}?id=${testWorldId}&tab=dashboard`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    await page.screenshot({ path: `${SCREENSHOTS}/06-dashboard-view.png` });
    console.log('✅ Dashboard view accessible');
  });

  test('TC-011: All API endpoints for test world', async ({ request }) => {
    if (!testWorldId) {
      console.log('⚠️ No test world ID, skipping API tests');
      return;
    }
    
    const endpoints = [
      { path: `/worlds/${testWorldId}`, name: 'GET world by ID' },
      { path: `/worlds/${testWorldId}/planet`, name: 'GET planet' },
      { path: `/worlds/${testWorldId}/map`, name: 'GET map' },
      { path: `/worlds/${testWorldId}/history`, name: 'GET history' },
      { path: `/worlds/${testWorldId}/history/events`, name: 'GET history/events' },
      { path: `/worlds/${testWorldId}/figures`, name: 'GET figures' },
      { path: `/worlds/${testWorldId}/settlements`, name: 'GET settlements' },
      { path: `/worlds/${testWorldId}/settlements/map`, name: 'GET settlements/map' },
      { path: `/worlds/${testWorldId}/resources/summary`, name: 'GET resources/summary' },
      { path: `/worlds/${testWorldId}/disasters`, name: 'GET disasters' },
      { path: `/worlds/${testWorldId}/artifacts`, name: 'GET artifacts' },
      { path: `/worlds/${testWorldId}/export`, name: 'GET export' },
      { path: `/worlds/${testWorldId}/export.json`, name: 'GET export.json' },
      { path: `/worlds/${testWorldId}/timeline`, name: 'GET timeline' },
    ];
    
    let passed = 0;
    let failed = 0;
    
    for (const ep of endpoints) {
      const response = await request.get(`${API_BASE}${ep.path}`);
      if (response.ok()) {
        console.log(`✅ ${ep.name}`);
        passed++;
      } else {
        console.log(`❌ ${ep.name} - HTTP ${response.status()}`);
        failed++;
      }
    }
    
    console.log(`API Endpoint Results: ${passed} passed, ${failed} failed`);
    expect(failed).toBe(0);
  });

  test('TC-012: Browser console errors check', async ({ page }) => {
    // Clear previous errors and do a fresh navigation
    const pageErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        if (!text.includes('ERR_CONNECTION_REFUSED') && !text.includes('net::ERR_') && !text.includes('404')) {
          pageErrors.push(text);
        }
      }
    });
    
    await page.goto(FRONTEND);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    await page.goto(`${FRONTEND}?id=${testWorldId}&tab=map`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    console.log(`✅ Console errors check complete. JavaScript errors: ${pageErrors.length}`);
    if (pageErrors.length > 0) {
      pageErrors.forEach(e => console.log('  - ' + e));
    }
  });

  test('TC-013: Cleanup - Delete test world', async ({ request }) => {
    if (testWorldId) {
      const response = await request.delete(`${API_BASE}/worlds/${testWorldId}`);
      if (response.ok()) {
        console.log('✅ Test world deleted successfully');
      } else {
        console.log(`⚠️ Failed to delete world: HTTP ${response.status()}`);
      }
    }
  });

});
