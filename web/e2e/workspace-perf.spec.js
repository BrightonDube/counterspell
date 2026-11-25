// Workspace performance contract (workspace-performance-costume WP01/WP02/WP08).
// Measures the problem workspace under a fixed mobile profile, writes a JSON
// artifact for before/after diffs, and asserts the readiness ordering the spec
// promises: shell and editor first, tools and atmosphere after interactivity.
//
// Fixed profile (repeatable): 390x844 viewport, CDP Fast-3G-equivalent shaping
// (150 ms RTT, 1.6 Mbps down / 750 kbps up) and 4x CPU throttle — the same
// numbers Lighthouse mobile uses. Output: PERF_OUT or
// ../.kiro/specs/workspace-performance-costume/baseline-latest.json
import { expect, test } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const URL_UNDER_TEST = '/problems/two-sum';
const OUT_FILE =
  process.env.PERF_OUT ??
  resolve(
    import.meta.dirname,
    '../../.kiro/specs/workspace-performance-costume/baseline-latest.json'
  );

// Ceilings the regression tests enforce, on the fixed profile above.
const SHELL_INTERACTIVE_MS = 3000;
// Monaco's editor core is ~2.64 MB of script — the floor for `monaco-ready`.
// The 3.0 MB ceiling leaves room for the app shell (~260 kB measured) while
// still catching eagerly re-added language packs or contribs (~+500 kB each).
const SCRIPT_BYTES_BEFORE_EDITOR = 3_000_000;
// The shell's own critical path is cheap: entry + layout + route chunks
// measured ≈172 kB. A 350 kB ceiling catches route-graph bloat early.
const SCRIPT_BYTES_BEFORE_SHELL = 350_000;

/** Instruments one page under the fixed mobile profile and returns its waterfall. */
async function measure(page, context, url) {
  const cdp = await context.newCDPSession(page);
  await cdp.send('Network.enable');
  await cdp.send('Network.emulateNetworkConditions', {
    offline: false,
    latency: 150,
    downloadThroughput: (1.6 * 1024 * 1024) / 8,
    uploadThroughput: (750 * 1024) / 8
  });
  await cdp.send('Emulation.setCPUThrottlingRate', { rate: 4 });

  const requests = new Map();
  const finished = [];
  cdp.on('Network.requestWillBeSent', (e) => {
    requests.set(e.requestId, {
      url: e.request.url,
      type: e.type,
      wallTime: e.wallTime
    });
  });
  cdp.on('Network.loadingFinished', (e) => {
    const req = requests.get(e.requestId);
    if (req) finished.push({ ...req, bytes: e.encodedDataLength });
  });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.addInitScript(() => {
    window.__cls = 0;
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (!entry.hadRecentInput) window.__cls += entry.value;
      }
    }).observe({ type: 'layout-shift', buffered: true });
    window.__navStart = performance.timeOrigin;
  });

  await page.goto(url, { waitUntil: 'networkidle' });
  // Monaco loads after first paint; wait for its mark rather than a fixed delay.
  await page
    .waitForFunction(
      () => performance.getEntriesByName('monaco-ready').length > 0,
      undefined,
      {
        timeout: 60_000
      }
    )
    .catch(() => {});
  await page.waitForLoadState('networkidle').catch(() => {});

  const measured = await page.evaluate(() => {
    const marks = Object.fromEntries(
      performance
        .getEntriesByType('mark')
        .map((m) => [m.name, Math.round(m.startTime)])
    );
    const nav = performance.getEntriesByType('navigation')[0];
    return {
      marks,
      navStartEpoch: performance.timeOrigin,
      navigation: nav
        ? {
            responseEnd: Math.round(nav.responseEnd),
            domContentLoaded: Math.round(nav.domContentLoadedEventEnd),
            load: Math.round(nav.loadEventEnd)
          }
        : null,
      cls: window.__cls
    };
  });
  return { finished, requests, measured };
}

function writeBaseline(finished, measured) {
  const byType = {};
  let totalBytes = 0;
  for (const r of finished) {
    totalBytes += r.bytes;
    byType[r.type] = (byType[r.type] ?? 0) + r.bytes;
  }
  const report = {
    profile: {
      viewport: '390x844',
      network: '150ms RTT, 1.6Mbps down, 750kbps up',
      cpuThrottle: '4x'
    },
    url: URL_UNDER_TEST,
    measured,
    totals: { requests: finished.length, bytes: totalBytes, byType },
    apiCalls: finished
      .filter((r) => r.url.includes('/api/'))
      .map((r) => ({ url: new URL(r.url).pathname, bytes: r.bytes })),
    // Start offsets relative to navigation — the critical-path evidence WP01
    // needs: what the editor shell waits on, in order.
    waterfall: finished
      .sort((a, b) => a.wallTime - b.wallTime)
      .map((r) => ({
        start: Math.round((r.wallTime - measured.navStartEpoch / 1000) * 1000),
        type: r.type,
        bytes: r.bytes,
        url: r.url.startsWith('http') ? new URL(r.url).pathname : r.url
      })),
    topTransfers: finished
      .sort((a, b) => b.bytes - a.bytes)
      .slice(0, 15)
      .map((r) => ({
        url: r.url.split('/').pop(),
        type: r.type,
        bytes: r.bytes
      }))
  };
  mkdirSync(dirname(OUT_FILE), { recursive: true });
  writeFileSync(OUT_FILE, JSON.stringify(report, null, 2));
  return report;
}

