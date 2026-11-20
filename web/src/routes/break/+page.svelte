<script>
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import { fetchDaily, fetchScores } from '#lib/break/api.js';

  /** @type {'loading' | 'ready' | 'not_imported' | 'error'} */
  let puzzle = $state('loading');
  /** @type {Array<{ id: number, game: string, score: number }>} */
  let scores = $state([]);
  let signedIn = $state(false);

  onMount(async () => {
    const daily = await fetchDaily();
    puzzle = daily.ok ? 'ready' : daily.code === 'not_imported' ? 'not_imported' : 'error';
    const own = await fetchScores();
    if (own.ok) {
      signedIn = true;
      scores = /** @type {any} */ (own.scores) ?? [];
    }
  });
</script>

<svelte:head><title>Break room</title></svelte:head>

<main class="mx-auto max-w-3xl space-y-8 p-6">
  <header class="space-y-1">
    <h1 class="text-2xl font-bold">Break room</h1>
    <p class="text-text-muted">A few ungraded games. Nothing here affects your practice.</p>
  </header>

  <section class="grid gap-4 sm:grid-cols-3" aria-label="Games">
    <a
      href={resolve('/break/puzzle')}
      class="rounded-lg border border-border bg-raised p-4 hover:border-border-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
    >
      <h2 class="font-semibold">Daily chess puzzle</h2>
      <p class="mt-1 text-sm text-text-muted">
        {#if puzzle === 'loading'}Loading today's puzzle…
        {:else if puzzle === 'not_imported'}Today's puzzle is not available yet.
        {:else if puzzle === 'error'}Could not load today's puzzle.
        {:else}One puzzle a day, picked for you.{/if}
      </p>
    </a>
    <a
      href={resolve('/break/bot')}
      class="rounded-lg border border-border bg-raised p-4 hover:border-border-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
    >
      <h2 class="font-semibold">Chess bot</h2>
      <p class="mt-1 text-sm text-text-muted">A casual game against a small local engine.</p>
    </a>
    <a
      href={resolve('/break/memory')}
      class="rounded-lg border border-border bg-raised p-4 hover:border-border-strong focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
    >
      <h2 class="font-semibold">Lantern recall</h2>
      <p class="mt-1 text-sm text-text-muted">Repeat the growing sequence of nine lanterns.</p>
    </a>
  </section>

  {#if signedIn && scores.length > 0}
    <section aria-labelledby="recent-scores">
      <h2 id="recent-scores" class="text-lg font-semibold">Your recent scores</h2>
      <ul class="mt-2 divide-y divide-border rounded-lg border border-border">
        {#each scores as s (s.id)}
          <li class="flex justify-between px-4 py-2 text-sm">
            <span class="text-text-muted">{s.game}</span>
            <span>{s.score}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
</main>
