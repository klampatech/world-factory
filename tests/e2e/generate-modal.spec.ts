import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Generate World Modal
 * Tests the form submission and validation
 */

test.describe('Generate World Modal', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('modal should be hidden initially', async ({ page }) => {
    const modal = page.locator('#generate-modal');
    await expect(modal).not.toHaveClass(/active/);
  });

  test('modal should open from empty state button', async ({ page }) => {
    // The empty-generate-btn is hidden when there are demo worlds
    // But should work when empty
    const emptyBtn = page.locator('#empty-generate-btn');
    if (await emptyBtn.isVisible()) {
      await emptyBtn.click();
      await expect(page.locator('#generate-modal')).toHaveClass(/active/);
    }
  });

  test('world name input should accept text', async ({ page }) => {
    await page.click('#generate-btn');
    
    const nameInput = page.locator('#world-name-input');
    await nameInput.fill('My Test World');
    
    await expect(nameInput).toHaveValue('My Test World');
  });

  test('seed input should accept numbers and text', async ({ page }) => {
    await page.click('#generate-btn');
    
    const seedInput = page.locator('#world-seed-input');
    await seedInput.fill('12345');
    
    await expect(seedInput).toHaveValue('12345');
  });

  test('width slider should have range 16-128', async ({ page }) => {
    await page.click('#generate-btn');
    
    const widthSlider = page.locator('#width-slider');
    const min = await widthSlider.getAttribute('min');
    const max = await widthSlider.getAttribute('max');
    
    expect(min).toBe('16');
    expect(max).toBe('128');
  });

  test('height slider should have range 16-128', async ({ page }) => {
    await page.click('#generate-btn');
    
    const heightSlider = page.locator('#height-slider');
    const min = await heightSlider.getAttribute('min');
    const max = await heightSlider.getAttribute('max');
    
    expect(min).toBe('16');
    expect(max).toBe('128');
  });

  test('years slider should have range 100-10000', async ({ page }) => {
    await page.click('#generate-btn');
    
    const yearsSlider = page.locator('#years-slider');
    const min = await yearsSlider.getAttribute('min');
    const max = await yearsSlider.getAttribute('max');
    
    expect(min).toBe('100');
    expect(max).toBe('10000');
  });

  test('resource richness should have all options', async ({ page }) => {
    await page.click('#generate-btn');
    
    const select = page.locator('#resource-richness');
    const options = await select.locator('option').allTextContents();
    
    expect(options).toContain('Scarce');
    expect(options).toContain('Low');
    expect(options).toContain('Medium');
    expect(options).toContain('High');
    expect(options).toContain('Abundant');
  });

  test('disaster frequency should have all options', async ({ page }) => {
    await page.click('#generate-btn');
    
    const select = page.locator('#disaster-freq');
    const options = await select.locator('option').allTextContents();
    
    expect(options).toContain('None');
    expect(options).toContain('Rare');
    expect(options).toContain('Low');
    expect(options).toContain('Medium');
    expect(options).toContain('High');
    expect(options).toContain('Extreme');
  });

  test('create button should be visible and enabled', async ({ page }) => {
    await page.click('#generate-btn');
    
    const createBtn = page.locator('#modal-create');
    await expect(createBtn).toBeVisible();
    await expect(createBtn).toBeEnabled();
  });

  test('cancel button should close modal without action', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Fill in some data
    await page.fill('#world-name-input', 'Test World');
    
    // Click cancel
    await page.click('#modal-cancel');
    
    // Modal should be closed
    await expect(page.locator('#generate-modal')).not.toHaveClass(/active/);
  });

  test('escape key should close modal', async ({ page }) => {
    await page.click('#generate-btn');
    
    await page.keyboard.press('Escape');
    
    await expect(page.locator('#generate-modal')).not.toHaveClass(/active/);
  });

  test('species checkboxes should be toggleable', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Toggle human
    await page.check('input[value="human"]');
    await expect(page.locator('input[value="human"]')).toBeChecked();
    
    // Uncheck
    await page.uncheck('input[value="human"]');
    await expect(page.locator('input[value="human"]')).not.toBeChecked();
  });

  test('slider updates should reflect in all display elements', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Change width
    const widthSlider = page.locator('#width-slider');
    await widthSlider.fill('100');
    
    // All displays should show 100
    await expect(page.locator('#width-display')).toContainText('100');
    await expect(page.locator('#width-value')).toContainText('100');
  });

  test('changing sliders should update value display', async ({ page }) => {
    await page.click('#generate-btn');
    
    // Change height
    const heightSlider = page.locator('#height-slider');
    await heightSlider.fill('80');
    
    await expect(page.locator('#height-display')).toContainText('80');
    await expect(page.locator('#height-value')).toContainText('80');
    
    // Change years
    const yearsSlider = page.locator('#years-slider');
    await yearsSlider.fill('5000');
    
    await expect(page.locator('#years-display')).toContainText('5000');
    await expect(page.locator('#years-value')).toContainText('5000');
  });

});