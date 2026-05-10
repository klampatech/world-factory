/**
 * WOR-1029: Playwright Screenshot Capture
 * Captures screenshots of the frontend UI for smoke test verification
 */

const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const FRONTEND_URL = 'http://localhost:8765';
const SCREENSHOTS_DIR = 'screenshots/WOR-1029';
const SCREENSHOT_LOG = 'screenshots/WOR-1029/screenshot-log.txt';

// Ensure screenshot directory exists
if (!fs.existsSync(SCREENSHOTS_DIR)) {
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
}

const screenshotLog = [];

async function capturePage(page, name, url) {
  console.log(`  📸 Capturing ${name}...`);
  try {
    await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(2000); // Allow JS to render
    
    const filePath = path.join(SCREENSHOTS_DIR, `${name}.png`);
    await page.screenshot({ path: filePath, fullPage: true });
    screenshotLog.push({ name, file: `${name}.png`, url, status: 'success' });
    console.log(`    ✅ Saved: ${name}.png`);
    return true;
  } catch (e) {
    screenshotLog.push({ name, file: null, url, status: 'failed', error: e.message });
    console.log(`    ❌ Failed: ${e.message}`);
    return false;
  }
}

async function captureConsoleErrors(page, name) {
  const errors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });
  await page.goto(FRONTEND_URL, { waitUntil: 'networkidle', timeout: 30000 });
  await page.waitForTimeout(2000);
  
  const errorFile = path.join(SCREENSHOTS_DIR, `${name}-console-errors.txt`);
  if (errors.length > 0) {
    fs.writeFileSync(errorFile, errors.join('\n'));
    console.log(`    ⚠️  Console errors captured: ${errors.length}`);
  } else {
    fs.writeFileSync(errorFile, 'No console errors detected');
    console.log(`    ✅ No console errors`);
  }
  return errors;
}

async function runPlaywrightTests() {
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║      WOR-1029: Playwright Screenshot Capture               ║');
  console.log('╚════════════════════════════════════════════════════════════╝\n');
  
  let browser;
  try {
    console.log('Launching browser...');
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1920, height: 1080 } });
    const page = await context.newPage();
    
    // Track console errors
    const consoleErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push({ timestamp: new Date().toISOString(), text: msg.text() });
      }
    });
    
    console.log('=== Capturing Screenshots ===\n');
    
    // 1. Main page / World list
    await capturePage(page, '01-main-page', FRONTEND_URL);
    
    // 2. World creation page (if accessible)
    await capturePage(page, '02-create-world', `${FRONTEND_URL}/#/create`);
    await capturePage(page, '03-create-world-alt', `${FRONTEND_URL}/#/worlds/create`);
    
    // 3. World view (if a world ID is known)
    await capturePage(page, '04-world-view', `${FRONTEND_URL}/#/world`);
    
    // 4. Try to find and view a world
    try {
      const response = await page.request.get('http://localhost:8080/api/v1/worlds');
      const data = await response.json();
      if (data?.data?.worlds?.length > 0) {
        const worldId = data.data.worlds[0].id;
        await capturePage(page, '05-world-detail', `${FRONTEND_URL}/#/world/${worldId}`);
      }
    } catch (e) {
      console.log('  ⚠️  Could not fetch worlds for detail view');
    }
    
    // 5. Check map view
    await capturePage(page, '06-map-view', `${FRONTEND_URL}/#/map`);
    
    // 6. Check history/timeline
    await capturePage(page, '07-timeline', `${FRONTEND_URL}/#/history`);
    
    // Capture console errors summary
    console.log('\n=== Console Error Summary ===\n');
    if (consoleErrors.length > 0) {
      console.log(`⚠️  Found ${consoleErrors.length} console errors:`);
      consoleErrors.forEach((err, i) => {
        console.log(`  ${i + 1}. ${err.text}`);
      });
    } else {
      console.log('✅ No console errors detected');
    }
    
    // Write console errors to file
    const errorsFile = path.join(SCREENSHOTS_DIR, 'console-errors.txt');
    fs.writeFileSync(errorsFile, JSON.stringify(consoleErrors, null, 2));
    
    // Write screenshot log
    const logContent = screenshotLog.map(s => 
      `[${s.status}] ${s.name} - ${s.url}${s.error ? ` - Error: ${s.error}` : ''}`
    ).join('\n');
    fs.writeFileSync(SCREENSHOT_LOG, logContent);
    
    console.log('\n=== Screenshots Captured ===\n');
    screenshotLog.forEach(s => {
      const icon = s.status === 'success' ? '✅' : '❌';
      console.log(`${icon} ${s.name}: ${s.file || s.error}`);
    });
    
    const successCount = screenshotLog.filter(s => s.status === 'success').length;
    console.log(`\nTotal: ${successCount}/${screenshotLog.length} screenshots captured`);
    
    await browser.close();
    
    return { success: successCount, total: screenshotLog.length, errors: consoleErrors };
  } catch (e) {
    console.error('Fatal error:', e.message);
    if (browser) await browser.close();
    throw e;
  }
}

runPlaywrightTests()
  .then(result => {
    console.log(`\n✅ Playwright capture complete: ${result.success}/${result.total} screenshots`);
    process.exit(result.errors.length > 0 ? 1 : 0);
  })
  .catch(err => {
    console.error('Fatal error:', err);
    process.exit(1);
  });