// ide-workspace T27 on the real stack: workspace timing marks, lazy loading of Monaco and GSAP,
// and a responsive shell while heavy assets load, at a tablet viewport. Same prerequisites as
// real-api-check.mjs (see README.md). Writes the measured numbers to .scratch/perf-report.json.
// The numbers are one local measurement, not a promise about production.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  BASE,
  createReporter,
  launchBrowser,
  signIn,
  typeSource
} from './support.mjs';

const here = fileURLToPath(new URL('.', import.meta.url));
const { check, finish } = createReporter();
const browser = await launchBrowser();
const context = await browser.newContext({
  viewport: { width: 1024, height: 768 }
});
const page = await context.newPage();

// Which heavy libraries each page pulled in, judged by script content, not by file names.
const scripts = [];
page.on('response', async (r) => {
  if (!r.url().endsWith('.js')) return;
  const body = await r.text().catch(() => '');
  scripts.push({
    url: r.url(),
    path: new URL(page.url()).pathname,
    monaco: /monaco-editor|vs\/editor\/|editor\.main/.test(body),
    gsap: /GreenSock|gsap\.version/.test(body),
    bytes: body.length
  });
});
const heavy = (kind, from) => scripts.slice(from).some((s) => s[kind]);

check('perf: sign-in', await signIn(context, page, 'perf'));

// The landing page and the problem list never load Monaco.
let from = scripts.length;
await page.goto(`${BASE}/`, { waitUntil: 'networkidle' });
check('lazy: the landing page loads no Monaco code', !heavy('monaco', from));
from = scripts.length;
await page.goto(`${BASE}/problems`, { waitUntil: 'networkidle' });
check('lazy: the problem list loads no Monaco code', !heavy('monaco', from));

// The workspace: on a slow link the shell is usable while Monaco is still loading. The
// network is throttled through the DevTools protocol only for this load.
const cdp = await context.newCDPSession(page);
await cdp.send('Network.enable');
await cdp.send('Network.emulateNetworkConditions', {
  offline: false,
  latency: 150,
  downloadThroughput: (1.5 * 1024 * 1024) / 8,
  uploadThroughput: (750 * 1024) / 8
});
from = scripts.length;
await page.goto(`${BASE}/problems/add-two`);
await page.getByRole('button', { name: 'Run samples' }).waitFor();
const whileLoading = await page.evaluate(() => ({
  editorLoading: document.body.innerText.includes('Loading editor'),
  editorReady: performance.getEntriesByName('monaco-ready').length > 0
}));
// The problem drawer button is a control the learner can use right away.
const drawerWorks = await page
  .getByRole('button', { name: /problem/i })
  .first()
  .click({ timeout: 5000 })
  .then(() => true)
  .catch(() => false);
await page.keyboard.press('Escape');
check(
  'responsive: on a 1.5 Mbps link the shell is usable while the editor still loads',
  whileLoading.editorLoading && !whileLoading.editorReady && drawerWorks,
  JSON.stringify({ ...whileLoading, drawerWorks })
);
await page.waitForFunction(
  () => performance.getEntriesByName('monaco-ready').length > 0,
  null,
  { timeout: 120000 }
);
await cdp.send('Network.emulateNetworkConditions', {
  offline: false,
  latency: 0,
  downloadThroughput: -1,
  uploadThroughput: -1
});
check('lazy: Monaco code loads only on the workspace', heavy('monaco', from));
check(
  'lazy: GSAP is not loaded before anything animates',
  !heavy('gsap', from)
);

// A failing submit produces every run and ghost mark.
await page.waitForSelector('.monaco-editor textarea', { state: 'attached' });
await page.waitForFunction(() =>
  document
    .querySelector('.monaco-editor .view-lines')
    ?.textContent?.includes('add_two')
);
await typeSource(page, 'def add_two(a, b):\n    return a * b\n');
await page.keyboard.press('Control+Shift+Enter');
await page
  .getByRole('button', { name: 'Rerun ghost' })
  .waitFor({ timeout: 180000 });
// At this tablet width the results sit behind a drawer; the replay plays when it opens.
await page.getByRole('button', { name: 'Results', exact: true }).click();
await page.waitForFunction(
  () => performance.getEntriesByName('ghost-playback-end').length > 0,
  null,
  { timeout: 10000 }
);
const entries = await page.evaluate(() =>
  performance.getEntriesByType('mark').map((m) => ({
    name: m.name,
    at: Math.round(m.startTime),
    detail: m.detail ?? null
  }))
);
const at = (name) => entries.find((e) => e.name === name)?.at;
const order = [
  'workspace-shell-interactive',
  'problem-detail-loaded',
  'monaco-ready',
  'run-first-event',
  'run-terminal',
  'ghost-playback-start',
  'ghost-playback-end'
];
check(
  'marks: every documented mark is recorded',
  order.every((name) => at(name) !== undefined),
  order.map((n) => `${n}=${at(n)}`).join(' ')
);
check(
  'marks: the run marks follow the editor, and the replay ends after it starts',
  at('monaco-ready') <= at('run-first-event') &&
    at('run-first-event') <= at('run-terminal') &&
    at('ghost-playback-start') <= at('ghost-playback-end')
);
check(
  'marks: no mark carries a detail, the source or an id',
  entries.every((e) => e.detail === null) &&
    !JSON.stringify(entries).includes('a * b') &&
    !/[0-9a-f]{8}-[0-9a-f]{4}-/.test(JSON.stringify(entries))
);

const report = {
  viewport: '1024x768',
  marks: Object.fromEntries(order.map((n) => [n, at(n)])),
  durations: {
    shellToEditorMs: at('monaco-ready') - at('workspace-shell-interactive'),
    firstEventToTerminalMs: at('run-terminal') - at('run-first-event'),
    ghostPlaybackMs: at('ghost-playback-end') - at('ghost-playback-start')
  },
  scripts: scripts.map((s) => ({
    path: s.path,
    file: s.url.split('/').pop(),
    kb: Math.round(s.bytes / 1024),
    monaco: s.monaco,
    gsap: s.gsap
  }))
};
mkdirSync(join(here, '.scratch'), { recursive: true });
writeFileSync(
  join(here, '.scratch', 'perf-report.json'),
  JSON.stringify(report, null, 2)
);
console.log(JSON.stringify(report.durations));

await browser.close();
finish();
