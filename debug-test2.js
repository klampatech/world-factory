const { chromium } = require('@playwright/test');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  // Capture all errors
  page.on('console', msg => {
    console.log(`CONSOLE [${msg.type()}]:`, msg.text());
  });
  
  page.on('pageerror', error => {
    console.log('PAGE ERROR:', error.message);
  });
  
  await page.goto('http://localhost:9000/web/index.html');
  await page.waitForTimeout(3000);
  
  // Get page content
  const bodyHTML = await page.evaluate(() => document.body.innerHTML.substring(0, 2000));
  console.log('Body HTML (first 2000 chars):\n', bodyHTML);
  
  await browser.close();
})();
