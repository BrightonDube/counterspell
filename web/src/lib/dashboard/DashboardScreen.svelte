<script lang="ts">
  // The signed-in learner's dashboard. It loads once, shows one of four phases (loading, signed out, error or
  // the data), and retries on request. Every status is written in words. Colour only adds emphasis.
  import { resolve } from '$app/paths';
  import { ApiRequestError } from '../api/client';
  import { loadDashboard } from './client';
  import {
    cardStateLabel,
    reasonLabel,
    reviewKindLabel,
    runStatusLabel
  } from './labels';
  import type { DashboardView, RecentSubmission } from './types';
  import NavEntry from '../nav/NavEntry.svelte';
  import AnimatedJacko from '../familiar/AnimatedJacko.svelte';
  import ProgressRing from '../art/ProgressRing.svelte';

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

  function verdictTone(run: RecentSubmission): string {
    if (run.verdict === 'accepted') return 'text-success';
    if (run.status !== 'completed') return 'text-text-muted';
    return 'text-danger';
  }

  const LINK_FOCUS =
    'rounded-sm underline-offset-4 hover:underline focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-focus focus-visible:outline-none';

  const PANEL =
    'rounded-xl border border-border bg-surface/80 p-5 shadow-[0_18px_44px_-16px_rgba(0,0,0,0.7)] backdrop-blur-sm sm:p-6';
</script>

<section
  class="mx-auto max-w-4xl space-y-6 text-text"
  aria-labelledby="dashboard-title"
