// Shared helpers for the real-stack browser scripts (ide-workspace T21, T22). They sign in through
// the real magic-link flow and drive the Monaco editor. Nothing here intercepts a request.
import { chromium } from 'playwright-core';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
export const BASE = process.env.REAL_API_BASE ?? 'http://127.0.0.1:8088';
const SERVER_LOG = join(here, '.scratch', 'server.log');

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

/** Signs a fresh user in on `page` through the dev mail link. Returns true when it worked. */
export async function signIn(context, page, tag) {
  const logSize = readFileSync(SERVER_LOG, 'utf8').length;
  const email = `real-api-${tag}-${Date.now()}@example.test`;
  await context.request.post(`${BASE}/api/auth/email/magic`, {
    data: { email }
  });
  let token = '';
  for (let i = 0; i < 40 && !token; i++) {
    await new Promise((r) => setTimeout(r, 500));
    const match = readFileSync(SERVER_LOG, 'utf8')
      .slice(logSize)
      .match(/verify#token=([A-Za-z0-9_-]+)/);
    if (match) token = match[1];
  }
  if (!token) return false;
  await page.goto(`${BASE}/auth/verify#token=${token}`, {
    waitUntil: 'networkidle'
  });
  await page.getByRole('button', { name: 'Confirm sign-in' }).click();
  return page
    .waitForFunction(() => !location.pathname.includes('/auth/verify'), null, {
      timeout: 15000
    })
    .then(() => true)
    .catch(() => false);
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
