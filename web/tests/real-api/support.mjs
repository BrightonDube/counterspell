// Shared helpers for the real-stack browser scripts (ide-workspace T21, T22). They sign in by
// minting a session row in the dev database and drive the Monaco editor. Nothing here intercepts a request.
import { chromium } from 'playwright-core';
import { execFileSync } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';

export const BASE = process.env.REAL_API_BASE ?? 'http://127.0.0.1:8088';

export function createReporter() {
  const results = [];
  return {
    check(name, ok, detail = '') {
      const line = `${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`;
      results.push(line);
      console.log(line);
    },
    finish() {
      const failed = results.filter((r) => r.startsWith('FAIL')).length;
      console.log(
        `\n${results.length - failed}/${results.length} checks passed`
      );
      process.exit(failed ? 1 : 0);
    }
  };
}

export function launchBrowser() {
  return chromium.launch({
    headless: true,
    ...(process.env.E2E_CHANNEL ? { channel: process.env.E2E_CHANNEL } : {}),
    ...(process.env.E2E_EXECUTABLE
      ? { executablePath: process.env.E2E_EXECUTABLE }
      : {})
  });
}

/**
 * Creates a fresh user and a live session row in the dev database. The session id is the
 * SHA-256 of the cookie value, the same shape the OAuth verify handler would create, so the
 * session resolver treats it identically. Returns the raw cookie value, or '' on failure.
 * Needs DATABASE_URL (or E2E_DATABASE_URL) pointing at the stack's Postgres.
 * @param {string} tag short unique marker for the handle
 */
export function mintSession(tag) {
  const databaseUrl = process.env.E2E_DATABASE_URL ?? process.env.DATABASE_URL;
  if (!databaseUrl) return '';
  const cookie = `e2e-${tag}-${randomBytes(24).toString('base64url')}`;
  const hash = createHash('sha256').update(cookie).digest('hex');
  const handle = `e2e-${tag}-${Date.now().toString(36)}`.slice(0, 32);
  const sql = `INSERT INTO users (handle, display_name)
    VALUES ('${handle}', 'E2E ${tag}')
    ON CONFLICT (handle) DO NOTHING;
    INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at, expires_at, absolute_expires_at)
    SELECT '${hash}', id, 'oauth_github', now(), now(), now() + interval '7 days', now() + interval '90 days'
    FROM users WHERE handle = '${handle}';`;
  try {
    execFileSync(
      'psql',
      [databaseUrl, '-v', 'ON_ERROR_STOP=1', '-q', '-c', sql],
      {
        stdio: 'pipe'
      }
    );
    return cookie;
  } catch {
    return '';
  }
}

/**
 * Signs a fresh user in on `page` by minting a session and setting the cookie for BASE's host.
 * Returns true when the cookie landed and /api/me answers with a user.
 */
export async function signIn(context, page, tag) {
  const cookie = mintSession(tag);
  if (!cookie) return false;
  await context.addCookies([
    {
      name: 'cs_session',
      value: cookie,
      url: BASE
    }
  ]);
  const me = await page.request.get(`${BASE}/api/me`);
  return me.ok();
}

/** Opens the problem and waits until Monaco has mounted. */
export async function openProblem(page, slug) {
  await page.goto(`${BASE}/problems/${slug}`, { waitUntil: 'networkidle' });
  await page.waitForSelector('button:has-text("Run samples")', {
    timeout: 15000
  });
  await page.waitForSelector('.monaco-editor textarea', {
    state: 'attached',
    timeout: 30000
  });
  // The textarea exists before the starter source is loaded; typing earlier would be overwritten.
  await page.waitForFunction(
    () =>
      document
        .querySelector('.monaco-editor .view-lines')
        ?.textContent?.includes('add_two'),
    null,
    { timeout: 30000 }
  );
}

/** Puts the caret in the editor with Tab presses only, as a keyboard user would. */
export async function focusEditorByKeyboard(page) {
  for (let i = 0; i < 60; i++) {
    if (await page.$('.monaco-editor.focused')) return true;
    await page.keyboard.press('Tab');
  }
  return Boolean(await page.$('.monaco-editor.focused'));
}

/** Replaces the source with the keyboard. */
export async function typeSource(page, source) {
  if (!(await focusEditorByKeyboard(page)))
    throw new Error('the editor could not be reached with Tab');
  await page.keyboard.press('Control+a');
  await page.keyboard.insertText(source);
  // Confirm the editor took the text, so a lost keystroke cannot pass as a wrong answer.
  const marker = source.includes('a * b') ? 'a * b' : 'a + b';
  await page.waitForFunction(
    (text) =>
      document
        .querySelector('.monaco-editor .view-lines')
        ?.textContent?.replace(/\u00a0/g, ' ')
        .includes(text),
    marker,
    { timeout: 10000 }
  );
}

/** The focused element's role-ish label, for tab-order assertions. */
export function focusLabel(page) {
  return page.evaluate(() => {
    const el = document.activeElement;
    if (!el || el === document.body) return 'body';
    if (el.closest('.monaco-editor')) return 'editor';
    return (
      el.getAttribute('aria-label') ||
      el.textContent?.trim().replace(/\s+/g, ' ').slice(0, 60) ||
      el.tagName.toLowerCase()
    );
  });
}
