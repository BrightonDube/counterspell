// ide-workspace T24/T25 harness check: drives the built SPA served by the real
// Axum server, with the real judge executing inside the rootless sandbox
// daemon and Postgres holding the queue and events. Nothing is intercepted;
// the only stub in the path is the log mail transport (development APP_ENV).
//
// Prereqs (see README.md): server on 127.0.0.1:8088 with stderr teed to
// .scratch/server.log, judge running against the same dev Postgres with
// SANDBOX_DOCKER_SOCKET, and the add-two pack seeded via seed-demo.
import { chromium } from 'playwright-core';
import { execFileSync, execSync } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
const BASE = process.env.REAL_API_BASE ?? 'http://127.0.0.1:8088';
const results = [];
const check = (name, ok, detail = '') => {
  const line = `${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`;
  results.push(line);
  console.log(line);
};

const browser = await chromium.launch({
  headless: true,
  ...(process.env.E2E_CHANNEL ? { channel: process.env.E2E_CHANNEL } : {}),
  ...(process.env.E2E_EXECUTABLE
    ? { executablePath: process.env.E2E_EXECUTABLE }
    : {})
});
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

// Every speech-bubble request the page makes. The page may ask once per finished run.
const speechPosts = [];
page.on('request', (r) => {
  if (r.method() === 'POST' && r.url().includes('/familiar-hint'))
    speechPosts.push(r.url().split('/api/runs/')[1].split('/')[0]);
});
// The report records what the real stack produced, for the T25 evidence.
const report = { runIds: [], events: {}, notes: {} };

// --- Real sign-in: a session row minted in the dev database ---
// The session id is the SHA-256 of the cookie value, the same shape the OAuth verify
// handler writes. Needs DATABASE_URL (or E2E_DATABASE_URL) for the stack's Postgres.
const databaseUrl = process.env.E2E_DATABASE_URL ?? process.env.DATABASE_URL;
const cookieValue = `e2e-harness-${randomBytes(24).toString('base64url')}`;
const sessionHash = createHash('sha256').update(cookieValue).digest('hex');
const handle = `e2e-check-${Date.now().toString(36)}`.slice(0, 32);
let minted = false;
if (databaseUrl) {
  const sql =
    `INSERT INTO users (handle, display_name) VALUES ('${handle}', 'E2E harness') ` +
    `ON CONFLICT (handle) DO NOTHING; ` +
    `INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at, ` +
    `expires_at, absolute_expires_at) SELECT '${sessionHash}', id, 'oauth_github', ` +
    `now(), now(), now() + interval '7 days', now() + interval '90 days' ` +
    `FROM users WHERE handle='${handle}';`;
  try {
    execFileSync(
      'psql',
      [databaseUrl, '-v', 'ON_ERROR_STOP=1', '-q', '-c', sql],
      {
        stdio: 'pipe'
      }
    );
    minted = true;
  } catch {
    minted = false;
  }
}
check('auth: session minted in the dev database', minted);
await context.addCookies([
  { name: 'cs_session', value: cookieValue, url: BASE }
]);
const me = await page.request.get(`${BASE}/api/me`);
check('auth: minted session identifies a user', me.ok());

// --- The sign-in contract: providers and the demo account, email retired ---
const cfg = await (await page.request.get(`${BASE}/api/auth/config`)).json();
check(
  'auth: config advertises provider sign-in, email routes retired',
  cfg.email?.magic_link === false && cfg.email?.password === false
);

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

// --- Correct source: the draft survives a reload, then Run samples ---
await setSource('def add_two(a, b):\n    return a + b\n');
await page.waitForTimeout(1500); // the draft saver is debounced
await page.reload({ waitUntil: 'networkidle' });
await page.waitForSelector('button:has-text("Run samples")', {
  timeout: 15000
});
const restored = await page
  .waitForFunction(
    () =>
      document
        .querySelector('.monaco-editor .view-lines')
        ?.textContent?.replace(/\u00a0/g, ' ')
        .includes('return a + b'),
    null,
    { timeout: 15000 }
  )
  .then(() => true)
  .catch(() => false);
