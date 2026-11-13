<script lang="ts">
  import { buildAutopsy } from '../ghost/autopsy';
  import type { GhostView } from '../ghost/types';

  let { ghost }: { ghost: GhostView | null } = $props();

  const result = $derived(ghost ? buildAutopsy(ghost) : null);
</script>

<!-- Every value is shown as a text node. Nothing here is inserted as HTML. -->
<section
  aria-labelledby="autopsy-heading"
  class="flex flex-col gap-3 rounded border border-border bg-surface p-4"
>
  <h2 id="autopsy-heading" class="text-lg font-semibold text-text">
    Bug Autopsy
  </h2>

  {#if result === null}
    <p class="text-text-subtle">
      A failing submission produces the Bug Autopsy.
    </p>
  {:else if result.status === 'unavailable'}
    <p role="status" class="text-text-muted">{result.reason}</p>
  {:else}
    {@const view = result.view}
    <p class="font-semibold text-accent-moss">{view.headline}</p>
    <p class="text-sm text-text-muted">{view.kindLabel}</p>

    <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 font-mono text-sm">
      <dt class="text-text-muted">Input</dt>
      <dd><code class="break-all">{view.input}</code></dd>
      {#if view.expected !== null}
        <dt class="text-success">Expected</dt>
        <dd class="text-success">
          <code class="break-all">{view.expected}</code>
        </dd>
      {/if}
      {#if view.actual !== null}
        <dt class="text-danger">{view.actualLabel}</dt>
        <dd class="text-danger">
          <code class="break-all">{view.actual}</code>
        </dd>
      {/if}
    </dl>

    <p class="text-text-muted">{view.explanation}</p>

    {#if view.explanationSource.label !== null}
      <p class="font-semibold text-accent-candle">
        {view.explanationSource.label}
      </p>
      <p class="text-text">{view.explanationSource.text}</p>
    {/if}

    {#if view.differences.length > 0}
      <ul class="flex list-none flex-col gap-1 font-mono text-sm">
        {#each view.differences as difference (difference.path)}
          <li>
            {difference.path}: expected {difference.expected}, got {difference.got}
          </li>
        {/each}
      </ul>
    {/if}
    {#if view.differencesTruncated}
      <p class="text-sm text-text-subtle">
        More differences exist. Only the first ones are shown.
      </p>
    {/if}

    {#if view.conceptTags.length > 0}
      <p class="text-sm text-text-subtle">
        Concepts: {view.conceptTags.join(', ')}
      </p>
    {/if}
  {/if}
</section>
