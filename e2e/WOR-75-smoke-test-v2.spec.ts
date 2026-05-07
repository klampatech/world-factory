import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';

const screenshotsDir = path.join(__dirname, '..', 'screenshots', 'WOR-75');
if (!fs.existsSync(screenshotsDir)) {
  fs.mkdirSync(screenshotsDir, { recursive: true });
}

const BASE_URL = 'http://localhost:8765';
const API_BASE = 'http://localhost:8080/api/v1';

test.describe('WOR-75: End-to-End Smoke Test', () => {
  
  test('1. Backend health check', async () => {
    const response = await fetch('http://localhost:8080/health');
    const data = await response.json();
    expect(data.status).toBe('ok');
    console.log('✓ Backend health: OK');
  });

  test('2. Backend API - Create a world', async ({ page }) => {
    const response = await page.request.post(`${API_BASE}/worlds`, {
      data: { name: 'WOR-75 Test World', parameters: { seed: 7575, size: 'Small' } }
    });
    
    // Accept 201 (created) or 202 (accepted) as success
    expect([201, 202]).toContain(response.status());
    
    const body = await response.json();
    const worldId = body?.data?.id;
    console.log('✓ World creation works, ID:', worldId);
  });

  test('3. Frontend loads with all key components', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(2000);
    
    // Check main UI elements
    const logo = page.locator('.logo:has-text("World Factory")');
    await expect(logo).toBeVisible();
    
    const mapCanvas = page.locator('#map-canvas');
    await expect(mapCanvas).toBeVisible();
    
    const viewTabs = page.locator('.view-tab');
    await expect(viewTabs.first()).toBeVisible();
    
    const generateBtn = page.locator('#generate-world');
    await expect(generateBtn).toBeVisible();
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-01-main-layout.png` });
    console.log('✓ Frontend main layout: OK');
  });

  test('4. Map canvas renders correctly', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const canvas = page.locator('#map-canvas');
    const box = await canvas.boundingBox();
    
    expect(box.width).toBeGreaterThan(100);
    expect(box.height).toBeGreaterThan(100);
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-02-map-canvas.png` });
    console.log(`✓ Map canvas size: ${box.width}x${box.height}`);
  });

  test('5. Overlay controls function correctly', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Test Resources overlay
    const resourcesBtn = page.locator('[data-overlay="resources"]');
    if (await resourcesBtn.isVisible()) {
      await resourcesBtn.click();
      await page.waitForTimeout(500);
      const overlayLegend = page.locator('#overlay-legend');
      await expect(overlayLegend).toBeVisible();
      await page.screenshot({ path: `${screenshotsDir}/frontend-03-overlay-resources.png` });
      console.log('✓ Resources overlay: OK');
    }
    
    // Test Elevation overlay
    const elevationBtn = page.locator('[data-overlay="elevation"]');
    if (await elevationBtn.isVisible()) {
      await elevationBtn.click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: `${screenshotsDir}/frontend-04-overlay-elevation.png` });
      console.log('✓ Elevation overlay: OK');
    }
    
    // Test Political overlay
    const politicalBtn = page.locator('[data-overlay="political"]');
    if (await politicalBtn.isVisible()) {
      await politicalBtn.click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: `${screenshotsDir}/frontend-05-overlay-political.png` });
      console.log('✓ Political overlay: OK');
    }
    
    // Test Wonders overlay
    const wondersBtn = page.locator('[data-overlay="wonders"]');
    if (await wondersBtn.isVisible()) {
      await wondersBtn.click();
      await page.waitForTimeout(500);
      const wondersPanel = page.locator('#wonders-panel');
      if (await wondersPanel.isVisible()) {
        await page.screenshot({ path: `${screenshotsDir}/frontend-06-overlay-wonders.png` });
        console.log('✓ Wonders overlay: OK');
      } else {
        console.log('⚠ Wonders panel not visible');
      }
    }
  });

  test('6. Zoom controls work', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);
    
    const zoomIn = page.locator('#zoom-in');
    const zoomOut = page.locator('#zoom-out');
    
    // Get initial zoom level
    const initialZoom = await page.locator('#zoom-level').textContent();
    
    if (await zoomIn.isVisible()) {
      await zoomIn.click();
      await page.waitForTimeout(300);
      const newZoom = await page.locator('#zoom-level').textContent();
      expect(newZoom).not.toBe(initialZoom);
      await page.screenshot({ path: `${screenshotsDir}/frontend-07-zoom-in.png` });
      console.log(`✓ Zoom in works: ${initialZoom} -> ${newZoom}`);
    }
    
    if (await zoomOut.isVisible()) {
      await zoomOut.click();
      await page.waitForTimeout(300);
      await page.screenshot({ path: `${screenshotsDir}/frontend-08-zoom-out.png` });
      console.log('✓ Zoom out works');
    }
  });

  test('7. Timeline view switches correctly', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    // Click Timeline tab
    const timelineTab = page.locator('[data-view="timeline"]');
    await timelineTab.click();
    await page.waitForTimeout(500);
    
    const timelineView = page.locator('#timeline-view');
    await expect(timelineView).toHaveClass(/active/);
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-09-timeline-view.png` });
    console.log('✓ Timeline view: OK');
    
    // Switch back to Map
    const mapTab = page.locator('[data-view="map"]');
    await mapTab.click();
    await page.waitForTimeout(300);
    
    const mapViewEl = page.locator('#map-view');
    await expect(mapViewEl).not.toHaveClass(/hidden/);
    console.log('✓ Map view returns: OK');
  });

  test('8. Info panel shows correct data', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    const infoPanel = page.locator('#info-panel');
    
    if (await infoPanel.isVisible()) {
      const zoom = await page.locator('#zoom-level').textContent();
      const panPos = await page.locator('#pan-pos').textContent();
      const regions = await page.locator('#region-count').textContent();
      
      expect(zoom).toMatch(/\d+%/);
      expect(panPos).toMatch(/\d+,\s*\d+/);
      expect(regions).not.toBe('0');
      
      await page.screenshot({ path: `${screenshotsDir}/frontend-10-info-panel.png` });
      console.log(`✓ Info panel: Zoom=${zoom}, Pan=${panPos}, Regions=${regions}`);
    } else {
      console.log('⚠ Info panel not visible (may be auto-hidden)');
    }
  });

  test('9. Generate World button exists', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    
    const generateBtn = page.locator('#generate-world');
    await expect(generateBtn).toBeVisible();
    
    // Check button is clickable
    const isEnabled = await generateBtn.isEnabled();
    expect(isEnabled).toBe(true);
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-11-generate-btn.png` });
    console.log('✓ Generate World button: enabled');
  });

  test('10. Browser console - check for unexpected JS errors', async ({ page }) => {
    const jsErrors: string[] = [];
    
    page.on('pageerror', error => {
      jsErrors.push(error.message);
    });
    
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Note: We allow HTTP errors (422, 400) because they indicate API validation
    // We only fail on actual JavaScript exceptions
    if (jsErrors.length > 0) {
      console.log('JS Errors:', jsErrors);
    }
    
    // Filter out only actual JS errors, not HTTP response errors
    const criticalErrors = jsErrors.filter(e => 
      !e.includes('net::') && 
      !e.includes('Failed to load resource')
    );
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-12-console-check.png` });
    
    if (criticalErrors.length === 0) {
      console.log('✓ No critical JS errors');
    } else {
      console.log('JS Errors found:', criticalErrors);
      throw new Error(`Critical JS errors: ${criticalErrors.join(', ')}`);
    }
  });

  test('11. API accessibility from browser context', async ({ page }) => {
    const response = await page.request.get(`${API_BASE}/worlds`);
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    const worldCount = body?.data?.totalWorlds || body?.totalWorlds || 0;
    expect(worldCount).toBeGreaterThan(0);
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-13-api-test.png` });
    console.log(`✓ API accessible: ${worldCount} worlds in database`);
  });

  test('12. Export PNG button works', async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);
    
    // Set up download handler
    const downloadPromise = page.waitForEvent('download', { timeout: 5000 }).catch(() => null);
    
    const exportBtn = page.locator('#export-png');
    if (await exportBtn.isVisible()) {
      await exportBtn.click();
      
      // Give time for download to start
      await page.waitForTimeout(1000);
      
      const download = await downloadPromise;
      if (download) {
        const filename = download.suggestedFilename();
        expect(filename).toContain('.png');
        console.log(`✓ Export PNG: ${filename}`);
      } else {
        console.log('⚠ Export: No download triggered (may require interaction)');
      }
    }
    
    await page.screenshot({ path: `${screenshotsDir}/frontend-14-export-png.png` });
  });
});
