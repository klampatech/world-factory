# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: frontend-smoke-tests.spec.ts >> Frontend Smoke Tests (TC-UI-001 to TC-UI-012) >> TC-UI-001: Page loads with HTTP 200
- Location: e2e/frontend-smoke-tests.spec.ts:16:7

# Error details

```
Error: page.goto: NS_ERROR_CONNECTION_REFUSED
Call log:
  - navigating to "http://0.0.0.0:8787/", waiting until "load"

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - generic [ref=e3]:
    - heading "Unable to connect" [level=1] [ref=e5]
    - paragraph [ref=e6]: Firefox can’t establish a connection to the server at 0.0.0.0:8787.
    - paragraph
    - list [ref=e8]:
      - listitem [ref=e9]: The site could be temporarily unavailable or too busy. Try again in a few moments.
      - listitem [ref=e10]: If you are unable to load any pages, check your computer’s network connection.
      - listitem [ref=e11]: If your computer or network is protected by a firewall or proxy, make sure that Nightly is permitted to access the web.
  - button "Try Again" [active] [ref=e13]
```

# Test source

```ts
  1   | import { test, expect } from '@playwright/test';
  2   | 
  3   | /**
  4   |  * WOR-130: Phase 2 — Frontend Smoke Test Suite
  5   |  * Tests for World Factory web frontend on http://localhost:8765
  6   |  * 
  7   |  * Test Cases: TC-UI-001 to TC-UI-012
  8   |  * Parent: WOR-128 Testing Roadmap
  9   |  */
  10  | 
  11  | const BASE_URL = 'http://0.0.0.0:8787';
  12  | 
  13  | test.describe('Frontend Smoke Tests (TC-UI-001 to TC-UI-012)', () => {
  14  | 
  15  |   // TC-UI-001: Page loads with HTTP 200
  16  |   test('TC-UI-001: Page loads with HTTP 200', async ({ page }) => {
> 17  |     const response = await page.goto(BASE_URL + '/');
      |                                 ^ Error: page.goto: NS_ERROR_CONNECTION_REFUSED
  18  |     expect(response?.status()).toBe(200);
  19  |   });
  20  | 
  21  |   // TC-UI-002: Canvas map container exists
  22  |   test('TC-UI-002: Canvas map container exists', async ({ page }) => {
  23  |     await page.goto(BASE_URL + '/');
  24  |     const canvas = page.locator('#map-canvas');
  25  |     await expect(canvas).toBeVisible();
  26  |   });
  27  | 
  28  |   // TC-UI-003: Map renders with at least 1 region (canvas has drawn content)
  29  |   test('TC-UI-003: Map canvas has non-empty content', async ({ page }) => {
  30  |     await page.goto(BASE_URL + '/');
  31  |     
  32  |     // Wait for canvas to be rendered
  33  |     const canvas = page.locator('#map-canvas');
  34  |     await expect(canvas).toBeVisible();
  35  |     
  36  |     // Check canvas has non-zero dimensions
  37  |     const box = await canvas.boundingBox();
  38  |     expect(box?.width).toBeGreaterThan(0);
  39  |     expect(box?.height).toBeGreaterThan(0);
  40  |   });
  41  | 
  42  |   // TC-UI-004: Overlay controls are visible (Resources, Elevation, Political, Wonders)
  43  |   test('TC-UI-004: Overlay controls visible', async ({ page }) => {
  44  |     await page.goto(BASE_URL + '/');
  45  |     
  46  |     const overlayControls = page.locator('#overlay-controls');
  47  |     await expect(overlayControls).toBeVisible();
  48  |     
  49  |     // Check all 4 overlay buttons exist
  50  |     await expect(page.locator('[data-overlay="resources"]')).toBeVisible();
  51  |     await expect(page.locator('[data-overlay="elevation"]')).toBeVisible();
  52  |     await expect(page.locator('[data-overlay="political"]')).toBeVisible();
  53  |     await expect(page.locator('[data-overlay="wonders"]')).toBeVisible();
  54  |   });
  55  | 
  56  |   // TC-UI-005: Switching overlays updates display
  57  |   test('TC-UI-005: Overlay switching updates display', async ({ page }) => {
  58  |     await page.goto(BASE_URL + '/');
  59  |     
  60  |     const legend = page.locator('#overlay-legend');
  61  |     
  62  |     // Initially legend should be hidden
  63  |     await expect(legend).toBeHidden();
  64  |     
  65  |     // Click elevation overlay
  66  |     await page.locator('[data-overlay="elevation"]').click();
  67  |     
  68  |     // Legend should now be visible
  69  |     await expect(legend).toBeVisible();
  70  |     
  71  |     // Click political overlay
  72  |     await page.locator('[data-overlay="political"]').click();
  73  |     await expect(legend).toBeVisible();
  74  |     
  75  |     // Click resources overlay  
  76  |     await page.locator('[data-overlay="resources"]').click();
  77  |     await expect(legend).toBeVisible();
  78  |   });
  79  | 
  80  |   // TC-UI-006: Zoom controls are visible
  81  |   test('TC-UI-006: Zoom controls visible', async ({ page }) => {
  82  |     await page.goto(BASE_URL + '/');
  83  |     
  84  |     // Check for zoom level indicator (zoom via mousewheel, no dedicated buttons)
  85  |     const hasZoomLevel = await page.locator('#zoom-level').count() > 0;
  86  |     const hasZoom = hasZoomLevel;
  87  |     
  88  |     // At minimum, verify the map area is functional
  89  |     const mapCanvas = page.locator('#map-canvas');
  90  |     await expect(mapCanvas).toBeVisible();
  91  |   });
  92  | 
  93  |   // TC-UI-007: Pan interaction works (mouse drag pans the map)
  94  |   test('TC-UI-007: Pan interaction works', async ({ page }) => {
  95  |     await page.goto(BASE_URL + '/');
  96  |     
  97  |     const canvas = page.locator('#map-canvas');
  98  |     await expect(canvas).toBeVisible();
  99  |     
  100 |     const box = await canvas.boundingBox();
  101 |     if (!box) throw new Error('Canvas not found');
  102 |     
  103 |     // Get initial center position
  104 |     const initialCenterX = box.x + box.width / 2;
  105 |     const initialCenterY = box.y + box.height / 2;
  106 |     
  107 |     // Perform drag
  108 |     await page.mouse.move(initialCenterX, initialCenterY);
  109 |     await page.mouse.down();
  110 |     await page.mouse.move(initialCenterX + 100, initialCenterY + 50);
  111 |     await page.mouse.up();
  112 |     
  113 |     // Canvas should still be visible after drag
  114 |     await expect(canvas).toBeVisible();
  115 |   });
  116 | 
  117 |   // TC-UI-008: Timeline section exists
```