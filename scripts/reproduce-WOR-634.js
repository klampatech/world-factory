/**
 * WOR-634: Bug Reproduction Test
 * World Selector shows 0 cards despite API returning 20 worlds
 */

const { chromium } = require('playwright');

async function reproduceBug() {
    console.log('=== WOR-634 Bug Reproduction Test ===\n');
    
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext();
    const page = await context.newPage();
    
    // Capture console messages
    const consoleLogs = [];
    page.on('console', msg => {
        consoleLogs.push({ type: msg.type(), text: msg.text() });
    });
    
    // Capture network requests
    const networkRequests = [];
    page.on('request', req => {
        networkRequests.push({ url: req.url(), method: req.method() });
    });
    
    // Capture network responses
    const networkResponses = [];
    page.on('response', async resp => {
        if (resp.url().includes('/api/')) {
            try {
                const body = await resp.text();
                networkResponses.push({ url: resp.url(), status: resp.status(), body: body.substring(0, 500) });
            } catch (e) {
                networkResponses.push({ url: resp.url(), status: resp.status(), error: e.message });
            }
        }
    });
    
    try {
        console.log('1. Opening World Selector at http://localhost:8765...');
        await page.goto('http://localhost:8765', { waitUntil: 'networkidle', timeout: 10000 });
        
        // Wait a bit for any async loading
        await page.waitForTimeout(2000);
        
        console.log('\n2. Checking server status...');
        const serverStatus = await page.$eval('#server-status-text', el => el.textContent).catch(() => 'not found');
        console.log(`   Server Status: "${serverStatus}"`);
        
        console.log('\n3. Checking for world cards...');
        const worldCards = await page.$$('.world-list-card');
        console.log(`   Found ${worldCards.length} .world-list-card elements`);
        
        const worldGrid = await page.$('#world-grid');
        if (worldGrid) {
            const innerHTML = await worldGrid.innerHTML();
            console.log(`   World grid innerHTML length: ${innerHTML.length} chars`);
            if (innerHTML.length < 100) {
                console.log(`   World grid content: "${innerHTML.substring(0, 200)}"`);
            }
        }
        
        console.log('\n4. Checking empty state visibility...');
        const emptyState = await page.$('#empty-state');
        if (emptyState) {
            const isVisible = await emptyState.isVisible();
            console.log(`   Empty state visible: ${isVisible}`);
        }
        
        console.log('\n5. Network API responses:');
        for (const resp of networkResponses) {
            console.log(`   ${resp.status} ${resp.url}`);
            if (resp.body) {
                try {
                    const json = JSON.parse(resp.body);
                    if (json.data?.worlds) {
                        console.log(`      → Contains ${json.data.worlds.length} worlds`);
                    }
                } catch (e) {}
            }
        }
        
        console.log('\n6. Console errors:');
        const errors = consoleLogs.filter(l => l.type === 'error');
        if (errors.length > 0) {
            errors.forEach(e => console.log(`   ERROR: ${e.text}`));
        } else {
            console.log('   No console errors');
        }
        
        console.log('\n=== Result ===');
        if (worldCards.length === 0) {
            console.log('BUG CONFIRMED: 0 world cards displayed');
            const apiResponse = networkResponses.find(r => r.url.includes('/api/v1/worlds'));
            if (apiResponse && apiResponse.body) {
                try {
                    const json = JSON.parse(apiResponse.body);
                    const worldCount = json.data?.worlds?.length || 0;
                    console.log(`API returned ${worldCount} worlds but UI shows 0 cards`);
                } catch (e) {}
            }
        } else {
            console.log('PASS: World cards are displaying correctly');
        }
        
    } catch (error) {
        console.error('Test error:', error.message);
    } finally {
        await browser.close();
    }
}

reproduceBug().catch(console.error);