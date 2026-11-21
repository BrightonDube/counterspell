import { chromium } from 'playwright';

const BASE = 'http://127.0.0.1:8090';
const browser = await chromium.launch({
  headless: true,
  executablePath: '/home/brighton/.cache/ms-playwright/chromium_headless_shell-1255/chrome-headless-shell-linux64/chrome-headless-shell',
});
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });

// signed-out, from the landing CTA
await page.goto(BASE, { waitUntil: 'networkidle' });
await page.click('text=Start practising');
await page.waitForLoadState('networkidle');
console.log('after CTA:', page.url());
await page.screenshot({ path: '/tmp/e2e-shots/so-1-catalogue.png' });

await page.click('a[href="/problems/add-two"]');
await page.waitForSelector('.monaco-editor', { timeout: 45000 });
await page.waitForTimeout(800);
await page.screenshot({ path: '/tmp/e2e-shots/so-2-workspace.png' });
console.log('workspace loaded, url:', page.url());

await page.getByRole('button', { name: 'Run samples' }).click();
await page.waitForSelector('[role=alert]', { timeout: 20000 }).catch(() => console.log('no alert'));
await page.waitForTimeout(800);
const alert = await page.locator('[role=alert]').textContent().catch(() => null);
console.log('alert text:', alert);
await page.screenshot({ path: '/tmp/e2e-shots/so-3-after-run.png' });

await browser.close();
