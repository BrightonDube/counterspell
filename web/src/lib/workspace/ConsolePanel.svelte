<script lang="ts">
  // Read-only console surface for bounded public output. Server/compiler text is rendered as
  // ordinary text nodes, never HTML.
  import type { ConsoleEntry } from './console-view';

  let {
    entries,
    open,
    onToggle
  }: {
    entries: readonly ConsoleEntry[];
    open: boolean;
    onToggle: () => void;
  } = $props();

  const statusText = $derived(
    entries.length === 0
      ? 'No console output yet.'
      : `${entries.length} console ${entries.length === 1 ? 'block' : 'blocks'} available.`
  );
</script>

<section
  aria-label="Console"
  class="rounded border border-border bg-surface p-3"
>
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 class="text-base font-semibold text-text">Console</h2>
    <button
      type="button"
      class="min-h-11 rounded border border-border-strong px-4 text-text"
      aria-expanded={open}
      aria-controls="workspace-console-output"
      onclick={onToggle}
    >
      Console
    </button>
  </div>
  <p role="status" class="mt-2 text-sm text-text-muted">{statusText}</p>
  <div
    id="workspace-console-output"
    class="mt-3 flex flex-col gap-3"
    hidden={!open}
  >
    {#each entries as entry (entry.id)}
      <section aria-labelledby={`${entry.id}-heading`}>
        <h3 id={`${entry.id}-heading`} class="text-sm font-medium text-text">
          {entry.heading}
        </h3>
        <pre
          class="mt-1 whitespace-pre-wrap break-all rounded bg-raised p-3 font-mono text-xs text-text">{entry.text}</pre>
      </section>
    {/each}
  </div>
</section>
