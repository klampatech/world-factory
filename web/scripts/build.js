/**
 * Build script for World Factory web frontend
 * Copies static files to dist/ directory for serving
 */

const fs = require('fs');
const path = require('path');

const srcDir = __dirname;
const webDir = path.dirname(srcDir);
const distDir = path.join(webDir, 'dist');

// Files to copy
const files = [
  'index.html',
  'world.html',
  'api-integration.js',
  'wor205-qa-test.html',
];

function copyFile(src, dest) {
  const destDir = path.dirname(dest);
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }
  fs.copyFileSync(src, dest);
  console.log(`  Copied: ${path.relative(webDir, src)}`);
}

console.log('Building World Factory web frontend...');

// Ensure dist directory exists
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// Copy files
console.log('Copying static files:');
files.forEach(file => {
  const src = path.join(webDir, file);
  if (fs.existsSync(src)) {
    copyFile(src, path.join(distDir, file));
  } else {
    console.warn(`  Warning: ${file} not found, skipping`);
  }
});

console.log('\nBuild complete! Output in dist/');
