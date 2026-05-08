import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Hash-Based Routing
 * Tests URL-based navigation in the SPA
 */

test.describe('Hash-Based Routing', () => {
  
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should load World Selector at #/', async ({ page }) => {
    // Set hash to root
    await page.goto('/#');
    
    // World Selector view should be visible
    const worldSelector = page.locator('#view-world-selector');
    await expect(worldSelector).toBeVisible();
    
    // Tabs should be hidden
    const tabsContainer = page.locator('#tabs-container');
    await expect(tabsContainer).toBeHidden();
  });

  test('should navigate to world detail via view button', async ({ page }) => {
    // Click View Map button on first world card
    const viewMapBtn = page.locator('.view-btn').first();
    await viewMapBtn.click();
    
    // Wait for URL to change
    await page.waitForFunction(() => window.location.hash.includes('worlds'));
    
    // Check URL contains world detail route
    expect(page.url()).toMatch(/#\/worlds\//);
    
    // World detail view should be visible
    const worldDetail = page.locator('#view-world-detail');
    await expect(worldDetail).toBeVisible();
  });

  test('should navigate with specific tab', async ({ page }) => {
    // Navigate directly to a world with specific tab
    await page.goto('/#/worlds/test-world-id/timeline');
    
    // Wait for navigation
    await page.waitForTimeout(500);
    
    // Tabs should be visible
    const tabsContainer = page.locator('#tabs-container');
    await expect(tabsContainer).toBeVisible();
    
    // Timeline tab should be active
    const timelineTab = page.locator('.tab-button[data-tab="timeline"]');
    await expect(timelineTab).toHaveClass(/active/);
  });

  test('should support browser back/forward navigation', async ({ page }) => {
    // First navigate to a world
    await page.goto('/#/worlds/test-world-id');
    await page.waitForTimeout(500);
    
    // Click Timeline tab
    await page.click('.tab-button[data-tab="timeline"]');
    await page.waitForTimeout(300);
    
    // Go back
    await page.goBack();
    await page.waitForTimeout(500);
    
    // Should be back at root
    const worldSelector = page.locator('#view-world-selector');
    await expect(worldSelector).toBeVisible();
  });

  test('should update hash when switching tabs', async ({ page }) => {
    await page.goto('/#/worlds/test-world-id');
    await page.waitForTimeout(500);
    
    // Click Dashboard tab
    await page.click('.tab-button[data-tab="dashboard"]');
    await page.waitForTimeout(300);
    
    // Hash should be updated
    expect(page.url()).toContain('#/worlds/test-world-id/dashboard');
  });

  test('should show back link on world detail page', async ({ page }) => {
    await page.goto('/#/worlds/test-world-id');
    await page.waitForTimeout(500);
    
    // Back link should be visible
    const backLink = page.locator('#back-link');
    await expect(backLink).toBeVisible();
  });

  test('should return to world selector from back link', async ({ page }) => {
    await page.goto('/#/worlds/test-world-id');
    await page.waitForTimeout(500);
    
    // Click back link
    await page.click('#back-link');
    await page.waitForTimeout(500);
    
    // Hash should be root
    expect(page.url()).toMatch(/#\/?$/);
    
    // World selector should be visible
    const worldSelector = page.locator('#view-world-selector');
    await expect(worldSelector).toBeVisible();
  });

  test('should handle deep link to world overview', async ({ page }) => {
    await page.goto('/#/worlds/abc123');
    await page.waitForTimeout(500);
    
    // Should show world detail
    const worldDetail = page.locator('#view-world-detail');
    await expect(worldDetail).toBeVisible();
    
    // Should default to overview tab
    const overviewTab = page.locator('.tab-button[data-tab="overview"]');
    await expect(overviewTab).toHaveClass(/active/);
  });

  test('should handle deep link to world map', async ({ page }) => {
    await page.goto('/#/worlds/abc123/map');
    await page.waitForTimeout(500);
    
    // Map tab should be active
    const mapTab = page.locator('.tab-button[data-tab="map"]');
    await expect(mapTab).toHaveClass(/active/);
  });

  test('should handle deep link to world dashboard', async ({ page }) => {
    await page.goto('/#/worlds/abc123/dashboard');
    await page.waitForTimeout(500);
    
    // Dashboard tab should be active
    const dashboardTab = page.locator('.tab-button[data-tab="dashboard"]');
    await expect(dashboardTab).toHaveClass(/active/);
  });
});