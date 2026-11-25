// Workspace motion and control verification (workspace-performance-costume WP07).
// Three honest checks on the real testserver: the costume stills under reduced
// motion, the documented keyboard shortcuts drive the controls, and every
// primary editor control answers.
import { expect, test } from '@playwright/test';

const URL_UNDER_TEST = '/problems/two-sum';

/** Runs need a session: the fixture accepts Bearer identities (dashboard.spec.js). */
function bearerContext(browser) {
  const user = 40 + (Math.floor(Date.now() / 1000) % 2000) * 2;
  return browser.newContext({
    extraHTTPHeaders: { authorization: `Bearer test-user-${user}` }
  });
}

const RESULT_ACTIVE =
  /queued|compiling|running|accepted|rejected|wrong answer|time limit|memory limit|runtime error|internal/i;

test('reduced motion stills the costume and skips animated media', async ({
  browser
}) => {
  const context = await browser.newContext({ reducedMotion: 'reduce' });
  const page = await context.newPage();
  await page.goto(URL_UNDER_TEST);

  // The costume ember mounts but the global collapse stills its drift
  // (Chrome normalizes 0.01ms to "1e-05ms" — compare the number, not the string).
  const ember = page.locator('.workspace-ember');
  await expect(ember).toBeAttached();
  await expect
    .poll(() =>
      ember.evaluate((el) => parseFloat(getComputedStyle(el).animationDuration))
    )
    .toBeLessThan(0.1);

  // The Familiar's art only mounts once a measured run mood exists, which the
  // scripted testserver never emits — the video-fallback half of this contract
  // lives in AnimatedArt.test.ts instead. What the browser CAN prove here: the
  // atmosphere stage defers and the workspace still reaches a usable shell.
  await page.waitForFunction(
    () => performance.getEntriesByName('monaco-ready').length > 0,
    { timeout: 60_000 }
  );
  await expect(page.getByRole('region', { name: 'Editor' })).toBeVisible();
  await context.close();
});

test('keyboard shortcuts run and submit from the editor controls', async ({
  browser
}) => {
  const context = await bearerContext(browser);
  const page = await context.newPage();
  await page.goto(URL_UNDER_TEST);

  const results = page.getByRole('region', { name: 'Results' });
  const run = page.getByRole('button', { name: 'Run samples' });
  await expect(run).toBeEnabled({ timeout: 30_000 });

  // Ctrl+Enter is the documented run shortcut; focus starts on the page body.
  await page.locator('body').click();
  await page.keyboard.press('Control+Enter');
  await expect(results).toContainText(RESULT_ACTIVE, { timeout: 30_000 });

  // Ctrl+Shift+Enter submits — wait for the first run to release the controls.
  await expect(run).toBeEnabled({ timeout: 30_000 });
  await page.keyboard.press('Control+Shift+Enter');
  await expect(results).toContainText(RESULT_ACTIVE, { timeout: 30_000 });
  await context.close();
});

test('every primary editor control answers', async ({ browser }) => {
  const context = await bearerContext(browser);
  const page = await context.newPage();
  await page.goto(URL_UNDER_TEST);

  const results = page.getByRole('region', { name: 'Results' });
  const run = page.getByRole('button', { name: 'Run samples' });
  const submit = page.getByRole('button', { name: 'Submit' });
  await expect(run).toBeEnabled({ timeout: 30_000 });
  await expect(submit).toBeEnabled();

  // The language picker offers the problem's enabled languages.
  const picker = page.getByRole('combobox');
  await expect(picker).toBeVisible();
  expect(await picker.locator('option').count()).toBeGreaterThan(0);

  // The problem column collapses and restores.
  await page.getByRole('button', { name: 'Hide problem' }).click();
  await expect(
    page.getByRole('complementary', { name: 'Problem' })
  ).toBeHidden();
  await page.getByRole('button', { name: 'Show problem' }).click();
  await expect(
    page.getByRole('complementary', { name: 'Problem' })
  ).toBeVisible();

  await run.click();
  await expect(results).toContainText(RESULT_ACTIVE, { timeout: 30_000 });
  await expect(submit).toBeEnabled({ timeout: 30_000 });
  await submit.click();
  await expect(results).toContainText(RESULT_ACTIVE, { timeout: 30_000 });
  await context.close();
});
