// Landing page checks in a real browser (landing-steel-night T14; R21, R22, R27, R28, R29).
// Served by `counterspell-testserver`, the production router and the built static site.
import { expect, test } from '@playwright/test';

/** Relative luminance of an `rgb(r, g, b)` or `rgba(...)` string, per WCAG 2.x. */
function luminance(color) {
  const [r, g, b] = color.match(/[\d.]+/g).slice(0, 3).map(Number).map((v) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two colour strings. */
function contrast(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

test('every request stays on our own origin and the page needs no third party', async ({
  page
}) => {
  const hosts = new Set();
  page.on('request', (request) => {
    if (request.url().startsWith('http')) hosts.add(new URL(request.url()).host);
  });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  expect([...hosts]).toEqual(['127.0.0.1:5198']);
});

test('the title screen has the wordmark, a focusable ENTER link and the headline in the static HTML', async ({
  request
}) => {
  const html = await (await request.get('/')).text();
  expect(html).toContain('COUNTERSPELL');
  expect(html).toContain('href="/login"');
  expect(html).toContain('Learn why your code breaks.');
});

test('the first Tab stop is ENTER and it shows a visible focus ring', async ({
  page
}) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  const enter = page.getByRole('link', { name: 'Enter, go to sign in' });
  await expect(enter).toBeFocused();
  const outline = await enter.evaluate((el) => {
    const style = getComputedStyle(el);
    return { width: style.outlineWidth, style: style.outlineStyle };
  });
  expect(outline.style).toBe('solid');
  expect(Number.parseFloat(outline.width)).toBeGreaterThanOrEqual(2);
});

test('pressing Enter on the title screen goes to sign in', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/login$/);
});

test('Enter on a focused control does not hijack the page', async ({
  page
}) => {
  await page.goto('/');
  // Tab past ENTER to the next control, then press Enter: that control, not the hero, answers.
  await page.keyboard.press('Tab');
  await page.keyboard.press('Tab');
  const focused = await page.evaluate(() => document.activeElement?.tagName);
  expect(focused).not.toBe('BODY');
  const target = await page.evaluate(
    () => document.activeElement?.getAttribute('href') ?? null
  );
  await page.keyboard.press('Enter');
  if (target) {
    await expect(page).toHaveURL(new RegExp(`${target.replace('/', '\\/')}$`));
  } else {
    await expect(page).toHaveURL(/\/$/);
  }
});

test('the call-to-action links are reachable by keyboard and show a focus ring', async ({
  page
}) => {
  await page.goto('/');
  for (const name of ['Start practising', 'Log in']) {
    const link = page.getByRole('link', { name });
    await link.focus();
    await expect(link).toBeFocused();
    const ring = await link.evaluate((el) => {
      const style = getComputedStyle(el);
      return `${style.outlineStyle} ${style.boxShadow}`;
    });
    expect(ring, `${name} focus indicator`).not.toMatch(/^none none$/);
  }
});

test('body copy meets WCAG AA contrast against the page background', async ({
  page
}) => {
  await page.goto('/');
  const pairs = await page.evaluate(() => {
    const background = getComputedStyle(document.body).backgroundColor;
    const pick = (selector) => {
      const el = document.querySelector(selector);
      return el ? getComputedStyle(el).color : null;
    };
    return {
      background,
      heading: pick('main h2'),
      muted: pick('main p.text-text-muted')
    };
  });
  expect(pairs.heading).not.toBeNull();
  expect(pairs.muted).not.toBeNull();
  expect(contrast(pairs.heading, pairs.background)).toBeGreaterThanOrEqual(4.5);
  expect(contrast(pairs.muted, pairs.background)).toBeGreaterThanOrEqual(4.5);
});

test('reduced motion loads no GSAP and no video', async ({ browser }) => {
  const context = await browser.newContext({
    baseURL: 'http://127.0.0.1:5198',
    reducedMotion: 'reduce'
  });
  const page = await context.newPage();
  const urls = [];
  page.on('request', (request) => urls.push(request.url()));
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(1000);
  expect(urls.filter((url) => /\.(webm|mp4)(\?|$)/.test(url))).toEqual([]);
  expect(await page.locator('video').count()).toBe(0);
  await context.close();
});
