import { test, expect } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:8765';

test('WOR-531 Frontend Screenshots', async ({ page }) => {
  // Capture home page
  await page.goto(FRONTEND_URL);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshots/WOR-531-01-frontend-home.png', fullPage: true });
  console.log('Home page screenshot captured');
  
  // Check title
  const title = await page.title();
  console.log('Page title:', title);
  expect(title).toContain('World Factory');
  
  // Check for UI elements
  const headerText = await page.locator('h1').first().textContent();
  console.log('Header:', headerText);
  
  // Check create button
  const createButton = page.locator('button:has-text("Create")');
  const buttonExists = await createButton.count() > 0;
  console.log('Create button exists:', buttonExists);
  
  await page.screenshot({ path: 'screenshots/WOR-531-02-frontend-after-load.png', fullPage: true });
  
  // Log any console errors
  const errors: string[] = [];
  page.on('console', msg => {
    if (msg.type() === 'error' && !msg.text().includes('favicon')) {
      errors.push(msg.text());
    }
  });
  
  await page.waitForTimeout(1000);
  
  if (errors.length > 0) {
    console.log('Console errors:', errors.join('\n'));
    await page.screenshot({ path: 'screenshots/WOR-531-03-console-errors.png', fullPage: true });
  }
  
  console.log('Screenshots captured successfully');
});
