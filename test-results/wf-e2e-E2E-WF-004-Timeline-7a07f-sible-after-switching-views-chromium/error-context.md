# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: wf-e2e.spec.ts >> E2E-WF-004: Timeline View >> E2E-WF-004.3: Map remains accessible after switching views
- Location: e2e/wf-e2e.spec.ts:262:7

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator:  locator('#map-canvas')
Expected: visible
Received: hidden
Timeout:  5000ms

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('#map-canvas')
    9 × locator resolved to <canvas width="1280" height="659" id="map-canvas"></canvas>
      - unexpected value "hidden"

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
      - button "Timeline" [active] [ref=e11] [cursor=pointer]
    - generic [ref=e12]:
      - button "Reset View" [ref=e13] [cursor=pointer]
      - button "Generate World" [ref=e14] [cursor=pointer]
  - main [ref=e15]:
    - generic [ref=e16]:
      - generic [ref=e17]:
        - generic [ref=e18]:
          - generic [ref=e19]: Event Type
          - combobox [ref=e20] [cursor=pointer]:
            - option "All Types" [selected]
            - option "War"
            - option "Discovery"
            - option "Settlement"
            - option "Plague"
            - option "Treaty"
            - option "Innovation"
        - generic [ref=e21]:
          - generic [ref=e22]: Society
          - combobox [ref=e23] [cursor=pointer]:
            - option "All Societies" [selected]
            - option "Kingdom of Aldoria"
            - option "Empire of Brenn"
            - option "Confederation of Caldara"
            - option "Realm of Drevon"
        - generic [ref=e24]:
          - generic [ref=e25]: Region
          - combobox [ref=e26] [cursor=pointer]:
            - option "All Regions" [selected]
            - option "Northern Plains"
            - option "Eastern Highlands"
            - option "Western Forests"
            - option "Southern Shores"
      - generic [ref=e27]:
        - generic [ref=e28]:
          - generic [ref=e29]:
            - generic [ref=e31]: "0"
            - generic [ref=e32]: Age of Origins
          - generic [ref=e33] [cursor=pointer]:
            - generic [ref=e34]:
              - generic [ref=e35]: Battle of Southern Shores
              - generic [ref=e36]: war
            - generic [ref=e37]:
              - generic [ref=e38]: 📅 100
              - generic [ref=e39]: 📍 Southern Shores
            - paragraph [ref=e40]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e42]: Confederation of Caldara
          - generic [ref=e43] [cursor=pointer]:
            - generic [ref=e44]:
              - generic [ref=e45]: Confederation of Caldara founds new settlement
              - generic [ref=e46]: settlement
            - generic [ref=e47]:
              - generic [ref=e48]: 📅 147
              - generic [ref=e49]: 📍 Northern Plains
            - paragraph [ref=e50]: Colonists establish a new foothold, beginning a new chapter in regional history.
            - generic [ref=e52]: Confederation of Caldara
          - generic [ref=e53] [cursor=pointer]:
            - generic [ref=e54]:
              - generic [ref=e55]: Explorers find ancient ruins in Southern Shores
              - generic [ref=e56]: discovery
            - generic [ref=e57]:
              - generic [ref=e58]: 📅 200
              - generic [ref=e59]: 📍 Southern Shores
            - paragraph [ref=e60]: Bold explorers venture into uncharted territory, making remarkable findings.
            - generic [ref=e62]: Realm of Drevon
          - generic [ref=e63] [cursor=pointer]:
            - generic [ref=e64]:
              - generic [ref=e65]: The Great Sickness hits Western Forests
              - generic [ref=e66]: plague
            - generic [ref=e67]:
              - generic [ref=e68]: 📅 261
              - generic [ref=e69]: 📍 Western Forests
            - paragraph [ref=e70]: A devastating illness spreads across the land, causing widespread suffering.
            - generic [ref=e72]: Confederation of Caldara
          - generic [ref=e73] [cursor=pointer]:
            - generic [ref=e74]:
              - generic [ref=e75]: Plague sweeps Eastern Highlands
              - generic [ref=e76]: plague
            - generic [ref=e77]:
              - generic [ref=e78]: 📅 359
              - generic [ref=e79]: 📍 Eastern Highlands
            - paragraph [ref=e80]: A devastating illness spreads across the land, causing widespread suffering.
            - generic [ref=e82]: Realm of Drevon
          - generic [ref=e83] [cursor=pointer]:
            - generic [ref=e84]:
              - generic [ref=e85]: Breakthrough in Eastern Highlands
              - generic [ref=e86]: innovation
            - generic [ref=e87]:
              - generic [ref=e88]: 📅 405
              - generic [ref=e89]: 📍 Eastern Highlands
            - paragraph [ref=e90]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e92]: Kingdom of Aldoria
          - generic [ref=e93] [cursor=pointer]:
            - generic [ref=e94]:
              - generic [ref=e95]: Scholars unlock secrets of Northern Plains
              - generic [ref=e96]: innovation
            - generic [ref=e97]:
              - generic [ref=e98]: 📅 440
              - generic [ref=e99]: 📍 Northern Plains
            - paragraph [ref=e100]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e102]: Realm of Drevon
          - generic [ref=e103] [cursor=pointer]:
            - generic [ref=e104]:
              - generic [ref=e105]: Kingdom of Aldoria invades Northern Plains
              - generic [ref=e106]: war
            - generic [ref=e107]:
              - generic [ref=e108]: 📅 495
              - generic [ref=e109]: 📍 Northern Plains
            - paragraph [ref=e110]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e112]: Kingdom of Aldoria
        - generic [ref=e113]:
          - generic [ref=e114]:
            - generic [ref=e116]: "500"
            - generic [ref=e117]: Era of Discovery
          - generic [ref=e118] [cursor=pointer]:
            - generic [ref=e119]:
              - generic [ref=e120]: Expedition reaches Northern Plains
              - generic [ref=e121]: discovery
            - generic [ref=e122]:
              - generic [ref=e123]: 📅 576
              - generic [ref=e124]: 📍 Northern Plains
            - paragraph [ref=e125]: Bold explorers venture into uncharted territory, making remarkable findings.
            - generic [ref=e127]: Realm of Drevon
          - generic [ref=e128] [cursor=pointer]:
            - generic [ref=e129]:
              - generic [ref=e130]: Great War of Northern Plains
              - generic [ref=e131]: war
            - generic [ref=e132]:
              - generic [ref=e133]: 📅 616
              - generic [ref=e134]: 📍 Northern Plains
            - paragraph [ref=e135]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e137]: Confederation of Caldara
          - generic [ref=e138] [cursor=pointer]:
            - generic [ref=e139]:
              - generic [ref=e140]: Empire of Brenn vs Realm of Drevon - Eastern Highlands
              - generic [ref=e141]: war
            - generic [ref=e142]:
              - generic [ref=e143]: 📅 677
              - generic [ref=e144]: 📍 Eastern Highlands
            - paragraph [ref=e145]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e147]: Empire of Brenn
          - generic [ref=e148] [cursor=pointer]:
            - generic [ref=e149]:
              - generic [ref=e150]: New farming technique in Southern Shores
              - generic [ref=e151]: innovation
            - generic [ref=e152]:
              - generic [ref=e153]: 📅 763
              - generic [ref=e154]: 📍 Southern Shores
            - paragraph [ref=e155]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e157]: Kingdom of Aldoria
          - generic [ref=e158] [cursor=pointer]:
            - generic [ref=e159]:
              - generic [ref=e160]: Trade agreement with Empire of Brenn
              - generic [ref=e161]: treaty
            - generic [ref=e162]:
              - generic [ref=e163]: 📅 786
              - generic [ref=e164]: 📍 Eastern Highlands
            - paragraph [ref=e165]: Diplomats negotiate a landmark agreement, bringing hope for lasting peace.
            - generic [ref=e167]: Empire of Brenn
          - generic [ref=e168] [cursor=pointer]:
            - generic [ref=e169]:
              - generic [ref=e170]: Battle of Eastern Highlands
              - generic [ref=e171]: war
            - generic [ref=e172]:
              - generic [ref=e173]: 📅 816
              - generic [ref=e174]: 📍 Eastern Highlands
            - paragraph [ref=e175]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e177]: Kingdom of Aldoria
          - generic [ref=e178] [cursor=pointer]:
            - generic [ref=e179]:
              - generic [ref=e180]: New farming technique in Northern Plains
              - generic [ref=e181]: innovation
            - generic [ref=e182]:
              - generic [ref=e183]: 📅 846
              - generic [ref=e184]: 📍 Northern Plains
            - paragraph [ref=e185]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e187]: Kingdom of Aldoria
          - generic [ref=e188] [cursor=pointer]:
            - generic [ref=e189]:
              - generic [ref=e190]: Alliance formed at Western Forests
              - generic [ref=e191]: treaty
            - generic [ref=e192]:
              - generic [ref=e193]: 📅 882
              - generic [ref=e194]: 📍 Western Forests
            - paragraph [ref=e195]: Diplomats negotiate a landmark agreement, bringing hope for lasting peace.
            - generic [ref=e197]: Realm of Drevon
          - generic [ref=e198] [cursor=pointer]:
            - generic [ref=e199]:
              - generic [ref=e200]: The Great Sickness hits Western Forests
              - generic [ref=e201]: plague
            - generic [ref=e202]:
              - generic [ref=e203]: 📅 954
              - generic [ref=e204]: 📍 Western Forests
            - paragraph [ref=e205]: A devastating illness spreads across the land, causing widespread suffering.
            - generic [ref=e207]: Realm of Drevon
          - generic [ref=e208] [cursor=pointer]:
            - generic [ref=e209]:
              - generic [ref=e210]: Trade agreement with Confederation of Caldara
              - generic [ref=e211]: treaty
            - generic [ref=e212]:
              - generic [ref=e213]: 📅 977
              - generic [ref=e214]: 📍 Eastern Highlands
            - paragraph [ref=e215]: Diplomats negotiate a landmark agreement, bringing hope for lasting peace.
            - generic [ref=e217]: Confederation of Caldara
        - generic [ref=e218]:
          - generic [ref=e219]:
            - generic [ref=e221]: "1000"
            - generic [ref=e222]: Age of Empires
          - generic [ref=e223] [cursor=pointer]:
            - generic [ref=e224]:
              - generic [ref=e225]: Battle of Northern Plains
              - generic [ref=e226]: war
            - generic [ref=e227]:
              - generic [ref=e228]: 📅 1029
              - generic [ref=e229]: 📍 Northern Plains
            - paragraph [ref=e230]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e232]: Empire of Brenn
          - generic [ref=e233] [cursor=pointer]:
            - generic [ref=e234]:
              - generic [ref=e235]: Disease spreads through Eastern Highlands
              - generic [ref=e236]: plague
            - generic [ref=e237]:
              - generic [ref=e238]: 📅 1053
              - generic [ref=e239]: 📍 Eastern Highlands
            - paragraph [ref=e240]: A devastating illness spreads across the land, causing widespread suffering.
            - generic [ref=e242]: Empire of Brenn
          - generic [ref=e243] [cursor=pointer]:
            - generic [ref=e244]:
              - generic [ref=e245]: Explorers find ancient ruins in Southern Shores
              - generic [ref=e246]: discovery
            - generic [ref=e247]:
              - generic [ref=e248]: 📅 1119
              - generic [ref=e249]: 📍 Southern Shores
            - paragraph [ref=e250]: Bold explorers venture into uncharted territory, making remarkable findings.
            - generic [ref=e252]: Confederation of Caldara
          - generic [ref=e253] [cursor=pointer]:
            - generic [ref=e254]:
              - generic [ref=e255]: Village of Northern Plains established
              - generic [ref=e256]: settlement
            - generic [ref=e257]:
              - generic [ref=e258]: 📅 1176
              - generic [ref=e259]: 📍 Northern Plains
            - paragraph [ref=e260]: Colonists establish a new foothold, beginning a new chapter in regional history.
            - generic [ref=e262]: Kingdom of Aldoria
          - generic [ref=e263] [cursor=pointer]:
            - generic [ref=e264]:
              - generic [ref=e265]: Empire of Brenn vs Realm of Drevon - Western Forests
              - generic [ref=e266]: war
            - generic [ref=e267]:
              - generic [ref=e268]: 📅 1263
              - generic [ref=e269]: 📍 Western Forests
            - paragraph [ref=e270]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e272]: Empire of Brenn
          - generic [ref=e273] [cursor=pointer]:
            - generic [ref=e274]:
              - generic [ref=e275]: New land discovered in Northern Plains
              - generic [ref=e276]: discovery
            - generic [ref=e277]:
              - generic [ref=e278]: 📅 1353
              - generic [ref=e279]: 📍 Northern Plains
            - paragraph [ref=e280]: Bold explorers venture into uncharted territory, making remarkable findings.
            - generic [ref=e282]: Empire of Brenn
          - generic [ref=e283] [cursor=pointer]:
            - generic [ref=e284]:
              - generic [ref=e285]: Peace treaty signed between Confederation of Caldara and Confederation of Caldara
              - generic [ref=e286]: treaty
            - generic [ref=e287]:
              - generic [ref=e288]: 📅 1452
              - generic [ref=e289]: 📍 Northern Plains
            - paragraph [ref=e290]: Diplomats negotiate a landmark agreement, bringing hope for lasting peace.
            - generic [ref=e292]: Confederation of Caldara
          - generic [ref=e293] [cursor=pointer]:
            - generic [ref=e294]:
              - generic [ref=e295]: Colonists establish Western Forests outpost
              - generic [ref=e296]: settlement
            - generic [ref=e297]:
              - generic [ref=e298]: 📅 1494
              - generic [ref=e299]: 📍 Western Forests
            - paragraph [ref=e300]: Colonists establish a new foothold, beginning a new chapter in regional history.
            - generic [ref=e302]: Realm of Drevon
        - generic [ref=e303]:
          - generic [ref=e304]:
            - generic [ref=e306]: "1500"
            - generic [ref=e307]: Era of Conflict
          - generic [ref=e308] [cursor=pointer]:
            - generic [ref=e309]:
              - generic [ref=e310]: The Great Sickness hits Southern Shores
              - generic [ref=e311]: plague
            - generic [ref=e312]:
              - generic [ref=e313]: 📅 1533
              - generic [ref=e314]: 📍 Southern Shores
            - paragraph [ref=e315]: A devastating illness spreads across the land, causing widespread suffering.
            - generic [ref=e317]: Kingdom of Aldoria
          - generic [ref=e318] [cursor=pointer]:
            - generic [ref=e319]:
              - generic [ref=e320]: Cartographers chart Northern Plains
              - generic [ref=e321]: discovery
            - generic [ref=e322]:
              - generic [ref=e323]: 📅 1558
              - generic [ref=e324]: 📍 Northern Plains
            - paragraph [ref=e325]: Bold explorers venture into uncharted territory, making remarkable findings.
            - generic [ref=e327]: Confederation of Caldara
          - generic [ref=e328] [cursor=pointer]:
            - generic [ref=e329]:
              - generic [ref=e330]: Peace treaty signed between Confederation of Caldara and Kingdom of Aldoria
              - generic [ref=e331]: treaty
            - generic [ref=e332]:
              - generic [ref=e333]: 📅 1651
              - generic [ref=e334]: 📍 Southern Shores
            - paragraph [ref=e335]: Diplomats negotiate a landmark agreement, bringing hope for lasting peace.
            - generic [ref=e337]: Confederation of Caldara
          - generic [ref=e338] [cursor=pointer]:
            - generic [ref=e339]:
              - generic [ref=e340]: Breakthrough in Northern Plains
              - generic [ref=e341]: innovation
            - generic [ref=e342]:
              - generic [ref=e343]: 📅 1719
              - generic [ref=e344]: 📍 Northern Plains
            - paragraph [ref=e345]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e347]: Confederation of Caldara
          - generic [ref=e348] [cursor=pointer]:
            - generic [ref=e349]:
              - generic [ref=e350]: Realm of Drevon vs Kingdom of Aldoria - Southern Shores
              - generic [ref=e351]: war
            - generic [ref=e352]:
              - generic [ref=e353]: 📅 1812
              - generic [ref=e354]: 📍 Southern Shores
            - paragraph [ref=e355]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e357]: Realm of Drevon
          - generic [ref=e358] [cursor=pointer]:
            - generic [ref=e359]:
              - generic [ref=e360]: Realm of Drevon vs Empire of Brenn - Southern Shores
              - generic [ref=e361]: war
            - generic [ref=e362]:
              - generic [ref=e363]: 📅 1897
              - generic [ref=e364]: 📍 Southern Shores
            - paragraph [ref=e365]: Armies clash in a decisive battle that will shape the future of the region.
            - generic [ref=e367]: Realm of Drevon
          - generic [ref=e368] [cursor=pointer]:
            - generic [ref=e369]:
              - generic [ref=e370]: Scholars unlock secrets of Western Forests
              - generic [ref=e371]: innovation
            - generic [ref=e372]:
              - generic [ref=e373]: 📅 1987
              - generic [ref=e374]: 📍 Western Forests
            - paragraph [ref=e375]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e377]: Realm of Drevon
        - generic [ref=e378]:
          - generic [ref=e379]:
            - generic [ref=e381]: "2000"
            - generic [ref=e382]: Modern Age
          - generic [ref=e383] [cursor=pointer]:
            - generic [ref=e384]:
              - generic [ref=e385]: The Great Sickness hits Northern Plains
              - generic [ref=e386]: plague
            - generic [ref=e387]:
              - generic [ref=e388]: 📅 2037
              - generic [ref=e389]: 📍 Northern Plains
            - paragraph [ref=e390]: A devastating illness spreads across the land, causing widespread suffering.
            - generic [ref=e392]: Confederation of Caldara
          - generic [ref=e393] [cursor=pointer]:
            - generic [ref=e394]:
              - generic [ref=e395]: Breakthrough in Eastern Highlands
              - generic [ref=e396]: innovation
            - generic [ref=e397]:
              - generic [ref=e398]: 📅 2106
              - generic [ref=e399]: 📍 Eastern Highlands
            - paragraph [ref=e400]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e402]: Empire of Brenn
          - generic [ref=e403] [cursor=pointer]:
            - generic [ref=e404]:
              - generic [ref=e405]: Breakthrough in Eastern Highlands
              - generic [ref=e406]: innovation
            - generic [ref=e407]:
              - generic [ref=e408]: 📅 2130
              - generic [ref=e409]: 📍 Eastern Highlands
            - paragraph [ref=e410]: Brilliant minds innovate, creating technologies that transform society.
            - generic [ref=e412]: Empire of Brenn
          - generic [ref=e413] [cursor=pointer]:
            - generic [ref=e414]:
              - generic [ref=e415]: Trade agreement with Kingdom of Aldoria
              - generic [ref=e416]: treaty
            - generic [ref=e417]:
              - generic [ref=e418]: 📅 2199
              - generic [ref=e419]: 📍 Southern Shores
            - paragraph [ref=e420]: Diplomats negotiate a landmark agreement, bringing hope for lasting peace.
            - generic [ref=e422]: Kingdom of Aldoria
