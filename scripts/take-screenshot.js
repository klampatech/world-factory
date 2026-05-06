#!/usr/bin/env node
/**
 * take-screenshot.js - World Factory screenshot utility
 */
const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const BASE_URL = process.env.WORLD_FACTORY_URL || 'http://localhost:8765';
const OUTPUT_DIR = process.env.SCREENSHOT_DIR || path.join(__dirname, '../screenshots');
const OUTPUT = process.argv[2] || path.join(OUTPUT_DIR, 'hex-alignment-' + Date.now() + '.png');

if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

async function main() {
  const browser = await chromium.launch({
    executablePath: '/usr/bin/chromium',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
  });
  const page = await browser.newPage();
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
  await page.waitForTimeout(2000);
  await page.screenshot({ path: OUTPUT, fullPage: false });
  await browser.close();
  console.log(OUTPUT);
}

main().catch(err => {
  console.error('Screenshot failed: ' + err.message);
  process.exit(1);
});
