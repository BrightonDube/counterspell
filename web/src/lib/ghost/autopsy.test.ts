// Tests for the Bug Autopsy view model (ide-workspace T16). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { buildAutopsy } from './autopsy';
import { parseGhost } from './types';

/** A raw ghost payload in the wire shape. Tests change one field at a time. */
function rawGhost(
  overrides: Record<string, unknown> = {}
): Record<string, unknown> {
  return {
    kind: 'wrong',
    input: [0, 0],
    expected: [0, 1],
    got: [0, 0],
    explanation: 'You returned [0, 0]; expected [0, 1] for [0, 0].',
    budget_exhausted: false,
    shrink: {
      steps: [],
      total_steps: 0,
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
    },
    ...overrides
  };
}

describe('buildAutopsy for a wrong answer', () => {
  // Breaks if the expected and actual values are not shown as their exact text.
  it('shows the failing input, expected output and actual output as text', () => {
    const result = buildAutopsy(parseGhost(rawGhost()));
    expect(result).toMatchObject({
      status: 'ready',
      view: {
        input: '[0,0]',
        expected: '[0,1]',
        actual: '[0,0]',
        actualLabel: 'Got'
      }
    });
  });

  // Breaks if an engine diff is recomputed or reordered in the browser and disagrees with the trusted data.
  it('passes the engine differences through unchanged', () => {
    const result = buildAutopsy(parseGhost(rawGhost()));
    if (result.status !== 'ready') throw new Error('expected a ready autopsy');
    expect(result.view.differences).toEqual([
      { path: '[1]', expected: '1', got: '0' }
    ]);
  });

  // Breaks if a capped difference list is shown as complete, which would hide that more differences exist.
  it('marks a truncated difference list', () => {
    const result = buildAutopsy(
      parseGhost(
        rawGhost({
          autopsy: {
            ...(rawGhost().autopsy as object),
            differences_truncated: true
          }
        })
      )
    );
    if (result.status !== 'ready') throw new Error('expected a ready autopsy');
    expect(result.view.differencesTruncated).toBe(true);
  });

  // Breaks if the card claims an authored explanation the pack never supplied.
  it('labels the conceptual mistake as authored only when one was received', () => {
    const withoutAuthored = buildAutopsy(parseGhost(rawGhost()));
    if (withoutAuthored.status !== 'ready')
      throw new Error('expected a ready autopsy');
    expect(withoutAuthored.view.explanationSource.label).toBeNull();

    const withAuthored = buildAutopsy(
      parseGhost(
        rawGhost({
          autopsy: {
            ...(rawGhost().autopsy as object),
            authored_mistake: {
              known_ghost_id: 'same-index',
              text: 'You reused index i.'
            }
          }
        })
      )
    );
    if (withAuthored.status !== 'ready')
      throw new Error('expected a ready autopsy');
    expect(withAuthored.view.explanationSource).toEqual({
      label: 'Authored explanation',
      text: 'You reused index i.'
    });
  });

  // Breaks if an incomplete wrong-answer payload renders guessed values instead of an unavailable state.
  it('reports unavailable when the expected value is missing', () => {
    const { expected: _omitted, ...incomplete } = rawGhost();
    const result = buildAutopsy(parseGhost(incomplete));
    expect(result.status).toBe('unavailable');
  });
});

describe('buildAutopsy for crashes and timeouts', () => {
  // Breaks if a timeout is presented as a minimized counterexample when the engine only found a slow input.
  it('does not call a timeout ghost minimized', () => {
    const result = buildAutopsy(
      parseGhost(
        rawGhost({ kind: 'timeout', expected: undefined, got: undefined })
      )
    );
    if (result.status !== 'ready') throw new Error('expected a ready autopsy');
    expect(result.view.headline).toBe('Slow input found');
    expect(result.view.actualLabel).toBe('Timed out');
  });

  // Breaks if a crash is labelled as a wrong answer, which would mislead the learner about the failure kind.
  it('labels a crash with its own actual label', () => {
    const result = buildAutopsy(
      parseGhost(
        rawGhost({ kind: 'crash', expected: undefined, got: undefined })
      )
    );
    if (result.status !== 'ready') throw new Error('expected a ready autopsy');
    expect(result.view.kindLabel).toBe('Crash');
    expect(result.view.actualLabel).toBe('Crash');
  });
});

describe('buildAutopsy hidden data', () => {
  // Breaks if an unexpected server field that carries hidden data reaches the card text.
  it('never shows a field the contract does not define', () => {
    const result = buildAutopsy(
      parseGhost(rawGhost({ hidden_expected: 'CANARY-7f3a-hidden' }))
    );
    expect(JSON.stringify(result)).not.toContain('CANARY-7f3a-hidden');
  });
});
