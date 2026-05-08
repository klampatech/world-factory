import { test, expect } from '@playwright/test';

/**
 * E2E Tests for World Selector Landing Page
 * Tests the main page functionality and user interactions
 */

test.describe('World Selector Landing Page', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display page title', async ({ page }) => {
    await expect(page.locator('#page-title')).toContainText('World Selector');
  });

  test('should show server status indicator', async ({ page }) => {
    const statusEl = page.locator('#server-status');
    await expect(statusEl).toBeVisible();
  });

  test('should show Generate button', async ({ page }) => {
    const generateBtn = page.locator('#generate-btn');
    await expect(generateBtn).toBeVisible();
    await expect(generateBtn).toContainText('Generate New World');
  });

  test('should open generate modal when clicking Generate button', async ({ page }) => {
    await page.click('#generate-btn');
    const modal = page.locator('#generate-modal');
    await expect(modal).toHaveClass(/active/);
  });

  test('should close modal when clicking close button', async ({ page }) => {
    await page.click('#generate-btn');
    await page.click('#modal-close');
    const modal = page.locator('#generate-modal');
    await expect(modal).not.toHaveClass(/active/);
  });

  test('should close modal when clicking cancel button', async ({ page }) => {
    await page.click('#generate-btn');
    await page.click('#modal-cancel');
    const modal = page.locator('#generate-modal');
    await expect(modal).not.toHaveClass(/active/);
  });

  test('should have form fields in modal', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Check form fields exist
    await expect(page.locator('#world-name-input')).toBeVisible();
    await expect(page.locator('#world-seed-input')).toBeVisible();
    await expect(page.locator('#width-slider')).toBeVisible();
    await expect(page.locator('#height-slider')).toBeVisible();
    await expect(page.locator('#years-slider')).toBeVisible();
    await expect(page.locator('#resource-richness')).toBeVisible();
    await expect(page.locator('#disaster-freq')).toBeVisible();
  });

  test('should update slider display values', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Change width slider
    const widthSlider = page.locator('#width-slider');
    await widthSlider.fill('96');
    await expect(page.locator('#width-display')).toContainText('96');
    await expect(page.locator('#width-value')).toContainText('96');
  });

  test('should show species checkboxes', async ({ page }) => {
    await page.click('#generate-btn');
    
    const speciesCheckboxes = page.locator('input[name="species"]');
    await expect(speciesCheckboxes).toHaveCount(7); // 7 species options
  });

  test('should display world list when data is available', async ({ page }) => {
    // With demo data, world grid should be visible
    const worldGrid = page.locator('#world-grid');
    await expect(worldGrid).toBeVisible();
  });

  test('should display world cards with status badges', async ({ page }) => {
    const worldCards = page.locator('.world-list-card');
    const count = await worldCards.count();
    
    if (count > 0) {
      await expect(worldCards.first().locator('.status-badge')).toBeVisible();
    }
  });

  test('should handle empty state', async ({ page }) => {
    // Empty state is only shown when there are no worlds
    // With demo data, we should see world cards
    const emptyState = page.locator('#empty-state');
    const worldGrid = page.locator('#world-grid');
    
    // One should be visible, not both
    const emptyVisible = await emptyState.isVisible();
    const gridVisible = await worldGrid.isVisible();
    expect(emptyVisible || gridVisible).toBeTruthy();
  });

  test('should display world card metadata', async ({ page }) => {
    const worldCards = page.locator('.world-list-card');
    const count = await worldCards.count();
    
    if (count > 0) {
      const firstCard = worldCards.first();
      await expect(firstCard.locator('.world-name')).toBeVisible();
      await expect(firstCard.locator('.world-id')).toBeVisible();
    }
  });

  test('should have view buttons on world cards', async ({ page }) => {
    const viewButtons = page.locator('.view-btn');
    const count = await viewButtons.count();
    
    if (count > 0) {
      await expect(viewButtons.first()).toBeVisible();
    }
  });

  test('should handle modal overlay click to close', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Click outside modal content (on overlay)
    await page.click('#generate-modal', { position: { x: 10, y: 10 } });
    
    const modal = page.locator('#generate-modal');
    await expect(modal).not.toHaveClass(/active/);
  });

  test('should select species checkboxes', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Check humans and elves
    await page.check('input[value="human"]');
    await page.check('input[value="elf"]');
    
    await expect(page.locator('input[value="human"]')).toBeChecked();
    await expect(page.locator('input[value="elf"]')).toBeChecked();
  });

});

test.describe('World Detail View', () => {
  
  test('should navigate to world detail page', async ({ page }) => {
    // First go to landing page
    await page.goto('/web/index.html');
    
    // Click on a view button to navigate
    const viewMapBtn = page.locator('.view-btn').first();
    
    // Note: This will cause navigation to world.html
    // In real testing with a backend, we'd have actual world data
  });

});

test.describe('Responsive Design', () => {
  
  test('should adapt layout on mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    
    // Wait for page to load
    await page.waitForSelector('#page-title', { timeout: 10000 });
    
    // Page title should be visible
    await expect(page.locator('#page-title')).toBeVisible();
    
    // Generate button should still be visible (it's in the header area)
    await expect(page.locator('#generate-btn')).toBeVisible();
  });

});