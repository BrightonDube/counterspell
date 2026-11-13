// Component tests for the ghost hunting state and the ghost card (ide-workspace T15).
// Each test names the break it catches.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import type { Json } from '../contract';
import type { GhostView } from '../ghost/types';
import GhostPanel from './GhostPanel.svelte';

afterEach(() => {
  cleanup();
});

/** A wrong-answer ghost with three recorded shrink steps that end at the ghost input. */
function wrongGhost(overrides: Partial<GhostView> = {}): GhostView {
  return {
    kind: 'wrong',
    input: [0, 0] as Json,
    expected: 1,
    got: 0,
    explanation: 'You returned 0; expected 1 for [0, 0].',
    budgetExhausted: false,
    shrink: {
      steps: [
        { index: 0, input: [3, 5, 0], evaluations: 1 },
        { index: 1, input: [0, 0, 1], evaluations: 4 },
        { index: 2, input: [0, 0], evaluations: 9 }
      ],
      totalSteps: 3,
      omittedSteps: 0,
      stop: 'no_smaller_input'
    },
    provenance: {
      seed: 7,
      generatorVersion: 'gen-1',
      engineVersion: 'eng-1',
      problemVersion: 2,
      imageDigest: 'sha256:abc'
    },
    autopsy: {
      explanation: 'You returned 0; expected 1 for [0, 0].',
      differences: [],
      differencesTruncated: false,
      conceptTags: [],
      authoredMistake: null
    },
    ...overrides
  };
}

describe('GhostPanel hunting state', () => {
  // Breaks if the search shows a percentage, which would invent progress the engine never reported.
  it('shows honest search text with no percentage while the server is hunting', () => {
    const { container } = render(GhostPanel, {
      props: {
        status: 'ghost_hunting',
        ghost: null,
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(screen.getByText(/Searching for a failing input/)).toBeTruthy();
    expect(container.textContent).not.toContain('%');
  });

  // Breaks if the panel shows a hunt on its own, before the server reports ghost_hunting.
  it('does not show a hunt while the run is still running', () => {
    render(GhostPanel, {
      props: {
        status: 'running',
        ghost: null,
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(screen.queryByText(/Searching for a failing input/)).toBeNull();
  });
});

describe('GhostPanel ghost card', () => {
  // Breaks if a wrong-answer ghost loses its input, expected or got values.
  it('shows the input, expected and got values as text', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: wrongGhost(),
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(screen.getByText('[0,0]', { selector: 'code' })).toBeTruthy();
    expect(screen.getByText('1', { selector: 'code' })).toBeTruthy();
    expect(screen.getByText('0', { selector: 'code' })).toBeTruthy();
  });

  // Breaks if a learner reads the ghost as a proven minimum rather than the smallest found.
  it('calls a wrong-answer ghost a minimized counterexample, not a proven minimum', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: wrongGhost(),
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(screen.getByText(/Minimized counterexample/)).toBeTruthy();
    expect(screen.getByText(/not a proven minimum/)).toBeTruthy();
  });

  // Breaks if a timed-out ghost is called minimized, which the engine never confirmed.
  it('does not call a timeout ghost minimized', () => {
    const { container } = render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: wrongGhost({
          kind: 'timeout',
          expected: undefined,
          got: undefined,
          budgetExhausted: true
        }),
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(container.textContent).not.toContain('Minimized counterexample');
    expect(screen.getByText(/Timed out/)).toBeTruthy();
  });

  // Breaks if the replay shows a step the engine never recorded, or drops one it did record.
  it('lists every recorded shrink step in order', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: wrongGhost(),
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    const items = screen.getAllByRole('listitem');
    expect(
      items.map((item) => item.textContent?.replace(/\s+/g, ' ').trim())
    ).toEqual(['Step 1: [3,5,0]', 'Step 2: [0,0,1]', 'Step 3: [0,0]']);
  });

  // Breaks if the replay runs with a timed animation under reduced motion, or skips the static list.
  it('marks the replay static under reduced motion', () => {
    const { container } = render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: wrongGhost(),
        shrinkSteps: [],
        reducedMotion: true
      }
    });
    expect(container.querySelector('[data-motion="static"]')).not.toBeNull();
    expect(screen.getAllByRole('listitem')).toHaveLength(3);
  });

  // Breaks if a trace that does not end at the ghost input is played as if it were the real search.
  it('does not replay steps that disagree with the ghost input', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: wrongGhost({ input: [9, 9] as Json }),
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(screen.queryAllByRole('listitem')).toHaveLength(0);
    expect(screen.getByText(/do not end at this input/)).toBeTruthy();
  });

  // Breaks if a search that was cut off (aborted, expired or internal error) reads as "no ghost exists".
  it('does not claim the search finished when the run ended without one', () => {
    render(GhostPanel, {
      props: {
        status: 'internal_error',
        ghost: null,
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(
      screen.getByText(/The ghost search did not finish/)
    ).toBeTruthy();
    expect(
      screen.queryByText(/No ghost input was found within the search budget/)
    ).toBeNull();
  });

  // Breaks if a ghost payload that failed validation is silently dropped instead of saying so.
  it('says the ghost data could not be verified when the payload is malformed', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: null,
        ghostError: 'malformed',
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(
      screen.getByText(/could not be verified, so nothing is shown/)
    ).toBeTruthy();
    expect(screen.queryAllByRole('listitem')).toHaveLength(0);
  });

  // Breaks if a finished run with no ghost is left blank instead of saying so.
  it('says when no ghost was found within the search budget', () => {
    render(GhostPanel, {
      props: {
        status: 'completed',
        ghost: null,
        shrinkSteps: [],
        reducedMotion: false
      }
    });
    expect(
      screen.getByText(/No ghost input was found within the search budget/)
    ).toBeTruthy();
  });
});
