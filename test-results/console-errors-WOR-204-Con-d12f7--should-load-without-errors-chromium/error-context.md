# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: console-errors.spec.ts >> WOR-204: Console Error Detection >> dashboard view should load without errors
- Location: e2e/console-errors.spec.ts:188:7

# Error details

```
Error: Found 1 errors in dashboard view
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
      - button "Export PNG" [ref=e15] [cursor=pointer]
  - main [ref=e16]:
    - generic [ref=e17]:
      - generic [ref=e19]:
        - img [ref=e20]
        - generic [ref=e23]:
          - strong [ref=e24]: "Demo Mode:"
          - text: Showing sample data. Backend unavailable.
      - generic [ref=e25]:
        - heading "View Info" [level=3] [ref=e26]
        - generic [ref=e27]:
          - generic [ref=e28]: Zoom
          - generic [ref=e29]: 100%
        - generic [ref=e30]:
          - generic [ref=e31]: Pan
          - generic [ref=e32]: 0, 0
        - generic [ref=e33]:
          - generic [ref=e34]: Regions
          - generic [ref=e35]: "165"
        - generic [ref=e36]:
          - button "−" [ref=e37] [cursor=pointer]
          - button "+" [ref=e38] [cursor=pointer]
          - button "Reset" [ref=e39] [cursor=pointer]
      - generic [ref=e40]:
        - heading "Biomes" [level=4] [ref=e41]
        - generic [ref=e42]:
          - generic [ref=e45]: Ocean
          - generic [ref=e48]: Shallow Ocean
          - generic [ref=e51]: Grassland
          - generic [ref=e54]: Forest
          - generic [ref=e57]: Scrubland
          - generic [ref=e60]: Highland
          - generic [ref=e63]: Swamp
          - generic [ref=e66]: Rainforest
          - generic [ref=e69]: Desert
          - generic [ref=e72]: Mountain
          - generic [ref=e75]: Beach
      - generic [ref=e76]:
        - button "Resources" [ref=e77] [cursor=pointer]:
          - img [ref=e78]
          - text: Resources
        - button "Elevation" [ref=e80] [cursor=pointer]:
          - img [ref=e81]
          - text: Elevation
        - button "Political" [ref=e83] [cursor=pointer]:
          - img [ref=e84]
          - text: Political
        - button "Wonders" [ref=e86] [cursor=pointer]:
          - img [ref=e87]
          - text: Wonders
```

# Test source

