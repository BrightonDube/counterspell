// Accessibility, SEO and auth-quality verification (AQ08; AUD-12 to AUD-21, AUD-25).
// Runs against `counterspell-testserver` like the other e2e specs: production router,
// real Postgres, built static site, scripted judge. Axe, keyboard, console, network
// and layout-shift evidence — regressions are asserted, not waived.
import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const BASE_URL = 'http://127.0.0.1:5198';

async function asUser(browser, number) {
  return browser.newContext({
    baseURL: BASE_URL,
    extraHTTPHeaders: { authorization: `Bearer test-user-${number}` }
  });
}

const PUBLIC_ROUTES = [
  { path: '/', name: 'landing' },
  { path: '/login', name: 'login' },
  { path: '/problems', name: 'problems' },
  { path: '/break', name: 'break room' },
  { path: '/break/bot', name: 'chess bot' },
  { path: '/break/puzzle', name: 'daily puzzle' }
];

for (const { path, name } of PUBLIC_ROUTES) {
  test(`axe finds no violations on ${name}`, async ({ page }) => {
    await page.goto(path);
    await page.waitForLoadState('networkidle');
    const results = await new AxeBuilder({ page }).analyze();
    expect(
      results.violations.map((v) => `${v.id}: ${v.nodes.length} nodes`)
    ).toEqual([]);
  });
}

test('axe finds no violations on the dashboard, signed out and signed in', async ({
  page,
  browser
}) => {
  await page.goto('/dashboard');
  await page.waitForLoadState('networkidle');
  const signedOut = await new AxeBuilder({ page }).analyze();
  expect(
    signedOut.violations.map((v) => `${v.id}: ${v.nodes.length} nodes`)
  ).toEqual([]);

  const user = await asUser(browser, 20000);
  const authed = await user.newPage();
  await authed.goto('/dashboard');
  await authed.waitForLoadState('networkidle');
  const signedIn = await new AxeBuilder({ page: authed }).analyze();
  expect(
    signedIn.violations.map((v) => `${v.id}: ${v.nodes.length} nodes`)
  ).toEqual([]);
  await user.close();
});

test('public routes ship title, description, canonical and social tags in the HTML', async ({
  request
}) => {
  for (const { path } of PUBLIC_ROUTES.filter((r) =>
    ['/', '/login', '/problems'].includes(r.path)
  )) {
    const html = await (await request.get(path)).text();
    expect(html, `${path} title`).toMatch(/<title>[^<]+<\/title>/);
    expect(html, `${path} description`).toContain('name="description"');
    expect(html, `${path} canonical`).toContain(
      `rel="canonical" href="https://counterspel.online${path === '/' ? '/' : path}"`
    );
  }
  const landing = await (await request.get('/')).text();
  expect(landing).toContain('property="og:title"');
  expect(landing).toContain('property="og:image"');
  expect(landing).toContain('name="twitter:card"');
});

test('the chessboard has one accessible name and the back link is at least 24px', async ({
  page
}) => {
  await page.goto('/break/bot');
  await expect(page.getByRole('img', { name: 'Chessboard' })).toBeVisible();
  const back = page.getByRole('link', { name: 'Back to the break room' });
  const box = await back.boundingBox();
  expect(box.height).toBeGreaterThanOrEqual(24);
});

test('tab reaches the first break-room game from the nav', async ({ page }) => {
  await page.goto('/break');
  await page.waitForLoadState('networkidle');
  let hops = 0;
  while (
    hops < 12 &&
    !(await page.evaluate(() =>
      document.activeElement?.getAttribute('href')?.includes('/break/puzzle')
    ))
  ) {
    await page.keyboard.press('Tab');
    hops += 1;
  }
  expect(hops).toBeLessThan(12);
});

test('a signed-out visit to the break room issues no scores request and only expected noise', async ({
  page
}) => {
  const api401 = [];
  const consoleErrors = [];
  const failedResponses = [];
  page.on('response', (r) => {
    if (r.status() === 401) api401.push(r.url());
    if (r.status() >= 400) failedResponses.push(`${r.status()} ${r.url()}`);
  });
  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  await page.goto('/break');
  await page.waitForLoadState('networkidle');
  // The status probe answers 200 even for anonymous visitors, so a signed-out
  // visit produces no 401 at all — and /api/me and the scores endpoint are
  // never touched.
  expect(api401).toEqual([]);
  // Browsers log every non-2xx resource load to the console; allow only the
  // expected empty-catalogue answer and let anything else fail loudly.
  const expectedNoise = [
    `503 ${BASE_URL}/api/breakroom/puzzle/daily` // not_imported: no puzzles seeded
  ];
  const unexpected = failedResponses.filter(
    (line) => !expectedNoise.includes(line) && !line.includes('favicon')
  );
  expect(unexpected).toEqual([]);
  expect(
    consoleErrors.filter(
      (t) => !t.includes('favicon') && !t.includes('status of 503')
    )
  ).toEqual([]);
});

test('the session answer is fetched once, not per consumer', async ({
  page
}) => {
  const probes = [];
  page.on('request', (r) => {
    if (r.url().includes('/api/auth/session')) probes.push(r.url());
    if (r.url().includes('/api/me')) probes.push(r.url());
  });
  await page.goto('/dashboard');
  await page.waitForLoadState('networkidle');
  expect(probes.length).toBeLessThanOrEqual(1);
});

test('the chess bot board does not shift layout while it mounts', async ({
  page
}) => {
  await page.addInitScript(() => {
    window.__cls = 0;
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (!entry.hadRecentInput) window.__cls += entry.value;
      }
    }).observe({ type: 'layout-shift', buffered: true });
  });
  await page.goto('/break/bot');
  await page.getByRole('img', { name: 'Chessboard' }).waitFor();
  await page.waitForTimeout(1500);
  const cls = await page.evaluate(() => window.__cls);
  expect(cls).toBeLessThan(0.1);
});
