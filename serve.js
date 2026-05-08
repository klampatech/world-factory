const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 9000;
const BACKEND_HOST = 'localhost';
const BACKEND_PORT = 8080;

const mimeTypes = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.mjs': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
};

// Proxy request to backend
function proxyToBackend(req, res) {
  const options = {
    hostname: BACKEND_HOST,
    port: BACKEND_PORT,
    path: req.url,
    method: req.method,
    headers: {
      ...req.headers,
      'host': `${BACKEND_HOST}:${BACKEND_PORT}`,
      'X-Forwarded-Host': `${BACKEND_HOST}:${BACKEND_PORT}`,
      'X-Forwarded-Proto': 'http',
    },
  };

  const proxyReq = http.request(options, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });

  proxyReq.on('error', (err) => {
    console.error(`Proxy error: ${err.message}`);
    res.writeHead(502, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      error: 'Backend unavailable',
      type: 'PROXY_ERROR',
      message: 'The API server is not responding. Please ensure the backend is running on port 8080.',
    }));
  });

  req.pipe(proxyReq);
}

const server = http.createServer((req, res) => {
  // Proxy API requests to backend
  if (req.url.startsWith('/api/')) {
    return proxyToBackend(req, res);
  }

  // Serve static files
  let filePath = path.join(__dirname, req.url === '/' ? '/web/index.html' : req.url);
  
  const ext = path.extname(filePath);
  const contentType = mimeTypes[ext] || 'text/plain';
  
  fs.readFile(filePath, (err, content) => {
    if (err) {
      if (err.code === 'ENOENT') {
        res.writeHead(404);
        res.end('File not found');
      } else {
        res.writeHead(500);
        res.end('Server error');
      }
    } else {
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content);
    }
  });
});

server.listen(PORT, () => {
  console.log(`Static server running at http://localhost:${PORT}/`);
  console.log(`Proxying /api/* requests to http://${BACKEND_HOST}:${BACKEND_PORT}/api/*`);
});
