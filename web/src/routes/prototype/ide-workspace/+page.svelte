<script lang="ts">
  import {
    FIXTURE_PROBLEM,
    PROTOTYPE_BANNER
  } from '../../../lib/prototype/fixtures';

  let note = $state('');

  // The fixture action only changes local text. It never calls the run API.
  function runFixture() {
    note = 'No judge run was started. This fixture shows the layout only.';
  }
</script>

<svelte:head>
  <title>Prototype: IDE workspace</title>
</svelte:head>

<main class="min-h-screen bg-bg px-6 py-10 text-text sm:px-10">
  <p
    class="mb-6 rounded border-2 border-accent-candle bg-surface px-4 py-3 font-semibold text-accent-candle"
  >
    {PROTOTYPE_BANNER}
  </p>

  <section
    class="mx-auto flex max-w-3xl flex-col gap-4"
    aria-labelledby="fixture-title"
  >
    <h1 id="fixture-title" class="text-3xl font-semibold tracking-tight">
      {FIXTURE_PROBLEM.title}
    </h1>
    <p class="text-text-muted">{FIXTURE_PROBLEM.statementMarkdown}</p>
    <div>
      <button
        type="button"
        class="rounded bg-accent-candle px-4 py-2 font-semibold text-bg focus-visible:outline-2"
        onclick={runFixture}
      >
        Run samples (fixture)
      </button>
    </div>
    {#if note}
      <p role="status" class="text-text-subtle">{note}</p>
    {/if}
  </section>
</main>
