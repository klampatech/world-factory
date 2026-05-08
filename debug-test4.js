const { chromium } = require('@playwright/test');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  page.on('console', msg => {
    console.log(`CONSOLE [${msg.type()}]:`, msg.text());
  });
  
  page.on('pageerror', error => {
    console.log('PAGE ERROR:', error.message);
  });
  
  await page.goto('http://localhost:9000/');
  await page.waitForTimeout(3000);
  
  // Check if API functions are available
  const apiCheck = await page.evaluate(() => {
    return {
      hasCheckHealth: typeof checkHealth !== 'undefined',
      hasFetchWorlds: typeof fetchWorlds !== 'undefined',
      hasWorldApiClient: typeof WorldApiClient !== 'undefined',
      hasApi: typeof api !== 'undefined'
    };
  });
  console.log('API Functions:', apiCheck);
  
  // Check modal state
  const modalState = await page.$eval('#generate-modal', el => el.className);
  console.log('Initial modal class:', modalState);
  
  // Try clicking the button
  await page.click('#generate-btn');
  await page.waitForTimeout(500);
  
  const modalStateAfter = await page.$eval('#generate-modal', el => el.className);
  console.log('Modal class after click:', modalStateAfter);
  
  // Check for input visibility
  const inputVisible = await page.$eval('#world-name-input', el => {
    const style = window.getComputedStyle(el);
    return style.display !== 'none' && style.visibility !== 'hidden';
  });
  console.log('Input visible:', inputVisible);
  
  await browser.close();
})();
