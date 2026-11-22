// Tests for the dashboard response parser (dashboard-fsrs T09, DASH-1 and DASH-5 to DASH-9).
// Each test names the break it catches. No test calls the network.
import { describe, expect, it } from 'vitest';
import { ContractError } from '../contract';
import { parseDashboard } from './types';

const emptyDashboard = {
  solved: { total: 0, by_difficulty: { easy: 0, medium: 0, hard: 0 } },
  streak: { current_days: 0, active_today: false, last_activity_date: null },
  weak_concepts: [],
  recent_submissions: [],
  suggested_challenges: []
};

const populatedDashboard = {
  solved: { total: 2, by_difficulty: { easy: 1, medium: 1, hard: 0 } },
  streak: {
    current_days: 3,
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
};

describe('parseDashboard', () => {
  // Breaks if an empty learner's response is rejected, so a new account would see an error page.
  it('accepts the empty shape and keeps the zero counts', () => {
    const view = parseDashboard(emptyDashboard);
    expect(view.solved).toEqual({
      total: 0,
      byDifficulty: { easy: 0, medium: 0, hard: 0 }
    });
    expect(view.streak).toEqual({
      currentDays: 0,
      activeToday: false,
      lastActivityDate: null
    });
    expect(view.recentSubmissions).toEqual([]);
    expect(view.suggestedChallenges).toEqual([]);
  });

  // Breaks if snake_case JSON is not mapped to the camelCase view model the components read.
  it('maps a populated response to the camelCase view model', () => {
    const view = parseDashboard(populatedDashboard);
    expect(view.solved.byDifficulty.medium).toBe(1);
    expect(view.streak).toEqual({
      currentDays: 3,
      activeToday: true,
      lastActivityDate: '2026-11-15'
    });
    expect(view.weakConcepts[0]).toEqual({
      slug: 'hash-maps',
      name: 'Hash maps',
      failureCount: 2,
      lastFailedAt: '2026-11-14T18:20:00Z'
    });
    expect(view.suggestedChallenges[0]?.reason).toBe('practice_weak_concept');
  });

  // Breaks if an active run shows an invented verdict or a finish time it does not have.
  it('keeps a queued run without a verdict or finish time', () => {
    const view = parseDashboard(populatedDashboard);
    const queued = view.recentSubmissions[1];
    expect(queued?.status).toBe('queued');
    expect(queued?.verdict).toBeNull();
    expect(queued?.finishedAt).toBeNull();
    expect(queued?.passed).toBeNull();
  });

  // Breaks if a difficulty the catalogue does not use is shown as if it were a real level.
  it('rejects an unknown difficulty instead of guessing', () => {
    const broken = structuredClone(populatedDashboard);
    broken.suggested_challenges[0].difficulty = 'expert';
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if the page renders more than the ten recent runs the server promises to send.
  it('rejects more than ten recent runs', () => {
    const run = populatedDashboard.recent_submissions[0];
    const broken: Record<string, unknown> = {
      ...structuredClone(emptyDashboard),
      recent_submissions: Array.from({ length: 11 }, () => structuredClone(run))
    };
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if a missing section is treated as zero, which would show fabricated progress.
  it('rejects a response that is missing a section', () => {
    const { streak: _streak, ...withoutStreak } = populatedDashboard;
    expect(() => parseDashboard(withoutStreak)).toThrow(ContractError);
  });

  // Breaks if the server's weak list is not capped at the documented ten, so the page could render any length.
  it('rejects more than ten weak concepts', () => {
    const concept = populatedDashboard.weak_concepts[0];
    const broken: Record<string, unknown> = {
      ...structuredClone(populatedDashboard),
      weak_concepts: Array.from({ length: 11 }, (_, index) => ({
        ...concept,
        slug: `concept-${index}`
      }))
    };
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if two rows share a key. Svelte cannot render duplicate keys, so the page would go blank.
  it('rejects a weak concept list with a repeated slug', () => {
    const concept = populatedDashboard.weak_concepts[0];
    const broken: Record<string, unknown> = {
      ...structuredClone(populatedDashboard),
      weak_concepts: [concept, { ...concept }]
    };
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if a timestamp in another format is shown as if it were a date the server wrote.
  it('rejects a timestamp that is not UTC RFC 3339 text', () => {
    const broken = structuredClone(populatedDashboard);
    broken.recent_submissions[0].created_at = 'yesterday at 8am';
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if an unknown lifecycle status is shown as a valid state instead of revealing contract drift.
  it('rejects a run status the lifecycle does not define', () => {
    const broken = structuredClone(populatedDashboard);
    broken.recent_submissions[0].status = 'half_done';
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if an active learner with no streak days is shown "no streak", which contradicts DASH-3.4.
  it('rejects a streak that says the learner is active today but has no streak days', () => {
    const broken = structuredClone(populatedDashboard);
    broken.streak = {
      current_days: 0,
      active_today: true,
      last_activity_date: '2026-11-15'
    };
    expect(() => parseDashboard(broken)).toThrow(ContractError);
  });

  // Breaks if the additive review summary is dropped or its fields are guessed.
  it('maps a due review summary and keeps a missing one empty', () => {
    const withReview = {
      ...structuredClone(populatedDashboard),
      review_summary: {
        due_count: 2,
        next_review: {
          concept: { slug: 'hash-maps', name: 'Hash maps' },
          card: { state: 'review', due: '2026-11-15T09:00:00Z', version: 4 },
          challenge: {
            kind: 'sibling',
            slug: 'pairs-four',
            title: 'Pairs Four',
            difficulty: 'hard'
          }
        }
      }
    };
    const view = parseDashboard(withReview);
    expect(view.reviewSummary?.dueCount).toBe(2);
    expect(view.reviewSummary?.nextReview.card).toEqual({
      state: 'review',
      due: '2026-11-15T09:00:00Z',
      version: 4
    });
    expect(view.reviewSummary?.nextReview.challenge?.kind).toBe('sibling');

    expect(parseDashboard(emptyDashboard).reviewSummary).toBeNull();
    const nullSummary = { ...emptyDashboard, review_summary: null };
    expect(parseDashboard(nullSummary).reviewSummary).toBeNull();
  });

  // Breaks if a summary with zero due cards is fabricated into a review, or a bad card state passes.
  it('rejects a review summary with no due cards or an unknown card state', () => {
    const zero = {
      ...structuredClone(populatedDashboard),
      review_summary: {
        due_count: 0,
        next_review: {
          concept: { slug: 'hash-maps', name: 'Hash maps' },
          card: { state: 'review', due: '2026-11-15T09:00:00Z', version: 1 },
          challenge: null
        }
      }
    };
    expect(() => parseDashboard(zero)).toThrow(ContractError);

    const badState = structuredClone(zero);
    badState.review_summary.due_count = 1;
    badState.review_summary.next_review.card.state = 'suspended';
    expect(() => parseDashboard(badState)).toThrow(ContractError);
  });
});
