/**
 * Preview server for World Factory web frontend
 * Serves the dist/ directory on a local HTTP server
 * Includes API proxy to backend server
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const distDir = path.join(__dirname, '..', 'dist');
const PORT = process.env.PORT || process.env.FRONTEND_PORT || 8765;
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

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

// Helper to proxy HTTP requests to backend
function proxyRequest(req, res, targetUrl) {
  const target = new URL(targetUrl);
  const options = {
    hostname: target.hostname,
    port: target.port || 80,
    path: req.url,
    method: req.method,
    headers: {
      ...req.headers,
      'X-Forwarded-Proto': 'http',
      'X-Forwarded-Host': `localhost:${PORT}`
    }
  };

  const proxyReq = http.request(options, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });

  proxyReq.on('error', (err) => {
    console.error(`Proxy error for ${req.url}: ${err.message}`);
    res.writeHead(502, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Backend unavailable', message: err.message }));
  });

  req.pipe(proxyReq);
}

const server = http.createServer((req, res) => {
  // Proxy API requests to backend
  if (req.url.startsWith('/api/') || req.url === '/health') {
    const targetUrl = `${BACKEND_URL}${req.url}`;
    console.log(`Proxying ${req.method} ${req.url} -> ${targetUrl}`);
    return proxyRequest(req, res, targetUrl);
  }

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
