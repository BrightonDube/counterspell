<script lang="ts">
  // The workspace layout. Wide screens show the problem beside the editor. Narrow screens stack the regions
  // in reading order, with the problem first and collapsible. The results region shows the live run, or the
  // last finished run until its replacement finishes. The shell never invents a result.
  import { onDestroy } from 'svelte';
  import type { LanguageSummary, ProblemDetail } from '../api/types';
  import { statusLabel } from './test-rows';
  import EditorPanel from './EditorPanel.svelte';
  import ProblemPanel from './ProblemPanel.svelte';
  import TestPanel from './TestPanel.svelte';
  import {
    browserRunSessionDeps,
    createRunSession,
    type RunRequest,
    type RunSessionSnapshot
  } from './run-session';

  let {
    problem,
    languages
  }: { problem: ProblemDetail; languages: LanguageSummary[] } = $props();

  let problemOpen = $state(true);
  let runs = $state<RunSessionSnapshot>({ phase: 'idle' });
  const session = createRunSession(browserRunSessionDeps, (next) => {
    runs = next;
  });
  const inProgress = $derived(
    runs.phase === 'starting' ||
      runs.phase === 'streaming' ||
      runs.phase === 'loading'
  );

  onDestroy(() => session.cancel());

  function requestRun(request: RunRequest): void {
    session.start({
      problem: problem.slug,
      lang: request.lang,
      code: request.code,
      mode: request.mode
    });
  }
</script>

<div class="grid gap-4 lg:grid-cols-[minmax(16rem,22rem)_minmax(0,1fr)]">
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
      <ProblemPanel {problem} />
    </aside>
  </div>

  <div class="flex min-w-0 flex-col gap-4">
    <EditorPanel
      {problem}
      {languages}
      busy={inProgress}
      onRequest={requestRun}
    />
    <section
      aria-label="Results"
      class="flex flex-col gap-3 rounded border border-border bg-surface p-4"
    >
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
    </section>
  </div>
</div>
