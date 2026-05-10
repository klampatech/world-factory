const { chromium } = require('@playwright/test');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  // Track ALL responses
  const responses = [];
  page.on('response', async (response) => {
    responses.push({
      url: response.url(),
      status: response.status(),
      contentType: response.headers()['content-type']
    });
  });

  // Create world
  const createResp = await fetch('http://localhost:8080/api/v1/worlds', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name: 'Debug', config: { width: 32, height: 32 } })
  });
  const newWorld = await createResp.json();
  const worldId = newWorld.data?.id || newWorld.id;
  console.log('World:', worldId);

  // Wait for ready
  for (let i = 0; i < 30; i++) {
    await new Promise(r => setTimeout(r, 2000));
    const s = await fetch(`http://localhost:8080/api/v1/worlds/${worldId}`);
    const d = await s.json();
    if (d.data?.status === 'ready') break;
  }

  await page.goto(`http://localhost:8787/worlds/${worldId}`);
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);

  console.log('\n=== All Network Responses (filtered) ===');
  responses
    .filter(r => r.url.includes('localhost:8787'))
    .forEach(r => console.log(`${r.status} ${r.contentType} ${r.url}`));

  // Check what's in the page head
  const headContent = await page.evaluate(() => {
    const scripts = Array.from(document.querySelectorAll('script'));
    return scripts.map(s => ({
      src: s.src || '(inline)',
      loaded: s.readyState || 'unknown'
    }));
  });
  console.log('\n=== Scripts in DOM ===');
  console.log(JSON.stringify(headContent, null, 2));

  await browser.close();
})();
