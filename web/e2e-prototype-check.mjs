// ide-workspace T15: browser visual check of the fixture-only prototype route.
// The prototype must never be mistaken for a real judge result, so its banner
// has to be visible on first paint at desktop and phone widths.
// Usage: npm run build && node e2e-prototype-check.mjs
//        (E2E_EXECUTABLE / E2E_CHANNEL as in e2e-shell-check.mjs; E2E_BASE to test another server)
// Without E2E_BASE it serves build/ itself, falling back to 200.html for
// unknown routes. (The Rust server only falls back for declared routes in
// routes.json and answers 404 otherwise; this dev server skips that check.)
// Prints PASS/FAIL per check and exits non-zero on any failure.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
import { chromium } from 'playwright-core';

const TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.webp': 'image/webp',
  '.png': 'image/png',
  '.woff2': 'font/woff2'
};

/** Serves the static build on an ephemeral loopback address; returns its base URL. */
async function serveBuild() {
  const root = join(import.meta.dirname, 'build');
  const server = createServer(async (req, res) => {
    const path = normalize(
      decodeURIComponent(new URL(req.url, 'http://x').pathname)
    );
    const candidates = [
      join(root, path),
      join(root, path, 'index.html'),
      `${join(root, path)}.html`
    ];
    for (const file of candidates) {
      if (!file.startsWith(root)) break;
      try {
        const body = await readFile(file);
        res.writeHead(200, {
          'content-type': TYPES[extname(file)] ?? 'application/octet-stream'
        });
        res.end(body);
        return;
      } catch {
        // try the next candidate
      }
    }
    res.writeHead(200, { 'content-type': 'text/html' });
    res.end(await readFile(join(root, '200.html')));
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  return { server, base: `http://127.0.0.1:${server.address().port}` };
}

const served = process.env.E2E_BASE ? null : await serveBuild();
const BASE = process.env.E2E_BASE ?? served.base;
const BANNER = 'Prototype fixture — not a real run';
const results = [];
const check = (name, ok, detail = '') =>
  results.push(
    `${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`
  );

const browser = await chromium.launch({
  headless: true,
  ...(process.env.E2E_CHANNEL ? { channel: process.env.E2E_CHANNEL } : {}),
  ...(process.env.E2E_EXECUTABLE
    ? { executablePath: process.env.E2E_EXECUTABLE }
    : {})
});

for (const [label, width, height] of [
  ['desktop', 1440, 900],
  ['phone', 390, 844]
]) {
  const page = await browser.newPage({ viewport: { width, height } });
  await page.goto(`${BASE}/prototype/ide-workspace`, {
    waitUntil: 'networkidle'
  });
  const banner = page.getByText(BANNER, { exact: false }).first();
  const visible = await banner.isVisible().catch(() => false);
  check(`${label}: banner visible`, visible);
  const box = visible ? await banner.boundingBox() : null;
  check(
    `${label}: banner on first screen`,
    box !== null && box.y >= 0 && box.y + box.height <= height,
    box ? `y=${Math.round(box.y)}` : 'no box'
  );
  await page.screenshot({
    path: `/tmp/prototype-${label}.png`,
    fullPage: false
  });
  await page.close();
}

await browser.close();
served?.server.close();
console.log(results.join('\n'));
process.exit(results.some((line) => line.startsWith('FAIL')) ? 1 : 0);
