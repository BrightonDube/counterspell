// T09 responsive WorkspaceShell browser checks against the vite+stub harness.
// Real Chrome via channel (no downloaded browser). Prints PASS/FAIL per check.
// Usage: npx vite dev --port 5173 & node e2e-stub.mjs &
//        npm i --no-save playwright-core && node e2e-shell-check.mjs
import { chromium } from 'playwright-core';

const BASE = 'http://127.0.0.1:5199';
const results = [];
const check = (name, ok, detail = '') =>
  results.push(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`);

const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage();
page.on('console', (m) => {
  if (m.type() === 'error') results.push(`CONSOLE-ERROR ${m.text().slice(0, 160)}`);
});

const shell = {
  problem: 'aside[aria-label="Problem"], [role="complementary"][aria-label="Problem"]',
  results: '[role="region"][aria-label="Results"], section[aria-label="Results"]'
};

async function load(width, height) {
  await page.setViewportSize({ width, height });
  await page.goto(`${BASE}/problems/two-sum`, { waitUntil: 'networkidle' });
  await page.waitForSelector('button:has-text("Run samples")', { timeout: 15000 });
}

const overflow = () =>
  page.evaluate(() => ({
    scroll: document.documentElement.scrollWidth,
    client: document.documentElement.clientWidth,
    bodyScroll: document.body.scrollWidth
  }));

async function minTarget() {
  return page.evaluate(() => {
    const els = [...document.querySelectorAll('button, a[href], [role="tab"], select, input')].filter(
      (el) => el.offsetParent !== null
    );
    let min = Infinity;
    let name = '';
    for (const el of els) {
      const r = el.getBoundingClientRect();
      const h = Math.min(r.width, r.height);
      if (h > 0 && h < min) {
        min = h;
        name = `${el.tagName.toLowerCase()} "${(el.textContent || el.getAttribute('aria-label') || '').trim().slice(0, 30)}" ${Math.round(r.width)}x${Math.round(r.height)}`;
      }
    }
    return { min, name, count: els.length };
  });
}

// --- Desktop 1440x900 ---
await load(1440, 900);
check('desktop: page renders problem', await page.isVisible('text=Two Sum'));
check('desktop: Problem complementary landmark', await page.isVisible(shell.problem));
check('desktop: Results region landmark', await page.isVisible(shell.results));
const sideBy = await page.evaluate((sel) => {
  const p = document.querySelector(sel.problem);
  const r = document.querySelector(sel.results);
  if (!p || !r) return { ok: false };
  const pb = p.getBoundingClientRect();
  const rb = r.getBoundingClientRect();
  return { ok: rb.left > pb.left, pRight: pb.right, rLeft: rb.left, overlapY: !(pb.bottom < rb.top || rb.bottom < pb.top) };
}, shell);
check('desktop: problem + results side by side', sideBy.ok && sideBy.overlapY, JSON.stringify(sideBy));
let o = await overflow();
check('desktop: no horizontal overflow', o.scroll <= o.client + 1, `${o.scroll}px vs ${o.client}px`);
let t = await minTarget();
check('desktop: all interactive targets >=44px', t.min >= 44, `min ${Math.round(t.min)}px (${t.name}) of ${t.count}`);

// --- Collapse + focus return ---
const toggle = page.getByRole('button', { name: /hide problem/i }).first();
if (await toggle.count()) {
  await toggle.click();
  check('desktop: collapse hides problem column', !(await page.isVisible(shell.problem)));
  const focused = await page.evaluate(() => document.activeElement?.textContent?.trim() ?? '');
  check('desktop: focus lands on a visible control after collapse', focused.length > 0, `focus="${focused}"`);
  const show = page.getByRole('button', { name: /show problem/i }).first();
  await show.click();
  check('desktop: restore shows problem column', await page.isVisible(shell.problem));
} else {
  check('desktop: collapse control present', false, 'no hide-problem button found');
}

// --- Tablet 900x1024 (768-1199 band): drawers + status strip ---
await load(900, 1024);
o = await overflow();
check('tablet: no horizontal overflow', o.scroll <= o.client + 1, `${o.scroll}px vs ${o.client}px`);
t = await minTarget();
check('tablet: all interactive targets >=44px', t.min >= 44, `min ${Math.round(t.min)}px (${t.name}) of ${t.count}`);
check('tablet: editor stays visible', await page.isVisible('[aria-label="Editor"]'));
check('tablet: persistent status strip', await page.isVisible('[role="status"][aria-label="Run status"]'));
check('tablet: problem not inline', !(await page.isVisible(shell.problem)));
check('tablet: results not inline', !(await page.isVisible(shell.results)));
const pTrigger = page.getByRole('button', { name: 'Problem', exact: true });
const rTrigger = page.getByRole('button', { name: 'Results', exact: true });
check('tablet: drawer triggers present', (await pTrigger.count()) === 1 && (await rTrigger.count()) === 1);
await pTrigger.click();
const drawer = page.getByRole('dialog', { name: 'Problem' });
check('tablet: problem drawer opens', await drawer.isVisible());
const inDrawer = await page.evaluate(() => {
  const d = document.querySelector('#problem-drawer');
  return d && d.contains(document.activeElement);
});
check('tablet: focus moves into drawer', !!inDrawer);
await page.keyboard.press('Escape');
check('tablet: Escape closes drawer', !(await page.isVisible('#problem-drawer')));
const backOnTrigger = await page.evaluate(
  () => document.activeElement?.textContent?.trim() === 'Problem'
);
check('tablet: focus returns to trigger', backOnTrigger);
await rTrigger.click();
check('tablet: results drawer opens', await page.getByRole('dialog', { name: 'Results' }).isVisible());
await page.keyboard.press('Escape');

// --- Live breakpoint switching (no reload) ---
await page.setViewportSize({ width: 1440, height: 900 });
await page.waitForSelector(shell.problem, { timeout: 5000 });
check('resize: tablet→desktop swaps to columns live', await page.isVisible(shell.problem));
await page.setViewportSize({ width: 390, height: 844 });
await page.waitForSelector(shell.problem, { timeout: 5000 });
const order = await page.evaluate((sel) => {
  const p = document.querySelector(sel.problem);
  const e = document.querySelector('[aria-label="Editor"]');
  const r = document.querySelector(sel.results);
  return p && e && r
    ? p.compareDocumentPosition(e) & Node.DOCUMENT_POSITION_FOLLOWING &&
        e.compareDocumentPosition(r) & Node.DOCUMENT_POSITION_FOLLOWING
    : false;
}, shell);
check('resize: desktop→narrow stacks in reading order', !!order);

// --- Portrait 390x844 ---
await load(390, 844);
o = await overflow();
check('portrait: no horizontal overflow', o.scroll <= o.client + 1, `${o.scroll}px vs ${o.client}px`);
const stacked = await page.evaluate((sel) => {
  const r = document.querySelector(sel.results);
  if (!r) return { ok: false, why: 'no results' };
  const rb = r.getBoundingClientRect();
  return { ok: rb.left < 24, left: rb.left };
}, shell);
check('portrait: results stacked full-width', stacked.ok, JSON.stringify(stacked));
t = await minTarget();
check('portrait: all interactive targets >=44px', t.min >= 44, `min ${Math.round(t.min)}px (${t.name}) of ${t.count}`);

// --- 200% zoom (deviceScaleFactor=2 at 720 CSS px ≈ 200% of 1440 layout) ---
await load(720, 450);
o = await overflow();
check('zoom200: no core horizontal overflow', o.scroll <= o.client + 1, `${o.scroll}px vs ${o.client}px`);

// --- Route-change abort / no stale slug ---
await load(1440, 900);
await page.goto(`${BASE}/problems/count-pairs`, { waitUntil: 'networkidle' });
await page.waitForSelector('text=Count Pairs', { timeout: 15000 });
const stale = await page.locator('h1:has-text("Two Sum"), h2:has-text("Two Sum")').count();
check('route-change: new slug loads, no stale title', stale === 0 && (await page.isVisible('text=Count Pairs')), `stale h1/h2 count=${stale}`);
await page.goBack();
await page.waitForSelector('text=Two Sum', { timeout: 15000 });
check('route-change: back returns to previous slug', await page.isVisible('text=Two Sum'));

console.log(results.join('\n'));
await browser.close();
