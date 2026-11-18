// ide-workspace T24/T25 harness check: drives the built SPA served by the real
// Axum server, with the real judge executing inside the rootless sandbox
// daemon and Postgres holding the queue and events. Nothing is intercepted;
// the only stub in the path is the log mail transport (development APP_ENV).
//
// Prereqs (see README.md): server on 127.0.0.1:8088 with stderr teed to
// .scratch/server.log, judge running against the same dev Postgres with
// SANDBOX_DOCKER_SOCKET, and the add-two pack seeded via seed-demo.
import { chromium } from 'playwright-core';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
const BASE = process.env.REAL_API_BASE ?? 'http://127.0.0.1:8088';
const SERVER_LOG = join(here, '.scratch', 'server.log');
const results = [];
const check = (name, ok, detail = '') => {
  const line = `${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`;
  results.push(line);
  console.log(line);
};

const browser = await chromium.launch({ channel: 'chrome', headless: true });
const context = await browser.newContext();
const page = await context.newPage();
page.on('console', (m) => {
  if (m.type() === 'error')
    results.push(`CONSOLE-ERROR ${m.text().slice(0, 160)}`);
});
// The page makes every run request itself; the harness only observes the ids
// so it can read the authoritative terminal model afterwards.
const runIds = [];
page.on('response', (r) => {
  if (
    (r.url().endsWith('/api/runs') || r.url().endsWith('/api/runs/ghost')) &&
    r.request().method() === 'POST' &&
    r.status() === 202
  )
    r.json()
      .then((body) => runIds.push(body.run_id))
      .catch(() => {});
});

