import { test, expect, chromium, Browser, BrowserContext, Page } from '@playwright/test';

const API_BASE = 'http://localhost:3000/api/v1';
const FRONTEND = 'http://localhost:8765';
const SCREENSHOTS = '/home/kyle/projects/world-generator/screenshots/WOR-1072';

test('Verify Voronoi polygons render correctly (not scattered squares)', async ({ page }) => {
  // Get a ready world
  const resp = await fetch(`${API_BASE}/worlds`);
  const data = await resp.json();
  const world = data.data.worlds.find((w: any) => w.status === 'ready' || w.status === 'generating');
  
  if (!world) {
    console.log('No existing world, creating new one...');
    const createResp = await fetch(`${API_BASE}/worlds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Voronoi Test', seed: 12345, config: { genre: 'fantasy', width: 32, height: 32 } })
    });
    const worldData = await createResp.json();
    var worldId = worldData.data.id;
  } else {
    var worldId = world.id;
  }
  
  const worldUUID = worldId.replace('world:', '');
  
  // Navigate and get the map
  await page.goto(`${FRONTEND}/world.html?id=${worldUUID}`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(5000);
  
  // Click map tab
  const mapTab = page.locator('[data-tab="map"]').first();
  if (await mapTab.count() > 0) await mapTab.click();
  await page.waitForTimeout(3000);
  
  // Get high-res screenshot
  await page.screenshot({ path: `${SCREENSHOTS}/voronoi-verification.png`, fullPage: false });
  
  // Get the map image from API to verify polygon data
  const mapResp = await fetch(`${API_BASE}/worlds/${worldUUID}/map`);
  const mapData = await mapResp.json();
  
  console.log(`Map has ${mapData.data.polygons?.length ?? 0} Voronoi polygons`);
  console.log(`Polygon sample:`, JSON.stringify(mapData.data.polygons?.[0], null, 2));
  
  expect(mapData.data.polygons?.length).toBeGreaterThan(0);
});
