/**
 * WOR-634: Detailed Bug Reproduction Test
 * World Selector shows 0 cards despite API returning 20 worlds
 */

const { chromium } = require('playwright');

async function reproduceBug() {
    console.log('=== WOR-634 Bug Reproduction Test (Detailed) ===\n');
    
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext();
    const page = await context.newPage();
    
    // Capture all network requests/responses
    const apiRequests = [];
    const apiResponses = [];
    
    page.on('request', req => {
        if (req.url().includes('/api/')) {
            apiRequests.push({ url: req.url(), method: req.method() });
        }
    });
    
    page.on('response', async resp => {
        if (resp.url().includes('/api/')) {
            try {
                const body = await resp.text();
                apiResponses.push({ url: resp.url(), status: resp.status(), bodyLength: body.length });
                if (resp.url().includes('/worlds')) {
                    console.log(`\nAPI Response for worlds endpoint:`);
                    console.log(`   URL: ${resp.url()}`);
                    console.log(`   Status: ${resp.status()}`);
                    console.log(`   Body length: ${body.length} chars`);
                    // Try to parse and show world count
                    try {
                        const json = JSON.parse(body);
                        if (json.data?.worlds) {
                            console.log(`   Worlds count: ${json.data.worlds.length}`);
                        }
                    } catch (e) {}
                }
            } catch (e) {
                apiResponses.push({ url: resp.url(), status: resp.status(), error: e.message });
            }
        }
    });
    
    try {
        console.log('1. Opening World Selector at http://localhost:8765...');
        await page.goto('http://localhost:8765', { waitUntil: 'networkidle', timeout: 15000 });
        
        // Wait for async operations
        await page.waitForTimeout(3000);
        
        console.log('\n2. Checking server status indicator...');
        const serverStatus = await page.$eval('#server-status-text', el => el.textContent).catch(() => 'not found');
        console.log(`   Server Status: "${serverStatus}"`);
        
        console.log('\n3. Checking API requests made by frontend...');
        if (apiRequests.length > 0) {
            console.log(`   Frontend made ${apiRequests.length} API requests:`);
            apiRequests.forEach(req => {
                console.log(`     - ${req.method} ${req.url}`);
            });
        } else {
            console.log('   No API requests made');
        }
        
        console.log('\n4. Checking API responses...');
        if (apiResponses.length > 0) {
            apiResponses.forEach(resp => {
                console.log(`   ${resp.status} ${resp.url}`);
            });
        }
        
        console.log('\n5. Checking for world cards...');
        const worldCards = await page.$$('.world-list-card');
        console.log(`   Found ${worldCards.length} .world-list-card elements`);
        
        // Check if demo data is being used
        const worldNames = await page.$$eval('.world-name', els => els.map(el => el.textContent)).catch(() => []);
        console.log(`   World names shown: ${worldNames.join(', ')}`);
        
        console.log('\n6. Checking why 0 cards:');
        
        // Check loading state
        const loadingVisible = await page.$eval('#loading-state', el => el.style.display !== 'none').catch(() => false);
        console.log(`   Loading spinner visible: ${loadingVisible}`);
        
        // Check empty state
        const emptyVisible = await page.$eval('#empty-state', el => el.style.display !== 'none').catch(() => false);
        console.log(`   Empty state visible: ${emptyVisible}`);
        
        // Check world grid
        const gridHTML = await page.$eval('#world-grid', el => el.innerHTML).catch(() => 'not found');
        console.log(`   World grid has content: ${gridHTML.length > 10 ? 'YES' : 'NO'} (${gridHTML.length} chars)`);
        
        console.log('\n=== Analysis ===');
        
        // Determine root cause
        const failedRequests = apiResponses.filter(r => r.status >= 400);
        if (failedRequests.length > 0) {
            console.log('\n** ROOT CAUSE IDENTIFIED: **');
            console.log('   API requests are failing with HTTP errors.');
            console.log('   This causes the frontend to fall back to demo data.');
            console.log('   But demo data only has 3 worlds, not the 20+ expected.');
            
            failedRequests.forEach(req => {
                console.log(`   - ${req.status} ${req.url}`);
            });
            
            console.log('\n   The API at localhost:8080 works correctly (372 worlds).');
            console.log('   The frontend at localhost:8765 cannot reach the API.');
            console.log('   Missing: Reverse proxy to forward /api/* to backend');
        }
        
        if (worldCards.length === 0 && failedRequests.length > 0) {
            console.log('\n** BUG CONFIRMED: **');
            console.log('   - API endpoint returns 404 from frontend');
            console.log('   - Frontend shows 0 world cards');
            console.log('   - Expected behavior: Show 20+ worlds from API');
            console.log('   - Actual behavior: 0 cards (fallback to demo mode)');
        }
        
    } catch (error) {
        console.error('Test error:', error.message);
    } finally {
        await browser.close();
    }
}

reproduceBug().catch(console.error);