check('drafts: the saved source is restored after a reload', restored);
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
check(
  'ghost: minimized counterexample renders after the submit',
  !!counterexample
);

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
    } catch {
      // This docker prefix did not work here; try the next one.
    }
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
} catch {
  // No ghost row or unreadable JSON: ghostJson stays empty and the check below fails with it.
}
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

// --- Authored hints reveal in order from the real ladder ---
await page.getByRole('button', { name: 'Show hint 1' }).click();
const rung1 = await page
  .waitForSelector('text=Read the signature: the judge calls add_two', {
    timeout: 30000
  })
  .catch(() => null);
check('hints: rung 1 is the authored text from the pack', !!rung1);
const nextAfterOne = await page
  .getByRole('button', { name: 'Show hint 2' })
  .isVisible();
check(
  'hints: rung 2 is offered only after rung 1',
  nextAfterOne &&
    !(await page.getByRole('button', { name: 'Show hint 3' }).isVisible())
);

// --- The animated shrink list equals the persisted trace, step for step ---
const shownSteps = (
  await page.evaluate(() =>
    [...document.querySelectorAll('li')]
      .map((li) => li.textContent?.trim() ?? '')
      .filter((text) => /^Step \d+: /.test(text))
  )
).map((text) => text.replace(/^Step \d+: /, ''));
const recordedSteps = (ghostJson.shrink?.steps ?? []).map((step) =>
  JSON.stringify(step.input)
);
check(
  'ghost: the shrink list on the page equals the persisted trace',
  recordedSteps.length > 0 &&
    JSON.stringify(shownSteps.slice(-recordedSteps.length)) ===
      JSON.stringify(recordedSteps),
  `page=${shownSteps.join(' | ')} stored=${recordedSteps.join(' | ')}`
);
report.notes.shrink = { shown: shownSteps, stored: recordedSteps };
report.notes.ghostProvenance = ghostJson.provenance ?? null;

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
  !/hidden.*(input|expected|got)/i.test(body) &&
    !/\[\[?-?\d+ *, *-?\d+.*\]\]/.test(body.replace(/\[\d+, \d+\]/g, '')),
  'no generated tuples visible'
);

// --- Measured Familiar and one speech request per finished run ---
const finalId = runIds[runIds.length - 1] ?? '';
const familiarEvent = pgScalar(
  `select payload::text from run_events where submission_id = '${finalId}' and type = 'familiar'`
);
let familiarData = {};
try {
  familiarData = JSON.parse(familiarEvent);
} catch {
  // No familiar event: the check below fails with the empty value.
}
check(
  'familiar: a measured mood was persisted for the accepted run',
  typeof familiarData.mood === 'number' && familiarData.layer === 'measured',
  familiarEvent.slice(0, 120)
);
for (let i = 0; i < 40 && !speechPosts.includes(finalId); i++)
  await new Promise((r) => setTimeout(r, 500));
check(
  'familiar: one speech request for the final run, none repeated',
  speechPosts.filter((id) => id === finalId).length === 1 &&
    new Set(speechPosts).size === speechPosts.length,
  `requests=${speechPosts.length} runs=${runIds.length}`
);
report.runIds = runIds;
for (const id of runIds) {
  const seqs = pgScalar(
    `select string_agg(seq::text || ':' || type, ',' order by seq) from run_events where submission_id = '${id}'`
  );
  report.events[id] = seqs;
}
report.notes.familiar = familiarData;
report.notes.speechRequests = speechPosts;
const finalRead = await context.request.get(`${BASE}/api/runs/${finalId}`);
report.notes.finalTerminal = await finalRead.json().catch(() => ({}));
mkdirSync(join(here, '.scratch'), { recursive: true });
writeFileSync(
  join(here, '.scratch', 'ide-workspace-report.json'),
  JSON.stringify(report, null, 2)
);

await browser.close();
const failed = results.filter((r) => r.startsWith('FAIL')).length;
console.log(`\n${results.length - failed}/${results.length} checks passed`);
process.exit(failed ? 1 : 0);