test('workspace readiness ordering under mobile shaping', async ({
  page,
  context
}) => {
  // measure() waits on monaco-ready which lands ~18s under the mobile profile.
  test.setTimeout(120_000);
  const { finished, requests, measured } = await measure(
    page,
    context,
    URL_UNDER_TEST
  );
  const report = writeBaseline(finished, measured);
  // eslint-disable-next-line no-console -- the artifact summary belongs in test output
  console.log(
    `workspace: ${finished.length} requests, ${(report.totals.bytes / 1024).toFixed(0)}k, ` +
      `CLS ${measured.cls?.toFixed(3) ?? '?'}, marks ${JSON.stringify(measured.marks)}`
  );

  // Sanity: the marks this contract leans on exist.
  expect(measured.marks['workspace-shell-interactive']).toBeDefined();
  expect(measured.marks['problem-detail-loaded']).toBeDefined();
  expect(measured.marks['monaco-ready']).toBeDefined();

  // Stage 1: the shell — problem text, editor geometry and controls — is the
  // agreed TTI proxy, and must land under three seconds on this profile.
  expect(
    measured.marks['workspace-shell-interactive'],
    `shell interactive at ${measured.marks['workspace-shell-interactive']}ms > ${SHELL_INTERACTIVE_MS}ms`
  ).toBeLessThan(SHELL_INTERACTIVE_MS);

  // wallTime (CDP) is epoch seconds; timeOrigin/marks are milliseconds.
  const navStartSec = measured.navStartEpoch / 1000;
  const editorReadyEpoch = navStartSec + measured.marks['monaco-ready'] / 1000;
  const shellEpoch =
    navStartSec + measured.marks['workspace-shell-interactive'] / 1000;

  // Stage 1 budget: scripts fetched for the shell itself stay small — the
  // heavy Monaco download belongs to stage 2, not the interactive mark.
  const shellScriptBytes = finished
    .filter((r) => r.wallTime <= shellEpoch && r.type === 'Script')
    .reduce((sum, r) => sum + r.bytes, 0);
  expect(
    shellScriptBytes,
    `${(shellScriptBytes / 1024).toFixed(0)}kB of script before the shell mark`
  ).toBeLessThan(SCRIPT_BYTES_BEFORE_SHELL);
  // Ordering is judged on when requests STARTED — a still-streaming video never
  // reaches loadingFinished, so check every observed request, not only finished
  // ones.
  const startedBeforeEditor = [...requests.values()].filter(
    (r) => r.wallTime <= editorReadyEpoch
  );

  // Stage 4 atmosphere stays off the critical path: no familiar media may be
  // fetched before the editor is usable.
  expect(
    startedBeforeEditor
      .filter((r) => r.url.includes('/art/familiar/'))
      .map((r) => r.url),
    'familiar media fetched before the editor was ready'
  ).toEqual([]);

  // Stage 3 tools stay off the critical path: per-user preferences are not a
  // shell concern.
  expect(
    startedBeforeEditor
      .filter((r) => r.url.includes('/api/me/familiar'))
      .map((r) => r.url),
    'familiar preferences fetched before the editor was ready'
  ).toEqual([]);

  // Optional editor features (extra language packs, find, comment) load after
  // the editor is usable — cap the script bytes that can land before the mark.
  // Finished responses carry bytes; scripts always finish before they run.
  const scriptBytes = finished
    .filter((r) => r.wallTime <= editorReadyEpoch && r.type === 'Script')
    .reduce((sum, r) => sum + r.bytes, 0);
  expect(
    scriptBytes,
    `${(scriptBytes / 1048576).toFixed(2)}MB of script before monaco-ready`
  ).toBeLessThan(SCRIPT_BYTES_BEFORE_EDITOR);
});

test('workspace controls still run and answer under the deferred schedule', async ({
  browser
}) => {
  // Runs need a session: the fixture accepts Bearer identities (same scheme as
  // dashboard.spec.js). Slots avoid reusing a spent per-user run window.
  const user = 9 + (Math.floor(Date.now() / 1000) % 2000) * 2;
  const context = await browser.newContext({
    extraHTTPHeaders: { authorization: `Bearer test-user-${user}` }
  });
  const page = await context.newPage();
  await page.goto(URL_UNDER_TEST);
  // The editor's run path is stage 1/2 function: it must work the moment the
  // controls are visible, regardless of what deferred behind it.
  await page.getByRole('button', { name: 'Run samples' }).click();
  await expect(page.getByRole('region', { name: 'Results' })).toContainText(
    /queued|compiling|running|accepted|rejected|wrong answer/i,
    {
      timeout: 30_000
    }
  );
  await context.close();
});
