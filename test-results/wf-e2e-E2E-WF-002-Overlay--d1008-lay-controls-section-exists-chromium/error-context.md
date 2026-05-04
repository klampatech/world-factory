# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: wf-e2e.spec.ts >> E2E-WF-002: Overlay System >> E2E-WF-002.2: Overlay controls section exists
- Location: e2e/wf-e2e.spec.ts:114:7

# Error details

```
Test timeout of 30000ms exceeded while running "beforeEach" hook.
```

# Page snapshot

```yaml
- generic [ref=e2]:
  - banner [ref=e3]:
    - generic [ref=e4]:
      - img [ref=e5]
      - generic [ref=e8]: World Factory
    - generic [ref=e9]:
      - button "Map" [ref=e10] [cursor=pointer]
      - button "Timeline" [ref=e11] [cursor=pointer]
    - generic [ref=e12]:
      - button "Reset View" [ref=e13] [cursor=pointer]
      - button "Generate World" [ref=e14] [cursor=pointer]
  - main [ref=e15]:
    - generic [ref=e16]:
      - heading "Biomes" [level=4] [ref=e21]
      - generic [ref=e22]:
        - button "Resources" [ref=e23] [cursor=pointer]:
          - img [ref=e24]
          - text: Resources
        - button "Elevation" [ref=e26] [cursor=pointer]:
          - img [ref=e27]
          - text: Elevation
        - button "Political" [ref=e29] [cursor=pointer]:
          - img [ref=e30]
          - text: Political
        - button "Wonders" [ref=e32] [cursor=pointer]:
          - img [ref=e33]
          - text: Wonders
```

# Test source

