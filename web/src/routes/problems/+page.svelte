<script lang="ts">
  // The grimoire catalogue: every published problem, listed for anyone to browse.
  // Anonymous visitors get the list with status null; signing in happens inside the workspace.
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import AppShell from '../../lib/shell/AppShell.svelte';

  type ProblemCard = {
    slug: string;
    title: string;
    difficulty: string;
    track: { slug: string; name: string };
    concepts: { slug: string; name: string }[];
    status: string | null;
  };

  let problems = $state<ProblemCard[]>([]);
  let phase = $state<'loading' | 'error' | 'ready'>('loading');

  onMount(async () => {
    try {
      const resp = await fetch('/api/problems', { credentials: 'same-origin' });
      if (!resp.ok) throw new Error(`catalogue ${resp.status}`);
      const data = (await resp.json()) as { problems: ProblemCard[] };
      problems = data.problems;
      phase = 'ready';
    } catch {
      phase = 'error';
    }
  });

  const LINK_FOCUS =
    'focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:outline-none';
</script>

<svelte:head>
  <title>Problems · Counterspell</title>
</svelte:head>

<AppShell>
  <section
    class="mx-auto max-w-4xl space-y-6 text-text"
    aria-labelledby="problems-title"
  >
    <div>
      <h1 id="problems-title" class="text-2xl font-semibold tracking-tight">
        Problems
      </h1>
      <p class="mt-1 text-sm text-text-muted">
        Open the grimoire and pick a spell to practise.
      </p>
    </div>

    {#if phase === 'loading'}
      <p role="status" class="text-text-muted">Loading the grimoire</p>
    {:else if phase === 'error'}
      <p role="alert" class="text-danger">
        The catalogue could not load. Try again shortly.
      </p>
    {:else if problems.length === 0}
      <p class="text-text-muted">No published problems yet.</p>
    {:else}
      <ol class="grid gap-3 sm:grid-cols-2">
        {#each problems as problem (problem.slug)}
          <li
            class="rounded-xl border border-border bg-surface/80 p-5 shadow-[0_18px_44px_-16px_rgba(0,0,0,0.7)] backdrop-blur-sm transition-colors hover:border-accent-candle"
          >
            <a
              href={resolve('/problems/[slug]', { slug: problem.slug })}
              class="text-lg font-medium text-accent-moss underline-offset-4 hover:underline {LINK_FOCUS}"
            >
              {problem.title}
            </a>
            <p class="mt-1 text-sm capitalize text-text-muted">
              {problem.track.name} · {problem.difficulty}
            </p>
            {#if problem.concepts.length > 0}
              <p class="mt-2 text-sm text-text-muted">
                Concepts: {problem.concepts.map((c) => c.name).join(', ')}
              </p>
            {/if}
          </li>
        {/each}
      </ol>
    {/if}
  </section>
</AppShell>
