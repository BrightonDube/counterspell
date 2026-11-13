// Tests for the ghost payload contract (ide-workspace T01, ghost-engine provenance). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ContractError } from '../contract';
import { parseGhost, parseShrinkStep } from './types';

const ghost = {
  kind: 'wrong',
  input: [0, 0],
  expected: [0, 1],
  got: [0, 0],
  explanation: 'You returned [0, 0]; expected [0, 1] for [0, 0].',
  budget_exhausted: false,
  shrink: {
    steps: [{ index: 1, input: [3, 3, 0], evaluations: 14 }],
    total_steps: 1,
    omitted_steps: 0,
    stop: 'no_smaller_failure'
  },
  provenance: {
    seed: 42,
    generator_version: 'gen-1',
    engine_version: 'engine-1',
    problem_version: 2,
    image_digest: 'sha256:abc'
  },
  autopsy: {
    explanation: 'The inner loop reuses the same index.',
    differences: [{ path: '[1]', expected: 1, got: 0 }],
    differences_truncated: false,
    concept_tags: ['hash-maps'],
    authored_mistake: null
  }
};

describe('parseGhost', () => {
  // Breaks if a ghost without its seed and versions is shown, since the learner could not reproduce it.
  it('rejects a ghost without provenance', () => {
    const { provenance: _omitted, ...withoutProvenance } = ghost;
    expect(() => parseGhost(withoutProvenance)).toThrow(ContractError);
  });

  // Breaks if an unknown failure kind is rendered as a real failure class.
  it('rejects a failure kind outside the engine set', () => {
    expect(() => parseGhost({ ...ghost, kind: 'maybe' })).toThrow(
      ContractError
    );
  });

  // Breaks if the shrink trace is accepted with a step count that disagrees with the steps shown.
  it('keeps the shrink trace and its omission count', () => {
    const parsed = parseGhost({
      ...ghost,
      shrink: { ...ghost.shrink, total_steps: 25, omitted_steps: 5 }
    });
    expect(parsed.shrink.totalSteps).toBe(25);
    expect(parsed.shrink.omittedSteps).toBe(5);
    expect(parsed.shrink.steps).toHaveLength(1);
  });

  // Breaks if a complete, valid ghost loses its expected or actual values on the way to the view model.
  it('maps a complete ghost into the view model', () => {
    const parsed = parseGhost(ghost);
    expect(parsed.kind).toBe('wrong');
    expect(parsed.expected).toEqual([0, 1]);
    expect(parsed.got).toEqual([0, 0]);
    expect(parsed.provenance.seed).toBe(42);
    expect(parsed.autopsy.authoredMistake).toBeNull();
  });

  // Breaks if an oversized ghost explanation reaches the card without the display cap.
  it('rejects an explanation over the display cap', () => {
    expect(() =>
      parseGhost({ ...ghost, explanation: 'x'.repeat(64 * 1024 + 1) })
    ).toThrow(ContractError);
  });
});

describe('parseShrinkStep', () => {
  // Breaks if a shrink step with no recorded input is accepted, which would let the UI invent a value.
  it('rejects a shrink step without an input', () => {
    expect(() => parseShrinkStep({ index: 1, evaluations: 3 })).toThrow(
      ContractError
    );
  });
});
