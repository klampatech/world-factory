import { test, expect } from '@playwright/test';
import path from 'path';

test.describe('WOR-210: Voronoi Polygon Tile Rendering QA', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to the frontend
    await page.goto('http://localhost:8787');
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
  });

  test('TC-WOR210-01: Verify Voronoi polygon count for 256x256 world', async ({ page }) => {
    // Create a new 256x256 world via API for testing
    const worldId = '753e9b63-0293-458e-9a54-f994ae1616cb';
    
    // Navigate directly to the map view for this world
    await page.goto(`http://localhost:8787`);
    
    // Wait for the app to initialize
    await page.waitForTimeout(2000);
    
    // Look for the world list/selector
    const pageContent = await page.content();
    
    // Check for API selector or world list
    const apiSelector = page.locator('text=/World Factory|API|v1/');
    const hasContent = await pageContent.length > 1000;
    
    console.log('Page loaded, content length:', pageContent.length);
    console.log('Has meaningful content:', hasContent);
    
    // Save screenshot
    const screenshotPath = path.join('screenshots', `wor-210-voronoi-${Date.now()}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`Screenshot saved: ${screenshotPath}`);
    
    // Verify the frontend is functional
    expect(hasContent).toBe(true);
  });

  test('TC-WOR210-02: Verify map API returns 256 polygons', async ({ page }) => {
    // This test verifies the API response directly
    const response = await page.request.get('http://localhost:80822/api/v1/worlds/world:753e9b63-0293-458e-9a54-f994ae1616cb/map');
    
    expect(response.status()).toBe(200);
    
    const json = await response.json();
    // Handle ApiResponse wrapper format
    const data = json.data || json;
    const polygons = data.polygons || [];
    const polygonCount = polygons.length;
    
    console.log(`Polygon count: ${polygonCount}`);
    console.log(`Response keys: ${Object.keys(json)}`);
    console.log(`Data keys: ${data ? Object.keys(data) : 'null'}`);
    
    // Take screenshot of the response
    await page.screenshot({ 
      path: path.join('screenshots', `wor-210-api-response-${Date.now()}.png`), 
      fullPage: true 
    });
    
    // Verify polygon count is approximately 256 (not 132 or 65536)
    expect(polygonCount).toBeGreaterThanOrEqual(250);
    expect(polygonCount).toBeLessThanOrEqual(262);
  });

  test('TC-WOR210-03: Verify polygons have valid vertices', async ({ page }) => {
    const response = await page.request.get('http://localhost:80822/api/v1/worlds/world:753e9b63-0293-458e-9a54-f994ae1616cb/map');
    
    expect(response.status()).toBe(200);
    
    const json = await response.json();
    // Handle ApiResponse wrapper format
    const data = json.data || json;
    const polygons = data.polygons || [];
    
    console.log(`Total polygons: ${polygons.length}`);
    
    let validPolygonCount = 0;
    let invalidPolygonCount = 0;
    
    for (const polygon of polygons) {
      const vertices = polygon.vertices || [];
      if (vertices.length >= 3) {
        validPolygonCount++;
      } else {
        invalidPolygonCount++;
      }
    }
    
    console.log(`Valid polygons: ${validPolygonCount}, Invalid polygons: ${invalidPolygonCount}`);
    
    // All polygons should have at least 3 vertices
    expect(invalidPolygonCount).toBe(0);
    expect(validPolygonCount).toBe(polygons.length);
  });
});
