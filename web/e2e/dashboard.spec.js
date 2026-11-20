// Dashboard smoke path (dashboard-fsrs T10; DASH-1, DASH-7, DASH-8, DASH-9).
// Runs against `counterspell-testserver`: the production router, real Postgres and the built
// static site, with a SCRIPTED judge that passes every run. These tests check the dashboard
// read path and its states, not judging. Users are `Bearer test-user-N`.
import { expect, test } from '@playwright/test';

const BASE_URL = 'http://127.0.0.1:5198';

// The fixture seeds 4096 users. Slots of 2 users advance each second so a rerun inside the
// 600 s per-user run window never reuses a spent user (same scheme as run-session.axum.test.ts).
const SLOTS = 2000;
const firstUser = 9 + (Math.floor(Date.now() / 1000) % SLOTS) * 2;
const OWNER = firstUser;
const OTHER = firstUser + 1;

/** A browser context that signs in as one fixture user on every request. */
async function asUser(browser, number) {
  return browser.newContext({
    baseURL: BASE_URL,
    extraHTTPHeaders: { authorization: `Bearer test-user-${number}` }
  });
}

/** Submits through the real API and waits for the run to finish accepted. */
async function submitAccepted(context, code) {
  const created = await context.request.post('/api/runs', {
    headers: { 'Idempotency-Key': `e2e-${Date.now()}-${Math.random()}` },
    data: { problem: 'two-sum', lang: 'python', code, mode: 'submit' }
  });
  expect(created.status()).toBe(202);
  const { run_id: runId } = await created.json();
  await expect
    .poll(
      async () => {
        const response = await context.request.get(`/api/runs/${runId}`);
        const body = await response.json();
        return body.terminal ? body.verdict : 'running';
      },
      { timeout: 20_000 }
    )
    .toBe('accepted');
  return runId;
}

test('an owner sees real counts and no other user sees them or the code', async ({
  browser
}) => {
  const canary = `canary_${Math.random().toString(36).slice(2)}`;
  const owner = await asUser(browser, OWNER);
  await submitAccepted(
    owner,
    `# ${canary}\ndef two_sum(nums, target):\n    return [0, 1]\n`
  );

  const ownerPage = await owner.newPage();
  await ownerPage.goto('/dashboard');
  const solved = ownerPage.locator('section[aria-labelledby="solved-title"]');
  await expect(solved).toBeVisible();
  await expect(solved.locator('p.text-3xl')).toHaveText(/^\s*1\s+problems\s*$/);
  await expect(solved).toContainText('Your newest run was accepted.');
  await expect(ownerPage.getByText('No practice yet')).toHaveCount(0);
  // Recent runs list titles and verdicts, never source code.
  await expect(ownerPage.locator('body')).not.toContainText(canary);

  const other = await asUser(browser, OTHER);
  const otherPage = await other.newPage();
  await otherPage.goto('/dashboard');
  await expect(
    otherPage.getByRole('heading', { name: 'No practice yet' })
  ).toBeVisible();
  await expect(otherPage.locator('body')).not.toContainText(canary);

  await owner.close();
  await other.close();
});

test('a failed dashboard request shows the retry state and recovers', async ({
  browser
}) => {
  const context = await asUser(browser, OTHER);
  const page = await context.newPage();
  // The server-side 503 for a failed database read is covered by Rust tests. Here the
  // browser's reaction to that response is checked by answering the request at the network edge.
  await page.route('**/api/dashboard', (route) =>
    route.fulfill({
      status: 503,
      contentType: 'application/json',
      body: JSON.stringify({ code: 'unavailable', message: 'try later' })
    })
  );
  await page.goto('/dashboard');
  await expect(page.getByRole('alert')).toContainText('Try again in a moment');
  await page.unroute('**/api/dashboard');
  await page.getByRole('button', { name: 'Try again' }).click();
  await expect(
    page.getByRole('heading', { name: 'No practice yet' })
  ).toBeVisible();
  await context.close();
});

test('without a session the dashboard asks the visitor to sign in', async ({
  page
}) => {
  await page.goto('/dashboard');
  await expect(page.getByRole('link', { name: 'Sign in' })).toBeVisible();
});
