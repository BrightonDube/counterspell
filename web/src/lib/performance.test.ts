// Tests for the workspace timing marks (ide-workspace T27, R15.5). Each test names the break it catches.
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  createRunMarks,
  markWorkspace,
  playbackDurationMs,
  scheduleGhostMarks,
  WORKSPACE_MARKS
} from './performance';

function recorder() {
  const calls: unknown[][] = [];
  return {
    calls,
    perf: {
      mark: (...args: unknown[]) => {
        calls.push(args);
      }
    }
  };
}

afterEach(() => {
  vi.useRealTimers();
});

describe('markWorkspace', () => {
  // Breaks if a mark carries a detail object, which is where a source or user id could leak (R15.5).
  it('records the fixed mark name and nothing else', () => {
    const { calls, perf } = recorder();
    markWorkspace('monaco-ready', perf);
    expect(calls).toEqual([['monaco-ready']]);
  });

  // Breaks if a browser without the Performance API crashes the workspace.
  it('does nothing when there is no performance object', () => {
    expect(() => markWorkspace('run-terminal', undefined)).not.toThrow();
  });

  // Breaks if the documented mark list drifts from what the browser check reads.
  it('offers exactly the documented marks', () => {
    expect([...WORKSPACE_MARKS]).toEqual([
      'workspace-shell-interactive',
      'problem-detail-loaded',
      'monaco-ready',
      'run-first-event',
      'run-terminal',
      'ghost-playback-start',
      'ghost-playback-end'
    ]);
  });
});

describe('createRunMarks', () => {
  // Breaks if every streamed event adds a mark, so "first event" stops meaning first.
  it('marks the first event and the terminal once per run', () => {
    const { calls, perf } = recorder();
    const marks = createRunMarks(perf);
    marks.event('run-1');
    marks.event('run-1');
    marks.terminal('run-1');
    marks.terminal('run-1');
    expect(calls).toEqual([['run-first-event'], ['run-terminal']]);
  });

  // Breaks if a second run on the same page is never measured.
  it('marks a new run again', () => {
    const { calls, perf } = recorder();
    const marks = createRunMarks(perf);
    marks.event('run-1');
    marks.terminal('run-1');
    marks.event('run-2');
    marks.terminal('run-2');
    expect(calls).toEqual([
      ['run-first-event'],
      ['run-terminal'],
      ['run-first-event'],
      ['run-terminal']
    ]);
  });

  // Breaks if the run id ends up in a mark name.
  it('never puts the run id in a mark', () => {
    const { calls, perf } = recorder();
    const marks = createRunMarks(perf);
    marks.event('run-secret-7f3a');
    marks.terminal('run-secret-7f3a');
    expect(JSON.stringify(calls)).not.toContain('7f3a');
  });
});

describe('playbackDurationMs', () => {
  // Breaks if the end mark under reduced motion waits for an animation that never plays.
  it('is zero for a static plan', () => {
    expect(
      playbackDurationMs({ mode: 'static', stepCount: 3, stepDelayMs: 0 })
    ).toBe(0);
  });

  // Breaks if the measured reveal is shorter than the steps actually played.
  it('is the step count times the step delay for an animated plan', () => {
    expect(
      playbackDurationMs({ mode: 'animated', stepCount: 3, stepDelayMs: 250 })
    ).toBe(750);
  });
});

describe('scheduleGhostMarks', () => {
  // Breaks if the end mark fires before the replay could have finished.
  it('marks the start now and the end after the playback duration', () => {
    vi.useFakeTimers();
    const { calls, perf } = recorder();
    scheduleGhostMarks(750, perf);
    expect(calls).toEqual([['ghost-playback-start']]);
    vi.advanceTimersByTime(749);
    expect(calls).toEqual([['ghost-playback-start']]);
    vi.advanceTimersByTime(1);
    expect(calls).toEqual([['ghost-playback-start'], ['ghost-playback-end']]);
  });

  // Breaks if leaving the page during a replay still writes a mark from a dead component.
  it('writes no end mark once cancelled', () => {
    vi.useFakeTimers();
    const { calls, perf } = recorder();
    const cancel = scheduleGhostMarks(750, perf);
    cancel();
    vi.advanceTimersByTime(1000);
    expect(calls).toEqual([['ghost-playback-start']]);
  });

  // Breaks if a static replay leaves the end mark waiting on a timer.
  it('marks start and end together for a zero duration', () => {
    vi.useFakeTimers();
    const { calls, perf } = recorder();
    scheduleGhostMarks(0, perf);
    expect(calls).toEqual([['ghost-playback-start'], ['ghost-playback-end']]);
  });
});