```ts
  137 |     
  138 |     page.on('console', (msg: ConsoleMessage) => {
  139 |       if (msg.type() === 'error') {
  140 |         // Ignore 400 errors for non-existent worlds in navigation - expected
  141 |         const text = msg.text();
  142 |         if (text.includes('400') && text.includes('Failed to load resource')) {
  143 |           console.log('Ignoring expected 400 for fake world ID navigation');
  144 |           return;
  145 |         }
  146 |         errors.push({
  147 |           type: 'console.error',
  148 |           text: text,
  149 |           url: page.url()
  150 |         });
  151 |       }
  152 |     });
  153 |     
  154 |     page.on('pageerror', (err: Error) => {
  155 |       errors.push({
  156 |         type: 'pageerror',
  157 |         text: err.message,
  158 |         url: page.url(),
  159 |         stack: err.stack
  160 |       });
  161 |     });
  162 |     
  163 |     await page.goto('http://localhost:8765/');
  164 |     await page.waitForTimeout(2000);
  165 |     
  166 |     const readyWorld = page.locator('.world-card .status-badge.ready').first();
  167 |     if (await readyWorld.count() > 0) {
  168 |       // Click on a real world to navigate
  169 |       await page.locator('.btn-view.primary').first().click();
  170 |       await page.waitForTimeout(2000);
  171 |       
  172 |       // Navigate to timeline tab
  173 |       await page.locator('.view-tab:has-text("Timeline")').click();
  174 |       await page.waitForTimeout(2000);
  175 |       
  176 |       // Verify timeline container
  177 |       await expect(page.locator('.timeline-container')).toBeVisible();
  178 |     } else {
  179 |       console.log('No ready worlds found, skipping timeline test');
  180 |     }
  181 |     
  182 |     if (errors.length > 0) {
  183 |       console.log('Errors in timeline view:', JSON.stringify(errors, null, 2));
  184 |       throw new Error(`Found ${errors.length} errors in timeline view`);
  185 |     }
  186 |   });
  187 |   
  188 |   test('dashboard view should load without errors', async ({ page }) => {
  189 |     const errors: ConsoleError[] = [];
  190 |     
  191 |     page.on('console', (msg: ConsoleMessage) => {
  192 |       if (msg.type() === 'error') {
  193 |         // Ignore 400 errors for non-existent worlds in navigation - expected
  194 |         const text = msg.text();
  195 |         if (text.includes('400') && text.includes('Failed to load resource')) {
  196 |           console.log('Ignoring expected 400 for fake world ID navigation');
  197 |           return;
  198 |         }
  199 |         errors.push({
  200 |           type: 'console.error',
  201 |           text: text,
  202 |           url: page.url()
  203 |         });
  204 |       }
  205 |     });
  206 |     
  207 |     page.on('pageerror', (err: Error) => {
  208 |       errors.push({
  209 |         type: 'pageerror',
  210 |         text: err.message,
  211 |         url: page.url(),
  212 |         stack: err.stack
  213 |       });
  214 |     });
  215 |     
  216 |     await page.goto('http://localhost:8765/');
  217 |     await page.waitForTimeout(2000);
  218 |     
  219 |     const readyWorld = page.locator('.world-card .status-badge.ready').first();
  220 |     if (await readyWorld.count() > 0) {
  221 |       // Click on a real world to navigate
  222 |       await page.locator('.btn-view.primary').first().click();
  223 |       await page.waitForTimeout(2000);
  224 |       
  225 |       // Navigate to dashboard tab
  226 |       await page.locator('.view-tab:has-text("Dashboard")').click();
  227 |       await page.waitForTimeout(2000);
  228 |       
  229 |       // Verify dashboard container
  230 |       await expect(page.locator('.dashboard-container')).toBeVisible();
  231 |     } else {
  232 |       console.log('No ready worlds found, skipping dashboard test');
  233 |     }
  234 |     
  235 |     if (errors.length > 0) {
  236 |       console.log('Errors in dashboard view:', JSON.stringify(errors, null, 2));
> 237 |       throw new Error(`Found ${errors.length} errors in dashboard view`);
      |             ^ Error: Found 1 errors in dashboard view
  238 |     }
  239 |   });
  240 |   
  241 |   test('create world modal should work without errors', async ({ page }) => {
  242 |     const errors: ConsoleError[] = [];
  243 |     
  244 |     page.on('console', (msg: ConsoleMessage) => {
  245 |       if (msg.type() === 'error') {
  246 |         errors.push({
  247 |           type: 'console.error',
  248 |           text: msg.text(),
  249 |           url: page.url()
  250 |         });
  251 |       }
  252 |     });
  253 |     
  254 |     page.on('pageerror', (err: Error) => {
  255 |       errors.push({
  256 |         type: 'pageerror',
  257 |         text: err.message,
  258 |         url: page.url(),
  259 |         stack: err.stack
  260 |       });
  261 |     });
  262 |     
  263 |     await page.goto('http://localhost:8765/');
  264 |     await page.waitForTimeout(2000);
  265 |     
  266 |     // Open create modal - use first() to avoid strict mode violation
  267 |     await page.locator('.btn-create').first().click();
  268 |     await page.waitForTimeout(500);
  269 |     
  270 |     // Verify modal opened
  271 |     await expect(page.locator('#create-modal')).toHaveClass(/active/);
  272 |     
  273 |     // Interact with form elements
  274 |     await page.fill('#world-name', 'Test World Console Check');
  275 |     await page.locator('#world-width').fill('32');
  276 |     await page.waitForTimeout(200);
  277 |     
  278 |     // Close modal
  279 |     await page.locator('.btn:has-text("Cancel")').click();
  280 |     await page.waitForTimeout(500);
  281 |     
  282 |     if (errors.length > 0) {
  283 |       console.log('Errors in create modal:', JSON.stringify(errors, null, 2));
  284 |       throw new Error(`Found ${errors.length} errors`);
  285 |     }
  286 |   });
  287 |   
  288 |   test('world card interactions should not produce errors', async ({ page }) => {
  289 |     const errors: ConsoleError[] = [];
  290 |     
  291 |     page.on('console', (msg: ConsoleMessage) => {
  292 |       if (msg.type() === 'error') {
  293 |         errors.push({
  294 |           type: 'console.error',
  295 |           text: msg.text(),
  296 |           url: page.url()
  297 |         });
  298 |       }
  299 |     });
  300 |     
  301 |     page.on('pageerror', (err: Error) => {
  302 |       errors.push({
  303 |         type: 'pageerror',
  304 |         text: err.message,
  305 |         url: page.url(),
  306 |         stack: err.stack
  307 |       });
  308 |     });
  309 |     
  310 |     await page.goto('http://localhost:8765/');
  311 |     await page.waitForTimeout(3000);
  312 |     
  313 |     // Hover over world cards
  314 |     const cards = page.locator('.world-card');
  315 |     const count = await cards.count();
  316 |     
  317 |     for (let i = 0; i < Math.min(count, 3); i++) {
  318 |       await cards.nth(i).hover();
  319 |       await page.waitForTimeout(200);
  320 |     }
  321 |     
  322 |     // Test filter buttons
  323 |     await page.locator('.filter-btn:has-text("Ready")').click();
  324 |     await page.waitForTimeout(500);
  325 |     
  326 |     await page.locator('.filter-btn:has-text("All")').click();
  327 |     await page.waitForTimeout(500);
  328 |     
  329 |     // Refresh button
  330 |     await page.locator('button:has-text("↻")').click();
  331 |     await page.waitForTimeout(2000);
  332 |     
  333 |     if (errors.length > 0) {
  334 |       console.log('Errors during card interactions:', JSON.stringify(errors, null, 2));
  335 |       throw new Error(`Found ${errors.length} errors during interactions`);
  336 |     }
  337 |   });
```