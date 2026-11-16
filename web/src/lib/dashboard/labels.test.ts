// Tests for the words the dashboard shows for run states and suggestion reasons (dashboard-fsrs T09, DASH-5
// and DASH-6). Status is always text, never colour alone. Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { reasonLabel, runStatusLabel } from './labels';

describe('runStatusLabel', () => {
  // Breaks if a finished run without a verdict shows as finished when it is still active.
  it('names an active run by its lifecycle status and never invents a verdict', () => {
    expect(runStatusLabel('queued', null)).toBe('Queued');
    expect(runStatusLabel('running', null)).toBe('Running');
  });

  // Breaks if a wrong answer is displayed as a success or with a generic word.
  it('shows the user-code verdict in words for a completed run', () => {
    expect(runStatusLabel('completed', 'accepted')).toBe('Accepted');
    expect(runStatusLabel('completed', 'wrong_answer')).toBe('Wrong answer');
    expect(runStatusLabel('completed', 'time_limit')).toBe('Time limit');
  });

  // Breaks if an infrastructure failure is shown as the learner's own wrong answer.
  it('labels an internal error as not the learner answer', () => {
    expect(runStatusLabel('internal_error', null)).toBe(
      'Internal error (not a wrong answer)'
    );
  });
});

describe('reasonLabel', () => {
  // Breaks if a review-style suggestion is labelled as a normal track continuation, or the reverse.
  it('describes each suggestion reason in plain words', () => {
    expect(reasonLabel('practice_weak_concept')).toBe(
      'Practise a concept you missed'
    );
    expect(reasonLabel('continue_track')).toBe('Continue your track');
  });
});
