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
  
  await page.goto('http://localhost:9000/');
  await page.waitForTimeout(3000);
  
  // Check if the button exists and has onclick
  const btnExists = await page.$('#generate-btn');
  console.log('Button exists:', !!btnExists);
  
  // Try to click and check class
  if (btnExists) {
    await btnExists.click();
    await page.waitForTimeout(1000);
    const modalClass = await page.$eval('#generate-modal', el => el.className);
    console.log('Modal class after click:', modalClass);
  }
  
  // Check if there's any event listener
  const hasEventListener = await page.evaluate(() => {
    const btn = document.getElementById('generate-btn');
    // Check if function is defined
    return typeof openGenerateModal !== 'undefined';
  });
  console.log('openGenerateModal defined:', hasEventListener);
  
  // Check page error
  const errorPage = await page.evaluate(() => {
    const main = document.querySelector('main');
    return main ? main.textContent.substring(0, 200) : 'No main';
  });
  console.log('Main content:', errorPage);
  
  await browser.close();
})();
