<script lang="ts">
  // The workspace layout. Desktop (1200px+) shows the problem beside the editor. Tablet
  // (768-1199px) keeps the editor visible and moves the problem and results behind
  // drawers with a persistent status strip. Narrow (<768px) stacks the regions in
  // reading order with the problem first and collapsible. The results region shows the
  // live run, or the last finished run until its replacement finishes. The shell never
  // invents a result.
  import { onDestroy, untrack } from 'svelte';
  import type { LanguageSummary, ProblemDetail } from '../api/types';
  import {
    createSpeechGuard,
    loadPreferences,
    requestSpeech,
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
  import Familiar from '../familiar/Familiar.svelte';
  import { runOutcomeLabel, statusLabel } from './test-rows';
  import AutopsyCard from './AutopsyCard.svelte';
  import EditorPanel from './EditorPanel.svelte';
  import GhostPanel from './GhostPanel.svelte';
  import ProblemPanel from './ProblemPanel.svelte';
  import StatusAnnouncer from './StatusAnnouncer.svelte';
  import TestPanel from './TestPanel.svelte';
  import {
    createLayoutMode,
    type LayoutMode
  } from './layout-mode.svelte.js';
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
    sessionDeps = browserRunSessionDeps,
    familiarDeps = { loadPreferences, savePreferences, requestSpeech }
  }: {
    problem: ProblemDetail;
    languages: LanguageSummary[];
    layout?: LayoutMode;
    reducedMotion?: boolean;
    sessionDeps?: RunSessionDeps;
    familiarDeps?: FamiliarDeps;
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
  // Session deps are injected once at mount; swapping them mid-run is not supported.
  const session = createRunSession(untrack(() => sessionDeps), (next) => {
    runs = next;
  });
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
  let prefs = $state<FamiliarPrefs>({ muted: false, hidden: false });
  let prefsTouched = false;
  $effect(() => {
    untrack(() => familiarDeps)
      .loadPreferences()
      .then((loaded) => {
        if (loaded !== null && !prefsTouched) prefs = loaded;
      })
      .catch(() => {});
  });

  function onPrefsChange(next: FamiliarPrefs): void {
    prefs = next;
    prefsTouched = true;
    familiarDeps.savePreferences(next).catch(() => {});
  }

  // One speech request per finished run. The guard covers remounts and reconnects.
  const speechGuard = createSpeechGuard();
  let speech = $state<SpeechResult | null>(null);
  let speechRunId = '';
  $effect(() => {
    if (runs.phase !== 'finished' || runs.runId === undefined) return;
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

  function requestRun(request: RunRequest): void {
    const ghost = request.ghost ? runs.stream?.ghost : undefined;
    session.start({
      problem: problem.slug,
      lang: request.lang,
      code: request.code,
      mode: request.mode,
      ghostInput: ghost?.input
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
  <StatusAnnouncer
    message={announcement}
    label="Run announcements"
    srOnly
  />
  {#if runs.error}
    <p role="alert" class="text-sm text-danger">{runs.error}</p>
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
  <Familiar
    measured={familiar}
    {prefs}
    {speech}
    reducedMotion={motion.value}
    {onPrefsChange}
  />
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
    onRequest={requestRun}
  />
{/snippet}

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