```ts
  1   | // World Factory E2E Test Suite
  2   | // WOR-604: Create e2e test suite
  3   | //
  4   | // Tests browser-based UI interactions for the World Factory application.
  5   | // Target: http://localhost:8765
  6   | //
  7   | // Run:
  8   | //   npx playwright test --config=playwright.e2e.config.ts e2e/wf-e2e.spec.ts
  9   | 
  10  | import { test, expect, Page } from '@playwright/test';
  11  | 
  12  | // =======================================================================
  13  | // Test Configuration
  14  | // =======================================================================
  15  | const BASE_URL = 'http://localhost:8765';
  16  | const MAP_CANVAS = '#map-canvas';
  17  | const OVERLAY_CONTROLS = '#overlay-controls';
  18  | 
  19  | // =======================================================================
  20  | // Helper Functions
  21  | // =======================================================================
  22  | async function waitForMapReady(page: Page, timeout = 15000): Promise<void> {
  23  |   // Wait for canvas to be visible
  24  |   await expect(page.locator(MAP_CANVAS)).toBeVisible({ timeout });
  25  |   
  26  |   // Wait for loading overlay to disappear (if it appears)
  27  |   try {
  28  |     await page.locator('#map-loading').waitFor({ state: 'hidden', timeout: 30000 });
  29  |   } catch {
  30  |     // Loading overlay may not exist or already hidden
  31  |   }
  32  | }
  33  | 
  34  | async function clickOverlay(page: Page, overlayName: string): Promise<void> {
  35  |   const overlayBtn = page.locator(`[data-overlay="${overlayName}"]`);
  36  |   await overlayBtn.click({ timeout: 5000 });
  37  | }
  38  | 
  39  | // =======================================================================
  40  | // E2E-WF-001: Basic Page Load
  41  | // =======================================================================
  42  | test.describe('E2E-WF-001: Page Load & Initialization', () => {
  43  |   
  44  |   test('E2E-WF-001.1: Page loads without crash', async ({ page }) => {
  45  |     const response = await page.goto(BASE_URL + '/');
  46  |     expect(response?.status()).toBe(200);
  47  |     console.log('✓ Page loaded with HTTP 200');
  48  |   });
  49  | 
  50  |   test('E2E-WF-001.2: Map canvas exists and is visible', async ({ page }) => {
  51  |     await page.goto(BASE_URL + '/');
  52  |     await waitForMapReady(page);
  53  |     await expect(page.locator(MAP_CANVAS)).toBeVisible();
  54  |     console.log('✓ Map canvas is visible');
  55  |   });
  56  | 
  57  |   test('E2E-WF-001.3: Canvas has non-zero dimensions', async ({ page }) => {
  58  |     await page.goto(BASE_URL + '/');
  59  |     await waitForMapReady(page);
  60  |     
  61  |     const canvas = page.locator(MAP_CANVAS);
  62  |     const box = await canvas.boundingBox();
  63  |     expect(box?.width).toBeGreaterThan(0);
  64  |     expect(box?.height).toBeGreaterThan(0);
  65  |     console.log(`✓ Canvas dimensions: ${box?.width}x${box?.height}`);
  66  |   });
  67  | 
  68  |   test('E2E-WF-001.4: No critical console errors on load', async ({ page }) => {
  69  |     const errors: string[] = [];
  70  |     
  71  |     page.on('console', msg => {
  72  |       if (msg.type() === 'error') {
  73  |         errors.push(msg.text());
  74  |       }
  75  |     });
  76  |     
  77  |     await page.goto(BASE_URL + '/');
  78  |     await page.waitForTimeout(2000);
  79  |     
  80  |     // Filter out known benign errors
  81  |     const criticalErrors = errors.filter(e => 
  82  |       !e.includes('favicon') && 
  83  |       !e.includes('net::ERR') &&
  84  |       !e.includes('Failed to load resource')
  85  |     );
  86  |     
  87  |     expect(criticalErrors).toHaveLength(0);
  88  |     console.log(`✓ No critical console errors (total: ${errors.length})`);
  89  |   });
  90  | 
  91  | });
  92  | 
  93  | // =======================================================================
  94  | // E2E-WF-002: Overlay System
  95  | // =======================================================================
  96  | test.describe('E2E-WF-002: Overlay System', () => {
  97  |   
> 98  |   test.beforeEach(async ({ page }) => {
      |        ^ Test timeout of 30000ms exceeded while running "beforeEach" hook.
  99  |     await page.goto(BASE_URL + '/');
  100 |     await waitForMapReady(page);
  101 |   });
  102 | 
  103 |   test('E2E-WF-002.1: All overlay control buttons exist', async ({ page }) => {
  104 |     const overlays = ['resources', 'elevation', 'political', 'wonders'];
  105 |     
  106 |     for (const overlay of overlays) {
  107 |       const btn = page.locator(`[data-overlay="${overlay}"]`);
  108 |       await expect(btn).toBeVisible();
  109 |       console.log(`  ✓ ${overlay} overlay button visible`);
  110 |     }
  111 |     console.log('✓ All 4 overlay buttons exist');
  112 |   });
  113 | 
  114 |   test('E2E-WF-002.2: Overlay controls section exists', async ({ page }) => {
  115 |     await expect(page.locator(OVERLAY_CONTROLS)).toBeVisible();
  116 |     console.log('✓ Overlay controls section visible');
  117 |   });
  118 | 
  119 |   test('E2E-WF-002.3: Clicking Resources overlay activates it', async ({ page }) => {
  120 |     await clickOverlay(page, 'resources');
  121 |     await page.waitForTimeout(300);
  122 |     
  123 |     const legend = page.locator('#overlay-legend');
  124 |     await expect(legend).toBeVisible();
  125 |     console.log('✓ Resources overlay activates legend');
  126 |   });
  127 | 
  128 |   test('E2E-WF-002.4: Clicking Elevation overlay activates it', async ({ page }) => {
  129 |     await clickOverlay(page, 'elevation');
  130 |     await page.waitForTimeout(300);
  131 |     
  132 |     const legend = page.locator('#overlay-legend');
  133 |     await expect(legend).toBeVisible();
  134 |     console.log('✓ Elevation overlay activates legend');
  135 |   });
  136 | 
  137 |   test('E2E-WF-002.5: Clicking Political overlay activates it', async ({ page }) => {
  138 |     await clickOverlay(page, 'political');
  139 |     await page.waitForTimeout(300);
  140 |     
  141 |     const legend = page.locator('#overlay-legend');
  142 |     await expect(legend).toBeVisible();
  143 |     console.log('✓ Political overlay activates legend');
  144 |   });
  145 | 
  146 |   test('E2E-WF-002.6: Clicking Wonders overlay activates it', async ({ page }) => {
  147 |     await clickOverlay(page, 'wonders');
  148 |     await page.waitForTimeout(300);
  149 |     
  150 |     const legend = page.locator('#overlay-legend');
  151 |     await expect(legend).toBeVisible();
  152 |     console.log('✓ Wonders overlay activates legend');
  153 |   });
  154 | 
  155 |   test('E2E-WF-002.7: Only one overlay can be active at a time', async ({ page }) => {
  156 |     // Activate resources
  157 |     await clickOverlay(page, 'resources');
  158 |     await page.waitForTimeout(300);
  159 |     
  160 |     // Activate elevation (should deactivate resources)
  161 |     await clickOverlay(page, 'elevation');
  162 |     await page.waitForTimeout(300);
  163 |     
  164 |     // Both buttons should still exist
  165 |     await expect(page.locator('[data-overlay="resources"]')).toBeVisible();
  166 |     await expect(page.locator('[data-overlay="elevation"]')).toBeVisible();
  167 |     
  168 |     // Legend should be visible
  169 |     await expect(page.locator('#overlay-legend')).toBeVisible();
  170 |     console.log('✓ Overlay exclusivity works');
  171 |   });
  172 | 
  173 | });
  174 | 
  175 | // =======================================================================
  176 | // E2E-WF-003: Map Interaction (Pan & Zoom)
  177 | // =======================================================================
  178 | test.describe('E2E-WF-003: Map Interaction', () => {
  179 |   
  180 |   test.beforeEach(async ({ page }) => {
  181 |     await page.goto(BASE_URL + '/');
  182 |     await waitForMapReady(page);
  183 |   });
  184 | 
  185 |   test('E2E-WF-003.1: Map canvas responds to mouse pan', async ({ page }) => {
  186 |     const canvas = page.locator(MAP_CANVAS);
  187 |     const box = await canvas.boundingBox();
  188 |     
  189 |     if (!box) throw new Error('Canvas not found');
  190 |     
  191 |     const startX = box.x + box.width / 2;
  192 |     const startY = box.y + box.height / 2;
  193 |     
  194 |     // Perform drag (pan)
  195 |     await page.mouse.move(startX, startY);
  196 |     await page.mouse.down();
  197 |     await page.mouse.move(startX + 100, startY + 50);
  198 |     await page.mouse.up();
```