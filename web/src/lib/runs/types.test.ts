// Tests for the public run and SSE contracts (ide-workspace T01, api-conventions.md). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ContractError } from '../contract';
import { parseEventEnvelope, parsePublicTest, parseRunView } from './types';

const hidden = {
  ordinal: 3,
  verdict: 'wrong_answer',
  time_ms: 12,
  sample: false
};
const sample = {
  ordinal: 1,
  verdict: 'pass',
  time_ms: 4,
  sample: true,
  input: '[2, 7]',
  expected: '[0, 1]',
  got: '[0, 1]'
};

describe('parsePublicTest', () => {
  // Breaks if a hidden row leaks input or expected output into the view model.
  it('rejects a hidden test that carries an input', () => {
    expect(() => parsePublicTest({ ...hidden, input: '[1, 2]' })).toThrow(
      ContractError
    );
  });

  // Breaks if a hidden row carrying expected output is accepted as a normal public row.
  it('rejects a hidden test that carries expected output', () => {
    expect(() => parsePublicTest({ ...hidden, expected: '3' })).toThrow(
      ContractError
    );
  });

  // Breaks if hidden stdout or stderr is accepted, since the public surface must not expose them.
  it('rejects a hidden test that carries captured stdout', () => {
    expect(() => parsePublicTest({ ...hidden, stdout: 'secret' })).toThrow(
      ContractError
    );
  });

  // Breaks if a valid hidden row loses its ordinal, verdict or time on the way in.
  it('accepts a hidden test with only ordinal, verdict and time', () => {
    expect(parsePublicTest(hidden)).toEqual({
      sample: false,
      ordinal: 3,
      verdict: 'wrong_answer',
      timeMs: 12
    });
  });

  // Breaks if a sample row without its detail is accepted and shown as if the detail existed.
  it('rejects a sample test without its got value', () => {
    const { got: _omitted, ...withoutGot } = sample;
    expect(() => parsePublicTest(withoutGot)).toThrow(ContractError);
  });

  // Breaks if sample output larger than the display cap reaches the DOM unbounded.
  it('rejects sample output over the display cap', () => {
    expect(() =>
      parsePublicTest({ ...sample, got: 'x'.repeat(64 * 1024 + 1) })
    ).toThrow(ContractError);
  });
});

describe('parseEventEnvelope', () => {
  // Breaks if an event name outside the documented set is rendered as a real lifecycle step.
  it('rejects an event name outside the documented set', () => {
    expect(() =>
      parseEventEnvelope('judge_debug', '{"seq":1,"attempt":1,"payload":{}}')
    ).toThrow(ContractError);
  });

  // Breaks if malformed JSON from the stream reaches the reducer instead of being refused.
  it('rejects data that is not JSON', () => {
    expect(() => parseEventEnvelope('test', '{"seq":')).toThrow(ContractError);
  });

  // Breaks if a sequence number of zero or a fraction is accepted and corrupts ordering.
  it('rejects a sequence number that is not a positive integer', () => {
    expect(() =>
      parseEventEnvelope('compiling', '{"seq":0,"attempt":1,"payload":{}}')
    ).toThrow(ContractError);
  });

  // Breaks if a restarted event loses the attempt number the UI uses to clear transient rows.
  it('reads the attempt from a restarted event', () => {
    const parsed = parseEventEnvelope(
      'restarted',
      '{"seq":5,"attempt":2,"payload":{"attempt":2}}'
    );
    expect(parsed).toEqual({
      seq: 5,
      attempt: 2,
      event: 'restarted',
      payload: { kind: 'restarted', attempt: 2 }
    });
  });

  // Breaks if the terminal summary drops the verdict and score the UI shows as the authoritative result.
  it('reads the terminal done summary', () => {
    const parsed = parseEventEnvelope(
      'done',
      '{"seq":9,"attempt":1,"payload":{"verdict":"accepted","passed":5,"total":5,"score":100}}'
    );
    expect(parsed.payload).toEqual({
      kind: 'done',
      verdict: 'accepted',
      passed: 5,
      total: 5,
      score: 100
    });
  });
});

describe('parseRunView', () => {
  // Breaks if a run read model with an unknown lifecycle status is accepted as current state.
  it('rejects an unknown status', () => {
    expect(() =>
      parseRunView({
        run_id: 'r1',
        problem: 'two-sum',
        lang: 'python',
        mode: 'submit',
        status: 'floating',
        terminal: false,
        attempt: 1,
        last_seq: 0,
        created_at: '2026-11-13T10:00:00.000Z',
        tests: []
      })
    ).toThrow(ContractError);
  });

  // Breaks if a foreign hidden test row inside the read model is accepted without the same checks.
  it('applies the hidden test rules inside the read model', () => {
    expect(() =>
      parseRunView({
        run_id: 'r1',
        problem: 'two-sum',
        lang: 'python',
        mode: 'submit',
        status: 'completed',
        terminal: true,
        verdict: 'wrong_answer',
        attempt: 1,
        last_seq: 4,
        created_at: '2026-11-13T10:00:00.000Z',
        tests: [{ ...hidden, input: '[9]' }]
      })
    ).toThrow(ContractError);
  });
});
