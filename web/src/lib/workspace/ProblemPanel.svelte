<script lang="ts">
  // Problem statement panel. Every value is rendered as text. The statement is parsed into plain
  // blocks, so no markup from the server or a pack can reach the DOM.
  import type { ProblemDetail } from '../api/types';
  import { parseStatement, type InlineSegment } from './statement';

  let { problem }: { problem: ProblemDetail } = $props();

  const blocks = $derived(parseStatement(problem.statementMarkdown));
  const signature = $derived(
    `${problem.signature.functionName}(${problem.signature.parameters
      .map((parameter) => `${parameter.name}: ${parameter.type}`)
      .join(', ')}) -> ${problem.signature.returnType}`
  );

  let statementOpen = $state(true);
  let examplesOpen = $state(true);

  /** Shows a sample value as JSON text. It is data, never markup. */
  function formatValue(value: unknown): string {
    return JSON.stringify(value);
  }

  function segmentKey(segment: InlineSegment, index: number): string {
    return `${index}-${segment.code}`;
  }
</script>

<section aria-labelledby="problem-title" class="flex flex-col gap-4 text-text">
  <nav aria-label="Breadcrumb" class="text-sm text-text-muted">
    <span>{problem.track.name}</span>
  </nav>

  <header class="flex flex-col gap-2">
    <h2 id="problem-title" class="text-xl font-semibold">{problem.title}</h2>
    <p class="text-sm text-text-muted">
      Difficulty: <span class="text-text">{problem.difficulty}</span>
    </p>
    {#if problem.concepts.length > 0}
      <ul aria-label="Concepts" class="flex flex-wrap gap-2">
        {#each problem.concepts as concept (concept.slug)}
          <li class="rounded border border-border-strong px-2 py-0.5 text-sm">
            {concept.name}
          </li>
        {/each}
      </ul>
    {/if}
  </header>

  {#if problem.constraints.length > 0}
    <section aria-labelledby="constraints-heading" class="flex flex-col gap-1">
      <h3 id="constraints-heading" class="text-base font-semibold">
        Constraints
      </h3>
      <ul aria-label="Constraints" class="list-disc pl-5 break-words">
        {#each problem.constraints as constraint, index (index)}
          <li>{constraint}</li>
        {/each}
      </ul>
    </section>
  {/if}

  <section aria-labelledby="signature-heading" class="flex flex-col gap-1">
    <h3 id="signature-heading" class="text-base font-semibold">Signature</h3>
    <code class="font-mono text-sm break-words">{signature}</code>
  </section>

  <div class="flex flex-col gap-2">
    <button
      type="button"
      class="min-h-11 self-start text-left font-semibold"
      aria-expanded={statementOpen}
      aria-controls="statement-body"
      onclick={() => (statementOpen = !statementOpen)}
    >
      Statement
    </button>
    <div
      id="statement-body"
      class="flex flex-col gap-3"
      hidden={!statementOpen}
    >
      {#each blocks as block, blockIndex (blockIndex)}
        {#if block.kind === 'heading'}
          <svelte:element this={`h${block.level}`} class="font-semibold">
            {#each block.segments as segment, index (segmentKey(segment, index))}
              {#if segment.code}<code class="font-mono">{segment.text}</code
                >{:else}{segment.text}{/if}
            {/each}
          </svelte:element>
        {:else if block.kind === 'paragraph'}
          <p class="whitespace-pre-line break-words">
            {#each block.segments as segment, index (segmentKey(segment, index))}
              {#if segment.code}<code class="font-mono">{segment.text}</code
                >{:else}{segment.text}{/if}
            {/each}
          </p>
        {:else if block.kind === 'list'}
          <ul class="list-disc pl-5">
            {#each block.items as item, itemIndex (itemIndex)}
              <li>
                {#each item as segment, index (segmentKey(segment, index))}
                  {#if segment.code}<code class="font-mono">{segment.text}</code
                    >{:else}{segment.text}{/if}
                {/each}
              </li>
            {/each}
          </ul>
        {:else}
          <pre
            class="overflow-x-auto rounded bg-surface p-3 font-mono text-sm"><code
              >{block.text}</code
            ></pre>
        {/if}
      {/each}
    </div>
  </div>

  <div class="flex flex-col gap-2">
    <button
      type="button"
      class="min-h-11 self-start text-left font-semibold"
      aria-expanded={examplesOpen}
      aria-controls="examples-body"
      onclick={() => (examplesOpen = !examplesOpen)}
    >
      Examples
    </button>
    <div id="examples-body" hidden={!examplesOpen}>
      {#if problem.samples.length === 0}
        <p class="text-sm text-text-muted">
          No examples are published for this problem.
        </p>
      {:else}
        <ol class="flex flex-col gap-3">
          {#each problem.samples as sample, index (index)}
            <li
              class="rounded border border-border p-3 font-mono text-sm break-all"
            >
              <div>Input: {formatValue(sample.input)}</div>
              <div>Output: {formatValue(sample.expected)}</div>
            </li>
          {/each}
        </ol>
      {/if}
    </div>
  </div>
</section>
