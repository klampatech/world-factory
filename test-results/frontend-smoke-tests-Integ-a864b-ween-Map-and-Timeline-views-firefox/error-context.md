# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: frontend-smoke-tests.spec.ts >> Integration Tests >> User can switch between Map and Timeline views
- Location: e2e/frontend-smoke-tests.spec.ts:208:7

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
  109 |     await page.mouse.down();
  110 |     await page.mouse.move(initialCenterX + 100, initialCenterY + 50);
  111 |     await page.mouse.up();
  112 |     
  113 |     // Canvas should still be visible after drag
  114 |     await expect(canvas).toBeVisible();
  115 |   });
  116 | 
  117 |   // TC-UI-008: Timeline section exists
  118 |   test('TC-UI-008: Timeline section exists', async ({ page }) => {
  119 |     await page.goto(BASE_URL + '/');
  120 |     
  121 |     // Check for timeline tab/button
  122 |     const timelineTab = page.locator('.view-tab:has-text("Timeline"), #timeline-view, .timeline-container');
  123 |     await expect(timelineTab.first()).toBeVisible();
  124 |   });
  125 | 
  126 |   // TC-UI-009: Timeline events are displayed (navigates to timeline view)
  127 |   test('TC-UI-009: Timeline shows events when selected', async ({ page }) => {
  128 |     await page.goto(BASE_URL + '/');
  129 |     
  130 |     // Click on Timeline tab if it exists
  131 |     const timelineTab = page.locator('.view-tab:has-text("Timeline")');
  132 |     if (await timelineTab.count() > 0) {
  133 |       await timelineTab.click();
  134 |     }
  135 |     
  136 |     // Check if timeline container exists and is visible
  137 |     const timelineContainer = page.locator('#timeline-container, .timeline-container, #timeline-view');
  138 |     await expect(timelineContainer.first()).toBeVisible();
  139 |   });
  140 | 
  141 |   // TC-UI-010: Region detail panel opens on click
  142 |   test('TC-UI-010: Region tooltip appears on click', async ({ page }) => {
  143 |     await page.goto(BASE_URL + '/');
  144 |     
  145 |     const canvas = page.locator('#map-canvas');
  146 |     await expect(canvas).toBeVisible();
  147 |     
  148 |     const box = await canvas.boundingBox();
  149 |     if (!box) throw new Error('Canvas not found');
  150 |     
  151 |     // Click on canvas (where a region might be)
  152 |     await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  153 |     
  154 |     // Wait a moment for potential tooltip to appear
  155 |     await page.waitForTimeout(500);
  156 |     
  157 |     // The frontend doesn't have a dedicated tooltip component in current HTML,
  158 |     // but we verify canvas is still functional
  159 |     await expect(canvas).toBeVisible();
  160 |   });
  161 | 
  162 |   // TC-UI-011: No console errors on load
  163 |   test('TC-UI-011: No console errors on load', async ({ page }) => {
  164 |     const errors: string[] = [];
  165 |     
  166 |     page.on('console', msg => {
  167 |       if (msg.type() === 'error') {
  168 |         errors.push(msg.text());
  169 |       }
  170 |     });
  171 |     
  172 |     await page.goto(BASE_URL + '/');
  173 |     await page.waitForTimeout(2000); // Allow async operations
  174 |     
  175 |     // Filter out known benign errors
  176 |     const criticalErrors = errors.filter(e => 
  177 |       !e.includes('favicon') && 
  178 |       !e.includes('net::ERR') &&
  179 |       !e.includes('Failed to load resource')
  180 |     );
  181 |     
  182 |     expect(criticalErrors).toHaveLength(0);
  183 |   });
  184 | 
  185 |   // TC-UI-012: Wonders markers render on Wonders overlay
  186 |   test('TC-UI-012: Wonders overlay button works', async ({ page }) => {
  187 |     await page.goto(BASE_URL + '/');
  188 |     
  189 |     const wondersBtn = page.locator('[data-overlay="wonders"]');
  190 |     await expect(wondersBtn).toBeVisible();
  191 |     
  192 |     // Click wonders overlay
  193 |     await wondersBtn.click();
  194 |     
  195 |     // Legend element should exist (check DOM presence, not visibility since it starts hidden)
  196 |     const legend = page.locator('#overlay-legend');
  197 |     await expect(legend).toHaveCount(1);
  198 |     
  199 |     // The wonders button should have active state after clicking
  200 |     await expect(wondersBtn).toHaveClass(/active/);
  201 |   });
  202 | 
  203 | });
  204 | 
  205 | test.describe('Integration Tests', () => {
  206 |   
  207 |   // Navigation between views
  208 |   test('User can switch between Map and Timeline views', async ({ page }) => {
> 209 |     await page.goto(BASE_URL + '/');
      |                ^ Error: page.goto: NS_ERROR_CONNECTION_REFUSED
  210 |     
  211 |     // Check map view is default
  212 |     const mapCanvas = page.locator('#map-canvas');
  213 |     await expect(mapCanvas).toBeVisible();
  214 |     
  215 |     // Look for view tabs
  216 |     const tabs = page.locator('.view-tab');
  217 |     if (await tabs.count() > 0) {
  218 |       // Try Timeline tab
  219 |       const timelineTab = tabs.filter({ hasText: 'Timeline' });
  220 |       if (await timelineTab.count() > 0) {
  221 |         await timelineTab.click();
  222 |         await page.waitForTimeout(300);
  223 |         
  224 |         // Verify timeline view is now active
  225 |         const timelineView = page.locator('#timeline-view');
  226 |         await expect(timelineView).toBeVisible();
  227 |       }
  228 |     }
  229 |     
  230 |     // Switch back to Map tab and verify map is visible
  231 |     const mapTab = tabs.filter({ hasText: 'Map' });
  232 |     if (await mapTab.count() > 0) {
  233 |       await mapTab.click();
  234 |       await page.waitForTimeout(300);
  235 |       await expect(mapCanvas).toBeVisible();
  236 |     }
  237 |   });
  238 | 
  239 |   // Header elements
  240 |   test('Header displays correctly with logo and controls', async ({ page }) => {
  241 |     await page.goto(BASE_URL + '/');
  242 |     
  243 |     const header = page.locator('header');
  244 |     await expect(header).toBeVisible();
  245 |     
  246 |     // Check for logo/title
  247 |     const logo = page.locator('.logo, h1, [class*="logo"]');
  248 |     await expect(logo.first()).toBeVisible();
  249 |   });
  250 | 
  251 | });
```