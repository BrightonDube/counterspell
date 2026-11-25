<script lang="ts">
  // The workspace layout. Desktop (1200px+) shows the problem beside the editor. Tablet
  // (768-1199px) keeps the editor visible and moves the problem and results behind
  // drawers with a persistent status strip. Narrow (<768px) stacks the regions in
  // reading order with the problem first and collapsible. The results region shows the
  // live run, or the last finished run until its replacement finishes. The shell never
  // invents a result.
  import { onDestroy, onMount, untrack } from 'svelte';
  import { createRunMarks, markWorkspace } from '../performance';
  import type { LanguageSummary, ProblemDetail } from '../api/types';
  import {
    createSpeechGuard,
    loadPreferences,
    savePreferences,
    type FamiliarDeps,
    type FamiliarPrefs,
    type SpeechResult
  } from '../api/familiar';
  import {
    applyComplexity,
    applyMeasuredFamiliar,
    applyTerminal,
    initialFamiliarState,
    type FamiliarState
  } from '../familiar/familiar-state';
  import { requestAuthoredHint, type AuthoredHint } from '../api/hints';
  import { requestSpeechOnce } from '../familiar/bubble/request';
  // Familiar loads as a stage-4 atmosphere chunk: it mounts only after the
  // editor is usable, so its code and media never sit on the critical path.
  let familiarChunk: ReturnType<typeof importFamiliar> | undefined;
  function importFamiliar() {
    return import('../familiar/Familiar.svelte');
  }
  function loadFamiliar() {
    return (familiarChunk ??= importFamiliar());
  }
  import { runOutcomeLabel, statusLabel } from './test-rows';
  import { session as authSession } from '../auth/session.svelte.js';
  import AutopsyCard from './AutopsyCard.svelte';
  import ConsolePanel from './ConsolePanel.svelte';
  import { buildConsoleEntries } from './console-view';
  import EditorPanel from './EditorPanel.svelte';
  import GhostPanel from './GhostPanel.svelte';
  import HintPanel from './HintPanel.svelte';
  import {
    failRung,
    initialLadder,
    receiveRung,
    requestNext,
    type LadderState
  } from './hint-ladder';
  import ProblemPanel from './ProblemPanel.svelte';
  import StatusAnnouncer from './StatusAnnouncer.svelte';
  import TestPanel from './TestPanel.svelte';
  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import { createLayoutMode, type LayoutMode } from './layout-mode.svelte.js';
  import { createReducedMotion } from './reduced-motion.svelte.js';
  import {
    browserRunSessionDeps,
    createRunSession,
    type RunRequest,
    type RunSessionDeps,
    type RunSessionSnapshot
  } from './run-session';

  let {
    problem,
    languages,
    layout,
    reducedMotion,
    atmosphere,
    sessionDeps = browserRunSessionDeps,
    familiarDeps = {
      loadPreferences,
      savePreferences,
      requestSpeech: requestSpeechOnce
    },
    requestHint = requestAuthoredHint
  }: {
    problem: ProblemDetail;
    languages: LanguageSummary[];
    layout?: LayoutMode;
    reducedMotion?: boolean;
    /** Force the stage-4 atmosphere (Familiar, guess bubble) on. Tests inject
     * this; production derives it from the `monaco-ready` mark. */
    atmosphere?: boolean;
    sessionDeps?: RunSessionDeps;
    familiarDeps?: FamiliarDeps;
    /** Asks for one authored hint rung. Tests inject a fake. */
    requestHint?: (submissionId: string, rung: number) => Promise<AuthoredHint>;
  } = $props();

  const detected = createLayoutMode();
  const mode = $derived(layout ?? detected.mode);
  // The injected pins only seed the detectors; live values come from the media query.
  const motion = createReducedMotion(untrack(() => reducedMotion));

  let problemOpen = $state(true);
  let drawer = $state<'problem' | 'results' | null>(null);
  let problemTrigger = $state<HTMLButtonElement>();
  let resultsTrigger = $state<HTMLButtonElement>();
  let problemDrawerEl = $state<HTMLElement>();
  let resultsDrawerEl = $state<HTMLElement>();

  let runs = $state<RunSessionSnapshot>({ phase: 'idle' });
  let consoleOpen = $state(false);
  let consoleRunId = $state<string | undefined>(undefined);
  let consoleAutoOpened = false;
  let consoleCollapsed = false;
  const consoleEntries = $derived(
    buildConsoleEntries({
      tests:
        runs.phase === 'finished' && runs.view
          ? runs.view.tests
          : (runs.stream?.tests ?? runs.view?.tests ?? []),
      terminal: runs.stream?.terminal
    })
  );
  // Session deps are injected once at mount; swapping them mid-run is not supported.
  const session = createRunSession(
    untrack(() => sessionDeps),
    (next) => {
      runs = next;
    }
  );
  const inProgress = $derived(
    runs.phase === 'starting' ||
      runs.phase === 'streaming' ||
      runs.phase === 'loading'
  );
  const statusStrip = $derived(
    inProgress
      ? statusLabel(runs.stream?.status ?? 'queued')
      : runs.phase === 'finished' && runs.view
        ? statusLabel(runs.view.status)
        : 'No run yet'
  );

  $effect(() => {
    if (runs.runId !== consoleRunId) {
      consoleRunId = runs.runId;
      consoleOpen = false;
      consoleAutoOpened = false;
      consoleCollapsed = false;
    }
    if (
      runs.runId !== undefined &&
      runs.phase === 'finished' &&
      consoleEntries.length > 0 &&
      !consoleAutoOpened &&
      !consoleCollapsed
    ) {
      consoleOpen = true;
      consoleAutoOpened = true;
    }
  });

  function toggleConsole(): void {
    if (consoleOpen) {
      consoleOpen = false;
      consoleCollapsed = true;
    } else {
      consoleOpen = true;
    }
  }

  // Polite announcements for the phase changes a screen-reader user would miss.
  // Identical text never repeats, so only transitions speak.
  const announcement = $derived.by((): string => {
    switch (runs.phase) {
      case 'starting':
        return 'Starting the run.';
      case 'streaming':
        return statusLabel(runs.stream?.status ?? 'queued');
      case 'loading':
        return 'Fetching the result.';
      case 'finished':
        return runs.stream?.terminal
          ? `Finished: ${runOutcomeLabel(runs.stream.terminal)}.`
          : 'Finished.';
      case 'failed':
        return 'The run could not be started.';
      default:
        return '';
    }
  });

  onDestroy(() => session.cancel());

  // Stage-4 atmosphere waits for stage 2: the Familiar, its media and the
  // preference fetch mount only once the editor is usable. The performance mark
  // is the ordering signal so no props have to be drilled; a generous fallback
  // keeps the mascot reachable even if the editor never reports ready.
  let atmosphereReady = $state(false);
  const atmosphereOn = $derived(atmosphere ?? atmosphereReady);
  onMount(() => {
    if (atmosphere || typeof PerformanceObserver === 'undefined') {
      // Forced (tests) and environments without the API get the atmosphere eagerly.
      atmosphereReady = true;
      return;
    }
    if (performance.getEntriesByName('monaco-ready').length > 0) {
      atmosphereReady = true;
      return;
    }
    const observer = new PerformanceObserver((list) => {
      if (list.getEntries().some((e) => e.name === 'monaco-ready')) {
        atmosphereReady = true;
        observer.disconnect();
      }
    });
    observer.observe({ type: 'mark' });
    const fallback = setTimeout(() => (atmosphereReady = true), 30_000);
    return () => {
      observer.disconnect();
      clearTimeout(fallback);
    };
  });

  // Timing marks for the browser performance check. Fixed names only; no run id or source.
  onMount(() => markWorkspace('workspace-shell-interactive'));
  const runMarks = createRunMarks();
  $effect(() => {
    const runId = runs.runId;
    if (runId === undefined) return;
    if ((runs.stream?.lastSeq ?? 0) > 0) runMarks.event(runId);
    if (runs.phase === 'finished') runMarks.terminal(runId);
  });

  const canRerunGhost = $derived(runs.stream?.ghost !== undefined);

  // The measured-only Familiar state. Only stream facts reach it; the mood can
  // never be guessed from edits or speech.
  const familiar = $derived.by((): FamiliarState => {
    const stream = runs.stream;
    let state = initialFamiliarState();
    if (!stream) return state;
    if (stream.complexity) state = applyComplexity(state, stream.complexity);
    if (stream.familiar) state = applyMeasuredFamiliar(state, stream.familiar);
    if (stream.terminal?.kind === 'done') {
      state = applyTerminal(state, {
        verdict: stream.terminal.verdict,
        hasBench: stream.benchSizes.length > 0
      });
    }
    return state;
  });

  // Mute/hide choices. Saved through the API; a failed save keeps the local choice.
  // Signed-out visits never ask — the 401 is expected noise, and the request is
  // not part of the shell anyway, so it waits for the atmosphere stage. A direct
  // workspace visit has no AppShell to resolve the store, so an `unknown` status
  // is asked once here — the probe endpoint answers 200 signed in or out.
  let prefs = $state<FamiliarPrefs>({ muted: false, hidden: false });
  // Per-field touches: a toggle that beats the pending read wins that field, and
  // the late read still applies the fields the learner never touched.
  const prefsTouched = { muted: false, hidden: false };
  $effect(() => {
    if (!atmosphereOn) return;
    if (authSession.status === 'unknown') {
      authSession.load().catch(() => {});
      return;
    }
    if (authSession.status !== 'signed_in') return;
    untrack(() => familiarDeps)
      .loadPreferences()
      .then((loaded) => {
        if (loaded === null) return;
        prefs = {
          ...loaded,
          muted: prefsTouched.muted ? prefs.muted : loaded.muted,
          hidden: prefsTouched.hidden ? prefs.hidden : loaded.hidden
        };
      })
      .catch(() => {});
  });

  function onPrefsChange(next: FamiliarPrefs): void {
    // The server merges every named field, so a toggle may only send the field
    // it changed — otherwise a pending read's default would clobber the saved
    // sibling preference.
    const changed: Partial<FamiliarPrefs> = {};
    if (next.muted !== prefs.muted) {
      changed.muted = next.muted;
      prefsTouched.muted = true;
    }
    if (next.hidden !== prefs.hidden) {
      changed.hidden = next.hidden;
      prefsTouched.hidden = true;
    }
    if (changed.muted === undefined && changed.hidden === undefined) return;
    prefs = { ...prefs, ...changed };
    // A signed-out toggle stays local — the save endpoint needs a session.
    if (authSession.status !== 'signed_in') return;
    familiarDeps.savePreferences(changed).catch(() => {});
  }

  // One speech request per finished run. The guard covers remounts and reconnects.
  const speechGuard = createSpeechGuard();
  let speech = $state<SpeechResult | null>(null);
  let speechRunId = '';
  $effect(() => {
    if (runs.phase !== 'finished' || runs.runId === undefined) return;
    // The speech endpoint needs a session — a signed-out finish never asks.
    if (authSession.status !== 'signed_in') return;
    // Speech off means the page never asks, so the code is not sent anywhere.
    if (prefs.speech === 'off') return;
    const runId = runs.runId;
    if (runId === speechRunId) return;
    if (!speechGuard.claim(runId)) return;
    speech = null;
    speechRunId = runId;
    untrack(() => familiarDeps)
      .requestSpeech(runId)
      .then((result) => {
        if (speechRunId === runId) speech = result;
      })
      .catch(() => {});
  });

  // The authored ladder belongs to one finished run. A new run starts a new ladder; the server
  // keeps what was already revealed, so asking again for a rung is safe.
  const HINT_RUNGS = 4;
  let ladder = $state<LadderState>(initialLadder(HINT_RUNGS));
  let ladderRunId = '';
  $effect(() => {
    const runId = runs.phase === 'finished' ? runs.runId : undefined;
    if (runId === undefined || runId === ladderRunId) return;
    ladderRunId = runId;
    ladder = initialLadder(HINT_RUNGS);
  });

  async function onNextHint(): Promise<void> {
    // Only ask for the ladder the page is showing. A click before the ladder has switched to a
    // newly finished run does nothing, so no rung is left pending without a request.
    const runId = runs.phase === 'finished' ? runs.runId : undefined;
    if (runId === undefined || runId !== ladderRunId) return;
    const next = requestNext(ladder);
    ladder = next.state;
    if (next.rung === null) return;
    const rung = next.rung;
    try {
      const hint = await requestHint(runId, rung);
      if (runId === ladderRunId) ladder = receiveRung(ladder, rung, hint.text);
    } catch {
      // A fixed line: the server's own message is never shown here.
      if (runId === ladderRunId) {
        ladder = failRung(
          ladder,
          rung,
          'This hint could not be loaded. Try again.'
        );
      }
    }
  }

  function requestRun(request: RunRequest): void {
    const ghost = request.ghost ? runs.stream?.ghost : undefined;
    session.start({
      problem: problem.slug,
      lang: request.lang,
      code: request.code,
      mode: request.mode,
      ghostInput: ghost?.input,
      ghostSourceId: ghost === undefined ? undefined : runs.stream?.runId
    });
  }

  function focusFirst(el: HTMLElement): void {
    const target = el.querySelector<HTMLElement>(
      'button, [href], input, select, [tabindex]:not([tabindex="-1"])'
    );
    (target ?? el).focus();
  }

  const drawerEls = {
    problem: () => problemDrawerEl,
    results: () => resultsDrawerEl
  } as const;
  const triggers = {
    problem: () => problemTrigger,
    results: () => resultsTrigger
  } as const;

  function toggleDrawer(which: 'problem' | 'results'): void {
    const closing = drawer === which;
    drawer = closing ? null : which;
    if (closing) triggers[which]()?.focus();
  }

  function closeDrawer(which: 'problem' | 'results'): void {
    drawer = null;
    triggers[which]()?.focus();
  }

  function drawerKeydown(
    which: 'problem' | 'results',
    event: KeyboardEvent
  ): void {
    if (event.key === 'Escape') {
      event.stopPropagation();
      closeDrawer(which);
    }
  }

  $effect(() => {
    if (drawer !== null) {
      const el = drawerEls[drawer]();
      if (el) focusFirst(el);
    }
  });
