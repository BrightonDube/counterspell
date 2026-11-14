import { describe, expect, it } from 'vitest';
import { applyEvent, initialRunState } from './reducer';
import type { RunEventEnvelope, RunEventPayload } from './types';

function envelope(
  seq: number,
  payload: RunEventPayload,
  attempt = 1
): RunEventEnvelope {
  const event = payload.kind === 'test' ? 'test' : payload.kind;
  return { seq, attempt, event, payload } as RunEventEnvelope;
}

const sampleRow = (ordinal: number, verdict: 'pass' | 'wrong_answer') => ({
  sample: true as const,
  ordinal,
  verdict,
  timeMs: 4,
  input: '[1, 2]',
  expected: '3',
  got: verdict === 'pass' ? '3' : '2'
});

const hiddenRow = (ordinal: number, verdict: 'pass' | 'time_limit') => ({
  sample: false as const,
  ordinal,
  verdict,
  timeMs: 12
});

describe('applyEvent', () => {
  it('returns the same state object for a duplicate sequence number', () => {
    const first = applyEvent(
      initialRunState('run-1'),
      envelope(1, { kind: 'compiling' })
    );
    const again = applyEvent(first, envelope(1, { kind: 'compiling' }));
    expect(again).toBe(first);
  });

  it('marks a gap and does not apply the event that skipped ahead', () => {
    const state = applyEvent(
      initialRunState('run-1'),
      envelope(2, { kind: 'compiling' })
    );
    expect(state.gap).toBe(true);
    expect(state.lastSeq).toBe(0);
    expect(state.status).toBe('queued');
  });

  it('replaces a test row that repeats an ordinal and keeps rows sorted', () => {
    let state = initialRunState('run-1');
    state = applyEvent(
      state,
      envelope(1, { kind: 'test', test: hiddenRow(2, 'pass') })
    );
    state = applyEvent(
      state,
      envelope(2, { kind: 'test', test: sampleRow(1, 'pass') })
    );
    state = applyEvent(
      state,
      envelope(3, { kind: 'test', test: hiddenRow(2, 'time_limit') })
    );
    expect(state.tests).toEqual([
      sampleRow(1, 'pass'),
      hiddenRow(2, 'time_limit')
    ]);
    expect(state.status).toBe('running');
    expect(state.lastSeq).toBe(3);
  });

  it('restarted clears per-attempt rows and steps but keeps the sequence', () => {
    let state = initialRunState('run-1');
    state = applyEvent(
      state,
      envelope(1, { kind: 'test', test: sampleRow(1, 'pass') })
    );
    state = applyEvent(
      state,
      envelope(2, {
        kind: 'shrink_step',
        step: { index: 0, input: [5], evaluations: 1 }
      })
    );
    state = applyEvent(
      state,
      envelope(3, { kind: 'restarted', attempt: 2 }, 2)
    );
    expect(state.tests).toEqual([]);
    expect(state.shrinkSteps).toEqual([]);
    expect(state.attempt).toBe(2);
    expect(state.lastSeq).toBe(3);
    expect(state.status).toBe('queued');
  });

  it('records a passing done as the terminal result and then ignores later events', () => {
    const done = applyEvent(
      initialRunState('run-1'),
      envelope(1, {
        kind: 'done',
        verdict: 'accepted',
        passed: 12,
        total: 12,
        score: 100
      })
    );
    expect(done.terminal).toEqual({
      kind: 'done',
      verdict: 'accepted',
      passed: 12,
      total: 12,
      score: 100
    });
    expect(done.status).toBe('completed');
    const after = applyEvent(done, envelope(2, { kind: 'compiling' }));
    expect(after).toBe(done);
  });

  it('maps an internal_error to the internal_error status, not a wrong answer', () => {
    const state = applyEvent(
      initialRunState('run-1'),
      envelope(1, {
        kind: 'error',
        code: 'internal_error',
        message: 'The judge failed.'
      })
    );
    expect(state.status).toBe('internal_error');
    expect(state.terminal).toEqual({
      kind: 'error',
      code: 'internal_error',
      message: 'The judge failed.'
    });
  });

  it('gives the same state for a noisy replay as for the log applied once', () => {
    const log: RunEventEnvelope[] = [
      envelope(1, { kind: 'compiling' }),
      envelope(2, { kind: 'test', test: sampleRow(1, 'wrong_answer') }),
      envelope(3, { kind: 'test', test: hiddenRow(2, 'pass') }),
      envelope(4, { kind: 'restarted', attempt: 2 }, 2),
      envelope(5, { kind: 'test', test: sampleRow(1, 'pass') }, 2),
      envelope(6, { kind: 'ghost_hunting' }, 2),
      envelope(
        7,
        { kind: 'done', verdict: 'accepted', passed: 2, total: 2, score: 100 },
        2
      )
    ];
    const applyAll = (events: RunEventEnvelope[]) =>
      events.reduce(applyEvent, initialRunState('run-1'));
    const once = applyAll(log);

    // Seeded linear congruential generator: the same stream every run.
    let seed = 20261113;
    const next = () => {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      return seed / 2 ** 32;
    };
    // After each real event, sometimes replay a random earlier one, and sometimes repeat the event itself.
    const noisy: RunEventEnvelope[] = [];
    log.forEach((event, index) => {
      noisy.push(event);
      if (next() < 0.6) noisy.push(log[Math.floor(next() * (index + 1))]);
      if (next() < 0.5) noisy.push(event);
    });

    const replayed = applyAll(noisy);
    expect(replayed).toEqual(once);
    expect(replayed.gap).toBe(false);
  });

  // P10: 200 fixed seeds, each with random duplicates and replays of earlier events. Every seed must
  // end in the same state as the log applied once.
  it('matches the single-apply state for 200 seeded noisy replays (P10)', () => {
    const log: RunEventEnvelope[] = [
      envelope(1, { kind: 'compiling' }),
      envelope(2, { kind: 'test', test: sampleRow(1, 'pass') }),
      envelope(3, { kind: 'restarted', attempt: 2 }, 2),
      envelope(4, { kind: 'test', test: hiddenRow(1, 'time_limit') }, 2),
      envelope(
        5,
        {
          kind: 'done',
          verdict: 'wrong_answer',
          passed: 0,
          total: 1,
          score: 0
        },
        2
      )
    ];
    const applyAll = (events: RunEventEnvelope[]) =>
      events.reduce(applyEvent, initialRunState('run-1'));
    const once = applyAll(log);

    for (let seedValue = 1; seedValue <= 200; seedValue += 1) {
      let seed = seedValue;
      const next = () => {
        seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
        return seed / 2 ** 32;
      };
      const noisy: RunEventEnvelope[] = [];
      log.forEach((event, index) => {
        noisy.push(event);
        const extra = Math.floor(next() * 3);
        for (let copy = 0; copy < extra; copy += 1) {
          noisy.push(log[Math.floor(next() * (index + 1))]);
        }
      });
      expect(applyAll(noisy), `seed ${seedValue}`).toEqual(once);
    }
  });
});
