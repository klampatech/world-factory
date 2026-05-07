import { test, expect, Page, ConsoleMessage } from '@playwright/test';

interface ConsoleError {
  type: string;
  text: string;
  url: string;
  line?: number;
  column?: number;
  stack?: string;
}

async function captureConsoleErrors(page: Page): Promise<ConsoleError[]> {
  const errors: ConsoleError[] = [];
  
  page.on('console', (msg: ConsoleMessage) => {
    if (msg.type() === 'error') {
      errors.push({
        type: 'console.error',
        text: msg.text(),
        url: page.url()
      });
    }
  });
  
  page.on('pageerror', (err: Error) => {
    errors.push({
      type: 'pageerror',
      text: err.message,
      url: page.url(),
      stack: err.stack
    });
  });
  
  return errors;
}

/**
 * Filter out benign errors that are expected in CI (no backend available).
 * These include network errors from the frontend trying to reach the backend API.
 */
function filterBenignErrors(errors: ConsoleError[]): ConsoleError[] {
  return errors.filter(e => {
    const text = e.text.toLowerCase();
    // Filter out network errors when backend is not available
    if (text.includes('err_connection_refused')) return false;
    if (text.includes('failed to load resource')) return false;
    if (text.includes('err_name_not_resolved')) return false;
    if (text.includes('favicon')) return false;
    // Filter backend API errors (frontend runs without backend in CI)
    if (e.url && e.url.includes('localhost:8080')) return false;
    return true;
  });
}

