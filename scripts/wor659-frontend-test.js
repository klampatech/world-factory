const { chromium } = require('playwright');

(async () => {
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    const page = await context.newPage();
    
    // Collect console errors
    const consoleErrors = [];
    page.on('console', msg => {
        if (msg.type() === 'error') {
            consoleErrors.push(msg.text());
        }
    });
    
    console.log('=== WOR-659 SMOKE TEST - FRONTEND VERIFICATION ===\n');
    
    // Test 1: World Selector Landing Page
    console.log('1. Testing World Selector landing page...');
    await page.goto('http://localhost:8787/', { waitUntil: 'networkidle', timeout: 15000 });
    await page.waitForTimeout(2000);
    
    const title = await page.title();
    console.log('   Page title: ' + title);
    
    const pageTitle = await page.textContent('.page-title').catch(() => 'NOT FOUND');
    console.log('   Header title: ' + pageTitle);
    
    const serverStatus = await page.textContent('#server-status-text').catch(() => 'NOT FOUND');
    console.log('   Server status: ' + serverStatus);
    
    // Take screenshot
    await page.screenshot({ path: 'screenshots/wor659-01-landing-page.png' });
    console.log('   Screenshot: screenshots/wor659-01-landing-page.png');
    
    // Test 2: API Health Check
    console.log('\n2. Testing API connectivity...');
    const healthResponse = await page.evaluate(async () => {
        try {
            const res = await fetch('/api/v1/health');
            return { status: res.status, ok: res.ok };
        } catch (e) {
            return { error: e.message };
        }
    });
    console.log('   Health check: ' + JSON.stringify(healthResponse));
    
    // Test 3: World List Loading
    console.log('\n3. Testing world list loading...');
    await page.waitForSelector('#world-grid', { timeout: 5000 }).catch(() => {});
    const worldCards = await page.$$('.world-list-card');
    console.log('   Worlds found: ' + worldCards.length);
    
    // Test 4: Generate Modal
    console.log('\n4. Testing generate modal...');
    await page.click('.generate-btn', { timeout: 5000 }).catch(() => console.log('   Generate button not found'));
    await page.waitForTimeout(500);
    const modalVisible = await page.isVisible('#generate-modal.active');
    console.log('   Modal visible: ' + modalVisible);
    
    // Fill form
    if (modalVisible) {
        await page.fill('#world-name-input', 'Smoke Test World');
        await page.screenshot({ path: 'screenshots/wor659-02-modal-open.png' });
        console.log('   Screenshot: screenshots/wor659-02-modal-open.png');
        
        // Close modal
        await page.click('#modal-cancel');
        await page.waitForTimeout(300);
    }
    
    // Test 5: Click on a world card if exists
    if (worldCards.length > 0) {
        console.log('\n5. Testing world detail view...');
        await worldCards[0].click();
        await page.waitForTimeout(2000);
        
        const worldTitle = await page.textContent('.world-name').catch(() => 'NOT FOUND');
        console.log('   World name: ' + worldTitle);
        
        // Check tabs
        const tabs = await page.$$('.tab-button');
        console.log('   Tabs found: ' + tabs.length);
        
        await page.screenshot({ path: 'screenshots/wor659-03-world-detail.png' });
        console.log('   Screenshot: screenshots/wor659-03-world-detail.png');
        
        // Test Map Tab
        console.log('\n6. Testing Map tab...');
        const mapTab = await page.$('.tab-button[data-tab="map"]');
        if (mapTab) {
            await mapTab.click();
            await page.waitForTimeout(1000);
            await page.screenshot({ path: 'screenshots/wor659-04-map-tab.png' });
            console.log('   Screenshot: screenshots/wor659-04-map-tab.png');
            
            const mapCanvas = await page.$('#world-map');
            console.log('   Canvas element found: ' + (mapCanvas !== null));
        }
        
        // Test Timeline Tab
        console.log('\n7. Testing Timeline tab...');
        const timelineTab = await page.$('.tab-button[data-tab="timeline"]');
        if (timelineTab) {
            await timelineTab.click();
            await page.waitForTimeout(1000);
            await page.screenshot({ path: 'screenshots/wor659-05-timeline-tab.png' });
            console.log('   Screenshot: screenshots/wor659-05-timeline-tab.png');
        }
        
        // Test Dashboard Tab
        console.log('\n8. Testing Dashboard tab...');
        const dashboardTab = await page.$('.tab-button[data-tab="dashboard"]');
        if (dashboardTab) {
            await dashboardTab.click();
            await page.waitForTimeout(1000);
            await page.screenshot({ path: 'screenshots/wor659-06-dashboard-tab.png' });
            console.log('   Screenshot: screenshots/wor659-06-dashboard-tab.png');
        }
    }
    
    // Summary
    console.log('\n=== CONSOLE ERRORS ===');
    if (consoleErrors.length === 0) {
        console.log('No console errors detected!');
    } else {
        console.log('Errors found: ' + consoleErrors.length);
        consoleErrors.forEach((err, i) => console.log((i+1) + '. ' + err.substring(0, 200)));
    }
    
    await browser.close();
    
    console.log('\n=== SCREENSHOTS CAPTURED ===');
    console.log('screenshots/wor659-01-landing-page.png');
    console.log('screenshots/wor659-02-modal-open.png');
    console.log('screenshots/wor659-03-world-detail.png');
    console.log('screenshots/wor659-04-map-tab.png');
    console.log('screenshots/wor659-05-timeline-tab.png');
    console.log('screenshots/wor659-06-dashboard-tab.png');
})();