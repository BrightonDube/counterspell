<script lang="ts">
  // The grimoire catalogue: every published problem, grouped by track and browsable by anyone.
  // Anonymous visitors see hollow status circles; signing in happens inside the workspace.
  import { onMount } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import { resolve } from '$app/paths';
  import AppShell from '../../lib/shell/AppShell.svelte';
  import ProgressRing from '../../lib/art/ProgressRing.svelte';
  import AnimatedArt from '../../lib/art/AnimatedArt.svelte';
  import Seo from '../../lib/seo/Seo.svelte';

  type ProblemCard = {
    slug: string;
    title: string;
    difficulty: string;
    track: { slug: string; name: string };
    concepts: { slug: string; name: string }[];
    status: string | null;
  };

  type Difficulty = 'all' | 'easy' | 'medium' | 'hard';
  type StatusPick = 'all' | 'solved' | 'open';

  const DIFF_COLORS: Record<string, string> = {
    easy: 'var(--success)',
    medium: 'var(--accent-candle)',
    hard: 'var(--danger)'
  };

  let problems = $state<ProblemCard[]>([]);
  let phase = $state<'loading' | 'error' | 'ready'>('loading');
  let query = $state('');
  let difficulty = $state<Difficulty>('all');
  let statusPick = $state<StatusPick>('all');

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

  const solvedCount = $derived(
    problems.filter((p) => p.status === 'solved').length
  );
  const byDifficulty = $derived({
    easy: problems.filter((p) => p.difficulty === 'easy').length,
    medium: problems.filter((p) => p.difficulty === 'medium').length,
    hard: problems.filter((p) => p.difficulty === 'hard').length
  });
  const solvedByDifficulty = $derived({
    easy: problems.filter(
      (p) => p.difficulty === 'easy' && p.status === 'solved'
    ).length,
    medium: problems.filter(
      (p) => p.difficulty === 'medium' && p.status === 'solved'
    ).length,
    hard: problems.filter(
      (p) => p.difficulty === 'hard' && p.status === 'solved'
    ).length
  });

  const filtered = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    return problems.filter((p) => {
      if (difficulty !== 'all' && p.difficulty !== difficulty) return false;
      if (statusPick === 'solved' && p.status !== 'solved') return false;
      if (statusPick === 'open' && p.status === 'solved') return false;
      if (!needle) return true;
      return (
        p.title.toLowerCase().includes(needle) ||
        p.track.name.toLowerCase().includes(needle) ||
        p.concepts.some((c) => c.name.toLowerCase().includes(needle))
      );
    });
  });

  const groups = $derived.by(() => {
    const order: string[] = [];
    const map = new SvelteMap<string, ProblemCard[]>();
    for (const p of filtered) {
      if (!map.has(p.track.slug)) order.push(p.track.slug);
      map.set(p.track.slug, [...(map.get(p.track.slug) ?? []), p]);
    }
    return order.map((slug) => ({
      name: map.get(slug)?.[0]?.track.name ?? slug,
      rows: map.get(slug) ?? []
    }));
  });

  const LINK_FOCUS =
    'focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:outline-none';
  const PANEL =
    'rounded-xl border border-border bg-surface/80 shadow-[0_18px_44px_-16px_rgba(0,0,0,0.7)] backdrop-blur-sm';
  const pill = (active: boolean) =>
    `rounded-full border px-3 py-1 text-xs font-medium transition-colors ${LINK_FOCUS} ` +
    (active
      ? 'border-accent-candle bg-accent-candle/15 text-accent-candle'
      : 'border-border text-text-muted hover:border-border-strong hover:text-text');
</script>

<Seo
  title="Problems · Counterspell"
  description="Browse the Counterspell catalogue by track, difficulty and concept, and pick your next problem."
  path="/problems"
/>

