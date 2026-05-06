# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: console-errors.spec.ts >> WOR-204: Console Error Detection >> world selector view should load without errors
- Location: e2e/console-errors.spec.ts:55:7

# Error details

```
Error: expect(locator).toContainText(expected) failed

Locator: locator('.hero h2')
Expected substring: "Choose Your World"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toContainText" with timeout 5000ms
  - waiting for locator('.hero h2')

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
          - generic [ref=e54]: Mountain
          - generic [ref=e57]: Highland
          - generic [ref=e60]: Desert
          - generic [ref=e63]: Forest
          - generic [ref=e66]: Swamp
          - generic [ref=e69]: Rainforest
          - generic [ref=e72]: Scrubland
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
  1   | import { test, expect, Page, ConsoleMessage } from '@playwright/test';
  2   | 
  3   | interface ConsoleError {
  4   |   type: string;
  5   |   text: string;
  6   |   url: string;
  7   |   line?: number;
  8   |   column?: number;
  9   |   stack?: string;
  10  | }
  11  | 
  12  | async function captureConsoleErrors(page: Page): Promise<ConsoleError[]> {
  13  |   const errors: ConsoleError[] = [];
  14  |   
  15  |   page.on('console', (msg: ConsoleMessage) => {
  16  |     if (msg.type() === 'error') {
  17  |       errors.push({
  18  |         type: 'console.error',
  19  |         text: msg.text(),
  20  |         url: page.url()
  21  |       });
  22  |     }
  23  |   });
  24  |   
  25  |   page.on('pageerror', (err: Error) => {
  26  |     errors.push({
  27  |       type: 'pageerror',
  28  |       text: err.message,
  29  |       url: page.url(),
  30  |       stack: err.stack
  31  |     });
  32  |   });
  33  |   
  34  |   return errors;
  35  | }
  36  | 
  37  | test.describe('WOR-204: Console Error Detection', () => {
  38  |   
  39  |   test('main page should have no console errors', async ({ page }) => {
  40  |     const errors = await captureConsoleErrors(page);
  41  |     
  42  |     await page.goto('http://localhost:8765/');
  43  |     await page.waitForTimeout(2000); // Wait for any async operations
  44  |     
  45  |     // Filter out expected warnings (not errors)
  46  |     const actualErrors = errors.filter(e => !e.text.includes('favicon'));
  47  |     
  48  |     console.log('Console errors on main page:', JSON.stringify(actualErrors, null, 2));
  49  |     
  50  |     if (actualErrors.length > 0) {
  51  |       throw new Error(`Found ${actualErrors.length} console errors:\n${JSON.stringify(actualErrors, null, 2)}`);
  52  |     }
  53  |   });
  54  |   
  55  |   test('world selector view should load without errors', async ({ page }) => {
  56  |     const errors: ConsoleError[] = [];
  57  |     
  58  |     page.on('console', (msg: ConsoleMessage) => {
  59  |       if (msg.type() === 'error') {
  60  |         errors.push({
  61  |           type: 'console.error',
  62  |           text: msg.text(),
  63  |           url: page.url()
  64  |         });
  65  |       }
  66  |     });
  67  |     
  68  |     page.on('pageerror', (err: Error) => {
  69  |       errors.push({
  70  |         type: 'pageerror',
  71  |         text: err.message,
  72  |         url: page.url(),
  73  |         stack: err.stack
  74  |       });
  75  |     });
  76  |     
  77  |     await page.goto('http://localhost:8765/');
  78  |     await page.waitForTimeout(3000);
  79  |     
  80  |     // Verify the page loaded
> 81  |     await expect(page.locator('.hero h2')).toContainText('Choose Your World');
      |                                            ^ Error: expect(locator).toContainText(expected) failed
  82  |     
  83  |     // Check for stats that indicate API loaded
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
```