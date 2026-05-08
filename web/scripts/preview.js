/**
 * Preview server for World Factory web frontend
 * Serves the dist/ directory on a local HTTP server
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const distDir = path.join(__dirname, '..', 'dist');
const PORT = process.env.PORT || 8765;

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
};

function serveFile(res, filePath) {
  const ext = path.extname(filePath).toLowerCase();
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';
  
  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('File not found');
      return;
    }
    res.writeHead(200, { 'Content-Type': contentType });
    res.end(data);
  });
}

const server = http.createServer((req, res) => {
  let url = req.url.split('?')[0];
  
  // Default to index.html
  if (url === '/') {
    url = '/index.html';
  }
  
  const filePath = path.join(distDir, url);
  
  // Security: prevent directory traversal
  if (!filePath.startsWith(distDir)) {
    res.writeHead(403, { 'Content-Type': 'text/plain' });
    res.end('Forbidden');
    return;
  }
  
  if (fs.existsSync(filePath) && fs.statSync(filePath).isFile()) {
    serveFile(res, filePath);
  } else {
    // Fallback to index.html for SPA routing (but serve world.html if directly requested)
    if (url.endsWith('world.html')) {
      const worldPath = path.join(distDir, 'world.html');
      if (fs.existsSync(worldPath)) {
        serveFile(res, worldPath);
        return;
      }
    }
    const indexPath = path.join(distDir, 'index.html');
    if (fs.existsSync(indexPath)) {
      serveFile(res, indexPath);
    } else {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('File not found. Run "npm run build" first.');
    }
  }
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`World Factory preview server running at http://localhost:${PORT}`);
  console.log(`Serving files from: ${distDir}`);
  console.log('Press Ctrl+C to stop');
});