</script>

{#snippet problemContent()}
  <ProblemPanel {problem} />
{/snippet}

{#snippet resultsContent()}
  <StatusAnnouncer message={announcement} label="Run announcements" srOnly />
  {#if runs.error}
    <p role="alert" class="text-sm text-danger">
      {#if runs.errorCode === 'unauthenticated'}
        <a
          class="underline underline-offset-4 hover:text-text"
          href={resolve(
            `/login?return_to=${encodeURIComponent(page.url.pathname)}`
          )}
        >
          {runs.error}
        </a>
      {:else}
        {runs.error}
      {/if}
    </p>
  {/if}
  {#if inProgress}
    <p role="status" class="text-sm text-text-muted">
      {statusLabel(runs.stream?.status ?? 'queued')}
    </p>
  {/if}
  {#if runs.phase === 'finished' && runs.view}
    <TestPanel
      status={runs.view.status}
      tests={runs.view.tests}
      terminal={runs.stream?.terminal}
    />
  {:else if runs.view && inProgress}
    <TestPanel status={runs.view.status} tests={runs.view.tests} />
  {:else if runs.stream}
    <TestPanel
      status={runs.stream.status}
      tests={runs.stream.tests}
      terminal={runs.stream.terminal}
    />
  {:else if runs.view}
    <TestPanel status={runs.view.status} tests={runs.view.tests} />
  {:else}
    <p class="text-sm text-text-muted">
      No run yet. Run samples or submit to see results here.
    </p>
  {/if}
  {#if runs.stream}
    <GhostPanel
      status={runs.stream.status}
      ghost={runs.stream.ghost ?? null}
      shrinkSteps={runs.stream.shrinkSteps}
      reducedMotion={motion.value}
    />
    <AutopsyCard ghost={runs.stream.ghost ?? null} />
  {/if}
  {#if runs.phase === 'finished' && runs.runId !== undefined}
    <HintPanel {ladder} onNext={onNextHint} />
  {/if}
  {#if atmosphereOn}
    {#await loadFamiliar() then { default: Familiar }}
      <Familiar
        measured={familiar}
        {prefs}
        {speech}
        reducedMotion={motion.value}
        {onPrefsChange}
      />
    {/await}
  {/if}
{/snippet}

{#snippet problemAside()}
  <div class="flex flex-col gap-3">
    <button
      type="button"
      class="min-h-11 self-start rounded border border-border-strong px-4 text-text"
      aria-expanded={problemOpen}
      aria-controls="problem-region"
      onclick={() => (problemOpen = !problemOpen)}
    >
      {problemOpen ? 'Hide problem' : 'Show problem'}
    </button>
    <aside
      id="problem-region"
      aria-label="Problem"
      class="rounded border border-border bg-surface p-4"
      hidden={!problemOpen}
    >
      {@render problemContent()}
    </aside>
  </div>
{/snippet}

{#snippet editor()}
  <EditorPanel
    {problem}
    {languages}
    busy={inProgress}
    {canRerunGhost}
    atmosphere={atmosphereOn}
    onRequest={requestRun}
  />
  <ConsolePanel
    entries={consoleEntries}
    open={consoleOpen}
    onToggle={toggleConsole}
  />
{/snippet}

<!-- Costume layer: one candle-glow hairline along the frame's top edge. Pure
     decoration — aria-hidden, pointer-events none, and no script ever queries
     it, so its absence can never produce a console warning. -->
<div class="workspace-ember" aria-hidden="true"></div>

{#if mode === 'tablet'}
  <div class="flex flex-col gap-4">
    <p
      role="status"
      aria-label="Run status"
      class="rounded border border-border bg-surface px-4 py-3 text-sm text-text"
    >
      {statusStrip}
    </p>
    <div class="flex gap-3">
      <button
        type="button"
        bind:this={problemTrigger}
        class="min-h-11 rounded border border-border-strong px-4 text-text"
        aria-expanded={drawer === 'problem'}
        aria-controls="problem-drawer"
        onclick={() => toggleDrawer('problem')}
      >
        Problem
      </button>
      <button
        type="button"
        bind:this={resultsTrigger}
        class="min-h-11 rounded border border-border-strong px-4 text-text"
        aria-expanded={drawer === 'results'}
        aria-controls="results-drawer"
        onclick={() => toggleDrawer('results')}
      >
        Results
      </button>
    </div>
    {@render editor()}
    {#if drawer === 'problem'}
      <div
        id="problem-drawer"
        role="dialog"
        aria-label="Problem"
        tabindex="-1"
        bind:this={problemDrawerEl}
        onkeydown={(event) => drawerKeydown('problem', event)}
        class="rounded border border-border bg-surface p-4"
      >
        <div class="mb-3 flex justify-end">
          <button
            type="button"
            class="min-h-11 rounded border border-border-strong px-4 text-text"
            onclick={() => closeDrawer('problem')}
          >
            Close
          </button>
        </div>
        {@render problemContent()}
      </div>
    {:else if drawer === 'results'}
      <div
        id="results-drawer"
        role="dialog"
        aria-label="Results"
        tabindex="-1"
        bind:this={resultsDrawerEl}
        onkeydown={(event) => drawerKeydown('results', event)}
        class="rounded border border-border bg-surface p-4"
      >
        <div class="mb-3 flex justify-end">
          <button
            type="button"
            class="min-h-11 rounded border border-border-strong px-4 text-text"
            onclick={() => closeDrawer('results')}
          >
            Close
          </button>
        </div>
        {@render resultsContent()}
      </div>
    {/if}
  </div>
{:else if mode === 'narrow'}
  <div class="flex min-w-0 flex-col gap-4">
    {@render problemAside()}
    {@render editor()}
    <section
      aria-label="Results"
      class="flex flex-col gap-3 rounded border border-border bg-surface p-4"
    >
      {@render resultsContent()}
    </section>
  </div>
{:else}
  <div class="grid gap-4 grid-cols-[minmax(16rem,22rem)_minmax(0,1fr)]">
    {@render problemAside()}
    <div class="flex min-w-0 flex-col gap-4">
      {@render editor()}
      <section
        aria-label="Results"
        class="flex flex-col gap-3 rounded border border-border bg-surface p-4"
      >
        {@render resultsContent()}
      </section>
    </div>
  </div>
{/if}
