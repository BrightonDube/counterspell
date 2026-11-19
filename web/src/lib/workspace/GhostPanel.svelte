<script lang="ts">
  import { planPlayback } from '../ghost/shrink-playback';
  import type { GhostView, ShrinkStepView } from '../ghost/types';

  let {
    status,
    ghost,
    ghostError = null,
    shrinkSteps,
    reducedMotion
  }: {
    status: string;
    ghost: GhostView | null;
    /** Set when the ghost payload failed validation, so nothing is shown from it. */
    ghostError?: 'malformed' | null;
    shrinkSteps: readonly ShrinkStepView[];
    reducedMotion: boolean;
  } = $props();

  /** Run endings that stop the search before it can report a result. */
  const CUT_OFF_STATUSES = [
    'aborted',
    'cancelled',
    'expired',
    'internal_error'
  ];

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
    {:else if ghostError === 'malformed'}
      <p role="alert" class="text-text-muted">
        The ghost data could not be verified, so nothing is shown.
      </p>
    {:else if CUT_OFF_STATUSES.includes(status)}
      <p role="status" class="text-text-muted">
        The ghost search did not finish, so no input is shown.
      </p>
    {:else if status === 'completed'}
      <p role="status" class="text-text-muted">
        No ghost input was found within the search budget.
      </p>
    {/if}
  {:else if plan !== null}
    <div
      class={plan.mode === 'animated'
        ? 'ghost-reveal flex flex-col gap-3'
        : 'flex flex-col gap-3'}
    >
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
            {#if plan.mode === 'animated'}
              <li
                class="ghost-step"
                style="animation-delay: {position * plan.stepDelayMs}ms"
              >
                Step {step.index + 1}: {show(step.input)}
              </li>
            {:else}
              <li>Step {step.index + 1}: {show(step.input)}</li>
            {/if}
          {/each}
        </ol>
        {#if plan.omittedSteps > 0}
          <p class="text-sm text-text-subtle">
            The engine summarised {plan.omittedSteps} more steps. They are not shown.
          </p>
        {/if}
      {/if}
    </div>
  {/if}
</section>

<style>
  /* One calm 400 ms reveal for the card (palette: ghost-reveal fade and blur-in). */
  .ghost-reveal {
    animation: ghost-reveal 400ms ease-out both;
  }

  /* Each recorded step fades in on its planned delay; 200 ms sits in the 150-250 ms range. The
     items stay in the DOM from the start, so screen readers get the whole list at once. */
  .ghost-step {
    animation: ghost-step-in 200ms ease-out both;
  }

  @keyframes ghost-reveal {
    from {
      opacity: 0;
      filter: blur(4px);
    }
    to {
      opacity: 1;
      filter: none;
    }
  }

  @keyframes ghost-step-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  /* The component already picks static mode under reduced motion; this is belt and braces. */
  @media (prefers-reduced-motion: reduce) {
    .ghost-reveal,
    .ghost-step {
      animation: none;
    }
  }
</style>