// --- Real sign-in through the dev mail transport ---
const logSize = readFileSync(SERVER_LOG, 'utf8').length;
// Unique address per run: the magic endpoint limits to 3 links per address per
// hour, and the verify handler creates the user on first sign-in.
const email = `real-api-harness-${Date.now()}@example.test`;
await context.request.post(`${BASE}/api/auth/email/magic`, {
  data: { email }
});
let token = '';
for (let i = 0; i < 40 && !token; i++) {
  await new Promise((r) => setTimeout(r, 500));
  const fresh = readFileSync(SERVER_LOG, 'utf8').slice(logSize);
  const match = fresh.match(/verify#token=([A-Za-z0-9_-]+)/);
  if (match) token = match[1];
}
check('auth: dev mail link reaches the server log', token.length > 0);
await page.goto(`${BASE}/auth/verify#token=${token}`, {
  waitUntil: 'networkidle'
});
// The verify page holds the token in memory only; the user must confirm.
await page.getByRole('button', { name: 'Confirm sign-in' }).click();
const leftVerify = await page
  .waitForFunction(() => !location.pathname.includes('/auth/verify'), null, {
    timeout: 15000
  })
  .then(() => true)
  .catch(() => false);
check('auth: magic link signs in and redirects', leftVerify, page.url());

// --- Real problem page over Axum-served static output ---
await page.goto(`${BASE}/problems/add-two`, { waitUntil: 'networkidle' });
await page.waitForSelector('button:has-text("Run samples")', {
  timeout: 15000
});
check('problem: page loads through Axum', await page.isVisible('text=Add Two'));

// Monaco's hidden textarea stays readonly until the editor is focused; fill()
// cannot drive it. Focus the editor surface, select all, then insertText.
async function setSource(src) {
  await page.locator('.monaco-editor .view-lines').first().click();
  await page.waitForSelector('.monaco-editor.focused', { timeout: 10000 });
  await page.keyboard.press('Control+a');
  await page.keyboard.insertText(src);
}

// --- Correct source: Run samples through the real judge ---
await setSource('def add_two(a, b):\n    return a + b\n');
await page.getByRole('button', { name: 'Run samples' }).click();
const accepted = await page
  .waitForSelector('text=Accepted', { timeout: 90000 })
  .catch(() => null);
check('run: samples reach a real Accepted verdict', !!accepted);

// --- Wrong source: Submit runs the real hidden suite in the sandbox ---
await setSource('def add_two(a, b):\n    return a * b\n');
await page.getByRole('button', { name: 'Submit', exact: true }).click();
const wrong = await page
  .waitForSelector('text=Wrong answer', { timeout: 180000 })
  .catch(() => null);
check('submit: wrong answer reaches a real Wrong answer verdict', !!wrong);

// --- Ghost hunt lifecycle on the real stack (ide-workspace T25) ---
// The verdict lands before the hunt finishes, so the counterexample may still
// be arriving: `done` follows the ghost events in the stream, but the panel
// renders the ghost as soon as its event parses.
const submitId = runIds[runIds.length - 1] ?? '';
const counterexample = await page
  .waitForSelector('text=Minimized counterexample', { timeout: 120000 })
  .catch(() => null);
check('ghost: minimized counterexample renders after the submit', !!counterexample);

// --- Persisted events + terminal read straight from Postgres ---
function pgScalar(sql) {
  for (const docker of ['docker exec', 'wsl docker exec']) {
    try {
      return execSync(
        `${docker} cs-dev-pg psql -U counterspell -d counterspell -tAc "${sql}"`,
        { stdio: ['ignore', 'pipe', 'ignore'] }
      )
        .toString()
        .trim();
    } catch {}
  }
  return '';
}
const rows = pgScalar(
  "select count(*) from run_events union all select count(*) from submissions where status = 'completed' union all select count(*) from run_events where type = 'test' and payload->>'sample' = 'false' and (payload ? 'input' or payload ? 'expected' or payload ? 'got')"
)
  .split('\n')
  .map((s) => parseInt(s, 10) || 0);
check('pg: run events persisted', rows[0] > 0, `events=${rows[0]}`);
check('pg: terminal submissions recorded', rows[1] >= 1, `terminal=${rows[1]}`);
check(
  'pg: hidden test events carry no input/expected/got',
  rows[2] === 0,
  `leaking=${rows[2]}`
);

// The submit's event stream must read familiar -> ghost_hunting -> shrink_step*
// -> ghost -> done, matching the engine order the client renders.
const seqLines = pgScalar(
  `select type from run_events where submission_id = '${submitId}' order by seq`
).split('\n');
const at = (type) => seqLines.indexOf(type);
check(
  'ghost: hunt follows the familiar verdict in the stream',
  at('familiar') >= 0 && at('ghost_hunting') > at('familiar'),
  `stream=${seqLines.join(',')}`
);
check(
  'ghost: shrink steps precede the ghost event',
  at('shrink_step') > at('ghost_hunting') &&
    seqLines.lastIndexOf('shrink_step') < at('ghost'),
  `stream=${seqLines.join(',')}`
);
let ghostJson = {};
try {
  ghostJson = JSON.parse(
    pgScalar(
      `select payload::text from run_events where submission_id = '${submitId}' and type = 'ghost'`
    )
  );
} catch {}
check(
  'ghost: payload carries kind, input, provenance and autopsy',
  ghostJson.kind === 'wrong' &&
    ghostJson.input !== undefined &&
    (ghostJson.provenance?.problem_version ?? 0) >= 1 &&
    (ghostJson.autopsy?.explanation?.length ?? 0) > 0,
  JSON.stringify(ghostJson).slice(0, 140)
);

// --- Authoritative terminal read over the same session ---
if (runIds.length === 0) {
  check('terminal: run ids observed for read model', false);
} else {
  const last = runIds[runIds.length - 1];
  const read = await context.request.get(`${BASE}/api/runs/${last}`);
  const model = await read.json().catch(() => ({}));
  check(
    'terminal: read model reports the real verdict',
    read.status() === 200 &&
      model.status === 'completed' &&
      model.verdict === 'wrong_answer',
    `status=${read.status()} verdict=${model.verdict}`
  );
}

// The autopsy's explanation must survive the whole path into the page; the
// empty card always renders its placeholder line instead.
const ghostBody = await page.evaluate(() => document.body.innerText);
check(
  'ghost: autopsy explanation renders on the page',
  typeof ghostJson.autopsy?.explanation === 'string' &&
    ghostJson.autopsy.explanation.length > 0 &&
    ghostBody.includes(ghostJson.autopsy.explanation),
  (ghostJson.autopsy?.explanation ?? '').slice(0, 80)
);

// --- Rerun the found ghost through the custom ghost endpoint (T02) ---
// The panel's rerun button re-judges the same failing input; the wrong source
// is still in the editor, so the custom run must reach a real verdict.
await page.getByRole('button', { name: 'Rerun ghost' }).click();
let rerunVerdict = '';
for (let i = 0; i < 180 && !rerunVerdict; i++) {
  await new Promise((r) => setTimeout(r, 1000));
  const rerunId = runIds[2];
  if (!rerunId) continue;
  rerunVerdict = pgScalar(
    `select verdict from submissions where id = '${rerunId}' and verdict is not null`
  );
}
check(
  'ghost: rerun reaches a real verdict',
  rerunVerdict === 'wrong_answer' || rerunVerdict === 'accepted',
  `verdict=${rerunVerdict || 'none'} runs=${runIds.length}`
);

// --- Fix the source, resubmit, reach the real Accepted verdict ---
await setSource('def add_two(a, b):\n    return a + b\n');
await page.getByRole('button', { name: 'Submit', exact: true }).click();
const fixed = await page
  .waitForSelector('text=Accepted', { timeout: 180000 })
  .catch(() => null);
check('submit: corrected source reaches a real Accepted verdict', !!fixed);

// --- Hidden canary: no hidden-test detail on the page ---
const body = await page.evaluate(() => document.body.innerText);
check(
  'safety: no hidden input/expected/got detail leaks',
  !/hidden.*(input|expected|got)/i.test(body) && !/\[\[?-?\d+ *, *-?\d+.*\]\]/.test(body.replace(/\[\d+, \d+\]/g, '')),
  'no generated tuples visible'
);

await browser.close();
const failed = results.filter((r) => r.startsWith('FAIL')).length;
console.log(`\n${results.length - failed}/${results.length} checks passed`);
process.exit(failed ? 1 : 0);
