<script lang="ts">
  // The signed-in learner's dashboard. It loads once, shows one of four phases (loading, signed out, error or
  // the data), and retries on request. Every status is written in words. Colour only adds emphasis.
  import { resolve } from '$app/paths';
  import { ApiRequestError } from '../api/client';
  import { loadDashboard } from './client';
  import { reasonLabel, runStatusLabel } from './labels';
  import type { DashboardView, RecentSubmission } from './types';

  let { load = loadDashboard }: { load?: () => Promise<DashboardView> } =
    $props();

  type Phase =
    | { kind: 'loading' }
    | { kind: 'signed_out' }
    | { kind: 'error'; message: string }
    | { kind: 'ready'; view: DashboardView };

  let phase = $state.raw<Phase>({ kind: 'loading' });

  async function refresh(): Promise<void> {
    phase = { kind: 'loading' };
    try {
      phase = { kind: 'ready', view: await load() };
    } catch (error) {
      if (error instanceof ApiRequestError && error.status === 401) {
        phase = { kind: 'signed_out' };
      } else {
        phase = {
          kind: 'error',
          message:
            error instanceof Error
              ? error.message
              : 'Your dashboard could not load. Try again.'
        };
      }
    }
  }

  $effect(() => {
    void refresh();
  });

  function hasNoActivity(view: DashboardView): boolean {
    return view.solved.total === 0 && view.recentSubmissions.length === 0;
  }

  function streakLine(view: DashboardView): string {
    const { currentDays, activeToday, lastActivityDate } = view.streak;
    if (currentDays === 0) {
      return lastActivityDate
        ? `No streak. Last practised on ${lastActivityDate}.`
        : 'No streak yet.';
    }
    const days = `${currentDays}-day streak`;
    return activeToday
      ? `${days}. Practised today.`
      : `${days}. Practise today to keep it.`;
  }

  function resultLine(run: RecentSubmission): string | null {
    if (run.passed === null || run.total === null) return null;
    return `${run.passed} of ${run.total} tests passed`;
  }

  function failureText(count: number): string {
    return count === 1 ? '1 failed attempt' : `${count} failed attempts`;
  }

  const LINK_FOCUS =
    'rounded-sm underline-offset-4 hover:underline focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-focus focus-visible:outline-none';
</script>

<section
  class="mx-auto max-w-4xl space-y-10 text-text"
  aria-labelledby="dashboard-title"