<AppShell>
  <div
    class="mx-auto grid max-w-6xl gap-6 text-text lg:grid-cols-[minmax(0,1fr)_280px]"
  >
    <section class="space-y-6" aria-labelledby="problems-title">
      <header class="panel-in">
        <div class="flex flex-wrap items-end justify-between gap-4">
          <div>
            <h1
              id="problems-title"
              class="text-3xl font-semibold tracking-tight"
            >
              Grimoire.
            </h1>
            <p class="mt-1 text-sm text-text-muted">
              Every published spell, grouped by school of magic.
            </p>
          </div>
          <div class="flex gap-3 text-center">
            <div class="rounded-lg border border-border bg-raised px-4 py-2">
              <p class="text-[10px] tracking-wide text-success uppercase">
                Solved
              </p>
              <p class="text-lg font-semibold tabular-nums">
                {solvedCount}<span class="text-text-muted">
                  /{problems.length}
                </span>
              </p>
            </div>
            <div class="rounded-lg border border-border bg-raised px-4 py-2">
              <p class="text-[10px] tracking-wide text-accent-candle uppercase">
                Spells
              </p>
              <p class="text-lg font-semibold tabular-nums">
                {problems.length}
              </p>
            </div>
          </div>
        </div>
      </header>

      <div class="panel-in space-y-3" style="animation-delay: 60ms">
        <div class="relative">
          <svg
            class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-text-muted"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            aria-hidden="true"
          >
            <circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
          </svg>
          <input
            type="search"
            bind:value={query}
            placeholder="Search problems"
            aria-label="Search problems"
            class="w-full rounded-lg border border-border bg-surface/80 py-2.5 pr-4 pl-10 text-sm text-text placeholder-text-subtle backdrop-blur-sm transition-colors focus:border-accent-candle focus:outline-none"
          />
        </div>
        <div
          class="flex flex-wrap items-center gap-2"
          role="group"
          aria-label="Filters"
        >
          {#each ['all', 'easy', 'medium', 'hard'] as d (d)}
            <button
              type="button"
              class={pill(difficulty === d)}
              onclick={() => (difficulty = d as Difficulty)}
            >
              {d === 'all' ? 'All' : d[0].toUpperCase() + d.slice(1)}
            </button>
          {/each}
          <span class="mx-1 h-4 w-px bg-border" aria-hidden="true"></span>
          {#each [['all', 'Any status'], ['open', 'Unsolved'], ['solved', 'Solved']] as [v, label] (v)}
            <button
              type="button"
              class={pill(statusPick === v)}
              onclick={() => (statusPick = v as StatusPick)}
            >
              {label}
            </button>
          {/each}
        </div>
      </div>

      {#if phase === 'loading'}
        <div class="{PANEL} overflow-hidden" aria-busy="true">
          <p role="status" class="sr-only">Loading the grimoire</p>
          {#each [0, 1, 2, 3, 4, 5] as i (i)}
            <div
              class="flex items-center gap-4 border-b border-border/60 px-5 py-4 last:border-0"
            >
              <div class="skeleton h-5 w-5 rounded-full"></div>
              <div
                class="skeleton h-4 flex-1 rounded"
                style="animation-delay: {i * 90}ms"
              ></div>
              <div class="skeleton h-4 w-14 rounded"></div>
            </div>
          {/each}
        </div>
      {:else if phase === 'error'}
        <p role="alert" class="{PANEL} p-5 text-danger">
          The catalogue could not load. Try again shortly.
        </p>
      {:else if problems.length === 0}
        <p class="{PANEL} p-5 text-text-muted">No published problems yet.</p>
      {:else if groups.length === 0}
        <p class="{PANEL} p-5 text-text-muted">
          No spells match that search. Loosen the filters.
        </p>
      {:else}
        <div class="space-y-6">
          {#each groups as group, gi (group.name)}
            <section
              class="{PANEL} panel-in overflow-hidden"
              style="animation-delay: {120 + gi * 80}ms"
              aria-labelledby="track-{gi}"
            >
              <h2
                id="track-{gi}"
                class="border-b border-border px-5 py-3 text-sm font-semibold tracking-wide text-text-muted uppercase"
              >
                {group.name}
              </h2>
              <div
                class="hidden grid-cols-[2rem_minmax(0,1fr)_5rem] items-center gap-4 border-b border-border/60 px-5 py-2 text-[10px] font-medium tracking-widest text-text-subtle uppercase sm:grid"
                aria-hidden="true"
              >
                <span>Status</span><span>Problem</span><span class="text-right">
                  Difficulty
                </span>
              </div>
              <ol>
                {#each group.rows as problem, i (problem.slug)}
                  <li
                    class="row-in"
                    style="animation-delay: {160 + gi * 80 + i * 35}ms"
                  >
                    <a
                      href={resolve('/problems/[slug]', { slug: problem.slug })}
                      class="group grid grid-cols-[2rem_minmax(0,1fr)_5rem] items-center gap-4 border-b border-border/60 px-5 py-3.5 transition-all duration-150 last:border-0 hover:bg-raised/70 hover:pl-6 {LINK_FOCUS}"
                    >
                      <span
                        class="grid h-5 w-5 place-items-center rounded-full border transition-colors {problem.status ===
                        'solved'
                          ? 'border-success bg-success text-bg'
                          : 'border-border-strong text-transparent group-hover:border-text-muted'}"
                        aria-label={problem.status === 'solved'
                          ? 'Solved'
                          : 'Not solved'}
                      >
                        <svg
                          class="h-3 w-3"
                          viewBox="0 0 12 12"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2.4"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          aria-hidden="true"
                        >
                          <path d="m2 6.5 2.5 2.5L10 3.5" />
                        </svg>
                      </span>
                      <span
                        class="truncate font-medium transition-colors group-hover:text-accent-candle"
                      >
                        {problem.title}
                      </span>
                      <span
                        class="text-right text-sm font-medium capitalize"
                        style:color={DIFF_COLORS[problem.difficulty] ??
                          'var(--text-muted)'}
                      >
                        {problem.difficulty}
                      </span>
                    </a>
                  </li>
                {/each}
              </ol>
            </section>
          {/each}
        </div>
      {/if}
    </section>

    <aside class="space-y-6 lg:sticky lg:top-6 lg:self-start">
      <section
        class="{PANEL} panel-in flex flex-col items-center gap-4 p-5"
        style="animation-delay: 90ms"
        aria-labelledby="progress-title"
      >
        <h2 id="progress-title" class="sr-only">Progress</h2>
        <ProgressRing
          segments={[
            { value: solvedByDifficulty.easy, color: 'var(--success)' },
            { value: solvedByDifficulty.medium, color: 'var(--accent-candle)' },
            { value: solvedByDifficulty.hard, color: 'var(--danger)' }
          ]}
          goal={problems.length}
        />
        <ul class="w-full space-y-1.5 text-sm">
          <li class="flex items-center justify-between">
            <span class="flex items-center gap-2">
              <span class="h-2 w-2 rounded-full bg-success"></span>Easy
            </span>
            <span class="tabular-nums text-text-muted">
              {solvedByDifficulty.easy} / {byDifficulty.easy}
            </span>
          </li>
          <li class="flex items-center justify-between">
            <span class="flex items-center gap-2">
              <span class="h-2 w-2 rounded-full bg-accent-candle"></span>Medium
            </span>
            <span class="tabular-nums text-text-muted">
              {solvedByDifficulty.medium} / {byDifficulty.medium}
            </span>
          </li>
          <li class="flex items-center justify-between">
            <span class="flex items-center gap-2">
              <span class="h-2 w-2 rounded-full bg-danger"></span>Hard
            </span>
            <span class="tabular-nums text-text-muted">
              {solvedByDifficulty.hard} / {byDifficulty.hard}
            </span>
          </li>
        </ul>
      </section>

      <section
        class="{PANEL} panel-in flex items-center gap-4 p-5"
        style="animation-delay: 150ms"
      >
        <AnimatedArt
          src="/art/objects/candle-flame.webp"
          srcset="/art/objects/candle-flame-192.webp 192w, /art/objects/candle-flame.webp 288w"
          sizes="48px"
          size={48}
          motion="flame"
          bounce
          centre={false}
          alt=""
        />
        <p class="text-sm text-text-muted">
          One problem a day keeps the streak alive.
        </p>
      </section>
    </aside>
  </div>
</AppShell>
