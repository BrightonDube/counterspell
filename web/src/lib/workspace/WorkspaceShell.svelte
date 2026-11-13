<script lang="ts">
  // The workspace layout. Wide screens show the problem beside the editor. Narrow screens stack the regions
  // in reading order, with the problem first and collapsible. Results are a placeholder until the run
  // lifecycle is connected; the shell never invents a result.
  import type { LanguageSummary, ProblemDetail } from '../api/types';
  import EditorPanel from './EditorPanel.svelte';
  import ProblemPanel from './ProblemPanel.svelte';

  let {
    problem,
    languages
  }: { problem: ProblemDetail; languages: LanguageSummary[] } = $props();

  let problemOpen = $state(true);
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
    <EditorPanel {problem} {languages} />
    <section
      aria-label="Results"
      class="rounded border border-border bg-surface p-4"
    >
      <p class="text-sm text-text-muted">
        No run yet. Run samples or submit to see results here.
      </p>
    </section>
  </div>
</div>
