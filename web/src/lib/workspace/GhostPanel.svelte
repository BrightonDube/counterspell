<script lang="ts">
  import { planPlayback } from '../ghost/shrink-playback';
  import type { GhostView, ShrinkStepView } from '../ghost/types';

  let {
    status,
    ghost,
    shrinkSteps,
    reducedMotion
  }: {
    status: string;
    ghost: GhostView | null;
    shrinkSteps: readonly ShrinkStepView[];
    reducedMotion: boolean;
  } = $props();

  // The replay plan orders and times only the steps the server recorded. Steps that do not end at the
  // ghost input are not shown at all.
  const plan = $derived(
    ghost ? planPlayback(ghost.shrink, ghost.input, reducedMotion) : null
  );

  /** Shows a JSON value as plain text. Nothing is inserted as HTML. */
  function show(value: unknown): string {
    return JSON.stringify(value) ?? 'undefined';
  }

  /** States what the engine actually established, never more than that. */
  function headline(view: GhostView): string {
    if (view.kind === 'timeout') {
      return 'Timed out on this input. The engine did not confirm a smaller one.';
    }
    return 'Minimized counterexample: the smallest failing input found within the search budget. It is not a proven minimum.';
  }
</script>

<section aria-labelledby="ghost-heading" class="flex flex-col gap-3">
  <h2 id="ghost-heading" class="text-base font-semibold text-text">
    Ghost input
  </h2>

  {#if ghost === null}
    {#if status === 'ghost_hunting'}
      <p role="status" aria-busy="true" class="text-text-muted">
        Searching for a failing input. This can take a few seconds.
      </p>
      {#if shrinkSteps.length > 0}
        <ol
          class="flex flex-col gap-1 font-mono text-sm text-text-muted"
          data-motion="live"
        >
          {#each shrinkSteps as step (step.index)}
            <li>Step {step.index + 1}: {show(step.input)}</li>
          {/each}
        </ol>
      {/if}
    {:else if status === 'completed'}
      <p role="status" class="text-text-muted">
        No ghost input was found within the search budget.
      </p>
    {/if}
  {:else if plan !== null}
    <p class="font-semibold text-accent-moss">{headline(ghost)}</p>
    <p class="text-sm text-text-muted">{ghost.explanation}</p>
    <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 font-mono text-sm">
      <dt class="text-text-muted">Input</dt>
      <dd><code class="break-all">{show(ghost.input)}</code></dd>
      {#if ghost.expected !== undefined}
        <dt class="text-success">Expected</dt>
        <dd><code class="break-all">{show(ghost.expected)}</code></dd>
      {/if}
      {#if ghost.got !== undefined}
        <dt class="text-danger">Got</dt>
        <dd><code class="break-all">{show(ghost.got)}</code></dd>
      {/if}
    </dl>

    {#if plan.trust === 'disagrees'}
      <p role="alert" class="text-sm text-text-muted">
        The recorded steps do not end at this input, so they are not shown.
      </p>
    {:else}
      <ol
        class="flex flex-col gap-1 font-mono text-sm text-text-muted"
        data-motion={plan.mode}
      >
        {#each plan.steps as step, position (step.index)}
          <li
            class="motion-safe:transition-opacity motion-safe:duration-200"
            style="transition-delay: {position * plan.stepDelayMs}ms"
          >
            Step {step.index + 1}: {show(step.input)}
          </li>
        {/each}
      </ol>
      {#if plan.omittedSteps > 0}
        <p class="text-sm text-text-subtle">
          The engine summarised {plan.omittedSteps} more steps. They are not shown.
        </p>
      {/if}
    {/if}
  {/if}
</section>