test.describe('WOR-204: Console Error Detection', () => {
  
  test('main page should have no console errors', async ({ page }) => {
    const errors = await captureConsoleErrors(page);
    
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(2000); // Wait for any async operations
    
    const criticalErrors = filterBenignErrors(errors);
    
    console.log('Console errors on main page:', JSON.stringify(criticalErrors, null, 2));
    
    if (criticalErrors.length > 0) {
      throw new Error(`Found ${criticalErrors.length} console errors:\n${JSON.stringify(criticalErrors, null, 2)}`);
    }
  });
  
  test('world selector view should load without errors', async ({ page }) => {
    const errors: ConsoleError[] = [];
    
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') {
        errors.push({
          type: 'console.error',
          text: msg.text(),
          url: page.url()
        });
      }
    });
    
    page.on('pageerror', (err: Error) => {
      errors.push({
        type: 'pageerror',
        text: err.message,
        url: page.url(),
        stack: err.stack
      });
    });
    
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(3000);
    
    // Verify the page loaded
    await expect(page.locator('.hero h2')).toContainText('Choose Your World');
    
    // Check for stats that indicate API loaded
    const totalWorlds = await page.locator('.stat-value').first().textContent();
    console.log('Total worlds displayed:', totalWorlds);
    
    const criticalErrors = filterBenignErrors(errors);
    if (criticalErrors.length > 0) {
      console.log('Errors found:', JSON.stringify(criticalErrors, null, 2));
      throw new Error(`Found ${criticalErrors.length} errors`);
    }
  });
  
  test('map view should load without errors', async ({ page }) => {
    const errors: ConsoleError[] = [];
    
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') {
        errors.push({
          type: 'console.error',
          text: msg.text(),
          url: page.url()
        });
      }
    });
    
    page.on('pageerror', (err: Error) => {
      errors.push({
        type: 'pageerror',
        text: err.message,
        url: page.url(),
        stack: err.stack
      });
    });
    
    // First get a world ID
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(2000);
    
    // Click on first ready world if exists
    const readyWorld = page.locator('.world-card .status-badge.ready').first();
    if (await readyWorld.count() > 0) {
      await page.locator('.btn-view.primary').first().click();
      await page.waitForTimeout(3000);
      
      // Verify map canvas exists
      await expect(page.locator('#map-canvas')).toBeVisible();
    }
    
    const criticalErrors = filterBenignErrors(errors);
    if (criticalErrors.length > 0) {
      console.log('Errors in map view:', JSON.stringify(criticalErrors, null, 2));
      throw new Error(`Found ${criticalErrors.length} errors in map view`);
    }
  });
  
  test('timeline view should load without errors', async ({ page }) => {
    const errors: ConsoleError[] = [];
    
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') {
        // Ignore 400 errors for non-existent worlds in navigation - expected
        const text = msg.text();
        if (text.includes('400') && text.includes('Failed to load resource')) {
          console.log('Ignoring expected 400 for fake world ID navigation');
          return;
        }
        errors.push({
          type: 'console.error',
          text: text,
          url: page.url()
        });
      }
    });
    
    page.on('pageerror', (err: Error) => {
      errors.push({
        type: 'pageerror',
        text: err.message,
        url: page.url(),
        stack: err.stack
      });
    });
    
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(2000);
    
    const readyWorld = page.locator('.world-card .status-badge.ready').first();
    if (await readyWorld.count() > 0) {
      // Click on a real world to navigate
      await page.locator('.btn-view.primary').first().click();
      await page.waitForTimeout(2000);
      
      // Navigate to timeline tab
      await page.locator('.view-tab:has-text("Timeline")').click();
      await page.waitForTimeout(2000);
      
      // Verify timeline container
      await expect(page.locator('.timeline-container')).toBeVisible();
    } else {
      console.log('No ready worlds found, skipping timeline test');
    }
    
    const criticalErrors = filterBenignErrors(errors);
    if (criticalErrors.length > 0) {
      console.log('Errors in timeline view:', JSON.stringify(criticalErrors, null, 2));
      throw new Error(`Found ${criticalErrors.length} errors in timeline view`);
    }
  });
  
  test('dashboard view should load without errors', async ({ page }) => {
    const errors: ConsoleError[] = [];
    
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') {
        // Ignore 400 errors for non-existent worlds in navigation - expected
        const text = msg.text();
        if (text.includes('400') && text.includes('Failed to load resource')) {
          console.log('Ignoring expected 400 for fake world ID navigation');
          return;
        }
        errors.push({
          type: 'console.error',
          text: text,
          url: page.url()
        });
      }
    });
    
    page.on('pageerror', (err: Error) => {
      errors.push({
        type: 'pageerror',
        text: err.message,
        url: page.url(),
        stack: err.stack
      });
    });
    
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(2000);
    
    const readyWorld = page.locator('.world-card .status-badge.ready').first();
    if (await readyWorld.count() > 0) {
      // Click on a real world to navigate
      await page.locator('.btn-view.primary').first().click();
      await page.waitForTimeout(2000);
      
      // Navigate to dashboard tab
      await page.locator('.view-tab:has-text("Dashboard")').click();
      await page.waitForTimeout(2000);
      
      // Verify dashboard container
      await expect(page.locator('.dashboard-container')).toBeVisible();
    } else {
      console.log('No ready worlds found, skipping dashboard test');
    }
    
    const criticalErrors = filterBenignErrors(errors);
    if (criticalErrors.length > 0) {
      console.log('Errors in dashboard view:', JSON.stringify(criticalErrors, null, 2));
      throw new Error(`Found ${criticalErrors.length} errors in dashboard view`);
    }
  });
  
  test('create world modal should work without errors', async ({ page }) => {
    const errors: ConsoleError[] = [];
    
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') {
        errors.push({
          type: 'console.error',
          text: msg.text(),
          url: page.url()
        });
      }
    });
    
    page.on('pageerror', (err: Error) => {
      errors.push({
        type: 'pageerror',
        text: err.message,
        url: page.url(),
        stack: err.stack
      });
    });
    
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(2000);
    
    // Open create modal - use first() to avoid strict mode violation
    await page.locator('.btn-create').first().click();
    await page.waitForTimeout(500);
    
    // Verify modal opened
    await expect(page.locator('#create-modal')).toHaveClass(/active/);
    
    // Interact with form elements
    await page.fill('#world-name', 'Test World Console Check');
    await page.locator('#world-width').fill('32');
    await page.waitForTimeout(200);
    
    // Close modal
    await page.locator('.btn:has-text("Cancel")').click();
    await page.waitForTimeout(500);
    
    const criticalErrors = filterBenignErrors(errors);
    if (criticalErrors.length > 0) {
      console.log('Errors in create modal:', JSON.stringify(criticalErrors, null, 2));
      throw new Error(`Found ${criticalErrors.length} errors`);
    }
  });
  
  test('world card interactions should not produce errors', async ({ page }) => {
    const errors: ConsoleError[] = [];
    
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') {
        errors.push({
          type: 'console.error',
          text: msg.text(),
          url: page.url()
        });
      }
    });
    
    page.on('pageerror', (err: Error) => {
      errors.push({
        type: 'pageerror',
        text: err.message,
        url: page.url(),
        stack: err.stack
      });
    });
    
    await page.goto('http://localhost:8765/');
    await page.waitForTimeout(3000);
    
    // Hover over world cards
    const cards = page.locator('.world-card');
    const count = await cards.count();
    
    for (let i = 0; i < Math.min(count, 3); i++) {
      await cards.nth(i).hover();
      await page.waitForTimeout(200);
    }
    
    // Test filter buttons
    await page.locator('.filter-btn:has-text("Ready")').click();
    await page.waitForTimeout(500);
    
    await page.locator('.filter-btn:has-text("All")').click();
    await page.waitForTimeout(500);
    
    // Refresh button
    await page.locator('button:has-text("↻")').click();
    await page.waitForTimeout(2000);
    
    const criticalErrors = filterBenignErrors(errors);
    if (criticalErrors.length > 0) {
      console.log('Errors during card interactions:', JSON.stringify(criticalErrors, null, 2));
      throw new Error(`Found ${criticalErrors.length} errors during interactions`);
    }
  });
});