>
  <h1 id="dashboard-title" class="text-2xl font-semibold">Dashboard</h1>

  {#if phase.kind === 'loading'}
    <div class="flex items-center gap-4">
      <img
        src="/art/familiar/jack-o-lantern-orange.webp"
        width="96"
        height="96"
        alt=""
        class="h-24 w-24 shrink-0"
      />
      <p role="status" class="text-text-muted">Loading dashboard</p>
    </div>
  {:else if phase.kind === 'signed_out'}
    <div class="space-y-3">
      <p class="text-text-muted">
        Sign in to see your progress, recent runs and next problems.
      </p>
      <a
        href={resolve('/login?return_to=/dashboard')}
        class="inline-block rounded-md bg-accent-candle px-4 py-2 font-medium text-bg hover:bg-accent-candle-glow focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-focus focus-visible:outline-none"
        >Sign in</a
      >
    </div>
  {:else if phase.kind === 'error'}
    <div class="space-y-3">
      <p role="alert" class="text-danger">{phase.message}</p>
      <button
        type="button"
        onclick={refresh}
        class="rounded-md border border-border-strong bg-raised px-4 py-2 hover:bg-surface focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-focus focus-visible:outline-none"
        >Try again</button
      >
    </div>
  {:else}
    {@const view = phase.view}

    {#if hasNoActivity(view)}
      <div class="flex items-start gap-4 border-t border-border pt-6">
        <img
          src="/art/familiar/jack-o-lantern-orange.webp"
          width="96"
          height="96"
          alt=""
          class="h-24 w-24 shrink-0"
        />
        <div class="space-y-3">
          <h2 class="text-lg font-semibold">No practice yet</h2>
          <p class="text-text-muted">
            Run a problem to see your solved count, streak and recent runs here.
          </p>
        </div>
      </div>
    {/if}

    <section
      class="space-y-4 border-t border-border pt-6"
      aria-labelledby="solved-title"
    >
      <h2 id="solved-title" class="text-lg font-semibold">Solved</h2>
      {#if view.recentSubmissions[0]?.verdict === 'accepted'}
        <div class="flex items-center gap-3">
          <img
            src="/art/familiar/jack-o-lantern-poison.webp"
            width="48"
            height="48"
            alt=""
            class="h-12 w-12 shrink-0"
          />
          <p class="text-text">Your newest run was accepted.</p>
        </div>
      {/if}
      <p class="text-3xl font-semibold tabular-nums">
        {view.solved.total}
        <span class="text-base font-normal text-text-muted">problems</span>
      </p>
      <dl class="grid grid-cols-3 gap-4 text-sm">
        <div>
          <dt class="text-text-muted">Easy</dt>
          <dd class="tabular-nums">{view.solved.byDifficulty.easy}</dd>
        </div>
        <div>
          <dt class="text-text-muted">Medium</dt>
          <dd class="tabular-nums">{view.solved.byDifficulty.medium}</dd>
        </div>
        <div>
          <dt class="text-text-muted">Hard</dt>
          <dd class="tabular-nums">{view.solved.byDifficulty.hard}</dd>
        </div>
      </dl>
      <p class="text-text-muted">{streakLine(view)}</p>
    </section>

    <section
      class="space-y-4 border-t border-border pt-6"
      aria-labelledby="weak-title"
    >
      <h2 id="weak-title" class="text-lg font-semibold">Concepts to revisit</h2>
      {#if view.weakConcepts.length === 0}
        <p class="text-text-muted">No failed concepts yet.</p>
      {:else}
        <ul class="space-y-3">
          {#each view.weakConcepts as concept (concept.slug)}
            <li class="flex flex-wrap items-baseline justify-between gap-2">
              <span class="font-medium">{concept.name}</span>
              <span class="text-sm text-text-muted"
                >{failureText(concept.failureCount)}</span
              >
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section
      class="space-y-4 border-t border-border pt-6"
      aria-labelledby="recent-title"
    >
      <h2 id="recent-title" class="text-lg font-semibold">Recent runs</h2>
      {#if view.recentSubmissions.length === 0}
        <p class="text-text-muted">No runs yet.</p>
      {:else}
        <ol class="space-y-4">
          {#each view.recentSubmissions as run (run.id)}
            <li class="space-y-1">
              <a
                href={resolve('/problems/[slug]', { slug: run.problem.slug })}
                class="font-medium text-accent-moss {LINK_FOCUS}"
                >{run.problem.title}</a
              >
              <p class="text-sm text-text-muted">
                {run.lang} · {run.mode} ·
                <span class="text-text"
                  >{runStatusLabel(run.status, run.verdict)}</span
                >
                {#if resultLine(run)}· {resultLine(run)}{/if}
              </p>
            </li>
          {/each}
        </ol>
      {/if}
    </section>

    <section
      class="space-y-4 border-t border-border pt-6"
      aria-labelledby="next-title"
    >
      <h2 id="next-title" class="text-lg font-semibold">Try next</h2>
      {#if view.suggestedChallenges.length === 0}
        <p class="text-text-muted">
          No suggestions right now. Browse the catalogue for more.
        </p>
      {:else}
        <ol class="space-y-4">
          {#each view.suggestedChallenges as challenge (challenge.slug)}
            <li class="space-y-1">
              <a
                href={resolve('/problems/[slug]', { slug: challenge.slug })}
                class="font-medium text-accent-moss {LINK_FOCUS}"
                >{challenge.title}</a
              >
              <p class="text-sm text-text-muted">
                {challenge.difficulty} ·
                <span class="text-text">{reasonLabel(challenge.reason)}</span>
              </p>
              {#if challenge.concepts.length > 0}
                <p class="text-sm text-text-muted">
                  Concepts: {challenge.concepts
                    .map((concept) => concept.name)
                    .join(', ')}
                </p>
              {/if}
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  {/if}
</section>