```

# Test source

```ts
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
  199 |     
  200 |     // Canvas should still be visible
  201 |     await expect(canvas).toBeVisible();
  202 |     console.log('✓ Map pan interaction works');
  203 |   });
  204 | 
  205 |   test('E2E-WF-003.2: Zoom controls are accessible', async ({ page }) => {
  206 |     // Look for any zoom control
  207 |     const zoomIn = page.locator('#zoom-in, .zoom-in, button:has-text("+"), [aria-label*="zoom in"]').first();
  208 |     
  209 |     if (await zoomIn.count() > 0) {
  210 |       await zoomIn.click();
  211 |       await page.waitForTimeout(200);
  212 |       console.log('✓ Zoom controls exist and are clickable');
  213 |     } else {
  214 |       console.log('  ℹ No dedicated zoom buttons found (zoom via scroll may be supported)');
  215 |     }
  216 |   });
  217 | 
  218 |   test('E2E-WF-003.3: Canvas maintains visibility after interactions', async ({ page }) => {
  219 |     const canvas = page.locator(MAP_CANVAS);
  220 |     
  221 |     // Multiple interactions
  222 |     await page.mouse.move(400, 300);
  223 |     await page.mouse.down();
  224 |     await page.mouse.move(500, 400);
  225 |     await page.mouse.up();
  226 |     
  227 |     await page.waitForTimeout(500);
  228 |     await expect(canvas).toBeVisible();
  229 |     console.log('✓ Canvas remains visible after pan');
  230 |   });
  231 | 
  232 | });
  233 | 
  234 | // =======================================================================
  235 | // E2E-WF-004: Timeline View
  236 | // =======================================================================
  237 | test.describe('E2E-WF-004: Timeline View', () => {
  238 |   
  239 |   test.beforeEach(async ({ page }) => {
  240 |     await page.goto(BASE_URL + '/');
  241 |     await waitForMapReady(page);
  242 |   });
  243 | 
  244 |   test('E2E-WF-004.1: Timeline tab/button exists', async ({ page }) => {
  245 |     const timelineTab = page.locator('.view-tab:has-text("Timeline"), #timeline-view, .timeline-container');
  246 |     await expect(timelineTab.first()).toBeVisible();
  247 |     console.log('✓ Timeline control exists');
  248 |   });
  249 | 
  250 |   test('E2E-WF-004.2: Timeline tab is clickable', async ({ page }) => {
  251 |     const timelineTab = page.locator('.view-tab:has-text("Timeline")');
  252 |     
  253 |     if (await timelineTab.count() > 0) {
  254 |       await timelineTab.click();
  255 |       await page.waitForTimeout(500);
  256 |       console.log('✓ Timeline tab is clickable');
  257 |     } else {
  258 |       console.log('  ℹ Timeline tab not found - may be integrated differently');
  259 |     }
  260 |   });
  261 | 
  262 |   test('E2E-WF-004.3: Map remains accessible after switching views', async ({ page }) => {
  263 |     // Map view is default
  264 |     await expect(page.locator(MAP_CANVAS)).toBeVisible();
  265 |     
  266 |     // Try timeline if exists
  267 |     const timelineTab = page.locator('.view-tab:has-text("Timeline")');
  268 |     if (await timelineTab.count() > 0) {
  269 |       await timelineTab.click();
  270 |       await page.waitForTimeout(300);
  271 |     }
  272 |     
  273 |     // Map should still be in DOM and functional
> 274 |     await expect(page.locator(MAP_CANVAS)).toBeVisible();
      |                                            ^ Error: expect(locator).toBeVisible() failed
  275 |     console.log('✓ Map remains after view switch');
  276 |   });
  277 | 
  278 | });
  279 | 
  280 | // =======================================================================
  281 | // E2E-WF-005: Header & Navigation
  282 | // =======================================================================
  283 | test.describe('E2E-WF-005: Header & Navigation', () => {
  284 |   
  285 |   test('E2E-WF-005.1: Header renders correctly', async ({ page }) => {
  286 |     await page.goto(BASE_URL + '/');
  287 |     
  288 |     const header = page.locator('header');
  289 |     await expect(header).toBeVisible();
  290 |     
  291 |     // Logo should be present
  292 |     const logo = page.locator('.logo, h1, [class*="logo"]');
  293 |     await expect(logo.first()).toBeVisible();
  294 |     console.log('✓ Header renders correctly');
  295 |   });
  296 | 
  297 |   test('E2E-WF-005.2: View tabs exist for navigation', async ({ page }) => {
  298 |     await page.goto(BASE_URL + '/');
  299 |     
  300 |     const tabs = page.locator('.view-tab');
  301 |     const tabCount = await tabs.count();
  302 |     
  303 |     expect(tabCount).toBeGreaterThan(0);
  304 |     console.log(`✓ ${tabCount} view tabs available`);
  305 |   });
  306 | 
  307 |   test('E2E-WF-005.3: Map view tab is active by default', async ({ page }) => {
  308 |     await page.goto(BASE_URL + '/');
  309 |     
  310 |     const mapTab = page.locator('.view-tab:has-text("Map"), .view-tab.active');
  311 |     await expect(mapTab.first()).toBeVisible();
  312 |     console.log('✓ Map view is default');
  313 |   });
  314 | 
  315 | });
  316 | 
  317 | // =======================================================================
  318 | // E2E-WF-006: Responsive Design
  319 | // =======================================================================
  320 | test.describe('E2E-WF-006: Responsive Design', () => {
  321 |   
  322 |   test('E2E-WF-006.1: Desktop viewport (1920x1080)', async ({ page }) => {
  323 |     await page.setViewportSize({ width: 1920, height: 1080 });
  324 |     await page.goto(BASE_URL + '/');
  325 |     await waitForMapReady(page);
  326 |     
  327 |     const canvas = page.locator(MAP_CANVAS);
  328 |     const box = await canvas.boundingBox();
  329 |     expect(box?.width).toBeGreaterThan(1000);
  330 |     console.log(`✓ Desktop layout: ${box?.width}x${box?.height}`);
  331 |   });
  332 | 
  333 |   test('E2E-WF-006.2: Tablet viewport (768x1024)', async ({ page }) => {
  334 |     await page.setViewportSize({ width: 768, height: 1024 });
  335 |     await page.goto(BASE_URL + '/');
  336 |     await waitForMapReady(page);
  337 |     
  338 |     const canvas = page.locator(MAP_CANVAS);
  339 |     await expect(canvas).toBeVisible();
  340 |     console.log('✓ Tablet layout works');
  341 |   });
  342 | 
  343 |   test('E2E-WF-006.3: Mobile viewport (375x667)', async ({ page }) => {
  344 |     await page.setViewportSize({ width: 375, height: 667 });
  345 |     await page.goto(BASE_URL + '/');
  346 |     await waitForMapReady(page);
  347 |     
  348 |     const controls = page.locator(OVERLAY_CONTROLS);
  349 |     await expect(controls).toBeVisible();
  350 |     
  351 |     const canvas = page.locator(MAP_CANVAS);
  352 |     await expect(canvas).toBeVisible();
  353 |     console.log('✓ Mobile layout works');
  354 |   });
  355 | 
  356 | });
  357 | 
  358 | // =======================================================================
  359 | // E2E-WF-007: Screenshot Capture for Visual QA
  360 | // =======================================================================
  361 | test.describe('E2E-WF-007: Visual QA Screenshots', () => {
  362 |   
  363 |   const SCREENSHOT_DIR = 'test-results/screenshots';
  364 | 
  365 |   test('E2E-WF-007.1: Capture initial page state', async ({ page }) => {
  366 |     await page.goto(BASE_URL + '/');
  367 |     await page.waitForTimeout(1500);
  368 |     
  369 |     await page.screenshot({ 
  370 |       path: `${SCREENSHOT_DIR}/E2E-WF-007-1-initial-state.png`,
  371 |       fullPage: true 
  372 |     });
  373 |     console.log('✓ Screenshot: initial state captured');
  374 |   });
```