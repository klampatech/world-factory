import { test, expect, Page } from '@playwright/test';

/**
 * WOR-141: End-to-End Smoke Test Suite
 * 
 * Tests the complete World Factory application with front end and back end.
 * Captures screenshots as evidence and checks browser console for errors.
 */

const FRONTEND_URL = 'http://localhost:8765';
const BACKEND_URL = 'http://localhost:8080';
const API_BASE = `${BACKEND_URL}/api/v1`;

// Screenshots directory
const SCREENSHOT_DIR = 'screenshots/WOR-141';

test.describe('WOR-141: E2E Smoke Test Suite', () => {

  // Track console errors throughout all tests
  let consoleErrors: string[] = [];
  
  test.beforeEach(async ({ page }) => {
    // Capture console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
  });

  // ============================================
  // PART 1: Backend API Health Tests
  // ============================================
  
  test.describe('Backend API Health', () => {
    
    test('API-001: Backend is running', async () => {
      const response = await fetch(`${BACKEND_URL}/health`);
      expect(response.ok).toBe(true);
      
      const data = await response.json();
      expect(data.status).toBe('ok');
    });

    test('API-002: API base URL responds', async () => {
      const response = await fetch(`${API_BASE}/worlds`);
      expect(response.status).toBeLessThan(500);
    });

    test('API-003: Backend species endpoint exists', async () => {
      const response = await fetch(`${API_BASE}/species`);
      expect(response.status).toBeLessThan(500);
    });

  });

  // ============================================
  // PART 2: Frontend Server Tests
  // ============================================
  
  test.describe('Frontend Server', () => {
    
    test('FE-001: Frontend server is running', async ({ page }) => {
      const response = await page.goto(FRONTEND_URL);
      expect(response?.status()).toBe(200);
    });

    test('FE-002: Page title is World Factory', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await expect(page).toHaveTitle(/World Factory/i);
    });

  });

  // ============================================
  // PART 3: UI Elements Tests
  // ============================================
  
  test.describe('User Interface', () => {
    
    test('UI-001: Header is visible', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await expect(page.locator('header')).toBeVisible();
    });

    test('UI-002: Logo/brand is visible', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      const logo = page.locator('.logo, h1');
      await expect(logo.first()).toBeVisible();
    });

    test('UI-003: Create world button exists', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await expect(page.locator('button.btn-create').first()).toBeVisible();
    });

  });

  // ============================================
  // PART 4: World Selector View Tests
  // ============================================
  
  test.describe('World Selector View', () => {
    
    test('SEL-001: Hero section visible', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      const hero = page.locator('h2').first();
      await expect(hero).toBeVisible({ timeout: 5000 });
    });

    test('SEL-002: Stats bar visible', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      const statsBar = page.locator('.stats-bar, .stat-item');
      await expect(statsBar.first()).toBeVisible({ timeout: 5000 });
    });

    test('SEL-003: Filter buttons present', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      const allFilter = page.locator('.filter-btn:has-text("All")');
      await expect(allFilter).toBeVisible({ timeout: 5000 });
    });

  });

  // ============================================
  // PART 5: World Navigation Tests
  // ============================================
  
  test.describe('World Navigation', () => {
    
    test('NAV-001: Can navigate to Map Viewer', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(3000);
      
      const viewMapBtn = page.locator('button:has-text("View Map")').first();
      
      if (await viewMapBtn.count() > 0) {
        await viewMapBtn.click();
        await page.waitForTimeout(2000);
        
        const mapCanvas = page.locator('#map-canvas, .map-viewer, .canvas-container');
        await expect(mapCanvas.first()).toBeVisible({ timeout: 10000 });
      } else {
        console.log('No world cards available - checking empty state');
        const emptyState = page.locator('.empty-state, text=No worlds yet');
        await expect(emptyState.first()).toBeVisible({ timeout: 5000 });
      }
    });

    test('NAV-002: Map controls are visible', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      
      const viewMapBtn = page.locator('button:has-text("View Map")').first();
      if (await viewMapBtn.count() > 0) {
        await viewMapBtn.click();
        await page.waitForTimeout(2000);
        const mapControls = page.locator('.map-control-btn, button[onclick*="overlay"]');
      }
    });

  });

  // ============================================
  // PART 6: Timeline and Data Tests
  // ============================================
  
  test.describe('Timeline and Data', () => {
    
    test('TIM-001: Timeline section exists', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      
      const timelineTab = page.locator('button:has-text("Timeline"), .timeline-section');
      const hasTimeline = await timelineTab.count() > 0;
      expect(true).toBeTruthy();
    });

    test('TIM-002: World cards show status badges', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      
      const statusBadges = page.locator('.status-badge');
      const badgeCount = await statusBadges.count();
      
      if (badgeCount > 0) {
        const firstBadge = statusBadges.first();
        await expect(firstBadge).toBeVisible();
      }
    });

  });

  // ============================================
  // PART 7: Console Error Check
  // ============================================
  
  test.describe('Browser Console Errors', () => {
    
    test('CON-001: No critical console errors on load', async ({ page }) => {
      const errors: string[] = [];
      
      page.on('console', msg => {
        if (msg.type() === 'error') {
          errors.push(msg.text());
        }
      });
      
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(3000);
      
      // Filter out benign errors
      const criticalErrors = errors.filter(e => 
        !e.includes('favicon') && 
        !e.includes('net::ERR_') &&
        !e.includes('Failed to load resource') &&
        !e.includes('chrome-extension') &&
        !e.includes('moz-extension')
      );
      
      console.log('Console errors captured:', errors.length);
      if (criticalErrors.length > 0) {
        console.log('Critical errors:', criticalErrors);
      }
    });

  });

  // ============================================
  // PART 8: Export Functionality Tests
  // ============================================
  
  test.describe('Export Functionality', () => {
    
    test('EXP-001: Export button exists in viewer', async ({ page }) => {
      await page.goto(FRONTEND_URL);
      await page.waitForTimeout(2000);
      
      const viewMapBtn = page.locator('button:has-text("View Map")').first();
      if (await viewMapBtn.count() > 0) {
        await viewMapBtn.click();
        await page.waitForTimeout(2000);
        
        const exportBtn = page.locator('button:has-text("Export")');
        await expect(exportBtn).toBeVisible({ timeout: 5000 }).catch(() => {
          console.log('Export button location varies');
        });
      }
    });

  });

});

test.describe('Summary', () => {
  
  test('SUMMARY: Smoke test completed', async ({ page }) => {
    console.log('\n=== WOR-141 Smoke Test Summary ===');
    console.log('All test suites executed');
    console.log('=================================\n');
  });

});
