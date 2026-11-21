// Product-surface flow over the real stack: login screen, auth redirects, catalogue,
// dashboard and the break-room chess bot. The exercise/judge pipeline itself is covered
// by real-api-check.mjs; this spec fills the UI legs nothing else exercises.
// Needs the same stack bring-up as the rest of this suite (see README.md).
import {
  BASE,
  createReporter,
  launchBrowser,
  signIn
} from './support.mjs';

const { check, finish } = createReporter();
const browser = await launchBrowser();
const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
const page = await context.newPage();
page.on('console', (m) => {
  // Signed-out auth checks legitimately log 401/403 resource errors; only flag real errors.
  if (m.type() === 'error' && !/status of (401|403|404)/.test(m.text())) {
    check(`console error: ${m.text()}`, false);
  }
});

// --- Signed-out login screen ---
await page.goto(`${BASE}/login`, { waitUntil: 'networkidle' });
const config = await (await page.request.get(`${BASE}/api/auth/config`)).json();
const github = page.getByRole('link', { name: /Continue with GitHub/i });
const google = page.getByRole('link', { name: /Continue with Google/i });
check('login: renders on the world', await page.locator('main, [class*=panel], form, a').first().isVisible());
if (config.providers?.github) {
  const href = await github.getAttribute('href');
  check('login: GitHub start carries dashboard return', /\/api\/auth\/oauth\/github\/start\?return_to=%2Fdashboard/.test(href ?? ''));
}
if (config.providers?.google) {
  const href = await google.getAttribute('href');
  check('login: Google start carries dashboard return', /\/api\/auth\/oauth\/google\/start\?return_to=%2Fdashboard/.test(href ?? ''));
}
check('login: no email/password inputs', (await page.locator('input[type=email], input[type=password]').count()) === 0);

// --- Signed-out dashboard is gated ---
await page.goto(`${BASE}/dashboard`, { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
const body = await page.textContent('body');
check('dashboard: signed-out visitor sees the auth gate', /sign in|log in|continue with/i.test(body ?? ''));

// --- Sign in (minted session, same shape OAuth verify writes) ---
const signed = await signIn(context, page, 'uiflow');
check('auth: minted session identifies a user', signed);

// --- Signed-in / must redirect to /dashboard ---
await page.goto(`${BASE}/`, { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
check('landing: signed-in user lands on /dashboard', page.url().startsWith(`${BASE}/dashboard`));

// --- Signed-in /login must also redirect ---
await page.goto(`${BASE}/login`, { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
check('login: signed-in user is redirected to /dashboard', page.url().startsWith(`${BASE}/dashboard`));

// --- Dashboard renders real data ---
const dash = await page.textContent('main');
check('dashboard: ready state renders', /solved|streak|suggest|run|problem/i.test(dash ?? ''));

// --- Problems catalogue is reachable and linked ---
await page.click('a[href="/problems"]');
await page.waitForLoadState('networkidle');
const catalogueLinks = await page.locator('a[href^="/problems/"]').count();
check('problems: catalogue lists problem links', catalogueLinks > 0, `links=${catalogueLinks}`);

// --- Break room: chess bot answers a real move ---
await page.goto(`${BASE}/break/bot`, { waitUntil: 'networkidle' });
await page.getByRole('button', { name: 'Play White' }).click();
await page.waitForSelector('.break-board', { timeout: 15000 });
await page.waitForTimeout(800);
const board = await page.locator('.break-board').boundingBox();
check('bot: board mounted', !!board);
if (board) {
  const sq = board.width / 8;
  const click = async (file, rank) => {
    await page.mouse.click(
      board.x + (file.charCodeAt(0) - 97 + 0.5) * sq,
      board.y + (8 - rank + 0.5) * sq
    );
  };
  const before = await page.locator('output').textContent();
  await click('e', 2);
  await page.waitForTimeout(300);
  await click('e', 4);
  const changed = await page
    .waitForFunction((prev) => document.querySelector('output')?.textContent !== prev, before, { timeout: 15000 })
    .then(() => true)
    .catch(() => false);
  check('bot: e2-e4 accepted and position updated', changed);
  await page.waitForTimeout(1500);
  const status = await page.locator('output').textContent();
  check('bot: bot replied (move 2 on the clock)', / 2$/.test((status ?? '').trim()), (status ?? '').slice(-60));
}

await browser.close();
finish();
