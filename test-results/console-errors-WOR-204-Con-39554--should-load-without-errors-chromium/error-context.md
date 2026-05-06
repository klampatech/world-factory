# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: console-errors.spec.ts >> WOR-204: Console Error Detection >> timeline view should load without errors
- Location: e2e/console-errors.spec.ts:135:7

# Error details

```
Error: Found 1 errors in timeline view
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
          - generic [ref=e51]: Mountain
          - generic [ref=e54]: Grassland
          - generic [ref=e57]: Forest
          - generic [ref=e60]: Scrubland
          - generic [ref=e63]: Rainforest
          - generic [ref=e66]: Swamp
          - generic [ref=e69]: Desert
          - generic [ref=e72]: Beach
          - generic [ref=e75]: Highland
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
  84  |     const totalWorlds = await page.locator('.stat-value').first().textContent();
  85  |     console.log('Total worlds displayed:', totalWorlds);
  86  |     
  87  |     if (errors.length > 0) {
  88  |       console.log('Errors found:', JSON.stringify(errors, null, 2));
  89  |       throw new Error(`Found ${errors.length} errors`);
  90  |     }
  91  |   });
  92  |   
  93  |   test('map view should load without errors', async ({ page }) => {
  94  |     const errors: ConsoleError[] = [];
  95  |     
  96  |     page.on('console', (msg: ConsoleMessage) => {
  97  |       if (msg.type() === 'error') {
  98  |         errors.push({
  99  |           type: 'console.error',
  100 |           text: msg.text(),
  101 |           url: page.url()
  102 |         });
  103 |       }
  104 |     });
  105 |     
  106 |     page.on('pageerror', (err: Error) => {
  107 |       errors.push({
  108 |         type: 'pageerror',
  109 |         text: err.message,
  110 |         url: page.url(),
  111 |         stack: err.stack
  112 |       });
  113 |     });
  114 |     
  115 |     // First get a world ID
  116 |     await page.goto('http://localhost:8765/');
  117 |     await page.waitForTimeout(2000);
  118 |     
  119 |     // Click on first ready world if exists
  120 |     const readyWorld = page.locator('.world-card .status-badge.ready').first();
  121 |     if (await readyWorld.count() > 0) {
  122 |       await page.locator('.btn-view.primary').first().click();
  123 |       await page.waitForTimeout(3000);
  124 |       
  125 |       // Verify map canvas exists
  126 |       await expect(page.locator('#map-canvas')).toBeVisible();
  127 |     }
  128 |     
  129 |     if (errors.length > 0) {
  130 |       console.log('Errors in map view:', JSON.stringify(errors, null, 2));
  131 |       throw new Error(`Found ${errors.length} errors in map view`);
  132 |     }
  133 |   });
  134 |   
  135 |   test('timeline view should load without errors', async ({ page }) => {
  136 |     const errors: ConsoleError[] = [];
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
> 184 |       throw new Error(`Found ${errors.length} errors in timeline view`);
      |             ^ Error: Found 1 errors in timeline view
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
  237 |       throw new Error(`Found ${errors.length} errors in dashboard view`);
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
```