// Tests for the dashboard screen states (dashboard-fsrs T09, DASH-9). The request is injected, so no test calls
// the network. Each test names the break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { ApiRequestError } from '../api/client';
import { parseDashboard, type DashboardView } from './types';
import DashboardScreen from './DashboardScreen.svelte';

const emptyView = parseDashboard({
  solved: { total: 0, by_difficulty: { easy: 0, medium: 0, hard: 0 } },
  streak: { current_days: 0, active_today: false, last_activity_date: null },
  weak_concepts: [],
  recent_submissions: [],
  suggested_challenges: []
});

const populatedView = parseDashboard({
  solved: { total: 1, by_difficulty: { easy: 1, medium: 0, hard: 0 } },
  streak: {
    current_days: 2,
    active_today: true,
    last_activity_date: '2026-11-15'
  },
  weak_concepts: [
    {
      slug: 'hash-maps',
      name: 'Hash maps',
      failure_count: 2,
      last_failed_at: '2026-11-14T18:20:00Z'
    }
  ],
  recent_submissions: [
    {
      id: '00000000-0000-0000-0000-00000000abcd',
      problem: { slug: 'two-sum', title: 'Two Sum', difficulty: 'easy' },
      lang: 'python',
      mode: 'submit',
      status: 'completed',
      verdict: 'wrong_answer',
      passed: 3,
      total: 12,
      score: 25,
      created_at: '2026-11-15T08:00:00Z',
      finished_at: '2026-11-15T08:00:02Z'
    },
    {
      id: '00000000-0000-0000-0000-00000000ef01',
      problem: {
        slug: 'lookup-pairs',
        title: 'Lookup Pairs',
        difficulty: 'medium'
      },
      lang: 'javascript',
      mode: 'run',
      status: 'queued',
      verdict: null,
      passed: null,
      total: null,
      score: null,
      created_at: '2026-11-15T09:00:00Z',
      finished_at: null
    }
  ],
  suggested_challenges: [
    {
      slug: 'pairs-four',
      title: 'Pairs Four',
      difficulty: 'hard',
      concepts: [{ slug: 'hash-maps', name: 'Hash maps' }],
      reason: 'practice_weak_concept'
    }
  ]
});

function signedOut(): ApiRequestError {
  return new ApiRequestError(
    401,
    'unauthenticated',
    'Sign in to view your dashboard.'
  );
}

function unavailable(): ApiRequestError {
  return new ApiRequestError(
    503,
    'dashboard_unavailable',
    'Your dashboard is unavailable right now.'
  );
}

describe('DashboardScreen', () => {
  afterEach(() => cleanup());

  // Breaks if the page shows nothing while the request is in flight, so the learner cannot tell it is loading.
  it('shows a loading message while the dashboard is being fetched', () => {
    render(DashboardScreen, {
      props: { load: () => new Promise<DashboardView>(() => {}) }
    });
    expect(screen.getByText('Loading dashboard')).toBeTruthy();
  });

  // Breaks if a signed-out visitor gets an error instead of a link back into sign-in.
  it('shows a sign-in prompt that returns to the dashboard when the visitor is signed out', async () => {
    render(DashboardScreen, {
      props: { load: () => Promise.reject(signedOut()) }
    });
    const link = await screen.findByRole('link', { name: 'Sign in' });
    expect(link.getAttribute('href')).toBe('/login?return_to=/dashboard');
  });

  // Breaks if a database failure leaves the page blank, or a retry does not refetch the dashboard.
  it('shows the safe error and a retry control, and retry loads the dashboard again', async () => {
    let calls = 0;
    const load = async () => {
      calls += 1;
      if (calls === 1) throw unavailable();
      return populatedView;
    };
    render(DashboardScreen, { props: { load } });

    expect(
      await screen.findByText('Your dashboard is unavailable right now.')
    ).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Try again' }));

    expect(await screen.findByText('Two Sum')).toBeTruthy();
    expect(calls).toBe(2);
  });

  // Breaks if a new learner sees fabricated progress or an empty page with no explanation.
  it('shows a clear empty state for a learner with no activity, without fabricated progress', async () => {
    render(DashboardScreen, { props: { load: async () => emptyView } });

    expect(await screen.findByText('No practice yet')).toBeTruthy();
    expect(screen.queryByRole('link')).toBeNull();
  });

  // Breaks if the solved count, streak, weak concept or recent run is missing or shows the wrong words.
  it('shows solved counts, streak, weak concepts and recent runs in words', async () => {
    render(DashboardScreen, { props: { load: async () => populatedView } });

    expect(await screen.findByText('Hash maps')).toBeTruthy();
    expect(screen.getByText('2 failed attempts')).toBeTruthy();
    expect(screen.getByText(/2-day streak/)).toBeTruthy();
    expect(screen.getByText('Wrong answer')).toBeTruthy();
    expect(screen.getByText('Queued')).toBeTruthy();
  });

  // Breaks if a suggestion loses its link, its reason, or shows an invented verdict on the active run.
  it('links each suggestion to its problem and labels why it was suggested', async () => {
    render(DashboardScreen, { props: { load: async () => populatedView } });

    const link = await screen.findByRole('link', { name: /Pairs Four/ });
    expect(link.getAttribute('href')).toBe('/problems/pairs-four');
    expect(screen.getByText('Practise a concept you missed')).toBeTruthy();
    expect(screen.queryByText('Accepted')).toBeNull();
  });

  // Breaks if a response the page cannot read shows an empty dashboard instead of the error state with retry.
  it('shows the error state, not an empty dashboard, when the response cannot be read', async () => {
    const malformed = new ApiRequestError(
      200,
      'malformed',
      'The server sent a response this page cannot read.'
    );
    render(DashboardScreen, {
      props: { load: () => Promise.reject(malformed) }
    });

    expect(
      await screen.findByText(
        'The server sent a response this page cannot read.'
      )
    ).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy();
    expect(screen.queryByText('No practice yet')).toBeNull();
  });
});
