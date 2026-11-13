// Tests for the recorded shrink playback controller (ide-workspace T14). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import {
  planPlayback,
  PLAYBACK_BUDGET_MS,
  type PlaybackPlan
} from './shrink-playback';
import type { ShrinkTraceView } from './types';

function trace(
  steps: { index: number; input: unknown }[],
  extra: Partial<ShrinkTraceView> = {}
): ShrinkTraceView {
  return {
    steps: steps.map((step) => ({
      index: step.index,
      input: step.input as never,
      evaluations: 1
    })),
    totalSteps: steps.length,
    omittedSteps: 0,
    stop: 'no_smaller_failure',
    ...extra
  };
}

describe('planPlayback step order', () => {
  // Breaks if the playback shows values in a different order from the recorded trace.
  it('keeps the received steps in their recorded order', () => {
    const plan = planPlayback(
      trace([
        { index: 1, input: [3, 3, 0] },
        { index: 2, input: [2, 2] },
        { index: 3, input: [0, 0] }
      ]),
      [0, 0],
      false
    );
    expect(plan.steps.map((step) => step.input)).toEqual([
      [3, 3, 0],
      [2, 2],
      [0, 0]
    ]);
  });

  // Breaks if a replayed duplicate step is shown twice, which would look like an extra shrink.
  it('drops a duplicate step and counts it as skipped', () => {
    const plan = planPlayback(
      trace([
        { index: 1, input: [3, 3, 0] },
        { index: 1, input: [3, 3, 0] },
        { index: 2, input: [0, 0] }
      ]),
      [0, 0],
      false
    );
    expect(plan.steps.map((step) => step.index)).toEqual([1, 2]);
    expect(plan.skipped).toBe(1);
  });

  // Breaks if a late, lower-index step is played after a higher one and the animation goes backwards.
  it('drops an out-of-order step instead of playing it', () => {
    const plan = planPlayback(
      trace([
        { index: 1, input: [3, 3, 0] },
        { index: 3, input: [0, 0] },
        { index: 2, input: [2, 2] }
      ]),
      [0, 0],
      false
    );
    expect(plan.steps.map((step) => step.index)).toEqual([1, 3]);
    expect(plan.skipped).toBe(1);
  });
});

describe('planPlayback final agreement', () => {
  // Breaks if a trace whose last step is not the ghost input is animated as if it were the counterexample.
  it('marks the trace as disagreeing when the last step differs from the ghost input', () => {
    const plan: PlaybackPlan = planPlayback(
      trace([{ index: 1, input: [3, 3, 0] }]),
      [0, 0],
      false
    );
    expect(plan.trust).toBe('disagrees');
    expect(plan.steps).toEqual([]);
  });

  // Breaks if key order in an object input makes equal values look different and hides a valid trace.
  it('treats objects with the same keys in another order as equal', () => {
    const plan = planPlayback(
      trace([{ index: 1, input: { a: 1, b: 2 } }]),
      { b: 2, a: 1 },
      false
    );
    expect(plan.trust).toBe('agrees');
  });

  // Breaks if an empty trace is shown as a verified minimized counterexample.
  it('reports no steps when the trace is empty', () => {
    const plan = planPlayback(trace([]), [0, 0], false);
    expect(plan.trust).toBe('no_steps');
    expect(plan.steps).toEqual([]);
  });
});

describe('planPlayback timing and omissions', () => {
  // Breaks if a long trace plays longer than the display budget, so the learner waits too long.
  it('keeps the total playback time within the budget', () => {
    const steps = Array.from({ length: 30 }, (_, i) => ({
      index: i + 1,
      input: [30 - i]
    }));
    const plan = planPlayback(trace(steps), [1], false);
    expect(plan.stepDelayMs).toBe(100);
    expect(plan.stepDelayMs * plan.steps.length).toBeLessThanOrEqual(
      PLAYBACK_BUDGET_MS
    );
  });

  // Breaks if a short trace is stretched past the per-step crossfade maximum of 250 ms.
  it('caps each step at 250 ms for a short trace', () => {
    const plan = planPlayback(
      trace([{ index: 1, input: [0, 0] }]),
      [0, 0],
      false
    );
    expect(plan.stepDelayMs).toBe(250);
  });

  // Breaks if the server's omitted-step count is lost, so the UI pretends the trace was complete.
  it('keeps the omitted-step count from the server', () => {
    const plan = planPlayback(
      trace([{ index: 1, input: [0, 0] }], {
        totalSteps: 25,
        omittedSteps: 24
      }),
      [0, 0],
      false
    );
    expect(plan.omittedSteps).toBe(24);
    expect(plan.totalSteps).toBe(25);
  });
});

describe('planPlayback reduced motion', () => {
  // Breaks if reduced motion removes steps, which would lose trace information the learner needs.
  it('shows the same steps as a static list with no delay', () => {
    const plan = planPlayback(
      trace([
        { index: 1, input: [3, 3, 0] },
        { index: 2, input: [0, 0] }
      ]),
      [0, 0],
      true
    );
    expect(plan.mode).toBe('static');
    expect(plan.stepDelayMs).toBe(0);
    expect(plan.steps.map((step) => step.input)).toEqual([
      [3, 3, 0],
      [0, 0]
    ]);
  });
});
