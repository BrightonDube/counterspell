#!/usr/bin/env node
// ide-workspace T26: reconnect, retry, ownership and failure paths on the real
// stack — the same Axum server, judge, sandbox daemon and Postgres as
// real-api-check.mjs. Nothing is mocked: every assertion reads the wire, the
// SSE stream or the authoritative Postgres rows.
//
//   node web/tests/real-api/reconnect.spec.mjs
//
// Requires the dev stack up (see README.md): Axum on :8088, judge worker,
// cs-dev-pg and the rootless sandbox daemon. The final leg kills and restarts
// the judge process to prove worker-retry behaviour, so the judge is left
// running under dev-test.env afterwards.
import { execFileSync } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const BASE = process.env.REAL_API_BASE ?? 'http://127.0.0.1:8088';
const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
// The judge this script may kill and restart. Only the PID in this file is touched, never any
// other judge on the machine. The restarted judge runs from this repository.
const JUDGE_PID_FILE = process.env.JUDGE_PID_FILE ?? '';

const results = [];
function check(name, ok, detail = '') {
  results.push([name, ok, detail]);
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`);
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Postgres scalar through the dev container, like real-api-check.mjs.
function pgScalar(sql) {
  for (const docker of ['docker exec', 'wsl docker exec']) {
    try {
      return execFileSync(
        docker.startsWith('wsl') ? 'wsl' : 'bash',
        docker.startsWith('wsl')
          ? [
              'docker',
              'exec',
              'cs-dev-pg',
              'psql',
              '-U',
              'counterspell',
              '-d',
              'counterspell',
              '-tAc',
              sql
            ]
          : [
              '-c',
              `${docker} cs-dev-pg psql -U counterspell -d counterspell -tAc "${sql}"`
            ],
        { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }
      ).trim();
    } catch {
      /* try next docker shim */
    }
  }
  return '';
}

// Runs a shell script on the machine that hosts the stack: directly on Linux, through WSL when
// this script runs Windows-side.
function hostShell(script, args = []) {
  const [command, argv] =
    process.platform === 'win32'
      ? ['wsl', ['-e', 'bash', '-c', script, 'host-shell', ...args]]
      : ['bash', ['-c', script, 'host-shell', ...args]];
  return execFileSync(command, argv, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe']
  }).trim();
}

// Kills only the judge named in JUDGE_PID_FILE, and only if that PID really is a
// counterspell-judge process. Paths reach the shell as arguments, never inside the script text.
function killJudge() {
  if (!JUDGE_PID_FILE) {
    console.error(
      'JUDGE_PID_FILE is not set; refusing to kill an unknown judge.'
    );
    process.exit(2);
  }
  const text = readFileSync(JUDGE_PID_FILE, 'utf8').trim();
  if (!/^[1-9][0-9]*$/.test(text) || Number(text) <= 1) {
    console.error(
      `JUDGE_PID_FILE does not hold a usable PID: ${JSON.stringify(text)}`
    );
    process.exit(2);
  }
  hostShell(
    'case "$(readlink /proc/"$1"/exe 2>/dev/null)" in */counterspell-judge | *"/counterspell-judge (deleted)") kill -9 "$1" ;; esac',
    [text]
  );
}

// Starts a fresh judge from this repository. The whole start runs detached with every stream
// redirected, so this call returns at once. The child writes its own PID and then becomes the
// judge, so the file names the judge.
function startJudge() {
  hostShell(
    '( cd "$1" && set -a && . ~/.config/counterspell/dev-test.env && set +a && ' +
      'LANGUAGES_FILE=languages/languages.toml RUST_LOG=info ' +
      `exec setsid nohup sh -c 'echo $$ > "$0"; exec ./target/debug/counterspell-judge' "$2" ` +
      '>> web/tests/real-api/.scratch/judge.log 2>&1 < /dev/null ) > /dev/null 2>&1 < /dev/null &',
    [REPO, JUDGE_PID_FILE]
  );
}

// --- auth ---------------------------------------------------------------
// A session row is minted straight into the dev database. The id is the SHA-256 of the
// cookie value — the same shape the OAuth verify handler writes — so the resolver treats it
// identically. Needs DATABASE_URL (or E2E_DATABASE_URL) for the stack's Postgres.
async function signIn(tag) {
  const databaseUrl = process.env.E2E_DATABASE_URL ?? process.env.DATABASE_URL;
  if (!databaseUrl) return { ok: false, cookie: '', status: 'no DATABASE_URL' };
  const value = `e2e-reconnect-${tag}-${randomBytes(24).toString('base64url')}`;
  const hash = createHash('sha256').update(value).digest('hex');
  const handle = `e2e-rec-${tag}-${Date.now().toString(36)}`.slice(0, 32);
  const sql =
    `INSERT INTO users (handle, display_name) VALUES ('${handle}', 'E2E ${tag}') ` +
    `ON CONFLICT (handle) DO NOTHING; ` +
    `INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at, ` +
    `expires_at, absolute_expires_at) SELECT '${hash}', id, 'oauth_github', now(), now(), ` +
    `now() + interval '7 days', now() + interval '90 days' FROM users WHERE handle='${handle}';`;
  let ok = true;
  try {
    execFileSync(
      'psql',
      [databaseUrl, '-v', 'ON_ERROR_STOP=1', '-q', '-c', sql],
      {
        stdio: 'pipe'
      }
    );
  } catch {
    ok = false;
  }
  return { ok, cookie: `cs_session=${value}`, status: `minted=${ok}` };
}

const WRONG = 'def add_two(a, b):\n    return a * b\n';
const RIGHT = 'def add_two(a, b):\n    return a + b\n';
const SYNTAX = 'def add_two(a, b:\n    return a + b\n';
const LOUD =
  'import sys\ndef add_two(a, b):\n    sys.stdout.write("x" * 1000000)\n' +
  '    sys.stderr.write("y" * 1000000)\n    return a + b\n';

async function postRun(cookie, code, mode = 'run') {
  const res = await fetch(`${BASE}/api/runs`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      cookie,
      origin: BASE,
      'idempotency-key': crypto.randomUUID()
    },
    body: JSON.stringify({ problem: 'add-two', lang: 'python', code, mode })
  });
  return { status: res.status, body: await res.json().catch(() => ({})) };
}

async function getRun(cookie, runId) {
  const res = await fetch(`${BASE}/api/runs/${runId}`, { headers: { cookie } });
  return { status: res.status, body: await res.json().catch(() => ({})) };
}

// Reads SSE frames until `until` fires or `timeoutMs` passes. Returns the
// parsed {seq, event, data} frames, aborting the socket on the way out.
async function readSse(
  cookie,
  runId,
  { lastEventId, timeoutMs = 15000, until } = {}
) {
  const headers = { accept: 'text/event-stream', cookie };
  if (lastEventId !== undefined) headers['last-event-id'] = String(lastEventId);
  const controller = new AbortController();
  const res = await fetch(`${BASE}/api/runs/${runId}/events`, {
    headers,
    signal: controller.signal
  });
  if (!res.ok)
    return {
      status: res.status,
      frames: [],
      error: await res.text().catch(() => '')
    };
  const frames = [];
  const decoder = new TextDecoder();
  let buf = '';
  const deadline = Date.now() + timeoutMs;
  try {
    for await (const chunk of res.body) {
      buf += decoder.decode(chunk, { stream: true });
      let idx;
      while ((idx = buf.indexOf('\n\n')) >= 0) {
        const block = buf.slice(0, idx);
        buf = buf.slice(idx + 2);
        const frame = { seq: null, event: '', data: '' };
        for (const line of block.split('\n')) {
          if (line.startsWith('id:'))
            frame.seq = parseInt(line.slice(3).trim(), 10);
          else if (line.startsWith('event:'))
            frame.event = line.slice(6).trim();
          else if (line.startsWith('data:')) frame.data += line.slice(5).trim();
        }
        if (frame.seq !== null || frame.event) frames.push(frame);
        if (until && until(frames)) return { status: res.status, frames };
      }
      if (Date.now() > deadline) break;
    }
  } catch {
    /* aborted or server closed */
  } finally {
    controller.abort();
  }
  return { status: res.status, frames };
}

async function waitTerminal(runId, timeoutMs = 240000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const status = pgScalar(
      `select status from submissions where id = '${runId}'`
    );
    if (
      status === 'completed' ||
      status === 'internal_error' ||
      status === 'cancelled'
    )
      return status;
    await sleep(1500);
  }
  return 'timeout';
}

function eventTypes(runId) {
  return pgScalar(
    `select type from run_events where submission_id = '${runId}' order by seq`
  )
    .split('\n')
    .filter(Boolean);
}

// -------------------------------------------------------------------------
// Leg 1: sign-in over the real mail transport.
const userA = await signIn('a');
check(
  'auth: user A signs in through dev mail',
  userA.ok,
  `status=${userA.status}`
);

// Leg 2: live SSE drop + Last-Event-ID resume during a real submit.
// The wrong submit hunts for ~8s, leaving a live window to cut and resume.
const submit = await postRun(userA.cookie, WRONG, 'submit');
check(
  'submit: wrong solution accepted for judging',
  submit.status === 202,
  `status=${submit.status}`
);
const submitId = submit.body.run_id;

const first = await readSse(userA.cookie, submitId, {
  until: (f) => f.length >= 2,
  timeoutMs: 30000
});
const cutAt = first.frames.at(-1)?.seq;
check(
  'sse: live stream delivers ordered frames',
  first.frames.length >= 2 &&
    first.frames.every((f, i) => i === 0 || f.seq > first.frames[i - 1].seq),
  `frames=${first.frames.map((f) => `${f.seq}:${f.event}`).join(',')}`
);

// Resume at the cut: the next frame must continue the sequence with no gap.
const resumed = await readSse(userA.cookie, submitId, {
  lastEventId: cutAt,
  until: (f) => f.length >= 1,
  timeoutMs: 20000
});
check(
  'sse: Last-Event-ID resumes without a gap',
  resumed.frames.length > 0 && resumed.frames[0].seq === cutAt + 1,
  `cut=${cutAt} resumed=${resumed.frames[0]?.seq}:${resumed.frames[0]?.event}`
);

// Resume one earlier: the server replays the cut frame identically, so a
// client that already applied it can de-duplicate by seq.
const replayed = await readSse(userA.cookie, submitId, {
  lastEventId: cutAt - 1,
  until: (f) => f.length >= 1,
  timeoutMs: 20000
});
const cutFrame = first.frames.at(-1);
const replayFrame = replayed.frames[0];
check(
  'sse: stale cursor replays the same frame for de-dup',
  replayFrame?.seq === cutFrame?.seq &&
    replayFrame?.event === cutFrame?.event &&
    replayFrame?.data === cutFrame?.data,
  `orig=${cutFrame?.seq}:${cutFrame?.event} replay=${replayFrame?.seq}:${replayFrame?.event}`
);

// Leg 3: a malformed cursor is a 400, not a silent restart.
const badCursor = await readSse(userA.cookie, submitId, {
  lastEventId: 'abc',
  timeoutMs: 8000
});
check(
  'sse: malformed Last-Event-ID is refused',
  badCursor.status === 400 && badCursor.error.includes('invalid_last_event_id'),
  `status=${badCursor.status}`
);

// Leg 4: the run finishes on its own; its terminal read model stands alone.
const terminal = await waitTerminal(submitId);
check(
  'submit: wrong solution reaches a terminal status',
  terminal === 'completed',
  `status=${terminal}`
);

// Leg 5: reload after terminal — the read model carries verdict and ghost.
const view = await getRun(userA.cookie, submitId);
check(
  'reload: terminal read model carries the verdict',
  view.status === 200 && view.body.verdict === 'wrong_answer',
  `verdict=${view.body.verdict}`
);
const seqTypes = eventTypes(submitId);
check(
  'reload: ghost lifecycle persisted for replay',
  seqTypes.includes('ghost_hunting') &&
    seqTypes.includes('ghost') &&
    seqTypes.at(-1) === 'done',
  `events=${seqTypes.length}`
);

// A finished run replays its whole stream from cursor 0 and then ends — this
// is the gap-recovery path after a long disconnect.
const full = await readSse(userA.cookie, submitId, {
  lastEventId: 0,
  timeoutMs: 15000
});
check(
  'sse: finished run replays the full stream for gap recovery',
  full.frames.length === seqTypes.length &&
    full.frames.at(-1)?.event === 'done',
  `replayed=${full.frames.length} persisted=${seqTypes.length}`
);

// Leg 6: ownership — a second user cannot read or stream the run.
const userB = await signIn('b');
check(
  'auth: user B signs in through dev mail',
  userB.ok,
  `status=${userB.status}`
);
const foreign = await getRun(userB.cookie, submitId);
const foreignSse = await readSse(userB.cookie, submitId, { timeoutMs: 8000 });
check(
  'ownership: foreign run reads and streams are refused',
  foreign.status === 404 && foreignSse.status === 404,
  `read=${foreign.status} sse=${foreignSse.status}`
);

// Leg 7: a missing run id refuses the same way (existence is never revealed).
const missing = await getRun(userA.cookie, crypto.randomUUID());
check(
  'ownership: missing run is refused',
  missing.status === 404,
  `status=${missing.status}`
);

// Leg 8: source validation refuses oversized source before queueing.
const fat = await postRun(
  userA.cookie,
  `def add_two(a, b):\n    return a + b\n${'#'.padEnd(70000, '#')}\n`,
  'run'
);
check(
  'validation: oversized source refused',
  fat.status === 400 || fat.status === 413,
  `status=${fat.status} code=${fat.body?.error?.code ?? fat.body?.code ?? ''}`
);

// Leg 9: malformed source is refused at the API, not judged — for python the
// "compile" check is the gate itself, so compile_error is unreachable here.
const bad = await postRun(userA.cookie, SYNTAX, 'submit');
check(
  'validation: malformed source refused before judging',
  bad.status === 400,
  `status=${bad.status} code=${bad.body?.error?.code ?? bad.body?.code ?? ''}`
);

// Leg 10: an output-limit submit is a failing verdict the hunt must not take —
// the unsupported-verdict gate is proven on a real submit, not run mode. If the
// function driver does not count function stdout/stderr as run output the run
// is accepted instead, which still proves the verdict gate (accepted never hunts).
const loud = await postRun(userA.cookie, LOUD, 'submit');
check(
  'failure: output-limit submit accepted for judging',
  loud.status === 202,
  `status=${loud.status}`
);
const loudTerminal = await waitTerminal(loud.body.run_id, 180000);
const loudVerdict = pgScalar(
  `select verdict from submissions where id = '${loud.body.run_id}'`
);
const loudEvents = eventTypes(loud.body.run_id);
check(
  'failure: an unsupported verdict never hunts',
  loudTerminal === 'completed' &&
    (loudVerdict === 'output_limit' || loudVerdict === 'accepted') &&
    !loudEvents.includes('ghost_hunting') &&
    !loudEvents.includes('ghost'),
  `status=${loudTerminal} verdict=${loudVerdict}`
);

// Leg 12 (before the burst, so its POST is not throttled): worker retry — kill
// the judge mid-submit; the run is reclaimed (`restarted`) and still finished.
const huntRun = await postRun(userA.cookie, WRONG, 'submit');
check(
  'retry: hunt run accepted for judging',
  huntRun.status === 202,
  `status=${huntRun.status}`
);
const huntId = huntRun.body.run_id;
// Wait for the run to be claimed (first event) so the kill lands mid-work.
for (let i = 0; i < 30 && eventTypes(huntId).length === 0; i++)
  await sleep(1000);
killJudge();
await sleep(500);
startJudge();
const retryTerminal = await waitTerminal(huntId, 300000);
const retryEvents = eventTypes(huntId);
const retryVerdict = pgScalar(
  `select verdict from submissions where id = '${huntId}'`
);
const retryError = pgScalar(
  `select payload->>'code' from run_events where submission_id = '${huntId}' and type = 'error'`
);
// A killed run is reclaimed (`restarted`) and the retry judges it to its real
// verdict and ghost. Each attempt names its containers with its attempt number,
// so the killed attempt's container cannot block the retry.
check(
  'retry: killed run is reclaimed and terminated',
  retryEvents.includes('restarted') &&
    retryTerminal === 'completed' &&
    retryVerdict === 'wrong_answer' &&
    retryEvents.includes('ghost'),
  `status=${retryTerminal} verdict=${retryVerdict} error=${retryError} events=${retryEvents.join(',')}`
);

// Leg 13: failure verdicts are named for what happened, never Wrong Answer.
const LOOP = 'def add_two(a, b):\n    while True:\n        pass\n';
const CRASH = 'def add_two(a, b):\n    raise ValueError("boom")\n';
// Parses as Python for tree-sitter, but Python itself refuses it when the module loads.
const COMPILE = 'def add_two(a, b):\n    return a + b\nnonlocal a\n';
async function verdictOf(code) {
  const posted = await postRun(userB.cookie, code, 'submit');
  if (posted.status !== 202)
    return { posted: posted.status, code: posted.body?.code };
  const id = posted.body.run_id;
  const status = await waitTerminal(id, 240000);
  return {
    posted: posted.status,
    status,
    verdict: pgScalar(`select verdict from submissions where id = '${id}'`),
    read: (await getRun(userB.cookie, id)).body
  };
}
const looped = await verdictOf(LOOP);
check(
  'failure: an endless loop ends as Time limit, not Wrong answer',
  looped.status === 'completed' &&
    looped.verdict === 'time_limit' &&
    looped.read?.verdict === 'time_limit',
  `posted=${looped.posted} status=${looped.status} verdict=${looped.verdict}`
);
const crashed = await verdictOf(CRASH);
check(
  'failure: an exception ends as Runtime error, not Wrong answer',
  crashed.status === 'completed' && crashed.verdict === 'runtime_error',
  `posted=${crashed.posted} status=${crashed.status} verdict=${crashed.verdict}`
);
const refused = await verdictOf(COMPILE);
check(
  'failure: a module Python refuses is a Compile or Runtime error, never judged as an answer',
  (refused.posted === 400 && refused.code === 'source_does_not_parse') ||
    (refused.status === 'completed' &&
      (refused.verdict === 'compile_error' ||
        refused.verdict === 'runtime_error')),
  `posted=${refused.posted} code=${refused.code ?? ''} status=${refused.status} verdict=${refused.verdict}`
);

// Leg 14: an infrastructure failure is an internal error, never a verdict. The worker
// is killed each time it claims the run, until the attempts run out.
const doomed = await postRun(userB.cookie, WRONG, 'submit');
const doomedId = doomed.body?.run_id;
let kills = 0;
for (let i = 0; i < 600 && doomedId; i++) {
  const status = pgScalar(
    `select status from submissions where id = '${doomedId}'`
  );
  if (status === 'internal_error' || status === 'completed') break;
  const claimed = pgScalar(
    `select attempts from jobs where submission_id = '${doomedId}' and state = 'claimed'`
  );
  if (claimed !== '' && Number(claimed) >= kills) {
    killJudge();
    kills += 1;
    await sleep(500);
    startJudge();
  }
  await sleep(500);
}
const doomedTerminal = doomedId ? await waitTerminal(doomedId, 240000) : 'none';
const doomedRead = doomedId ? (await getRun(userB.cookie, doomedId)).body : {};
const doomedError = pgScalar(
  `select payload->>'code' from run_events where submission_id = '${doomedId}' and type = 'error'`
);
check(
  'failure: a worker lost on every attempt ends as internal_error with no verdict',
  doomedTerminal === 'internal_error' &&
    !doomedRead.verdict &&
    doomedError === 'retries_exhausted' &&
    !eventTypes(doomedId).includes('ghost'),
  `kills=${kills} status=${doomedTerminal} verdict=${doomedRead.verdict ?? 'none'} error=${doomedError}`
);

// Leg 11: the user rate limit / queue cap throttles a burst. Every accepted
// run is tiny and finishes on its own; the assertion is on the refusal codes.
const bursts = [];
for (let i = 0; i < 36; i++) {
  const res = await postRun(userA.cookie, RIGHT, 'run');
  bursts.push({
    status: res.status,
    code: res.body?.error?.code ?? res.body?.code ?? '',
    runId: res.body?.run_id
  });
}
const throttled = bursts.filter((b) => b.status === 429);
check(
  'limits: a rapid burst is throttled',
  throttled.length > 0,
  `429s=${throttled.length} codes=${[...new Set(throttled.map((b) => b.code))].join(',')}`
);

// The accepted burst runs drain on their own; nothing after this depends on
// them finishing, so the spec ends without waiting.

// -------------------------------------------------------------------------
const failed = results.filter(([, ok]) => !ok);
console.log(
  `\n${results.length - failed.length}/${results.length} checks passed`
);
process.exit(failed.length === 0 ? 0 : 1);