>
  <div class="flex flex-wrap items-end justify-between gap-3">
    <div>
      <h1 id="dashboard-title" class="text-2xl font-semibold tracking-tight">
        Dashboard
      </h1>
      <p class="mt-1 text-sm text-text-muted">Your practice, at a glance.</p>
    </div>
    {#if phase.kind === 'ready'}
      <NavEntry />
    {/if}
  </div>

  {#if phase.kind === 'loading'}
    <div class="{PANEL} flex flex-col items-center gap-4 py-8 text-center">
      <AnimatedJacko mood="orange" size={96} />
      <p role="status" class="text-text-muted">Loading dashboard</p>
    </div>
  {:else if phase.kind === 'signed_out'}
    <div class="{PANEL} space-y-3">
      <p class="text-text-muted">
        Sign in to see your progress, recent runs and next problems.
      </p>
      <a
        href={resolve('/login?return_to=/dashboard')}
        class="inline-block rounded-md bg-accent-candle px-4 py-2 font-medium text-bg hover:bg-accent-candle-glow focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-focus focus-visible:outline-none"
      >
        Sign in
      </a>
    </div>
  {:else if phase.kind === 'error'}
    <div class="{PANEL} space-y-3">
      <p role="alert" class="text-danger">{phase.message}</p>
      <button
        type="button"
        onclick={refresh}
        class="rounded-md border border-border-strong bg-raised px-4 py-2 hover:bg-surface focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-focus focus-visible:outline-none"
      >
        Try again
      </button>
    </div>
  {:else}
    {@const view = phase.view}

    {#if hasNoActivity(view)}
      <div
        class="{PANEL} flex flex-col items-center gap-4 py-8 text-center sm:flex-row sm:items-start sm:text-left"
      >
        <AnimatedJacko mood="orange" size={96} />
        <div class="space-y-3">
          <h2 class="text-lg font-semibold">No practice yet</h2>
          <p class="text-text-muted">
            Run a problem to see your solved count, streak and recent runs here.
          </p>
        </div>
      </div>
    {/if}

    <section
      class="{PANEL} panel-in space-y-4"
      style="animation-delay: 60ms"
      aria-labelledby="solved-title"
    >
      <div class="flex items-center justify-between gap-3">
        <h2 id="solved-title" class="text-lg font-semibold">Solved</h2>
        {#if view.recentSubmissions[0]?.verdict === 'accepted'}
          <div class="flex items-center gap-3">
            <AnimatedJacko mood="poison" size={40} />
            <p class="text-sm text-success">Your newest run was accepted.</p>
          </div>
        {/if}
      </div>
      <div class="flex flex-wrap items-center gap-6">
        <ProgressRing
          segments={[
            { value: view.solved.byDifficulty.easy, color: 'var(--success)' },
            {
              value: view.solved.byDifficulty.medium,
              color: 'var(--accent-candle)'
            },
            { value: view.solved.byDifficulty.hard, color: 'var(--danger)' }
          ]}
          goal={Math.max(view.solved.total, 1)}
        />
        <p class="text-4xl font-semibold tabular-nums">
          {view.solved.total}
          <span class="text-base font-normal text-text-muted">problems</span>
        </p>
      </div>
      <dl class="grid grid-cols-3 gap-3 text-sm">
        <div class="rounded-lg border border-border bg-raised px-4 py-3">
          <dt class="text-text-muted">Easy</dt>
          <dd class="mt-1 text-xl font-semibold tabular-nums">
            {view.solved.byDifficulty.easy}
          </dd>
        </div>
        <div class="rounded-lg border border-border bg-raised px-4 py-3">
          <dt class="text-text-muted">Medium</dt>
          <dd class="mt-1 text-xl font-semibold tabular-nums">
            {view.solved.byDifficulty.medium}
          </dd>
        </div>
        <div class="rounded-lg border border-border bg-raised px-4 py-3">
          <dt class="text-text-muted">Hard</dt>
          <dd class="mt-1 text-xl font-semibold tabular-nums">
            {view.solved.byDifficulty.hard}
          </dd>
        </div>
      </dl>
      <p class="text-text-muted">
        <span class="text-accent-candle" aria-hidden="true">●</span>
        {streakLine(view)}
      </p>
    </section>

    <section
      class="{PANEL} panel-in space-y-4"
      style="animation-delay: 120ms"
      aria-labelledby="review-title"
      aria-live="polite"
    >
      <div class="flex items-center justify-between gap-3">
        <h2 id="review-title" class="text-lg font-semibold">Review due</h2>
        <span
          class="rounded-full border border-border bg-raised px-3 py-1 text-xs font-medium text-accent-candle"
        >
          Spaced repetition
        </span>
      </div>
      {#if view.reviewSummary === null}
        <p class="text-text-muted">No reviews due.</p>
      {:else}
        {@const summary = view.reviewSummary}
        <p class="text-sm text-text-muted">
          {summary.dueCount}
          {summary.dueCount === 1 ? 'card' : 'cards'} due
        </p>
        <div class="space-y-2 rounded-lg border border-border bg-raised px-4 py-3">
          <p class="font-medium">{summary.nextReview.concept.name}</p>
          <p class="text-sm text-text-muted">
            {cardStateLabel(summary.nextReview.card.state)} · due {summary
              .nextReview.card.due}
          </p>
          {#if summary.nextReview.challenge}
            {@const challenge = summary.nextReview.challenge}
            <div>
              <a
                href={resolve('/problems/[slug]', { slug: challenge.slug })}
                class="font-medium text-accent-moss {LINK_FOCUS}"
              >
                {challenge.title}
              </a>
              <p class="text-sm text-text-muted">
                {challenge.difficulty} · {reviewKindLabel(challenge.kind)}
              </p>
            </div>
          {/if}
        </div>
      {/if}
    </section>

    <div
      class="grid gap-6 lg:grid-cols-2 panel-in"
      style="animation-delay: 140ms"
    >
      <section class="{PANEL} space-y-4" aria-labelledby="weak-title">
        <h2 id="weak-title" class="text-lg font-semibold">
          Concepts to revisit
        </h2>
        {#if view.weakConcepts.length === 0}
          <p class="text-text-muted">No failed concepts yet.</p>
        {:else}
          <ul class="space-y-2">
            {#each view.weakConcepts as concept, weakIdx (concept.slug)}
              <li
                class="row-in flex flex-wrap items-baseline justify-between gap-2 rounded-lg border border-border bg-raised px-4 py-3 transition-all duration-150 hover:-translate-y-0.5 hover:border-accent-candle"
                style="animation-delay: {200 + weakIdx * 50}ms"
              >
                <span class="font-medium">{concept.name}</span>
                <span class="text-sm text-text-muted">
                  {failureText(concept.failureCount)}
                </span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="{PANEL} space-y-4" aria-labelledby="recent-title">
        <h2 id="recent-title" class="text-lg font-semibold">Recent runs</h2>
        {#if view.recentSubmissions.length === 0}
          <p class="text-text-muted">No runs yet.</p>
        {:else}
          <ol class="space-y-2">
            {#each view.recentSubmissions as run, runIdx (run.id)}
              <li
                class="row-in space-y-1 rounded-lg border border-border bg-raised px-4 py-3 transition-all duration-150 hover:-translate-y-0.5 hover:border-accent-candle"
                style="animation-delay: {200 + runIdx * 50}ms"
              >
                <a
                  href={resolve('/problems/[slug]', { slug: run.problem.slug })}
                  class="font-medium text-accent-moss {LINK_FOCUS}"
                >
                  {run.problem.title}
                </a>
                <p class="text-sm text-text-muted">
                  {run.lang} · {run.mode} ·
                  <span class={verdictTone(run)}>
                    {runStatusLabel(run.status, run.verdict)}
                  </span>
                  {#if resultLine(run)}· {resultLine(run)}{/if}
                </p>
              </li>
            {/each}
          </ol>
        {/if}
      </section>
    </div>

    <section
      class="{PANEL} panel-in space-y-4"
      style="animation-delay: 220ms"
      aria-labelledby="next-title"
    >
      <h2 id="next-title" class="text-lg font-semibold">Try next</h2>
      {#if view.suggestedChallenges.length === 0}
        <p class="text-text-muted">
          No suggestions right now. Browse the catalogue for more.
        </p>
      {:else}
        <ol class="grid gap-3 sm:grid-cols-2">
          {#each view.suggestedChallenges as challenge, chIdx (challenge.slug)}
            <li
              class="row-in space-y-1 rounded-lg border border-border bg-raised px-4 py-3 transition-all duration-150 hover:-translate-y-0.5 hover:border-accent-candle"
              style="animation-delay: {260 + chIdx * 50}ms"
            >
              <a
                href={resolve('/problems/[slug]', { slug: challenge.slug })}
                class="font-medium text-accent-moss {LINK_FOCUS}"
              >
                {challenge.title}
              </a>
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
