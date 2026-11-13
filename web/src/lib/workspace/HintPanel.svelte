<script lang="ts">
  import type { LadderState } from './hint-ladder';

  let { ladder, onNext }: { ladder: LadderState; onNext: () => void } =
    $props();

  const exhausted = $derived(ladder.revealed.length >= ladder.total);
</script>

<!-- Hint text is shown as text nodes only. Nothing from the server is inserted as HTML. -->
<section
  aria-labelledby="hints-heading"
  class="flex flex-col gap-3 rounded border border-border bg-surface p-4"
>
  <h2 id="hints-heading" class="text-lg font-semibold text-text">Hints</h2>

  {#each ladder.revealed as hint (hint.rung)}
    <div class="rounded border border-border-strong bg-raised p-3">
      <p class="text-sm font-semibold text-accent-moss">Hint {hint.rung}</p>
      <p class="text-text">{hint.text}</p>
    </div>
  {/each}

  {#if ladder.error !== null}
    <p role="alert" class="text-sm text-danger">{ladder.error}</p>
  {/if}

  {#if ladder.capReached}
    <p class="text-sm text-text-muted">
      You have reached the daily AI hint limit.
    </p>
  {:else if !exhausted}
    <button
      type="button"
      class="min-h-11 self-start rounded border border-border-strong px-4 text-sm text-text disabled:opacity-60"
      disabled={ladder.pending !== null}
      onclick={onNext}
    >
      {ladder.pending !== null
        ? 'Loading hint…'
        : `Show hint ${ladder.revealed.length + 1}`}
    </button>
  {/if}
</section>
